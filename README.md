# Simple Beatmap Analyzer

A Rust **library and CLI** for turning osu! beatmaps into advanced statistics:
stars and pp for NM/DT/HR, BPM and object counts, and a stream/jump pattern
analysis (bursts, streams, deathstreams, jumps, 1/3 snaps, SI/JI/FCDBI).

The project is split so that other tools — a GUI, a website, a discord bot —
can depend on the analysis core without re-implementing (or copying) any of the
math:

| Component | Crate | Purpose |
|---|---|---|
| **Library** | `simple_beatmap_analyzer` | Parsing, statistics, batch scanning, cache, CSV/JSON-ready data |
| **CLI** | `sba` | Analyze a file or a Songs folder and write CSV/JSON |

`rosu-pp` performs `.osu` parsing and the difficulty/performance calculations;
everything else is implemented here and returns plain, serialisable structs. The
`rosu-pp` usage is isolated in the `beatmap` layer, so the `rosu-pp-gemini` fork
can be substituted without touching the CLI, cache or export code (see
[`ARCHITECTURE.md`](ARCHITECTURE.md#switching-the-parserdifficulty-backend)).

This repository is the library + CLI half of the planned split; the GUI is a
separate project that depends on the library.

---

## Install

```bash
# CLI from a checkout
cargo install --path .

# or run without installing
cargo run --release -- --help
```

The library is consumed as a normal path (or git) dependency:

```toml
[dependencies]
simple-beatmap-analyzer = { path = "../SimpleBeatmapAnalyzer" }
```

Requires a recent stable Rust toolchain (edition 2021).

---

## Library usage

```rust
use simple_beatmap_analyzer::{
    analyze_file, analyze_paths, find_beatmaps, AnalyzeOptions,
};

fn main() -> Result<(), simple_beatmap_analyzer::Error> {
    let options = AnalyzeOptions::default();
    let analysis = analyze_file("map.osu", &options)?;

    println!(
        "{} [{}] — {:.2}★, {:.2}pp, {:.0} BPM",
        analysis.metadata.title,
        analysis.metadata.version,
        analysis.difficulty.stars_nm,
        analysis.difficulty.pp_nm,
        analysis.timing.bpm,
    );

    if let Some(patterns) = &analysis.patterns {
        println!(
            "streams {:.2}% | jumps {:.2}% | longest stream {} | SI {:.2}",
            patterns.stream_pct(),
            patterns.jump_pct(),
            patterns.longest_stream,
            patterns.si,
        );
    }

    // Batch: every .osu file below a folder, analyzed in parallel.
    let paths = find_beatmaps("Songs")?;
    let outcome = analyze_paths(&paths, &options, None);
    println!(
        "{} analyzed, {} failed",
        outcome.analyzed(),
        outcome.errors.len()
    );

    Ok(())
}
```

The public surface is intentionally small:

* `analyze_file` / `analyze_bytes` → `Analysis`
* `find_beatmaps` / `analyze_paths` → `BatchOutcome`
* `Cache` for incremental runs
* `write_csv` / `write_csv_to_path` with `Column` selection
* `parse_metadata` for tools that need metadata without a full parse

All result types implement `serde::Serialize`/`Deserialize`, so they can be sent
over a channel, stored, or printed as JSON without conversion.

### Analysis structure

```
Analysis
├─ metadata    title, artist, creator, difficulty name, BeatmapID, BeatmapSetID,
│              mode, format version, audio filename, tags
├─ timing      BPM (duration-weighted), BPM min/max, DT BPM,
│              playable length, circle/slider/spinner/object counts
├─ difficulty  CS/AR/OD/HP, NM/DT/HR stars and pp, max combo,
│              DT_AR/DT_OD, HR_AR/HR_OD/HR_CS/HR_HP
└─ patterns    Option<PatternStats> — standard mode and 4+ objects only
```

---

## CLI usage

```text
sba <PATH> [OPTIONS]
```

`PATH` may be a single `.osu` file or a folder (searched recursively).

```bash
# One map, CSV on stdout
sba "Songs/Artist - Title/map.osu"

# A whole library
sba "D:/osu/Songs" -o results.csv --cache data/sba-cache.bin

# Pick columns
sba Songs --columns Title,DifName,Stars,Streams,JumpNotes,SI

# JSON for another tool
sba Songs -f json -o analyses.json

# Include taiko/catch/mania maps as well
sba Songs --all-modes
```

| Option | Meaning |
|---|---|
| `-o, --output <FILE>` | Output file; `-` means stdout. Default: stdout for a single file, `results.csv` (CSV) or `results.json` (JSON) for a folder. |
| `-f, --format csv\|json` | Output format (default `csv`). |
| `--columns <LIST>` | CSV columns, comma separated. Default: every column. |
| `--list-columns` | Print all column names. |
| `--stream-distance <PX>` | Max edge-to-edge spacing for 1/4 patterns (default `16`). |
| `--jump-distance <PX>` | Min edge-to-edge spacing for 1/2 jumps (default `110`). |
| `--accuracy <PCT>` | Accuracy used for NM/DT/HR pp (default `99`). |
| `--cache <FILE>` | Reuse analyses from a cache file. |
| `--no-progress` | Disable the progress bar. |
| `--all-modes` | Keep non-standard maps in the output. |
| `-h, --help` / `-V, --version` | Help / version. |

Exit codes: `0` success, `1` at least one file failed to analyze, `2` usage or
configuration error.

### Default CSV columns

```
Path, Title, Artist, Creator, DifName, MapID, MapSetID, Mode, MD5,
Stars, DT_Stars, HR_Stars, NM_99, DT_99, HR_99, MaxCombo,
CS, AR, OD, HP, DT_AR, DT_OD, HR_AR, HR_OD, HR_CS, HR_HP,
BPM, DT_BPM, BPM_Min, BPM_Max, PlayableLength,
Circles, Sliders, Spinners, Objects,
Doubles, Triples, Quads, Bursts, Streams, DeathStreams, LongestStream,
Streams100, ShortJumps, MidJumps, LongJumps, JumpNotes,
AvgJumpsDistance, AvgJumpsSpeed, OneThird, SI, JI, FCDBI,
OsuDirect, OsuWebLink
```

Star/pp/percentage/attribute values use two decimals. Pattern cells are empty
for maps without pattern statistics (non-standard modes, three or fewer hit
objects). `Doubles`/`Triples`/`Quads`/`Bursts`/`Streams`/`DeathStreams`/
`ShortJumps`/`MidJumps`/`LongJumps`/`OneThird` are percentages of hit objects,
while `LongestStream`, `Streams100` and `JumpNotes` are counts.
`OsuDirect` and `OsuWebLink` are Excel `HYPERLINK` formulas and are empty when
the map has no BeatmapID.


---

## What is measured

### Streams and bursts (1/4 patterns)

Consecutive notes are grouped when the edge-to-edge distance between their
centres (after subtracting the CS circle diameter) is at most
`stream_distance` **and** the spacing is a 1/4 snap
(`0.9 < dt / (beat_len / 4) < 1.1`).

| Run (notes) | Bucket |
|---:|---|
| 2 | doubles |
| 3 | triples |
| 4 | quads |
| 3–11 | bursts (includes triples and quads) |
| 12–32 | streams |
| 33+ | deathstreams |

`LongestStream` is the longest 1/4 run and `Streams100` counts runs of at least
100 notes. A run that reaches the last object of the map is counted (the 0.9.x
"last pattern" fix).

### Jumps (1/2 patterns)

A note is part of a jump when the edge-to-edge distance is greater than
`jump_distance` **and** the spacing is a 1/2 snap
(`1.9 < dt / (beat_len / 4) < 2.1`). Runs are measured in notes:

| Run (notes) | Bucket |
|---:|---|
| 2–12 | short jumps |
| 13–32 | mid jumps |
| 33+ | long jumps |

Detection is movement-based: the object before the first jump move of a run is
part of the run's note count, so a 20-move jump sequence reports 21 notes. This
matches the original analyzer and keeps `ShortJumps` comparable with `Streams`.

For consumers that use the GUI's `<8` / `>=8` split, `PatternStats` also exposes
`jump_notes_under_8` and `jump_notes_8_plus` with
`short_jump_pct_under_8()` / `long_jump_pct_8_plus()`.

Averages use only the moves that were actually measured, so
`AvgJumpsDistance`/`AvgJumpsSpeed` are true means (0.9.x divided by an inflated
counter).

### 1/3 snaps

Notes whose ratio is between `1.25` and `1.45` are counted as `OneThird`,
regardless of spacing.

### Indices

```
jump_value   = short + 1.5*mid + 2.0*long
stream_value = bursts + 1.5*streams + 2.0*deathstreams
SI    = (stream_value - jump_value - 0.5*doubles) / notes
JI    = (jump_value - stream_value - 0.5*doubles) / notes
FCDBI = (bursts - triples - quads + 1.75*doubles + 1.5*triples + 1.75*quads
         - jump_value - 1.35*streams - 1.7*deathstreams) / notes
```

All percentages are percentages of **hit objects**, not of the map duration.
Slider bodies are not walked; only object head positions are used, matching the
original analyzer.

### Other statistics

| Field | Definition |
|---|---|
| `BPM` | Most common BPM, weighted by how long each timing section is active (not by note count). |
| `BPM_Min` / `BPM_Max` | Range of uninherited timing points. |
| `PlayableLength` | Seconds from the first object to the end of the last object (sliders/spinners use their end time). |
| `Objects` | Total hit objects; `Circles`/`Sliders`/`Spinners` are the per-type counts. |
| `NM_99`/`DT_99`/`HR_99` | pp at the configured accuracy with no mods / Double Time / Hard Rock. |
| `MaxCombo` | Maximum combo from the NM difficulty calculation. |

---

## Caching

The cache is keyed by `(md5 of the .osu file, analysis options, algorithm
version)`:

* changing `--stream-distance`, `--jump-distance` or `--accuracy` recomputes
  only what is affected;
* moving or renaming a file reuses the analysis (the path is not part of the
  key);
* an algorithm change (`ANALYZER_VERSION`) invalidates old entries;
* the cache stores statistics only — never parsed beatmaps — so it stays small
  (a few KB for a handful of maps, well under 1 KB per map).

The cache is opt-in via `--cache`; the library works without one. When a cache
file is corrupt or was written by an incompatible version, the CLI warns and
starts with an empty cache instead of aborting.

---

## Migrating from 0.9.x

The 0.9.x application was a CSV-only binary with `columns_config.txt` and
`options_config.txt`. 0.10 moves the logic into a library and replaces the
configuration files with typed API values and CLI flags:

| 0.9.x | 0.10 |
|---|---|
| `columns_config.txt` | `--columns` / `Column` API |
| `options_config.txt` (`StreamDistance`, `JumpDistance`) | `--stream-distance`, `--jump-distance`, `AnalyzeOptions` |
| `results.csv` always | `-o`, `--format csv\|json`, stdout default for a single map |
| Interactive Songs-path prompt | `<PATH>` argument |
| `data/data.csv` cache | `--cache <FILE>`; statistics-only, content-addressed |

Behaviour changes that fix 0.9.x bugs:

* `StreamDistance`/`JumpDistance` are actually used (0.9.x had them
  hard-coded).
* `BeatmapSetID` is parsed (0.9.x always wrote an empty column).
* BPM is duration-weighted; playable length uses the last object's end time.
* Jump averages divide by the moves they measured.
* Paths are platform-neutral (no hard-coded `data\...`).
* Deleted files are no longer resurrected from a cache: the output is built
  from the files found in the current run.

The legacy 0.9.x implementation and its architecture notes are preserved in
[`docs/legacy-0.9.2-analysis.md`](docs/legacy-0.9.2-analysis.md).

---

## Development

```bash
cargo test --all-targets
cargo run -- tests/fixtures --all-modes --columns Title,Stars,Streams,JI
```

See [`ARCHITECTURE.md`](ARCHITECTURE.md) for the crate layout and design
decisions and [`CHANGELOG.md`](CHANGELOG.md) for the version history.

## Credits

* [rosu-pp](https://github.com/MaxOhn/rosu-pp) — `.osu` parsing, difficulty and
  performance calculation.
* Algorithm and original 0.9.x application: Raregendary.

## Licence

[MIT](LICENSE)

