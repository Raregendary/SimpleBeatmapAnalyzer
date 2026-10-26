//! CSV export with selectable columns.
//!
//! Columns are identified by [`Column`]; the default set is
//! [`DEFAULT_COLUMNS`]. Percentages are formatted with two decimals, star
//! ratings and pp with two decimals, and pattern cells stay empty for maps
//! without pattern statistics (non-standard modes, very short maps).

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use crate::beatmap::PatternStats;
use crate::{Error, FileAnalysis, Result};

/// One column of the CSV output.
///
/// The variant names are stable API; the header strings are what ends up in the
/// file and match the column names of the original 0.9.x analyzer where
/// possible.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Column {
    Path,
    Title,
    Artist,
    Creator,
    DifName,
    MapId,
    MapSetId,
    Mode,
    Md5,
    Stars,
    DtStars,
    HrStars,
    Nm99,
    Dt99,
    Hr99,
    MaxCombo,
    Cs,
    Ar,
    Od,
    Hp,
    DtAr,
    DtOd,
    HrAr,
    HrOd,
    HrCs,
    HrHp,
    Bpm,
    DtBpm,
    BpmMin,
    BpmMax,
    PlayableLength,
    Circles,
    Sliders,
    Spinners,
    Objects,
    Doubles,
    Triples,
    Quads,
    Bursts,
    Streams,
    DeathStreams,
    LongestStream,
    Streams100,
    ShortJumps,
    MidJumps,
    LongJumps,
    JumpNotes,
    AvgJumpDistance,
    AvgJumpSpeed,
    OneThird,
    Si,
    Ji,
    Fcdbi,
    OsuDirect,
    OsuWebLink,
}

/// Every supported column, in the default order.
pub const ALL_COLUMNS: &[Column] = &[
    Column::Path,
    Column::Title,
    Column::Artist,
    Column::Creator,
    Column::DifName,
    Column::MapId,
    Column::MapSetId,
    Column::Mode,
    Column::Md5,
    Column::Stars,
    Column::DtStars,
    Column::HrStars,
    Column::Nm99,
    Column::Dt99,
    Column::Hr99,
    Column::MaxCombo,
    Column::Cs,
    Column::Ar,
    Column::Od,
    Column::Hp,
    Column::DtAr,
    Column::DtOd,
    Column::HrAr,
    Column::HrOd,
    Column::HrCs,
    Column::HrHp,
    Column::Bpm,
    Column::DtBpm,
    Column::BpmMin,
    Column::BpmMax,
    Column::PlayableLength,
    Column::Circles,
    Column::Sliders,
    Column::Spinners,
    Column::Objects,
    Column::Doubles,
    Column::Triples,
    Column::Quads,
    Column::Bursts,
    Column::Streams,
    Column::DeathStreams,
    Column::LongestStream,
    Column::Streams100,
    Column::ShortJumps,
    Column::MidJumps,
    Column::LongJumps,
    Column::JumpNotes,
    Column::AvgJumpDistance,
    Column::AvgJumpSpeed,
    Column::OneThird,
    Column::Si,
    Column::Ji,
    Column::Fcdbi,
    Column::OsuDirect,
    Column::OsuWebLink,
];

/// Columns written when the caller does not select a subset.
pub const DEFAULT_COLUMNS: &[Column] = ALL_COLUMNS;

