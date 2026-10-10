//! Normalised script index: maps normalised words back to script tokens.

use crate::normalize::{normalize_word, Lang};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ScriptIndex {
    /// Normalised words in reading order.
    pub words: Vec<String>,
    /// For every normalised word, the id of the script token it came from.
    pub owner: Vec<usize>,
    /// For every script token id, the first normalised word (None for cues /
    /// tokens that normalise to nothing).
    first_norm: Vec<Option<usize>>,
    pub lang: Lang,
}

impl ScriptIndex {
    /// Builds the index from `(token_id, raw_text)` pairs of *spoken* tokens
    /// (e.g. `zt_core::Script::spoken_tokens()`), in reading order.
    pub fn build<'a, I>(tokens: I, lang: Lang) -> Self
    where
        I: IntoIterator<Item = (usize, &'a str)>,
    {
        let mut idx = ScriptIndex { lang, ..Default::default() };
        for (id, raw) in tokens {
            if idx.first_norm.len() <= id {
                idx.first_norm.resize(id + 1, None);
            }
            for w in normalize_word(raw, lang) {
                if idx.first_norm[id].is_none() {
                    idx.first_norm[id] = Some(idx.words.len());
                }
                idx.words.push(w);
                idx.owner.push(id);
            }
        }
        idx
    }

    pub fn len(&self) -> usize {
        self.words.len()
    }

    pub fn is_empty(&self) -> bool {
        self.words.is_empty()
    }

    pub fn token_of(&self, norm: usize) -> Option<usize> {
        self.owner.get(norm).copied()
    }

    /// Normalised position for a token id; for cue tokens the next spoken word.
    pub fn norm_of_token(&self, token: usize) -> usize {
        if let Some(Some(n)) = self.first_norm.get(token) {
            return *n;
        }
        // first normalised word whose owner token is >= token
        self.owner.partition_point(|&o| o < token).min(self.words.len().saturating_sub(1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_expand_and_map_back() {
        let toks = [(0, "Im"), (1, "Jahr"), (3, "2026"), (4, "gehts")];
        let i = ScriptIndex::build(toks, Lang::En);
        assert_eq!(i.words, ["im", "jahr", "two", "thousand", "twenty", "six", "gehts"]);
        assert_eq!(i.token_of(4), Some(3));
        assert_eq!(i.norm_of_token(3), 2);
        assert_eq!(i.norm_of_token(2), 2); // token 2 is a cue -> next spoken
        assert_eq!(i.norm_of_token(4), 6);
    }
}
