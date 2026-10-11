//! Streaming resampler to 16 kHz mono.
//!
//! Design: windowed-sinc low-pass FIR (anti-aliasing, applied at the input
//! rate) followed by fractional linear interpolation. For speech recognition
//! at 16 kHz this is sufficient (passband ≤ ~7 kHz) and keeps the crate free
//! of DSP dependencies. `rubato` (MIT) is the planned drop-in if fixture WER
//! shows a measurable difference (RESEARCH.md §latency budget).

use std::f32::consts::PI;

/// Averages interleaved channels into mono.
///
/// Caution with microphone arrays: an empty channel halves the level (−6 dB),
/// phase-inverted channels cancel each other. [`Resampler`] therefore uses
/// [`ChannelMode::Auto`] (loudest channel) by default.
pub fn downmix_to_mono(interleaved: &[f32], channels: usize, out: &mut Vec<f32>) {
    let ch = channels.max(1);
    out.extend(interleaved.chunks_exact(ch).map(|f| f.iter().sum::<f32>() / ch as f32));
}

/// How multi-channel input becomes mono.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ChannelMode {
    /// Loudest channel (smoothed RMS, 3 dB hysteresis). Robust for mic arrays
    /// with silent or phase-inverted channels.
    #[default]
    Auto,
    /// Average of all channels (classic downmix).
    Mix,
    /// One fixed channel (0-based); falls back to Auto if out of range.
    Channel(usize),
}

impl ChannelMode {
    /// Parses the settings value: "auto" | "mix" | "1".."N" (1-based).
    pub fn parse(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "mix" => Self::Mix,
            n => match n.parse::<usize>() {
                Ok(i) if i >= 1 => Self::Channel(i - 1),
                _ => Self::Auto,
            },
        }
    }
}

/// Smoothing time constant of the per-channel level (seconds).
const CHANNEL_TAU_S: f32 = 0.5;
/// A different channel must be this much louder (dB) before Auto switches.
const SWITCH_HYSTERESIS_DB: f32 = 3.0;

/// Per-channel level tracking + channel selection (state of [`ChannelMode::Auto`]).
#[derive(Debug, Clone)]
pub struct ChannelSelector {
    pub mode: ChannelMode,
    channels: usize,
    rate: u32,
    /// Smoothed mean square per channel.
    ms: Vec<f32>,
    selected: usize,
}

impl ChannelSelector {
    pub fn new(channels: usize, rate: u32, mode: ChannelMode) -> Self {
        let channels = channels.max(1);
        Self { mode, channels, rate: rate.max(1), ms: vec![0.0; channels], selected: 0 }
    }

    /// Smoothed level per channel in dBFS (−100 for silence).
    pub fn channel_dbfs(&self) -> Vec<f32> {
        self.ms.iter().map(|m| ms_to_dbfs(*m)).collect()
    }

    /// Channel currently used (None = mix).
    pub fn active_channel(&self) -> Option<usize> {
        match self.mode {
            ChannelMode::Mix if self.channels > 1 => None,
            ChannelMode::Channel(i) if i < self.channels => Some(i),
            _ => Some(self.selected),
        }
    }

    /// Converts interleaved input to mono according to `mode`, appending to `out`.
    pub fn process(&mut self, interleaved: &[f32], out: &mut Vec<f32>) {
        let ch = self.channels;
        let frames = interleaved.len() / ch;
        if frames == 0 {
            return;
        }
        // Per-channel block energy -> exponential smoothing.
        let mut block = vec![0.0f32; ch];
        for f in interleaved.chunks_exact(ch) {
            for (b, s) in block.iter_mut().zip(f) {
                *b += s * s;
            }
        }
        let alpha = 1.0 - (-(frames as f32) / (self.rate as f32 * CHANNEL_TAU_S)).exp();
        for (m, b) in self.ms.iter_mut().zip(&block) {
            *m += alpha * (b / frames as f32 - *m);
        }
        if ch == 1 {
            out.extend_from_slice(&interleaved[..frames]);
            return;
        }
        // Auto: switch only when another channel is clearly louder.
        let mut best = 0;
        for (i, m) in self.ms.iter().enumerate() {
            if *m > self.ms[best] {
                best = i;
            }
        }
        let gain_db = ms_to_dbfs(self.ms[best]) - ms_to_dbfs(self.ms[self.selected]);
        if gain_db >= SWITCH_HYSTERESIS_DB {
            self.selected = best;
        }
        match self.active_channel() {
            None => downmix_to_mono(interleaved, ch, out),
            Some(i) => out.extend(interleaved.chunks_exact(ch).map(|f| f[i])),
        }
    }
}

