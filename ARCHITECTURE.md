# Architecture

This document describes the 0.10 library + CLI. The previous application's
analysis is preserved in [`docs/legacy-0.9.2-analysis.md`](docs/legacy-0.9.2-analysis.md).

## Goals

1. **One implementation of the statistics.** A GUI, a CLI, a bot and a service
   must all produce identical numbers for the same file and options.
2. **A small, stable data API.** Consumers receive plain structs (serde
   compatible) and never need `rosu-pp` types or internal modules.
3. **Correct configurability.** Every option that exists changes the result and
   participates in the cache key.
4. **Predictable performance.** Files are hashed and parsed once per run, the
   batch scan is parallel, and the cache stores only computed statistics.
5. **Cross-platform.** No hard-coded separators, no Windows-only runtime
   behaviour, no global mutable state in the library.

Non-goals: audio processing, beatmap editing, osu! memory reading and UI code.
Those belong to the application that consumes the library.

---

## Crate layout

A single crate produces both targets:

```text
Cargo.toml
├─ [lib]  simple_beatmap_analyzer   (src/lib.rs)
└─ [[bin]] sba                      (src/bin/sba.rs)

src/
├─ lib.rs                 public API, re-exports, crate documentation
├─ analysis.rs            Analysis type + analyze_file/analyze_bytes
├─ error.rs               Error/Result (no panics on user input)
├─ options.rs             AnalyzeOptions, defaults, validation, cache fingerprint
├─ cache.rs               content-addressed statistics cache (bincode)
├─ scan.rs                find_beatmaps + parallel analyze_paths
├─ csv.rs                 Column enum, CSV writer
└─ beatmap/
   ├─ mod.rs              parsing entry point, re-exports
   ├─ metadata.rs         [General]/[Metadata] parser + Mode
   ├─ timing.rs           BPM, length, object counts
   ├─ difficulty.rs       NM/DT/HR stars, pp, modded attributes
   └─ patterns.rs         1/4, 1/2 and 1/3 pattern detection
```

The CLI depends only on the public crate API; it contains no statistics code.
Removing the binary target leaves a fully usable library.

---

## Layering

```text
             ┌─────────────────────────────────────────────┐
             │ CLI (src/bin/sba.rs)                        │
             │ args · progress · CSV/JSON · cache saving   │
             └───────────────▲─────────────────────────────┘
                             │ public API
             ┌───────────────┴─────────────────────────────┐
             │ scan.rs        batch, rayon, cache lookup   │
             │ csv.rs         column selection + writer    │
             │ cache.rs       persistence                  │
             └───────────────▲─────────────────────────────┘
                             │
             ┌───────────────┴─────────────────────────────┐
             │ analysis.rs    orchestration                │
             │  beatmap/{metadata,timing,difficulty,       │
             │           patterns}                         │
             │  options.rs    validation + fingerprint     │
             └───────────────▲─────────────────────────────┘
                             │
             ┌───────────────┴─────────────────────────────┐
             │ rosu-pp 0.9.5  (.osu parsing, stars, pp)    │
             └─────────────────────────────────────────────┘
```

Dependency direction is strictly downwards. `beatmap::*` never touches the
filesystem; `analysis` reads bytes; `scan` owns files and threads; the CLI owns
process concerns (stdout, exit code).

---

## Public data contract

```
Analysis {
  md5: String,                  // content hash, cache identity
  analyzer_version: u32,        // algorithm version
  metadata: BeatmapMetadata,    // title/artist/IDs/mode/tags
  timing: TimingStats,          // BPM, length, object counts
  difficulty: DifficultyStats,  // stars, pp, AR/OD/CS/HP (+ DT/HR)
  patterns: Option<PatternStats>
}
```

* `Analysis` and every nested type derive `Serialize` + `Deserialize`, so they
  can be cached, sent over a channel or converted to JSON without adapters.
* `patterns` is `None` for non-standard modes and maps with fewer than four hit
  objects. Consumers that prefer numbers can call
  `Analysis::patterns_or_default()`.
* Counts are the source of truth; percentages are methods
  (`stream_pct()`, `jump_pct()`, …). This lets a GUI show "12 notes" while the
  CSV shows "24.00%".
* `PatternStats` records the thresholds (`stream_distance`, `jump_distance`)
  that produced the numbers, so cached/exported rows are self-describing.

`FileAnalysis { path, analysis }` is the batch row type: the path lives beside
the analysis, never inside it, so moving a file does not invalidate the cache
value.

---

## Analysis pipeline

