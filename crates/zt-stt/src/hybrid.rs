//! Hybrid orchestrator (ADR-006): streaming engine + optional offline resync.
//!
//! ```text
//! audio --+--> streaming engine --> Partial/Final/Vad   (same thread, low latency)
//!         +--> segmenter --(bounded try_send)--> worker thread: OfflineEngine::transcribe
//!                                           +--(channel)--> SttEvent::Resync
//! ```
//!
//! The streaming path never waits for the offline engine: segments are
//! handed over with `try_send` on a bounded queue; when the worker is busy
//! and the queue is full the segment is dropped (counted in [`ResyncStats`]),
//! because a stale resync is worthless anyway. `reset()` bumps a generation
//! counter so results of segments recorded before a manual seek are discarded.

use crate::{
    Hotword, OfflineEngine, Segment, Segmenter, SpeechEngine, SttError, SttEvent, SAMPLE_RATE,
};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::mpsc::{channel, sync_channel, Receiver, SyncSender, TrySendError};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResyncOptions {
    /// Segments waiting for the worker (beyond the one being decoded).
    pub queue: usize,
    /// Ignore offline results with fewer words (too little context to anchor).
    pub min_words: usize,
}

impl Default for ResyncOptions {
    fn default() -> Self {
        Self { queue: 1, min_words: 3 }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ResyncStats {
    pub segments: u64,
    pub dropped: u64,
    pub decoded: u64,
    pub errors: u64,
    pub stale: u64,
}

struct Job {
    seg: Segment,
    /// Samples fed when the segment was closed (age reference).
    closed_at: u64,
    generation: u64,
}

struct Done {
    text: Result<String, String>,
    end_sample: u64,
    generation: u64,
}

struct Resync {
    name: String,
    segmenter: Box<dyn Segmenter>,
    tx: Option<SyncSender<Job>>,
    rx: Receiver<Done>,
    worker: Option<JoinHandle<()>>,
    in_flight: Arc<AtomicUsize>,
    generation: Arc<AtomicU64>,
    opts: ResyncOptions,
    stats: ResyncStats,
}

pub struct HybridEngine {
    streaming: Box<dyn SpeechEngine>,
    resync: Option<Resync>,
    fed: u64,
    name: String,
    /// Offline engine errors for the operator UI (drained by `take_errors`).
    errors: Vec<String>,
}

impl std::fmt::Debug for HybridEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HybridEngine")
            .field("name", &self.name)
            .field("resync", &self.resync.as_ref().map(|r| (&r.name, r.stats)))
            .finish_non_exhaustive()
    }
}

impl HybridEngine {
    /// Streaming only (resync off).
    pub fn streaming_only(streaming: Box<dyn SpeechEngine>) -> Self {
        let name = streaming.name().to_string();
        Self { streaming, resync: None, fed: 0, name, errors: Vec::new() }
    }

    /// Streaming + offline resync on a background thread.
    pub fn with_resync(
        streaming: Box<dyn SpeechEngine>,
        mut offline: Box<dyn OfflineEngine>,
        segmenter: Box<dyn Segmenter>,
        opts: ResyncOptions,
    ) -> Result<Self, SttError> {
        let off_name = offline.name().to_string();
        let name = format!("{} + {} (resync)", streaming.name(), off_name);
        let (tx, job_rx) = sync_channel::<Job>(opts.queue);
        let (done_tx, rx) = channel::<Done>();
        let in_flight = Arc::new(AtomicUsize::new(0));
        let generation = Arc::new(AtomicU64::new(0));
        let (inf, gen) = (in_flight.clone(), generation.clone());
        let worker = std::thread::Builder::new()
            .name("zt-resync".into())
            .spawn(move || {
                // Ends when the job sender is dropped.
                while let Ok(job) = job_rx.recv() {
                    // Skip segments made obsolete by a reset while queued.
                    let text = if job.generation == gen.load(Ordering::SeqCst) {
                        offline.transcribe(&job.seg.samples).map_err(|e| e.to_string())
                    } else {
                        Ok(String::new())
                    };
                    let done = Done { text, end_sample: job.closed_at, generation: job.generation };
                    let _ = done_tx.send(done);
                    inf.fetch_sub(1, Ordering::SeqCst);
                }
            })
            .map_err(|e| SttError::Engine(format!("cannot spawn resync thread: {e}")))?;
        Ok(Self {
            streaming,
            resync: Some(Resync {
                name: off_name,
                segmenter,
                tx: Some(tx),
                rx,
                worker: Some(worker),
                in_flight,
                generation,
                opts,
                stats: ResyncStats::default(),
            }),
            fed: 0,
            name,
            errors: Vec::new(),
        })
    }

