//! Controller (zt-core) behaviour: patch flags, scroll modes, clamping,
//! voice override, wire format, and property-based invariants.

use proptest::prelude::*;
use zt_core::{
    AppState, Command, CoreError, LostPolicy, PlaybackState, ScrollMode, Settings, StatePatch,
    UiMode, VoicePhase, VoiceSignal,
};
use zt_testkit::builders::{apply_all, run_for, SHORT_SCRIPT};
use zt_testkit::ControllerBuilder;

fn sig(phase: VoicePhase, cursor: Option<usize>) -> VoiceSignal {
    VoiceSignal { phase, cursor, confidence: 0.9, rate_wps: 2.0 }
}

fn patch(f: impl FnOnce(&mut StatePatch)) -> StatePatch {
    let mut p = StatePatch::default();
    f(&mut p);
    p
}

// ---------------------------------------------------------------------------
// Patch flags: the adapter only sends flagged sub-trees, so wrong flags = stale UI.
// ---------------------------------------------------------------------------

#[test]
fn patch_flags_table() {
    let rows: Vec<(Command, StatePatch)> = vec![
        (Command::SetWpm(200), patch(|p| p.scroll = true)),
        (Command::NudgeWpm(10), patch(|p| p.scroll = true)),
        (Command::SetMirror { h: true, v: false }, patch(|p| p.output = true)),
        (Command::ToggleMirrorH, patch(|p| p.output = true)),
        (Command::SetFullscreen(true), patch(|p| p.output = true)),
        (Command::SetScrollMode(ScrollMode::Manual), patch(|p| p.scroll = true)),
        (Command::ToggleVoice, patch(|p| {
            p.voice = true;
            p.scroll = true;
        })),
        (Command::SetTheme("daylight".into()), patch(|p| p.settings = true)),
        (Command::SetFontSize(80), patch(|p| p.settings = true)),
        (Command::SeekToken(2), patch(|p| p.scroll = true)),
        (Command::JumpParagraph(1), patch(|p| p.scroll = true)),
        (Command::EnterLive, patch(|p| p.playback = true)),
        (Command::LoadScript("a b".into()), patch(|p| {
            p.script = true;
            p.scroll = true;
        })),
        // voice off -> ignored
        (Command::Voice(sig(VoicePhase::Tracking, Some(1))), StatePatch::default()),
        // stopped -> nothing
        (Command::Tick(16), StatePatch::default()),
    ];
    for (cmd, expected) in rows {
        let mut c = ControllerBuilder::new().build();
        let got = c.apply(cmd.clone()).unwrap();
        assert_eq!(got, expected, "{cmd:?}");
        assert_eq!(got.is_empty(), expected == StatePatch::default());
    }
}

// ---------------------------------------------------------------------------
// Scroll modes x voice phase x lost policy (1 s at 120 wpm = 2 tokens)
// ---------------------------------------------------------------------------

#[test]
fn scroll_mode_table() {
    struct Row {
        name: &'static str,
        voice: bool,
        mode: Option<ScrollMode>,
        policy: LostPolicy,
        signal: Option<VoiceSignal>,
        expected: f64,
    }
    let ease_3 = 3.0 * (1.0 - (-1000.0f64 / 300.0).exp());
    #[rustfmt::skip]
    let rows = [
        Row { name: "manual never moves",            voice: false, mode: Some(ScrollMode::Manual),         policy: LostPolicy::Stop,          signal: None, expected: 0.0 },
        Row { name: "constant: wpm/60 per second",   voice: false, mode: None,                             policy: LostPolicy::Stop,          signal: None, expected: 2.0 },
        Row { name: "voice-activated, listening",    voice: true,  mode: Some(ScrollMode::VoiceActivated), policy: LostPolicy::Stop,          signal: None, expected: 2.0 },
        Row { name: "voice-activated, tracking",     voice: true,  mode: Some(ScrollMode::VoiceActivated), policy: LostPolicy::Stop,          signal: Some(sig(VoicePhase::Tracking, None)), expected: 2.0 },
        Row { name: "voice-activated, hold stops",   voice: true,  mode: Some(ScrollMode::VoiceActivated), policy: LostPolicy::Stop,          signal: Some(sig(VoicePhase::Hold, None)), expected: 0.0 },
        Row { name: "voice-activated, lost stops",   voice: true,  mode: Some(ScrollMode::VoiceActivated), policy: LostPolicy::Stop,          signal: Some(sig(VoicePhase::Lost, None)), expected: 0.0 },
        Row { name: "tracking, lost + Stop policy",  voice: true,  mode: None,                             policy: LostPolicy::Stop,          signal: Some(sig(VoicePhase::Lost, None)), expected: 0.0 },
        Row { name: "tracking, lost + Constant",     voice: true,  mode: None,                             policy: LostPolicy::ConstantSpeed, signal: Some(sig(VoicePhase::Lost, None)), expected: 2.0 },
        Row { name: "tracking eases toward target",  voice: true,  mode: None,                             policy: LostPolicy::Stop,          signal: Some(sig(VoicePhase::Tracking, Some(3))), expected: ease_3 },
    ];
    for r in rows {
        let mut b = ControllerBuilder::new().wpm(120).lost_policy(r.policy);
        if r.voice {
            b = b.voice();
        }
        if let Some(m) = r.mode {
            b = b.mode(m);
        }
        let mut c = b.playing().build();
        if let Some(s) = r.signal {
            c.apply(Command::Voice(s)).unwrap();
        }
        run_for(&mut c, 1000, 16);
        let pos = c.state().scroll.position;
        assert!(
            (pos - r.expected).abs() < 1e-6,
            "{}: position {pos}, expected {}",
            r.name,
            r.expected
        );
    }
}

