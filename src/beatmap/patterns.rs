//! Advanced stream/jump pattern detection.
//!
//! The detector walks the hit objects once and classifies runs of notes based
//! on two things:
//!
//! * **Spacing** — the edge-to-edge distance between consecutive object heads,
//!   with the circle diameter derived from CS (like osu! itself).
//! * **Snap** — `dt / (beat_len / 4)`. A ratio of `1.0` is a 1/4 snap, `2.0` is
//!   a 1/2 snap and `1.333…` is a 1/3 snap.
//!
//! A run is measured in **notes** (not moves). Counting starts when a note
//! qualifies and is flushed by the next note that does not, so a single 1/4
//! note that is close enough to its predecessor is reported as a 2-note double.
//!
//! All percentages are percentages of the map's hit objects, not of its
//! duration.

use rosu_pp::beatmap::TimingPoint;
use rosu_pp::parse::HitObject;
use serde::{Deserialize, Serialize};

/// Largest 1/2 run (in notes) still reported as a short jump.
pub const SHORT_JUMP_MAX: u32 = 12;
/// Largest 1/2 run (in notes) still reported as a mid jump.
pub const MID_JUMP_MAX: u32 = 32;

/// Counts and derived indices for the pattern analysis of one map.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PatternStats {
    /// Total number of hit objects, the denominator of every percentage.
    pub total_notes: u32,
    /// Notes that are part of a 2-note 1/4 run.
    pub doubles: u32,
    /// Notes that are part of a 3-note 1/4 run.
    pub triples: u32,
    /// Notes that are part of a 4-note 1/4 run.
    pub quads: u32,
    /// Notes in 1/4 runs of 3 to 11 notes (includes triples and quads).
    pub burst_notes: u32,
    /// Notes in 1/4 runs of 12 to 32 notes.
    pub stream_notes: u32,
    /// Notes in 1/4 runs of 33 or more notes.
    pub deathstream_notes: u32,
    /// Length of the longest 1/4 run, in notes.
    pub longest_stream: u32,
    /// Number of 1/4 runs with at least 100 notes.
    pub streams_100: u32,
    /// Notes in 1/2 runs of 2 to 12 notes.
    pub short_jump_notes: u32,
    /// Notes in 1/2 runs of 13 to 32 notes.
    pub mid_jump_notes: u32,
    /// Notes in 1/2 runs of 33 or more notes.
    pub long_jump_notes: u32,
    /// Notes in 1/2 runs shorter than 8 notes (the `<8 = short` split used by
    /// the GUI).
    pub jump_notes_under_8: u32,
    /// Notes in 1/2 runs of 8 or more notes (the `>=8 = long` split used by the
    /// GUI).
    pub jump_notes_8_plus: u32,
    /// Average edge-to-edge distance of every 1/2 jump, in pixels.
    pub avg_jump_distance: f32,
    /// Average jump speed (`1000 * distance / dt`), in pixels per second.
    pub avg_jump_speed: f32,
    /// Notes on a 1/3 snap, regardless of their spacing.
    pub one_third_notes: u32,
    /// Stream index: `(stream_value - jump_value - 0.5 * doubles) / notes`.
    pub si: f64,
    /// Jump index: `(jump_value - stream_value - 0.5 * doubles) / notes`.
    pub ji: f64,
    /// Experimental finger-control double-burst index.
    pub fcdbi: f64,
    /// Stream spacing threshold used for this map, in pixels.
    pub stream_distance: f32,
    /// Jump spacing threshold used for this map, in pixels.
    pub jump_distance: f32,
}

impl PatternStats {
    /// Number of notes in any 1/4 pattern of 3 or more notes
    /// (bursts + streams + deathstreams).
    pub fn stream_notes_total(&self) -> u32 {
        self.burst_notes
            .saturating_add(self.stream_notes)
            .saturating_add(self.deathstream_notes)
    }

    /// Number of notes that are part of a 1/2 jump run.
    pub fn jump_notes(&self) -> u32 {
        self.short_jump_notes
            .saturating_add(self.mid_jump_notes)
            .saturating_add(self.long_jump_notes)
    }

    /// Converts a note count into a percentage of the map's hit objects.
    pub fn percent(&self, notes: u32) -> f32 {
        if self.total_notes == 0 {
            0.0
        } else {
            notes as f32 * 100.0 / self.total_notes as f32
        }
    }

    /// Percentage of notes in doubles.
    pub fn doubles_pct(&self) -> f32 {
        self.percent(self.doubles)
    }

