//! `.osu` metadata parsing.
//!
//! rosu-pp parses the parts needed for difficulty and performance calculation,
//! but it intentionally does not expose the metadata block. This parser reads
//! `[General]` and `[Metadata]` directly and is independent of section order,
//! so maps that put `[Metadata]` before `[General]` (or omit optional keys) are
//! handled just as well as the usual layout.

use std::fmt;

use serde::{Deserialize, Serialize};

/// Game mode of a beatmap.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    /// osu!standard
    #[default]
    Standard,
    /// osu!taiko
    Taiko,
    /// osu!catch
    Catch,
    /// osu!mania
    Mania,
}

impl Mode {
    /// Lower-case name used in CSV/JSON output.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Standard => "standard",
            Self::Taiko => "taiko",
            Self::Catch => "catch",
            Self::Mania => "mania",
        }
    }

    /// Maps the numeric `Mode` value of an `.osu` file to a [`Mode`].
    ///
    /// Unknown values fall back to standard, matching osu!'s own behaviour.
    pub const fn from_number(value: u8) -> Self {
        match value {
            1 => Self::Taiko,
            2 => Self::Catch,
            3 => Self::Mania,
            _ => Self::Standard,
        }
    }

    /// Maps a `rosu-pp` game mode to a [`Mode`].
    pub const fn from_rosu(mode: rosu_pp::GameMode) -> Self {
        match mode {
            rosu_pp::GameMode::Osu => Self::Standard,
            rosu_pp::GameMode::Taiko => Self::Taiko,
            rosu_pp::GameMode::Catch => Self::Catch,
            rosu_pp::GameMode::Mania => Self::Mania,
        }
    }
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Metadata of an `.osu` file.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BeatmapMetadata {
    /// Format version from the `osu file format vN` header, `0` if missing.
    pub format_version: u8,
    /// Game mode declared by the map.
    pub mode: Mode,
    /// `Title`
    pub title: String,
    /// `TitleUnicode` (empty when the map does not provide one).
    pub title_unicode: String,
    /// `Artist`
    pub artist: String,
    /// `ArtistUnicode` (empty when the map does not provide one).
    pub artist_unicode: String,
    /// `Creator` (the mapper).
    pub creator: String,
    /// `Version` (the difficulty name).
    pub version: String,
    /// `Source`
    pub source: String,
    /// `Tags`, split on whitespace.
    pub tags: Vec<String>,
    /// `BeatmapID`, `0` for unsubmitted maps.
    pub beatmap_id: u32,
    /// `BeatmapSetID`, `0` for unsubmitted maps.
    pub beatmap_set_id: u32,
    /// `AudioFilename`
    pub audio_filename: String,
}

impl BeatmapMetadata {
    /// The Unicode title when present, otherwise the romanised one.
    pub fn display_title(&self) -> &str {
        if self.title_unicode.is_empty() {
            &self.title
        } else {
            &self.title_unicode
        }
    }

    /// The Unicode artist when present, otherwise the romanised one.
    pub fn display_artist(&self) -> &str {
        if self.artist_unicode.is_empty() {
            &self.artist
        } else {
            &self.artist_unicode
        }
    }
}

/// Parses the metadata of an `.osu` file from its text.
///
/// This never fails: missing keys stay empty and malformed numeric IDs become
/// `0`. Consumers that need strict validation should rely on
/// [`Beatmap::from_bytes`](rosu_pp::Beatmap::from_bytes) for the playable part
/// of the file.
pub fn parse_metadata(osu_text: &str) -> BeatmapMetadata {
    let mut metadata = BeatmapMetadata::default();
    let mut section = String::new();

    for (index, raw_line) in osu_text.lines().enumerate() {
        let line = raw_line.trim();
        // A UTF-8 byte order mark is legal at the start of an .osu file.
        let line = if index == 0 {
            line.trim_start_matches('\u{feff}')
        } else {
            line
        };

        if line.is_empty() || line.starts_with("//") {
            continue;
        }

        if let Some(version) = line
            .strip_prefix("osu file format v")
            .and_then(|value| value.trim().parse::<u8>().ok())
        {
            metadata.format_version = version;
            continue;
        }

        if line.starts_with('[') && line.ends_with(']') {
            section.clear();
            section.push_str(&line[1..line.len() - 1]);
            continue;
        }

        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        let key = key.trim();
        let value = value.trim();

        match section.as_str() {
            "General" => match key {
                "AudioFilename" => metadata.audio_filename = value.to_string(),
                "Mode" => {
                    metadata.mode = value
                        .parse::<u8>()
                        .map(Mode::from_number)
                        .unwrap_or_default();
                }
                _ => {}
            },
            "Metadata" => match key {
                "Title" => metadata.title = value.to_string(),
                "TitleUnicode" => metadata.title_unicode = value.to_string(),
                "Artist" => metadata.artist = value.to_string(),
                "ArtistUnicode" => metadata.artist_unicode = value.to_string(),
                "Creator" => metadata.creator = value.to_string(),
                "Version" => metadata.version = value.to_string(),
                "Source" => metadata.source = value.to_string(),
                "Tags" => {
                    metadata.tags = value
                        .split_whitespace()
                        .filter(|tag| !tag.is_empty())
                        .map(str::to_string)
                        .collect();
                }
                "BeatmapID" => metadata.beatmap_id = parse_id(value),
                "BeatmapSetID" => metadata.beatmap_set_id = parse_id(value),
                _ => {}
            },
            _ => {}
        }
    }

    metadata
}

fn parse_id(value: &str) -> u32 {
    value.trim().parse::<u32>().unwrap_or(0)
}
