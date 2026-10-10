//! MockEngine-backed pipeline tests: STT events -> VoiceTracker -> Controller.
//! No threads, no audio device, no models; time is derived from samples fed.

use zt_align::VoiceState;
use zt_core::{Command, ScrollMode, VoicePhase};
use zt_stt::{MockEngine, SpeechEngine};
use zt_testkit::fixtures::{END_S1, END_S4, S1, SCRIPT_DE};
use zt_testkit::pipeline::{phase_of, tracker_event, voice_signal, CHUNK_SAMPLES};
use zt_testkit::{Pipeline, SpeechTimeline};

fn reading(wps: f32) -> Pipeline {
    // Same demo engine the app uses without a model (voice.rs: start 1 s, 2.3 wps).
    Pipeline::new(SCRIPT_DE, MockEngine::reading(SCRIPT_DE, 1.0, wps))
}

#[test]
fn chunk_is_50ms_at_16khz() {
    assert_eq!(CHUNK_SAMPLES, 800);
}

#[test]
fn demo_reading_tracks_whole_script() {
    let mut p = reading(2.3);
    p.run_secs(60.0 / 2.3 + 3.0);
    assert!(p.visited(VoiceState::Tracking));
    assert!(!p.visited(VoiceState::Lost), "never lost while reading the script verbatim");
    let trace = p.cursor_trace();
    assert!(trace.windows(2).all(|w| w[0] <= w[1]), "cursor went backwards: {trace:?}");
    let last = p.cursor().expect("cursor set");
    assert!(last >= END_S4 - 1, "ended at {last}");
}

#[test]
fn controller_follows_the_tracker() {
    let mut p = reading(2.3);
    p.run_secs(8.0);
    let st = p.ctl.state();
    assert!(st.voice.enabled);
    assert_eq!(st.scroll.mode, ScrollMode::VoiceTracking);
    assert_eq!(Some(st.voice.cursor_token), p.cursor());
    assert_eq!(st.voice.phase, phase_of(p.state()));
    assert_eq!(st.scroll.target, st.voice.cursor_token as f64);
    // scroll eases toward the target, never past it while reading forward
    assert!(st.scroll.position > 0.0 && st.scroll.position <= st.scroll.target + 1e-9);
}

#[test]
fn scroll_settles_on_target_after_speech_stops() {
    let mut p = Pipeline::new(SCRIPT_DE, SpeechTimeline::new(3.0).say(S1).build());
    p.run_secs(S1.split_whitespace().count() as f32 / 3.0 + 3.0);
    let st = p.ctl.state();
    assert_eq!(st.voice.cursor_token, END_S1);
    assert!((st.scroll.position - END_S1 as f64).abs() < 0.01, "position {}", st.scroll.position);
    assert_eq!(p.state(), VoiceState::Hold, "VAD off after the utterance -> hold");
}

#[test]
fn ad_lib_in_the_middle_does_not_move_the_text() {
    let engine = SpeechTimeline::new(3.0)
        .say("Willkommen zu meinem Kanal Heute zeige ich euch")
        .pause(0.5)
        .ad_lib("oh sorry kurz vergessen hier etwas")
        .pause(0.5)
        .say("wie man einen Teleprompter")
        .build();
    let mut p = Pipeline::new(SCRIPT_DE, engine);
    p.run_secs(8.0 / 3.0 + 0.5);
    assert_eq!(p.cursor(), Some(7));
    p.run_secs(0.5 + 6.0 / 3.0);
    assert_eq!(p.cursor(), Some(7), "ad-lib must not move the text");
    assert_eq!(p.ctl.state().voice.cursor_token, 7);
    p.run_secs(0.5 + 4.0 / 3.0 + 0.5);
    assert_eq!(p.cursor(), Some(11));
    assert_eq!(p.state(), VoiceState::Hold, "VAD off after the last utterance");
}

#[test]
fn long_pause_holds_and_resumes() {
    let engine = SpeechTimeline::new(2.0)
        .say("Willkommen zu meinem Kanal")
        .pause(20.0)
        .say("Heute zeige ich euch")
        .build();
    let mut p = Pipeline::new(SCRIPT_DE, engine);
    p.run_secs(2.0 + 10.0);
    assert_eq!((p.state(), p.cursor()), (VoiceState::Hold, Some(3)));
    assert_eq!(p.ctl.state().voice.phase, VoicePhase::Hold);
    p.run_secs(10.0 + 2.0 + 0.5);
    assert_eq!(p.cursor(), Some(7));
}

#[test]
fn hotwords_from_script_reach_the_engine() {
    let p = reading(2.3);
    let hw: Vec<&str> = p.engine.hotwords.iter().map(|h| h.text.as_str()).collect();
    assert!(hw.contains(&"teleprompter"), "{hw:?}");
    assert!(hw.contains(&"geschwindigkeit"));
    assert!(!hw.contains(&"heute"), "stop words are excluded");
}

#[test]
fn voice_is_ignored_during_manual_override() {
    let mut p = reading(2.3);
    p.run_secs(4.0);
    p.ctl.apply(Command::SeekToken(40)).unwrap();
    assert_eq!(p.ctl.state().voice.phase, VoicePhase::Hold);
    p.run_secs(1.0); // < manual_override_ms (2 s)
    assert_eq!(p.ctl.state().scroll.target, 40.0, "voice signals ignored during override");
}

#[test]
fn event_mapping_matches_the_app() {
    use zt_align::TrackerEvent as T;
    use zt_stt::SttEvent as S;
    assert_eq!(
        tracker_event(S::Partial("a".into())),
        T::Hypothesis { text: "a".into(), is_final: false }
    );
    assert_eq!(
        tracker_event(S::Final("a".into())),
        T::Hypothesis { text: "a".into(), is_final: true }
    );
    assert_eq!(tracker_event(S::Vad(true)), T::VadSpeech);
    assert_eq!(tracker_event(S::Vad(false)), T::VadSilence);
    for st in [
        VoiceState::Idle,
        VoiceState::Listening,
        VoiceState::Tracking,
        VoiceState::Hold,
        VoiceState::Lost,
        VoiceState::Resync,
    ] {
        // phase token keys match the state names used in tokens.json
        assert_eq!(phase_of(st).token_key(), format!("{st:?}").to_lowercase());
    }
    let out = zt_align::TrackerOutput {
        state: VoiceState::Tracking,
        cursor_token: Some(3),
        confidence: 0.9,
        rate_wps: 2.0,
    };
    let sig = voice_signal(out);
    assert_eq!((sig.phase, sig.cursor), (VoicePhase::Tracking, Some(3)));
}

#[test]
fn mock_engine_reset_drops_pending_events() {
    let mut e = MockEngine::reading("eins zwei drei", 0.0, 1.0);
    e.accept_waveform(&[0.0; 16_000]);
    e.reset();
    assert!(e.poll().is_empty());
    assert_eq!(e.resets, 1);
}
