//! zt-stt: the speech-recognition ports (hybrid architecture, ADR-006).
//!
//! - [`SpeechEngine`] (alias [`StreamingEngine`]): low-latency streaming
//!   recognizer that drives the cursor. Adapters: [`MockEngine`],
//!   `SherpaEngine` (feature `sherpa`, Zipformer transducer, `OnlineRecognizer`).
//! - [`OfflineEngine`]: transcribes one finished speech segment. Adapters:
//!   [`MockOfflineEngine`], `SherpaOfflineEngine` (feature `sherpa`,
//!   Parakeet TDT via `OfflineRecognizer` + `model_type = "nemo_transducer"`).
//! - [`Segmenter`]: cuts the live stream into speech segments for the offline
//!   engine: [`RmsSegmenter`] (energy based, always available) or
//!   `SileroSegmenter` (feature `sherpa`, sherpa-onnx Silero VAD).
//! - [`HybridEngine`]: orchestrator. Itself a [`SpeechEngine`]; streams as
//!   usual and, if resync is enabled, decodes segments on a background thread
//!   and emits [`SttEvent::Resync`] without ever blocking the streaming path.
//! - [`build_with_fallback`]: tries a chain of engine constructors.
//!
//! Engines are driven from a dedicated STT thread: feed 16 kHz mono samples
//! with [`SpeechEngine::accept_waveform`], then drain [`SpeechEngine::poll`].

mod fallback;
mod hotwords;
mod hybrid;
mod mock;
mod offline;
mod segment;
pub use fallback::{build_with_fallback, Fallback};
pub use hotwords::{build_hotwords, Hotword};
pub use hybrid::{HybridEngine, ResyncOptions, ResyncStats};
pub use mock::{MockEngine, ScriptedEvent};
pub use offline::{MockOfflineEngine, OfflineEngine};
pub use segment::{RmsSegmenter, RmsSegmenterConfig, Segment, Segmenter};

#[cfg(feature = "sherpa")]
mod sherpa;
#[cfg(feature = "sherpa")]
mod sherpa_offline;
#[cfg(feature = "sherpa")]
pub use sherpa::{SherpaConfig, SherpaEngine};
#[cfg(feature = "sherpa")]
pub use sherpa_offline::{SherpaOfflineConfig, SherpaOfflineEngine, SileroSegmenter, SileroVadConfig};

pub const SAMPLE_RATE: u32 = 16_000;

#[derive(Debug, Clone, PartialEq)]
pub enum SttEvent {
    /// Current, still changing hypothesis of the running utterance (full text).
    Partial(String),
    /// Endpoint detected; the utterance text is final.
    Final(String),
    /// Voice activity changed (true = speech started).
    Vad(bool),
    /// Offline (resync) transcript of a finished speech segment.
    /// `age_ms` = audio fed since the segment ended (decode latency).
    Resync { text: String, age_ms: u64 },
}

#[derive(Debug, thiserror::Error)]
pub enum SttError {
    #[error("model files missing or invalid: {0}")]
    Model(String),
    #[error("engine error: {0}")]
    Engine(String),
    #[error("feature not supported by this engine: {0}")]
    Unsupported(&'static str),
}

/// Port implemented by every streaming STT backend.
pub trait SpeechEngine: Send {
    /// Human-readable engine / model name for the operator UI.
    fn name(&self) -> &str;
    /// Feeds 16 kHz mono f32 samples.
    fn accept_waveform(&mut self, samples: &[f32]);
    /// Returns events produced since the last call (non-blocking).
    fn poll(&mut self) -> Vec<SttEvent>;
    /// Biases decoding toward script words (contextual biasing / hotwords).
    fn set_hotwords(&mut self, hotwords: &[Hotword]) -> Result<(), SttError>;
    /// Drops the current utterance (e.g. after a manual seek).
    fn reset(&mut self);
}

/// Name used in ADR-006 for the streaming port (same trait).
pub use self::SpeechEngine as StreamingEngine;

impl<E: SpeechEngine + ?Sized> SpeechEngine for Box<E> {
    fn name(&self) -> &str {
        (**self).name()
    }
    fn accept_waveform(&mut self, samples: &[f32]) {
        (**self).accept_waveform(samples)
    }
    fn poll(&mut self) -> Vec<SttEvent> {
        (**self).poll()
    }
    fn set_hotwords(&mut self, hotwords: &[Hotword]) -> Result<(), SttError> {
        (**self).set_hotwords(hotwords)
    }
    fn reset(&mut self) {
        (**self).reset()
    }
}
