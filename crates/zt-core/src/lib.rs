//! zt-core: the pure, I/O-free domain core of zero-teleprompter.
//!
//! Hexagonal architecture (see `docs/adr/ADR-005-code-architecture.md`):
//! this crate knows nothing about Tauri, audio devices, files or threads.
//! All mutations go through [`Command`] -> [`Controller::apply`] -> [`StatePatch`].

pub mod command;
pub mod controller;
pub mod metrics;
pub mod playback;
pub mod script;
pub mod settings;
pub mod state;

pub use command::{Command, ScrollMode, VoiceSignal};
pub use controller::{Controller, CoreError, StatePatch};
pub use playback::{Playback, PlaybackEvent, PlaybackState, UiMode};
pub use script::{Paragraph, Script, Token, TokenId};
pub use settings::{
    LostPolicy, PrompterTheme, Settings, StateStyle, TextAlign, TextTransform, ThemeSet, Tokens,
};
pub use state::{AppState, OutputState, ScrollState, VoicePhase, VoiceView};
