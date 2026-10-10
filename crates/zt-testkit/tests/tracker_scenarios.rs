//! Voice state machine (zt-align `VoiceTracker`): transition table +
//! alignment scenario fixtures + a few step-by-step checks.
//!
//! Moved here from `crates/zt-align/tests/tracker_scenarios.rs` so that the
//! simulated speaker and fixtures are shared instead of duplicated. Scenario
//! categories (ad-lib, backtrack, forward jump) follow EdNutting/autocue
//! (MIT); texts and code are our own.

use zt_align::{TrackerConfig, TrackerEvent, VoiceState};
use zt_testkit::fixtures::{Category, SCENARIOS, SCRIPT_DE};
use zt_testkit::Speaker;

// ---------------------------------------------------------------------------
// Fixture-driven scenarios
// ---------------------------------------------------------------------------

#[test]
fn alignment_fixtures() {
    for sc in SCENARIOS {
        sc.run();
    }
}

#[test]
fn fixtures_cover_every_category() {
    use Category::*;
    for cat in [Normal, Versprecher, Zurueckspringen, Ueberspringen, Stille, AdLib, Manual, Lost] {
        assert!(SCENARIOS.iter().any(|s| s.category == cat), "no fixture for {cat:?}");
    }
}

#[test]
fn reference_script_has_one_norm_word_per_token() {
    let idx = zt_testkit::speaker::index_of(SCRIPT_DE, zt_align::Lang::De);
    assert_eq!(idx.len(), zt_testkit::fixtures::SCRIPT_DE_WORDS);
    assert!(idx.owner.iter().enumerate().all(|(i, &o)| i == o));
}

// ---------------------------------------------------------------------------
// Transition table (CONCEPT.md §6.3, table in tracker.rs)
// ---------------------------------------------------------------------------

/// Brings a fresh (idle) speaker into the named state.
fn setup(s: &mut Speaker, from: VoiceState) {
    use VoiceState::*;
    if from == Idle {
        return;
    }
    s.ev(TrackerEvent::Start, 0);
    match from {
        Listening => {}
        Tracking => {
            s.say("Willkommen zu meinem Kanal");
        }
        Hold => {
            s.say("Willkommen zu meinem Kanal");
            s.silence(100);
        }
        Lost => {
            s.say("Willkommen zu meinem Kanal");
            for _ in 0..5 {
                s.say("bla blub quux zzz");
            }
        }
        Resync => {
            s.say("Willkommen zu meinem Kanal Heute zeige ich euch");
            s.partial("Danach", 250);
            s.partial("Danach öffnen", 250);
            s.partial("Danach öffnen wir", 250); // 2nd consistent candidate -> resync
        }
        Idle => unreachable!(),
    }
    assert_eq!(s.state(), from, "setup for {from:?} failed: {:?}", s.out());
}

fn hyp(text: &str) -> TrackerEvent {
    TrackerEvent::Hypothesis { text: text.into(), is_final: false }
}

#[test]
fn transition_table() {
    use TrackerEvent as E;
    use VoiceState::*;
    #[rustfmt::skip]
    let table: Vec<(VoiceState, TrackerEvent, u64, VoiceState, &str)> = vec![
        // from       event                                 dt      to          why
        (Idle,      E::Start,                               0,      Listening, "start"),
        (Idle,      hyp("Willkommen zu meinem"),            0,      Idle,      "idle ignores speech"),
        (Idle,      E::VadSpeech,                           0,      Idle,      "idle ignores VAD"),
        (Idle,      E::ManualSeek { token: 5 },             0,      Idle,      "idle ignores seeks"),
        (Idle,      E::Tick,                                10_000, Idle,      "idle ignores time"),
        (Idle,      E::Stop,                                0,      Idle,      "stop is idempotent"),
        (Listening, hyp("Willkommen"),                      250,    Listening, "one word is not enough evidence"),
        (Listening, hyp("Willkommen zu meinem"),            250,    Tracking,  "match >= theta"),
        (Listening, E::VadSilence,                          0,      Listening, "only tracking falls to hold"),
        (Listening, E::Start,                               0,      Listening, "start while running is a no-op"),
        (Listening, E::ManualSeek { token: 5 },             0,      Hold,      "manual seek -> hold"),
        (Listening, E::Stop,                                0,      Idle,      "stop"),
        (Tracking,  hyp("Heute zeige"),                     250,    Tracking,  "match continues"),
        (Tracking,  E::VadSilence,                          0,      Hold,      "silence -> hold"),
        (Tracking,  hyp("Zum Schluss stellen wir"),         250,    Hold,      "1st far-jump candidate -> hold"),
        (Tracking,  hyp("oh sorry kurz vergessen"),         250,    Hold,      "ad-lib -> hold"),
        (Tracking,  E::Tick,                                5_000,  Lost,      "speech (no VAD silence) without match > timeout"),
        (Tracking,  E::Start,                               0,      Tracking,  "start while running is a no-op"),
        (Tracking,  E::ManualSeek { token: 20 },            0,      Hold,      "manual seek -> hold"),
        (Tracking,  E::Stop,                                0,      Idle,      "stop"),
        (Hold,      hyp("Heute zeige"),                     250,    Tracking,  "match near cursor"),
        (Hold,      E::Tick,                                60_000, Hold,      "silence is never lost"),
        (Hold,      E::VadSpeech,                           0,      Hold,      "speech alone changes nothing"),
        (Hold,      E::Stop,                                0,      Idle,      "stop"),
        (Lost,      hyp("Heute zeige ich euch"),            250,    Lost,      "lost needs 2 confirmations"),
        (Lost,      E::VadSilence,                          0,      Lost,      "silence keeps lost"),
        (Lost,      E::ManualSeek { token: 10 },            0,      Hold,      "operator resync"),
        (Lost,      E::Stop,                                0,      Idle,      "stop"),
        (Resync,    E::Tick,                                100,    Resync,    "still easing"),
        (Resync,    E::Tick,                                300,    Tracking,  "after ease -> tracking"),
        (Resync,    E::VadSilence,                          0,      Resync,    "silence does not abort the ease"),
        (Resync,    E::ManualSeek { token: 3 },             0,      Hold,      "manual seek -> hold"),
        (Resync,    E::Stop,                                0,      Idle,      "stop"),
    ];
    let mut failures = Vec::new();
    for (i, (from, ev, dt, to, why)) in table.into_iter().enumerate() {
        let mut s = Speaker::idle(SCRIPT_DE, TrackerConfig::default());
        setup(&mut s, from);
        let desc = format!("#{i} {from:?} --{ev:?}/+{dt}ms--> {to:?} ({why})");
        let got = s.ev(ev, dt);
        if got.state != to {
            failures.push(format!("{desc}: got {:?}", got.state));
        }
    }
    assert!(failures.is_empty(), "transition table failures:\n{}", failures.join("\n"));
}