    /// Percentage of notes in triples.
    pub fn triples_pct(&self) -> f32 {
        self.percent(self.triples)
    }

    /// Percentage of notes in quads.
    pub fn quads_pct(&self) -> f32 {
        self.percent(self.quads)
    }

    /// Percentage of notes in bursts (3-11 note 1/4 runs).
    pub fn burst_pct(&self) -> f32 {
        self.percent(self.burst_notes)
    }

    /// Percentage of notes in streams (12-32 note 1/4 runs).
    pub fn stream_pct(&self) -> f32 {
        self.percent(self.stream_notes)
    }

    /// Percentage of notes in deathstreams (33+ note 1/4 runs).
    pub fn deathstream_pct(&self) -> f32 {
        self.percent(self.deathstream_notes)
    }

    /// Percentage of notes in any 1/4 pattern of 3+ notes.
    pub fn overall_stream_pct(&self) -> f32 {
        self.percent(self.stream_notes_total())
    }

    /// Percentage of notes in short jumps (2-12 note 1/2 runs).
    pub fn short_jump_pct(&self) -> f32 {
        self.percent(self.short_jump_notes)
    }

    /// Percentage of notes in mid jumps (13-32 note 1/2 runs).
    pub fn mid_jump_pct(&self) -> f32 {
        self.percent(self.mid_jump_notes)
    }

    /// Percentage of notes in long jumps (33+ note 1/2 runs).
    pub fn long_jump_pct(&self) -> f32 {
        self.percent(self.long_jump_notes)
    }

    /// Percentage of notes in 1/2 runs shorter than 8 notes.
    ///
    /// This is the `short jumps` bucket used by the GUI, in addition to the
    /// 2–12/13–32/33+ buckets above.
    pub fn short_jump_pct_under_8(&self) -> f32 {
        self.percent(self.jump_notes_under_8)
    }

    /// Percentage of notes in 1/2 runs of 8 or more notes.
    ///
    /// This is the `long jumps` bucket used by the GUI, in addition to the
    /// 2–12/13–32/33+ buckets above.
    pub fn long_jump_pct_8_plus(&self) -> f32 {
        self.percent(self.jump_notes_8_plus)
    }

    /// Percentage of notes that are part of any jump run.
    pub fn jump_pct(&self) -> f32 {
        self.percent(self.jump_notes())
    }

    /// Percentage of notes on a 1/3 snap.
    pub fn one_third_pct(&self) -> f32 {
        self.percent(self.one_third_notes)
    }

    /// Recomputes the weighted indices from the note counters.
    fn update_indices(&mut self) {
        let total = f64::from(self.total_notes);

        if total == 0.0 {
            self.si = 0.0;
            self.ji = 0.0;
            self.fcdbi = 0.0;
            return;
        }

        let jump_value = f64::from(self.short_jump_notes)
            + 1.5 * f64::from(self.mid_jump_notes)
            + 2.0 * f64::from(self.long_jump_notes);
        let stream_value = f64::from(self.burst_notes)
            + 1.5 * f64::from(self.stream_notes)
            + 2.0 * f64::from(self.deathstream_notes);
        let doubles = f64::from(self.doubles);

        self.si = (stream_value - jump_value - 0.5 * doubles) / total;
        self.ji = (jump_value - stream_value - 0.5 * doubles) / total;
        self.fcdbi =
            (f64::from(self.burst_notes) - f64::from(self.triples) - f64::from(self.quads)
                + 1.75 * doubles
                + 1.5 * f64::from(self.triples)
                + 1.75 * f64::from(self.quads)
                - jump_value
                - 1.35 * f64::from(self.stream_notes)
                - 1.7 * f64::from(self.deathstream_notes))
                / total;
    }
}

