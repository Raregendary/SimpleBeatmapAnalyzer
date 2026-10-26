# SimpleBeatmapAnalyzer — Legacy 0.9.x Architecture & Behaviour Analysis

> **Historical document.** This describes the 0.9.x application (a CSV-only
> Windows binary). It is kept for reference, including the list of bugs that
> the 0.10 library rewrite fixed. For the current library + CLI, see
> [`../ARCHITECTURE.md`](../ARCHITECTURE.md) and
> [`../CHANGELOG.md`](../CHANGELOG.md).


Repository snapshot: `master` @ `c404834` (2023-09-16, "temporary changes save").
Crate version: `0.9.2` (the last *release* commit is `49afd89`, 2023-09-15).

Everything below was checked against the source and, where possible, by compiling and running the
program (see [How this was verified](#how-this-was-verified)).

---

## TL;DR

* It is a single-binary Rust console tool for Windows that walks an osu! `Songs` folder, parses every
  `.osu` file with [`rosu-pp`](https://github.com/MaxOhn/rosu-pp), runs a hand-written pattern detector
  on the hit objects, and dumps one CSV row per beatmap (`results.csv`).
* The architecture is a straightforward batch pipeline with three ideas layered on top:
  1. **MD5 content hash as the primary key**, so already-computed maps are skipped on later runs.
  2. **A persistent cache** in a `data` folder (`data/data.csv` in v0.9.2, a bincode dump in HEAD).
  3. **Enum-driven column configuration**, so CSV columns/order can be changed from a text file.
* `master` **does not compile**. The last commit is an unfinished v0.9.3 refactor that tries to store the
  fully parsed beatmap in the cache; it calls `SortedVec::with_capacity`/`push`, which are `pub(crate)`
  in `rosu-pp 0.9.5`. The last working state is `49afd89` / `77419ca` (v0.9.2).
* Several advertised features are not actually wired up: the stream/jump spacing options are ignored
  (hard-coded `16.0` / `110.0`), and `MapSetID` is always written empty because of a dead branch in the
  metadata reader.

---

## What the program does (user view)

1. Start `SimpleBeatmapAnalyzer.exe`.
2. It creates/loads `columns_config.txt` and `options_config.txt` next to the executable.
3. If `FilePath` in `options_config.txt` is empty it asks for the osu! Songs folder
   (e.g. `D:\osu\Songs`); otherwise it uses the configured path.
4. It recursively finds every `*.osu` file (`jwalk`).
5. For every file it computes an MD5 of the file bytes, skips it when that MD5 is already cached,
   otherwise parses it, filters to osu!standard, calculates stars/PP for NM/DT/HR at 99% accuracy and
   runs the stream/jump detector.
6. It writes
   * `results.csv` — the configurable table (40 columns by default), one row per cached map,
   * a cache in `./data/` — `data.csv` (v0.9.2) or `data_v0-9-3.bin` (HEAD),
   * a copy of `options_config.txt` into `./data/`, used to detect "settings changed" on the next run.
7. Waits for Enter before exiting (`end()` in `src/main.rs:182`), because the console window would
   otherwise close immediately when double-clicked.

`results.csv` is meant to be filtered/sorted in Excel: BPM, stars, pattern percentages, DT/HR-adjusted
values, `SI` (stream index), `JI` (jump index), `FCDBI` (experimental finger-control index).

---

## Module map

| File | Lines | Responsibility |
|---|---:|---|
| `src/main.rs` | 187 | Wiring: startup, config bootstrap, file discovery, Rayon processing loop, cache merge, save call, exit prompt. |
| `src/osufile_reader.rs` | 51 | Hand-rolled line scanner that pulls `Title`, `Creator`, `Version`, `BeatmapID` (and, in theory, `BeatmapSetID`) out of the `.osu` text. |
| `src/beatmap_stat_processor.rs` | 241 | The core algorithm: converts hit objects + timing points into pattern counts and indexes (`SongParams`). |
| `src/full_data_struct.rs` | 744 | Row/cache model: `FullData` (v0.9.2 cache row), `SavableFullData` (HEAD cache row), the CSV column enum + `FromStr`, and ~415 lines of serde-friendly mirror types for `rosu_pp`'s parsed beatmap plus conversions both ways. |
| `src/columns_config_manager.rs` | 58 | Reads/writes `columns_config.txt` (`ColumnName=0|1`), returns an ordered `Vec<FullDataEnum>`. |
| `src/options_config_manager.rs` | 105 | Reads/writes `options_config.txt` (`FilePath`, `StreamDistance`, `JumpDistance`) and compares old vs new settings. |
| `src/savedata_manager.rs` | 100 | MD5 hashing, cache (de)serialisation, CSV writing, `data/` folder handling. |
| `build.rs` | 8 | Embeds `src/sba_icon.ico` into the Windows executable via `winres` (no-op on other targets). |
| `.github/workflows/rust.yml` | 22 | CI: `cargo build` + `cargo test` on `ubuntu-latest` for pushes/PRs to `master`. |

Dependencies (`Cargo.toml`): `rosu-pp 0.9.5` (parsing, difficulty, PP), `rayon` (data parallelism),
`jwalk` (parallel directory walk), `csv` (output), `bincode`/`serde` (cache), `md-5` (identity),
`indicatif` (progress bar), `strum`/`strum_macros` (enum iteration/`FromStr`), `winres` (icon).

---

## Execution pipeline

```
main()                                    src/main.rs:156
 ├─ init_columns_config("columns_config.txt")      -> Vec<FullDataEnum>
 ├─ init_options_config("options_config.txt")      -> OptionsConfig
 ├─ compare_options_config(current, data/options_config.txt)
 ├─ load_bin_data() (HEAD)  /  read_main_data_csv() (v0.9.2)
 │                     └─ HashMap<md5, SavableFullData>   (or Vec<FullData>)
 ├─ (prompt for Songs path if FilePath empty)
 └─ process_beatmaps_new()
     ├─ find_osu_files6(path)              recursive *.osu scan, Vec<String>
     ├─ data_processor_new(files, cache)   rayon par_iter over files
     │   ├─ md5 = calculate_md5(file)      content hash = cache key
     │   ├─ if cached:
     │   │    └─ update_mut_savable_fulldata()  (HEAD only; recompute stats if options changed)
     │   ├─ else:
     │   │    └─ process_new_beatmap()
     │   │        ├─ Beatmap::from_path()          rosu-pp parse
     │   │        ├─ map.mode != Osu -> cache empty row, skip stats
     │   │        ├─ process_stats(timing_points, hit_objects, cs, thresholds)
     │   │        ├─ read_osu_file()               title/creator/version/IDs
     │   │        └─ map.pp().accuracy(99).mods(64|16).calculate()   NM/DT/HR stars+PP
     │   └─ insert into cache under md5
     └─ data_save_manager()
         ├─ save cache (bincode: HEAD, CSV: v0.9.2)
         ├─ copy options_config.txt -> data/options_config.txt   (change detection)
         └─ write results.csv (header from columns config, rows filtered to osu!standard)
```

### Startup & configuration
`main()` loads the column list first, then the options, then compares the current options file with the
copy in `data/`. On the very first run the copy does not exist, so the comparison falls back to defaults
and any customised settings are treated as "changed" (full recompute is the safe path).

### Scanning
`find_osu_files6()` (`src/main.rs:31`) uses `jwalk` and collects every file whose extension is `osu`.
It returns early on the first walk error (so one unreadable directory aborts the scan). `"69"` as the
path is a leftover developer shortcut for `D:\osu\Songs` (`src/main.rs:34`).

### Per-map processing
`process_new_beatmap()` (`src/main.rs:53`):

* `Beatmap::from_path` → `rosu_pp::Beatmap`.
* Non-standard modes are stored as an empty row (only the serialized beatmap + md5) in HEAD, or skipped
  in v0.9.2; either way they never reach `results.csv`.
* `process_stats()` gets raw `timing_points`, all `hit_objects`, `cs`, and the two spacing thresholds.
* Metadata is read a second time from the raw text by `read_osu_file()` — rosu-pp does not expose
  `Title`/`Creator`/`Version`/`BeatmapID`.
* Three full difficulty/PP calculations: `.pp().accuracy(99).calculate()` for NM, `.mods(64)` (DT),
  `.mods(16)` (HR).
* Any error (short map, no timing points, parse failure) makes the whole map silently disappear from the
  dataset — there is no "unprocessable" row, and it is retried on every run.

### Cache
* **v0.9.2**: `data/data.csv` is the cache — it serialises the flat `FullData` rows via `serde`/`csv`.
  Cache = one row of ~40 scalar values per map (~300 bytes in my test). Options change ⇒
  `already_processed` is empty ⇒ every file is parsed again.
* **HEAD (WIP)**: `data/data_v0-9-3.bin` is a `bincode` dump of `HashMap<md5, SavableFullData>`.
  Each value contains the **entire parsed beatmap** (all hit objects, slider control points, sounds,
  timing/difficulty/effect points, breaks) plus the computed stats. That is what enables the new
  "recompute stats for already-cached maps without re-reading the .osu file" feature
  (`update_mut_savable_fulldata`, `src/main.rs:100`), but it also makes the cache and RAM usage grow with
  the size of the whole library, not with the number of maps.

### CSV output
`write_results_csv()` (`src/savedata_manager.rs:67`) writes the header from the `FullDataEnum` names and
then one row per map. Every cell is produced by `SavableFullData::get_string` (HEAD) /
`FullData::get_string` (v0.9.2) — a single `match` over the column enum. Rows come from a `HashMap` in
HEAD, so **row order is not stable** between runs (v0.9.2 used a `Vec`, so its order was stable).

---

## Data model

```
FullDataEnum  (src/full_data_struct.rs:163)   ~40 variants == CSV columns
   │  Debug name = CSV header, FromStr = columns_config.txt key
   ▼
SavableFullData {                             HEAD cache value / CSV row
    beatmap: Beatmap_Serialized,              full parsed map (mirror of rosu_pp::Beatmap)
    title, version, beatmap_id, beatmap_set_id, creator,
    nm_stars, dt_stars, hr_stars, nm_pp, dt_pp, hr_pp,
    song: SongParams,                         pattern metrics
    md5: String                               identity
}
FullData { ...same scalars, flattened... }    v0.9.2 cache row; dead code in HEAD
```

* `SongParams` (`beatmap_stat_processor.rs:188`) holds the 18 computed features:
  `playable_length`, `bpm`, `doubles`, `triples`, `quads`, `bursts`, `streams`, `deathstreams`,
  `short_jumps`, `mid_jumps`, `long_jumps`, `fcdbi`, `si`, `ji`, `longest_stream`, `streams100`,
  `avg_jump_distance`, `avg_jump_speed`.
* `Beatmap_Serialized` (`full_data_struct.rs:330`) is a serde-capable mirror of `rosu_pp::Beatmap`
  (`HitObject_Serialized`, `Pos2_Serialized`, `Slider`/`Spinner`/`Hold` kinds,
  `PathControlPoint_Serialized`, timing/difficulty/effect points, breaks).
  Two conversion directions live in the same file: `normal_to_serialized_beatmap` (used) and
  `serialized_to_normal_beatmap` (used for cache recompute, and the part that no longer compiles).
* `FullData` and `SavableFullData` both implement a `get_string` trait with duplicated formatting logic.
  In HEAD the old `FullData` path is no longer used by the pipeline but is still compiled and imported
  (`savedata_manager.rs:14`).

---

## The pattern-detection algorithm

`process_stats(sorted_bpms, xyt, cs, stream_spacing, jump_spacing)` (`beatmap_stat_processor.rs:6`).
It is a single sequential pass over the hit objects (only head positions; slider bodies are ignored, as
the header comment states).

1. **Circle diameter** from CS: `r2 = round(54.4 - 4.48*cs, 3) * 2`. For CS 4 → `72.96` px.
2. **Edge-to-edge distance** between consecutive objects: `d_r = hypot(dx, dy) - r2`.
3. **Current BPM** by advancing a `bpm_index` over the sorted (uninherited) timing points.
   `beat_len` is milliseconds per beat (`500 ms` @ 120 BPM).
4. **Snap ratio**: `ratio = (t - t_prev) / (0.25 * beat_len)`.
   `ratio ≈ 1` ⇒ 1/4 snap, `ratio ≈ 2` ⇒ 1/2 snap (tolerance ±0.1).
5. **1/4 (stream-side) run** — qualifies when `d_r <= 16.0` **and** `0.9 < ratio < 1.1`.
   The run counter is flushed when a note no longer qualifies; the run length classifies the pattern:

   | run length | bucket |
   |---:|---|
   | 2 | `doubles += 2` |
   | 3 | `triples += 3`, `bursts += 3` |
   | 4 | `quads += 4`, `bursts += 4` |
   | 5–11 | `bursts += n` |
   | 12–32 | `streams += n` |
   | ≥ 33 | `deathstreams += n` (+ `streams100` when ≥ 100) |

   Also tracks `longest_stream`. A run still open at the end of the map is flushed afterwards
   (`:120–143`, the v0.9.1 "last pattern" fix).
6. **1/2 (jump-side) run** — qualifies when `d_r > 110.0` **and** `1.9 < ratio < 2.1`.
   Accumulates distance and `1000*d_r/dt` (px/s) for the averages. Flushed when the next note is not a
   qualifying jump and at least 2 were accumulated; run length → `short_jumps` (≤ 11),
   `mid_jumps` (12–31), `long_jumps` (≥ 32).
7. **Aggregation**: all percentages are `count / total_hit_objects * 100` (note-based, not time-based).
   * `jump_value  = short + 1.5*mid + 2*long`
   * `stream_value = bursts + 1.5*streams + 2*deathstreams`
   * `SI = (stream_value - jump_value - 0.5*doubles) / notes`
   * `JI = (jump_value - stream_value - 0.5*doubles) / notes`
   * `FCDBI = (bursts - triples - quads + 1.75*doubles + 1.5*triples + 1.75*quads
               - jump_value - 1.35*streams - 1.7*deathstreams) / notes` (experimental; can be negative)
   * `playable_length` = (last object time − first object time) / 1000, rounded to seconds
   * `bpm` = BPM of the most frequent `beat_len` (per hit object, not per unit of time).

### Observed sample (synthetic 50-note map: 20-note 1/4 stream, 20 jumps, 1 double, 1 triple, 1 five-note burst)

```
BPM=120  Bursts=16.00  Streams=40.00  ShortJumps=0.00  MidJumps=42.00  Longjumps=0.00
Doubles=4.00  Triples=6.00  SI=0.11  JI=-0.15  FCDBI=-0.91  PlayableLength=11
LongestStream=20  AvgJumpsDistance=269  AvgJumpsSpeed=1078
```

These match the formulas above exactly (e.g. `bursts = (3+5)/50*100 = 16`,
`SI = (8 + 30 − 31.5 − 1)/50 = 0.11`). Two details worth knowing:

* **`MidJumps = 42.00`** is 21 hits for a 20-note jump run: the transition *into* the run is itself a
  qualifying jump move, and the flush adds "+1 for the first note" on top, so jump runs are inflated by
  one when the preceding note also forms a jump move. Stream runs are not inflated the same way because
  their first note is the one that starts the run.
* **`AvgJumpsDistance = 269`** while the mean of the 20 measured jump moves is `283.2`:
  `avarage_jump_distance_counter / jumps_counter` (`:158–159`) divides by a count that includes the
  never-measured "+1" note, understating the average (same for `avg_jump_speed`).

---

## Configuration system

**`columns_config.txt`** (`columns_config_manager.rs`)
```
Title=1
DifName=1
...
MD5        (never written; always appended last by read_config)
```
`1` keeps a column, `0` hides it; line order = CSV column order. `MD5` is forced in so the cache keeps a
stable key, and `FromStr` deliberately rejects it (`full_data_struct.rs:250`).

**`options_config.txt`** (`options_config_manager.rs`)
```
FilePath=            # default Songs folder, skips the interactive prompt when set
StreamDistance=16.0  # expected max edge-to-edge spacing for 1/4 patterns
JumpDistance=110.0   # expected min edge-to-edge spacing for 1/2 patterns
```
The file is created with comments and defaults if missing. Unknown/invalid values silently fall back to
the defaults. The `data/options_config.txt` copy is compared against the live file on startup to decide
whether cached results are still valid (`compare_options_config`, tolerance `0.01`).

> **The two distance values have no effect on the results** — see finding F2.

---

## Findings

Severity: 🔴 breaks the product, 🟠 wrong output / silent data loss, 🟡 maintenance / performance,
🔵 cosmetic.

| # | Sev | Finding | Evidence |
|---|---|---|---|
| F1 | 🔴 | `master` does not compile | `cargo check`: 6 × `E0624` |
| F2 | 🔴 | `StreamDistance` / `JumpDistance` are ignored | thresholds hard-coded; experiment |
| F3 | 🟠 | `MapSetID` is always empty | dead branch; observed blank cell |
| F4 | 🟡 | Rayon parallelism is serialised by a global write lock (HEAD) | `main.rs:120` |
| F5 | 🟡 | HEAD cache stores whole parsed beatmaps → big `.bin` and RAM | data model |
| F6 | 🟠 | HEAD can fail to write anything on a fresh install (Windows) | call order |
| F7 | 🟠 | Deleted/moved maps stay in `results.csv` forever | experiment |
| F8 | 🟠 | Duplicate `.osu` files produce duplicate rows (v0.9.2) | experiment |
| F9 | 🟡 | Average jump metrics divide by an inflated counter | experiment |
| F10 | 🟡 | Hard-coded `data\...` separators → Windows-only at runtime | experiment |
| F11 | 🟡 | No tests at all; CI only builds | `cargo test`: 0 tests |
| F12 | 🔵 | "beatmaps per second" metric is meaningless for cached runs | `main.rs:142` |
| F13 | 🔵 | Duplicated row structs / fragile enum-last assumption | code |

### F1 — the current checkout cannot be built

```
$ cargo check --all-targets
error[E0624]: associated function `with_capacity` is private   (×3)
error[E0624]: method `push` is private                         (×3)
  --> src/full_data_struct.rs:561,563,573,575,587,589
  rosu-pp-0.9.5/src/util/sorted_vec.rs:34  pub(crate) fn push
  rosu-pp-0.9.5/src/util/sorted_vec.rs:98  pub(crate) fn with_capacity
EXIT: 101
```

The last commit replaced the flat v0.9.2 cache with "cache the whole beatmap, rebuild it later". The
rebuild path (`timing_points_to_normal`, `difficulty_points_to_normal`, `effect_points_to_normal`, used by
`serialized_to_normal_beatmap`) constructs `rosu_pp::SortedVec` values — but that type exposes no public
constructor or `push` in `rosu-pp 0.9.5`. It is therefore not fixable locally without either an upstream
API that allows building `SortedVec`, or a different design that never rebuilds a `Beatmap` from scratch.

Practical consequence: **the shipped/working state is v0.9.2 (`49afd89` / `77419ca`)**; `master` is an
unreleased, half-finished v0.9.3 refactor. `Cargo.toml` still says `0.9.2` while the cache file is named
`data_v0-9-3.bin`, so CI on `master` cannot pass.

Related: the compiler reports **48 warnings** on HEAD (unused imports, dead code, unused variables) vs.
23 on v0.9.2, including `unused variable: stream_spacing` and `unused variable: jump_spacing` — the
direct proof of F2.

### F2 — the spacing options are dead settings

`process_stats` takes `stream_spacing`/`jump_spacing`, but the checks use literals:

```rust
// src/beatmap_stat_processor.rs:67
if d_r - 16.0 <= 0.0 && time_deviser_ratio > 0.9 && time_deviser_ratio < 1.1 {
// src/beatmap_stat_processor.rs:95
if d_r - 110.0 > 0.0 && time_deviser_ratio > 1.9 && time_deviser_ratio < 2.1 {
```

Verified: I added a probe map whose 1/4 section has 37 px edge-to-edge spacing and whose 1/2 section has
150 px, then reran with `StreamDistance=100.0` / `JumpDistance=300.0`. The app detected the change and
recomputed all 4 maps ("Processed 4 …"), but the probe row was byte-identical:

```
defaults (16/110):  Streams=0.00  ShortJumps=33.33
100/300:            Streams=0.00  ShortJumps=33.33   <-- unchanged
```

So editing the options costs a full recomputation and changes nothing. (The README lists "expose
variables … for editing in the config" as an open TODO, while the program already advertises the file.)

### F3 — `MapSetID` is never parsed

`read_osu_file` (`src/osufile_reader.rs:39–47`):

```rust
if b4 && line.starts_with("Beatm") {      // matches "BeatmapID:"
    b4 = false;
    b5 = true;                            // b5 never read again
    beatmap_id = ...;
    continue;
}
if b4 && line.starts_with("Beatm") {      // unreachable: b4 is false now
    return (..., line.split(":").nth(1)...);   // "BeatmapSetID"
}
```

The second branch is dead, so `beatmap_set_id` is always `""`. Observed in every generated row:
`...,0.00,,20,...` — the `MapSetID` cell is empty even though the file contains `BeatmapSetID:6789`.
Also note the parser is positional (`b1..b5` gates) and `unwrap`s on `split(':')`, so unusual metadata
(a reordered section or a value without `:`) can panic, and titles containing `:` are truncated.

### F4 — HEAD's cache lock kills the parallelism

```rust
// src/main.rs:117-129
files.par_iter().for_each(|file| {
    let md5 = calculate_md5(file);
    let mut old_data = old_data.write().unwrap();   // exclusive lock ...
    if let Some(item) = old_data.get_mut(&md5) { ...update... }
    else { ...process_new_beatmap(...)... }          // ... held during parse + 3 PP calculations
});
```

The write guard lives for the whole closure, so only one Rayon worker can parse at a time. v0.9.2 used
`par_iter().fold(...)` with a per-thread `Vec` accumulator and was genuinely parallel; the WIP cache
rewrite accidentally made the hot path single-threaded, which contradicts the README's 2500 maps/s
"scales with cores" claim. Better: hash outside the lock, parse outside the lock, lock only for the
insert/lookup.

### F5 — the WIP cache is a full snapshot of every map

`SavableFullData` embeds `Beatmap_Serialized`, i.e. every hit object, every slider control point, every
sound byte, all timing/difficulty/effect points and breaks. For a real library (thousands of maps,
1–3k objects each) the bincode file and the in-memory `HashMap` grow by tens of KB per map — orders of
magnitude more than v0.9.2's ~300 B/map `data.csv`. *(Size estimate; the file could not be measured
because HEAD does not build.)* Recomputation only needs `hit_objects`, `timing_points` and `cs`, so a
cache of those three (or of the raw `.osu` bytes) would deliver the same feature at a fraction of the
cost.

### F6 — fresh install may save nothing (inferred)

`data_save_manager` (`savedata_manager.rs:55`) saves the cache **before** the CSV:

```rust
if let Ok(data_path) = save_bin_data(full_data) {      // File::create("data\\data_v0-9-3.bin")
    copy_options_config(...);                          // fs::copy -> data\...
    if let Ok(results_path) = write_results_csv(...)   // create_data_dir() happens in here
```

`create_data_dir()` is only called inside `write_results_csv` (`:68`). On a fresh Windows install with no
`data` folder, `File::create("data\\data_v0-9-3.bin")` fails with `NotFound`, so `data_save_manager`
returns `Err` and the user sees `FAILED TO SAVE CSV FILE!!!` — after the whole folder was processed.
v0.9.2 called `create_data_dir()` first (`write_main_data_csv`), so this is a regression introduced by
the WIP commit. (Inferred from the call order; not executed, because the Windows-specific path cannot be
reproduced here and HEAD does not build.)

### F7 / F8 — stale and duplicate rows

Verified with the v0.9.2 binary (HEAD shares the merge logic):

* Deleting `Songs/22222 Second/second.osu` and rerunning printed `Processed 0 …` and still emitted the
  `Second Song` row. The cache is only ever *merged*; entries are never invalidated when a file
  disappears or changes (changed content = new MD5 = a new row, old row stays).
* Two byte-identical `.osu` files are both processed on the first run, producing two identical rows.
  On later runs both are skipped (their MD5 is in the cache) but the duplicate rows remain.
  HEAD's `HashMap<md5, …>` fixes the first-run duplication but not the staleness.

### F9 — jump averages are slightly wrong

Covered in the algorithm section: `jumps_counter` is incremented by `run_length + 1` (`:102`, `:145`)
while the distance/speed accumulators only receive the qualifying moves, so averages are divided by one
extra unit per run (observed 269 vs. a true mean of 283.2 px).

### F10 — runtime paths are Windows-only

Hard-coded separators: `data\\{}` (`main.rs:162`, `options_config_manager.rs:15`,
`savedata_manager.rs:20`), `data\\data_v0-9-3.bin` (`savedata_manager.rs:90,97`). On Linux the app
"works" but writes files literally named `data\data.csv` and `data\options_config.txt` next to the
binary and leaves `data/` empty (observed). `PathBuf::push`/`join` would fix it. `build.rs`'s
`cfg!(target_os = "windows")` is a *runtime* check, so `winres` is still compiled on every platform.

### F11–F13 — maintenance items

* **No tests** anywhere (`grep '#[test]' → 0`). CI's `cargo test` runs zero tests, and because `master`
  does not compile, the workflow would currently fail at `cargo build`.
* **Throughput metric**: HEAD prints `old_data.len() / duration` (`main.rs:142`) — the *cache* size, not
  the work done — and `max(1)` turns sub-second runs into "3 beatmaps per second".
* **Leftover debug output**: v0.9.2 writes an unused `beatmaps.bin` bincode dump of the merged rows on
  every run (`main.rs:68–71`, commented `//test … //end of test`); observed in the working directory.
* **Duplication/fragility**: `FullData` and `SavableFullData` duplicate every field and both
  `get_string` implementations; adding a column means editing the enum, `FromStr`, and both `match`es.
  `write_enum_to_file` writes `variants[0..len-1]`, silently assuming `MD5` is the last variant.
  Error handling is `Box<dyn Error>` + `println!`; failures still exit with code 0.

---

## How this was verified

Environment: Linux, stable `rustc`/`cargo` installed for this analysis; no Windows.

```bash
# 1. Current checkout: does it build?
cargo check --all-targets          # -> 6 × E0624, exit 101

# 2. Last release: build and run
git archive 77419ca | tar -x -C /tmp/sba-v092
cd /tmp/sba-v092 && cargo build     # ok, 23 warnings, 0 tests

# 3. Synthetic library: 5 hand-written .osu files
#    - 50-note standard map (20-note stream, 20 jumps, double, triple, 5-burst)
#    - byte-identical duplicate of it
#    - a second distinct standard map
#    - a taiko map (Mode: 1)
#    - a 2-object map
# 4. options_config.txt: FilePath=<songs dir>, StreamDistance=16, JumpDistance=110
cd /tmp/sba-run && /tmp/sba-target/debug/SimpleBeatmapAnalyzer   # run, then Enter
#   -> "Found 5 osu beatmaps ... Processed 3 ... CSV saved", results.csv has 3 rows
# 5. rerun after adding a 24-note probe map (37 px and 150 px spacings)
#   -> "Processed 1", probe row: Streams=0.00 ShortJumps=33.33
# 6. change options to StreamDistance=100 / JumpDistance=300, rerun
#   -> "Processed 4" (full recompute), probe row unchanged        => F2
# 7. delete one .osu file, rerun -> "Processed 0", deleted map still in results.csv   => F7
```

Observed `results.csv` (first run, middle columns trimmed):

```
Title,DifName,MapID,Stars,BPM,Bursts,Streams,...,MidJumps,...,Quads,MapSetID,LongestStream,...,AvgJumpsDistance,AvgJumpsSpeed,Creator,...,MD5
Test Song,Insane,106789,3.95,120,16.00,40.00,...,42.00,...,0.00,,20,...,269,1078,Mapper,...,1702026d7e13a8badc7f019c5c765072
Test Song,Insane,106789,3.95,120,16.00,40.00,...,42.00,...,0.00,,20,...,269,1078,Mapper,...,1702026d7e13a8badc7f019c5c765072
Second Song,Insane,122222,3.95,120,16.00,40.00,...,42.00,...,0.00,,20,...,269,1078,Other,...,bde1aa96c04b15d1ff9aff0658ee1041
```

The taiko map and the 2-object map are absent (filtered / silently dropped), `MapSetID` is blank, and the
duplicate produces a duplicate row — all as described above.

---

## If you revive it (the planned v0.9.3 rewrite)

1. **Start from `77419ca` (v0.9.2), not `master`** — or restore the WIP refactor only after deciding how
   to cache. The current `serialized_to_normal_beatmap` approach is blocked by `rosu-pp 0.9.5`'s private
   `SortedVec` API.
2. Prefer a **small cache**: `md5 → (hit_objects, timing_points, cs, metadata, stats, options_hash)` or
   just the raw `.osu` bytes plus a stats row. Do not cache whole `Beatmap` graphs.
3. **Hash and parse outside the lock**; keep the critical section to a single insert. Go back to
   `par_iter().fold`/`reduce` or a sharded map.
4. Make the **spacing thresholds real** (`d_r <= stream_spacing`, `d_r > jump_spacing`) and key the cache
   on them, so changing options only recomputes what is affected — or at least fix F2.
5. Fix metadata parsing: parse the `[Metadata]` block into a struct, handle `BeatmapSetID`, don't
   `unwrap` on `:`, and replace the positional `b1..b5` flags.
6. **Prune** cached rows for files that no longer exist, or rebuild the CSV from the files found in the
   current scan.
7. Use `PathBuf`/`join` everywhere, and gate `winres` under
   `[target.'cfg(windows)'.build-dependencies]`.
8. Add unit tests around `process_stats` (a synthetic 50-note map like the one above is a good fixture)
   and around metadata parsing; run CI on the same toolchain the cache types expect.
9. Smaller wins: sort the CSV rows for stable diffs, report newly processed vs. cached counts separately,
   return a non-zero exit code on failure, and drop the unused `FullData`/duplicate `get_string`.