```
analyze_file(path)
  └─ fs::read(path)                     one read per file
       └─ analyze_bytes(bytes)
            ├─ options.validate()       reject NaN / out-of-range values
            ├─ md5_hex(bytes)           cache identity
            ├─ Beatmap::from_bytes()    rosu-pp parse (once)
            ├─ parse_metadata(text)     independent, order-insensitive
            ├─ TimingStats::calculate   bpm(), timing points, object counts
            ├─ DifficultyStats::calculate
            │    ├─ pp/accuracy         NM
            │    ├─ pp/mods(64)         DT
            │    ├─ pp/mods(16)         HR
            │    └─ attributes().mods   DT_AR/DT_OD, HR_AR/…/HR_HP
            └─ patterns (standard only)
                 └─ detect_patterns(hit_objects, timing_points, cs,
                                    options.stream_distance,
                                    options.jump_distance)
```

`analyze_bytes` is the only place that orchestrates modules. It returns an
`Analysis` and never performs I/O beyond the input slice.

### Metadata

`parse_metadata` is a small section-aware parser for `[General]` and
`[Metadata]`. It is deliberately independent of section order, tolerates a UTF-8
BOM, splits on the **first** colon (so `Title:A: B` stays intact), keeps Unicode
title/artist, parses both `BeatmapID` and `BeatmapSetID`, and never panics —
malformed IDs become `0`. The parsed mode is overwritten with rosu-pp's value in
`analysis.rs`, because rosu-pp is authoritative for the playable part.

### Difficulty

All calculations delegate to `rosu-pp`:

* `map.pp().accuracy(acc)` → NM pp/stars/max combo,
* `.mods(64)` → DT, `.mods(16)` → HR,
* `map.attributes().mods(..).build()` → DT_AR/DT_OD and HR_AR/HR_OD/HR_CS/HR_HP.

Three performance calculations per map are required because DT/HR change the
difficulty attributes. They are the dominant cost of a full scan.

### Patterns

`detect_patterns` is a single O(n) pass over the hit objects. For each object it
computes:

* `distance = hypot(dx, dy) - circle_diameter`, where the diameter is
  `round(54.4 - 4.48*cs, 3) * 2`;
* the current beat length from the timing points;
* `ratio = dt / (beat_len / 4)`.

