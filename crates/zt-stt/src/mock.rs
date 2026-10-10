//! Deterministic mock engine: emits scripted events once a given number of
//! samples has been fed. Lets the whole pipeline (audio -> stt -> align ->
//! core) run in tests and demo mode without models.

use crate::{Hotword, SpeechEngine, SttError, SttEvent};
use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq)]
pub struct ScriptedEvent {
    /// Emit after this many samples (16 kHz) have been accepted in total.
    pub at_sample: u64,
    pub event: SttEvent,
}

#[derive(Debug, Clone, Default)]
pub struct MockEngine {
    script: VecDeque<ScriptedEvent>,
    fed: u64,
    ready: Vec<SttEvent>,
    pub hotwords: Vec<Hotword>,
    pub resets: usize,
}

impl MockEngine {
    pub fn new(mut events: Vec<ScriptedEvent>) -> Self {
        events.sort_by_key(|e| e.at_sample);
        Self { script: events.into(), ..Default::default() }
    }

    /// Simulates reading `text` at `wps` words per second starting at `start_s`:
    /// growing partials per word, a final per sentence end ('.', '!', '?').
    pub fn reading(text: &str, start_s: f32, wps: f32) -> Self {
        let sr = crate::SAMPLE_RATE as f32;
        let mut events =
            vec![ScriptedEvent { at_sample: (start_s * sr) as u64, event: SttEvent::Vad(true) }];
        let mut utter: Vec<&str> = Vec::new();
        for (i, w) in text.split_whitespace().enumerate() {
            let at = ((start_s + (i + 1) as f32 / wps) * sr) as u64;
            utter.push(w);
            events.push(ScriptedEvent { at_sample: at, event: SttEvent::Partial(utter.join(" ")) });
            if w.ends_with(['.', '!', '?']) {
                events
                    .push(ScriptedEvent { at_sample: at, event: SttEvent::Final(utter.join(" ")) });
                utter.clear();
            }
        }
        if !utter.is_empty() {
            let last = events.last().map(|e| e.at_sample).unwrap_or(0);
            events.push(ScriptedEvent { at_sample: last, event: SttEvent::Final(utter.join(" ")) });
        }
        let end = events.last().map(|e| e.at_sample).unwrap_or(0);
        events.push(ScriptedEvent { at_sample: end, event: SttEvent::Vad(false) });
        Self::new(events)
    }

    pub fn samples_fed(&self) -> u64 {
        self.fed
    }
}

impl SpeechEngine for MockEngine {
    fn name(&self) -> &str {
        "mock"
    }

    fn accept_waveform(&mut self, samples: &[f32]) {
        self.fed += samples.len() as u64;
        while self.script.front().is_some_and(|e| e.at_sample <= self.fed) {
            if let Some(e) = self.script.pop_front() {
                self.ready.push(e.event);
            }
        }
    }

    fn poll(&mut self) -> Vec<SttEvent> {
        std::mem::take(&mut self.ready)
    }

    fn set_hotwords(&mut self, hotwords: &[Hotword]) -> Result<(), SttError> {
        self.hotwords = hotwords.to_vec();
        Ok(())
    }

    fn reset(&mut self) {
        self.resets += 1;
        self.ready.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emits_events_by_sample_position() {
        let mut m = MockEngine::reading("Hallo Welt. Wie geht's", 0.0, 2.0);
        m.accept_waveform(&[0.0; 8_000]); // 0.5 s -> first word
        assert_eq!(m.poll(), [SttEvent::Vad(true), SttEvent::Partial("Hallo".into())]);
        m.accept_waveform(&[0.0; 8_000]); // 1.0 s
        assert_eq!(
            m.poll(),
            [SttEvent::Partial("Hallo Welt.".into()), SttEvent::Final("Hallo Welt.".into())]
        );
        m.accept_waveform(&[0.0; 32_000]);
        let rest = m.poll();
        assert_eq!(rest.last(), Some(&SttEvent::Vad(false)));
        assert!(rest.contains(&SttEvent::Final("Wie geht's".into())));
        assert!(m.poll().is_empty());
    }
}
