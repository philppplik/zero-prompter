//! Voice pipeline adapter: microphone (zt-audio) -> STT (zt-stt) -> tracker (zt-align)
//! -> `Command::Voice` (zt-core). Runs on its own thread; the cpal stream is
//! created *inside* that thread, so `cpal::Stream: !Send` is never an issue.
//!
//! Two modes share the same thread:
//! - `tracking = true`  : voice tracking during Live (fed into the tracker);
//! - `tracking = false` : microphone test (onboarding / settings), only emits
//!   `voice://level` and `voice://transcript`.
//!
//! Engine choice (no silent fallbacks, ADR-007):
//! - real engine: zt_stt::HybridEngine with the streaming model for the
//!   script language (user pick -> manifest default -> fallback chain ->
//!   installed model of another language, reported as a note);
//! - demo (`MockEngine::reading`, auto-reads the script) ONLY when no
//!   streaming model is installed at all or the build has no `sherpa`
//!   feature; `EngineInfo.reason` says which, the UI shows "Modell laden";
//! - engine construction failed although models are installed: no demo, a
//!   silent `NullEngine` + error (toast + log + diagnostics panel), so the
//!   text never scrolls by itself pretending to follow the speaker.
//!
//! Every pipeline stage logs structured `tracing` events (targets `zt::*`)
//! and feeds latency metrics into `diag` (docs/book/src/user/voice-tracking.md).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{sync_channel, RecvTimeoutError};
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::Serialize;
use tauri::{AppHandle, Emitter};
use zt_align::{ResyncOutcome, TrackerEvent, VoiceState};
use zt_audio::{rms_dbfs, Agc, AudioSource, ChannelMode, GainMode, OnsetConfig, OnsetDetector};
use zt_core::metrics::RateCounter;
use zt_stt::{build_hotwords, Hotword, MockEngine, SpeechEngine, SttError, SttEvent};

use crate::appcore::{apply_tracker_output, Shared};
use crate::diag;

pub const EV_LEVEL: &str = "voice://level";
pub const EV_TRANSCRIPT: &str = "voice://transcript";
pub const EV_ENGINE: &str = "voice://engine";
pub const EV_RESYNC: &str = "voice://resync";

/// After this long without any non-zero sample the microphone is reported.
const MIC_SILENCE_REPORT: Duration = Duration::from_secs(3);
/// Speech level (p90 of the raw input over QUIET_WINDOW) below which the
/// microphone is reported as very quiet.
const QUIET_DBFS: f32 = -50.0;
/// Observation window of the quiet-microphone check.
const QUIET_WINDOW: Duration = Duration::from_secs(10);
/// How often the gain / channel settings are re-read while running.
const SETTINGS_POLL: Duration = Duration::from_millis(500);

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
    /// Level after gain (what the speech engine gets).
    db: f32,
    /// Level of the microphone itself (before gain).
    raw_db: f32,
    /// Applied digital gain (dB).
    gain_db: f32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct Transcript {
    text: String,
    is_final: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EngineInfo {
    pub name: String,
    /// Demo engine (auto-reads the script); only with reason "noModel"/"noSherpa".
    pub demo: bool,
    /// Why there is no real engine: "noModel" (no streaming model installed),
    /// "noSherpa" (build without STT), "engineError" (model failed to load).
    pub reason: Option<String>,
    pub microphone: bool,
    pub error: Option<String>,
    /// Offline resync running (ADR-006).
    pub resync: bool,
    /// Fallback / resync notes, e.g. "zipformer-en-kroko-2025-08-06 failed, fallback".
    pub notes: Vec<String>,
    /// Primary script language the engine was chosen for.
    pub language: String,
    /// Audio stream description (device, native format).
    pub audio: Option<String>,
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

/// Restarts a running voice session so a new model / setting takes effect
/// (after a model install or a model choice change).
pub fn restart_if_running(app: &AppHandle, shared: &Shared) {
    let running = shared.lock().voice_rt.as_ref().map(|r| r.tracking);
    match running {
        Some(true) => set_tracking(app, shared, true),
        Some(false) => set_mic_test(app, shared, true),
        None => {}
    }
}

pub fn stop(shared: &Shared) {
    if let Some(rt) = shared.lock().voice_rt.take() {
        rt.shutdown();
    }
}

/// Everything the voice thread needs, moved in at start.
struct Ctx {
    app: AppHandle,
    shared: Shared,
    stop: Arc<AtomicBool>,
    tracking: bool,
    words: Vec<String>,
    lang: String,
    device: Option<String>,
    prefs: crate::models::VoiceModelPrefs,
    gain: GainMode,
    channel: ChannelMode,
}

/// Gain + channel mode from the settings (micGainAuto, micGainDb, micChannel).
pub fn mic_config(s: &zt_core::Settings) -> (GainMode, ChannelMode) {
    let gain = if s.mic_gain_auto { GainMode::Auto } else { GainMode::Manual(s.mic_gain_db) };
    (gain, ChannelMode::parse(&s.mic_channel))
}

fn start(app: &AppHandle, shared: &Shared, tracking: bool) {
    let stop = Arc::new(AtomicBool::new(false));
    let ctx = {
        let mut g = shared.lock();
        g.voice_rt = Some(VoiceRuntime { stop: stop.clone(), tracking });
        let (gain, channel) = mic_config(&g.ctl.state().settings);
        Ctx {
            app: app.clone(),
            shared: shared.clone(),
            stop,
            tracking,
            words: g.spoken_words(),
            lang: g.ctl.state().settings.language.clone(),
            device: g.audio_device.clone(),
            prefs: g.voice_models.clone(),
            gain,
            channel,
        }
    };
    let spawned = std::thread::Builder::new().name("zt-voice".into()).spawn(move || {
        // A panic must not vanish with the thread: report it (toast + log).
        let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run(&ctx)));
        if let Err(p) = res {
            let msg = p
                .downcast_ref::<&str>()
                .map(|s| s.to_string())
                .or_else(|| p.downcast_ref::<String>().cloned())
                .unwrap_or_else(|| "unbekannter Fehler".into());
            let text = format!("Voice-Thread abgestürzt: {msg}");
            diag::report_error(&ctx.app, "voice-thread", text);
        }
        let mut d = diag::state();
        d.running = false;
        d.level_db = -100.0;
        d.raw_level_db = -100.0;
        d.channel_db.clear();
        d.active_channel = None;
    });
    if let Err(e) = spawned {
        diag::report_error(app, "voice-thread", format!("Voice-Thread nicht startbar: {e}"));
    }
}