#[test]
fn voice_ease_is_frame_rate_independent() {
    let pos_with = |frame| {
        let mut c = ControllerBuilder::new().voice().playing().build();
        c.apply(Command::Voice(sig(VoicePhase::Tracking, Some(4)))).unwrap();
        run_for(&mut c, 600, frame);
        c.state().scroll.position
    };
    let (a, b) = (pos_with(8), pos_with(33));
    assert!((a - b).abs() < 1e-9, "{a} vs {b}");
}

#[test]
fn constant_scroll_reaches_the_end() {
    let mut c = ControllerBuilder::new().wpm(120).playing().build();
    run_for(&mut c, 5_000, 16);
    assert_eq!(c.state().playback.state, PlaybackState::Ended);
    assert_eq!(c.state().scroll.position, 5.0);
    // Space restarts from Ended
    c.apply(Command::TogglePlay).unwrap();
    assert_eq!(c.state().playback.state, PlaybackState::Playing);
}

// ---------------------------------------------------------------------------
// Clamping & navigation
// ---------------------------------------------------------------------------

#[test]
fn clamping_table() {
    let t = zt_core::Tokens::embedded().typography.prompter;
    type Check = Box<dyn Fn(&AppState) -> bool>;
    #[rustfmt::skip]
    let rows: Vec<(Command, Check, &str)> = vec![
        (Command::SetWpm(0),          Box::new(|s: &AppState| s.scroll.wpm == Settings::WPM_MIN), "wpm min"),
        (Command::SetWpm(u16::MAX),   Box::new(|s: &AppState| s.scroll.wpm == Settings::WPM_MAX), "wpm max"),
        (Command::NudgeWpm(i16::MIN), Box::new(|s: &AppState| s.scroll.wpm == Settings::WPM_MIN), "nudge down"),
        (Command::NudgeWpm(i16::MAX), Box::new(|s: &AppState| s.scroll.wpm == Settings::WPM_MAX), "nudge up"),
        (Command::SetFontSize(1),     Box::new(move |s: &AppState| s.settings.font_size_px == t.size_min), "font min"),
        (Command::SetFontSize(9999),  Box::new(move |s: &AppState| s.settings.font_size_px == t.size_max), "font max"),
        (Command::SeekToken(999),     Box::new(|s: &AppState| s.scroll.position == 5.0), "seek past end"),
        (Command::JumpParagraph(1),   Box::new(|s: &AppState| s.scroll.position == 4.0), "next paragraph"),
        (Command::JumpParagraph(99),  Box::new(|s: &AppState| s.scroll.position == 4.0), "paragraph clamp"),
        (Command::JumpParagraph(-1),  Box::new(|s: &AppState| s.scroll.position == 0.0), "prev paragraph at start"),
    ];
    for (cmd, check, name) in rows {
        let mut c = ControllerBuilder::new().build();
        c.apply(cmd).unwrap();
        assert!(check(c.state()), "{name}: {:?}", c.state().scroll);
    }
}

