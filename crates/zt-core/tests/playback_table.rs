//! Table tests for the Playback x Mode state machine (CONCEPT.md §6.2) plus
//! property tests for its invariants.

use proptest::prelude::*;
use zt_core::{Playback, PlaybackEvent, PlaybackState, UiMode};

use PlaybackEvent as E;
use PlaybackState as S;
use UiMode::{Edit, Live};

const CD: S = S::Countdown { remaining: 3, ms_left: 1000 };

fn pb(mode: UiMode, state: S) -> Playback {
    Playback { mode, state, countdown_secs: 3 }
}

/// `None` = transition must be rejected (state unchanged).
type Row = (UiMode, S, E, Option<(UiMode, S)>);

#[rustfmt::skip]
const TABLE: &[Row] = &[
    // ---- Edit (only reachable state: Stopped)
    (Edit, S::Stopped, E::EnterLive,  Some((Live, S::Stopped))),
    (Edit, S::Stopped, E::EnterEdit,  Some((Edit, S::Stopped))),
    (Edit, S::Stopped, E::Toggle,     Some((Live, CD))),          // zero friction: Space starts
    (Edit, S::Stopped, E::Play,       Some((Live, CD))),
    (Edit, S::Stopped, E::Pause,      None),
    (Edit, S::Stopped, E::Stop,       None),
    (Edit, S::Stopped, E::ReachedEnd, None),
    // ---- Live / Stopped
    (Live, S::Stopped, E::Toggle,     Some((Live, CD))),
    (Live, S::Stopped, E::Play,       Some((Live, CD))),
    (Live, S::Stopped, E::Pause,      None),
    (Live, S::Stopped, E::Stop,       Some((Live, S::Stopped))),
    (Live, S::Stopped, E::EnterEdit,  Some((Edit, S::Stopped))),
    (Live, S::Stopped, E::EnterLive,  Some((Live, S::Stopped))),
    (Live, S::Stopped, E::ReachedEnd, None),
    // ---- Live / Countdown (text locked)
    (Live, CD,         E::Toggle,     Some((Live, S::Stopped))),
    (Live, CD,         E::Pause,      Some((Live, S::Stopped))),
    (Live, CD,         E::Stop,       Some((Live, S::Stopped))),
    (Live, CD,         E::Play,       Some((Live, CD))),          // ignored: countdown keeps running
    (Live, CD,         E::EnterEdit,  None),
    (Live, CD,         E::EnterLive,  Some((Live, CD))),
    (Live, CD,         E::ReachedEnd, None),
    // ---- Live / Playing (text locked)
    (Live, S::Playing, E::Toggle,     Some((Live, S::Paused))),
    (Live, S::Playing, E::Pause,      Some((Live, S::Paused))),
    (Live, S::Playing, E::Play,       Some((Live, S::Playing))),
    (Live, S::Playing, E::Stop,       Some((Live, S::Stopped))),
    (Live, S::Playing, E::ReachedEnd, Some((Live, S::Ended))),
    (Live, S::Playing, E::EnterEdit,  None),
    (Live, S::Playing, E::EnterLive,  Some((Live, S::Playing))),
    // ---- Live / Paused
    (Live, S::Paused,  E::Toggle,     Some((Live, S::Playing))),  // resume: no countdown
    (Live, S::Paused,  E::Play,       Some((Live, S::Playing))),
    (Live, S::Paused,  E::Pause,      Some((Live, S::Paused))),
    (Live, S::Paused,  E::Stop,       Some((Live, S::Stopped))),
    (Live, S::Paused,  E::EnterEdit,  Some((Edit, S::Stopped))),
    (Live, S::Paused,  E::EnterLive,  Some((Live, S::Paused))),
    (Live, S::Paused,  E::ReachedEnd, None),
    // ---- Live / Ended
    (Live, S::Ended,   E::Toggle,     Some((Live, CD))),          // restart
    (Live, S::Ended,   E::Play,       Some((Live, CD))),
    (Live, S::Ended,   E::Pause,      None),
    (Live, S::Ended,   E::Stop,       Some((Live, S::Stopped))),
    (Live, S::Ended,   E::EnterEdit,  Some((Edit, S::Stopped))),
    (Live, S::Ended,   E::EnterLive,  Some((Live, S::Ended))),
    (Live, S::Ended,   E::ReachedEnd, None),
];

#[test]
fn transition_table() {
    let mut failures = Vec::new();
    for (i, &(mode, state, ev, expected)) in TABLE.iter().enumerate() {
        let mut p = pb(mode, state);
        let res = p.handle(ev);
        let got = res.map(|_| (p.mode, p.state)).ok();
        if got != expected {
            failures.push(format!(
                "#{i}: ({mode:?}, {state:?}) --{ev:?}--> expected {expected:?}, got {got:?}"
            ));
        }
        if expected.is_none() {
            assert_eq!(
                (p.mode, p.state),
                (mode, state),
                "#{i}: rejected event must not change state"
            );
        }
    }
    assert!(failures.is_empty(), "playback table failures:\n{}", failures.join("\n"));
}