/// Engine used when a real engine was expected but failed to load: produces
/// nothing, so the prompter never moves on its own (unlike the demo engine).
#[derive(Debug)]
#[cfg_attr(not(feature = "sherpa"), allow(dead_code))]
struct NullEngine;

impl SpeechEngine for NullEngine {
    fn name(&self) -> &str {
        "keine Engine (Fehler)"
    }
    fn accept_waveform(&mut self, _samples: &[f32]) {}
    fn poll(&mut self) -> Vec<SttEvent> {
        Vec::new()
    }
    fn set_hotwords(&mut self, _hotwords: &[Hotword]) -> Result<(), SttError> {
        Ok(())
    }
    fn reset(&mut self) {}
}

struct MadeEngine {
    engine: Box<dyn SpeechEngine>,
    demo: bool,
    reason: Option<&'static str>,
    resync: bool,
    error: Option<String>,
    notes: Vec<String>,
}

fn demo_engine(ctx: &Ctx, reason: &'static str, notes: Vec<String>) -> MadeEngine {
    MadeEngine {
        engine: Box::new(MockEngine::reading(&ctx.words.join(" "), 1.0, 2.3)),
        demo: true,
        reason: Some(reason),
        resync: false,
        error: None,
        notes,
    }
}

#[cfg(feature = "sherpa")]
fn make_engine(ctx: &Ctx) -> MadeEngine {
    match crate::models::build_engine(&ctx.app, &ctx.lang, &ctx.prefs) {
        Ok(Some(b)) => MadeEngine {
            engine: b.engine,
            demo: false,
            reason: None,
            resync: b.resync,
            error: None,
            notes: b.notes,
        },
        Ok(None) => {
            tracing::warn!(target: "zt::engine", lang = %ctx.lang,
                "no streaming model installed -> demo mode");
            demo_engine(ctx, "noModel", vec![])
        }
        Err(err) => MadeEngine {
            engine: Box::new(NullEngine),
            demo: false,
            reason: Some("engineError"),
            resync: false,
            error: Some(format!("Sprachmodell konnte nicht geladen werden: {err}")),
            notes: vec![],
        },
    }
}

#[cfg(not(feature = "sherpa"))]
fn make_engine(ctx: &Ctx) -> MadeEngine {
    tracing::warn!(target: "zt::engine", "build without `sherpa` feature -> demo mode");
    let note = "Dieser Build enthält keine Spracherkennung (sherpa).".to_string();
    demo_engine(ctx, "noSherpa", vec![note])
}

