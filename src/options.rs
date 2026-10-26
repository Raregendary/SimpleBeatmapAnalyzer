//! Analysis options shared by the CLI and by library consumers.
//!
//! The options are part of the cache key: changing any of them invalidates
//! cached results, but never requires re-reading the `.osu` files.

use serde::{Deserialize, Serialize};

use crate::{Error, Result};

/// Bumped whenever the statistical algorithm changes, so caches produced by an
/// older algorithm are discarded instead of silently reused.
pub const ANALYZER_VERSION: u32 = 1;

/// Default maximum edge-to-edge spacing for 1/4 patterns (streams/bursts), in
/// osu! pixels.
pub const DEFAULT_STREAM_DISTANCE: f32 = 16.0;

/// Default minimum edge-to-edge spacing for 1/2 patterns (jumps), in osu!
/// pixels.
pub const DEFAULT_JUMP_DISTANCE: f32 = 110.0;

/// Default accuracy used for the NM/DT/HR pp values.
pub const DEFAULT_ACCURACY: f64 = 99.0;

/// Settings that influence the computed statistics.
///
/// All values are validated by [`AnalyzeOptions::validate`] before a beatmap is
/// read, so invalid input is reported instead of producing NaN metrics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnalyzeOptions {
    /// Maximum edge-to-edge spacing (pixels) for a note to count as part of a
    /// 1/4 pattern.
    pub stream_distance: f32,
    /// Minimum edge-to-edge spacing (pixels) for a note to count as a 1/2
    /// jump.
    pub jump_distance: f32,
    /// Accuracy (0-100) used for the NM/DT/HR pp values.
    pub accuracy: f64,
}

impl Default for AnalyzeOptions {
    fn default() -> Self {
        Self {
            stream_distance: DEFAULT_STREAM_DISTANCE,
            jump_distance: DEFAULT_JUMP_DISTANCE,
            accuracy: DEFAULT_ACCURACY,
        }
    }
}

impl AnalyzeOptions {
    /// Creates validated options.
    pub fn new(stream_distance: f32, jump_distance: f32, accuracy: f64) -> Result<Self> {
        let options = Self {
            stream_distance,
            jump_distance,
            accuracy,
        };
        options.validate()?;
        Ok(options)
    }

    /// Rejects non-finite or out-of-range values.
    pub fn validate(&self) -> Result<()> {
        if !self.stream_distance.is_finite() || self.stream_distance < 0.0 {
            return Err(Error::InvalidOptions(format!(
                "stream_distance must be a finite value >= 0 (got {})",
                self.stream_distance
            )));
        }

        if !self.jump_distance.is_finite() || self.jump_distance < 0.0 {
            return Err(Error::InvalidOptions(format!(
                "jump_distance must be a finite value >= 0 (got {})",
                self.jump_distance
            )));
        }

        if !self.accuracy.is_finite() || !(0.0..=100.0).contains(&self.accuracy) {
            return Err(Error::InvalidOptions(format!(
                "accuracy must be between 0 and 100 (got {})",
                self.accuracy
            )));
        }

        Ok(())
    }

    /// Stable fingerprint of the algorithm version and every option.
    ///
    /// Used as part of the cache key; it is deliberately independent of how the
    /// options were constructed (only their values matter).
    pub fn fingerprint(&self) -> u64 {
        let input = format!(
            "v{}|{:.6}|{:.6}|{:.6}",
            ANALYZER_VERSION, self.stream_distance, self.jump_distance, self.accuracy
        );
        fnv1a64(input.as_bytes())
    }
}

/// FNV-1a 64 bit. Small, deterministic and dependency-free, which is all a
/// cache-key fingerprint needs.
pub(crate) fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;

    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }

    hash
}
