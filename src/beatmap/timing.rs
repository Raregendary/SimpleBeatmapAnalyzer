//! Timing and object statistics.

use rosu_pp::Beatmap;
use serde::{Deserialize, Serialize};

/// Tempo, length and object counts of a beatmap.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TimingStats {
    /// Most common BPM, weighted by the time each timing section is active.
    pub bpm: f64,
    /// Lowest uninherited BPM in the map.
    pub bpm_min: f64,
    /// Highest uninherited BPM in the map.
    pub bpm_max: f64,
    /// `bpm` multiplied by the Double Time clock rate (1.5x).
    pub bpm_dt: f64,
    /// Start time of the first hit object in milliseconds.
    pub first_object_ms: f64,
    /// End time of the last hit object in milliseconds (sliders use their end).
    pub last_object_end_ms: f64,
    /// Playable length from the first object to the last object's end.
    pub playable_length_ms: f64,
    /// [`TimingStats::playable_length_ms`] in seconds.
    pub playable_length_sec: f64,
    /// Number of circles.
    pub circles: u32,
    /// Number of sliders.
    pub sliders: u32,
    /// Number of spinners.
    pub spinners: u32,
    /// Total number of hit objects.
    pub objects: u32,
    /// Number of uninherited timing points.
    pub timing_points: u32,
}

impl TimingStats {
    pub(crate) fn calculate(map: &Beatmap) -> Self {
        let bpm = map.bpm();
        let bpm = if bpm.is_finite() && bpm > 0.0 {
            bpm
        } else {
            0.0
        };

        let mut bpm_min = f64::INFINITY;
        let mut bpm_max = 0.0_f64;

        for timing_point in map.timing_points.iter() {
            if timing_point.beat_len > 0.0 {
                let value = 60_000.0 / timing_point.beat_len;
                bpm_min = bpm_min.min(value);
                bpm_max = bpm_max.max(value);
            }
        }

        if !bpm_min.is_finite() {
            bpm_min = 0.0;
        }

        let first_object_ms = map
            .hit_objects
            .first()
            .map(|object| object.start_time)
            .unwrap_or(0.0);
        let last_object_end_ms = map
            .hit_objects
            .last()
            .map(|object| object.end_time())
            .unwrap_or(0.0);
        let playable_length_ms = (last_object_end_ms - first_object_ms).max(0.0);

        Self {
            bpm,
            bpm_min,
            bpm_max,
            bpm_dt: bpm * 1.5,
            first_object_ms,
            last_object_end_ms,
            playable_length_ms,
            playable_length_sec: playable_length_ms / 1000.0,
            circles: map.n_circles,
            sliders: map.n_sliders,
            spinners: map.n_spinners,
            objects: map.hit_objects.len() as u32,
            timing_points: map.timing_points.len() as u32,
        }
    }
}
