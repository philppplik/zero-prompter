//! zt-audio: microphone capture behind a port.
//!
//! - [`AudioSource`] is the port the app depends on.
//! - [`CpalSource`] (feature `cpal`) is the production adapter.
//! - [`MockSource`] replays samples for tests and fixture runs.
//! - [`Resampler`] converts any input rate / channel count to 16 kHz mono f32,
//!   the format sherpa-onnx expects.
//!
//! The audio callback must never block: frames are handed to a [`FrameSink`]
//! closure; adapters should use a bounded, non-blocking channel behind it.

mod resample;
pub use resample::{downmix_to_mono, Resampler};

#[cfg(feature = "cpal")]
mod cpal_source;
#[cfg(feature = "cpal")]
pub use cpal_source::CpalSource;

/// Target sample rate for STT.
pub const TARGET_RATE: u32 = 16_000;

/// Receives 16 kHz mono f32 frames (range −1..1). Called on the audio thread.
pub type FrameSink = Box<dyn FnMut(&[f32]) + Send + 'static>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceInfo {
    pub id: String,
    pub name: String,
    pub is_default: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum AudioError {
    #[error("no input device available")]
    NoDevice,
    #[error("input device '{0}' not found")]
    DeviceNotFound(String),
    #[error("unsupported sample format: {0}")]
    UnsupportedFormat(String),
    #[error("audio backend error: {0}")]
    Backend(String),
    #[error("source already running")]
    AlreadyRunning,
}

/// Port: a capture source producing 16 kHz mono frames.
pub trait AudioSource {
    fn list_devices(&self) -> Result<Vec<DeviceInfo>, AudioError>;
    /// Starts capturing from `device_id` (None = system default).
    fn start(&mut self, device_id: Option<&str>, sink: FrameSink) -> Result<(), AudioError>;
    fn stop(&mut self);
    fn is_running(&self) -> bool;
    /// Human-readable description of the running stream (device, native
    /// rate/channels/format) for logs and diagnostics.
    fn describe(&self) -> Option<String> {
        None
    }
}

/// Replays a fixed 16 kHz buffer synchronously in `chunk`-sized frames on `start`.
#[derive(Debug, Clone, Default)]
pub struct MockSource {
    pub samples: Vec<f32>,
    pub chunk: usize,
    running: bool,
}

impl MockSource {
    pub fn new(samples: Vec<f32>, chunk: usize) -> Self {
        Self { samples, chunk: chunk.max(1), running: false }
    }
}

impl AudioSource for MockSource {
    fn list_devices(&self) -> Result<Vec<DeviceInfo>, AudioError> {
        Ok(vec![DeviceInfo { id: "mock".into(), name: "Mock input".into(), is_default: true }])
    }

    fn start(&mut self, _device_id: Option<&str>, mut sink: FrameSink) -> Result<(), AudioError> {
        if self.running {
            return Err(AudioError::AlreadyRunning);
        }
        self.running = true;
        for c in self.samples.chunks(self.chunk.max(1)) {
            sink(c);
        }
        Ok(())
    }

    fn stop(&mut self) {
        self.running = false;
    }

    fn is_running(&self) -> bool {
        self.running
    }
}

/// RMS level in dBFS (−inf..0) for the input meter. Returns −100 for silence.
pub fn rms_dbfs(frame: &[f32]) -> f32 {
    if frame.is_empty() {
        return -100.0;
    }
    let mean_sq = frame.iter().map(|s| s * s).sum::<f32>() / frame.len() as f32;
    if mean_sq <= 1e-10 {
        -100.0
    } else {
        10.0 * mean_sq.log10()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn mock_source_delivers_all_samples() {
        let got = Arc::new(Mutex::new(Vec::new()));
        let g = got.clone();
        let mut src = MockSource::new(vec![0.1; 1000], 160);
        src.start(None, Box::new(move |f| g.lock().unwrap().extend_from_slice(f))).unwrap();
        assert_eq!(got.lock().unwrap().len(), 1000);
        assert!(src.start(None, Box::new(|_| {})).is_err());
    }

    #[test]
    fn dbfs() {
        assert_eq!(rms_dbfs(&[]), -100.0);
        assert!((rms_dbfs(&[1.0, -1.0]) - 0.0).abs() < 1e-5);
        assert!((rms_dbfs(&[0.5; 10]) - (-6.0206)).abs() < 1e-3);
    }
}