#[test]
fn table_covers_every_live_state_and_event() {
    let events = [E::EnterLive, E::EnterEdit, E::Toggle, E::Play, E::Pause, E::Stop, E::ReachedEnd];
    for st in [S::Stopped, CD, S::Playing, S::Paused, S::Ended] {
        for ev in events {
            assert!(
                TABLE.iter().any(|r| r.0 == Live && r.1 == st && r.2 == ev),
                "missing row (Live, {st:?}, {ev:?})"
            );
        }
    }
}

#[test]
fn zero_countdown_starts_immediately() {
    let mut p = Playback { countdown_secs: 0, ..Default::default() };
    p.handle(E::Toggle).unwrap();
    assert_eq!((p.mode, p.state), (Live, S::Playing));
}

#[rustfmt::skip]
#[test]
fn countdown_tick_table() {
    // (remaining, ms_left, dt) -> (state after, started?)
    let table: &[(u8, u32, u32, S, bool)] = &[
        (3, 1000, 0,        S::Countdown { remaining: 3, ms_left: 1000 }, false),
        (3, 1000, 1,        S::Countdown { remaining: 3, ms_left: 999 },  false),
        (3, 1000, 1000,     S::Countdown { remaining: 2, ms_left: 1000 }, false),
        (3, 1000, 1500,     S::Countdown { remaining: 2, ms_left: 500 },  false),
        (1, 1000, 999,      S::Countdown { remaining: 1, ms_left: 1 },    false),
        (1, 1000, 1000,     S::Playing,                                   true),
        (3, 1000, 3000,     S::Playing,                                   true),
        (3, 1000, u32::MAX, S::Playing,                                   true),
    ];
    for &(remaining, ms_left, dt, expected, started) in table {
        let mut p = pb(Live, S::Countdown { remaining, ms_left });
        assert_eq!(p.tick(dt), started, "({remaining}, {ms_left}) +{dt}");
        assert_eq!(p.state, expected, "({remaining}, {ms_left}) +{dt}");
    }
}

#[rustfmt::skip]
#[test]
fn countdown_overflow_table() {
    // (remaining, ms_left, dt) -> overflow ms after the countdown ended
    let table: &[(u8, u32, u32, Option<u32>)] = &[
        (3, 1000, 2999,     None),
        (3, 1000, 3000,     Some(0)),
        (3, 1000, 3016,     Some(16)),
        (1, 10,   16,       Some(6)),
        (2, 500,  1600,     Some(100)),
        (3, 1000, u32::MAX, Some(u32::MAX - 3000)),
    ];
    for &(remaining, ms_left, dt, expected) in table {
        let mut p = pb(Live, S::Countdown { remaining, ms_left });
        assert_eq!(p.tick_overflow(dt), expected, "({remaining}, {ms_left}) +{dt}");
    }
    for st in [S::Stopped, S::Playing, S::Paused, S::Ended] {
        assert_eq!(pb(Live, st).tick_overflow(100), None);
    }
}

#[test]
fn tick_outside_countdown_is_a_no_op() {
    for st in [S::Stopped, S::Playing, S::Paused, S::Ended] {
        let mut p = pb(Live, st);
        assert!(!p.tick(5000));
        assert_eq!(p.state, st);
    }
}

#[derive(Debug, Clone, Copy)]
enum Input {
    Ev(E),
    Tick(u32),
}

fn input() -> impl Strategy<Value = Input> {
    prop_oneof![
        prop::sample::select(vec![
            E::EnterLive,
            E::EnterEdit,
            E::Toggle,
            E::Play,
            E::Pause,
            E::Stop,
            E::ReachedEnd,
        ])
        .prop_map(Input::Ev),
        (0u32..2500).prop_map(Input::Tick),
    ]
}

proptest! {
    // Case count: proptest default (256), override with PROPTEST_CASES (CI: weekly 8192).

    #[test]
    fn invariants_hold_for_any_input_sequence(countdown in 0u8..=5, inputs in prop::collection::vec(input(), 0..80)) {
        let mut p = Playback { countdown_secs: countdown, ..Default::default() };
        for i in inputs {
            let before = p;
            match i {
                Input::Ev(e) => {
                    if p.handle(e).is_err() {
                        prop_assert_eq!(p, before, "rejected event changed state");
                    }
                }
                Input::Tick(dt) => {
                    p.tick(dt);
                }
            }
            // Edit mode is only ever Stopped.
            prop_assert!(p.mode == Live || p.state == S::Stopped, "{:?}", p);
            // Locked text implies Live mode.
            prop_assert!(!p.text_locked() || p.mode == Live, "{:?}", p);
            // A countdown always has a second left and at most 1000 ms in it.
            if let S::Countdown { remaining, ms_left } = p.state {
                prop_assert!((1..=countdown.max(1)).contains(&remaining), "{:?}", p);
                prop_assert!((1..=1000).contains(&ms_left), "{:?}", p);
            }
            // Stop always works in Live.
            let mut q = p;
            if q.mode == Live {
                prop_assert!(q.handle(E::Stop).is_ok());
                prop_assert_eq!(q.state, S::Stopped);
            }
        }
    }
}
