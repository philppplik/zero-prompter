//! Application state (CONCEPT.md §6.1). Plain data, serialisable, no behaviour
//! beyond small helpers. Behaviour lives in [`crate::controller`].

use crate::command::ScrollMode;
use crate::playback::Playback;
use crate::script::{Script, TokenId};
use crate::settings::Settings;
use serde::{Deserialize, Serialize};

/// Mirror of the `zt-align` voice states, as seen by the UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum VoicePhase {
    #[default]
    Idle,
    Listening,
    Tracking,
    Hold,
    Lost,
    Resync,
}

impl VoicePhase {
    /// Key into `tokens.json` -> `state`.
    pub fn token_key(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Listening => "listening",
            Self::Tracking => "tracking",
            Self::Hold => "hold",
            Self::Lost => "lost",
            Self::Resync => "resync",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct VoiceView {
    pub enabled: bool,
    pub phase: VoicePhase,
    pub confidence: f32,
    pub cursor_token: TokenId,
    pub rate_wps: f32,
    /// Remaining manual-override time in ms (voice ignored while > 0).
    pub override_ms: u32,
}

/// Scroll position is expressed in (fractional) token units; the frontend
/// maps tokens to pixels via its layout map. This keeps the core layout-free.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ScrollState {
    pub mode: ScrollMode,
    pub position: f64,
    pub target: f64,
    pub wpm: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct OutputState {
    pub monitor_id: Option<u32>,
    pub fullscreen: bool,
    pub mirror_h: bool,
    pub mirror_v: bool,
    pub overlay: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AppState {
    pub script: Script,
    pub playback: Playback,
    pub scroll: ScrollState,
    pub voice: VoiceView,
    pub output: OutputState,
    pub settings: Settings,
}

impl AppState {
    pub fn new(settings: Settings) -> Self {
        let playback = Playback { countdown_secs: settings.countdown_secs, ..Default::default() };
        let scroll = ScrollState { wpm: settings.wpm, ..Default::default() };
        Self { playback, scroll, settings, ..Default::default() }
    }

    pub fn current_token(&self) -> TokenId {
        let max = self.script.len().saturating_sub(1);
        (self.scroll.position.max(0.0).floor() as usize).min(max)
    }
}
