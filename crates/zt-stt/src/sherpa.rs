//! sherpa-onnx streaming adapter (feature `sherpa`).
//!
//! Written against the official `sherpa-onnx` crate 1.13.8 (k2-fsa,
//! Apache-2.0): `OnlineRecognizer::create(&OnlineRecognizerConfig) ->
//! Option<Self>`, `create_stream()`, `OnlineStream::accept_waveform`,
//! `is_ready/decode/is_endpoint/reset`, `get_result -> Option<RecognizerResult>`.
//! All calls into the crate are confined to `binding` below.
//!
//! Robustness (bug fixed 2026-10: "failed to create OnlineRecognizer" after
//! 1 ms with the Kroko models): sherpa's `OnlineModelConfig::Validate`
//! rejects `modeling_unit = "bpe"` whenever `bpe_vocab` does not exist, and
//! none of the bundled models ships a `bpe.vocab`. Now:
//! 1. preflight checks every file (existence + size) and mirrors sherpa's
//!    validation in Rust ([`crate::model_check`]) for precise error texts;
//! 2. `modeling_unit`/`bpe_vocab` are only passed together; for
//!    SentencePiece token tables without `bpe.vocab` one is synthesized from
//!    `tokens.txt` so hotwords keep working;
//! 3. creation runs a fallback ladder, every stage reported to the log sink:
//!    beam+hotwords -> beam without hotwords -> greedy_search.

use crate::model_check::{self, RecognizerSpec, TokenVocab};
use crate::{Hotword, SpeechEngine, SttError, SttEvent, SAMPLE_RATE};
use std::path::PathBuf;
use std::sync::Arc;

/// Log callback (the crate has no tracing dependency; the app forwards
/// these lines to its log file).
#[derive(Clone)]
pub struct LogSink(pub Arc<dyn Fn(&str) + Send + Sync>);

impl std::fmt::Debug for LogSink {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("LogSink")
    }
}

/// Model + decoding configuration for a streaming Zipformer transducer
/// (e.g. `sherpa-onnx-streaming-zipformer-de-kroko-2025-08-06`).
#[derive(Debug, Clone)]
pub struct SherpaConfig {
    pub encoder: PathBuf,
    pub decoder: PathBuf,
    pub joiner: PathBuf,
    pub tokens: PathBuf,
    /// `bpe.vocab` (NOT `bpe.model`). `None`: synthesized from `tokens.txt`
    /// for SentencePiece models, otherwise hotwords are disabled.
    pub bpe_vocab: Option<PathBuf>,
    /// "bpe", "cjkchar" or "cjkchar+bpe" (only passed with a bpe vocab).
    pub modeling_unit: String,
    pub num_threads: i32,
    /// Preferred method; hotwords require `modified_beam_search`.
    pub decoding_method: String,
    /// Beam size for modified_beam_search. Crate default is 0, so set it.
    pub max_active_paths: i32,
    pub hotwords_score: f32,
    /// Endpointing rules (seconds); sherpa CLI defaults 2.4 / 1.2 / 20.
    pub rule1_min_trailing_silence: f32,
    pub rule2_min_trailing_silence: f32,
    pub rule3_min_utterance_length: f32,
    pub model_name: String,
    /// ONNX Runtime provider ("cpu").
    pub provider: String,
    /// sherpa `debug` flag (verbose C++ output on stderr; diagnostics mode).
    pub debug: bool,
    /// Per-stream `language` option for multilingual prompt models (NVIDIA
    /// Nemotron 3.5 streaming: "de", "en", "auto"). Set right after
    /// `create_stream` and after every `reset`. `None` for Zipformer models.
    pub language: Option<String>,
    pub log: Option<LogSink>,
}

impl SherpaConfig {
    pub fn zipformer(dir: impl Into<PathBuf>, model_name: impl Into<String>) -> Self {
        let dir = dir.into();
        let vocab = dir.join("bpe.vocab");
        Self {
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
            provider: "cpu".into(),
            debug: false,
            language: None,
            log: None,
        }
    }

    /// NVIDIA Nemotron 3.5 ASR streaming (NeMo cache-aware transducer).
    /// sherpa-onnx 1.13.8 only implements `greedy_search` for streaming NeMo
    /// transducers; any other method makes the C++ side call `exit(-1)`
    /// (whole process, not catchable), so this config never tries beam search
    /// and therefore has no hotwords. `language`: "de", "en" or "auto".
    pub fn nemotron(
        dir: impl Into<PathBuf>,
        model_name: impl Into<String>,
        language: impl Into<String>,
    ) -> Self {
        let dir = dir.into();
        Self {
            encoder: dir.join("encoder.int8.onnx"),
            decoder: dir.join("decoder.int8.onnx"),
            joiner: dir.join("joiner.int8.onnx"),
            num_threads: 4,
            decoding_method: "greedy_search".into(),
            language: Some(language.into()),
            ..Self::zipformer(dir, model_name)
        }
    }

    fn beam(&self) -> bool {
        self.decoding_method == "modified_beam_search"
    }

