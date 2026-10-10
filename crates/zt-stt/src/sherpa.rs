//! sherpa-onnx streaming adapter (feature `sherpa`).
//!
//! Written against the official `sherpa-onnx` crate 1.13.8 (k2-fsa,
//! Apache-2.0), API checked on https://docs.rs/sherpa-onnx/1.13.8 :
//! `OnlineRecognizer::create(&OnlineRecognizerConfig) -> Option<Self>`,
//! `create_stream()`, `OnlineStream::accept_waveform(&self, i32, &[f32])`,
//! `is_ready/decode/is_endpoint/reset(&self, &OnlineStream)`,
//! `get_result(&self, &OnlineStream) -> Option<RecognizerResult>`.
//! Both `OnlineRecognizer` and `OnlineStream` are `Send + Sync`.
//! Not compiled on the authoring machine (no Rust toolchain).
//! All calls into the crate are confined to `binding` below.

use crate::{Hotword, SpeechEngine, SttError, SttEvent, SAMPLE_RATE};
use std::path::PathBuf;

/// Model + decoding configuration for a streaming Zipformer transducer
/// (e.g. `sherpa-onnx-streaming-zipformer-de-kroko-2025-08-06`).
#[derive(Debug, Clone)]
pub struct SherpaConfig {
    pub encoder: PathBuf,
    pub decoder: PathBuf,
    pub joiner: PathBuf,
    pub tokens: PathBuf,
    /// `bpe.vocab` (NOT `bpe.model`); needed so plain-text hotwords can be
    /// tokenized for BPE models. `None` disables hotwords.
    pub bpe_vocab: Option<PathBuf>,
    /// "bpe", "cjkchar" or "cjkchar+bpe" (must match the model).
    pub modeling_unit: String,
    pub num_threads: i32,
    /// Hotwords require `modified_beam_search` (greedy_search ignores them).
    pub decoding_method: String,
    /// Beam size for modified_beam_search. Crate default is 0, so set it.
    pub max_active_paths: i32,
    pub hotwords_score: f32,
    /// Endpointing rules (seconds); sherpa CLI defaults 2.4 / 1.2 / 20.
    /// The crate's `Default` sets them to 0.0, so they must be set explicitly.
    pub rule1_min_trailing_silence: f32,
    pub rule2_min_trailing_silence: f32,
    pub rule3_min_utterance_length: f32,
    pub model_name: String,
}

impl SherpaConfig {
    pub fn zipformer(dir: impl Into<PathBuf>, model_name: impl Into<String>) -> Self {
        let dir = dir.into();
        let vocab = dir.join("bpe.vocab");
        Self {
            // VERIFY: file names differ per model release (often *.int8.onnx with
            // epoch/chunk suffixes); the zt-models manifest provides the actual names.
            encoder: dir.join("encoder.onnx"),
            decoder: dir.join("decoder.onnx"),
            joiner: dir.join("joiner.onnx"),
            tokens: dir.join("tokens.txt"),
            bpe_vocab: vocab.is_file().then_some(vocab),
            modeling_unit: "bpe".into(),
            num_threads: 2,
            decoding_method: "modified_beam_search".into(),
            max_active_paths: 4,
            hotwords_score: 1.75,
            rule1_min_trailing_silence: 2.4,
            rule2_min_trailing_silence: 0.8,
            rule3_min_utterance_length: 20.0,
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

/// Narrow internal port over the binding; the only surface that must match the crate.
trait StreamingRecognizer: Send {
    fn accept_waveform(&mut self, sample_rate: i32, samples: &[f32]);
    /// Decodes everything that is ready and returns the current text.
    fn decode_and_text(&mut self) -> String;
    fn is_endpoint(&mut self) -> bool;
    fn reset(&mut self);
}

pub struct SherpaEngine {
    cfg: SherpaConfig,
    rec: Box<dyn StreamingRecognizer>,
    last_partial: String,
    in_speech: bool,
    events: Vec<SttEvent>,
}

impl std::fmt::Debug for SherpaEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SherpaEngine").field("model", &self.cfg.model_name).finish()
    }
}

impl SherpaEngine {
    pub fn new(cfg: SherpaConfig) -> Result<Self, SttError> {
        cfg.validate()?;
        let rec = binding::create(&cfg, None)?;
        Ok(Self { cfg, rec, last_partial: String::new(), in_speech: false, events: Vec::new() })
    }
}

impl SpeechEngine for SherpaEngine {
    fn name(&self) -> &str {
        &self.cfg.model_name
    }

