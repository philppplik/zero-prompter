//! Model-free checks that mirror sherpa-onnx's C++ config validation, so a
//! failing `OnlineRecognizer::create` (which only returns `None`; the reason
//! goes to the C++ stderr, invisible in a GUI app) can be explained precisely.
//!
//! Rules mirrored from sherpa-onnx v1.13.8 `csrc/`:
//! - `online-model-config.cc` `Validate`: `num_threads > 0`; if
//!   `modeling_unit` is `bpe` or `cjkchar+bpe`, `bpe_vocab` must exist
//!   (checked even when no hotwords are given!).
//! - `online-transducer-model-config.cc`: encoder/decoder/joiner must exist.
//! - `online-recognizer.cc`: `max_active_paths > 0` for modified_beam_search,
//!   hotwords only with modified_beam_search.
//! - `utils.cc` `EncodeHotwords`: with hotwords and an empty / unknown
//!   `modeling_unit` sherpa calls `exit(-1)` (whole process!), so hotwords
//!   are only ever passed with `modeling_unit = "bpe"` + a bpe vocab.
//!
//! Always compiled (no sherpa dependency) so it is unit tested in core CI.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// SentencePiece word-boundary marker used in `tokens.txt` of BPE models.
pub const SP_MARK: char = '\u{2581}';

/// Checks that every file exists, is a regular file and is non-empty.
/// Returns one line per file (`role=path (N bytes)`) or a precise error
/// naming every missing / empty file.
pub fn check_files(files: &[(&str, &Path)]) -> Result<Vec<String>, String> {
    let mut ok = Vec::new();
    let mut bad = Vec::new();
    for (role, p) in files {
        match std::fs::metadata(p) {
            Ok(m) if m.is_file() && m.len() > 0 => {
                ok.push(format!("{role}={} ({} bytes)", p.display(), m.len()))
            }
            Ok(m) if m.is_file() => bad.push(format!("{role}: {} is empty (0 bytes)", p.display())),
            Ok(_) => bad.push(format!("{role}: {} is not a file", p.display())),
            Err(e) => bad.push(format!("{role}: {} missing ({e})", p.display())),
        }
    }
    if bad.is_empty() {
        Ok(ok)
    } else {
        Err(bad.join("; "))
    }
}

/// Paths are handed to sherpa-onnx as narrow `std::string`s; on Windows the
/// C++ side opens them in the ANSI code page, so non-ASCII paths may fail.
pub fn non_ascii_warning(p: &Path) -> Option<String> {
    let s = p.to_string_lossy();
    (cfg!(windows) && !s.is_ascii()).then(|| {
        format!("path contains non-ASCII characters (may fail in sherpa-onnx on Windows): {s}")
    })
}

/// Recognizer settings as sherpa-onnx validates them.
#[derive(Debug, Clone)]
pub struct RecognizerSpec<'a> {
    pub num_threads: i32,
    pub decoding_method: &'a str,
    pub max_active_paths: i32,
    pub modeling_unit: Option<&'a str>,
    pub bpe_vocab: Option<&'a Path>,
    pub has_hotwords: bool,
}

/// Re-implements sherpa-onnx's validation; returns the error text sherpa
/// would print to stderr.
pub fn validate_spec(s: &RecognizerSpec<'_>) -> Result<(), String> {
    if s.num_threads <= 0 {
        return Err(format!("num_threads should be > 0. Given {}", s.num_threads));
    }
    if !matches!(s.decoding_method, "greedy_search" | "modified_beam_search") {
        return Err(format!("unsupported decoding_method '{}'", s.decoding_method));
    }
    if s.decoding_method == "modified_beam_search" && s.max_active_paths <= 0 {
        return Err(format!("max_active_paths must be > 0. Given: {}", s.max_active_paths));
    }
    if s.has_hotwords && s.decoding_method != "modified_beam_search" {
        return Err(format!(
            "hotwords need decoding_method=modified_beam_search. Given {}",
            s.decoding_method
        ));
    }
    match s.modeling_unit {
        Some("bpe") | Some("cjkchar+bpe") => match s.bpe_vocab {
            Some(v) if v.is_file() => {}
            Some(v) => return Err(format!("bpe_vocab: '{}' does not exist", v.display())),
            None => {
                return Err(
                    "bpe_vocab: '' does not exist (modeling_unit=bpe requires bpe_vocab)".into()
                )
            }
        },
        Some("cjkchar") | None => {}
        Some(other) => return Err(format!("unknown modeling_unit '{other}'")),
    }
    if s.has_hotwords && s.modeling_unit.is_none() {
        return Err("hotwords without modeling_unit would abort the process in sherpa-onnx".into());
    }
    Ok(())
}

