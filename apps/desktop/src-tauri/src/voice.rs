//! Voice pipeline adapter: microphone (zt-audio) -> STT (zt-stt) -> tracker (zt-align)
//! -> `Command::Voice` (zt-core). Runs on its own thread; the cpal stream is
//! created *inside* that thread, so `cpal::Stream: !Send` is never an issue.
//!
//! Two modes share the same thread:
//! - `tracking = true`  : voice tracking during Live (fed into the tracker);
//! - `tracking = false` : microphone test (onboarding / inspector), only emits
//!   `voice://level` and `voice://transcript`.
//!
//! Without the `sherpa` feature (or without an installed model) the engine is
//! `MockEngine::reading(script)` - a deterministic demo that "reads" the script
//! at ~2.3 words/s, so the full UI path can be exercised on any machine.
//!
//! With models (ADR-006) the engine is a zt_stt::HybridEngine: the streaming
//! model chosen per language (user pick -> manifest default -> fallback chain)
//! plus, if enabled, offline resync (Parakeet) on its own zt-resync thread;
//! its SttEvent::Resync results re-anchor the tracker via VoiceTracker::resync.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{sync_channel, RecvTimeoutError};
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::Serialize;
use tauri::{AppHandle, Emitter};
use zt_align::{ResyncOutcome, TrackerEvent};
use zt_audio::{rms_dbfs, AudioSource};
use zt_stt::{build_hotwords, MockEngine, SpeechEngine, SttEvent};

use crate::appcore::{apply_tracker_output, Shared};

pub const EV_LEVEL: &str = "voice://level";
pub const EV_TRANSCRIPT: &str = "voice://transcript";
pub const EV_ENGINE: &str = "voice://engine";
pub const EV_RESYNC: &str = "voice://resync";

#[derive(Debug)]
pub struct VoiceRuntime {
    stop: Arc<AtomicBool>,
    pub tracking: bool,
}

