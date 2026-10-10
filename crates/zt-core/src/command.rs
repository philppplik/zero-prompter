//! The single entry point for state changes (CONCEPT.md §6.1).
//! Every input source (UI, hotkey, remote, MIDI, voice adapter) produces `Command`s.

use crate::script::TokenId;
use crate::settings::Settings;
use crate::state::VoicePhase;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum ScrollMode {
    Manual,
    #[default]
    Constant,
    VoiceActivated,
    VoiceTracking,
}

/// Output of the voice pipeline, translated by the app layer from `zt-align`.
/// Keeps `zt-core` independent of `zt-align`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceSignal {
    pub phase: VoicePhase,
    /// Token the speaker is currently at (None = unchanged).
    pub cursor: Option<TokenId>,
    pub confidence: f32,
    /// Smoothed speaking rate in words per second.
    pub rate_wps: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "camelCase")]
pub enum Command {
    LoadScript(String),
    EditScript(String),
    EnterLive,
    EnterEdit,
    TogglePlay,
    Play,
    Pause,
    Stop,
    SetWpm(u16),
    NudgeWpm(i16),
    JumpParagraph(i32),
    SeekToken(TokenId),
    /// Operator "resync here" / click on word: hard cursor set + voice override.
    ResyncHere(TokenId),
    SetMirror {
        h: bool,
        v: bool,
    },
    ToggleMirrorH,
    SetFullscreen(bool),
    SetScrollMode(ScrollMode),
    ToggleVoice,
    SetTheme(String),
    SetFontSize(u16),
    ReplaceSettings(Box<Settings>),
    /// Voice pipeline update.
    Voice(VoiceSignal),
    /// Advance time (frame/timer tick), milliseconds since last tick.
    Tick(u32),
}
