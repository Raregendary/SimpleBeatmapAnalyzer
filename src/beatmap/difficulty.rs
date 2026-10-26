//! Difficulty and performance computation.
//!
//! All work is delegated to `rosu-pp`; this module only normalises the results
//! into a plain, serialisable struct with the NM/DT/HR values the analyzer
//! reports.

use rosu_pp::{Beatmap, BeatmapExt};
use serde::{Deserialize, Serialize};

/// Star ratings, pp and mod-adjusted map attributes.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DifficultyStats {
    /// Circle size of the original map.
    pub cs: f32,
    /// Approach rate of the original map.
    pub ar: f32,
    /// Overall difficulty of the original map.
    pub od: f32,
    /// Drain rate of the original map.
    pub hp: f32,
    /// Star rating without mods.
    pub stars_nm: f64,
    /// Star rating with Double Time.
    pub stars_dt: f64,
    /// Star rating with Hard Rock.
    pub stars_hr: f64,
    /// pp for a 99% (configurable) play without mods.
    pub pp_nm: f64,
    /// pp for a 99% (configurable) play with Double Time.
    pub pp_dt: f64,
    /// pp for a 99% (configurable) play with Hard Rock.
    pub pp_hr: f64,
    /// Maximum combo of the map.
    pub max_combo: u32,
    /// Accuracy used for the pp values.
    pub accuracy: f64,
    /// Approach rate after applying Double Time.
    pub ar_dt: f32,
    /// Overall difficulty after applying Double Time.
    pub od_dt: f32,
    /// Approach rate after applying Hard Rock.
    pub ar_hr: f32,
    /// Overall difficulty after applying Hard Rock.
    pub od_hr: f32,
    /// Circle size after applying Hard Rock.
    pub cs_hr: f32,
    /// Drain rate after applying Hard Rock.
    pub hp_hr: f32,
}

impl DifficultyStats {
    pub(crate) fn calculate(map: &Beatmap, accuracy: f64) -> Self {
        let accuracy = accuracy.clamp(0.0, 100.0);

        let nm = map.pp().accuracy(accuracy).calculate();
        let dt = map.pp().accuracy(accuracy).mods(64).calculate();
        let hr = map.pp().accuracy(accuracy).mods(16).calculate();

        let dt_attributes = map.attributes().mods(64).build();
        let hr_attributes = map.attributes().mods(16).build();

        Self {
            cs: map.cs,
            ar: map.ar,
            od: map.od,
            hp: map.hp,
            stars_nm: nm.stars(),
            stars_dt: dt.stars(),
            stars_hr: hr.stars(),
            pp_nm: nm.pp(),
            pp_dt: dt.pp(),
            pp_hr: hr.pp(),
            max_combo: nm.max_combo() as u32,
            accuracy,
            ar_dt: dt_attributes.ar as f32,
            od_dt: dt_attributes.od as f32,
            ar_hr: hr_attributes.ar as f32,
            od_hr: hr_attributes.od as f32,
            cs_hr: hr_attributes.cs as f32,
            hp_hr: hr_attributes.hp as f32,
        }
    }
}