type Frame = (Instant, Vec<f32>);

#[cfg(feature = "cpal")]
fn make_source() -> Option<Box<dyn AudioSource>> {
    Some(Box::new(zt_audio::CpalSource::new()))
}

#[cfg(not(feature = "cpal"))]
fn make_source() -> Option<Box<dyn AudioSource>> {
    None
}

fn state_name(s: VoiceState) -> &'static str {
    match s {
        VoiceState::Idle => "idle",
        VoiceState::Listening => "listening",
        VoiceState::Tracking => "tracking",
        VoiceState::Hold => "hold",
        VoiceState::Lost => "lost",
        VoiceState::Resync => "resync",
    }
}

fn run(ctx: &Ctx) {
    let app = &ctx.app;
    let t_start = Instant::now();
    tracing::info!(target: "zt::voice", tracking = ctx.tracking, lang = %ctx.lang,
        device = ?ctx.device, prefs = ?ctx.prefs, script_words = ctx.words.len(), "voice start");
    {
        let mut d = diag::state();
        d.running = true;
        d.tracking = ctx.tracking;
        d.last_partial = None;
    }
    diag::reset_latencies();

    let MadeEngine { mut engine, demo, reason, resync, error: engine_err, notes } =
        make_engine(ctx);
    tracing::info!(target: "zt::engine", engine = engine.name(), demo, reason = ?reason,
        resync, notes = ?notes, ms = t_start.elapsed().as_millis() as u64, "engine selected");
    for n in &notes {
        diag::record_warning("stt", n.clone());
    }
    if let Some(e) = &engine_err {
        diag::report_error(app, "stt", e.clone());
    }
    let hotwords = build_hotwords(ctx.words.iter().map(String::as_str), 5, 200, 1.5);
    match engine.set_hotwords(&hotwords) {
        Ok(()) => tracing::info!(target: "zt::stt", count = hotwords.len(), "hotwords applied"),
        // Models without a usable bpe vocab, or sherpa rejected the hotwords
        // (the engine then keeps running without biasing; see zt::engine log).
        Err(e) => tracing::info!(target: "zt::stt", error = %e, "hotwords not applied"),
    }

    // Audio callback -> bounded channel (try_send: never blocks the audio thread).
    let (tx, rx) = sync_channel::<Frame>(64);
    let mut source = make_source();
    let mut mic_ok = false;
    let mut mic_err = None;
    let mut audio_desc = None;
    if let Some(src) = source.as_mut() {
        src.set_channel_mode(ctx.channel);
        let tx2 = tx.clone();
        match src.start(
            ctx.device.as_deref(),
            Box::new(move |f: &[f32]| {
                let _ = tx2.try_send((Instant::now(), f.to_vec()));
            }),
        ) {
            Ok(()) => {
                mic_ok = true;
                audio_desc = src.describe();
                tracing::info!(target: "zt::audio", stream = ?audio_desc, gain = ?ctx.gain,
                    channel = ?ctx.channel, "microphone started");
            }
            Err(e) => {
                let msg = format!("Mikrofon konnte nicht geöffnet werden: {e}");
                diag::report_error(app, "audio", msg.clone());
                mic_err = Some(msg);
            }
        }
    } else {
        diag::record_warning("audio", "Build ohne Mikrofon-Unterstützung (cpal)");
    }
    let info = EngineInfo {
        name: engine.name().to_string(),
        demo,
        reason: reason.map(str::to_string),
        microphone: mic_ok,
        error: engine_err.or(mic_err),
        resync,
        notes,
        language: zt_models::primary_language(&ctx.lang),
        audio: audio_desc.clone(),
    };
    {
        let mut d = diag::state();
        d.engine = Some(info.clone());
        d.audio = audio_desc;
    }
    let _ = app.emit(EV_ENGINE, info);

    let mut level_acc: Vec<f32> = Vec::with_capacity(2048);
    let mut raw_acc: Vec<f32> = Vec::with_capacity(2048);
    let mut agc = Agc::new(ctx.gain, zt_audio::TARGET_RATE);
    let mut channel = ctx.channel;
    let mut last_settings = Instant::now();
    let mut quiet = QuietWatch::new();
    let mut last_level = Instant::now();
    let mut fps = RateCounter::default();
    let mut mic_watch = MicWatch::new();
    // Speech onset (capture time of the first voiced block, relative to the
    // tracked noise floor; re-armed after silence) for onset->partial.
    let mut onset = OnsetDetector::<Instant>::new(OnsetConfig::default());
    let mut last_state = VoiceState::Idle;

    while !ctx.stop.load(Ordering::SeqCst) {
        let (captured, mut frame) = match rx.recv_timeout(Duration::from_millis(50)) {
            Ok(f) => f,
            // No microphone: feed realtime silence so demo/mock engines advance.
            Err(RecvTimeoutError::Timeout) if !mic_ok => (Instant::now(), vec![0.0; 800]),
            Err(RecvTimeoutError::Timeout) => {
                if let Some(msg) = mic_watch.check() {
                    diag::report_error(app, "audio", msg);
                }
                continue;
            }
            Err(RecvTimeoutError::Disconnected) => break,
        };
        if mic_ok {
            mic_watch.frame(&frame);
            if let Some(msg) = mic_watch.check() {
                diag::report_error(app, "audio", msg);
            }
        }
        if last_settings.elapsed() >= SETTINGS_POLL {
            last_settings = Instant::now();
            let (g, ch) = mic_config(&ctx.shared.lock().ctl.state().settings);
            if g != agc.mode() {
                tracing::info!(target: "zt::audio", gain = ?g, "mic gain changed");
                agc.set_mode(g);
            }
            if ch != channel {
                tracing::info!(target: "zt::audio", channel = ?ch, "mic channel mode changed");
                channel = ch;
                if let Some(src) = source.as_mut() {
                    src.set_channel_mode(ch);
                }
            }
        }
        raw_acc.extend_from_slice(&frame);
        if mic_ok {
            agc.process(&mut frame);
        }
        let frame_db = rms_dbfs(&frame);
        onset.push(frame_db, frame.len(), captured);
        level_acc.extend_from_slice(&frame);
        if let Some(rate) = fps.hit(t_start.elapsed().as_millis() as u64) {
            let db = rms_dbfs(&level_acc);
            let raw = rms_dbfs(&raw_acc);
            let ch = source.as_ref().and_then(|s| s.channel_levels());
            let ch_db: Vec<f32> = ch.as_ref().map(|c| c.dbfs.clone()).unwrap_or_default();
            let active = ch.as_ref().and_then(|c| c.active);
            tracing::debug!(target: "zt::audio", frames_per_s = rate, rms_dbfs = db, raw_dbfs = raw,
                gain_db = agc.gain_db(), channels_dbfs = ?ch_db, active_channel = ?active,
                "audio stats");
            let mut d = diag::state();
            d.frames_per_s = rate;
            d.channel_db = ch_db;
            d.active_channel = active;
        }
        if last_level.elapsed() >= Duration::from_millis(66) {
            let db = rms_dbfs(&level_acc);
            let raw_db = rms_dbfs(&raw_acc);
            let gain_db = agc.gain_db();
            {
                let mut d = diag::state();
                d.level_db = db;
                d.raw_level_db = raw_db;
                d.gain_db = gain_db;
            }
            let _ = app.emit(EV_LEVEL, Level { db, raw_db, gain_db });
            if mic_ok {
                if let Some(p90) = quiet.block(raw_db) {
                    diag::report_error(app, "audio", quiet_message(p90, gain_db));
                }
            }
            level_acc.clear();
            raw_acc.clear();
            last_level = Instant::now();
        }
        engine.accept_waveform(&frame);
        for ev in engine.poll() {
            let tev = match ev {
                SttEvent::Partial(text) => {
                    let ms = captured.elapsed().as_secs_f64() * 1000.0;
                    tracing::debug!(target: "zt::stt", t_ms = t_start.elapsed().as_millis() as u64,
                        latency_ms = ms, %text, "partial");
                    if !demo {
                        let mut d = diag::state();
                        d.lat.audio_to_partial.push(ms);
                        if let Some(on) = onset.claim() {
                            let ms = on.elapsed().as_secs_f64() * 1000.0;
                            d.lat.onset_to_partial.push(ms);
                            tracing::debug!(target: "zt::stt", onset_ms = ms,
                                noise_floor_db = ?onset.noise_floor(), "first partial of utterance");
                        }
                        d.last_partial = Some(text.clone());
                    }
                    let _ =
                        app.emit(EV_TRANSCRIPT, Transcript { text: text.clone(), is_final: false });
                    TrackerEvent::Hypothesis { text, is_final: false }
                }
                SttEvent::Final(text) => {
                    tracing::info!(target: "zt::stt", t_ms = t_start.elapsed().as_millis() as u64,
                        %text, "final");
                    let _ =
                        app.emit(EV_TRANSCRIPT, Transcript { text: text.clone(), is_final: true });
                    TrackerEvent::Hypothesis { text, is_final: true }
                }
                SttEvent::Vad(true) => {
                    tracing::debug!(target: "zt::vad", "speech start");
                    TrackerEvent::VadSpeech
                }
                SttEvent::Vad(false) => {
                    tracing::debug!(target: "zt::vad", "speech end");
                    TrackerEvent::VadSilence
                }
                SttEvent::Resync { text, age_ms } => {
                    if ctx.tracking {
                        resync_event(app, &ctx.shared, text, age_ms);
                    }
                    continue;
                }
            };
            if ctx.tracking {
                let t_hyp = Instant::now();
                let (before, out) = {
                    let mut g = ctx.shared.lock();
                    let before = g.tracker.output().cursor_token;
                    let t = g.now_ms();
                    (before, g.tracker.handle(tev, t))
                };
                apply_tracker_output(app, &ctx.shared, out);
                let mut d = diag::state();
                if out.cursor_token != before && !demo {
                    let align_ms = t_hyp.elapsed().as_secs_f64() * 1000.0;
                    let e2e_ms = captured.elapsed().as_secs_f64() * 1000.0;
                    d.lat.hyp_to_cursor.push(align_ms);
                    d.lat.end_to_end.push(e2e_ms);
                    tracing::debug!(target: "zt::tracker", cursor = ?out.cursor_token,
                        confidence = out.confidence, align_ms, e2e_ms, "cursor moved");
                }
                if out.state != last_state {
                    tracing::info!(target: "zt::tracker", from = state_name(last_state),
                        to = state_name(out.state), confidence = out.confidence,
                        cursor = ?out.cursor_token, "tracker state");
                    last_state = out.state;
                }
                d.voice_state = state_name(out.state).to_string();
                d.confidence = out.confidence;
            }
        }
    }
    if let Some(src) = source.as_mut() {
        src.stop();
    }
    drop(tx);
    tracing::info!(target: "zt::voice", secs = t_start.elapsed().as_secs_f64(), "voice stop");
    let _ = app.emit(EV_LEVEL, Level { db: -100.0, raw_db: -100.0, gain_db: 0.0 });
}

