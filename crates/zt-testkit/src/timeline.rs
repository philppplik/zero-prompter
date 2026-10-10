//! Readable builder for deterministic STT event streams.
//!
//! ```
//! use zt_testkit::SpeechTimeline;
//! let engine = SpeechTimeline::new(2.0)
//!     .say("Willkommen zu meinem Kanal")
//!     .pause(1.5)
//!     .say("Heute zeige ich euch")
//!     .build();
//! # let _ = engine;
//! ```

use zt_stt::{MockEngine, ScriptedEvent, SttEvent, SAMPLE_RATE};

#[derive(Debug, Clone)]
pub struct SpeechTimeline {
    /// Speaking rate in words per second.
    wps: f32,
    /// Current time in seconds.
    t: f32,
    events: Vec<ScriptedEvent>,
}

impl SpeechTimeline {
    pub fn new(wps: f32) -> Self {
        assert!(wps > 0.0, "wps must be positive");
        Self { wps, t: 0.0, events: Vec::new() }
    }

    fn push(&mut self, at_s: f32, event: SttEvent) {
        let at_sample = (at_s.max(0.0) * SAMPLE_RATE as f32).round() as u64;
        self.events.push(ScriptedEvent { at_sample, event });
    }

    /// One utterance: VAD on, a growing partial per word, final, VAD off.
    pub fn say(mut self, text: &str) -> Self {
        let words: Vec<&str> = text.split_whitespace().collect();
        if words.is_empty() {
            return self;
        }
        self.push(self.t, SttEvent::Vad(true));
        for n in 1..=words.len() {
            self.t += 1.0 / self.wps;
            self.push(self.t, SttEvent::Partial(words[..n].join(" ")));
        }
        self.push(self.t, SttEvent::Final(words.join(" ")));
        self.push(self.t, SttEvent::Vad(false));
        self
    }

    /// Speech that the STT recognises but that is not in the script.
    pub fn ad_lib(self, text: &str) -> Self {
        self.say(text)
    }

    /// A partial that is later revised by the final (STT self-correction).
    pub fn revised(mut self, partial: &str, final_text: &str) -> Self {
        self.push(self.t, SttEvent::Vad(true));
        self.t += 1.0 / self.wps;
        self.push(self.t, SttEvent::Partial(partial.into()));
        self.t += 1.0 / self.wps;
        self.push(self.t, SttEvent::Final(final_text.into()));
        self.push(self.t, SttEvent::Vad(false));
        self
    }

    /// Silence (no events) for `secs`.
    pub fn pause(mut self, secs: f32) -> Self {
        self.t += secs.max(0.0);
        self
    }

    /// Time of the last event in seconds.
    pub fn end_s(&self) -> f32 {
        self.t
    }

    pub fn events(&self) -> &[ScriptedEvent] {
        &self.events
    }

    pub fn build(self) -> MockEngine {
        MockEngine::new(self.events)
    }
}