/// Token table of a transducer model (`tokens.txt`: `<piece> <id>` per line).
#[derive(Debug, Clone)]
pub struct TokenVocab {
    pieces: Vec<(String, i64)>,
    chars: HashSet<char>,
}

impl TokenVocab {
    pub fn load(path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path)
            .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
        Ok(Self::parse(&text))
    }

    pub fn parse(text: &str) -> Self {
        let mut pieces = Vec::new();
        let mut chars = HashSet::new();
        for line in text.lines() {
            let mut it = line.split_whitespace();
            let (Some(piece), Some(id), None) = (it.next(), it.next(), it.next()) else {
                continue;
            };
            let Ok(id) = id.parse::<i64>() else { continue };
            // Specials (<blk>, <unk>, <sos/eos>) say nothing about the casing.
            if !(piece.len() > 2 && piece.starts_with('<') && piece.ends_with('>')) {
                chars.extend(piece.chars().filter(|c| *c != SP_MARK));
            }
            pieces.push((piece.to_string(), id));
        }
        Self { pieces, chars }
    }

    pub fn len(&self) -> usize {
        self.pieces.len()
    }

    pub fn is_empty(&self) -> bool {
        self.pieces.is_empty()
    }

    /// True for SentencePiece BPE token tables (pieces with `▁`).
    pub fn is_sentencepiece(&self) -> bool {
        self.pieces.iter().any(|(p, _)| p.starts_with(SP_MARK))
    }

    /// True if the model emits lower-case letters (mixed case, e.g. SentencePiece DE models);
    /// false for upper-case-only models (LibriSpeech Zipformer).
    pub fn has_lowercase(&self) -> bool {
        self.chars.iter().any(|c| c.is_lowercase())
    }

    /// Synthesizes a `bpe.vocab` for sherpa's simple-sentencepiece encoder
    /// (`<piece> <score>` per line, parsed with `>>`, malformed line = exit).
    /// Scores: constant -1 per piece (fewest pieces wins, like BPE merges),
    /// with a tiny id-based tie-break toward frequent (low-id) pieces.
    pub fn bpe_vocab_text(&self) -> String {
        let mut out = String::new();
        for (piece, id) in &self.pieces {
            let disambig = piece.len() > 1
                && piece.starts_with('#')
                && piece[1..].chars().all(|c| c.is_ascii_digit());
            // <unk> stays: simple-sentencepiece uses it for uncovered input.
            if matches!(piece.as_str(), "<blk>" | "<sos/eos>" | "<s>" | "</s>") || disambig {
                continue; // blank / sentence markers, #0.. disambiguation ids
            }
            let score = -1.0 - (*id as f64) * 1e-6;
            out.push_str(&format!("{piece} {score:.6}\n"));
        }
        out
    }

    /// Hotword line in model casing; `None` if a character is not covered by
    /// the vocabulary (sherpa would log an OOV and skip it anyway).
    pub fn hotword_line(&self, text: &str, score: f32) -> Option<String> {
        let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
        if text.is_empty() {
            return None;
        }
        let text = if self.has_lowercase() { text } else { text.to_uppercase() };
        if !text.chars().filter(|c| !c.is_whitespace()).all(|c| self.chars.contains(&c)) {
            return None;
        }
        Some(format!("{text} :{score:.2}"))
    }
}

