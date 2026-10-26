//! # Simple Beatmap Analyzer
//!
//! A library for turning osu! beatmaps into advanced statistics.
//!
//! The crate is deliberately free of UI and filesystem-layout assumptions: it
//! parses `.osu` data with `rosu-pp`, returns plain serialisable structs and
//! can be used by a CLI, a GUI, a web service or another tool.
//!
//! ## Quick start
//!
//! ```no_run
//! use simple_beatmap_analyzer::{analyze_file, AnalyzeOptions};
//!
//! let options = AnalyzeOptions::default();
//! let analysis = analyze_file("map.osu", &options)?;
//!
//! println!("{} [{}] {:.2}*", analysis.metadata.title, analysis.metadata.version, analysis.difficulty.stars_nm);
//!
//! if let Some(patterns) = &analysis.patterns {
//!     println!("streams {:.2}% | jumps {:.2}%", patterns.stream_pct(), patterns.jump_pct());
//! }
//! # Ok::<(), simple_beatmap_analyzer::Error>(())
//! ```
//!
//! ## Batch processing
//!
//! [`scan::find_beatmaps`] finds every `.osu` file below a folder and
//! [`scan::analyze_paths`] analyzes them in parallel. Pass a [`Cache`] to reuse
//! previous results; cache keys combine the file content hash and the analysis
//! options, so changing a threshold never returns stale numbers.
//!
//! ## Feature overview
//!
//! * Metadata (title, artist, creator, difficulty name, both IDs, mode).
//! * NM/DT/HR star ratings and pp at a configurable accuracy.
//! * BPM (duration-weighted), BPM range, playable length, object counts.
//! * 1/4 patterns: doubles, triples, quads, bursts, streams, deathstreams,
//!   longest stream and 100+ streams.
//! * 1/2 patterns: short/mid/long jump runs, average jump distance and speed.
//! * 1/3 snap notes and the SI/JI/FCDBI indices.
//! * CSV and JSON export (CSV with selectable columns).

#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

pub mod analysis;
pub mod beatmap;
pub mod cache;
pub mod csv;
pub mod error;
pub mod options;
pub mod scan;

pub use analysis::{analyze_bytes, analyze_file, md5_hex, Analysis};
pub use beatmap::{
    detect_patterns, parse_metadata, BeatmapMetadata, DifficultyStats, Mode, PatternStats,
    TimingStats,
};
pub use cache::Cache;
pub use csv::{parse_columns, write_csv, write_csv_to_path, Column, ALL_COLUMNS, DEFAULT_COLUMNS};
pub use error::{Error, Result};
pub use options::{
    AnalyzeOptions, ANALYZER_VERSION, DEFAULT_ACCURACY, DEFAULT_JUMP_DISTANCE,
    DEFAULT_STREAM_DISTANCE,
};
pub use scan::{
    analyze_paths, analyze_paths_with_progress, find_beatmaps, BatchOutcome, FileAnalysis,
    FileError,
};
