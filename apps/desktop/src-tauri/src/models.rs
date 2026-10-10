//! Model manager adapter (zt-models): manifest, status, install via HTTP,
//! remove, per-language model choice and engine construction (ADR-006).
//!
//! Manifest lookup order: `<app_data>/models/manifest.json` (user/updated) ->
//! bundled `resources/models.json` (4 entries: Kroko DE, Kroko EN,
//! Zipformer EN 20M as Apache-2.0 fallback, Parakeet TDT v3 for resync).
//!
//! Engine choice: per primary language the user's pick (if installed) else
//! the manifest's `defaultRank` order; if constructing an engine fails the
//! next installed candidate is tried (`zt_stt::build_with_fallback`).
//! Resync (tier L, optional) is added on top when enabled and installed.

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
#[cfg(feature = "download")]
use tauri::Emitter;
use tauri::{AppHandle, Manager};
use zt_models::{
    primary_language, Manifest, ModelEntry, ModelRole, ModelStatus, ModelStore, ModelType,
};

#[cfg(feature = "download")]
pub const EV_PROGRESS: &str = "models://progress";
const BUNDLED_MANIFEST: &str = include_str!("../resources/models.json");
/// Computed hashes of files the manifest could not verify (per model dir).
const UNVERIFIED_FILE: &str = "unverified-sha256.json";
const PREFS_FILE: &str = "voice-models.json";
/// Optional Silero VAD model for resync segmentation (else RMS segmentation).
pub const SILERO_VAD_FILE: &str = "silero_vad.onnx";

pub fn models_root(app: &AppHandle) -> Result<PathBuf, String> {
    app.path().app_data_dir().map(|d| d.join("models")).map_err(|e| e.to_string())
}

pub fn manifest(app: &AppHandle) -> Result<Manifest, String> {
    let user = models_root(app)?.join("manifest.json");
    let text = std::fs::read_to_string(&user).unwrap_or_else(|_| BUNDLED_MANIFEST.to_string());
    // `Manifest::parse` validates (https-only, safe paths, sha256 format);
    // missing hashes are warnings, not errors (ADR-006).
    let m = Manifest::parse(&text).map_err(|e| e.to_string())?;
    for w in m.warnings() {
        eprintln!("zt: manifest: {w}");
    }
    Ok(m)
}

// ---------------------------------------------------------------- preferences

/// User choice of models, persisted in `<config>/voice-models.json`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct VoiceModelPrefs {
    /// Primary language ("de", "en") -> preferred streaming model id.
    /// Absent = automatic (manifest default + fallback chain).
    pub streaming: BTreeMap<String, String>,
    /// Offline resync on/off (off by default: tier L, ~670 MB, extra CPU).
    pub resync: bool,
    /// Preferred resync model (absent = manifest default).
    pub resync_model: Option<String>,
}

fn prefs_path(app: &AppHandle) -> Option<PathBuf> {
    app.path().app_config_dir().ok().map(|d| d.join(PREFS_FILE))
}

pub fn load_prefs(app: &AppHandle) -> VoiceModelPrefs {
    prefs_path(app)
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|t| serde_json::from_str(t.trim_start_matches('\u{feff}')).ok())
        .unwrap_or_default()
}

pub fn save_prefs(app: &AppHandle, prefs: &VoiceModelPrefs) -> Result<(), String> {
    let path = prefs_path(app).ok_or("no config dir")?;
    let json = serde_json::to_vec_pretty(prefs).map_err(|e| e.to_string())?;
    crate::persist::write_atomic(&path, &json).map_err(|e| e.to_string())
}

// ---------------------------------------------------------------- listing

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelRow {
    pub entry: ModelEntry,
    pub status: &'static str,
    pub missing: Vec<String>,
    pub total_size: u64,
    pub requires_attribution: bool,
    pub model_type: Option<ModelType>,
    pub roles: Vec<ModelRole>,
    /// Files without manifest hash ("unverified"), with the SHA-256 computed
    /// after download if known: `(path, Some(sha))`.
    pub unverified: Vec<(String, Option<String>)>,
}

fn read_computed_hashes(store: &ModelStore, e: &ModelEntry) -> BTreeMap<String, String> {
    std::fs::read_to_string(store.model_dir(e).join(UNVERIFIED_FILE))
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default()
}

