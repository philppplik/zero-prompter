//! Speech segmentation for the resync path.
//!
//! [`RmsSegmenter`] is a simple energy gate (frame RMS from `zt-audio`) with
//! onset / hangover hysteresis, pre-roll and a maximum segment length. It
//! needs no model and is the fallback when the Silero VAD (feature `sherpa`,
//! `SileroSegmenter`) is not available.

use crate::SAMPLE_RATE;
use std::collections::VecDeque;

/// One finished speech segment.
#[derive(Debug, Clone, PartialEq)]
pub struct Segment {
    /// Absolute position (samples since start) of the first sample.
    pub start_sample: u64,
    pub samples: Vec<f32>,
}

impl Segment {
    pub fn end_sample(&self) -> u64 {
        self.start_sample + self.samples.len() as u64
    }
}

/// Port: cuts a 16 kHz mono stream into speech segments.
pub trait Segmenter: Send {
    fn accept(&mut self, samples: &[f32]);
    /// Next finished segment, if any.
    fn pop(&mut self) -> Option<Segment>;
    /// Closes a running segment (e.g. at stop).
    fn flush(&mut self);
    /// Drops all state (e.g. after a manual seek).
    fn reset(&mut self);
}

impl<S: Segmenter + ?Sized> Segmenter for Box<S> {
    fn accept(&mut self, samples: &[f32]) {
        (**self).accept(samples)
    }
    fn pop(&mut self) -> Option<Segment> {
        (**self).pop()
    }
    fn flush(&mut self) {
        (**self).flush()
    }
    fn reset(&mut self) {
        (**self).reset()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RmsSegmenterConfig {
    pub frame_ms: u32,
    /// Frame is "speech" if its RMS is at least this loud (dBFS).
    pub threshold_db: f32,
    /// Speech must last this long before a segment opens.
    pub min_speech_ms: u32,
    /// Silence of this length closes a segment.
    pub min_silence_ms: u32,
    /// Segments are cut at this length (bounded offline decode time).
    pub max_segment_ms: u32,
    /// Segments with less *voiced* audio are discarded (clicks, breaths).
    pub min_voiced_ms: u32,
    /// Audio kept before the onset so the first word is not clipped.
    pub preroll_ms: u32,
}

impl Default for RmsSegmenterConfig {
    fn default() -> Self {
        Self {
            frame_ms: 30,
            threshold_db: -42.0,
            min_speech_ms: 120,
            min_silence_ms: 500,
            max_segment_ms: 12_000,
            min_voiced_ms: 300,
            preroll_ms: 200,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RmsSegmenter {
    cfg: RmsSegmenterConfig,
    frame: Vec<f32>,
    /// Absolute sample index of the next sample entering `frame`'s end.
    pos: u64,
    preroll: VecDeque<f32>,
    onset_frames: u32,
    in_speech: bool,
    silence_frames: u32,
    seg: Vec<f32>,
    seg_start: u64,
    voiced: usize,
    out: VecDeque<Segment>,
}

const fn ms_to_samples(ms: u32) -> usize {
    (SAMPLE_RATE as usize / 1000) * ms as usize
}

impl RmsSegmenter {
    pub fn new(cfg: RmsSegmenterConfig) -> Self {
        Self {
            cfg,
            frame: Vec::with_capacity(ms_to_samples(cfg.frame_ms)),
            pos: 0,
            preroll: VecDeque::new(),
            onset_frames: 0,
            in_speech: false,
            silence_frames: 0,
            seg: Vec::new(),
            seg_start: 0,
            voiced: 0,
            out: VecDeque::new(),
        }
    }

    fn frames(&self, ms: u32) -> u32 {
        ms.div_ceil(self.cfg.frame_ms.max(1))
    }

    fn close(&mut self) {
        let seg = std::mem::take(&mut self.seg);
        self.in_speech = false;
        self.onset_frames = 0;
        self.silence_frames = 0;
        self.preroll.clear();
        let voiced = std::mem::take(&mut self.voiced);
        if voiced >= ms_to_samples(self.cfg.min_voiced_ms) {
            self.out.push_back(Segment { start_sample: self.seg_start, samples: seg });
        }
    }

    fn on_frame(&mut self, frame: &[f32]) {
        let loud = zt_audio::rms_dbfs(frame) >= self.cfg.threshold_db;
        let end = self.pos; // absolute index after this frame
        if !self.in_speech {
            self.preroll.extend(frame.iter().copied());
            let keep = ms_to_samples(self.cfg.preroll_ms) + ms_to_samples(self.cfg.min_speech_ms) + frame.len();
            while self.preroll.len() > keep {
                self.preroll.pop_front();
            }
            self.onset_frames = if loud { self.onset_frames + 1 } else { 0 };
            if self.onset_frames >= self.frames(self.cfg.min_speech_ms) {
                self.in_speech = true;
                self.silence_frames = 0;
                self.seg = self.preroll.drain(..).collect();
                self.seg_start = end - self.seg.len() as u64;
                self.voiced = self.onset_frames as usize * frame.len();
            }
            return;
        }
        self.seg.extend_from_slice(frame);
        if loud {
            self.voiced += frame.len();
        }
        self.silence_frames = if loud { 0 } else { self.silence_frames + 1 };
        if self.silence_frames >= self.frames(self.cfg.min_silence_ms)
            || self.seg.len() >= ms_to_samples(self.cfg.max_segment_ms)
        {
            self.close();
        }
    }
}

impl Default for RmsSegmenter {
    fn default() -> Self {
        Self::new(RmsSegmenterConfig::default())
    }
}

impl Segmenter for RmsSegmenter {
    fn accept(&mut self, samples: &[f32]) {
        let n = ms_to_samples(self.cfg.frame_ms).max(1);
        for &s in samples {
            self.frame.push(s);
            if self.frame.len() == n {
                self.pos += n as u64;
                let frame = std::mem::take(&mut self.frame);
                self.on_frame(&frame);
                self.frame = frame;
                self.frame.clear();
            }
        }
    }

    fn pop(&mut self) -> Option<Segment> {
        self.out.pop_front()
    }

    fn flush(&mut self) {
        if self.in_speech {
            self.close();
        }
    }

    fn reset(&mut self) {
        let pos = self.pos + self.frame.len() as u64;
        *self = Self::new(self.cfg);
        self.pos = pos;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tone(ms: u32) -> Vec<f32> {
        (0..ms_to_samples(ms)).map(|i| if i % 2 == 0 { 0.3 } else { -0.3 }).collect()
    }
    fn silence(ms: u32) -> Vec<f32> {
        vec![0.0; ms_to_samples(ms)]
    }

    #[test]
    fn cuts_speech_between_silences() {
        let mut s = RmsSegmenter::default();
        s.accept(&silence(1000));
        s.accept(&tone(2000));
        assert!(s.pop().is_none(), "segment still open");
        s.accept(&silence(600));
        let seg = s.pop().expect("closed by silence");
        // pre-roll reaches back into the leading silence, never before 0
        assert!(seg.start_sample < ms_to_samples(1000) as u64);
        assert!(seg.start_sample >= ms_to_samples(700) as u64);
        assert!(seg.samples.len() >= ms_to_samples(2000));
        assert!(s.pop().is_none());
    }

    #[test]
    fn discards_clicks_and_splits_long_speech() {
        let mut s = RmsSegmenter::default();
        s.accept(&tone(200));
        s.accept(&silence(800));
        assert!(s.pop().is_none(), "200 ms click is below min_voiced_ms");
        s.accept(&tone(30_000));
        let mut n = 0;
        while let Some(seg) = s.pop() {
            assert!(seg.samples.len() <= ms_to_samples(12_000) + ms_to_samples(30));
            n += 1;
        }
        assert!(n >= 2, "max_segment_ms forces cuts");
    }

    #[test]
    fn flush_and_reset() {
        let mut s = RmsSegmenter::default();
        s.accept(&tone(1500));
        s.flush();
        assert!(s.pop().is_some());
        s.accept(&tone(1500));
        s.reset();
        s.flush();
        assert!(s.pop().is_none());
    }
}
