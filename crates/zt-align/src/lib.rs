//! zt-align: pure, deterministic alignment of live speech to a script.
//!
//! Pipeline: STT hypothesis -> [`normalize`] -> last k words -> windowed
//! Smith-Waterman ([`sw`]) against the [`ScriptIndex`] -> hysteresis +
//! voice state machine ([`VoiceTracker`]) -> cursor token for `zt-core`.
//!
//! No I/O, no clocks, no threads, no dependencies: every behaviour is
//! reproducible in unit tests by feeding events with explicit timestamps.

pub mod index;
pub mod normalize;
pub mod similarity;
pub mod sw;
pub mod tracker;

pub use index::ScriptIndex;
pub use normalize::{normalize_hypothesis, normalize_word, Lang};
pub use sw::{align, QueryWord, SwHit, SwParams};
pub use tracker::{
    ResyncConfig, ResyncOutcome, TrackerConfig, TrackerEvent, TrackerOutput, VoiceState,
    VoiceTracker,
};
