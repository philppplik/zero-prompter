//! cpal adapter for [`AudioSource`].
//!
//! Targets cpal 0.15.x (workspace pins "0.15"). cpal 0.16+/0.18 changed the API
//! (`build_input_stream` takes `StreamConfig` by value, `name()` replaced by
//! `description()`/`id()`); do not bump without porting. Not compiled here (no toolchain
//! on the authoring machine). Points to check when first building:
//!  - `build_input_stream` takes a 4th argument `Option<Duration>` (0.15+).
//!  - `DeviceTrait::name()` may be deprecated in favour of a description API
//!    in newer cpal versions.
//!  - `cpal::Stream` is `!Send` on some platforms; `CpalSource` must then
//!    stay on the thread that created it (the app keeps it on a dedicated
//!    audio-control thread).

use crate::{
    AudioError, AudioSource, ChannelLevels, ChannelMode, DeviceInfo, FrameSink, Resampler,
};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

pub struct CpalSource {
    host: cpal::Host,
    stream: Option<cpal::Stream>,
    desc: Option<String>,
    /// Channel mode, read by the data callback on every block.
    mode: Arc<AtomicUsize>,
    /// Per-channel levels, written by the data callback (try_lock, never blocks).
    levels: Arc<Mutex<Option<ChannelLevels>>>,
}

fn encode_mode(m: ChannelMode) -> usize {
    match m {
        ChannelMode::Auto => 0,
        ChannelMode::Mix => 1,
        ChannelMode::Channel(i) => i.saturating_add(2),
    }
}

fn decode_mode(v: usize) -> ChannelMode {
    match v {
        0 => ChannelMode::Auto,
        1 => ChannelMode::Mix,
        n => ChannelMode::Channel(n - 2),
    }
}

impl std::fmt::Debug for CpalSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CpalSource").field("running", &self.stream.is_some()).finish()
    }
}

impl Default for CpalSource {
    fn default() -> Self {
        Self {
            host: cpal::default_host(),
            stream: None,
            desc: None,
            mode: Arc::new(AtomicUsize::new(0)),
            levels: Arc::new(Mutex::new(None)),
        }
    }
}

fn be(e: impl std::fmt::Display) -> AudioError {
    AudioError::Backend(e.to_string())
}

impl CpalSource {
    pub fn new() -> Self {
        Self::default()
    }

    fn find_device(&self, id: Option<&str>) -> Result<cpal::Device, AudioError> {
        match id {
            None => self.host.default_input_device().ok_or(AudioError::NoDevice),
            Some(id) => self
                .host
                .input_devices()
                .map_err(be)?
                .find(|d| d.name().map(|n| n == id).unwrap_or(false))
                .ok_or_else(|| AudioError::DeviceNotFound(id.to_string())),
        }
    }
}

impl AudioSource for CpalSource {
    fn list_devices(&self) -> Result<Vec<DeviceInfo>, AudioError> {
        let default_name = self.host.default_input_device().and_then(|d| d.name().ok());
        let devices = self.host.input_devices().map_err(be)?;
        Ok(devices
            .filter_map(|d| d.name().ok())
            .map(|name| DeviceInfo {
                is_default: default_name.as_deref() == Some(name.as_str()),
                id: name.clone(), // cpal has no stable id; the name is used
                name,
            })
            .collect())
    }

    fn start(&mut self, device_id: Option<&str>, sink: FrameSink) -> Result<(), AudioError> {
        if self.stream.is_some() {
            return Err(AudioError::AlreadyRunning);
        }
        let device = self.find_device(device_id)?;
        let supported = device.default_input_config().map_err(be)?;
        let format = supported.sample_format();
        let config: cpal::StreamConfig = supported.into();
        let channels = config.channels as usize;
        let rate = config.sample_rate.0;

        // Shared between the (single) data callback and nothing else; Mutex
        // only to satisfy `Send` for the boxed FnMut. Uncontended -> cheap.
        let state = Arc::new(Mutex::new((
            Resampler::to_16k(rate, channels),
            sink,
            Vec::<f32>::new(),
            Vec::<f32>::new(),
        )));
        let err_fn = |e: cpal::StreamError| eprintln!("zt-audio: stream error: {e}");

        macro_rules! build {
            ($t:ty, $conv:expr) => {{
                let st = state.clone();
                let mode = self.mode.clone();
                let levels = self.levels.clone();
                device.build_input_stream(
                    &config,
                    move |data: &[$t], _: &cpal::InputCallbackInfo| {
                        if let Ok(mut g) = st.try_lock() {
                            let (rs, sink, tmp, out) = &mut *g;
                            tmp.clear();
                            tmp.extend(data.iter().map($conv));
                            rs.selector.mode = decode_mode(mode.load(Ordering::Relaxed));
                            out.clear();
                            rs.process(tmp.as_slice(), out);
                            if let Ok(mut l) = levels.try_lock() {
                                *l = Some(ChannelLevels {
                                    dbfs: rs.selector.channel_dbfs(),
                                    active: rs.selector.active_channel(),
                                });
                            }
                            if !out.is_empty() {
                                sink(out.as_slice());
                            }
                        }
                    },
                    err_fn,
                    None, // timeout: Option<Duration> (0.15)
                )
            }};
        }

        let stream = match format {
            cpal::SampleFormat::F32 => build!(f32, |s: &f32| *s),
            cpal::SampleFormat::I16 => build!(i16, |s: &i16| *s as f32 / 32768.0),
            cpal::SampleFormat::U16 => build!(u16, |s: &u16| (*s as f32 - 32768.0) / 32768.0),
            other => return Err(AudioError::UnsupportedFormat(format!("{other:?}"))),
        }
        .map_err(be)?;
        stream.play().map_err(be)?;
        self.stream = Some(stream);
        let name = device.name().unwrap_or_else(|_| "?".into());
        self.desc = Some(format!("{name} · {rate} Hz · {channels} ch · {format:?} -> 16 kHz mono"));
        Ok(())
    }

    fn stop(&mut self) {
        // Dropping the stream stops capture.
        self.stream = None;
        self.desc = None;
        if let Ok(mut l) = self.levels.lock() {
            *l = None;
        }
    }

    fn describe(&self) -> Option<String> {
        self.desc.clone()
    }

    fn is_running(&self) -> bool {
        self.stream.is_some()
    }

    fn set_channel_mode(&mut self, mode: ChannelMode) {
        self.mode.store(encode_mode(mode), Ordering::Relaxed);
    }

    fn channel_levels(&self) -> Option<ChannelLevels> {
        self.levels.lock().ok()?.clone()
    }
}
