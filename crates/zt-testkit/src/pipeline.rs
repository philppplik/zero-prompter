//! In-process voice pipeline: STT engine -> VoiceTracker -> Controller.
//!
//! Mirrors `apps/desktop/src-tauri/src/voice.rs` (event mapping) and
//! `appcore.rs::apply_tracker_output` (VoiceSignal translation, only on
//! change) but runs single-threaded on simulated time: every [`Pipeline::step`]
//! feeds one 50 ms chunk of silence samples (the MockEngine only counts them)
//! and advances the controller by the same 50 ms.

use crate::builders::ControllerBuilder;
use zt_align::{
    Lang, ScriptIndex, TrackerConfig, TrackerEvent, TrackerOutput, VoiceState, VoiceTracker,
};
use zt_core::{Command, Controller, VoicePhase, VoiceSignal};
use zt_stt::{build_hotwords, MockEngine, SpeechEngine, SttEvent, SAMPLE_RATE};

pub const CHUNK_MS: u64 = 50;
pub const CHUNK_SAMPLES: usize = (SAMPLE_RATE as u64 * CHUNK_MS / 1000) as usize;

/// Same mapping as `appcore::phase_of`.
pub fn phase_of(v: VoiceState) -> VoicePhase {
    match v {
        VoiceState::Idle => VoicePhase::Idle,
        VoiceState::Listening => VoicePhase::Listening,
        VoiceState::Tracking => VoicePhase::Tracking,
        VoiceState::Hold => VoicePhase::Hold,
        VoiceState::Lost => VoicePhase::Lost,
        VoiceState::Resync => VoicePhase::Resync,
    }
}

pub fn voice_signal(out: TrackerOutput) -> VoiceSignal {
    VoiceSignal {
        phase: phase_of(out.state),
        cursor: out.cursor_token,
        confidence: out.confidence,
        rate_wps: out.rate_wps,
    }
}

/// Same mapping as the voice thread in `voice.rs`.
pub fn tracker_event(ev: SttEvent) -> TrackerEvent {
    match ev {
        SttEvent::Partial(text) => TrackerEvent::Hypothesis { text, is_final: false },
        SttEvent::Final(text) => TrackerEvent::Hypothesis { text, is_final: true },
        SttEvent::Vad(true) => TrackerEvent::VadSpeech,
        SttEvent::Vad(false) => TrackerEvent::VadSilence,
        SttEvent::Resync { text, age_ms } => TrackerEvent::Resync { text, age_ms },
    }
}

#[derive(Debug)]
pub struct Pipeline<E: SpeechEngine = MockEngine> {
    pub engine: E,
    pub tracker: VoiceTracker,
    pub ctl: Controller,
    /// Simulated time in ms (derived from the samples fed).
    pub t_ms: u64,
    /// Every tracker output with its timestamp, in order.
    pub outputs: Vec<(u64, TrackerOutput)>,
    last_sent: Option<TrackerOutput>,
}

impl<E: SpeechEngine> Pipeline<E> {
    /// Controller with `script` loaded, voice on, playing; tracker started;
    /// hotwords from the script applied to the engine (like the app does).
    pub fn new(script: &str, engine: E) -> Self {
        Self::with(
            ControllerBuilder::new().script(script).voice().playing().build(),
            TrackerConfig::default(),
            engine,
        )
    }

    pub fn with(ctl: Controller, cfg: TrackerConfig, mut engine: E) -> Self {
        let state = ctl.state();
        let index = ScriptIndex::build(state.script.spoken_tokens(), Lang::De);
        let words: Vec<&str> = state.script.spoken_tokens().map(|(_, w)| w).collect();
        engine.set_hotwords(&build_hotwords(words, 5, 200, 1.5)).expect("hotwords");
        let mut tracker = VoiceTracker::new(index, cfg);
        tracker.handle(TrackerEvent::Start, 0);
        Self { engine, tracker, ctl, t_ms: 0, outputs: Vec::new(), last_sent: None }
    }

    /// One 50 ms chunk: audio -> STT -> tracker -> controller, then a controller tick.
    pub fn step(&mut self) {
        self.engine.accept_waveform(&[0.0; CHUNK_SAMPLES]);
        self.t_ms += CHUNK_MS;
        for ev in self.engine.poll() {
            self.feed(tracker_event(ev));
        }
        // Time-only transitions (resync ease, lost timeout) are forwarded too,
        // so controller and tracker never disagree about the phase.
        self.feed(TrackerEvent::Tick);
        self.ctl.apply(Command::Tick(CHUNK_MS as u32)).expect("tick");
    }

    /// Feeds one tracker event at the current time and forwards a changed
    /// output to the controller (like `appcore::apply_tracker_output`).
    pub fn feed(&mut self, ev: TrackerEvent) -> TrackerOutput {
        let out = self.tracker.handle(ev, self.t_ms);
        self.outputs.push((self.t_ms, out));
        if self.last_sent != Some(out) {
            self.last_sent = Some(out);
            self.ctl.apply(Command::Voice(voice_signal(out))).expect("voice signal never fails");
        }
        out
    }

    pub fn run_ms(&mut self, ms: u64) {
        for _ in 0..ms.div_ceil(CHUNK_MS) {
            self.step();
        }
    }

    pub fn run_secs(&mut self, secs: f32) {
        self.run_ms((secs.max(0.0) * 1000.0) as u64);
    }

    pub fn cursor(&self) -> Option<usize> {
        self.tracker.output().cursor_token
    }

    pub fn state(&self) -> VoiceState {
        self.tracker.state()
    }

    /// Cursor tokens in the order they were reported (deduplicated).
    pub fn cursor_trace(&self) -> Vec<usize> {
        let mut v: Vec<usize> = Vec::new();
        for c in self.outputs.iter().filter_map(|(_, o)| o.cursor_token) {
            if v.last() != Some(&c) {
                v.push(c);
            }
        }
        v
    }

    pub fn visited(&self, s: VoiceState) -> bool {
        self.outputs.iter().any(|(_, o)| o.state == s)
    }
}