fn is_installed(store: &ModelStore, e: &ModelEntry) -> bool {
    store.quick_status(e) == ModelStatus::Installed
}

pub fn list(app: &AppHandle) -> Result<Vec<ModelRow>, String> {
    let m = manifest(app)?;
    let store = ModelStore::new(models_root(app)?);
    Ok(m.models
        .into_iter()
        .map(|e| {
            let (status, missing) = match store.quick_status(&e) {
                ModelStatus::Installed => ("installed", vec![]),
                ModelStatus::NotInstalled => ("notInstalled", vec![]),
                ModelStatus::Incomplete(v) => ("incomplete", v),
            };
            let computed = read_computed_hashes(&store, &e);
            let unverified = e
                .unverified_files()
                .map(|f| (f.path.clone(), computed.get(&f.path).cloned()))
                .collect();
            ModelRow {
                total_size: e.total_size(),
                requires_attribution: e.requires_attribution(),
                model_type: e.model_type(),
                roles: e.effective_roles(),
                unverified,
                entry: e,
                status,
                missing,
            }
        })
        .collect())
}

pub fn remove(app: &AppHandle, id: &str) -> Result<(), String> {
    let m = manifest(app)?;
    let entry = m.get(id).map_err(|e| e.to_string())?;
    let dir = ModelStore::new(models_root(app)?).model_dir(entry);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).map_err(|e| e.to_string())?;
    }
    Ok(())
}

// ---------------------------------------------------------------- selection

/// What would be used for a language (shown in the Inspector).
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Selection {
    pub language: String,
    /// Installed streaming candidates in fallback order (first = used).
    pub streaming_chain: Vec<String>,
    /// All streaming candidates for the language (installed or not).
    pub streaming_options: Vec<String>,
    /// Resync model that would run (None = off or not installed).
    pub resync: Option<String>,
    pub resync_options: Vec<String>,
    pub silero_vad: bool,
}

pub fn selection(
    app: &AppHandle,
    lang: &str,
    prefs: &VoiceModelPrefs,
) -> Result<Selection, String> {
    let m = manifest(app)?;
    let root = models_root(app)?;
    let store = ModelStore::new(&root);
    let short = primary_language(lang);
    let pref = prefs.streaming.get(&short).map(String::as_str);
    let ids = |v: Vec<&ModelEntry>| v.into_iter().map(|e| e.id.clone()).collect::<Vec<_>>();
    let streaming_chain = ids(m
        .fallback_chain(&short, ModelRole::Streaming, pref)
        .into_iter()
        .filter(|e| is_installed(&store, e))
        .collect());
    let resync = prefs
        .resync
        .then(|| {
            m.select(&short, ModelRole::Resync, prefs.resync_model.as_deref(), |e| {
                is_installed(&store, e)
            })
        })
        .flatten()
        .map(|e| e.id.clone());
    Ok(Selection {
        language: short.clone(),
        streaming_chain,
        streaming_options: ids(m.candidates(&short, ModelRole::Streaming)),
        resync,
        resync_options: ids(m.candidates(&short, ModelRole::Resync)),
        silero_vad: root.join(SILERO_VAD_FILE).is_file(),
    })
}

// ---------------------------------------------------------------- install

#[cfg(feature = "download")]
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct Progress {
    id: String,
    done: u64,
    total: u64,
    finished: bool,
    error: Option<String>,
    /// `(path, computed sha256)` of files installed without manifest hash.
    unverified: Vec<(String, String)>,
}

