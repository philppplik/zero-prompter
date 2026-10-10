//! Simulated speaker for the voice tracker: explicit clock, growing partials.

use zt_align::{
    Lang, ScriptIndex, TrackerConfig, TrackerEvent, TrackerOutput, VoiceState, VoiceTracker,
};

/// Gap between two growing partial hypotheses (≈ 4 words/s).
pub const PARTIAL_GAP_MS: u64 = 250;
/// Gap between the last partial and the final of an utterance.
pub const FINAL_GAP_MS: u64 = 50;

/// Index over whitespace tokens (`token id == word position`), as long as
/// every word normalises to exactly one word (true for [`crate::SCRIPT_DE`]).
pub fn index_of(script: &str, lang: Lang) -> ScriptIndex {
    ScriptIndex::build(script.split_whitespace().enumerate(), lang)
}

#[derive(Debug, Clone)]
pub struct Speaker {
    pub tracker: VoiceTracker,
    /// Simulated monotonic time in ms.
    pub t: u64,
}

impl Speaker {
    /// Tracker over `script` (German), *not* started (state `Idle`).
    pub fn idle(script: &str, cfg: TrackerConfig) -> Self {
        Self { tracker: VoiceTracker::new(index_of(script, Lang::De), cfg), t: 0 }
    }

    /// Tracker over `script`, already started (state `Listening`).
    pub fn new(script: &str, cfg: TrackerConfig) -> Self {
        let mut s = Self::idle(script, cfg);
        s.ev(TrackerEvent::Start, 0);
        s
    }

    /// The default reference script with the default configuration, started.
    pub fn default_de() -> Self {
        Self::new(crate::SCRIPT_DE, TrackerConfig::default())
    }

    /// Advances the clock by `dt` ms, then feeds `e`.
    pub fn ev(&mut self, e: TrackerEvent, dt: u64) -> TrackerOutput {
        self.t += dt;
        self.tracker.handle(e, self.t)
    }

    pub fn partial(&mut self, text: &str, dt: u64) -> TrackerOutput {
        self.ev(TrackerEvent::Hypothesis { text: text.to_string(), is_final: false }, dt)
    }

    pub fn final_(&mut self, text: &str, dt: u64) -> TrackerOutput {
        self.ev(TrackerEvent::Hypothesis { text: text.to_string(), is_final: true }, dt)
    }

    /// Speaks `text` word by word (growing partials, [`PARTIAL_GAP_MS`] apart)
    /// and closes it with a final. Returns every intermediate output
    /// (`len == words + 1`, the last one belongs to the final).
    pub fn say(&mut self, text: &str) -> Vec<TrackerOutput> {
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut outs = Vec::with_capacity(words.len() + 1);
        for n in 1..=words.len() {
            outs.push(self.partial(&words[..n].join(" "), PARTIAL_GAP_MS));
        }
        outs.push(self.final_(text, FINAL_GAP_MS));
        outs
    }

    /// VAD reports silence, then `ms` of time pass.
    pub fn silence(&mut self, ms: u64) -> TrackerOutput {
        self.ev(TrackerEvent::VadSilence, 0);
        self.tick(ms)
    }

    pub fn tick(&mut self, ms: u64) -> TrackerOutput {
        self.ev(TrackerEvent::Tick, ms)
    }

    pub fn seek(&mut self, token: usize) -> TrackerOutput {
        self.ev(TrackerEvent::ManualSeek { token }, 0)
    }

    pub fn out(&self) -> TrackerOutput {
        self.tracker.output()
    }

    pub fn state(&self) -> VoiceState {
        self.tracker.state()
    }

    pub fn cursor(&self) -> Option<usize> {
        self.out().cursor_token
    }

    /// Asserts state and cursor with a readable message.
    #[track_caller]
    pub fn assert_at(&self, state: VoiceState, cursor: Option<usize>) {
        let o = self.out();
        assert_eq!((o.state, o.cursor_token), (state, cursor), "unexpected tracker output: {o:?}");
    }
}