/// Watches the microphone for "opened but delivers nothing": Windows privacy
/// settings ("Mikrofonzugriff für Desktop-Apps") or a muted device give no
/// callbacks or digital silence instead of an error.
#[derive(Debug)]
struct MicWatch {
    t_start: Instant,
    got_frame: bool,
    got_signal: bool,
    reported: bool,
}

impl MicWatch {
    fn new() -> Self {
        Self { t_start: Instant::now(), got_frame: false, got_signal: false, reported: false }
    }

    fn frame(&mut self, samples: &[f32]) {
        self.got_frame = true;
        self.got_signal |= samples.iter().any(|s| *s != 0.0);
    }

    /// Error text once the watch period passed without signal (only once).
    fn check(&mut self) -> Option<&'static str> {
        if self.reported || self.got_signal || self.t_start.elapsed() < MIC_SILENCE_REPORT {
            return None;
        }
        self.reported = true;
        Some(mic_silence_message(self.got_frame))
    }
}

fn mic_silence_message(got_frame: bool) -> &'static str {
    if got_frame {
        "Mikrofon liefert nur digitale Stille. Ist es stummgeschaltet oder ist der \
         Mikrofonzugriff für Desktop-Apps in den Datenschutzeinstellungen gesperrt?"
    } else {
        "Mikrofon liefert keine Daten. Mikrofonzugriff in den System-Datenschutzeinstellungen \
         prüfen oder ein anderes Eingabegerät wählen."
    }
}

