//! Batch scanning and parallel analysis.

use std::io;
use std::path::{Path, PathBuf};

use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::analysis::analyze_bytes_with_md5;
use crate::{md5_hex, Analysis, AnalyzeOptions, Cache, Error, Result};

/// An analysis together with the file it belongs to.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FileAnalysis {
    /// Path of the `.osu` file.
    pub path: PathBuf,
    /// Computed statistics.
    pub analysis: Analysis,
}

/// A file that could not be analyzed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileError {
    /// Path of the `.osu` file.
    pub path: PathBuf,
    /// Human readable reason.
    pub message: String,
}

/// Result of a batch analysis.
#[derive(Debug, Default)]
pub struct BatchOutcome {
    /// Successful analyses, sorted by path.
    pub files: Vec<FileAnalysis>,
    /// Files that failed to read or parse, sorted by path.
    pub errors: Vec<FileError>,
    /// How many of `files` were served from the cache.
    pub cached: usize,
}

impl BatchOutcome {
    /// Number of analyses that had to be computed in this run.
    pub fn analyzed(&self) -> usize {
        self.files.len().saturating_sub(self.cached)
    }

    /// Total number of files that were looked at.
    pub fn total(&self) -> usize {
        self.files.len() + self.errors.len()
    }
}

/// Finds every `.osu` file at or below `root`.
///
/// `root` may also be a single `.osu` file. The returned paths are sorted so
/// repeated runs produce identical output.
pub fn find_beatmaps(root: impl AsRef<Path>) -> Result<Vec<PathBuf>> {
    let root = root.as_ref();

    if root.is_file() {
        if is_osu(root) {
            return Ok(vec![root.to_path_buf()]);
        }

        return Err(Error::InvalidOptions(format!(
            "{} is not a .osu file",
            root.display()
        )));
    }

    if !root.is_dir() {
        return Err(Error::io(
            format!("{} does not exist or is not a folder", root.display()),
            io::Error::new(io::ErrorKind::NotFound, "path not found"),
        ));
    }

    let mut files = Vec::new();

    for entry in jwalk::WalkDir::new(root).into_iter() {
        let entry = entry.map_err(|err| {
            let io_error: io::Error = err.into();
            Error::io(format!("failed to scan {}", root.display()), io_error)
        })?;

        if entry.file_type().is_file() && is_osu(&entry.path()) {
            files.push(entry.path());
        }
    }

    files.sort();
    Ok(files)
}

fn is_osu(path: &Path) -> bool {
    path.extension()
        .map(|extension| extension.eq_ignore_ascii_case("osu"))
        .unwrap_or(false)
}

/// Analyzes every path in parallel, optionally reusing a cache.
pub fn analyze_paths(
    paths: &[PathBuf],
    options: &AnalyzeOptions,
    cache: Option<&mut Cache>,
) -> BatchOutcome {
    analyze_paths_with_progress(paths, options, cache, || {})
}

/// Like [`analyze_paths`], but calls `progress` once per processed file.
///
/// The callback runs on Rayon worker threads, so it must be thread-safe; an
/// `indicatif::ProgressBar` is a typical argument.
pub fn analyze_paths_with_progress<F>(
    paths: &[PathBuf],
    options: &AnalyzeOptions,
    cache: Option<&mut Cache>,
    progress: F,
) -> BatchOutcome
where
    F: Fn() + Sync + Send,
{
    let cache_ref = cache.as_deref();

    let results: Vec<std::result::Result<(FileAnalysis, bool), FileError>> = paths
        .par_iter()
        .map(|path| {
            let result = analyze_one(path, options, cache_ref);
            progress();
            result
        })
        .collect();

    let mut outcome = BatchOutcome::default();
    let mut fresh = Vec::new();

    for result in results {
        match result {
            Ok((file, from_cache)) => {
                if from_cache {
                    outcome.cached += 1;
                } else {
                    fresh.push(outcome.files.len());
                }

                outcome.files.push(file);
            }
            Err(error) => outcome.errors.push(error),
        }
    }

    // Update the cache before sorting so the recorded indices stay valid.
    if let Some(cache) = cache {
        for index in fresh {
            if let Some(file) = outcome.files.get(index) {
                cache.insert(&file.analysis, options);
            }
        }
    }

    outcome.files.sort_by(|a, b| a.path.cmp(&b.path));
    outcome.errors.sort_by(|a, b| a.path.cmp(&b.path));
    outcome
}

fn analyze_one(
    path: &Path,
    options: &AnalyzeOptions,
    cache: Option<&Cache>,
) -> std::result::Result<(FileAnalysis, bool), FileError> {
    let bytes = std::fs::read(path).map_err(|err| FileError {
        path: path.to_path_buf(),
        message: err.to_string(),
    })?;

    let md5 = md5_hex(&bytes);

    if let Some(cache) = cache {
        if let Some(analysis) = cache.get(&md5, options) {
            return Ok((
                FileAnalysis {
                    path: path.to_path_buf(),
                    analysis,
                },
                true,
            ));
        }
    }

    analyze_bytes_with_md5(&bytes, md5, options)
        .map(|analysis| {
            (
                FileAnalysis {
                    path: path.to_path_buf(),
                    analysis,
                },
                false,
            )
        })
        .map_err(|err| FileError {
            path: path.to_path_buf(),
            message: err.to_string(),
        })
}
