//! `Controller`: applies [`Command`]s to [`AppState`] deterministically.
//! No clocks, no I/O: time arrives via `Command::Tick`.

use crate::command::{Command, ScrollMode};
use crate::playback::{PlaybackEvent, PlaybackState, TransitionError};
use crate::script::Script;
use crate::settings::{LostPolicy, Settings, Tokens};
use crate::state::{AppState, VoicePhase};

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum CoreError {
    #[error(transparent)]
    Transition(#[from] TransitionError),
    #[error("script is locked while playing")]
    ScriptLocked,
    #[error("unknown theme '{0}'")]
    UnknownTheme(String),
}

/// Which parts of the state changed; the adapter turns this into a
/// `state://patch` event (only the flagged sub-trees are sent).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct StatePatch {
    pub script: bool,
    pub playback: bool,
    pub scroll: bool,
    pub voice: bool,
    pub output: bool,
    pub settings: bool,
}

impl StatePatch {
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }
}

#[derive(Debug, Clone)]
pub struct Controller {
    state: AppState,
    tokens: Tokens,
}

/// Time constant for easing scroll position toward the voice target.
/// 150 ms (was 300): 63 % of a cursor step is on screen after 150 ms, which
/// keeps "word recognised -> scroll" inside the 300 ms p50 budget (ADR-007)
/// while still hiding the word-by-word steps.
pub const VOICE_EASE_MS: f64 = 150.0;

impl Controller {
    pub fn new(tokens: Tokens, settings: Settings) -> Self {
        let settings = settings.sanitized(&tokens);
        Self { state: AppState::new(settings), tokens }
    }

    pub fn with_embedded_tokens() -> Self {
        let t = Tokens::embedded();
        let s = Settings::from_tokens(&t);
        Self::new(t, s)
    }

    pub fn state(&self) -> &AppState {
        &self.state
    }

    pub fn tokens(&self) -> &Tokens {
        &self.tokens
    }

    fn playback(&mut self, ev: PlaybackEvent, p: &mut StatePatch) -> Result<(), CoreError> {
        self.state.playback.handle(ev)?;
        p.playback = true;
        Ok(())
    }

    fn seek(&mut self, token: usize, p: &mut StatePatch) {
        let max = self.state.script.len().saturating_sub(1) as f64;
        let pos = (token as f64).min(max).max(0.0);
        self.state.scroll.position = pos;
        self.state.scroll.target = pos;
        self.state.voice.cursor_token = pos as usize;
        p.scroll = true;
    }

    fn manual_override(&mut self, p: &mut StatePatch) {
        if self.state.voice.enabled {
            self.state.voice.override_ms = self.state.settings.manual_override_ms;
            self.state.voice.phase = VoicePhase::Hold;
            p.voice = true;
        }
    }

    pub fn apply(&mut self, cmd: Command) -> Result<StatePatch, CoreError> {
        let mut p = StatePatch::default();
        match cmd {
            Command::LoadScript(text) | Command::EditScript(text) => {
                if self.state.playback.text_locked() {
                    return Err(CoreError::ScriptLocked);
                }
                let v = self.state.script.version;
                self.state.script = Script::parse(text);
                self.state.script.version = v + 1;
                self.seek(0, &mut p);
                p.script = true;
            }
            Command::EnterLive => self.playback(PlaybackEvent::EnterLive, &mut p)?,
            Command::EnterEdit => self.playback(PlaybackEvent::EnterEdit, &mut p)?,
            Command::TogglePlay => self.playback(PlaybackEvent::Toggle, &mut p)?,
            Command::Play => self.playback(PlaybackEvent::Play, &mut p)?,
            Command::Pause => self.playback(PlaybackEvent::Pause, &mut p)?,
            Command::Stop => {
                self.playback(PlaybackEvent::Stop, &mut p)?;
                self.seek(0, &mut p);
            }
            Command::SetWpm(w) => {
                self.state.scroll.wpm = w.clamp(Settings::WPM_MIN, Settings::WPM_MAX);
                p.scroll = true;
            }
            Command::NudgeWpm(d) => {
                let w = (self.state.scroll.wpm as i32 + d as i32).clamp(0, u16::MAX as i32) as u16;
                return self.apply(Command::SetWpm(w));
            }
            Command::JumpParagraph(d) => {
                let to = self.state.script.paragraph_jump(self.state.current_token(), d);
                self.seek(to, &mut p);
                self.manual_override(&mut p);
            }
            Command::SeekToken(t) | Command::ResyncHere(t) => {
                self.seek(t, &mut p);
                self.manual_override(&mut p);
            }
            Command::SetMirror { h, v } => {
                self.state.output.mirror_h = h;
                self.state.output.mirror_v = v;
                p.output = true;
            }
            Command::ToggleMirrorH => {
                self.state.output.mirror_h = !self.state.output.mirror_h;
                p.output = true;
            }
            Command::SetFullscreen(f) => {
                self.state.output.fullscreen = f;
                p.output = true;
            }
            Command::SetScrollMode(m) => {
                self.state.scroll.mode = m;
                p.scroll = true;
            }
            Command::ToggleVoice => {
                let v = &mut self.state.voice;
                v.enabled = !v.enabled;
                v.phase = if v.enabled { VoicePhase::Listening } else { VoicePhase::Idle };
                v.override_ms = 0;
                self.state.scroll.mode =
                    if v.enabled { ScrollMode::VoiceTracking } else { ScrollMode::Constant };
                p.voice = true;
                p.scroll = true;
            }
            Command::SetTheme(id) => {
                if !self.tokens.prompter.contains_key(&id) {
                    return Err(CoreError::UnknownTheme(id));
                }
                self.state.settings.theme = id;
                p.settings = true;
            }
            Command::SetFontSize(px) => {
                let t = self.tokens.typography.prompter;
                self.state.settings.font_size_px = px.clamp(t.size_min, t.size_max);
                p.settings = true;
            }
            Command::ReplaceSettings(s) => {
                self.state.settings = s.sanitized(&self.tokens);
                self.state.playback.countdown_secs = self.state.settings.countdown_secs;
                p.settings = true;
            }
            Command::Voice(sig) => self.on_voice(sig, &mut p),
            Command::Tick(dt) => self.on_tick(dt, &mut p)?,
        }
        Ok(p)
    }