#[test]
fn stop_rewinds_and_load_resets_position() {
    let mut c = ControllerBuilder::new().playing().build();
    run_for(&mut c, 1000, 16);
    assert!(c.state().scroll.position > 0.0);
    c.apply(Command::Stop).unwrap();
    assert_eq!(
        (c.state().playback.state, c.state().scroll.position),
        (PlaybackState::Stopped, 0.0)
    );
    let v = c.state().script.version;
    c.apply(Command::SeekToken(3)).unwrap();
    c.apply(Command::LoadScript(SHORT_SCRIPT.into())).unwrap();
    assert_eq!(c.state().script.version, v + 1);
    assert_eq!(c.state().scroll.position, 0.0);
}

#[test]
fn script_lock_follows_playback() {
    let mut c = ControllerBuilder::new().playing().build();
    assert_eq!(c.apply(Command::EditScript("x".into())), Err(CoreError::ScriptLocked));
    c.apply(Command::Pause).unwrap();
    assert!(c.apply(Command::EditScript("x y".into())).is_ok());
    assert_eq!(c.state().script.len(), 2);
}

#[test]
fn countdown_then_playing() {
    let mut c = ControllerBuilder::new().countdown(3).build();
    c.apply(Command::Play).unwrap();
    assert!(matches!(c.state().playback.state, PlaybackState::Countdown { remaining: 3, .. }));
    assert_eq!(c.apply(Command::EditScript("x".into())), Err(CoreError::ScriptLocked));
    run_for(&mut c, 2_990, 10);
    assert!(matches!(c.state().playback.state, PlaybackState::Countdown { remaining: 1, .. }));
    assert_eq!(c.state().scroll.position, 0.0, "no scrolling during the countdown");
    // Play during the countdown is ignored, not an error.
    assert!(c.apply(Command::Play).is_ok());
    assert!(matches!(c.state().playback.state, PlaybackState::Countdown { remaining: 1, .. }));
    run_for(&mut c, 10, 10);
    assert_eq!(c.state().playback.state, PlaybackState::Playing);
    assert_eq!(c.state().scroll.position, 0.0, "end tick has no time left to scroll");
}

#[test]
fn unknown_theme_is_rejected_and_keeps_current() {
    let mut c = ControllerBuilder::new().build();
    let before = c.state().settings.theme.clone();
    assert_eq!(
        c.apply(Command::SetTheme("nope".into())),
        Err(CoreError::UnknownTheme("nope".into()))
    );
    assert_eq!(c.state().settings.theme, before);
}

#[test]
fn replace_settings_is_sanitized() {
    let mut c = ControllerBuilder::new().build();
    let s = Settings {
        wpm: 1,
        theme: "nope".into(),
        focus_line: 5.0,
        countdown_secs: 7,
        ..Default::default()
    };
    c.apply(Command::ReplaceSettings(Box::new(s))).unwrap();
    let st = c.state();
    assert_eq!(st.settings.wpm, Settings::WPM_MIN);
    assert_eq!(st.settings.theme, zt_core::ThemeSet::DEFAULT);
    assert_eq!(st.settings.focus_line, Settings::FOCUS_LINE_MAX);
    assert_eq!(st.playback.countdown_secs, 7);
}

// ---------------------------------------------------------------------------
// Voice override
// ---------------------------------------------------------------------------

#[test]
fn manual_navigation_overrides_voice_for_two_seconds() {
    let mut c = ControllerBuilder::new().voice().playing().build();
    c.apply(Command::Voice(sig(VoicePhase::Tracking, Some(1)))).unwrap();
    c.apply(Command::ResyncHere(4)).unwrap();
    assert_eq!(c.state().voice.phase, VoicePhase::Hold);
    assert_eq!(c.state().voice.override_ms, c.state().settings.manual_override_ms);

    run_for(&mut c, 1_990, 10);
    c.apply(Command::Voice(sig(VoicePhase::Tracking, Some(0)))).unwrap();
    assert_eq!(c.state().scroll.target, 4.0, "ignored during override");

    run_for(&mut c, 10, 10);
    assert_eq!(c.state().voice.override_ms, 0);
    c.apply(Command::Voice(sig(VoicePhase::Tracking, Some(5)))).unwrap();
    assert_eq!(c.state().scroll.target, 5.0, "accepted after override");
}

#[test]
fn toggling_voice_switches_scroll_mode() {
    let mut c = ControllerBuilder::new().build();
    c.apply(Command::ToggleVoice).unwrap();
    assert_eq!(
        (c.state().scroll.mode, c.state().voice.phase),
        (ScrollMode::VoiceTracking, VoicePhase::Listening)
    );
    c.apply(Command::ToggleVoice).unwrap();
    assert_eq!(
        (c.state().scroll.mode, c.state().voice.phase),
        (ScrollMode::Constant, VoicePhase::Idle)
    );
}