It then advances three independent run counters (1/4, 1/2, 1/3) and flushes them
when a note no longer qualifies. Details and bucket definitions are in the
[README](README.md#what-is-measured); the invariants are:

* thresholds are taken from `AnalyzeOptions` — no constants are hidden in the
  detector;
* every run, including one that touches the last object, is flushed exactly
  once;
* percentages divide by `hit_objects.len()`, so they always sum consistently;
* jump runs are movement-based: the object before the first jump move is part
  of the run's note count, while averages divide by the measured moves;
* only head positions are used; slider paths are not walked.

---

## Cache

```
key = md5(file bytes) : fingerprint(options) : ANALYZER_VERSION
value = Analysis
file = bincode(CacheFile { version, entries: HashMap<String, Analysis> })
```

Design points:

* **Content addressed.** Renaming/moving a file keeps the entry. Editing the map
  changes its MD5 and therefore misses.
* **Option-aware.** `AnalyzeOptions::fingerprint()` hashes the thresholds,
  accuracy and `ANALYZER_VERSION`; a changed threshold cannot return stale data.
* **Small.** Only scalar statistics are stored (roughly a few hundred bytes per
  map), never parsed beatmaps or hit objects.
* **Atomic writes.** The cache is written to a temporary file and renamed, so a
  crash cannot leave a half-written cache.
* **Fail-soft in the CLI.** A corrupt/incompatible cache produces a warning and
  an empty cache; the library returns `Error::Cache` so callers can choose.

---

## Concurrency

`scan::analyze_paths_with_progress` uses Rayon over the file list. Each task:

1. reads its file,
2. hashes it,
3. checks the read-only cache,
4. on a miss, parses and analyzes.

There is **no global lock around parsing** (the 0.9.x `RwLock` bug). The cache is
read through `&Cache` inside the parallel phase and updated sequentially
afterwards, so workers never block each other. Results are sorted by path before
being returned, making CSV/JSON output deterministic regardless of scheduling.

Memory stays bounded: raw bytes are dropped at the end of each task and only the
small `Analysis` values are retained.

---

## CLI

`src/bin/sba.rs` is intentionally a thin layer:

* hand-written argument parser (no `clap`) with `--flag value` and
  `--flag=value`, usage errors and exit code `2`;
* progress bar only for multi-file scans and only when enabled;
* CSV or JSON to stdout/file (`-` means stdout);
* default output: stdout for one file, `results.csv` for a folder;
* summaries and warnings go to **stderr**, keeping stdout clean for pipelines;
* broken pipes (e.g. `| head`) are treated as success;
* `--all-modes` filters the output after analysis (pattern stats are
  standard-only either way);
* cache load/save is optional and fail-soft.

The library has no CLI concerns: no printing, no `std::process`, no global
configuration.


---

## Errors

* One `Error` enum with `Display`/`Error` implementations; no `anyhow`/`thiserror`.
* I/O and parse failures carry enough context for a UI to show "this file is
  broken" instead of aborting a whole scan.
* A failed file becomes a `FileError` in `BatchOutcome`; the rest of the batch
  continues.
* Invalid options are rejected before any file is read (`Error::InvalidOptions`).
* `detect_patterns` returns `None` (not an error) for maps that cannot have
  pattern stats; this is a normal state, not a failure.

---

## Testing

| Layer | Tests |
|---|---|
| Metadata | Section order, BOM, colon in values, Unicode, both IDs, tags |
| Patterns | Expected counts/percentages on a 50-note fixture, 1/3 snaps, option sensitivity |
| Options | Threshold changes alter streams/jumps; invalid values rejected |
| Modes | Taiko map has no patterns; short map has no patterns |
| Analysis | `analyze_bytes` == `analyze_file`; MD5 length |
| CSV | Header, selected columns, empty pattern cells |
| Cache | Round-trip, option isolation |
| Scan | Finds all fixtures, deterministic order, no errors |
| CLI | CSV/JSON output, file output, cache reuse, exit codes, `--list-columns` |

Fixtures under `tests/fixtures/` are tiny, hand-built `.osu` files that target
specific behaviour (stream, jumps, 1/3, non-standard, too short). They are
deterministic, so assertions on counts and indices are exact.

```bash
cargo test --all-targets
```

---

## Versioning & compatibility

* Crate version: semver. 0.10.0 is a breaking rewrite of the 0.9.x API (which
  was not a library at all).
* `ANALYZER_VERSION` is bumped whenever the algorithm changes; it is part of
  every cache key and stored in every `Analysis`.
* CSV headers keep the 0.9.x names where the metric is the same (`Stars`,
  `DT_Stars`, `NM_99`, `Bursts`, …), and add explicit new ones (`Path`,
  `MaxCombo`, `BPM_Min`, `Objects`, `OneThird`, `JumpNotes`).
* `Cargo.lock` is committed for reproducible release builds.

---

## Integrating with the GUI

A GUI should depend on the library and treat it as the single source of truth:

```rust
let analysis = simple_beatmap_analyzer::analyze_file(path, &options)?;
ui.stars = analysis.difficulty.stars_nm;
ui.patterns = analysis.patterns_or_default();
```

Guidelines:

* Keep user-edited AR/OD/CS/HP and audio transformations in the GUI; this
  library reports the **beatmap as written** plus NM/DT/HR values.
* If the GUI already has a parsed `rosu_pp::Beatmap` in memory, it can avoid a
  second parse by calling the lower-level modules directly, but the supported
  path is `analyze_file`/`analyze_bytes` (parse cost is small compared to the
  three performance calculations).
* Do not copy statistics code into the GUI. If an additional metric is needed,
  add it to `PatternStats`/`TimingStats` here and bump `ANALYZER_VERSION`.

### Switching the parser/difficulty backend

`rosu-pp` is used in exactly three places: `beatmap::parse_beatmap`,
`beatmap/difficulty.rs`, and the `TimingStats`/`PatternStats` inputs (hit
objects, timing points, `cs`, `mode`). Swapping to the `rosu-pp-gemini` fork
therefore means:

1. update `Cargo.toml`;
2. adapt the import paths in those modules (`rosu_pp::Beatmap`,
   `HitObject`, `TimingPoint`, difficulty/performance builders);
3. keep the `Analysis` types unchanged.

No other module depends on `rosu-pp`, so the CLI, cache and CSV layers need no
changes.

---

## Why this shape

| Decision | Reason |
|---|---|
| Single crate, lib + bin | A consumer needs one dependency; the binary is a thin wrapper. |
| Plain structs instead of exposing `rosu-pp` | Stable API, serde for free, no parser-version leakage into a GUI. |
| Counts + percentage methods | One representation supports both "42 notes" and "42.00%" without duplicating fields. |
| Path outside `Analysis` | Content-addressed cache: moving a file must not invalidate it. |
| Statistics-only cache | A parsed beatmap cache grows with the library; statistics do not. |
| Rayon + read-only cache lookup | Parallel parsing without locks or serialised inserts. |
| `Option<PatternStats>` | Non-standard maps are still useful (stars/pp/metadata); pattern data is standard-specific. |
| Hand-written CLI parser | One fewer dependency, faster builds, and the option set is small. |

