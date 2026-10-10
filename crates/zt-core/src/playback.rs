//! Playback x Mode state machine (CONCEPT.md §6.2).
//!
//! ```text
//! Edit --(Live)--> Live/Stopped --Space--> Countdown --> Playing <-> Paused
//!                       ^                                  |
//!                       +------------ Stop/Home <-- Ended <+
//! ```
//! Edit is only reachable from Stopped/Paused; while Playing the text is locked.
//! Time is passed in explicitly (`tick(dt_ms)`), never read from a clock.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum UiMode {
    #[default]
    Edit,
    Live,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum PlaybackState {
    #[default]
    Stopped,
    /// Seconds remaining (shown) and ms left in the current second.
    Countdown {
        remaining: u8,
        ms_left: u32,
    },
    Playing,
    Paused,
    Ended,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackEvent {
    EnterLive,
    EnterEdit,
    /// Space: play/pause toggle.
    Toggle,
    Play,
    Pause,
    Stop,
    /// Reached the end of the script.
    ReachedEnd,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum TransitionError {
    #[error("event {event:?} not allowed in mode {mode:?} / state {state:?}")]
    NotAllowed { event: PlaybackEvent, mode: UiMode, state: PlaybackState },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Playback {
    pub mode: UiMode,
    pub state: PlaybackState,
    /// Countdown length in seconds; 0 = start immediately.
    pub countdown_secs: u8,
}

impl Default for Playback {
    fn default() -> Self {
        Self { mode: UiMode::Edit, state: PlaybackState::Stopped, countdown_secs: 3 }
    }
}

impl Playback {
    pub fn text_locked(&self) -> bool {
        matches!(self.state, PlaybackState::Playing | PlaybackState::Countdown { .. })
    }

    pub fn is_running(&self) -> bool {
        self.state == PlaybackState::Playing
    }

    fn start(&self) -> PlaybackState {
        if self.countdown_secs == 0 {
            PlaybackState::Playing
        } else {
            PlaybackState::Countdown { remaining: self.countdown_secs, ms_left: 1000 }
        }
    }

    pub fn handle(&mut self, ev: PlaybackEvent) -> Result<(), TransitionError> {
        use PlaybackEvent as E;
        use PlaybackState as S;
        let err = TransitionError::NotAllowed { event: ev, mode: self.mode, state: self.state };
        match (self.mode, self.state, ev) {
            (UiMode::Edit, _, E::EnterLive) => {
                self.mode = UiMode::Live;
                self.state = S::Stopped;
            }
            (UiMode::Live, S::Stopped | S::Paused | S::Ended, E::EnterEdit) => {
                self.mode = UiMode::Edit;
                self.state = S::Stopped;
            }
            (UiMode::Edit, _, E::EnterEdit) => {}
            // Space / Play from Edit implicitly enters Live (zero friction).
            (UiMode::Edit, _, E::Toggle | E::Play) => {
                self.mode = UiMode::Live;
                self.state = self.start();
            }
            (UiMode::Live, S::Stopped | S::Ended, E::Toggle | E::Play) => self.state = self.start(),
            (UiMode::Live, S::Paused, E::Toggle | E::Play) => self.state = S::Playing,
            (UiMode::Live, S::Playing, E::Toggle | E::Pause) => self.state = S::Paused,
            (UiMode::Live, S::Countdown { .. }, E::Toggle | E::Pause | E::Stop) => {
                self.state = S::Stopped;
            }
            // Play while already playing/counting down (or Pause while paused) is a no-op.
            (UiMode::Live, S::Playing | S::Countdown { .. }, E::Play)
            | (UiMode::Live, S::Paused, E::Pause) => {}
            (UiMode::Live, _, E::Stop) => self.state = S::Stopped,
            (UiMode::Live, S::Playing, E::ReachedEnd) => self.state = S::Ended,
            (UiMode::Live, _, E::EnterLive) => {}
            _ => return Err(err),
        }
        Ok(())
    }

    /// Advances the countdown. Returns `true` if playback just started.
    pub fn tick(&mut self, dt_ms: u32) -> bool {
        self.tick_overflow(dt_ms).is_some()
    }

    /// Advances the countdown. Returns `Some(overflow_ms)` if playback just
    /// started, where `overflow_ms` is the part of `dt_ms` left *after* the
    /// countdown ended (the only time that may be spent scrolling).
    pub fn tick_overflow(&mut self, dt_ms: u32) -> Option<u32> {
        let PlaybackState::Countdown { remaining, ms_left } = self.state else {
            return None;
        };
        // Total time still on the clock: current second + full remaining ones.
        let total = ms_left as u64 + remaining.saturating_sub(1) as u64 * 1000;
        let dt = dt_ms as u64;
        if dt >= total {
            self.state = PlaybackState::Playing;
            return Some((dt - total) as u32);
        }
        let left = total - dt; // > 0
        let remaining = left.div_ceil(1000) as u8;
        let ms_left = (left - (remaining as u64 - 1) * 1000) as u32;
        self.state = PlaybackState::Countdown { remaining, ms_left };
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use PlaybackEvent as E;

    #[test]
    fn full_cycle() {
        let mut p = Playback::default();
        p.handle(E::EnterLive).unwrap();
        p.handle(E::Toggle).unwrap();
        assert!(matches!(p.state, PlaybackState::Countdown { remaining: 3, .. }));
        assert!(!p.tick(1500));
        assert!(matches!(p.state, PlaybackState::Countdown { remaining: 2, ms_left: 500 }));
        assert!(p.tick(1500));
        assert_eq!(p.state, PlaybackState::Playing);
        assert!(p.text_locked());
        p.handle(E::Toggle).unwrap();
        assert_eq!(p.state, PlaybackState::Paused);
        p.handle(E::Toggle).unwrap();
        p.handle(E::ReachedEnd).unwrap();
        assert_eq!(p.state, PlaybackState::Ended);
        p.handle(E::EnterEdit).unwrap();
        assert_eq!(p.mode, UiMode::Edit);
    }

    #[test]
    fn edit_not_allowed_while_playing() {
        let mut p = Playback { countdown_secs: 0, ..Default::default() };
        p.handle(E::Play).unwrap();
        assert_eq!(p.state, PlaybackState::Playing);
        assert!(p.handle(E::EnterEdit).is_err());
    }

    #[test]
    fn stop_during_countdown() {
        let mut p = Playback::default();
        p.handle(E::Play).unwrap();
        p.handle(E::Stop).unwrap();
        assert_eq!(p.state, PlaybackState::Stopped);
    }
}