impl Column {
    /// Header text written for this column.
    pub const fn header(self) -> &'static str {
        match self {
            Self::Path => "Path",
            Self::Title => "Title",
            Self::Artist => "Artist",
            Self::Creator => "Creator",
            Self::DifName => "DifName",
            Self::MapId => "MapID",
            Self::MapSetId => "MapSetID",
            Self::Mode => "Mode",
            Self::Md5 => "MD5",
            Self::Stars => "Stars",
            Self::DtStars => "DT_Stars",
            Self::HrStars => "HR_Stars",
            Self::Nm99 => "NM_99",
            Self::Dt99 => "DT_99",
            Self::Hr99 => "HR_99",
            Self::MaxCombo => "MaxCombo",
            Self::Cs => "CS",
            Self::Ar => "AR",
            Self::Od => "OD",
            Self::Hp => "HP",
            Self::DtAr => "DT_AR",
            Self::DtOd => "DT_OD",
            Self::HrAr => "HR_AR",
            Self::HrOd => "HR_OD",
            Self::HrCs => "HR_CS",
            Self::HrHp => "HR_HP",
            Self::Bpm => "BPM",
            Self::DtBpm => "DT_BPM",
            Self::BpmMin => "BPM_Min",
            Self::BpmMax => "BPM_Max",
            Self::PlayableLength => "PlayableLength",
            Self::Circles => "Circles",
            Self::Sliders => "Sliders",
            Self::Spinners => "Spinners",
            Self::Objects => "Objects",
            Self::Doubles => "Doubles",
            Self::Triples => "Triples",
            Self::Quads => "Quads",
            Self::Bursts => "Bursts",
            Self::Streams => "Streams",
            Self::DeathStreams => "DeathStreams",
            Self::LongestStream => "LongestStream",
            Self::Streams100 => "Streams100",
            Self::ShortJumps => "ShortJumps",
            Self::MidJumps => "MidJumps",
            Self::LongJumps => "LongJumps",
            Self::JumpNotes => "JumpNotes",
            Self::AvgJumpDistance => "AvgJumpsDistance",
            Self::AvgJumpSpeed => "AvgJumpsSpeed",
            Self::OneThird => "OneThird",
            Self::Si => "SI",
            Self::Ji => "JI",
            Self::Fcdbi => "FCDBI",
            Self::OsuDirect => "OsuDirect",
            Self::OsuWebLink => "OsuWebLink",
        }
    }

    /// Parses a column name or alias, case-insensitively.
    pub fn parse(input: &str) -> Result<Self> {
        let name = input.trim();

        if let Some(column) = ALL_COLUMNS
            .iter()
            .copied()
            .find(|column| column.header().eq_ignore_ascii_case(name))
        {
            return Ok(column);
        }

        let column = match name.to_ascii_lowercase().as_str() {
            "file" | "filename" => Self::Path,
            "version" | "difficulty" | "diff" => Self::DifName,
            "mapid" | "beatmapid" | "beatmap_id" => Self::MapId,
            "mapsetid" | "beatmapsetid" | "beatmap_set_id" => Self::MapSetId,
            "sr" | "nm_stars" | "starrating" => Self::Stars,
            "pp" | "nm_pp" => Self::Nm99,
            "dt_pp" => Self::Dt99,
            "hr_pp" => Self::Hr99,
            "length" | "playable_length" | "duration" => Self::PlayableLength,
            "notes" | "note_count" | "hitobjects" => Self::Objects,
            "short_jumps" => Self::ShortJumps,
            "mid_jumps" => Self::MidJumps,
            "long_jumps" => Self::LongJumps,
            "jump_notes" => Self::JumpNotes,
            "death_streams" => Self::DeathStreams,
            "longest_stream" => Self::LongestStream,
            "streams_100" => Self::Streams100,
            "avgjumpsdistance" | "avg_jump_distance" | "avg_jumps_distance" => {
                Self::AvgJumpDistance
            }
            "avgjumpsspeed" | "avg_jump_speed" | "avg_jumps_speed" => Self::AvgJumpSpeed,
            "onethird_notes" | "one_third" | "one_third_notes" => Self::OneThird,
            _ => {
                return Err(Error::InvalidOptions(format!(
                    "unknown column {input:?} (use --list-columns to see all names)"
                )))
            }
        };

        Ok(column)
    }
}

/// Parses a comma-separated column list such as `"Title,Stars,BPM"`.
pub fn parse_columns(input: &str) -> Result<Vec<Column>> {
    let columns: Vec<Column> = input
        .split(',')
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(Column::parse)
        .collect::<Result<_>>()?;

    if columns.is_empty() {
        return Err(Error::InvalidOptions(
            "the column list is empty".to_string(),
        ));
    }

    Ok(columns)
}

impl Column {
    /// Formatted value of this column for one analyzed file.
    pub fn value(self, file: &FileAnalysis) -> String {
        let analysis = &file.analysis;
        let metadata = &analysis.metadata;
        let difficulty = &analysis.difficulty;

        match self {
            Self::Path => file.path.display().to_string(),
            Self::Title => metadata.title.clone(),
            Self::Artist => metadata.artist.clone(),
            Self::Creator => metadata.creator.clone(),
            Self::DifName => metadata.version.clone(),
            Self::MapId => optional_id(metadata.beatmap_id),
            Self::MapSetId => optional_id(metadata.beatmap_set_id),
            Self::Mode => metadata.mode.to_string(),
            Self::Md5 => analysis.md5.clone(),
            Self::Stars => format!("{:.2}", difficulty.stars_nm),
            Self::DtStars => format!("{:.2}", difficulty.stars_dt),
            Self::HrStars => format!("{:.2}", difficulty.stars_hr),
            Self::Nm99 => format!("{:.2}", difficulty.pp_nm),
            Self::Dt99 => format!("{:.2}", difficulty.pp_dt),
            Self::Hr99 => format!("{:.2}", difficulty.pp_hr),
            Self::MaxCombo => difficulty.max_combo.to_string(),
            Self::Cs => format!("{:.2}", difficulty.cs),
            Self::Ar => format!("{:.2}", difficulty.ar),
            Self::Od => format!("{:.2}", difficulty.od),
            Self::Hp => format!("{:.2}", difficulty.hp),
            Self::DtAr => format!("{:.2}", difficulty.ar_dt),
            Self::DtOd => format!("{:.2}", difficulty.od_dt),
            Self::HrAr => format!("{:.2}", difficulty.ar_hr),
            Self::HrOd => format!("{:.2}", difficulty.od_hr),
            Self::HrCs => format!("{:.2}", difficulty.cs_hr),
            Self::HrHp => format!("{:.2}", difficulty.hp_hr),
            _ => self.stat_value(file),
        }
    }