/// Detects a microphone that delivers signal, but far too little: the 90th
/// percentile of the raw level blocks (~66 ms) over the first QUIET_WINDOW
/// stays below QUIET_DBFS although the user is expected to speak
/// (Surface/Snapdragon arrays with low Windows input volume).
#[derive(Debug)]
struct QuietWatch {
    t_start: Instant,
    blocks: Vec<f32>,
    done: bool,
}

impl QuietWatch {
    fn new() -> Self {
        Self { t_start: Instant::now(), blocks: Vec::new(), done: false }
    }

    /// Adds one raw level block; returns the p90 level once, if too quiet.
    fn block(&mut self, raw_db: f32) -> Option<f32> {
        if self.done {
            return None;
        }
        self.blocks.push(raw_db);
        if self.t_start.elapsed() < QUIET_WINDOW {
            return None;
        }
        self.done = true;
        let p90 = percentile(&mut self.blocks, 0.9);
        // Pure digital silence is reported by MicWatch instead.
        (p90 < QUIET_DBFS && p90 > -100.0).then_some(p90)
    }
}

fn percentile(v: &mut [f32], q: f32) -> f32 {
    if v.is_empty() {
        return -100.0;
    }
    v.sort_by(|a, b| a.total_cmp(b));
    let i = ((v.len() - 1) as f32 * q).round() as usize;
    v[i.min(v.len() - 1)]
}