/// Detects stream and jump patterns in a standard-mode beatmap.
///
/// Returns `None` when the map is too short (3 or fewer objects), has no
/// timing points, or the thresholds are not finite. A map with no patterns at
/// all still returns `Some` with zero counters.
///
/// `stream_distance` and `jump_distance` are edge-to-edge distances in osu!
/// pixels; they are used verbatim, so the caller's configuration always has an
/// effect on the result.
pub fn detect_patterns(
    hit_objects: &[HitObject],
    timing_points: &[TimingPoint],
    cs: f32,
    stream_distance: f32,
    jump_distance: f32,
) -> Option<PatternStats> {
    if hit_objects.len() <= 3 || timing_points.is_empty() {
        return None;
    }

    if !stream_distance.is_finite()
        || !jump_distance.is_finite()
        || stream_distance < 0.0
        || jump_distance < 0.0
    {
        return None;
    }

    let mut stats = PatternStats {
        total_notes: hit_objects.len() as u32,
        stream_distance,
        jump_distance,
        ..PatternStats::default()
    };

    // osu!'s circle diameter formula, rounded like the game does.
    let circle_diameter = f64::from((((54.4 - 4.48 * cs) * 1000.0).round() / 1000.0) * 2.0);

    let first = hit_objects.first()?;
    let mut last_x = f64::from(first.pos.x);
    let mut last_y = f64::from(first.pos.y);
    let mut last_time = first.start_time;

    let mut quarter_run = 0u32;
    let mut half_run = 0u32;
    let mut third_run = 0u32;

    let mut jump_moves = 0u32;
    let mut jump_distance_sum = 0.0f64;
    let mut jump_speed_sum = 0.0f64;

    let mut bpm_index = 0usize;

    for object in hit_objects.iter().skip(1) {
        let x = f64::from(object.pos.x);
        let y = f64::from(object.pos.y);
        let start_time = object.start_time;

        let dx = last_x - x;
        let dy = last_y - y;
        let distance = (dx * dx + dy * dy).sqrt() - circle_diameter;

        while bpm_index + 1 < timing_points.len() && timing_points[bpm_index + 1].time <= start_time
        {
            bpm_index += 1;
        }

        let beat_len = timing_points
            .get(bpm_index)
            .map(|timing_point| timing_point.beat_len)
            .unwrap_or(0.0);
        let quarter_note_ms = 0.25 * beat_len;
        let ratio = if quarter_note_ms > 0.0 {
            (start_time - last_time) / quarter_note_ms
        } else {
            0.0
        };

        // 1/4 patterns: overlapping or very close notes on a 1/4 snap.
        if distance <= f64::from(stream_distance) && ratio > 0.9 && ratio < 1.1 {
            quarter_run += 1;
        } else if quarter_run > 0 {
            classify_quarter(&mut stats, quarter_run + 1);
            quarter_run = 0;
        }

        // 1/2 patterns: spaced notes on a 1/2 snap.
        if distance > f64::from(jump_distance) && ratio > 1.9 && ratio < 2.1 {
            half_run += 1;
            jump_moves += 1;
            jump_distance_sum += distance;

            let dt = start_time - last_time;
            if dt > 0.0 {
                jump_speed_sum += 1000.0 * distance / dt;
            }
        } else if half_run > 0 {
            classify_jump(&mut stats, half_run + 1);
            half_run = 0;
        }

        // 1/3 snap notes, independent of spacing.
        if ratio > 1.25 && ratio < 1.45 {
            third_run += 1;
        } else if third_run > 0 {
            stats.one_third_notes += third_run + 1;
            third_run = 0;
        }

        last_x = x;
        last_y = y;
        last_time = start_time;
    }

    // Flush runs that continue until the last object of the map.
    if quarter_run > 0 {
        classify_quarter(&mut stats, quarter_run + 1);
    }

    if half_run > 0 {
        classify_jump(&mut stats, half_run + 1);
    }

    if third_run > 0 {
        stats.one_third_notes += third_run + 1;
    }

    if jump_moves > 0 {
        let moves = f64::from(jump_moves);
        stats.avg_jump_distance = (jump_distance_sum / moves) as f32;
        stats.avg_jump_speed = (jump_speed_sum / moves) as f32;
    }

    stats.update_indices();
    Some(stats)
}

fn classify_quarter(stats: &mut PatternStats, notes: u32) {
    stats.longest_stream = stats.longest_stream.max(notes);

    match notes {
        2 => stats.doubles += 2,
        3 => {
            stats.triples += 3;
            stats.burst_notes += 3;
        }
        4 => {
            stats.quads += 4;
            stats.burst_notes += 4;
        }
        5..=11 => stats.burst_notes += notes,
        12..=32 => stats.stream_notes += notes,
        _ => {
            stats.deathstream_notes += notes;

            if notes >= 100 {
                stats.streams_100 += 1;
            }
        }
    }
}

fn classify_jump(stats: &mut PatternStats, notes: u32) {
    if notes < 8 {
        stats.jump_notes_under_8 += notes;
    } else {
        stats.jump_notes_8_plus += notes;
    }

    if notes <= SHORT_JUMP_MAX {
        stats.short_jump_notes += notes;
    } else if notes <= MID_JUMP_MAX {
        stats.mid_jump_notes += notes;
    } else {
        stats.long_jump_notes += notes;
    }
}
