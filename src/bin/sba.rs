//! `sba` — analyze osu! beatmaps and print advanced statistics.
//!
//! The binary is a thin wrapper around the library: argument parsing, progress
//! display, output formatting and cache persistence. All statistics come from
//! `simple_beatmap_analyzer`, so the CLI and a GUI produce identical numbers.

use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use indicatif::{ProgressBar, ProgressStyle};
use simple_beatmap_analyzer::{
    analyze_paths_with_progress, find_beatmaps, parse_columns, write_csv, write_csv_to_path,
    AnalyzeOptions, Cache, Column, Error as AnalyzerError, FileAnalysis, Mode, ALL_COLUMNS,
    DEFAULT_COLUMNS,
};

const HELP: &str = "\
sba — advanced statistics for osu! beatmaps

USAGE:
    sba <PATH> [OPTIONS]

ARGS:
    <PATH>    A .osu file or a folder that is searched recursively

OUTPUT:
    -o, --output <FILE>   Write to FILE (use - for stdout)
                          Default: stdout for one file, results.csv or
                          results.json for a folder (depending on --format)
    -f, --format <FORMAT> csv (default) or json
        --columns <LIST>  Comma-separated columns, e.g. Title,Stars,Streams,SI
        --list-columns    Print every column name and exit

ANALYSIS:
        --stream-distance <PX>   Max edge-to-edge spacing for 1/4 patterns [default: 16]
        --jump-distance <PX>     Min edge-to-edge spacing for 1/2 jumps [default: 110]
        --accuracy <PCT>         Accuracy for NM/DT/HR pp [default: 99]
        --all-modes              Include taiko/catch/mania maps (pattern stats are standard-only)

PERFORMANCE:
        --cache <FILE>    Reuse analyses from a cache file, e.g. data/sba-cache.bin
        --no-progress     Do not draw a progress bar

MISC:
    -h, --help            Print this help
    -V, --version         Print the version

EXAMPLES:
    sba map.osu
    sba \"D:/osu/Songs\" -o results.csv --cache data/sba-cache.bin
    sba Songs --columns Title,DifName,Stars,Streams,JumpNotes --format csv
    sba Songs -f json -o analyses.json --stream-distance 20
";

/// Output format selected with `--format`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Format {
    Csv,
    Json,
}

/// Parsed command line.
#[derive(Debug)]
struct Args {
    path: PathBuf,
    output: Option<PathBuf>,
    format: Format,
    options: AnalyzeOptions,
    columns: Vec<Column>,
    cache: Option<PathBuf>,
    progress: bool,
    all_modes: bool,
}

