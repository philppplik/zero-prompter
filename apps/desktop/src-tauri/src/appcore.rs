//! Shared application core: the single `Controller` (Single Source of Truth),
//! the voice tracker, and the one mutation path `dispatch()` used by Tauri
//! commands, global hotkeys, the tick loop and the voice pipeline.
//!
//! Events (CONCEPT §8.3):
//! - `state://patch`  - changed sub-trees of `AppState` (camelCase JSON)
//! - `scroll://frame` - position/target/velocity in token units, max ~60 Hz
//! - `voice://word`   - cursor token + confidence when the voice cursor moves

use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use serde::Serialize;
use serde_json::{Map, Value};
use tauri::{AppHandle, Emitter};
use zt_align::{
    Lang, ScriptIndex, TrackerConfig, TrackerEvent, TrackerOutput, VoiceState, VoiceTracker,
};
use zt_core::{AppState, Command, Controller, PlaybackState, StatePatch, VoicePhase, VoiceSignal};

use crate::persist;
use crate::voice::VoiceRuntime;

pub const EV_PATCH: &str = "state://patch";
pub const EV_SCROLL: &str = "scroll://frame";
pub const EV_WORD: &str = "voice://word";

pub struct AppCore {
    pub ctl: Controller,
    pub tracker: VoiceTracker,
    last_voice: Option<TrackerOutput>,
    t0: Instant,
    pub voice_rt: Option<VoiceRuntime>,
    pub audio_device: Option<String>,
    /// Per-language model choice + resync switch (ADR-006), read at voice start.
    pub voice_models: crate::models::VoiceModelPrefs,
    /// Latest layout report from the talent renderer (script_version, line count).
    /// Written by `report_layout`, echoed in every `scroll://frame` as
    /// `layoutLines` (only while it matches the current script version).
    pub layout: Option<(u64, u32)>,
    last_scroll_pos: f64,
}

impl std::fmt::Debug for AppCore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppCore").finish_non_exhaustive()
    }
}

pub fn lang_of(code: &str) -> Lang {
    if code.to_ascii_lowercase().starts_with("en") {
        Lang::En
    } else {
        Lang::De
    }
}

fn tracker_config(s: &zt_core::Settings) -> TrackerConfig {
    TrackerConfig {
        lost_timeout_ms: s.lost_timeout_ms as u64,
        manual_override_ms: s.manual_override_ms as u64,
        ..TrackerConfig::default()
    }
}

fn phase_of(v: VoiceState) -> VoicePhase {
    match v {
        VoiceState::Idle => VoicePhase::Idle,
        VoiceState::Listening => VoicePhase::Listening,
        VoiceState::Tracking => VoicePhase::Tracking,
        VoiceState::Hold => VoicePhase::Hold,
        VoiceState::Lost => VoicePhase::Lost,
        VoiceState::Resync => VoicePhase::Resync,
    }
}

impl AppCore {
    pub fn new(ctl: Controller) -> Self {
        let s = &ctl.state().settings;
        let idx = ScriptIndex::build(ctl.state().script.spoken_tokens(), lang_of(&s.language));
        let tracker = VoiceTracker::new(idx, tracker_config(s));
        Self {
            ctl,
            tracker,
            last_voice: None,
            t0: Instant::now(),
            voice_rt: None,
            audio_device: None,
            voice_models: Default::default(),
            layout: None,
            last_scroll_pos: 0.0,
        }
    }

    pub fn now_ms(&self) -> u64 {
        self.t0.elapsed().as_millis() as u64
    }

    pub fn rebuild_index(&mut self) {
        let st = self.ctl.state();
        let idx = ScriptIndex::build(st.script.spoken_tokens(), lang_of(&st.settings.language));
        self.tracker.set_index(idx);
        self.last_voice = None;
    }

    /// Spoken words of the current script (for hotwords and demo mode).
    pub fn spoken_words(&self) -> Vec<String> {
        self.ctl.state().script.spoken_tokens().map(|(_, w)| w.to_string()).collect()
    }
}

/// Cheaply clonable handle, stored in Tauri's managed state.
#[derive(Clone)]
pub struct Shared(pub Arc<Mutex<AppCore>>);