/// Where a synthesized bpe vocab goes: next to `tokens.txt`, or, if the model
/// dir is read-only, into the temp dir.
pub fn ensure_bpe_vocab(tokens: &Path, vocab: &TokenVocab) -> Result<PathBuf, String> {
    let text = vocab.bpe_vocab_text();
    let name = "zt-generated-bpe.vocab";
    let primary = tokens.with_file_name(name);
    let fallback = std::env::temp_dir().join(format!(
        "zt-{}-{name}",
        tokens
            .parent()
            .and_then(Path::file_name)
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default()
    ));
    for p in [primary, fallback] {
        if std::fs::read_to_string(&p).map(|t| t == text).unwrap_or(false) {
            return Ok(p);
        }
        if std::fs::write(&p, &text).is_ok() {
            return Ok(p);
        }
    }
    Err("cannot write generated bpe vocab (model dir and temp dir not writable)".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    const MIXED: &str = "<blk> 0\n<sos/eos> 1\n<unk> 2\n\u{2581} 3\nt 4\ne 5\n\u{2581}und 14\n\u{2581}T 85\nele 900\np 39\nr 12\no 13\nm 19\n#0 650\n";
    const LIBRI: &str = "<blk> 0\n\u{2581}THE 3\nS 4\nA 5\nB 6\n";

    #[test]
    fn detects_sentencepiece_and_case() {
        let k = TokenVocab::parse(MIXED);
        assert!(k.is_sentencepiece());
        assert!(k.has_lowercase());
        let l = TokenVocab::parse(LIBRI);
        assert!(!l.has_lowercase());
    }

    #[test]
    fn bpe_vocab_skips_specials_and_is_two_columns() {
        let v = TokenVocab::parse(MIXED).bpe_vocab_text();
        assert!(!v.contains("<blk>") && !v.contains("#0"));
        for line in v.lines() {
            assert_eq!(line.split_whitespace().count(), 2, "{line}");
        }
        assert!(v.contains("\u{2581}und -1.000014"));
    }

    #[test]
    fn hotword_casing_follows_model() {
        let k = TokenVocab::parse(MIXED);
        assert_eq!(k.hotword_line("Tempo", 1.5).as_deref(), Some("Tempo :1.50"));
        assert_eq!(k.hotword_line("  tempo  ", 1.5).as_deref(), Some("tempo :1.50"));
        assert_eq!(k.hotword_line("Zebra", 1.5), None);
        let l = TokenVocab::parse(LIBRI);
        assert_eq!(l.hotword_line("abs", 2.0).as_deref(), Some("ABS :2.00"));
        assert_eq!(l.hotword_line("xyz", 2.0), None);
    }

    #[test]
    fn validate_mirrors_sherpa_rules() {
        let base = RecognizerSpec {
            num_threads: 2,
            decoding_method: "modified_beam_search",
            max_active_paths: 4,
            modeling_unit: Some("bpe"),
            bpe_vocab: None,
            has_hotwords: false,
        };
        // The production bug: modeling_unit=bpe without bpe.vocab.
        assert!(validate_spec(&base).unwrap_err().contains("bpe_vocab"));
        let ok = RecognizerSpec { modeling_unit: None, ..base.clone() };
        assert!(validate_spec(&ok).is_ok());
        let hw = RecognizerSpec { has_hotwords: true, ..ok.clone() };
        assert!(validate_spec(&hw).is_err());
        let greedy = RecognizerSpec { decoding_method: "greedy_search", max_active_paths: 0, ..ok };
        assert!(validate_spec(&greedy).is_ok());
    }

    #[test]
    fn check_files_names_missing() {
        let err = check_files(&[("encoder", Path::new("/definitely/missing.onnx"))]).unwrap_err();
        assert!(err.contains("encoder") && err.contains("missing"));
    }
}