fn quiet_message(p90: f32, gain_db: f32) -> String {
    format!(
        "Mikrofon sehr leise (Sprachpegel ca. {p90:.0} dBFS, < {QUIET_DBFS:.0} dBFS beim Sprechen; \
         digitale Verstärkung aktuell {gain_db:+.0} dB). Windows: Einstellungen > System > Sound > \
         Eingabe > Gerät > Eingangslautstärke erhöhen (z. B. 80-100 %), ggf. anderes Mikrofon \
         wählen; in den Voice-Einstellungen Mikrofon-Gain auf Auto lassen."
    )
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
    tracing::info!(target: "zt::resync", age_ms, outcome, %text, "resync");
    let _ = app.emit(EV_RESYNC, ResyncInfo { text, age_ms, outcome });
    apply_tracker_output(app, shared, out);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn watch_started_ago(secs: u64) -> MicWatch {
        let mut w = MicWatch::new();
        w.t_start = Instant::now().checked_sub(Duration::from_secs(secs)).unwrap_or(w.t_start);
        w
    }

    #[test]
    fn mic_watch_reports_no_data_once() {
        let mut w = watch_started_ago(5);
        assert_eq!(w.check(), Some(mic_silence_message(false)));
        assert_eq!(w.check(), None, "reported only once");
    }

    #[test]
    fn mic_watch_reports_digital_silence() {
        let mut w = watch_started_ago(5);
        w.frame(&[0.0; 160]);
        assert_eq!(w.check(), Some(mic_silence_message(true)));
    }

    #[test]
    fn mic_watch_quiet_with_signal_or_early() {
        let mut w = watch_started_ago(5);
        w.frame(&[0.0, 0.001, 0.0]);
        assert_eq!(w.check(), None);
        let mut early = MicWatch::new();
        assert_eq!(early.check(), None);
    }

    fn quiet_started_ago(secs: u64) -> QuietWatch {
        let mut w = QuietWatch::new();
        w.t_start = Instant::now().checked_sub(Duration::from_secs(secs)).unwrap_or(w.t_start);
        w
    }

    #[test]
    fn quiet_watch_reports_low_speech_level_once() {
        let mut w = quiet_started_ago(11);
        for _ in 0..140 {
            w.blocks.push(-90.0);
        }
        assert_eq!(w.block(-43.0), Some(-90.0), "one loud block does not hide a quiet mic");
        assert_eq!(w.block(-90.0), None, "reported only once");
    }

    #[test]
    fn quiet_watch_silent_for_normal_level_digital_silence_or_early() {
        let mut w = quiet_started_ago(11);
        w.blocks.extend([-80.0; 50]);
        w.blocks.extend([-30.0; 50]);
        assert_eq!(w.block(-30.0), None);
        let mut w = quiet_started_ago(11);
        assert_eq!(w.block(-100.0), None, "digital silence is MicWatch's job");
        let mut early = QuietWatch::new();
        assert_eq!(early.block(-90.0), None);
        assert!(!early.done);
        assert!(quiet_message(-80.0, 40.0).contains("Eingangslautstärke"));
    }

    #[test]
    fn mic_config_from_settings() {
        let s = zt_core::Settings::default();
        assert_eq!(mic_config(&s), (GainMode::Auto, ChannelMode::Auto));
        let s = zt_core::Settings {
            mic_gain_auto: false,
            mic_gain_db: 12.0,
            mic_channel: "2".into(),
            ..Default::default()
        };
        assert_eq!(mic_config(&s), (GainMode::Manual(12.0), ChannelMode::Channel(1)));
    }

    #[test]
    fn null_engine_never_produces_events() {
        let mut e = NullEngine;
        e.accept_waveform(&[0.5; 1600]);
        assert!(e.poll().is_empty());
        assert!(e.set_hotwords(&[]).is_ok());
    }
}