#[cfg(feature = "download")]
pub fn install(app: &AppHandle, id: &str) -> Result<(), String> {
    let m = manifest(app)?;
    let entry = m.get(id).map_err(|e| e.to_string())?.clone();
    let store = ModelStore::new(models_root(app)?);
    let app = app.clone();
    let id = id.to_string();
    std::thread::Builder::new()
        .name("zt-model-dl".into())
        .spawn(move || {
            let mut last = std::time::Instant::now();
            let mut dl = http::UreqDownloader;
            let res = zt_models::install(&store, &entry, &mut dl, &mut |done, total| {
                if last.elapsed().as_millis() >= 100 {
                    last = std::time::Instant::now();
                    let _ = app.emit(
                        EV_PROGRESS,
                        Progress {
                            id: id.clone(),
                            done,
                            total,
                            finished: false,
                            error: None,
                            unverified: vec![],
                        },
                    );
                }
            });
            let total = entry.total_size();
            let (error, unverified) = match res {
                Ok(report) => {
                    if !report.unverified.is_empty() {
                        let map: BTreeMap<_, _> = report.unverified.iter().cloned().collect();
                        if let Ok(json) = serde_json::to_vec_pretty(&map) {
                            let _ = crate::persist::write_atomic(
                                &store.model_dir(&entry).join(UNVERIFIED_FILE),
                                &json,
                            );
                        }
                        for (p, sha) in &report.unverified {
                            eprintln!("zt: {}/{p} installed UNVERIFIED, sha256 {sha}", entry.id);
                        }
                    }
                    (None, report.unverified)
                }
                Err(e) => (Some(e.to_string()), vec![]),
            };
            let _ = app.emit(
                EV_PROGRESS,
                Progress { id, done: total, total, finished: true, error, unverified },
            );
        })
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(not(feature = "download"))]
pub fn install(_app: &AppHandle, _id: &str) -> Result<(), String> {
    Err("this build has no download support (feature `download`)".into())
}

#[cfg(feature = "download")]
mod http {
    use std::fs::OpenOptions;
    use std::io::{Read, Write};
    use std::path::Path;
    use zt_models::{Downloader, ModelError, Progress};

    /// HTTP downloader with resume via `Range` (ureq 2.x). // VERIFY: ureq 2 API
    #[derive(Debug)]
    pub struct UreqDownloader;

    fn io(path: &Path) -> impl FnOnce(std::io::Error) -> ModelError + '_ {
        move |source| ModelError::Io { path: path.to_path_buf(), source }
    }

    impl Downloader for UreqDownloader {
        fn download(
            &mut self,
            url: &str,
            dest: &Path,
            expected: u64,
            progress: Progress<'_>,
        ) -> Result<(), ModelError> {
            if !url.starts_with("https://") {
                return Err(ModelError::Download(format!("refusing non-HTTPS url {url}")));
            }
            let existing = std::fs::metadata(dest).map(|m| m.len()).unwrap_or(0);
            if expected > 0 && existing == expected {
                progress(existing, expected);
                return Ok(());
            }
            let mut req = ureq::get(url);
            if existing > 0 && existing < expected {
                req = req.set("Range", &format!("bytes={existing}-"));
            }
            let resp = req.call().map_err(|e| ModelError::Download(e.to_string()))?;
            let append = existing > 0 && existing < expected && resp.status() == 206;
            let mut file = OpenOptions::new()
                .create(true)
                .write(true)
                .append(append)
                .truncate(!append)
                .open(dest)
                .map_err(io(dest))?;
            let mut done = if append { existing } else { 0 };
            let mut reader = resp.into_reader();
            let mut buf = vec![0u8; 64 * 1024];
            loop {
                let n = reader.read(&mut buf).map_err(io(dest))?;
                if n == 0 {
                    break;
                }
                file.write_all(&buf[..n]).map_err(io(dest))?;
                done += n as u64;
                if expected > 0 && done > expected {
                    return Err(ModelError::Download("server sent more data than expected".into()));
                }
                progress(done, expected);
            }
            file.flush().map_err(io(dest))?;
            Ok(())
        }
    }
}

// ---------------------------------------------------------------- engines

/// Engine plus notes for the operator UI.
#[cfg_attr(not(feature = "sherpa"), allow(dead_code))]
pub struct BuiltEngine {
    pub engine: Box<dyn zt_stt::SpeechEngine>,
    pub resync: bool,
    /// e.g. "zipformer-en-kroko-2025-08-06 failed: …; using zipformer-en-20m-2023-02-17".
    pub notes: Vec<String>,
}