enum Action {
    Run(Args),
    Help,
    Version,
    ListColumns,
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(message) => {
            eprintln!("error: {message}");
            eprintln!("try `sba --help` for usage");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<ExitCode, String> {
    let action = parse_args()?;

    match action {
        Action::Help => {
            print!("{HELP}");
            Ok(ExitCode::SUCCESS)
        }
        Action::Version => {
            println!("sba {}", env!("CARGO_PKG_VERSION"));
            Ok(ExitCode::SUCCESS)
        }
        Action::ListColumns => {
            let names: Vec<&str> = ALL_COLUMNS.iter().map(|column| column.header()).collect();
            println!("{}", names.join(","));
            Ok(ExitCode::SUCCESS)
        }
        Action::Run(args) => run_analysis(args),
    }
}

fn run_analysis(args: Args) -> Result<ExitCode, String> {
    let paths = find_beatmaps(&args.path).map_err(|err| err.to_string())?;

    if paths.is_empty() {
        return Err(format!("no .osu files found in {}", args.path.display()));
    }

    let single_file = paths.len() == 1 && args.path.is_file();

    let mut cache = match &args.cache {
        Some(path) => match Cache::open(path) {
            Ok(cache) => Some(cache),
            Err(err) => {
                eprintln!("warning: {err}; starting with an empty cache");
                Some(Cache::empty(path))
            }
        },
        None => None,
    };

    let progress = if args.progress && paths.len() > 1 {
        let bar = ProgressBar::new(paths.len() as u64);
        if let Ok(style) =
            ProgressStyle::with_template("{spinner:.green} {pos}/{len} [{bar:30}] {msg}")
        {
            bar.set_style(style.progress_chars("=>-"));
        }
        bar.set_message("analyzing");
        Some(bar)
    } else {
        None
    };

    let progress_ref = progress.as_ref();
    let outcome = analyze_paths_with_progress(&paths, &args.options, cache.as_mut(), || {
        if let Some(bar) = progress_ref {
            bar.inc(1);
        }
    });

    if let Some(bar) = &progress {
        bar.finish_and_clear();
    }

    let total = outcome.total();
    let analyzed = outcome.analyzed();
    let cached = outcome.cached;
    let failed = outcome.errors.len();

    for error in &outcome.errors {
        eprintln!("warning: {}: {}", error.path.display(), error.message);
    }

    let files: Vec<FileAnalysis> = outcome
        .files
        .into_iter()
        .filter(|file| args.all_modes || file.analysis.metadata.mode == Mode::Standard)
        .collect();

    if files.is_empty() {
        if failed == 0 {
            return Err(
                "no standard-mode beatmaps to analyze (use --all-modes to include other modes)"
                    .to_string(),
            );
        }
        return Ok(ExitCode::from(1));
    }

    write_output(&args, &files, single_file)?;

    eprintln!(
        "analyzed {total} file(s): {analyzed} computed, {cached} cached, {failed} failed; wrote {} row(s)",
        files.len()
    );

    if let Some(cache) = cache.as_mut() {
        match cache.save() {
            Ok(()) => eprintln!(
                "cache: {} entries at {}",
                cache.len(),
                cache.path().display()
            ),
            Err(err) => eprintln!("warning: {err}"),
        }
    }

    Ok(if failed == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    })
}

fn write_output(args: &Args, files: &[FileAnalysis], single_file: bool) -> Result<(), String> {
    let target = args.output.clone().or_else(|| {
        if single_file {
            None
        } else {
            Some(match args.format {
                Format::Csv => PathBuf::from("results.csv"),
                Format::Json => PathBuf::from("results.json"),
            })
        }
    });

    match (args.format, target) {
        (Format::Csv, Some(path)) if path != Path::new("-") => {
            write_csv_to_path(&path, files, &args.columns).map_err(|err| err.to_string())?;
            eprintln!("wrote CSV to {}", path.display());
        }
        (Format::Csv, _) => {
            let stdout = io::stdout();
            let mut lock = stdout.lock();
            if let Err(err) = write_csv(&mut lock, files, &args.columns) {
                if is_broken_pipe(&err) {
                    return Ok(());
                }
                return Err(err.to_string());
            }
        }
        (Format::Json, Some(path)) if path != Path::new("-") => {
            let file = File::create(&path).map_err(|err| format!("{}: {err}", path.display()))?;
            let mut writer = BufWriter::new(file);
            serde_json::to_writer_pretty(&mut writer, files).map_err(|err| err.to_string())?;
            writer
                .write_all(b"\n")
                .map_err(|err| format!("{}: {err}", path.display()))?;
            writer
                .flush()
                .map_err(|err| format!("{}: {err}", path.display()))?;
            eprintln!("wrote JSON to {}", path.display());
        }
        (Format::Json, _) => {
            let stdout = io::stdout();
            let mut lock = stdout.lock();
            if let Err(err) = serde_json::to_writer_pretty(&mut lock, files) {
                if err.io_error_kind() == Some(io::ErrorKind::BrokenPipe) {
                    return Ok(());
                }
                return Err(err.to_string());
            }
            if let Err(err) = writeln!(lock) {
                if err.kind() == io::ErrorKind::BrokenPipe {
                    return Ok(());
                }
                return Err(err.to_string());
            }
        }
    }

    Ok(())
}

fn parse_args() -> Result<Action, String> {
    let mut path: Option<PathBuf> = None;
    let mut output: Option<PathBuf> = None;
    let mut format = Format::Csv;
    let mut options = AnalyzeOptions::default();
    let mut columns: Option<Vec<Column>> = None;
    let mut cache: Option<PathBuf> = None;
    let mut progress = true;
    let mut all_modes = false;

    let mut args = std::env::args().skip(1);

    while let Some(raw) = args.next() {
        let (flag, inline) = split_inline(raw);

        match flag.as_str() {
            "-h" | "--help" => return Ok(Action::Help),
            "-V" | "--version" => return Ok(Action::Version),
            "--list-columns" => return Ok(Action::ListColumns),
            "-o" | "--output" => {
                output = Some(PathBuf::from(flag_value(&flag, inline, &mut args)?));
            }
            "-f" | "--format" => {
                let value = flag_value(&flag, inline, &mut args)?;
                format = match value.to_ascii_lowercase().as_str() {
                    "csv" => Format::Csv,
                    "json" => Format::Json,
                    other => {
                        return Err(format!("unknown format {other:?} (expected csv or json)"))
                    }
                };
            }
            "--columns" => {
                let value = flag_value(&flag, inline, &mut args)?;
                columns = Some(parse_columns(&value).map_err(|err| err.to_string())?);
            }
            "--stream-distance" => {
                let value = flag_value(&flag, inline, &mut args)?;
                options.stream_distance = parse_number(&value, &flag)?;
            }
            "--jump-distance" => {
                let value = flag_value(&flag, inline, &mut args)?;
                options.jump_distance = parse_number(&value, &flag)?;
            }
            "--accuracy" => {
                let value = flag_value(&flag, inline, &mut args)?;
                options.accuracy = parse_number(&value, &flag)?;
            }
            "--cache" => {
                cache = Some(PathBuf::from(flag_value(&flag, inline, &mut args)?));
            }
            "--no-progress" => progress = false,
            "--all-modes" => all_modes = true,
            other if other.starts_with('-') && other != "-" => {
                return Err(format!("unknown option {other}"));
            }
            _ => {
                if path.is_some() {
                    return Err("only one <PATH> may be given".to_string());
                }
                path = Some(PathBuf::from(flag));
            }
        }
    }

    let path = path
        .ok_or_else(|| "missing <PATH> (pass a .osu file or a folder of beatmaps)".to_string())?;

    options.validate().map_err(|err| err.to_string())?;

    Ok(Action::Run(Args {
        path,
        output,
        format,
        options,
        columns: columns.unwrap_or_else(|| DEFAULT_COLUMNS.to_vec()),
        cache,
        progress,
        all_modes,
    }))
}

fn split_inline(raw: String) -> (String, Option<String>) {
    if raw.starts_with("--") {
        if let Some((flag, value)) = raw.split_once('=') {
            return (flag.to_string(), Some(value.to_string()));
        }
    }

    (raw, None)
}

fn flag_value(
    flag: &str,
    inline: Option<String>,
    args: &mut impl Iterator<Item = String>,
) -> Result<String, String> {
    if let Some(value) = inline {
        return Ok(value);
    }

    args.next()
        .ok_or_else(|| format!("{flag} requires a value"))
}

fn parse_number<T: std::str::FromStr>(value: &str, flag: &str) -> Result<T, String> {
    value
        .parse()
        .map_err(|_| format!("invalid value {value:?} for {flag}"))
}

fn is_broken_pipe(err: &AnalyzerError) -> bool {
    matches!(err, AnalyzerError::Io { source, .. } if source.kind() == io::ErrorKind::BrokenPipe)
}