    pub fn resync_enabled(&self) -> bool {
        self.resync.is_some()
    }

    pub fn resync_stats(&self) -> Option<ResyncStats> {
        self.resync.as_ref().map(|r| r.stats)
    }

    pub fn take_errors(&mut self) -> Vec<String> {
        std::mem::take(&mut self.errors)
    }

    /// Closes the running segment so it gets decoded (e.g. at stop).
    pub fn flush_resync(&mut self) {
        if let Some(r) = self.resync.as_mut() {
            r.segmenter.flush();
            r.dispatch(self.fed);
        }
    }

    /// Blocks until the worker has no queued/running job or `timeout` passed
    /// (tests and orderly shutdown only, never on the streaming path).
    pub fn wait_resync_idle(&self, timeout: Duration) -> bool {
        let Some(r) = self.resync.as_ref() else { return true };
        let t0 = Instant::now();
        while r.in_flight.load(Ordering::SeqCst) > 0 {
            if t0.elapsed() >= timeout {
                return false;
            }
            std::thread::sleep(Duration::from_millis(2));
        }
        true
    }
}

impl Resync {
    fn dispatch(&mut self, fed: u64) {
        while let Some(seg) = self.segmenter.pop() {
            self.stats.segments += 1;
            let Some(tx) = self.tx.as_ref() else { return };
            let job =
                Job { seg, closed_at: fed, generation: self.generation.load(Ordering::SeqCst) };
            self.in_flight.fetch_add(1, Ordering::SeqCst);
            match tx.try_send(job) {
                Ok(()) => {}
                Err(TrySendError::Full(_)) | Err(TrySendError::Disconnected(_)) => {
                    self.in_flight.fetch_sub(1, Ordering::SeqCst);
                    self.stats.dropped += 1;
                }
            }
        }
    }
}

impl SpeechEngine for HybridEngine {
    fn name(&self) -> &str {
        &self.name
    }

    fn accept_waveform(&mut self, samples: &[f32]) {
        self.streaming.accept_waveform(samples);
        self.fed += samples.len() as u64;
        if let Some(r) = self.resync.as_mut() {
            r.segmenter.accept(samples);
            r.dispatch(self.fed);
        }
    }

    fn poll(&mut self) -> Vec<SttEvent> {
        let mut out = self.streaming.poll();
        let Some(r) = self.resync.as_mut() else { return out };
        let current = r.generation.load(Ordering::SeqCst);
        // Empty or disconnected both end the drain.
        while let Ok(done) = r.rx.try_recv() {
            if done.generation != current {
                r.stats.stale += 1;
                continue;
            }
            match done.text {
                Ok(text) => {
                    r.stats.decoded += 1;
                    let text = text.trim().to_string();
                    if text.split_whitespace().count() >= r.opts.min_words {
                        let age = self.fed.saturating_sub(done.end_sample);
                        out.push(SttEvent::Resync {
                            text,
                            age_ms: age * 1000 / SAMPLE_RATE as u64,
                        });
                    }
                }
                Err(e) => {
                    r.stats.errors += 1;
                    self.errors.push(e);
                }
            }
        }
        out
    }

    fn set_hotwords(&mut self, hotwords: &[Hotword]) -> Result<(), SttError> {
        // Hotwords bias the streaming engine only; the offline engine already
        // sees the full segment context.
        self.streaming.set_hotwords(hotwords)
    }

    fn reset(&mut self) {
        self.streaming.reset();
        if let Some(r) = self.resync.as_mut() {
            r.generation.fetch_add(1, Ordering::SeqCst);
            r.segmenter.reset();
            while r.rx.try_recv().is_ok() {
                r.stats.stale += 1;
            }
        }
    }
}

