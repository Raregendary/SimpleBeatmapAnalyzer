# Changelog

All notable changes to this project are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project follows
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.10.0] - 2026-10-26

The project becomes a **library + CLI**. The analysis core is reusable by other
programs (the planned GUI), and the binary is a thin wrapper around it.

### Added

* Library crate `simple_beatmap_analyzer` with a typed, serde-compatible API.
* `AnalyzeOptions` (`stream_distance`, `jump_distance`, `accuracy`) with
  validation, defaults and a stable cache fingerprint.
* `Analysis` containing `BeatmapMetadata`, `TimingStats`, `DifficultyStats` and
  optional `PatternStats`.
* Pattern statistics: doubles, triples, quads, bursts, streams, deathstreams,
  longest stream, 100+ streams, short/mid/long jumps, total jump notes, average
  jump distance/speed, 1/3 snap notes, SI, JI and FCDBI.
* Batch scanning (`find_beatmaps`, `analyze_paths`) with a deterministic order
  and Rayon parallelism.
* Content-addressed, statistics-only `Cache` keyed by
  `md5 + options + ANALYZER_VERSION`.
* CSV export with selectable `Column`s and a JSON output format.
* `sba` CLI: files or folders, `--output`, `--format`, `--columns`,
  `--stream-distance`, `--jump-distance`, `--accuracy`, `--cache`,
  `--all-modes`, `--no-progress`, `--list-columns`.
* 19 tests covering metadata, patterns, options, modes, CSV, cache, scanning and
  the CLI, with synthetic `.osu` fixtures.
* `ARCHITECTURE.md` and this changelog; the old analysis moved to
  `docs/legacy-0.9.2-analysis.md`.

### Changed

* Package renamed to `simple-beatmap-analyzer`; the binary is now `sba`.
* BPM is the duration-weighted most common BPM (was: most common per note).
* `PlayableLength` measures first object start to last object end (was: last
  object start).
* Jump runs are counted from 2 notes upwards (was: 3+) and averages divide by
  the measured moves (was: an inflated counter).
* CSV output is built from the files found in the current run (no stale rows for
  deleted maps) and includes `Path`, `Mode`, `MaxCombo`, `BPM_Min`, `BPM_Max`,
  `Objects`, `JumpNotes` and `OneThird`.
* CSV rows are sorted by path.

### Fixed

* `StreamDistance`/`JumpDistance` now affect the result; they were hard-coded to
  `16.0`/`110.0` in 0.9.x.
* `BeatmapSetID` is parsed; the column was always empty in 0.9.x.
* Parsing no longer depends on metadata section order and no longer loses values
  containing `:`.
* The Rayon loop no longer holds a global write lock while parsing.
* Platform-neutral paths replace the hard-coded `data\...` separators.
* Corrupt or incompatible caches fail soft in the CLI instead of aborting.
* Broken stdout pipes (e.g. `sba … | head`) are treated as success.

### Removed

* `columns_config.txt` and `options_config.txt` (replaced by CLI flags / the
  typed API).
* The parsed-beatmap cache (`SavableFullData`, `Beatmap_Serialized`, bincode
  dump of every hit object).
* The interactive Songs-path prompt.
* `winres`/`build.rs` and the executable icon.
* `strum`/`strum_macros` (no longer needed).

### Breaking

* The crate, binary, module layout and API are new; 0.9.x was an application,
  not a library.
* CSV column set and some metric semantics changed (see *Changed*). Consumers
  that need the exact 0.9.x layout can select the old columns explicitly with
  `--columns`.

## [0.9.2] - 2023-09-15

* Added creator names, osu! web/direct links, average jump distance and speed.
* Fixed average jump distance/speed and several data-save bugs.

## [0.9.1] - 2023-09-10

* Added `columns_config.txt`, stream length and 100+ stream counters.
* Count the last pattern of a map.
* Cache computed maps instead of recomputing on every start.

## [0.9.0] - 2023-09-07

* First release: scan a Songs folder, compute NM/DT/HR stars and pp, classify
  streams/bursts/jumps, write `results.csv`.