#[test]
fn manual_seek_sets_cursor_and_clamps() {
    let mut s = Speaker::default_de();
    s.say("Willkommen zu meinem Kanal");
    assert_eq!(s.seek(29).cursor_token, Some(29));
    // beyond the end -> last word
    assert_eq!(s.seek(10_000).cursor_token, Some(59));
}

#[test]
fn stop_resets_and_restart_listens_again() {
    let mut s = Speaker::default_de();
    s.say("Willkommen zu meinem Kanal");
    s.ev(TrackerEvent::Stop, 0);
    s.say("Heute zeige ich euch");
    assert_eq!(s.state(), VoiceState::Idle);
    s.ev(TrackerEvent::Start, 0);
    assert_eq!(s.state(), VoiceState::Listening);
}

// ---------------------------------------------------------------------------
// Step-by-step checks that the fixture format cannot express
// ---------------------------------------------------------------------------

#[test]
fn normal_reading_step_by_step() {
    let mut s = Speaker::default_de();
    assert_eq!(s.state(), VoiceState::Listening);
    let outs = s.say("Willkommen zu meinem Kanal Heute zeige ich euch");
    assert_eq!(outs[0].state, VoiceState::Listening, "one word is not enough evidence");
    assert_eq!((outs[1].state, outs[1].cursor_token), (VoiceState::Tracking, Some(1)));
    // cursor never moves backwards while reading normally
    let cursors: Vec<usize> = outs.iter().filter_map(|o| o.cursor_token).collect();
    assert!(cursors.windows(2).all(|w| w[0] <= w[1]), "{cursors:?}");
    s.assert_at(VoiceState::Tracking, Some(7));
    assert!(s.out().rate_wps > 0.0);
    assert!((0.0..=1.0).contains(&s.out().confidence));
}

#[test]
fn speaking_rate_is_estimated() {
    // 250 ms per word = 4 words/s
    let mut s = Speaker::default_de();
    s.say("Willkommen zu meinem Kanal Heute zeige ich euch wie man einen Teleprompter");
    let r = s.out().rate_wps;
    assert!((3.0..=5.0).contains(&r), "rate {r}");
}

#[test]
fn forward_jump_needs_two_confirmations() {
    let mut s = Speaker::default_de();
    s.say("Willkommen zu meinem Kanal Heute zeige ich euch");
    let outs = s.say("Danach öffnen wir die Software");
    assert_eq!((outs[1].state, outs[1].cursor_token), (VoiceState::Hold, Some(7)), "1st candidate");
    assert_eq!((outs[2].state, outs[2].cursor_token), (VoiceState::Resync, Some(31)), "confirmed");
    let o = s.tick(300);
    assert_eq!((o.state, o.cursor_token), (VoiceState::Tracking, Some(33)));
}

#[test]
fn inconsistent_jump_candidates_do_not_confirm() {
    // Two far candidates in different places must not add up to a jump.
    let mut s = Speaker::default_de();
    s.say("Willkommen zu meinem Kanal");
    s.partial("Zum Schluss stellen", 250); // candidate near 47
    let o = s.partial("Danach öffnen wir", 250); // candidate near 31 -> new pending, count 1
    assert_ne!(o.state, VoiceState::Resync, "{o:?}");
    assert_eq!(o.cursor_token, Some(3));
}

#[test]
fn set_config_keeps_cursor() {
    let mut s = Speaker::default_de();
    s.say("Willkommen zu meinem Kanal");
    s.tracker.set_config(TrackerConfig { lost_timeout_ms: 100, ..TrackerConfig::default() });
    s.assert_at(VoiceState::Tracking, Some(3));
}
