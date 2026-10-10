//! Windowed Smith-Waterman (local alignment) of recent hypothesis words
//! against a window of normalised script words, with fuzzy word costs.
//! Complexity O(k * w); for k = 6, w = 70 this is ~420 cells per call.

use crate::similarity::word_similarity;

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SwParams {
    /// Minimum word similarity to count as a (fuzzy) match.
    pub sim_threshold: f32,
    /// Score of a perfect match (scaled by similarity and query weight).
    pub match_score: f32,
    pub mismatch: f32,
    /// Script word skipped by the speaker.
    pub gap_script: f32,
    /// Extra word spoken (ad-lib / STT insertion).
    pub gap_hyp: f32,
    /// Tie-break penalty per word of distance from the expected position.
    pub distance_penalty: f32,
    /// An alignment must end on one of the last `1 + max_trailing` query
    /// words. Old context words in the query must not out-vote what the
    /// speaker is saying *now*; 1 tolerates one garbled trailing partial.
    pub max_trailing: usize,
}

impl Default for SwParams {
    fn default() -> Self {
        Self {
            sim_threshold: 0.8,
            match_score: 2.0,
            mismatch: -1.5,
            gap_script: -1.0,
            gap_hyp: -1.0,
            distance_penalty: 0.01,
            max_trailing: 1,
        }
    }
}

/// One hypothesis word with a weight (finals > partials).
#[derive(Debug, Clone, PartialEq)]
pub struct QueryWord {
    pub text: String,
    pub weight: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SwHit {
    /// Index (into the full reference slice) of the script word aligned to
    /// the end of the local alignment, i.e. the last word spoken.
    pub end: usize,
    pub score: f32,
    /// `score / max_possible`, clamped to `[0, 1]`.
    pub confidence: f32,
    /// Number of fuzzy-matched word pairs on the best path.
    pub matches: u16,
}

/// Aligns `query` against `reference[range]`. `expected` is the position the
/// tracker expects next (for tie-breaking toward the nearest candidate).
pub fn align(
    query: &[QueryWord],
    reference: &[String],
    range: std::ops::Range<usize>,
    expected: usize,
    p: &SwParams,
) -> Option<SwHit> {
    let range = range.start.min(reference.len())..range.end.min(reference.len());
    let w = range.len();
    let k = query.len();
    if k == 0 || w == 0 {
        return None;
    }
    let cols = w + 1;
    let mut h = vec![0f32; (k + 1) * cols];
    let mut m = vec![0u16; (k + 1) * cols];
    // first query row (1-based) of the path ending in each cell
    let mut st = vec![0u16; (k + 1) * cols];
    // (adjusted, raw, end, matches, start_row)
    let mut best: Option<(f32, f32, usize, u16, u16)> = None;

    for i in 1..=k {
        let q = &query[i - 1];
        for j in 1..=w {
            let ref_idx = range.start + j - 1;
            let sim = word_similarity(&q.text, &reference[ref_idx]);
            let is_match = sim >= p.sim_threshold;
            let s = if is_match { p.match_score * sim * q.weight } else { p.mismatch };
            let diag = h[(i - 1) * cols + j - 1] + s;
            let up = h[(i - 1) * cols + j] + p.gap_hyp;
            let left = h[i * cols + j - 1] + p.gap_script;
            let start_of = |idx: usize, h: &[f32], st: &[u16]| {
                if h[idx] > 0.0 {
                    st[idx]
                } else {
                    i as u16
                }
            };
            let (val, mc, start) = if diag >= up && diag >= left {
                let pi = (i - 1) * cols + j - 1;
                (diag, m[pi] + u16::from(is_match), start_of(pi, &h, &st))
            } else if up >= left {
                let pi = (i - 1) * cols + j;
                (up, m[pi], start_of(pi, &h, &st))
            } else {
                let pi = i * cols + j - 1;
                (left, m[pi], start_of(pi, &h, &st))
            };
            let (val, mc, start) = if val <= 0.0 { (0.0, 0, 0) } else { (val, mc, start) };
            h[i * cols + j] = val;
            m[i * cols + j] = mc;
            st[i * cols + j] = start;
            // Only cells that end in a match on a recent query word are valid ends.
            if val > 0.0 && is_match && i + p.max_trailing >= k {
                let dist = (ref_idx as f32 - expected as f32).abs();
                let adj = val - p.distance_penalty * dist;
                // `!is_some_and(<=)` == "none or better"
                // (is_none_or needs Rust 1.82, MSRV is 1.80).
                if !best.is_some_and(|b| adj <= b.0) {
                    best = Some((adj, val, ref_idx, mc, start));
                }
            }
        }
    }
    best.map(|(_, raw, end, matches, start)| {
        // Confidence is relative to the query suffix from the alignment start:
        // leading words (older context / ad-libs) do not dilute a fresh match,
        // but trailing unmatched words (speaker went elsewhere) do.
        let from = (start as usize).saturating_sub(1);
        let max_possible: f32 = query[from..].iter().map(|q| p.match_score * q.weight).sum();
        SwHit {
            end,
            score: raw,
            confidence: if max_possible > 0.0 { (raw / max_possible).clamp(0.0, 1.0) } else { 0.0 },
            matches,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn q(words: &[&str]) -> Vec<QueryWord> {
        words.iter().map(|w| QueryWord { text: w.to_string(), weight: 1.0 }).collect()
    }
    fn r(words: &str) -> Vec<String> {
        words.split_whitespace().map(String::from).collect()
    }

    #[test]
    fn exact_match_in_window() {
        let reference = r("a b c der schnelle braune fuchs springt x y");
        let query = q(&["schnelle", "braune", "fuchs"]);
        let hit = align(&query, &reference, 0..reference.len(), 0, &SwParams::default()).unwrap();
        assert_eq!(hit.end, 6);
        assert_eq!(hit.matches, 3);
        assert!((hit.confidence - 1.0).abs() < 1e-6);
    }

    #[test]
    fn tolerates_adlib_and_skip() {
        let reference = r("heute zeige ich euch wie man einen teleprompter nutzt");
        // "ich" skipped, "also" inserted, STT typo on teleprompter
        let query = q(&["zeige", "euch", "also", "wie", "man", "einen", "teleprompta"]);
        let hit = align(&query, &reference, 0..reference.len(), 0, &SwParams::default()).unwrap();
        assert_eq!(hit.end, 7);
        assert!(hit.matches >= 5);
        assert!(hit.confidence > 0.55);
    }

    #[test]
    fn prefers_occurrence_near_expected() {
        let reference = r("wir sagen danke schoen und dann wieder wir sagen danke schoen");
        let p = SwParams::default();
        let query = q(&["sagen", "danke"]);
        let near_end = align(&query, &reference, 0..reference.len(), 9, &p).unwrap();
        assert_eq!(near_end.end, 9);
        let near_start = align(&query, &reference, 0..reference.len(), 1, &p).unwrap();
        assert_eq!(near_start.end, 2);
    }

    #[test]
    fn no_match_returns_none() {
        let reference = r("eins zwei drei");
        assert!(align(&q(&["xylophon"]), &reference, 0..3, 0, &SwParams::default()).is_none());
        assert!(align(&[], &reference, 0..3, 0, &SwParams::default()).is_none());
    }

    #[test]
    fn window_is_respected() {
        let reference = r("alpha beta gamma alpha beta gamma");
        let hit = align(&q(&["alpha", "beta"]), &reference, 3..6, 0, &SwParams::default()).unwrap();
        assert_eq!(hit.end, 4);
    }
}