impl Drop for HybridEngine {
    fn drop(&mut self) {
        if let Some(r) = self.resync.as_mut() {
            // Dropping the sender ends the worker loop. Do not join: a long
            // decode must not stall shutdown of the voice thread; the worker
            // finishes its current segment and exits on its own.
            r.tx.take();
            drop(r.worker.take());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MockEngine, MockOfflineEngine, RmsSegmenter, ScriptedEvent};

    const SR: usize = SAMPLE_RATE as usize;

    fn tone(ms: usize) -> Vec<f32> {
        (0..SR * ms / 1000).map(|i| if i % 2 == 0 { 0.3 } else { -0.3 }).collect()
    }
    fn silence(ms: usize) -> Vec<f32> {
        vec![0.0; SR * ms / 1000]
    }

    fn streaming() -> Box<dyn SpeechEngine> {
        Box::new(MockEngine::new(vec![ScriptedEvent {
            at_sample: (SR / 2) as u64,
            event: SttEvent::Partial("hallo".into()),
        }]))
    }

    fn hybrid(offline: MockOfflineEngine, queue: usize) -> HybridEngine {
        HybridEngine::with_resync(
            streaming(),
            Box::new(offline),
            Box::new(RmsSegmenter::default()),
            ResyncOptions { queue, ..Default::default() },
        )
        .unwrap()
    }

    fn resyncs(evs: &[SttEvent]) -> Vec<String> {
        evs.iter()
            .filter_map(|e| match e {
                SttEvent::Resync { text, .. } => Some(text.clone()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn streaming_only_passes_events_through() {
        let mut h = HybridEngine::streaming_only(streaming());
        h.accept_waveform(&silence(600));
        assert_eq!(h.poll(), [SttEvent::Partial("hallo".into())]);
        assert!(!h.resync_enabled());
        assert_eq!(h.name(), "mock");
    }

    #[test]
    fn segment_is_decoded_in_background_and_emitted_as_resync() {
        let mut h = hybrid(MockOfflineEngine::new(["eins zwei drei vier"]), 1);
        h.accept_waveform(&silence(300));
        h.accept_waveform(&tone(1500));
        h.accept_waveform(&silence(700)); // closes the segment
        assert!(h.wait_resync_idle(Duration::from_secs(5)));
        h.accept_waveform(&silence(100));
        let evs = h.poll();
        // Streaming partial still arrives (at 0.5 s), plus the resync.
        assert!(evs.contains(&SttEvent::Partial("hallo".into())));
        assert_eq!(resyncs(&evs), ["eins zwei drei vier"]);
        let age = evs.iter().find_map(|e| match e {
            SttEvent::Resync { age_ms, .. } => Some(*age_ms),
            _ => None,
        });
        assert!(age.unwrap() >= 100, "age counts audio fed after the segment end");
        assert_eq!(h.resync_stats().unwrap().decoded, 1);
    }

    #[test]
    fn slow_offline_engine_never_blocks_streaming() {
        let slow = MockOfflineEngine::new(["a b c", "d e f", "g h i"])
            .with_delay(Duration::from_millis(300));
        let mut h = hybrid(slow, 1);
        let t0 = Instant::now();
        for _ in 0..3 {
            h.accept_waveform(&tone(1000));
            h.accept_waveform(&silence(600));
        }
        // 3 segments x 300 ms decode would take >= 900 ms if it were inline.
        assert!(t0.elapsed() < Duration::from_millis(250), "{:?}", t0.elapsed());
        let st = h.resync_stats().unwrap();
        assert_eq!(st.segments, 3);
        // one decoding + one queued, the third is dropped (queue = 1)
        assert!(st.dropped >= 1);
        assert!(h.wait_resync_idle(Duration::from_secs(5)));
    }

    #[test]
    fn reset_discards_results_of_old_segments() {
        let slow = MockOfflineEngine::new(["alt alt alt"]).with_delay(Duration::from_millis(100));
        let mut h = hybrid(slow, 1);
        h.accept_waveform(&tone(1000));
        h.accept_waveform(&silence(600));
        h.reset(); // manual seek while decoding
        assert!(h.wait_resync_idle(Duration::from_secs(5)));
        assert!(resyncs(&h.poll()).is_empty());
        assert_eq!(h.resync_stats().unwrap().stale, 1);
    }

    #[test]
    fn short_or_empty_results_are_ignored() {
        let mut h = hybrid(MockOfflineEngine::new(["ja"]), 1);
        h.accept_waveform(&tone(1000));
        h.flush_resync();
        assert!(h.wait_resync_idle(Duration::from_secs(5)));
        assert!(resyncs(&h.poll()).is_empty(), "below min_words");
    }
}
