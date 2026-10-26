//! Beatmap parsing, metadata and statistical computation.
//!
//! `rosu-pp` is the only component that understands the `.osu` format. This
//! module exposes plain, serialisable data so consumers (the CLI, a GUI, other
//! tools) never have to depend on `rosu-pp` types themselves.

pub mod difficulty;
pub mod metadata;
pub mod patterns;
pub mod timing;

pub use difficulty::DifficultyStats;
pub use metadata::{parse_metadata, BeatmapMetadata, Mode};
pub use patterns::{detect_patterns, PatternStats};
pub use timing::TimingStats;

use rosu_pp::Beatmap;

use crate::{Error, Result};

/// Parses a beatmap from raw `.osu` bytes.
pub(crate) fn parse_beatmap(bytes: &[u8]) -> Result<Beatmap> {
    Beatmap::from_bytes(bytes).map_err(Error::from)
}