/// Builds the hybrid engine for `lang`: streaming via fallback chain, plus
/// resync if enabled and installed. `Ok(None)` = no streaming model installed
/// (caller uses demo mode). Resync problems never disable streaming.
#[cfg(feature = "sherpa")]
pub fn build_engine(
    app: &AppHandle,
    lang: &str,
    prefs: &VoiceModelPrefs,
) -> Result<Option<BuiltEngine>, String> {
    use zt_stt::{
        build_with_fallback, HybridEngine, OfflineEngine, ResyncOptions, RmsSegmenter, Segmenter,
        SherpaConfig, SherpaEngine, SherpaOfflineConfig, SherpaOfflineEngine, SileroSegmenter,
        SileroVadConfig, SpeechEngine, SttError,
    };

    let m = manifest(app)?;
    let root = models_root(app)?;
    let store = ModelStore::new(&root);
    let short = primary_language(lang);
    let pref = prefs.streaming.get(&short).map(String::as_str);
    let chain: Vec<&ModelEntry> = m
        .fallback_chain(&short, ModelRole::Streaming, pref)
        .into_iter()
        .filter(|e| is_installed(&store, e) && e.model_type() == Some(ModelType::OnlineTransducer))
        .collect();
    if chain.is_empty() {
        return Ok(None);
    }

    // Real file names come from the manifest roles (releases use e.g.
    // `encoder-epoch-99-avg-1.int8.onnx`), not the defaults.
    let path_of = |e: &ModelEntry, role: &str, slot: &mut PathBuf| {
        if let Some(f) = e.file_by_role(role) {
            *slot = store.file_path(e, f);
        }
    };
    let candidates = chain.iter().map(|e| {
        let e: &ModelEntry = e;
        let mut cfg = SherpaConfig::zipformer(store.model_dir(e), e.name.clone());
        path_of(e, "encoder", &mut cfg.encoder);
        path_of(e, "decoder", &mut cfg.decoder);
        path_of(e, "joiner", &mut cfg.joiner);
        path_of(e, "tokens", &mut cfg.tokens);
        let make = move || -> Result<Box<dyn SpeechEngine>, SttError> {
            Ok(Box::new(SherpaEngine::new(cfg)?))
        };
        (e.id.clone(), make)
    });
    let fb = build_with_fallback(candidates).map_err(|errs| {
        errs.iter().map(|(id, e)| format!("{id}: {e}")).collect::<Vec<_>>().join("; ")
    })?;
    let mut notes: Vec<String> =
        fb.skipped.iter().map(|(id, e)| format!("{id} failed ({e}), fallback")).collect();
    if fb.fell_back() {
        notes.push(format!("using {}", fb.chosen));
    }
    let streaming = fb.value;

    if !prefs.resync {
        return Ok(Some(BuiltEngine {
            engine: Box::new(HybridEngine::streaming_only(streaming)),
            resync: false,
            notes,
        }));
    }
    let Some(re) = m.select(&short, ModelRole::Resync, prefs.resync_model.as_deref(), |e| {
        is_installed(&store, e) && e.model_type() == Some(ModelType::OfflineNemoTransducer)
    }) else {
        notes.push("resync enabled but no resync model installed".into());
        return Ok(Some(BuiltEngine {
            engine: Box::new(HybridEngine::streaming_only(streaming)),
            resync: false,
            notes,
        }));
    };
    let mut ocfg = SherpaOfflineConfig::parakeet(store.model_dir(re), re.name.clone());
    path_of(re, "encoder", &mut ocfg.encoder);
    path_of(re, "decoder", &mut ocfg.decoder);
    path_of(re, "joiner", &mut ocfg.joiner);
    path_of(re, "tokens", &mut ocfg.tokens);
    if let Some(t) = re.model_type().and_then(ModelType::sherpa_model_type) {
        ocfg.model_type = t.into();
    }
    let offline: Box<dyn OfflineEngine> = match SherpaOfflineEngine::new(ocfg) {
        Ok(o) => Box::new(o),
        Err(e) => {
            notes.push(format!("resync model {} failed: {e}", re.id));
            return Ok(Some(BuiltEngine {
                engine: Box::new(HybridEngine::streaming_only(streaming)),
                resync: false,
                notes,
            }));
        }
    };
    // Silero VAD if its model is present, else RMS segmentation (zt-audio).
    let vad_path = root.join(SILERO_VAD_FILE);
    let segmenter: Box<dyn Segmenter> = match SileroSegmenter::new(&SileroVadConfig::new(&vad_path))
    {
        Ok(s) => Box::new(s),
        Err(e) => {
            if vad_path.exists() {
                notes.push(format!("Silero VAD unavailable ({e}), using RMS segmentation"));
            }
            Box::new(RmsSegmenter::default())
        }
    };
    let engine = HybridEngine::with_resync(streaming, offline, segmenter, ResyncOptions::default())
        .map_err(|e| e.to_string())?;
    Ok(Some(BuiltEngine { engine: Box::new(engine), resync: true, notes }))
}
