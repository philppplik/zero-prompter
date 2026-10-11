//! Digital input gain: fixed (manual dB) or a simple speech AGC, followed by
//! a soft limiter so the output always stays inside [-1, 1] (sherpa-onnx
//! expects float samples in that range).
//!
//! Runs on the 16 kHz mono stream (after [`crate::Resampler`]), in ~10 ms
//! blocks. The AGC only adapts on blocks above a noise gate, so pauses do not
//! pump the background noise up.

use crate::rms_dbfs;

/// Gain mode, from the "Mikrofon-Gain" setting.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GainMode {
    /// Automatic gain towards [`AGC_TARGET_DBFS`].
    Auto,
    /// Fixed gain in dB (0 = unchanged).
    Manual(f32),
}

/// Speech level the AGC aims for.
pub const AGC_TARGET_DBFS: f32 = -20.0;
/// Upper bound of the automatic gain.
pub const AGC_MAX_GAIN_DB: f32 = 40.0;
/// Lower bound of the automatic gain (attenuation of very loud input).
pub const AGC_MIN_GAIN_DB: f32 = -10.0;
/// Blocks quieter than this (before gain) are treated as noise: no adaptation.
pub const AGC_GATE_DBFS: f32 = -75.0;
/// Gain increase rate (slow, avoids pumping).
const RISE_DB_PER_S: f32 = 12.0;
/// Gain decrease rate (fast, avoids clipping on loud onsets).
const FALL_DB_PER_S: f32 = 60.0;
/// Limiter knee: above this magnitude the signal is compressed softly towards 1.
const LIMIT_KNEE: f32 = 0.9;
/// Range of the manual gain (dB).
pub const MANUAL_GAIN_RANGE_DB: (f32, f32) = (-20.0, 40.0);

#[derive(Debug, Clone)]
pub struct Agc {
    mode: GainMode,
    rate: u32,
    gain_db: f32,
}

impl Agc {
    pub fn new(mode: GainMode, rate: u32) -> Self {
        let mut a = Self { mode, rate: rate.max(1), gain_db: 0.0 };
        a.set_mode(mode);
        a
    }

    pub fn mode(&self) -> GainMode {
        self.mode
    }

    /// Changes the mode; a manual gain takes effect immediately, Auto keeps
    /// the current gain as its starting point.
    pub fn set_mode(&mut self, mode: GainMode) {
        self.mode = mode;
        if let GainMode::Manual(db) = mode {
            let (lo, hi) = MANUAL_GAIN_RANGE_DB;
            self.gain_db = if db.is_finite() { db.clamp(lo, hi) } else { 0.0 };
        }
    }

    /// Gain currently applied (dB).
    pub fn gain_db(&self) -> f32 {
        self.gain_db
    }

    /// Applies gain + limiter in place.
    pub fn process(&mut self, block: &mut [f32]) {
        if block.is_empty() {
            return;
        }
        if self.mode == GainMode::Auto {
            let level = rms_dbfs(block);
            if level > AGC_GATE_DBFS {
                let want = (AGC_TARGET_DBFS - level).clamp(AGC_MIN_GAIN_DB, AGC_MAX_GAIN_DB);
                let dt = block.len() as f32 / self.rate as f32;
                let diff = want - self.gain_db;
                let step = if diff > 0.0 {
                    diff.min(RISE_DB_PER_S * dt)
                } else {
                    diff.max(-FALL_DB_PER_S * dt)
                };
                self.gain_db += step;
            }
        }
        let g = 10f32.powf(self.gain_db / 20.0);
        for s in block.iter_mut() {
            *s = soft_limit(*s * g);
        }
    }
}

/// Identity below the knee, smooth tanh compression above; |out| < 1.
pub fn soft_limit(x: f32) -> f32 {
    let a = x.abs();
    if !a.is_finite() {
        return 0.0;
    }
    if a <= LIMIT_KNEE {
        return x;
    }
    let room = 1.0 - LIMIT_KNEE;
    let y = LIMIT_KNEE + room * ((a - LIMIT_KNEE) / room).tanh();
    // tanh saturates to exactly 1.0 in f32 for large inputs; keep strictly inside.
    y.min(0.999_9).copysign(x)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    fn tone(amp: f32, n: usize) -> Vec<f32> {
        (0..n).map(|i| amp * (2.0 * PI * 300.0 * i as f32 / 16_000.0).sin()).collect()
    }

    #[test]
    fn agc_lifts_quiet_speech_towards_target() {
        // -46 dBFS rms (like the Surface log): amplitude ~0.0071.
        let mut agc = Agc::new(GainMode::Auto, 16_000);
        for _ in 0..600 {
            // 6 s in 10 ms blocks
            agc.process(&mut tone(0.0071, 160));
        }
        let mut last = tone(0.0071, 160);
        agc.process(&mut last);
        let out = rms_dbfs(&last);
        assert!((out - AGC_TARGET_DBFS).abs() < 2.0, "out {out} dBFS, gain {}", agc.gain_db());
    }

    #[test]
    fn agc_does_not_pump_noise_and_limits() {
        let mut agc = Agc::new(GainMode::Auto, 16_000);
        for _ in 0..500 {
            let mut b = tone(1e-5, 160); // -100 dBFS: below the gate
            agc.process(&mut b);
        }
        assert_eq!(agc.gain_db(), 0.0, "no adaptation on noise");

        // Loud input: gain falls, output never clips.
        for _ in 0..200 {
            let mut b = tone(1.5, 160);
            agc.process(&mut b);
            assert!(b.iter().all(|s| s.abs() < 1.0));
        }
        assert!(agc.gain_db() < 0.0);
    }

    #[test]
    fn manual_gain_and_limiter() {
        let mut agc = Agc::new(GainMode::Manual(20.0), 16_000);
        let mut b = vec![0.01, -0.01, 0.5, -2.0];
        agc.process(&mut b);
        assert!((b[0] - 0.1).abs() < 1e-5 && (b[1] + 0.1).abs() < 1e-5);
        assert!(b[2] > 0.9 && b[2] < 1.0);
        assert!(b[3] < -0.9 && b[3] > -1.0);
        let mut agc = Agc::new(GainMode::Manual(999.0), 16_000);
        assert_eq!(agc.gain_db(), MANUAL_GAIN_RANGE_DB.1);
        agc.set_mode(GainMode::Manual(f32::NAN));
        assert_eq!(agc.gain_db(), 0.0);
        let mut zero = Agc::new(GainMode::Manual(0.0), 16_000);
        let mut b = vec![0.3, -0.7];
        zero.process(&mut b);
        assert_eq!(b, [0.3, -0.7]);
    }

    #[test]
    fn soft_limit_is_monotonic_and_bounded() {
        let mut prev = -1.0;
        for i in 0..=400 {
            let y = soft_limit(i as f32 * 0.01);
            assert!(y >= prev && y < 1.0);
            prev = y;
        }
        assert_eq!(soft_limit(f32::NAN), 0.0);
    }
}
