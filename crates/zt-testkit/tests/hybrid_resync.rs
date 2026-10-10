//! End-to-end (mock) test of the hybrid STT path (ADR-006):
//! audio -> HybridEngine (silent streaming mock + offline mock on a worker
//! thread) -> SttEvent::Resync -> VoiceTracker re-anchors the cursor.

use std::time::Duration;
use zt_align::{Lang, ScriptIndex, TrackerConfig, TrackerEvent, VoiceState, VoiceTracker};
use zt_stt::{
    HybridEngine, MockEngine, MockOfflineEngine, ResyncOptions, RmsSegmenter, SpeechEngine,
    SttEvent, SAMPLE_RATE,
};
use zt_testkit::pipeline::tracker_event;

const SYL: [&str; 8] = ["ka", "lo", "mi", "ru", "te", "so", "na", "pe"];
const CHUNK: usize = SAMPLE_RATE as usize / 20; // 50 ms

fn words() -> Vec<String> {
    (0..200).map(|i| format!("{}{}{}", SYL[i % 8], SYL[(i / 8) % 8], SYL[(i / 64) % 8])).collect()
}

fn chunk(loud: bool) -> Vec<f32> {
    (0..CHUNK)
        .map(|i| {
            if loud && i % 2 == 0 {
                0.3
            } else if loud {
                -0.3
            } else {
                0.0
            }
        })
        .collect()
}

#[test]
fn offline_resync_reanchors_a_stuck_streaming_tracker() {
    let w = words();
    let index = ScriptIndex::build(w.iter().enumerate().map(|(i, s)| (i, s.as_str())), Lang::De);
    let mut tracker = VoiceTracker::new(index, TrackerConfig::default());
    tracker.handle(TrackerEvent::Start, 0);

    // Streaming recognizer hears nothing useful; offline decodes words 120..132.
    let mut engine = HybridEngine::with_resync(
        Box::new(MockEngine::new(vec![])),
        Box::new(MockOfflineEngine::new([w[120..132].join(" ")])),
        Box::new(RmsSegmenter::default()),
        ResyncOptions::default(),
    )
    .unwrap();

    let mut t = 0u64;
    let mut seen_resync = false;
    let mut feed = |engine: &mut HybridEngine, tracker: &mut VoiceTracker, t: u64| {
        for ev in engine.poll() {
            seen_resync |= matches!(ev, SttEvent::Resync { .. });
            tracker.handle(tracker_event(ev), t);
        }
        tracker.handle(TrackerEvent::Tick, t);
    };
    let pattern = [(false, 20), (true, 40), (false, 20)]; // 1 s / 2 s / 1 s
    for (loud, n) in pattern {
        for _ in 0..n {
            engine.accept_waveform(&chunk(loud));
            t += 50;
            feed(&mut engine, &mut tracker, t);
        }
    }
    assert!(engine.wait_resync_idle(Duration::from_secs(5)));
    t += 50;
    feed(&mut engine, &mut tracker, t);

    assert!(seen_resync);
    assert_eq!(tracker.output().cursor_token, Some(131));
    assert_eq!(tracker.state(), VoiceState::Resync);
    t += 400;
    tracker.handle(TrackerEvent::Tick, t);
    assert_eq!(tracker.state(), VoiceState::Tracking);
}
