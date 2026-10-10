//! zt-testkit: shared test helpers for zero-teleprompter.
//!
//! - [`fixtures`]: reference scripts and table-driven alignment scenarios
//!   (Versprecher, Zurückspringen, Überspringen, Stille, Ad-lib);
//! - [`speaker`]: drives a [`zt_align::VoiceTracker`] with simulated speech
//!   and an explicit clock;
//! - [`timeline`]: builds deterministic [`zt_stt::MockEngine`]s from a
//!   readable speech timeline (words, pauses, ad-libs);
//! - [`pipeline`]: MockEngine -> VoiceTracker -> Controller harness that
//!   mirrors the app's voice thread without threads, audio or Tauri;
//! - [`builders`]: `Controller` / `Settings` builders.
//!
//! Scenario categories (ad-lib, backtrack, forward jump) follow the test
//! categories of EdNutting/autocue (MIT, https://github.com/EdNutting/autocue);
//! texts and code are our own. See THIRD_PARTY_NOTICES.md.

pub mod builders;
pub mod fixtures;
pub mod pipeline;
pub mod speaker;
pub mod timeline;

pub use builders::ControllerBuilder;
pub use fixtures::{Scenario, Step, SCRIPT_DE};
pub use pipeline::Pipeline;
pub use speaker::Speaker;
pub use timeline::SpeechTimeline;