    fn on_voice(&mut self, sig: crate::command::VoiceSignal, p: &mut StatePatch) {
        let v = &mut self.state.voice;
        if !v.enabled || v.override_ms > 0 {
            return;
        }
        v.phase = sig.phase;
        v.confidence = sig.confidence;
        v.rate_wps = sig.rate_wps;
        if let Some(c) = sig.cursor {
            // The tracker may lag behind a script edit: never point past the end.
            let c = c.min(self.state.script.len().saturating_sub(1));
            v.cursor_token = c;
            if self.state.scroll.mode == ScrollMode::VoiceTracking {
                self.state.scroll.target = c as f64;
                p.scroll = true;
            }
        }
        p.voice = true;
    }

    fn on_tick(&mut self, dt: u32, p: &mut StatePatch) -> Result<(), CoreError> {
        let was_countdown = matches!(self.state.playback.state, PlaybackState::Countdown { .. });
        let overflow = self.state.playback.tick_overflow(dt);
        if overflow.is_some() {
            p.playback = true;
        }
        if let PlaybackState::Countdown { .. } = self.state.playback.state {
            p.playback = true;
        }
        if self.state.voice.override_ms > 0 {
            self.state.voice.override_ms = self.state.voice.override_ms.saturating_sub(dt);
            p.voice = true;
        }
        if !self.state.playback.is_running() {
            return Ok(());
        }
        // Nothing to scroll: an empty script ends immediately instead of
        // growing `position` forever.
        if self.state.script.is_empty() {
            self.state.scroll.position = 0.0;
            self.state.scroll.target = 0.0;
            p.scroll = true;
            return self.playback(PlaybackEvent::ReachedEnd, p);
        }
        // On the tick that ends the countdown only the time after it scrolls.
        let dt = if was_countdown { overflow.unwrap_or(0) } else { dt };
        let dt_s = dt as f64 / 1000.0;
        let s = &mut self.state.scroll;
        let constant_step = s.wpm as f64 / 60.0 * dt_s;
        match s.mode {
            ScrollMode::Manual => {}
            ScrollMode::Constant => {
                s.position += constant_step;
                s.target = s.position;
                p.scroll = true;
            }
            ScrollMode::VoiceActivated => {
                // Moves at constant speed only while speech is detected.
                if matches!(self.state.voice.phase, VoicePhase::Tracking | VoicePhase::Listening) {
                    s.position += constant_step;
                    s.target = s.position;
                    p.scroll = true;
                }
            }
            ScrollMode::VoiceTracking => {
                let lost_const = self.state.voice.phase == VoicePhase::Lost
                    && self.state.settings.lost_policy == LostPolicy::ConstantSpeed;
                if lost_const {
                    s.position += constant_step;
                    s.target = s.position;
                } else {
                    // Exponential ease toward the target (frame-rate independent).
                    let alpha = 1.0 - (-(dt as f64) / VOICE_EASE_MS).exp();
                    s.position += (s.target - s.position) * alpha;
                }
                p.scroll = true;
            }
        }
        let end = self.state.script.len() as f64;
        if end > 0.0 && self.state.scroll.position >= end - 1.0 + 0.999 {
            self.state.scroll.position = end - 1.0;
            self.playback(PlaybackEvent::ReachedEnd, p)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::VoiceSignal;

    fn ctl() -> Controller {
        let mut c = Controller::with_embedded_tokens();
        let mut s = c.state().settings.clone();
        s.countdown_secs = 0;
        c.apply(Command::ReplaceSettings(Box::new(s))).unwrap();
        c.apply(Command::LoadScript("eins zwei drei vier\n\nfünf sechs".into())).unwrap();
        c
    }

    #[test]
    fn constant_scroll_advances_and_ends() {
        let mut c = ctl();
        c.apply(Command::SetWpm(120)).unwrap(); // 2 words/s
        c.apply(Command::Play).unwrap();
        c.apply(Command::Tick(1000)).unwrap();
        assert!((c.state().scroll.position - 2.0).abs() < 1e-9);
        c.apply(Command::Tick(5000)).unwrap();
        assert_eq!(c.state().playback.state, PlaybackState::Ended);
    }

    #[test]
    fn script_locked_while_playing() {
        let mut c = ctl();
        c.apply(Command::Play).unwrap();
        assert_eq!(c.apply(Command::EditScript("x".into())), Err(CoreError::ScriptLocked));
    }

    #[test]
    fn voice_target_and_override() {
        let mut c = ctl();
        c.apply(Command::ToggleVoice).unwrap();
        c.apply(Command::Play).unwrap();
        let sig = VoiceSignal {
            phase: VoicePhase::Tracking,
            cursor: Some(3),
            confidence: 0.9,
            rate_wps: 2.0,
        };
        c.apply(Command::Voice(sig)).unwrap();
        assert_eq!(c.state().scroll.target, 3.0);
        c.apply(Command::Tick(16)).unwrap();
        assert!(c.state().scroll.position > 0.0 && c.state().scroll.position < 3.0);
        // manual jump -> override, voice ignored for 2 s
        c.apply(Command::JumpParagraph(1)).unwrap();
        assert_eq!(c.state().voice.phase, VoicePhase::Hold);
        c.apply(Command::Voice(VoiceSignal { cursor: Some(0), ..sig })).unwrap();
        assert_eq!(c.state().scroll.target, 4.0);
    }

    #[test]
    fn voice_ease_reaches_most_of_a_step_within_budget() {
        let mut c = ctl();
        c.apply(Command::ToggleVoice).unwrap();
        c.apply(Command::Play).unwrap();
        let sig = VoiceSignal {
            phase: VoicePhase::Tracking,
            cursor: Some(3),
            confidence: 0.9,
            rate_wps: 2.0,
        };
        c.apply(Command::Voice(sig)).unwrap();
        for _ in 0..10 {
            c.apply(Command::Tick(16)).unwrap();
        }
        // 160 ms at tau = 150 ms -> ~66 % of the way (budget: >= 60 %).
        let pos = c.state().scroll.position;
        assert!((0.6 * 3.0..3.0).contains(&pos), "position {pos}");
    }

    #[test]
    fn voice_cursor_clamped_to_script() {
        let mut c = ctl(); // 6 tokens
        c.apply(Command::ToggleVoice).unwrap();
        let sig = VoiceSignal {
            phase: VoicePhase::Tracking,
            cursor: Some(999),
            confidence: 0.9,
            rate_wps: 2.0,
        };
        c.apply(Command::Voice(sig)).unwrap();
        let last = c.state().script.len() - 1;
        assert_eq!(c.state().voice.cursor_token, last);
        assert_eq!(c.state().scroll.target, last as f64);
    }

    #[test]
    fn empty_script_constant_mode_ends() {
        let mut c = ctl();
        c.apply(Command::LoadScript(String::new())).unwrap();
        c.apply(Command::SetScrollMode(ScrollMode::Constant)).unwrap();
        c.apply(Command::Play).unwrap();
        c.apply(Command::Tick(1000)).unwrap();
        assert_eq!(c.state().scroll.position, 0.0);
        assert_eq!(c.state().playback.state, PlaybackState::Ended);
        c.apply(Command::Tick(1000)).unwrap();
        assert_eq!(c.state().scroll.position, 0.0);
    }

    #[test]
    fn play_during_countdown_is_ignored() {
        let mut c = ctl();
        let mut s = c.state().settings.clone();
        s.countdown_secs = 3;
        c.apply(Command::ReplaceSettings(Box::new(s))).unwrap();
        c.apply(Command::Play).unwrap();
        let before = c.state().playback;
        assert!(c.apply(Command::Play).is_ok());
        assert_eq!(c.state().playback, before);
    }

    #[test]
    fn countdown_end_tick_scrolls_only_overflow() {
        let mut c = ctl();
        let mut s = c.state().settings.clone();
        s.countdown_secs = 1;
        c.apply(Command::ReplaceSettings(Box::new(s))).unwrap();
        c.apply(Command::SetScrollMode(ScrollMode::Constant)).unwrap();
        c.apply(Command::SetWpm(120)).unwrap(); // 2 words/s
        c.apply(Command::Play).unwrap();
        c.apply(Command::Tick(1250)).unwrap(); // 1000 ms countdown + 250 ms play
        assert_eq!(c.state().playback.state, PlaybackState::Playing);
        assert!((c.state().scroll.position - 0.5).abs() < 1e-9, "{}", c.state().scroll.position);
    }

    #[test]
    fn unknown_theme_rejected() {
        let mut c = ctl();
        assert!(c.apply(Command::SetTheme("daylight".into())).is_ok());
        assert!(c.apply(Command::SetTheme("nope".into())).is_err());
    }
}