impl Shared {
    pub fn new(core: AppCore) -> Self {
        Self(Arc::new(Mutex::new(core)))
    }
    /// Never panics on poisoning: the core is plain data and stays consistent
    /// because `Controller::apply` is all-or-nothing.
    pub fn lock(&self) -> MutexGuard<'_, AppCore> {
        self.0.lock().unwrap_or_else(|e| e.into_inner())
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScrollFrame {
    pub position: f64,
    pub target: f64,
    /// Tokens per second (for frontend extrapolation between frames).
    pub velocity: f64,
    pub playing: bool,
    pub script_version: u64,
    /// Rendered line count reported by the talent renderer for this script
    /// version (`None` until reported or after the script changed).
    pub layout_lines: Option<u32>,
}

/// Line count of the latest layout report, if it belongs to `script_version`.
pub fn current_layout_lines(layout: Option<(u64, u32)>, script_version: u64) -> Option<u32> {
    layout.filter(|(v, _)| *v == script_version).map(|(_, lines)| lines)
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WordEvent {
    pub token: usize,
    pub confidence: f32,
}

/// Serialises only the flagged sub-trees.
pub fn patch_json(s: &AppState, p: StatePatch) -> Value {
    let mut m = Map::new();
    let mut put = |k: &str, v: Result<Value, serde_json::Error>| {
        m.insert(k.to_string(), v.unwrap_or_default());
    };
    if p.script {
        put("script", serde_json::to_value(&s.script));
    }
    if p.playback {
        put("playback", serde_json::to_value(s.playback));
    }
    if p.scroll {
        put("scroll", serde_json::to_value(s.scroll));
    }
    if p.voice {
        put("voice", serde_json::to_value(s.voice));
    }
    if p.output {
        put("output", serde_json::to_value(s.output));
    }
    if p.settings {
        put("settings", serde_json::to_value(&s.settings));
    }
    Value::Object(m)
}

fn emit<P: Serialize + Clone>(app: &AppHandle, ev: &str, payload: P) {
    // Emitting fails only when the runtime is shutting down; nothing to do then.
    let _ = app.emit(ev, payload);
}

fn scroll_frame(core: &mut AppCore, dt_ms: u32) -> ScrollFrame {
    let st = core.ctl.state();
    let pos = st.scroll.position;
    let velocity =
        if dt_ms > 0 { (pos - core.last_scroll_pos) * 1000.0 / dt_ms as f64 } else { 0.0 };
    core.last_scroll_pos = pos;
    ScrollFrame {
        position: pos,
        target: st.scroll.target,
        velocity: if st.playback.is_running() { velocity } else { 0.0 },
        playing: st.playback.is_running(),
        script_version: st.script.version,
        layout_lines: current_layout_lines(core.layout, st.script.version),
    }
}

/// The one mutation path. Returns the error text for the UI toast.
pub fn dispatch(app: &AppHandle, shared: &Shared, cmd: Command) -> Result<(), String> {
    let is_seek = matches!(
        cmd,
        Command::JumpParagraph(_) | Command::SeekToken(_) | Command::ResyncHere(_) | Command::Stop
    );
    let (payload, frame, voice_toggle, settings_changed, settings_snapshot) = {
        let mut g = shared.lock();
        let was_voice = g.ctl.state().voice.enabled;
        let old_lang = g.ctl.state().settings.language.clone();
        let patch = g.ctl.apply(cmd).map_err(|e| e.to_string())?;
        let st = g.ctl.state();
        let now_voice = st.voice.enabled;
        let lang_changed = st.settings.language != old_lang;
        if patch.script || lang_changed {
            g.rebuild_index();
        }
        // Only tracker-relevant settings touch the tracker. Recreating it on every
        // settings patch (theme, font size, ...) would reset cursor + state mid-take.
        if patch.settings {
            let cfg = tracker_config(&g.ctl.state().settings);
            if cfg != *g.tracker.config() {
                g.tracker.set_config(cfg);
            }
        }
        if is_seek && now_voice {
            let token = g.ctl.state().current_token();
            let t = g.now_ms();
            g.tracker.handle(TrackerEvent::ManualSeek { token }, t);
        }
        let frame = if patch.scroll { Some(scroll_frame(&mut g, 0)) } else { None };
        let st = g.ctl.state();
        let toggle = if was_voice != now_voice { Some(now_voice) } else { None };
        let snap = if patch.settings { Some(st.settings.clone()) } else { None };
        (patch_json(st, patch), frame, toggle, patch.settings, snap)
    };
    emit(app, EV_PATCH, payload);
    if let Some(f) = frame {
        emit(app, EV_SCROLL, f);
    }
    if let Some(on) = voice_toggle {
        crate::voice::set_tracking(app, shared, on);
    }
    if settings_changed {
        if let Some(s) = settings_snapshot {
            persist::save_settings(app, &s);
        }
    }
    Ok(())
}

/// Feeds a tracker output into the controller if it changed.
pub fn apply_tracker_output(app: &AppHandle, shared: &Shared, out: TrackerOutput) {
    let word = {
        let mut g = shared.lock();
        if g.last_voice == Some(out) {
            return;
        }
        let cursor_changed = g.last_voice.map(|o| o.cursor_token) != Some(out.cursor_token);
        g.last_voice = Some(out);
        let sig = VoiceSignal {
            phase: phase_of(out.state),
            cursor: out.cursor_token,
            confidence: out.confidence,
            rate_wps: out.rate_wps,
        };
        if let Ok(p) = g.ctl.apply(Command::Voice(sig)) {
            let payload = patch_json(g.ctl.state(), StatePatch { scroll: false, ..p });
            emit(app, EV_PATCH, payload);
        }
        if cursor_changed {
            out.cursor_token.map(|token| WordEvent { token, confidence: out.confidence })
        } else {
            None
        }
    };
    if let Some(w) = word {
        emit(app, EV_WORD, w);
    }
}

/// ~60 Hz clock: the only place where wall time enters the core (`Command::Tick`).
pub fn spawn_tick_loop(app: AppHandle, shared: Shared) {
    std::thread::Builder::new()
        .name("zt-tick".into())
        .spawn(move || {
            let start = Instant::now();
            let mut last = start;
            let mut rate = zt_core::metrics::RateCounter::default();
            loop {
                std::thread::sleep(Duration::from_millis(16));
                let now = Instant::now();
                let dt = now.duration_since(last).as_millis().clamp(1, 250) as u32;
                last = now;
                tick(&app, &shared, dt);
                // Measured tick rate (target ~60 Hz) for the diagnostics panel.
                if let Some(hz) = rate.hit(now.duration_since(start).as_millis() as u64) {
                    crate::diag::state().tick_hz = hz;
                }
            }
        })
        .expect("spawn tick thread");
}

fn tick(app: &AppHandle, shared: &Shared, dt: u32) {
    let (payload, frame, tracker_out) = {
        let mut g = shared.lock();
        let was_countdown = matches!(g.ctl.state().playback.state, PlaybackState::Countdown { .. });
        let patch = match g.ctl.apply(Command::Tick(dt)) {
            Ok(p) => p,
            Err(_) => return,
        };
        let tracker_out = if g.ctl.state().voice.enabled {
            let t = g.now_ms();
            Some(g.tracker.handle(TrackerEvent::Tick, t))
        } else {
            None
        };
        let frame = if patch.scroll { Some(scroll_frame(&mut g, dt)) } else { None };
        // Countdown sends a playback patch every tick; only forward when the
        // visible second changes (or countdown ends) to keep IPC light.
        let mut p = StatePatch { scroll: false, ..patch };
        if p.playback && was_countdown {
            if let PlaybackState::Countdown { ms_left, .. } = g.ctl.state().playback.state {
                if ms_left < 1000 - dt.min(1000) {
                    p.playback = false;
                }
            }
        }
        // Override countdown is internal; only report when it reaches zero.
        if p.voice && g.ctl.state().voice.override_ms > 0 {
            p.voice = false;
        }
        let payload = if p.is_empty() { None } else { Some(patch_json(g.ctl.state(), p)) };
        (payload, frame, tracker_out)
    };
    if let Some(pl) = payload {
        emit(app, EV_PATCH, pl);
    }
    if let Some(f) = frame {
        emit(app, EV_SCROLL, f);
    }
    if let Some(out) = tracker_out {
        apply_tracker_output(app, shared, out);
    }
}

#[cfg(test)]
mod tests {
    use super::current_layout_lines;

    #[test]
    fn layout_lines_only_for_current_version() {
        assert_eq!(current_layout_lines(None, 1), None);
        assert_eq!(current_layout_lines(Some((1, 42)), 1), Some(42));
        assert_eq!(current_layout_lines(Some((1, 42)), 2), None);
    }
}
