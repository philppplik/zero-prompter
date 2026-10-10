//! Property and table tests for the script model (tokenisation, paragraphs, cues).

use proptest::prelude::*;
use zt_core::Script;

/// Script-like text: words, umlauts, punctuation, cue brackets, blank lines.
const TEXT: &str = "([A-Za-zÄÖÜäöüß0-9.,!?\\[\\]-]{1,10}[ \n]{1,3}){0,40}";

proptest! {
    // Case count: proptest default (256), override with PROPTEST_CASES (CI: weekly 8192).

    #[test]
    fn never_panics(s in any::<String>()) {
        let sc = Script::parse(s);
        let _ = sc.paragraph_jump(0, 1);
        let _ = sc.estimated_seconds(140);
    }

    #[test]
    fn tokens_map_back_to_source(s in TEXT) {
        let sc = Script::parse(s.as_str());
        for (i, t) in sc.tokens.iter().enumerate() {
            prop_assert_eq!(t.id, i);
            prop_assert!(!t.text.is_empty());
            prop_assert!(!t.text.chars().any(char::is_whitespace));
            prop_assert_eq!(&sc.source[t.range.clone()], t.text.as_str());
        }
        // tokens are exactly the whitespace-separated words
        let words: Vec<&str> = s.split_whitespace().collect();
        let toks: Vec<&str> = sc.tokens.iter().map(|t| t.text.as_str()).collect();
        prop_assert_eq!(toks, words);
    }

    #[test]
    fn paragraphs_partition_tokens(s in TEXT) {
        let sc = Script::parse(s);
        let mut next = 0;
        for (i, p) in sc.paragraphs.iter().enumerate() {
            prop_assert_eq!(p.index, i);
            prop_assert_eq!(p.tokens.start, next);
            prop_assert!(p.tokens.end > p.tokens.start, "no empty paragraphs");
            for t in &sc.tokens[p.tokens.clone()] {
                prop_assert_eq!(t.paragraph, i);
            }
            next = p.tokens.end;
        }
        prop_assert_eq!(next, sc.len());
    }

    #[test]
    fn paragraph_jump_lands_on_a_paragraph_start(s in TEXT, from in 0usize..200, delta in -5i32..5) {
        let sc = Script::parse(s);
        let to = sc.paragraph_jump(from, delta);
        if sc.is_empty() {
            prop_assert_eq!(to, 0);
        } else {
            prop_assert!(sc.paragraphs.iter().any(|p| p.tokens.start == to));
        }
    }

    #[test]
    fn spoken_plus_cues_is_everything(s in TEXT) {
        let sc = Script::parse(s);
        let cues = sc.tokens.iter().filter(|t| t.is_cue).count();
        prop_assert_eq!(sc.word_count() + cues, sc.len());
        prop_assert_eq!(sc.spoken_tokens().count(), sc.word_count());
    }

    #[test]
    fn replace_source_bumps_version(a in TEXT, b in TEXT) {
        let mut sc = Script::parse(a);
        let v = sc.version;
        sc.replace_source(b.clone());
        prop_assert_eq!(sc.version, v + 1);
        prop_assert_eq!(sc.source, b);
    }
}

#[test]
fn cue_table() {
    // (source, is_cue per token)
    let table: &[(&str, &[bool])] = &[
        ("Hallo Welt", &[false, false]),
        ("[PAUSE]", &[true]),
        ("a [Blick in Kamera] b", &[false, true, true, true, false]),
        ("[a] b [c", &[true, false, true]), // unclosed cue runs to the end
        ("x]", &[true]),                    // stray closing bracket marks the word
    ];
    for (src, expected) in table {
        let got: Vec<bool> = Script::parse(*src).tokens.iter().map(|t| t.is_cue).collect();
        assert_eq!(&got, expected, "{src:?}");
    }
}

#[test]
fn estimated_duration_table() {
    let sc = Script::parse("eins zwei drei vier [PAUSE] fünf sechs");
    for (wpm, secs) in [(0u16, 0.0), (60, 6.0), (120, 3.0), (360, 1.0)] {
        assert!((sc.estimated_seconds(wpm) - secs).abs() < 1e-9, "{wpm} wpm");
    }
}
