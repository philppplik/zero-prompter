//! Fuzzy word similarity in `[0, 1]` (Levenshtein ratio and Jaro-Winkler).
//! Own implementation (textbook algorithms) to keep `zt-align` dependency-free.

/// Normalised Levenshtein similarity: `1 - dist / max_len`.
pub fn levenshtein_ratio(a: &str, b: &str) -> f32 {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let max = a.len().max(b.len());
    if max == 0 {
        return 1.0;
    }
    1.0 - levenshtein(&a, &b) as f32 / max as f32
}

pub fn levenshtein(a: &[char], b: &[char]) -> usize {
    if a.is_empty() {
        return b.len();
    }
    if b.is_empty() {
        return a.len();
    }
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut cur = vec![0usize; b.len() + 1];
    for (i, ca) in a.iter().enumerate() {
        cur[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let sub = prev[j] + usize::from(ca != cb);
            cur[j + 1] = sub.min(prev[j + 1] + 1).min(cur[j] + 1);
        }
        std::mem::swap(&mut prev, &mut cur);
    }
    prev[b.len()]
}

pub fn jaro(a: &str, b: &str) -> f32 {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    let range = (a.len().max(b.len()) / 2).saturating_sub(1);
    let mut a_m = vec![false; a.len()];
    let mut b_m = vec![false; b.len()];
    let mut matches = 0usize;
    for (i, ca) in a.iter().enumerate() {
        let lo = i.saturating_sub(range);
        let hi = (i + range + 1).min(b.len());
        if lo >= hi {
            continue;
        }
        if let Some(off) = (lo..hi).position(|j| !b_m[j] && *ca == b[j]) {
            a_m[i] = true;
            b_m[lo + off] = true;
            matches += 1;
        }
    }
    if matches == 0 {
        return 0.0;
    }
    // Transpositions: compare matched chars of `a` and `b` in order.
    let b_matched = b.iter().zip(&b_m).filter(|(_, m)| **m).map(|(c, _)| c);
    let a_matched = a.iter().zip(&a_m).filter(|(_, m)| **m).map(|(c, _)| c);
    let t = a_matched.zip(b_matched).filter(|(x, y)| x != y).count();
    let m = matches as f32;
    (m / a.len() as f32 + m / b.len() as f32 + (m - t as f32 / 2.0) / m) / 3.0
}

pub fn jaro_winkler(a: &str, b: &str) -> f32 {
    let j = jaro(a, b);
    let prefix = a.chars().zip(b.chars()).take(4).take_while(|(x, y)| x == y).count();
    j + prefix as f32 * 0.1 * (1.0 - j)
}

/// Word similarity used by the aligner. Short words (< 4 chars) use the
/// stricter Levenshtein ratio only ("und" vs "uns" must not match strongly);
/// longer words with the same initial take the max of both measures
/// (robust to STT suffix errors).
pub fn word_similarity(a: &str, b: &str) -> f32 {
    if a == b {
        return 1.0;
    }
    let lev = levenshtein_ratio(a, b);
    // Jaro-Winkler is generous to single-letter swaps at the start ("haus"/"maus"),
    // so it is only consulted when the first letters agree.
    if a.chars().count() < 4 || b.chars().count() < 4 || a.chars().next() != b.chars().next() {
        lev
    } else {
        lev.max(jaro_winkler(a, b))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn levenshtein_basics() {
        assert_eq!(levenshtein(&['a', 'b', 'c'], &['a', 'b', 'c']), 0);
        assert_eq!(levenshtein_ratio("kitten", "sitting"), 1.0 - 3.0 / 7.0);
        assert_eq!(levenshtein_ratio("", ""), 1.0);
    }

    #[test]
    fn jaro_winkler_reference_values() {
        // Classic reference: MARTHA/MARHTA = 0.9611
        assert!((jaro_winkler("martha", "marhta") - 0.9611).abs() < 1e-3);
        assert!((jaro("dixon", "dicksonx") - 0.7667).abs() < 1e-3);
        assert_eq!(jaro("abc", "xyz"), 0.0);
    }

    #[test]
    fn word_similarity_behaviour() {
        assert_eq!(word_similarity("hallo", "hallo"), 1.0);
        assert!(word_similarity("teleprompter", "teleprompta") > 0.85);
        assert!(word_similarity("und", "uns") < 0.7);
        assert!(word_similarity("haus", "maus") < 0.8);
    }
}
