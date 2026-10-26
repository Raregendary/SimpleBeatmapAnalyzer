//! Persistent analysis cache.
//!
//! The cache is a single bincode file mapping `md5 + options + algorithm
//! version` to an [`Analysis`]. It contains only the computed statistics, never
//! parsed beatmaps, so a full osu! library stays small.
//!
//! A cache is optional: the library works without one, and the CLI only uses
//! one when `--cache <FILE>` is given.

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::options::ANALYZER_VERSION;
use crate::{Analysis, AnalyzeOptions, Error, Result};

const CACHE_VERSION: u32 = 1;

#[derive(Debug, Serialize, Deserialize)]
struct CacheFile {
    version: u32,
    entries: HashMap<String, Analysis>,
}

/// In-memory cache that can be persisted to disk.
#[derive(Debug)]
pub struct Cache {
    path: PathBuf,
    entries: HashMap<String, Analysis>,
    dirty: bool,
}

impl Cache {
    /// Creates an empty cache that will be written to `path` on
    /// [`Cache::save`].
    pub fn empty(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            entries: HashMap::new(),
            dirty: false,
        }
    }

    /// Loads a cache from disk.
    ///
    /// A missing file yields an empty cache. A corrupt or incompatible file
    /// yields [`Error::Cache`] so the caller can decide whether to ignore it;
    /// the CLI warns and starts with an empty cache in that case.
    pub fn open(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();

        if !path.is_file() {
            return Ok(Self::empty(path));
        }

        let file = File::open(&path).map_err(|err| Error::Cache {
            path: path.clone(),
            message: err.to_string(),
        })?;

        let decoded: CacheFile =
            bincode::deserialize_from(BufReader::new(file)).map_err(|err| Error::Cache {
                path: path.clone(),
                message: err.to_string(),
            })?;

        if decoded.version != CACHE_VERSION {
            return Err(Error::Cache {
                path,
                message: format!(
                    "unsupported cache version {} (expected {})",
                    decoded.version, CACHE_VERSION
                ),
            });
        }

        // An algorithm change makes older values unusable even if the cache
        // format itself did not change.
        let before = decoded.entries.len();
        let entries = decoded
            .entries
            .into_iter()
            .filter(|(_, analysis)| analysis.analyzer_version == ANALYZER_VERSION)
            .collect::<HashMap<_, _>>();
        let dirty = entries.len() != before;

        Ok(Self {
            path,
            entries,
            dirty,
        })
    }

    /// Builds the key under which an analysis is stored.
    pub fn key(md5: &str, options: &AnalyzeOptions) -> String {
        format!(
            "{}:{:016x}:{}",
            md5,
            options.fingerprint(),
            ANALYZER_VERSION
        )
    }

    /// Returns a cached analysis for the given content hash and options.
    pub fn get(&self, md5: &str, options: &AnalyzeOptions) -> Option<Analysis> {
        self.entries.get(&Self::key(md5, options)).cloned()
    }

    /// Inserts an analysis into the cache.
    pub fn insert(&mut self, analysis: &Analysis, options: &AnalyzeOptions) {
        self.entries
            .insert(Self::key(&analysis.md5, options), analysis.clone());
        self.dirty = true;
    }

    /// Number of entries currently held in memory.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// `true` when the cache holds no entries.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Path the cache is (or will be) stored at.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Writes the cache to disk with an atomic replace.
    ///
    /// Does nothing when no entry was inserted since the last save.
    pub fn save(&mut self) -> Result<()> {
        if !self.dirty {
            return Ok(());
        }

        if let Some(parent) = self.path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent).map_err(|err| Error::Cache {
                    path: self.path.clone(),
                    message: format!("failed to create {}: {}", parent.display(), err),
                })?;
            }
        }

        let temporary = self.path.with_extension("tmp");
        let file = File::create(&temporary).map_err(|err| Error::Cache {
            path: self.path.clone(),
            message: format!("failed to create {}: {}", temporary.display(), err),
        })?;
        let mut writer = BufWriter::new(file);
        let payload = CacheFile {
            version: CACHE_VERSION,
            entries: self.entries.clone(),
        };

        bincode::serialize_into(&mut writer, &payload).map_err(|err| Error::Cache {
            path: self.path.clone(),
            message: err.to_string(),
        })?;
        writer.flush().map_err(|err| Error::Cache {
            path: self.path.clone(),
            message: err.to_string(),
        })?;

        fs::rename(&temporary, &self.path).map_err(|err| Error::Cache {
            path: self.path.clone(),
            message: format!("failed to replace {}: {}", self.path.display(), err),
        })?;

        self.dirty = false;
        Ok(())
    }
}
