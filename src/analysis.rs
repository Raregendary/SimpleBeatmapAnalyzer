//! The high-level analysis entry points.
//!
//! [`analyze_bytes`] is the primitive used by everything else: it parses the
//! beatmap once, computes metadata, timing, difficulty/pp and pattern stats,
//! and returns a plain [`Analysis`] that can be serialised, cached or sent over
//! a channel.

use std::path::Path;

use md5::{Digest, Md5};
use rosu_pp::{Beatmap, GameMode};
use serde::{Deserialize, Serialize};

use crate::beatmap::{self, BeatmapMetadata, DifficultyStats, Mode, PatternStats, TimingStats};
use crate::options::ANALYZER_VERSION;
use crate::{AnalyzeOptions, Error, Result};

/// Everything the analyzer knows about one beatmap.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Analysis {
    /// MD5 of the `.osu` file bytes, suitable as a cache key.
    pub md5: String,
    /// Version of the statistical algorithm that produced this analysis.
    pub analyzer_version: u32,
    /// Title/artist/creator/IDs and mode.
    pub metadata: BeatmapMetadata,
    /// BPM, playable length and object counts.
    pub timing: TimingStats,
    /// Stars, pp and mod-adjusted attributes.
    pub difficulty: DifficultyStats,
    /// Stream/jump statistics; `None` for non-standard modes and maps with
    /// three or fewer hit objects.
    pub patterns: Option<PatternStats>,
}

impl Analysis {
    /// Returns [`PatternStats`] or a zeroed default when the map has no
    /// pattern analysis. Useful for consumers that always want numbers.
    pub fn patterns_or_default(&self) -> PatternStats {
        self.patterns.clone().unwrap_or_default()
    }
}

/// Analyzes a single `.osu` file.
pub fn analyze_file(path: impl AsRef<Path>, options: &AnalyzeOptions) -> Result<Analysis> {
    options.validate()?;

    let path = path.as_ref();
    let bytes = std::fs::read(path)
        .map_err(|err| Error::io(format!("failed to read {}", path.display()), err))?;

    analyze_bytes(&bytes, options)
}

/// Analyzes the raw bytes of an `.osu` file.
pub fn analyze_bytes(bytes: &[u8], options: &AnalyzeOptions) -> Result<Analysis> {
    options.validate()?;

    let md5 = md5_hex(bytes);
    analyze_bytes_with_md5(bytes, md5, options)
}

/// Analyzes raw bytes when the caller already hashed them (used by the batch
/// scanner so each file is hashed only once).
pub(crate) fn analyze_bytes_with_md5(
    bytes: &[u8],
    md5: String,
    options: &AnalyzeOptions,
) -> Result<Analysis> {
    options.validate()?;

    let map = beatmap::parse_beatmap(bytes)?;
    let text = String::from_utf8_lossy(bytes);
    let mut metadata = beatmap::parse_metadata(&text);
    // rosu-pp's parsed mode is authoritative; the text parser only provides it
    // for tooling that reads metadata without a full parse.
    metadata.mode = Mode::from_rosu(map.mode);

    let timing = TimingStats::calculate(&map);
    let difficulty = DifficultyStats::calculate(&map, options.accuracy);
    let patterns = patterns_for(&map, options);

    Ok(Analysis {
        md5,
        analyzer_version: ANALYZER_VERSION,
        metadata,
        timing,
        difficulty,
        patterns,
    })
}

fn patterns_for(map: &Beatmap, options: &AnalyzeOptions) -> Option<PatternStats> {
    if map.mode != GameMode::Osu {
        return None;
    }

    beatmap::detect_patterns(
        &map.hit_objects,
        &map.timing_points,
        map.cs,
        options.stream_distance,
        options.jump_distance,
    )
}

/// Lower-case hex MD5 of arbitrary bytes.
pub fn md5_hex(bytes: &[u8]) -> String {
    let mut hasher = Md5::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}