    fn log(&self, msg: &str) {
        if let Some(l) = &self.log {
            (l.0)(msg);
        }
    }

    /// Existence + size of all model files; logs sizes, names every missing file.
    pub fn preflight(&self) -> Result<(), SttError> {
        let files = [
            ("encoder", self.encoder.as_path()),
            ("decoder", self.decoder.as_path()),
            ("joiner", self.joiner.as_path()),
            ("tokens", self.tokens.as_path()),
        ];
        let lines = model_check::check_files(&files)
            .map_err(|e| SttError::Model(format!("{}: {e}", self.model_name)))?;
        self.log(&format!("preflight ok: {}", lines.join(", ")));
        for (_, p) in files {
            if let Some(w) = model_check::non_ascii_warning(p) {
                self.log(&w);
            }
        }
        Ok(())
    }
}

/// One concrete attempt of the creation ladder.
#[derive(Debug, Clone)]
struct Stage {
    label: &'static str,
    decoding_method: String,
    /// (modeling_unit, bpe_vocab) – always both or neither.
    bpe: Option<(String, PathBuf)>,
    hotwords: Option<String>,
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
    vocab: TokenVocab,
    /// Resolved bpe vocab (from the model or synthesized), if any.
    bpe: Option<(String, PathBuf)>,
    rec: Box<dyn StreamingRecognizer>,
    active: String,
    last_partial: String,
    in_speech: bool,
    events: Vec<SttEvent>,
}

impl std::fmt::Debug for SherpaEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SherpaEngine")
            .field("model", &self.cfg.model_name)
            .field("stage", &self.active)
            .finish()
    }
}

impl SherpaEngine {
    pub fn new(cfg: SherpaConfig) -> Result<Self, SttError> {
        cfg.preflight()?;
        let vocab = TokenVocab::load(&cfg.tokens).map_err(SttError::Model)?;
        if vocab.is_empty() {
            return Err(SttError::Model(format!(
                "{}: tokens.txt has no tokens",
                cfg.tokens.display()
            )));
        }
        let bpe = match &cfg.bpe_vocab {
            Some(v) if v.is_file() => Some((cfg.modeling_unit.clone(), v.clone())),
            _ if vocab.is_sentencepiece() => {
                match model_check::ensure_bpe_vocab(&cfg.tokens, &vocab) {
                    Ok(p) => {
                        cfg.log(&format!(
                            "no bpe.vocab; synthesized {} from tokens.txt ({} tokens)",
                            p.display(),
                            vocab.len()
                        ));
                        Some(("bpe".to_string(), p))
                    }
                    Err(e) => {
                        cfg.log(&format!("hotwords disabled: {e}"));
                        None
                    }
                }
            }
            _ => {
                cfg.log("hotwords disabled: no bpe.vocab and tokens.txt is not SentencePiece");
                None
            }
        };
        let mut me = Self {
            cfg,
            vocab,
            bpe,
            rec: Box::new(Dead),
            active: String::new(),
            last_partial: String::new(),
            in_speech: false,
            events: Vec::new(),
        };
        me.rebuild(None)?;
        Ok(me)
    }

    /// Name of the ladder stage the recognizer is running with.
    pub fn active_stage(&self) -> &str {
        &self.active
    }

    fn ladder(&self, hotwords: Option<String>) -> Vec<Stage> {
        let beam = "modified_beam_search".to_string();
        let mut v = Vec::new();
        if let (Some(h), Some(bpe), true) = (hotwords, &self.bpe, self.cfg.beam()) {
            v.push(Stage {
                label: "beam+hotwords",
                decoding_method: beam.clone(),
                bpe: Some(bpe.clone()),
                hotwords: Some(h),
            });
        }
        if self.cfg.beam() {
            v.push(Stage { label: "beam", decoding_method: beam, bpe: None, hotwords: None });
        }
        v.push(Stage {
            label: "greedy",
            decoding_method: "greedy_search".into(),
            bpe: None,
            hotwords: None,
        });
        v
    }

    fn rebuild(&mut self, hotwords: Option<String>) -> Result<(), SttError> {
        let mut errors = Vec::new();
        for st in self.ladder(hotwords) {
            let spec = RecognizerSpec {
                num_threads: self.cfg.num_threads,
                decoding_method: &st.decoding_method,
                max_active_paths: self.cfg.max_active_paths,
                modeling_unit: st.bpe.as_ref().map(|b| b.0.as_str()),
                bpe_vocab: st.bpe.as_ref().map(|b| b.1.as_path()),
                has_hotwords: st.hotwords.is_some(),
            };
            if let Err(e) = model_check::validate_spec(&spec) {
                self.cfg.log(&format!("stage {}: config rejected before create: {e}", st.label));
                errors.push(format!("{}: {e}", st.label));
                continue;
            }
            let t0 = std::time::Instant::now();
            match binding::create(&self.cfg, &st) {
                Ok(rec) => {
                    self.cfg.log(&format!(
                        "stage {} ok: decoding={} hotwords={} in {} ms",
                        st.label,
                        st.decoding_method,
                        st.hotwords.as_ref().map_or(0, |h| h.lines().count()),
                        t0.elapsed().as_millis()
                    ));
                    self.rec = rec;
                    self.active = st.label.to_string();
                    self.last_partial.clear();
                    self.in_speech = false;
                    return Ok(());
                }
                Err(e) => {
                    self.cfg.log(&format!(
                        "stage {} failed after {} ms: {e} (sherpa prints the reason to stderr; \
                         run with ZT_STT_DEBUG=1 from a console to see it)",
                        st.label,
                        t0.elapsed().as_millis()
                    ));
                    errors.push(format!("{}: {e}", st.label));
                }
            }
        }
        Err(SttError::Engine(format!(
            "failed to create OnlineRecognizer for {} (all stages failed: {})",
            self.cfg.model_name,
            errors.join("; ")
        )))
    }
}