    fn stat_value(self, file: &FileAnalysis) -> String {
        let timing = &file.analysis.timing;

        match self {
            Self::Bpm => format!("{:.2}", timing.bpm),
            Self::DtBpm => format!("{:.2}", timing.bpm_dt),
            Self::BpmMin => format!("{:.2}", timing.bpm_min),
            Self::BpmMax => format!("{:.2}", timing.bpm_max),
            Self::PlayableLength => {
                format!("{:.0}", timing.playable_length_sec.round())
            }
            Self::Circles => timing.circles.to_string(),
            Self::Sliders => timing.sliders.to_string(),
            Self::Spinners => timing.spinners.to_string(),
            Self::Objects => timing.objects.to_string(),
            Self::OsuDirect => hyperlink(file.analysis.metadata.beatmap_id, |id| {
                format!("osu://b/{id}")
            }),
            Self::OsuWebLink => hyperlink(file.analysis.metadata.beatmap_id, |id| {
                format!("https://osu.ppy.sh/b/{id}")
            }),
            _ => pattern_cell(file, |patterns| match self {
                Self::Doubles => format!("{:.2}", patterns.doubles_pct()),
                Self::Triples => format!("{:.2}", patterns.triples_pct()),
                Self::Quads => format!("{:.2}", patterns.quads_pct()),
                Self::Bursts => format!("{:.2}", patterns.burst_pct()),
                Self::Streams => format!("{:.2}", patterns.stream_pct()),
                Self::DeathStreams => format!("{:.2}", patterns.deathstream_pct()),
                Self::LongestStream => patterns.longest_stream.to_string(),
                Self::Streams100 => patterns.streams_100.to_string(),
                Self::ShortJumps => format!("{:.2}", patterns.short_jump_pct()),
                Self::MidJumps => format!("{:.2}", patterns.mid_jump_pct()),
                Self::LongJumps => format!("{:.2}", patterns.long_jump_pct()),
                Self::JumpNotes => patterns.jump_notes().to_string(),
                Self::AvgJumpDistance => format!("{:.2}", patterns.avg_jump_distance),
                Self::AvgJumpSpeed => format!("{:.2}", patterns.avg_jump_speed),
                Self::OneThird => format!("{:.2}", patterns.one_third_pct()),
                Self::Si => format!("{:.2}", patterns.si),
                Self::Ji => format!("{:.2}", patterns.ji),
                Self::Fcdbi => format!("{:.2}", patterns.fcdbi),
                _ => String::new(),
            }),
        }
    }
}

fn optional_id(id: u32) -> String {
    if id == 0 {
        String::new()
    } else {
        id.to_string()
    }
}

fn hyperlink(id: u32, target: impl Fn(u32) -> String) -> String {
    if id == 0 {
        String::new()
    } else {
        format!("=HYPERLINK(\"{}\")", target(id))
    }
}

fn pattern_cell(file: &FileAnalysis, render: impl FnOnce(&PatternStats) -> String) -> String {
    match &file.analysis.patterns {
        Some(patterns) => render(patterns),
        None => String::new(),
    }
}

/// Writes a CSV table with the given columns.
///
/// The writer is not flushed/closed by this function beyond flushing the CSV
/// buffer, so it can be reused (e.g. when writing to stdout).
pub fn write_csv<W: Write>(writer: W, files: &[FileAnalysis], columns: &[Column]) -> Result<()> {
    if columns.is_empty() {
        return Err(Error::InvalidOptions(
            "no CSV columns were selected".to_string(),
        ));
    }

    let mut writer = csv::Writer::from_writer(writer);
    writer.write_record(columns.iter().map(|column| column.header()))?;

    for file in files {
        writer.write_record(columns.iter().map(|column| column.value(file)))?;
    }

    writer
        .flush()
        .map_err(|err| Error::io("failed to flush CSV output", err))?;

    Ok(())
}

/// Writes a CSV table to `path`.
pub fn write_csv_to_path(
    path: impl AsRef<Path>,
    files: &[FileAnalysis],
    columns: &[Column],
) -> Result<()> {
    let path = path.as_ref();
    let file = File::create(path)
        .map_err(|err| Error::io(format!("failed to create {}", path.display()), err))?;

    write_csv(BufWriter::new(file), files, columns)
}
