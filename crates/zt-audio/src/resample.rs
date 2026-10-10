//! Streaming resampler to 16 kHz mono.
//!
//! Design: windowed-sinc low-pass FIR (anti-aliasing, applied at the input
//! rate) followed by fractional linear interpolation. For speech recognition
//! at 16 kHz this is sufficient (passband ≤ ~7 kHz) and keeps the crate free
//! of DSP dependencies. `rubato` (MIT) is the planned drop-in if fixture WER
//! shows a measurable difference (RESEARCH.md §latency budget).

use std::f32::consts::PI;

/// Averages interleaved channels into mono.
pub fn downmix_to_mono(interleaved: &[f32], channels: usize, out: &mut Vec<f32>) {
    let ch = channels.max(1);
    out.extend(interleaved.chunks_exact(ch).map(|f| f.iter().sum::<f32>() / ch as f32));
}

#[derive(Debug, Clone)]
pub struct Resampler {
    in_rate: u32,
    out_rate: u32,
    channels: usize,
    taps: Vec<f32>,
    /// FIR history (last taps.len()-1 input samples).
    history: Vec<f32>,
    /// Position of the next output sample, in input samples, relative to
    /// the first sample of the pending filtered buffer.
    pos: f64,
    /// Filtered samples not yet fully consumed (we keep the last one for interpolation).
    pending: Vec<f32>,
    mono: Vec<f32>,
}

impl Resampler {
    pub fn new(in_rate: u32, channels: usize, out_rate: u32) -> Self {
        let taps = if in_rate > out_rate {
            // cutoff slightly below the output Nyquist
            lowpass_taps(0.45 * out_rate as f32 / in_rate as f32, 63)
        } else {
            vec![1.0]
        };
        Self {
            in_rate,
            out_rate,
            channels: channels.max(1),
            history: vec![0.0; taps.len() - 1],
            taps,
            pos: 0.0,
            pending: Vec::new(),
            mono: Vec::new(),
        }
    }

    pub fn to_16k(in_rate: u32, channels: usize) -> Self {
        Self::new(in_rate, channels, crate::TARGET_RATE)
    }

    pub fn ratio(&self) -> f64 {
        self.in_rate as f64 / self.out_rate as f64
    }

    /// Processes interleaved input and appends resampled mono output to `out`.
    pub fn process(&mut self, interleaved: &[f32], out: &mut Vec<f32>) {
        self.mono.clear();
        downmix_to_mono(interleaved, self.channels, &mut self.mono);

        // FIR low-pass (direct form over history ++ new samples).
        let n_hist = self.history.len();
        let mut buf = std::mem::take(&mut self.history);
        buf.extend_from_slice(&self.mono);
        for i in 0..self.mono.len() {
            let window = &buf[i..i + self.taps.len()];
            let y: f32 = window.iter().zip(self.taps.iter().rev()).map(|(x, h)| x * h).sum();
            self.pending.push(y);
        }
        self.history = buf.split_off(buf.len() - n_hist);

        // Fractional linear interpolation.
        let step = self.ratio();
        while self.pos + 1.0 < self.pending.len() as f64 {
            let i = self.pos.floor() as usize;
            let frac = (self.pos - i as f64) as f32;
            out.push(self.pending[i] * (1.0 - frac) + self.pending[i + 1] * frac);
            self.pos += step;
        }
        // Drop consumed samples, keep the one needed for interpolation.
        let consumed = (self.pos.floor() as usize).min(self.pending.len());
        self.pending.drain(..consumed);
        self.pos -= consumed as f64;
    }

    pub fn reset(&mut self) {
        self.history.fill(0.0);
        self.pending.clear();
        self.pos = 0.0;
    }
}

/// Windowed-sinc (Blackman) low-pass; `cutoff` as fraction of the input rate (0..0.5).
fn lowpass_taps(cutoff: f32, n: usize) -> Vec<f32> {
    let m = (n - 1) as f32;
    let mut taps: Vec<f32> = (0..n)
        .map(|i| {
            let x = i as f32 - m / 2.0;
            let sinc =
                if x == 0.0 { 2.0 * cutoff } else { (2.0 * PI * cutoff * x).sin() / (PI * x) };
            let w = 0.42 - 0.5 * (2.0 * PI * i as f32 / m).cos()
                + 0.08 * (4.0 * PI * i as f32 / m).cos();
            sinc * w
        })
        .collect();
    let sum: f32 = taps.iter().sum();
    taps.iter_mut().for_each(|t| *t /= sum);
    taps
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sine(rate: u32, freq: f32, secs: f32) -> Vec<f32> {
        (0..(rate as f32 * secs) as usize)
            .map(|i| (2.0 * PI * freq * i as f32 / rate as f32).sin())
            .collect()
    }

    fn rms(x: &[f32]) -> f32 {
        (x.iter().map(|s| s * s).sum::<f32>() / x.len() as f32).sqrt()
    }

    #[test]
    fn output_length_matches_ratio_across_chunks() {
        let mut r = Resampler::to_16k(48_000, 1);
        let input = sine(48_000, 440.0, 1.0);
        let mut out = Vec::new();
        for c in input.chunks(441) {
            r.process(c, &mut out);
        }
        assert!((out.len() as i64 - 16_000).abs() <= 2, "len {}", out.len());
    }

    #[test]
    fn passband_kept_stopband_attenuated() {
        let mut r = Resampler::to_16k(48_000, 1);
        let mut out = Vec::new();
        r.process(&sine(48_000, 1000.0, 0.5), &mut out);
        assert!(rms(&out[200..]) > 0.65); // sine rms ≈ 0.707

        let mut r = Resampler::to_16k(48_000, 1);
        let mut out = Vec::new();
        r.process(&sine(48_000, 12_000.0, 0.5), &mut out); // above 8 kHz Nyquist
        assert!(rms(&out[200..]) < 0.05);
    }

    #[test]
    fn stereo_downmix_and_44k1() {
        let mono = sine(44_100, 500.0, 0.2);
        let stereo: Vec<f32> = mono.iter().flat_map(|s| [*s, *s]).collect();
        let mut r = Resampler::to_16k(44_100, 2);
        let mut out = Vec::new();
        r.process(&stereo, &mut out);
        let expected = (mono.len() as f64 / (44_100.0 / 16_000.0)) as i64;
        assert!((out.len() as i64 - expected).abs() <= 2);
    }

    #[test]
    fn identity_rate() {
        let mut r = Resampler::new(16_000, 1, 16_000);
        let mut out = Vec::new();
        r.process(&[0.1, 0.2, 0.3, 0.4], &mut out);
        assert_eq!(out, [0.1, 0.2, 0.3]); // last sample held back for interpolation
    }
}