impl VoiceRuntime {
    fn shutdown(&self) {
        self.stop.store(true, Ordering::SeqCst);
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct Level {
    db: f32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct Transcript {
    text: String,
    is_final: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineInfo {
    pub name: String,
    pub demo: bool,
    pub microphone: bool,
    pub error: Option<String>,
    /// Offline resync running (ADR-006).
    pub resync: bool,
    /// Fallback / resync notes, e.g. "zipformer-en-kroko-2025-08-06 failed, fallback".
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ResyncInfo {
    text: String,
    age_ms: u64,
    /// "ignored" | "noMatch" | "confirmed" | "reanchored"
    outcome: &'static str,
}

/// Starts/stops tracking mode (called after `ToggleVoice`).
pub fn set_tracking(app: &AppHandle, shared: &Shared, on: bool) {
    stop(shared);
    if on {
        start(app, shared, true);
        let mut g = shared.lock();
        let t = g.now_ms();
        g.tracker.handle(TrackerEvent::Start, t);
    } else {
        let mut g = shared.lock();
        let t = g.now_ms();
        g.tracker.handle(TrackerEvent::Stop, t);
    }
}

/// Microphone test (no tracking). Refused while tracking is active.
pub fn set_mic_test(app: &AppHandle, shared: &Shared, on: bool) {
    let tracking = shared.lock().voice_rt.as_ref().is_some_and(|r| r.tracking);
    if tracking {
        return;
    }
    stop(shared);
    if on {
        start(app, shared, false);
    }
}

pub fn stop(shared: &Shared) {
    if let Some(rt) = shared.lock().voice_rt.take() {
        rt.shutdown();
    }
}

fn start(app: &AppHandle, shared: &Shared, tracking: bool) {
    let stop = Arc::new(AtomicBool::new(false));
    let (words, lang, device, prefs) = {
        let mut g = shared.lock();
        g.voice_rt = Some(VoiceRuntime { stop: stop.clone(), tracking });
        (
            g.spoken_words(),
            g.ctl.state().settings.language.clone(),
            g.audio_device.clone(),
            g.voice_models.clone(),
        )
    };
    let app = app.clone();
    let shared = shared.clone();
    let spawned = std::thread::Builder::new().name("zt-voice".into()).spawn(move || {
        run(app, shared, stop, tracking, words, lang, device, prefs);
    });
    if let Err(e) = spawned {
        eprintln!("zt: cannot spawn voice thread: {e}");
    }
}

struct MadeEngine {
    engine: Box<dyn SpeechEngine>,
    demo: bool,
    resync: bool,
    error: Option<String>,
    notes: Vec<String>,
}

fn make_engine(
    app: &AppHandle,
    lang: &str,
    words: &[String],
    prefs: &crate::models::VoiceModelPrefs,
) -> MadeEngine {
    let demo = |error: Option<String>, notes: Vec<String>| MadeEngine {
        engine: Box::new(MockEngine::reading(&words.join(" "), 1.0, 2.3)),
        demo: true,
        resync: false,
        error,
        notes,
    };
    #[cfg(feature = "sherpa")]
    {
        match crate::models::build_engine(app, lang, prefs) {
            Ok(Some(b)) => {
                return MadeEngine {
                    engine: b.engine,
                    demo: false,
                    resync: b.resync,
                    error: None,
                    notes: b.notes,
                }
            }
            Ok(None) => {}
            Err(err) => return demo(Some(err), vec![]),
        }
    }
    let _ = (app, lang, prefs);
    demo(None, vec![])
}

#[cfg(feature = "cpal")]
fn make_source() -> Option<Box<dyn AudioSource>> {
    Some(Box::new(zt_audio::CpalSource::new()))
}

#[cfg(not(feature = "cpal"))]
fn make_source() -> Option<Box<dyn AudioSource>> {
    None
}

fn run(
    app: AppHandle,
    shared: Shared,
    stop: Arc<AtomicBool>,
    tracking: bool,
    words: Vec<String>,
    lang: String,
    device: Option<String>,
    prefs: crate::models::VoiceModelPrefs,
) {
    let MadeEngine { mut engine, demo, resync, error: engine_err, notes } =
        make_engine(&app, &lang, &words, &prefs);
    let hotwords = build_hotwords(words.iter().map(String::as_str), 5, 200, 1.5);
    if let Err(e) = engine.set_hotwords(&hotwords) {
        eprintln!("zt: hotwords not applied: {e}");
    }

    // Audio callback -> bounded channel (try_send: never blocks the audio thread).
    let (tx, rx) = sync_channel::<Vec<f32>>(64);
    let mut source = make_source();
    let mut mic_ok = false;
    let mut mic_err = None;
    if let Some(src) = source.as_mut() {
        let tx2 = tx.clone();
        match src.start(device.as_deref(), Box::new(move |f: &[f32]| {
            let _ = tx2.try_send(f.to_vec());
        })) {
            Ok(()) => mic_ok = true,
            Err(e) => mic_err = Some(e.to_string()),
        }
    }
    let _ = app.emit(
        EV_ENGINE,
        EngineInfo {
            name: engine.name().to_string(),
            demo,
            microphone: mic_ok,
            error: engine_err.or(mic_err),
            resync,
            notes,
        },
    );

    let mut level_acc: Vec<f32> = Vec::with_capacity(2048);
    let mut last_level = Instant::now();
    while !stop.load(Ordering::SeqCst) {
        let frame = match rx.recv_timeout(Duration::from_millis(50)) {
            Ok(f) => f,
            // No microphone: feed realtime silence so demo/mock engines advance.
            Err(RecvTimeoutError::Timeout) if !mic_ok => vec![0.0; 800],
            Err(RecvTimeoutError::Timeout) => continue,
            Err(RecvTimeoutError::Disconnected) => break,
        };
        level_acc.extend_from_slice(&frame);
        if last_level.elapsed() >= Duration::from_millis(66) {
            let _ = app.emit(EV_LEVEL, Level { db: rms_dbfs(&level_acc) });
            level_acc.clear();
            last_level = Instant::now();
        }
        engine.accept_waveform(&frame);
        for ev in engine.poll() {
            let tev = match ev {
                SttEvent::Partial(text) => {
                    let _ = app.emit(
                        EV_TRANSCRIPT,
                        Transcript { text: text.clone(), is_final: false },
                    );
                    TrackerEvent::Hypothesis { text, is_final: false }
                }
                SttEvent::Final(text) => {
                    let _ = app.emit(
                        EV_TRANSCRIPT,
                        Transcript { text: text.clone(), is_final: true },
                    );
                    TrackerEvent::Hypothesis { text, is_final: true }
                }
                SttEvent::Vad(true) => TrackerEvent::VadSpeech,
                SttEvent::Vad(false) => TrackerEvent::VadSilence,
                SttEvent::Resync { text, age_ms } => {
                    if tracking {
                        resync_event(&app, &shared, text, age_ms);
                    }
                    continue;
                }
            };
            if tracking {
                let out = {
                    let mut g = shared.lock();
                    let t = g.now_ms();
                    g.tracker.handle(tev, t)
                };
                apply_tracker_output(&app, &shared, out);
            }
        }
    }
    if let Some(src) = source.as_mut() {
        src.stop();
    }
    drop(tx);
    let _ = app.emit(EV_LEVEL, Level { db: -100.0 });
}

/// Feeds an offline resync hypothesis into the tracker (ADR-006). Runs on the
/// voice thread after `poll()`; decoding itself happened on `zt-resync`.
fn resync_event(app: &AppHandle, shared: &Shared, text: String, age_ms: u64) {
    let (out, outcome) = {
        let mut g = shared.lock();
        let t = g.now_ms();
        let outcome = g.tracker.resync(&text, age_ms, t);
        (g.tracker.output(), outcome)
    };
    let outcome = match outcome {
        ResyncOutcome::Ignored => "ignored",
        ResyncOutcome::NoMatch => "noMatch",
        ResyncOutcome::Confirmed => "confirmed",
        ResyncOutcome::Reanchored { .. } => "reanchored",
    };
    let _ = app.emit(EV_RESYNC, ResyncInfo { text, age_ms, outcome });
    apply_tracker_output(app, shared, out);
}
