//! sherpa-onnx offline adapter + Silero VAD segmenter (feature `sherpa`).
//!
//! API checked on https://docs.rs/sherpa-onnx/1.13.8 (not compiled here):
//! - `OfflineRecognizer::create(&OfflineRecognizerConfig) -> Option<Self>`,
//!   `create_stream(&self) -> OfflineStream`, `decode(&self, &OfflineStream)`;
//!   `OfflineStream::accept_waveform(&self, i32, &[f32])`,
//!   `OfflineStream::get_result(&self) -> Option<OfflineRecognizerResult>`
//!   (`text: String`, `tokens`, `timestamps`, `durations`). Both `Send + Sync`.
//! - `OfflineRecognizerConfig::default()` (16 kHz / 80-dim fbank,
//!   `decoding_method: None`); `model_config: OfflineModelConfig`
//!   (`transducer: OfflineTransducerModelConfig { encoder, decoder, joiner:
//!   Option<String> }`, `tokens`, `model_type: Option<String>`,
//!   `num_threads: i32` — derived default 0, so set it).
//!   Parakeet TDT example on docs.rs uses `model_type = "nemo_transducer"`.
//! - `VoiceActivityDetector::create(&VadModelConfig, f32) -> Option<Self>`,
//!   `accept_waveform(&self, &[f32])`, `is_empty`, `front() -> Option<SpeechSegment>`,
//!   `pop`, `flush`, `reset`; `SpeechSegment::{start() -> i32, samples() -> &[f32]}`.
//!   `VadModelConfig { silero_vad: SileroVadModelConfig { model, threshold,
//!   min_silence_duration, min_speech_duration, window_size, max_speech_duration },
//!   sample_rate, num_threads, provider, debug, .. }` — all derived defaults
//!   are zero, so every field is set explicitly.

use crate::{OfflineEngine, Segment, Segmenter, SttError, SAMPLE_RATE};
use std::path::PathBuf;

/// Offline NeMo transducer (Parakeet TDT 0.6B v3 int8) configuration.
#[derive(Debug, Clone)]
pub struct SherpaOfflineConfig {
    pub encoder: PathBuf,
    pub decoder: PathBuf,
    pub joiner: PathBuf,
    pub tokens: PathBuf,
    /// sherpa-onnx `model_type`; "nemo_transducer" for Parakeet.
    pub model_type: String,
    pub num_threads: i32,
    pub decoding_method: String,
    pub model_name: String,
}

impl SherpaOfflineConfig {
    /// Defaults for `sherpa-onnx-nemo-parakeet-tdt-0.6b-v3-int8`; the manifest
    /// roles override the file names.
    pub fn parakeet(dir: impl Into<PathBuf>, model_name: impl Into<String>) -> Self {
        let dir = dir.into();
        Self {
            encoder: dir.join("encoder.int8.onnx"),
            decoder: dir.join("decoder.int8.onnx"),
            joiner: dir.join("joiner.int8.onnx"),
            tokens: dir.join("tokens.txt"),
            model_type: "nemo_transducer".into(),
            // Runs beside the streaming engine; keep it from starving it.
            num_threads: 2,
            decoding_method: "greedy_search".into(),
            model_name: model_name.into(),
        }
    }

    fn validate(&self) -> Result<(), SttError> {
        for p in [&self.encoder, &self.decoder, &self.joiner, &self.tokens] {
            if !p.is_file() {
                return Err(SttError::Model(format!("missing {}", p.display())));
            }
        }
        Ok(())
    }
}

pub struct SherpaOfflineEngine {
    name: String,
    recognizer: sherpa_onnx::OfflineRecognizer,
}

impl std::fmt::Debug for SherpaOfflineEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SherpaOfflineEngine").field("model", &self.name).finish()
    }
}

