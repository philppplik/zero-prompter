//! Hotword selection from the script (RESEARCH.md: hotword score 1.75).
//! Rare, long words (names, jargon) profit most from biasing; function words
//! would only distort decoding, so they are excluded.

use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Hotword {
    pub text: String,
    pub score: f32,
}

impl Hotword {
    /// Line format of sherpa-onnx hotword files: `TEXT :score`.
    /// VERIFY: for BPE models sherpa expects the text in the model's casing
    /// (often upper case); `modeling_unit`/`bpe_vocab` must be configured.
    pub fn to_sherpa_line(&self) -> String {
        format!("{} :{:.2}", self.text.to_uppercase(), self.score)
    }
}

#[rustfmt::skip]
const STOP: &[&str] = &[
    "aber", "alle", "also", "auch", "dann", "dass", "denn", "diese", "dieser", "doch", "eine",
    "einem", "einen", "einer", "eines", "haben", "heute", "hier", "immer", "jetzt", "kann", "mehr",
    "nicht", "noch", "oder", "schon", "sehr", "sein", "sind", "über", "und", "weil", "wenn",
    "werden", "wird", "about", "after", "also", "been", "could", "from", "have", "just", "more",
    "only", "that", "their", "there", "these", "they", "this", "what", "when", "which", "will",
    "with", "would", "your",
];

/// Picks up to `max` hotwords: words with ≥ `min_len` letters, not stop words,
/// ranked by (capitalised first, rarer first, longer first). Punctuation stripped,
/// original spelling (with umlauts) kept because the STT emits it that way.
pub fn build_hotwords<'a>(
    words: impl IntoIterator<Item = &'a str>,
    min_len: usize,
    max: usize,
    score: f32,
) -> Vec<Hotword> {
    let mut freq: BTreeMap<String, (usize, bool)> = BTreeMap::new();
    for raw in words {
        let w: String = raw.chars().filter(|c| c.is_alphanumeric() || *c == '-').collect();
        let w = w.trim_matches('-').to_string();
        if w.chars().count() < min_len {
            continue;
        }
        let lower = w.to_lowercase();
        if STOP.contains(&lower.as_str()) {
            continue;
        }
        let cap = w.chars().next().map(char::is_uppercase).unwrap_or(false);
        let e = freq.entry(lower).or_insert((0, false));
        e.0 += 1;
        e.1 |= cap;
    }
    let mut v: Vec<(String, usize, bool)> =
        freq.into_iter().map(|(w, (n, cap))| (w, n, cap)).collect();
    v.sort_by(|a, b| {
        b.2.cmp(&a.2)
            .then(a.1.cmp(&b.1))
            .then(b.0.chars().count().cmp(&a.0.chars().count()))
            .then(a.0.cmp(&b.0))
    });
    v.into_iter().take(max).map(|(text, _, _)| Hotword { text, score }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selects_rare_long_words() {
        let text = "Heute zeige ich euch Kubernetes und Kubernetes-Cluster, aber nicht alles.";
        let hw = build_hotwords(text.split_whitespace(), 5, 10, 1.75);
        let words: Vec<_> = hw.iter().map(|h| h.text.as_str()).collect();
        assert!(words.contains(&"kubernetes"));
        assert!(words.contains(&"kubernetes-cluster"));
        assert!(!words.contains(&"heute"));
        assert!(!words.contains(&"aber"));
        assert_eq!(hw[0].to_sherpa_line().split(" :").nth(1), Some("1.75"));
    }

    #[test]
    fn respects_max() {
        let hw = build_hotwords(["Alpha", "Bravo", "Charlie", "Delta"], 4, 2, 2.0);
        assert_eq!(hw.len(), 2);
    }
}
