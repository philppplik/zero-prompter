//! Diagnostics: structured logging (tracing -> rotating file in the app log
//! dir), an in-memory snapshot of the voice pipeline for the "Diagnose" tab,
//! and the single error path `report_error` (log + ring buffer + toast event).
//!
//! Log file: `<app_log_dir>/zero-teleprompter.YYYY-MM-DD.log`, daily rotation,
//! 7 files kept. Filter via env `ZT_LOG` (EnvFilter syntax), default
//! `info,zero_teleprompter_lib=debug`.

use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use zt_core::metrics::{LatencySummary, LatencyWindow};

use crate::voice::EngineInfo;

pub const EV_ERROR: &str = "diag://error";
const LOG_PREFIX: &str = "zero-teleprompter";
const MAX_ERRORS: usize = 50;
const DEFAULT_FILTER: &str = "info,zero_teleprompter_lib=debug,zt=debug";

static GUARD: OnceLock<tracing_appender::non_blocking::WorkerGuard> = OnceLock::new();
static STATE: OnceLock<Mutex<DiagState>> = OnceLock::new();

// ---------------------------------------------------------------- state

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ErrorEntry {
    /// Unix time in ms.
    pub at_ms: u64,
    /// Pipeline stage, e.g. "audio", "stt", "models", "voice-thread".
    pub source: String,
    pub message: String,
}

/// Bounded list of recent errors (newest last).
#[derive(Debug, Default)]
pub struct ErrorRing {
    items: VecDeque<ErrorEntry>,
}

impl ErrorRing {
    pub fn push(&mut self, e: ErrorEntry) {
        if self.items.len() == MAX_ERRORS {
            self.items.pop_front();
        }
        self.items.push_back(e);
    }
    pub fn to_vec(&self) -> Vec<ErrorEntry> {
        self.items.iter().cloned().collect()
    }
}

#[derive(Debug, Default)]
pub struct Latencies {
    /// Capture time of the audio chunk -> partial hypothesis emitted (processing:
    /// queue + resample + decode; excludes the model's internal look-ahead).
    pub audio_to_partial: LatencyWindow,
    /// First voiced chunk of an utterance (RMS onset) -> first partial
    /// (includes the model's chunk/look-ahead latency).
    pub onset_to_partial: LatencyWindow,
    /// Hypothesis received -> tracker cursor moved (alignment cost).
    pub hyp_to_cursor: LatencyWindow,
    /// Capture time of the chunk -> `voice://word` emitted (end to end in Rust).
    pub end_to_end: LatencyWindow,
}

#[derive(Debug, Default)]
pub struct DiagState {
    pub engine: Option<EngineInfo>,
    pub audio: Option<String>,
    pub level_db: f32,
    pub frames_per_s: f64,
    pub tick_hz: f64,
    pub voice_state: String,
    pub confidence: f32,
    pub last_partial: Option<String>,
    pub running: bool,
    pub tracking: bool,
    pub lat: Latencies,
    pub errors: ErrorRing,
    pub log_dir: Option<PathBuf>,
}

pub fn state() -> MutexGuard<'static, DiagState> {
    STATE
        .get_or_init(|| Mutex::new(DiagState { level_db: -100.0, ..Default::default() }))
        .lock()
        .unwrap_or_else(|e| e.into_inner())
}

pub fn now_unix_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ErrorEvent<'a> {
    source: &'a str,
    message: &'a str,
}

/// The one error path: structured log + ring buffer + `diag://error` (toast).
/// Never fail silently: every fallback the pipeline takes goes through here.
pub fn report_error(app: &AppHandle, source: &str, message: impl Into<String>) {
    let message = message.into();
    tracing::error!(target: "zt::error", source, %message, "pipeline error");
    state().errors.push(ErrorEntry {
        at_ms: now_unix_ms(),
        source: source.to_string(),
        message: message.clone(),
    });
    let _ = app.emit(EV_ERROR, ErrorEvent { source, message: &message });
}

/// A degraded-but-working condition (fallback taken): logged as warning and
/// listed in the diagnostics panel. The UI shows it via the engine notes.
pub fn record_warning(source: &str, message: impl Into<String>) {
    let message = message.into();
    tracing::warn!(target: "zt::warn", source, %message, "pipeline warning");
    state().errors.push(ErrorEntry {
        at_ms: now_unix_ms(),
        source: format!("{source} (Hinweis)"),
        message,
    });
}

// ---------------------------------------------------------------- logging

/// Installs the global subscriber (file + stderr) and a panic hook that logs
/// panics of every thread (voice, resync, tick) instead of losing them.
pub fn init_logging(app: &AppHandle) {
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;
    use tracing_subscriber::EnvFilter;

    let dir = app.path().app_log_dir().ok();
    let filter =
        EnvFilter::try_from_env("ZT_LOG").unwrap_or_else(|_| EnvFilter::new(DEFAULT_FILTER));
    let stderr = tracing_subscriber::fmt::layer().with_writer(std::io::stderr).with_ansi(false);
    let file_layer = dir.as_ref().and_then(|d| {
        std::fs::create_dir_all(d).ok()?;
        let appender = tracing_appender::rolling::Builder::new()
            .rotation(tracing_appender::rolling::Rotation::DAILY)
            .filename_prefix(LOG_PREFIX)
            .filename_suffix("log")
            .max_log_files(7)
            .build(d)
            .ok()?;
        let (writer, guard) = tracing_appender::non_blocking(appender);
        let _ = GUARD.set(guard);
        let layer = tracing_subscriber::fmt::layer().with_writer(writer);
        Some(layer.with_ansi(false).with_thread_names(true))
    });
    let _ = tracing_subscriber::registry().with(filter).with(stderr).with(file_layer).try_init();
    state().log_dir = dir.clone();

    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let thread = std::thread::current().name().unwrap_or("?").to_string();
        tracing::error!(target: "zt::panic", thread, %info, "panic");
        // try_lock: the panicking thread may hold the diagnostics lock.
        if let Some(Ok(mut s)) = STATE.get().map(|m| m.try_lock()) {
            s.errors.push(ErrorEntry {
                at_ms: now_unix_ms(),
                source: format!("panic ({thread})"),
                message: info.to_string(),
            });
        }
        prev(info);
    }));

    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        os = std::env::consts::OS,
        arch = std::env::consts::ARCH,
        cpal = cfg!(feature = "cpal"),
        sherpa = cfg!(feature = "sherpa"),
        download = cfg!(feature = "download"),
        log_dir = ?dir,
        "zero-teleprompter starting"
    );
}

