//! Script model with word tokenisation.
//!
//! A script is split into paragraphs (separated by one or more blank lines)
//! and each paragraph into word tokens (separated by Unicode whitespace).
//! Tokens keep their byte range in the source so the UI can highlight them.
//! Text in square brackets (e.g. `[PAUSE]`, `[Blick in Kamera]`) is a *cue*:
//! it is displayed but never expected to be spoken, so alignment skips it.

use serde::{Deserialize, Serialize};
use std::ops::Range;

/// Index of a token in [`Script::tokens`].
pub type TokenId = usize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Token {
    pub id: TokenId,
    /// Raw text as written (including punctuation).
    pub text: String,
    /// Byte range in [`Script::source`].
    pub range: Range<usize>,
    pub paragraph: usize,
    /// `true` for stage directions in `[...]` that are not spoken.
    pub is_cue: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Paragraph {
    pub index: usize,
    /// Token ids `[start, end)`.
    pub tokens: Range<TokenId>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Script {
    pub source: String,
    pub tokens: Vec<Token>,
    pub paragraphs: Vec<Paragraph>,
    /// Incremented on every edit; lets clients drop stale layout maps.
    pub version: u64,
}

impl Script {
    pub fn parse(source: impl Into<String>) -> Self {
        let source = source.into();
        let mut tokens: Vec<Token> = Vec::new();
        let mut paragraphs: Vec<Paragraph> = Vec::new();

        let mut para_idx = 0usize;
        let mut para_start_tok = 0usize;
        let mut newlines_since_word = 0usize;
        let mut in_cue = false;
        let mut word_start: Option<usize> = None;

        let flush_word = |start: usize,
                          end: usize,
                          tokens: &mut Vec<Token>,
                          para: usize,
                          in_cue: bool| {
            let text = &source[start..end];
            let is_cue = in_cue || text.starts_with('[') || text.ends_with(']');
            tokens.push(Token {
                id: tokens.len(),
                text: text.to_string(),
                range: start..end,
                paragraph: para,
                is_cue,
            });
        };

        for (i, ch) in source.char_indices() {
            if ch.is_whitespace() {
                if let Some(s) = word_start.take() {
                    flush_word(s, i, &mut tokens, para_idx, in_cue);
                    // cue state ends after a word that closes the bracket
                    if tokens.last().map(|t| t.text.ends_with(']')).unwrap_or(false) {
                        in_cue = false;
                    }
                    newlines_since_word = 0;
                }
                if ch == '\n' {
                    newlines_since_word += 1;
                    if newlines_since_word == 2 && tokens.len() > para_start_tok {
                        paragraphs.push(Paragraph {
                            index: para_idx,
                            tokens: para_start_tok..tokens.len(),
                        });
                        para_idx += 1;
                        para_start_tok = tokens.len();
                    }
                }
            } else if word_start.is_none() {
                word_start = Some(i);
                if ch == '[' {
                    in_cue = true;
                }
            }
        }
        if let Some(s) = word_start.take() {
            flush_word(s, source.len(), &mut tokens, para_idx, in_cue);
        }
        if tokens.len() > para_start_tok {
            paragraphs.push(Paragraph { index: para_idx, tokens: para_start_tok..tokens.len() });
        }
        Script { source, tokens, paragraphs, version: 0 }
    }

    /// Replaces the text, bumping the version.
    pub fn replace_source(&mut self, source: impl Into<String>) {
        let v = self.version + 1;
        *self = Script::parse(source);
        self.version = v;
    }

    pub fn len(&self) -> usize {
        self.tokens.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }

    /// Tokens that are expected to be spoken (no cues), as `(id, text)`.
    pub fn spoken_tokens(&self) -> impl Iterator<Item = (TokenId, &str)> {
        self.tokens.iter().filter(|t| !t.is_cue).map(|t| (t.id, t.text.as_str()))
    }

    pub fn paragraph_of(&self, token: TokenId) -> Option<usize> {
        self.tokens.get(token).map(|t| t.paragraph)
    }

    /// First token of paragraph `current + delta` (clamped).
    pub fn paragraph_jump(&self, from: TokenId, delta: i32) -> TokenId {
        if self.paragraphs.is_empty() {
            return 0;
        }
        let cur = self.paragraph_of(from).unwrap_or(0) as i64;
        let target = (cur + delta as i64).clamp(0, self.paragraphs.len() as i64 - 1) as usize;
        self.paragraphs[target].tokens.start
    }

    pub fn word_count(&self) -> usize {
        self.tokens.iter().filter(|t| !t.is_cue).count()
    }

    /// Estimated reading duration in seconds at `wpm`.
    pub fn estimated_seconds(&self, wpm: u16) -> f64 {
        if wpm == 0 {
            return 0.0;
        }
        self.word_count() as f64 * 60.0 / wpm as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenizes_words_and_paragraphs() {
        let s = Script::parse("Hallo Welt.\nZweite Zeile\n\nNeuer  Absatz");
        let words: Vec<_> = s.tokens.iter().map(|t| t.text.as_str()).collect();
        assert_eq!(words, ["Hallo", "Welt.", "Zweite", "Zeile", "Neuer", "Absatz"]);
        assert_eq!(s.paragraphs.len(), 2);
        assert_eq!(s.paragraphs[1].tokens, 4..6);
        assert_eq!(&s.source[s.tokens[1].range.clone()], "Welt.");
    }

    #[test]
    fn umlauts_keep_byte_ranges() {
        let s = Script::parse("Grüße aus Köln");
        assert_eq!(&s.source[s.tokens[0].range.clone()], "Grüße");
        assert_eq!(&s.source[s.tokens[2].range.clone()], "Köln");
    }

    #[test]
    fn cues_are_marked() {
        let s = Script::parse("Hallo [Blick in Kamera] Welt [PAUSE]");
        let cues: Vec<bool> = s.tokens.iter().map(|t| t.is_cue).collect();
        assert_eq!(cues, [false, true, true, true, false, true]);
        assert_eq!(s.word_count(), 2);
    }

    #[test]
    fn paragraph_jump_clamps() {
        let s = Script::parse("a b\n\nc d\n\ne");
        assert_eq!(s.paragraph_jump(0, 1), 2);
        assert_eq!(s.paragraph_jump(0, 10), 4);
        assert_eq!(s.paragraph_jump(4, -10), 0);
    }

    #[test]
    fn empty_script() {
        let s = Script::parse("   \n\n ");
        assert!(s.is_empty());
        assert_eq!(s.paragraph_jump(0, 1), 0);
    }
}