    fn accept_waveform(&mut self, samples: &[f32]) {
        self.rec.accept_waveform(SAMPLE_RATE as i32, samples);
        let text = self.rec.decode_and_text().trim().to_string();
        if !text.is_empty() && text != self.last_partial {
            if !self.in_speech {
                self.in_speech = true;
                self.events.push(SttEvent::Vad(true));
            }
            self.events.push(SttEvent::Partial(text.clone()));
            self.last_partial = text.clone();
        }
        if self.rec.is_endpoint() {
            if !text.is_empty() {
                self.events.push(SttEvent::Final(text));
            }
            if self.in_speech {
                self.in_speech = false;
                self.events.push(SttEvent::Vad(false));
            }
            self.last_partial.clear();
            self.rec.reset();
        }
    }

    fn poll(&mut self) -> Vec<SttEvent> {
        std::mem::take(&mut self.events)
    }

    fn set_hotwords(&mut self, hotwords: &[Hotword]) -> Result<(), SttError> {
        if self.cfg.bpe_vocab.is_none() && self.cfg.modeling_unit.contains("bpe") {
            return Err(SttError::Unsupported("hotwords need bpe.vocab for BPE models"));
        }
        // Hotwords file format: one entry per line, optional trailing ` :score`.
        // Passed via `hotwords_buf` (in-memory replacement for hotwords_file),
        // so the recognizer is recreated.
        let lines: Vec<String> = hotwords.iter().map(Hotword::to_sherpa_line).collect();
        self.rec = binding::create(&self.cfg, Some(lines.join("\n")))?;
        self.last_partial.clear();
        self.in_speech = false;
        Ok(())
    }

    fn reset(&mut self) {
        self.rec.reset();
        self.last_partial.clear();
        self.events.clear();
    }
}

mod binding {
    use super::{SherpaConfig, StreamingRecognizer};
    use crate::SttError;

    // OnlineRecognizer and OnlineStream are both Send + Sync per docs.rs,
    // so no unsafe impl is needed.
    struct Rec {
        recognizer: sherpa_onnx::OnlineRecognizer,
        stream: sherpa_onnx::OnlineStream,
    }

    impl StreamingRecognizer for Rec {
        fn accept_waveform(&mut self, sample_rate: i32, samples: &[f32]) {
            self.stream.accept_waveform(sample_rate, samples);
        }
        fn decode_and_text(&mut self) -> String {
            while self.recognizer.is_ready(&self.stream) {
                self.recognizer.decode(&self.stream);
            }
            self.recognizer.get_result(&self.stream).map(|r| r.text).unwrap_or_default()
        }
        fn is_endpoint(&mut self) -> bool {
            self.recognizer.is_endpoint(&self.stream)
        }
        fn reset(&mut self) {
            self.recognizer.reset(&self.stream);
        }
    }

    pub(super) fn create(
        cfg: &SherpaConfig,
        hotwords: Option<String>,
    ) -> Result<Box<dyn StreamingRecognizer>, SttError> {
        let p = |x: &std::path::Path| x.to_string_lossy().into_owned();
        let mut c = sherpa_onnx::OnlineRecognizerConfig::default();
        // Default feat_config is already 16 kHz / 80-dim fbank.
        c.model_config.transducer.encoder = Some(p(&cfg.encoder));
        c.model_config.transducer.decoder = Some(p(&cfg.decoder));
        c.model_config.transducer.joiner = Some(p(&cfg.joiner));
        c.model_config.tokens = Some(p(&cfg.tokens));
        c.model_config.num_threads = cfg.num_threads;
        c.model_config.modeling_unit = Some(cfg.modeling_unit.clone());
        c.model_config.bpe_vocab = cfg.bpe_vocab.as_deref().map(p);
        c.decoding_method = Some(cfg.decoding_method.clone());
        c.max_active_paths = cfg.max_active_paths;
        c.enable_endpoint = true;
        c.rule1_min_trailing_silence = cfg.rule1_min_trailing_silence;
        c.rule2_min_trailing_silence = cfg.rule2_min_trailing_silence;
        c.rule3_min_utterance_length = cfg.rule3_min_utterance_length;
        c.hotwords_score = cfg.hotwords_score;
        c.hotwords_buf = hotwords.filter(|h| !h.is_empty()).map(String::into_bytes);
        let recognizer = sherpa_onnx::OnlineRecognizer::create(&c)
            .ok_or_else(|| SttError::Engine("failed to create OnlineRecognizer".into()))?;
        let stream = recognizer.create_stream();
        Ok(Box::new(Rec { recognizer, stream }))
    }
}