fn ms_to_dbfs(ms: f32) -> f32 {
    if ms <= 1e-10 {
        -100.0
    } else {
        10.0 * ms.log10()
    }
}

#[derive(Debug, Clone)]
pub struct Resampler {
    in_rate: u32,
    out_rate: u32,
    /// Channel selection / downmix (public for diagnostics and mode changes).
    pub selector: ChannelSelector,
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
            selector: ChannelSelector::new(channels, in_rate, ChannelMode::Auto),
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
        self.selector.process(interleaved, &mut self.mono);

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
    use std::f32::consts::FRAC_1_SQRT_2;

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

    fn db(x: f32) -> f32 {
        20.0 * x.log10()
    }

    #[test]
    fn sine_1k_48k_to_16k_keeps_amplitude_within_1db() {
        let input = sine(48_000, 1000.0, 1.0);
        let mut r = Resampler::to_16k(48_000, 1);
        let mut out = Vec::new();
        for c in input.chunks(480) {
            r.process(c, &mut out); // 10 ms callbacks like cpal
        }
        let g = db(rms(&out[400..]) / rms(&input));
        assert!(g.abs() <= 1.0, "gain {g} dB");
        let peak = out[400..].iter().fold(0.0f32, |a, s| a.max(s.abs()));
        assert!((0.89..=1.0).contains(&peak), "peak {peak}");
    }

    fn interleave(chs: &[Vec<f32>]) -> Vec<f32> {
        (0..chs[0].len()).flat_map(|i| chs.iter().map(move |c| c[i])).collect()
    }

    fn run_mode(chs: &[Vec<f32>], mode: ChannelMode) -> (Vec<f32>, Resampler) {
        let mut r = Resampler::to_16k(48_000, chs.len());
        r.selector.mode = mode;
        let mut out = Vec::new();
        for c in interleave(chs).chunks(480 * chs.len()) {
            r.process(c, &mut out);
        }
        (out, r)
    }

    #[test]
    fn downmix_antiphase_channels_auto_keeps_level_mix_cancels() {
        let a = sine(48_000, 500.0, 1.0);
        let b: Vec<f32> = a.iter().map(|s| -s).collect();
        let (mix, _) = run_mode(&[a.clone(), b.clone()], ChannelMode::Mix);
        assert!(rms(&mix[400..]) < 1e-3, "mix cancels antiphase channels");
        let (auto, _) = run_mode(&[a, b], ChannelMode::Auto);
        assert!(db(rms(&auto[400..]) / FRAC_1_SQRT_2).abs() < 1.0);
    }

    #[test]
    fn downmix_empty_channel_auto_picks_loud_one() {
        let a = sine(48_000, 500.0, 1.0);
        let z = vec![0.0; a.len()];
        // Signal on the second channel: Auto must switch away from channel 0.
        let (auto, r) = run_mode(&[z.clone(), a.clone()], ChannelMode::Auto);
        assert_eq!(r.selector.active_channel(), Some(1));
        assert!(db(rms(&auto[8000..]) / FRAC_1_SQRT_2).abs() < 1.0);
        let lv = r.selector.channel_dbfs();
        assert_eq!(lv[0], -100.0);
        // EMA is ~86 % settled after 2 tau.
        assert!((lv[1] + 3.0).abs() < 1.0, "{lv:?}");
        // Mix loses 6 dB.
        let (mix, _) = run_mode(&[z.clone(), a.clone()], ChannelMode::Mix);
        assert!((db(rms(&mix[400..]) / FRAC_1_SQRT_2) + 6.02).abs() < 1.0);
        // Fixed channel.
        let (fixed, r) = run_mode(&[z, a], ChannelMode::Channel(0));
        assert_eq!(r.selector.active_channel(), Some(0));
        assert!(rms(&fixed) < 1e-6);
    }

    #[test]
    fn channel_mode_parse() {
        assert_eq!(ChannelMode::parse("auto"), ChannelMode::Auto);
        assert_eq!(ChannelMode::parse("MIX"), ChannelMode::Mix);
        assert_eq!(ChannelMode::parse("2"), ChannelMode::Channel(1));
        assert_eq!(ChannelMode::parse("0"), ChannelMode::Auto);
        assert_eq!(ChannelMode::parse("x"), ChannelMode::Auto);
    }

    #[test]
    fn identity_rate() {
        let mut r = Resampler::new(16_000, 1, 16_000);
        let mut out = Vec::new();
        r.process(&[0.1, 0.2, 0.3, 0.4], &mut out);
        assert_eq!(out, [0.1, 0.2, 0.3]); // last sample held back for interpolation
    }
}