// ---------------------------------------------------------------- snapshot

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LatencyReport {
    pub audio_to_partial: LatencySummary,
    pub onset_to_partial: LatencySummary,
    pub hyp_to_cursor: LatencySummary,
    pub end_to_end: LatencySummary,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub version: &'static str,
    pub os: &'static str,
    pub arch: &'static str,
    pub sherpa: bool,
    pub cpal: bool,
    pub running: bool,
    pub tracking: bool,
    pub engine: Option<EngineInfo>,
    pub audio: Option<String>,
    pub level_db: f32,
    pub frames_per_s: f64,
    pub tick_hz: f64,
    pub voice_state: String,
    pub confidence: f32,
    pub last_partial: Option<String>,
    pub latency: LatencyReport,
    pub errors: Vec<ErrorEntry>,
    pub log_dir: Option<String>,
}

pub fn snapshot() -> Snapshot {
    let s = state();
    Snapshot {
        version: env!("CARGO_PKG_VERSION"),
        os: std::env::consts::OS,
        arch: std::env::consts::ARCH,
        sherpa: cfg!(feature = "sherpa"),
        cpal: cfg!(feature = "cpal"),
        running: s.running,
        tracking: s.tracking,
        engine: s.engine.clone(),
        audio: s.audio.clone(),
        level_db: s.level_db,
        frames_per_s: s.frames_per_s,
        tick_hz: s.tick_hz,
        voice_state: s.voice_state.clone(),
        confidence: s.confidence,
        last_partial: s.last_partial.clone(),
        latency: LatencyReport {
            audio_to_partial: s.lat.audio_to_partial.summary(),
            onset_to_partial: s.lat.onset_to_partial.summary(),
            hyp_to_cursor: s.lat.hyp_to_cursor.summary(),
            end_to_end: s.lat.end_to_end.summary(),
        },
        errors: s.errors.to_vec(),
        log_dir: s.log_dir.as_ref().map(|p| p.display().to_string()),
    }
}

pub fn reset_latencies() {
    state().lat = Latencies::default();
}

/// Log files in `dir`, oldest first.
pub fn log_files(dir: &Path) -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = std::fs::read_dir(dir)
        .map(|rd| {
            rd.filter_map(|e| e.ok().map(|e| e.path()))
                .filter(|p| {
                    p.file_name()
                        .and_then(|n| n.to_str())
                        .is_some_and(|n| n.starts_with(LOG_PREFIX) && n.ends_with(".log"))
                })
                .collect()
        })
        .unwrap_or_default();
    v.sort();
    v
}

/// Concatenates all log files (plus a JSON snapshot header) into `dest`.
pub fn export_logs(dest: &Path) -> Result<u64, String> {
    let dir = state().log_dir.clone().ok_or("kein Log-Verzeichnis")?;
    let mut out = Vec::new();
    let snap = serde_json::to_string_pretty(&snapshot()).map_err(|e| e.to_string())?;
    out.extend_from_slice(b"# zero-teleprompter diagnostics snapshot\n");
    out.extend_from_slice(snap.as_bytes());
    out.extend_from_slice(b"\n\n");
    for f in log_files(&dir) {
        out.extend_from_slice(format!("# ---- {}\n", f.display()).as_bytes());
        out.extend_from_slice(&std::fs::read(&f).map_err(|e| format!("{}: {e}", f.display()))?);
        out.push(b'\n');
    }
    std::fs::write(dest, &out).map_err(|e| format!("{}: {e}", dest.display()))?;
    tracing::info!(dest = %dest.display(), bytes = out.len(), "log exported");
    Ok(out.len() as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_ring_is_bounded() {
        let mut r = ErrorRing::default();
        for i in 0..(MAX_ERRORS + 5) {
            r.push(ErrorEntry { at_ms: i as u64, source: "t".into(), message: format!("{i}") });
        }
        let v = r.to_vec();
        assert_eq!(v.len(), MAX_ERRORS);
        assert_eq!(v[0].message, "5");
        assert_eq!(v.last().unwrap().message, format!("{}", MAX_ERRORS + 4));
    }

    #[test]
    fn log_files_filters_and_sorts() {
        let d = std::env::temp_dir().join(format!("zt-diag-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        let files =
            ["zero-teleprompter.2026-10-02.log", "zero-teleprompter.2026-10-01.log", "x.txt"];
        for n in files {
            std::fs::write(d.join(n), "x").unwrap();
        }
        let names: Vec<String> = log_files(&d)
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
            .collect();
        assert_eq!(names, ["zero-teleprompter.2026-10-01.log", "zero-teleprompter.2026-10-02.log"]);
        let _ = std::fs::remove_dir_all(&d);
    }
}
