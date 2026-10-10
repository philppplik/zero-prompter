//! Offline (non-streaming) recognition port, used for resync (ADR-006).

use crate::SttError;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// Transcribes one complete speech segment (16 kHz mono f32).
///
/// Called only from the resync worker thread, so implementations may block
/// for as long as decoding takes.
pub trait OfflineEngine: Send {
    fn name(&self) -> &str;
    fn transcribe(&mut self, samples: &[f32]) -> Result<String, SttError>;
}

impl<E: OfflineEngine + ?Sized> OfflineEngine for Box<E> {
    fn name(&self) -> &str {
        (**self).name()
    }
    fn transcribe(&mut self, samples: &[f32]) -> Result<String, SttError> {
        (**self).transcribe(samples)
    }
}

/// Deterministic offline engine: returns the queued texts in order (empty
/// string once exhausted), optionally after an artificial decode delay.
#[derive(Debug, Clone, Default)]
pub struct MockOfflineEngine {
    texts: VecDeque<String>,
    delay: Duration,
    calls: Arc<AtomicUsize>,
}

impl MockOfflineEngine {
    pub fn new<I, S>(texts: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self { texts: texts.into_iter().map(Into::into).collect(), ..Default::default() }
    }

    /// Simulated decode time per segment (to prove streaming is not blocked).
    pub fn with_delay(mut self, delay: Duration) -> Self {
        self.delay = delay;
        self
    }

    /// Shared counter of `transcribe` calls (readable after the engine moved
    /// into the worker thread).
    pub fn calls(&self) -> Arc<AtomicUsize> {
        self.calls.clone()
    }
}

impl OfflineEngine for MockOfflineEngine {
    fn name(&self) -> &str {
        "mock-offline"
    }

    fn transcribe(&mut self, samples: &[f32]) -> Result<String, SttError> {
        if samples.is_empty() {
            return Err(SttError::Engine("empty segment".into()));
        }
        if !self.delay.is_zero() {
            std::thread::sleep(self.delay);
        }
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(self.texts.pop_front().unwrap_or_default())
    }
}