// ---------------------------------------------------------------------------
// Wire format (frontend sends `{ type, value }`, see apps/desktop/src/lib/types.ts)
// ---------------------------------------------------------------------------

#[test]
fn command_wire_format() {
    use serde_json::json;
    let rows = [
        (Command::Play, json!({ "type": "play" })),
        (Command::SetWpm(120), json!({ "type": "setWpm", "value": 120 })),
        (Command::SeekToken(0), json!({ "type": "seekToken", "value": 0 })),
        (
            Command::SetMirror { h: true, v: false },
            json!({ "type": "setMirror", "value": { "h": true, "v": false } }),
        ),
        (
            Command::SetScrollMode(ScrollMode::VoiceTracking),
            json!({ "type": "setScrollMode", "value": "voiceTracking" }),
        ),
    ];
    for (cmd, expected) in rows {
        assert_eq!(serde_json::to_value(&cmd).unwrap(), expected, "{cmd:?}");
        assert_eq!(serde_json::from_value::<Command>(expected).unwrap(), cmd);
    }
}

// ---------------------------------------------------------------------------
// Properties
// ---------------------------------------------------------------------------

fn command() -> impl Strategy<Value = Command> {
    let phase = prop::sample::select(vec![
        VoicePhase::Idle,
        VoicePhase::Listening,
        VoicePhase::Tracking,
        VoicePhase::Hold,
        VoicePhase::Lost,
        VoicePhase::Resync,
    ]);
    let mode = prop::sample::select(vec![
        ScrollMode::Manual,
        ScrollMode::Constant,
        ScrollMode::VoiceActivated,
        ScrollMode::VoiceTracking,
    ]);
    // Nested groups keep each `prop_oneof!` at <= 10 arms.
    let simple = prop_oneof![
        Just(Command::EnterLive),
        Just(Command::EnterEdit),
        Just(Command::TogglePlay),
        Just(Command::Play),
        Just(Command::Pause),
        Just(Command::Stop),
        Just(Command::ToggleVoice),
        Just(Command::ToggleMirrorH),
    ];
    prop_oneof![
        simple,
        any::<u16>().prop_map(Command::SetWpm),
        any::<i16>().prop_map(Command::NudgeWpm),
        (-3i32..3).prop_map(Command::JumpParagraph),
        (0usize..20).prop_map(Command::SeekToken),
        (0usize..20).prop_map(Command::ResyncHere),
        mode.prop_map(Command::SetScrollMode),
        (phase, prop::option::of(0usize..10), 0.0f32..=1.0).prop_map(|(phase, cursor, confidence)| {
            Command::Voice(VoiceSignal { phase, cursor, confidence, rate_wps: 2.0 })
        }),
        (0u32..500).prop_map(Command::Tick),
        "[a-z ]{0,30}".prop_map(Command::EditScript),
    ]
}

proptest! {
    // Case count: proptest default (256), override with PROPTEST_CASES (CI: weekly 8192).

    #[test]
    fn controller_invariants(cmds in prop::collection::vec(command(), 0..120)) {
        let mut c = ControllerBuilder::new().build();
        for cmd in cmds {
            let _ = c.apply(cmd); // rejected transitions are fine, panics are not
            let s = c.state();
            prop_assert!((Settings::WPM_MIN..=Settings::WPM_MAX).contains(&s.scroll.wpm));
            prop_assert!(s.scroll.position.is_finite() && s.scroll.position >= 0.0);
            if !s.script.is_empty() {
                prop_assert!(s.scroll.position < s.script.len() as f64, "{:?}", s.scroll);
                prop_assert!(s.current_token() < s.script.len());
            }
            prop_assert!(
                s.playback.mode == UiMode::Live || s.playback.state == PlaybackState::Stopped
            );
            prop_assert!(s.voice.override_ms <= s.settings.manual_override_ms);
            if !s.voice.enabled {
                prop_assert_eq!(s.voice.phase, VoicePhase::Idle);
            }
        }
    }
}

#[test]
fn apply_all_collects_patches() {
    let mut c = ControllerBuilder::new().build();
    let patches = apply_all(&mut c, [Command::EnterLive, Command::SetWpm(100)]);
    assert!(patches[0].playback && patches[1].scroll);
}
