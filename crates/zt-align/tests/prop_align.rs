//! Property tests for the windowed Smith-Waterman aligner and the tracker.

use proptest::prelude::*;
use zt_align::{
    align, Lang, QueryWord, ScriptIndex, SwParams, TrackerConfig, TrackerEvent, VoiceTracker,
};

/// Pairwise dissimilar words (distinct initials, Levenshtein ratio < 0.8),
/// so a "match" in these tests can only be an exact word match.
#[rustfmt::skip]
const VOCAB: &[&str] = &[
    "apfel", "brot", "computer", "dach", "elefant", "fenster", "garten", "himmel", "igel", "jacke",
    "kerze", "lampe", "mond", "nudel", "ozean", "pinsel", "quelle", "regen", "sonne", "tiger",
];

fn reference(min: usize, max: usize) -> impl Strategy<Value = Vec<String>> {
    prop::collection::vec(prop::sample::select(VOCAB), min..max)
        .prop_map(|v| v.into_iter().map(String::from).collect())
}

fn query(words: &[String]) -> Vec<QueryWord> {
    words.iter().map(|w| QueryWord { text: w.clone(), weight: 1.0 }).collect()
}

#[test]
fn vocabulary_is_pairwise_dissimilar() {
    let p = SwParams::default();
    for (i, a) in VOCAB.iter().enumerate() {
        for b in &VOCAB[i + 1..] {
            let s = zt_align::similarity::word_similarity(a, b);
            assert!(s < p.sim_threshold, "{a}/{b} = {s}");
        }
    }
}

proptest! {
    // Case count: proptest default (256), override with PROPTEST_CASES (CI: weekly 8192).

    /// A verbatim slice of the script is found exactly where it is
    /// (ties with identical repeats are broken toward `expected`).
    #[test]
    fn exact_slice_is_found(
        r in reference(8, 80),
        start_frac in 0.0f64..1.0,
        len in 2usize..=6,
    ) {
        let start = ((r.len() - len) as f64 * start_frac) as usize;
        let end = start + len - 1;
        let hit = align(&query(&r[start..=end]), &r, 0..r.len(), end, &SwParams::default()).expect("hit");
        prop_assert_eq!(hit.end, end);
        prop_assert_eq!(hit.matches as usize, len);
        prop_assert!((hit.confidence - 1.0).abs() < 1e-5, "{:?}", hit);
    }

    /// Whatever the input, a hit lies inside the window and is well-formed.
    #[test]
    fn hits_respect_window_and_bounds(
        r in reference(1, 60),
        q in reference(0, 8),
        a in 0usize..70,
        b in 0usize..70,
        expected in 0usize..70,
    ) {
        let (lo, hi) = (a.min(b), a.max(b));
        if let Some(h) = align(&query(&q), &r, lo..hi, expected, &SwParams::default()) {
            prop_assert!(h.end >= lo && h.end < hi.min(r.len()), "{:?} not in {}..{}", h, lo, hi);
            prop_assert!((0.0..=1.0).contains(&h.confidence));
            prop_assert!(h.matches >= 1);
            prop_assert!(h.score > 0.0);
        }
    }

    /// One inserted unknown word (ad-lib / STT insertion) in the middle of a
    /// 5-word slice does not change where the alignment ends.
    #[test]
    fn insertion_is_tolerated(r in reference(10, 40), start_frac in 0.0f64..1.0, at in 1usize..4) {
        let len = 5;
        let start = ((r.len() - len) as f64 * start_frac) as usize;
        let end = start + len - 1;
        let mut words: Vec<String> = r[start..=end].to_vec();
        words.insert(at, "xylophon".into());
        let hit = align(&query(&words), &r, 0..r.len(), end, &SwParams::default()).expect("hit");
        prop_assert_eq!(hit.end, end);
        prop_assert!(hit.confidence > TrackerConfig::default().theta, "{:?}", hit);
    }

    /// Random event sequences never panic and outputs stay well-formed.
    #[test]
    fn tracker_outputs_are_well_formed(
        r in reference(5, 40),
        events in prop::collection::vec(event(), 0..60),
    ) {
        let n = r.len();
        let idx = ScriptIndex::build(r.iter().enumerate().map(|(i, w)| (i, w.as_str())), Lang::De);
        let mut tr = VoiceTracker::new(idx, TrackerConfig::default());
        let mut t = 0u64;
        for (ev, dt) in events {
            t += dt;
            let was_stop = ev == TrackerEvent::Stop;
            let o = tr.handle(ev, t);
            prop_assert!(!o.cursor_token.is_some_and(|c| c >= n), "{:?}", o);
            prop_assert!((0.0..=1.0).contains(&o.confidence));
            prop_assert!(o.rate_wps.is_finite() && o.rate_wps >= 0.0);
            if was_stop {
                prop_assert_eq!(o.state, zt_align::VoiceState::Idle);
            }
        }
    }
}

fn event() -> impl Strategy<Value = (TrackerEvent, u64)> {
    let ev = prop_oneof![
        Just(TrackerEvent::Start),
        Just(TrackerEvent::Stop),
        Just(TrackerEvent::VadSpeech),
        Just(TrackerEvent::VadSilence),
        Just(TrackerEvent::Tick),
        (0usize..80).prop_map(|token| TrackerEvent::ManualSeek { token }),
        (reference(1, 7), any::<bool>())
            .prop_map(|(w, is_final)| TrackerEvent::Hypothesis { text: w.join(" "), is_final }),
    ];
    (ev, 0u64..3000)
}
