//! Property tests for normalisation and word similarity.

use proptest::prelude::*;
use zt_align::normalize::{is_filler, number_words};
use zt_align::similarity::{jaro_winkler, levenshtein, levenshtein_ratio, word_similarity};
use zt_align::{normalize_hypothesis, normalize_word, Lang};

/// Realistic script/STT words: letters incl. German/French diacritics,
/// digits, apostrophes, hyphens and common punctuation.
const WORD: &str = "[A-Za-zÄÖÜäöüßéèàçñ0-9'’.,!?%&€$/-]{1,24}";

fn lang() -> impl Strategy<Value = Lang> {
    prop_oneof![Just(Lang::De), Just(Lang::En)]
}

proptest! {
    // Case count: proptest default (256), override with PROPTEST_CASES (CI: weekly 8192).

    #[test]
    fn never_panics_on_arbitrary_input(s in any::<String>(), l in lang()) {
        let _ = normalize_word(&s, l);
        let _ = normalize_hypothesis(&s, l);
    }

    #[test]
    fn output_words_are_clean(w in WORD, l in lang()) {
        for out in normalize_word(&w, l) {
            prop_assert!(!out.is_empty());
            prop_assert!(!out.chars().any(char::is_whitespace), "{out:?}");
            prop_assert!(!out.chars().any(char::is_uppercase), "{out:?}");
            prop_assert!(!out.chars().any(|c| c.is_ascii_digit()), "digits must be spelled: {out:?}");
            prop_assert!(!out.chars().any(|c| "äöüß'’-.,".contains(c)), "{out:?}");
        }
    }

    #[test]
    fn normalisation_is_idempotent(w in WORD, l in lang()) {
        for out in normalize_word(&w, l) {
            prop_assert_eq!(normalize_word(&out, l), vec![out.clone()]);
        }
    }

    #[test]
    fn case_insensitive(w in WORD, l in lang()) {
        prop_assert_eq!(normalize_word(&w.to_uppercase(), l), normalize_word(&w.to_lowercase(), l));
    }

    #[test]
    fn hypothesis_is_filler_free_and_stable(words in prop::collection::vec(WORD, 0..12), l in lang()) {
        let text = words.join(" ");
        let out = normalize_hypothesis(&text, l);
        prop_assert!(out.iter().all(|w| !is_filler(w)));
        prop_assert_eq!(normalize_hypothesis(&out.join(" "), l), out);
    }

    #[test]
    fn german_numbers_below_a_million_are_one_ascii_word(n in 0u64..1_000_000) {
        let w = number_words(&n.to_string(), Lang::De);
        prop_assert_eq!(w.len(), 1);
        prop_assert!(w[0].chars().all(|c| c.is_ascii_lowercase()), "{:?}", w);
    }

    #[test]
    fn english_numbers_are_ascii_words(n in 0u64..1_000_000) {
        let w = number_words(&n.to_string(), Lang::En);
        prop_assert!(!w.is_empty());
        prop_assert!(w.iter().all(|x| !x.is_empty() && x.chars().all(|c| c.is_ascii_lowercase())), "{:?}", w);
    }

    #[test]
    fn long_digit_strings_are_spelled_digit_by_digit(d in "[0-9]{8,15}", l in lang()) {
        let w = number_words(&d, l);
        if l == Lang::De {
            prop_assert_eq!(w.len(), d.len());
        } else {
            prop_assert!(w.len() >= d.len());
        }
    }

    #[test]
    fn thousands_separator_is_transparent(n in 1_000u64..1_000_000) {
        let de = format!("{}.{:03}", n / 1000, n % 1000);
        let en = format!("{},{:03}", n / 1000, n % 1000);
        prop_assert_eq!(normalize_word(&de, Lang::De), normalize_word(&n.to_string(), Lang::De));
        prop_assert_eq!(normalize_word(&en, Lang::En), normalize_word(&n.to_string(), Lang::En));
    }

    #[test]
    fn punctuation_around_words_is_ignored(w in "[a-zäöüß]{1,12}", l in lang()) {
        let plain = normalize_word(&w, l);
        for wrapped in [format!("{w},"), format!("„{w}“"), format!("({w})"), format!("{w}!"), format!("{w}…")] {
            prop_assert_eq!(normalize_word(&wrapped, l), plain.clone(), "{}", wrapped);
        }
    }

    // ---------------- similarity ----------------

    #[test]
    fn levenshtein_is_a_metric(a in "[a-z]{0,10}", b in "[a-z]{0,10}", c in "[a-z]{0,10}") {
        let (a, b, c): (Vec<char>, Vec<char>, Vec<char>) = (a.chars().collect(), b.chars().collect(), c.chars().collect());
        let ab = levenshtein(&a, &b);
        prop_assert_eq!(ab, levenshtein(&b, &a));
        prop_assert_eq!(levenshtein(&a, &a), 0);
        prop_assert!(ab <= a.len().max(b.len()));
        prop_assert!(ab >= a.len().abs_diff(b.len()));
        prop_assert!(levenshtein(&a, &c) <= ab + levenshtein(&b, &c));
    }

    #[test]
    fn similarities_are_bounded(a in "[a-zäöü]{0,12}", b in "[a-zäöü]{0,12}") {
        for s in [levenshtein_ratio(&a, &b), jaro_winkler(&a, &b), word_similarity(&a, &b)] {
            prop_assert!((0.0..=1.0 + 1e-6).contains(&s), "{s}");
        }
        prop_assert!((levenshtein_ratio(&a, &b) - levenshtein_ratio(&b, &a)).abs() < 1e-6);
        prop_assert_eq!(word_similarity(&a, &a), 1.0);
    }
}

#[test]
fn table_versprecher_similarity() {
    // (spoken by STT, script word, should match at the aligner threshold 0.8)
    let table = [
        ("teleprompta", "teleprompter", true),
        ("mikrofom", "mikrofon", true),
        ("geschwindichkeit", "geschwindigkeit", true),
        ("festplate", "festplatte", true),
        ("und", "uns", false),   // short words are strict
        ("haus", "maus", false), // different initial: no Jaro-Winkler bonus
        ("kanal", "software", false),
    ];
    for (a, b, expected) in table {
        assert_eq!(word_similarity(a, b) >= 0.8, expected, "{a} vs {b}: {}", word_similarity(a, b));
    }
}