/// Placeholder before the first successful create.
struct Dead;
impl StreamingRecognizer for Dead {
    fn accept_waveform(&mut self, _: i32, _: &[f32]) {}
    fn decode_and_text(&mut self) -> String {
        String::new()
    }
    fn is_endpoint(&mut self) -> bool {
        false
    }
    fn reset(&mut self) {}
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
        if self.bpe.is_none() {
            return Err(SttError::Unsupported("hotwords need a bpe vocab (model has none)"));
        }
        if !self.cfg.beam() {
            return Err(SttError::Unsupported(
                "hotwords need modified_beam_search; this model runs greedy_search only",
            ));
        }
        let lines: Vec<String> =
            hotwords.iter().filter_map(|h| self.vocab.hotword_line(&h.text, h.score)).collect();
        if lines.len() < hotwords.len() {
            self.cfg.log(&format!(
                "hotwords: {} of {} skipped (characters not in model vocabulary)",
                hotwords.len() - lines.len(),
                hotwords.len()
            ));
        }
        let buf = (!lines.is_empty()).then(|| lines.join("\n"));
        self.rebuild(buf)?;
        if lines.is_empty() && !hotwords.is_empty() {
            return Err(SttError::Unsupported("no hotword is encodable with this model's tokens"));
        }
        if !lines.is_empty() && self.active != "beam+hotwords" {
            return Err(SttError::Unsupported("hotwords rejected by sherpa-onnx; running without"));
        }
        Ok(())
    }

    fn reset(&mut self) {
        self.rec.reset();
        self.last_partial.clear();
        self.events.clear();
    }
}

mod binding {
    use super::{SherpaConfig, Stage, StreamingRecognizer};
    use crate::SttError;

    struct Rec {
        recognizer: sherpa_onnx::OnlineRecognizer,
        stream: sherpa_onnx::OnlineStream,
        language: Option<String>,
    }

    impl Rec {
        fn apply_language(&self) {
            if let Some(l) = &self.language {
                self.stream.set_option("language", l);
            }
        }
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
            self.apply_language();
        }
    }

    pub(super) fn create(
        cfg: &SherpaConfig,
        st: &Stage,
    ) -> Result<Box<dyn StreamingRecognizer>, SttError> {
        let p = |x: &std::path::Path| x.to_string_lossy().into_owned();
        let mut c = sherpa_onnx::OnlineRecognizerConfig::default();
        // Default feat_config is already 16 kHz / 80-dim fbank.
        c.model_config.transducer.encoder = Some(p(&cfg.encoder));
        c.model_config.transducer.decoder = Some(p(&cfg.decoder));
        c.model_config.transducer.joiner = Some(p(&cfg.joiner));
        c.model_config.tokens = Some(p(&cfg.tokens));
        c.model_config.num_threads = cfg.num_threads;
        c.model_config.provider = Some(cfg.provider.clone());
        c.model_config.debug = cfg.debug;
        // Only together: modeling_unit=bpe without an existing bpe_vocab is
        // rejected by OnlineModelConfig::Validate (the original bug).
        if let Some((unit, vocab)) = &st.bpe {
            c.model_config.modeling_unit = Some(unit.clone());
            c.model_config.bpe_vocab = Some(p(vocab));
        }
        c.decoding_method = Some(st.decoding_method.clone());
        c.max_active_paths = cfg.max_active_paths.max(1);
        c.enable_endpoint = true;
        c.rule1_min_trailing_silence = cfg.rule1_min_trailing_silence;
        c.rule2_min_trailing_silence = cfg.rule2_min_trailing_silence;
        c.rule3_min_utterance_length = cfg.rule3_min_utterance_length;
        c.hotwords_score = cfg.hotwords_score;
        c.hotwords_buf = st.hotwords.clone().filter(|h| !h.is_empty()).map(String::into_bytes);
        let recognizer = sherpa_onnx::OnlineRecognizer::create(&c)
            .ok_or_else(|| SttError::Engine("OnlineRecognizer::create returned None".into()))?;
        let stream = recognizer.create_stream();
        let rec = Rec { recognizer, stream, language: cfg.language.clone() };
        rec.apply_language();
        Ok(Box::new(rec))
    }
}