impl SherpaOfflineEngine {
    pub fn new(cfg: SherpaOfflineConfig) -> Result<Self, SttError> {
        cfg.validate()?;
        let p = |x: &std::path::Path| x.to_string_lossy().into_owned();
        let mut c = sherpa_onnx::OfflineRecognizerConfig::default();
        c.model_config.transducer = sherpa_onnx::OfflineTransducerModelConfig {
            encoder: Some(p(&cfg.encoder)),
            decoder: Some(p(&cfg.decoder)),
            joiner: Some(p(&cfg.joiner)),
        };
        c.model_config.tokens = Some(p(&cfg.tokens));
        c.model_config.model_type = Some(cfg.model_type.clone());
        c.model_config.num_threads = cfg.num_threads;
        c.decoding_method = Some(cfg.decoding_method.clone());
        let recognizer = sherpa_onnx::OfflineRecognizer::create(&c)
            .ok_or_else(|| SttError::Engine("failed to create OfflineRecognizer".into()))?;
        Ok(Self { name: cfg.model_name, recognizer })
    }
}

impl OfflineEngine for SherpaOfflineEngine {
    fn name(&self) -> &str {
        &self.name
    }

    fn transcribe(&mut self, samples: &[f32]) -> Result<String, SttError> {
        // One stream per segment (offline streams are single-use).
        let stream = self.recognizer.create_stream();
        stream.accept_waveform(SAMPLE_RATE as i32, samples);
        self.recognizer.decode(&stream);
        stream
            .get_result()
            .map(|r| r.text)
            .ok_or_else(|| SttError::Engine("offline decode returned no result".into()))
    }
}

/// Silero VAD parameters (seconds); sherpa's derived defaults are all 0.
#[derive(Debug, Clone)]
pub struct SileroVadConfig {
    /// `silero_vad.onnx` (k2-fsa asr-models release, not in the manifest yet).
    pub model: PathBuf,
    pub threshold: f32,
    pub min_silence_duration: f32,
    pub min_speech_duration: f32,
    pub max_speech_duration: f32,
    /// 512 samples at 16 kHz for Silero v4/v5.
    pub window_size: i32,
    pub buffer_seconds: f32,
}

impl SileroVadConfig {
    pub fn new(model: impl Into<PathBuf>) -> Self {
        Self {
            model: model.into(),
            threshold: 0.5,
            min_silence_duration: 0.5,
            min_speech_duration: 0.25,
            max_speech_duration: 12.0,
            window_size: 512,
            buffer_seconds: 30.0,
        }
    }
}

/// [`Segmenter`] backed by sherpa-onnx's Silero VAD.
pub struct SileroSegmenter {
    vad: sherpa_onnx::VoiceActivityDetector,
}

impl std::fmt::Debug for SileroSegmenter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SileroSegmenter").finish_non_exhaustive()
    }
}

impl SileroSegmenter {
    pub fn new(cfg: &SileroVadConfig) -> Result<Self, SttError> {
        if !cfg.model.is_file() {
            return Err(SttError::Model(format!("missing {}", cfg.model.display())));
        }
        let mut c = sherpa_onnx::VadModelConfig::default();
        c.silero_vad.model = Some(cfg.model.to_string_lossy().into_owned());
        c.silero_vad.threshold = cfg.threshold;
        c.silero_vad.min_silence_duration = cfg.min_silence_duration;
        c.silero_vad.min_speech_duration = cfg.min_speech_duration;
        c.silero_vad.max_speech_duration = cfg.max_speech_duration;
        c.silero_vad.window_size = cfg.window_size;
        c.sample_rate = SAMPLE_RATE as i32;
        c.num_threads = 1;
        let vad = sherpa_onnx::VoiceActivityDetector::create(&c, cfg.buffer_seconds)
            .ok_or_else(|| SttError::Engine("failed to create VoiceActivityDetector".into()))?;
        Ok(Self { vad })
    }
}

impl Segmenter for SileroSegmenter {
    fn accept(&mut self, samples: &[f32]) {
        self.vad.accept_waveform(samples);
    }

    fn pop(&mut self) -> Option<Segment> {
        if self.vad.is_empty() {
            return None;
        }
        let seg = self.vad.front()?;
        let out =
            Segment { start_sample: seg.start().max(0) as u64, samples: seg.samples().to_vec() };
        drop(seg);
        self.vad.pop();
        Some(out)
    }

    fn flush(&mut self) {
        self.vad.flush();
    }

    fn reset(&mut self) {
        // VERIFY: whether `reset` also restarts `SpeechSegment::start()`
        // counting; ages are computed from our own sample counter anyway.
        self.vad.reset();
        self.vad.clear();
    }
}
