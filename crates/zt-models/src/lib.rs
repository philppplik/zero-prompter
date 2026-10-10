//! zt-models: which STT models exist, where they come from, and whether the
//! installed files are intact.
//!
//! - [`Manifest`]: JSON list of models (id, language, tier, license, files with
//!   URL + SHA-256 + size).
//! - [`sha256_file`] / [`verify_file`]: streaming SHA-256 check (sha2 crate).
//! - [`ModelStore`]: install directory layout and status.
//! - [`Downloader`]: port for the HTTP adapter (implemented in the app, not
//!   here, so this crate stays free of network code).

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum ModelError {
    #[error("io error at {path}: {source}")]
    Io { path: PathBuf, source: io::Error },
    #[error("invalid manifest: {0}")]
    Manifest(#[from] serde_json::Error),
    #[error("manifest validation failed: {0}")]
    Invalid(String),
    #[error("checksum mismatch for {path}: expected {expected}, got {actual}")]
    Checksum { path: PathBuf, expected: String, actual: String },
    #[error("size mismatch for {path}: expected {expected} bytes, got {actual}")]
    Size { path: PathBuf, expected: u64, actual: u64 },
    #[error("unknown model '{0}'")]
    UnknownModel(String),
    #[error("download failed: {0}")]
    Download(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Tier {
    S,
    M,
    L,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Engine {
    SherpaOnlineTransducer,
    SherpaOfflineTransducer,
    SherpaVad,
}

/// How the model is decoded (maps to sherpa-onnx recognizer + `model_type`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelType {
    /// Streaming Zipformer transducer -> `OnlineRecognizer`.
    OnlineTransducer,
    /// NeMo (Parakeet TDT) transducer -> `OfflineRecognizer`,
    /// `model_config.model_type = "nemo_transducer"`.
    OfflineNemoTransducer,
}

impl ModelType {
    /// sherpa-onnx `model_type` string (None = let sherpa auto-detect).
    pub fn sherpa_model_type(self) -> Option<&'static str> {
        match self {
            ModelType::OnlineTransducer => None,
            ModelType::OfflineNemoTransducer => Some("nemo_transducer"),
        }
    }
}

/// What the model is used for in the hybrid pipeline (ADR-006).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ModelRole {
    /// Low-latency streaming recognizer that drives the cursor.
    Streaming,
    /// Offline recognizer that re-anchors the cursor from VAD segments.
    Resync,
}

/// Deserialises `""`/`null`/missing as `None` (see [`ModelFile::sha256`]).
fn de_opt_sha<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    let v: Option<String> = Option::deserialize(d)?;
    Ok(v.map(|s| s.trim().to_ascii_lowercase()).filter(|s| !s.is_empty()))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelFile {
    /// Relative path inside the model directory (no `..`, no absolute paths).
    pub path: String,
    pub url: String,
    /// Lower-case hex SHA-256, or `None` = *unverified*.
    ///
    /// Why optional instead of a hard error: Hugging Face exposes a SHA-256
    /// only for LFS objects. Small plain-git files (`tokens.txt`) have only a
    /// git blob SHA-1, so the manifest cannot carry an authoritative hash
    /// without downloading first. Rejecting the *whole* manifest for that
    /// disabled every model. Instead such files are integrity-checked by
    /// exact size + HTTPS, the computed hash is reported after download
    /// ([`InstallReport`]) and surfaced as a warning ("unverified"); a
    /// *malformed* hash is still a hard error, and a file whose hash is given
    /// is still verified strictly.
    #[serde(default, deserialize_with = "de_opt_sha", skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
    pub size: u64,
    /// Role for the engine config, e.g. "encoder", "decoder", "joiner", "tokens".
    #[serde(default)]
    pub role: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelEntry {
    pub id: String,
    pub name: String,
    /// BCP-47 primary languages, e.g. ["de"].
    pub languages: Vec<String>,
    pub tier: Tier,
    pub engine: Engine,
    /// Explicit decoder type; derived from `engine` when absent (schema v1).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_type: Option<ModelType>,
    /// Pipeline roles; derived from the model type when empty.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub roles: Vec<ModelRole>,
    /// Preferred default for its languages/role (lower = preferred, `None` = never auto-picked).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_rank: Option<u32>,
    /// SPDX id or "unknown" (must be cleared before bundling, RESEARCH.md §licences).
    pub license: String,
    #[serde(default)]
    pub attribution: Option<String>,
    pub files: Vec<ModelFile>,
}

impl ModelEntry {
    pub fn total_size(&self) -> u64 {
        self.files.iter().map(|f| f.size).sum()
    }

    pub fn file_by_role(&self, role: &str) -> Option<&ModelFile> {
        self.files.iter().find(|f| f.role.as_deref() == Some(role))
    }

    /// Attribution must be shown for CC-BY models (About + download dialog).
    pub fn requires_attribution(&self) -> bool {
        self.license.starts_with("CC-BY")
    }

    /// Effective model type (explicit, else derived from `engine`).
    pub fn model_type(&self) -> Option<ModelType> {
        self.model_type.or(match self.engine {
            Engine::SherpaOnlineTransducer => Some(ModelType::OnlineTransducer),
            Engine::SherpaOfflineTransducer => Some(ModelType::OfflineNemoTransducer),
            Engine::SherpaVad => None,
        })
    }

    /// Effective roles (explicit, else streaming for online, resync for offline).
    pub fn effective_roles(&self) -> Vec<ModelRole> {
        if !self.roles.is_empty() {
            return self.roles.clone();
        }
        match self.model_type() {
            Some(ModelType::OnlineTransducer) => vec![ModelRole::Streaming],
            Some(ModelType::OfflineNemoTransducer) => vec![ModelRole::Resync],
            None => vec![],
        }
    }

    pub fn has_role(&self, role: ModelRole) -> bool {
        self.effective_roles().contains(&role)
    }

    pub fn supports_language(&self, lang: &str) -> bool {
        self.languages.iter().any(|l| l == lang)
    }

    /// Files without a manifest hash (installed with size check only).
    pub fn unverified_files(&self) -> impl Iterator<Item = &ModelFile> {
        self.files.iter().filter(|f| f.sha256.is_none())
    }
}

/// Reduces "de-DE" / "en_US" / "DE" to the primary subtag "de".
pub fn primary_language(lang: &str) -> String {
    lang.split(['-', '_']).next().unwrap_or(lang).to_ascii_lowercase()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    pub schema_version: u32,
    pub models: Vec<ModelEntry>,
}

impl Manifest {
    /// v2 adds `modelType`, `roles`, `defaultRank` and optional `sha256`;
    /// v1 manifests are still accepted (fields are derived).
    pub const SCHEMA_VERSION: u32 = 2;

    pub fn parse(json: &str) -> Result<Self, ModelError> {
        let m: Manifest = serde_json::from_str(json.trim_start_matches('\u{feff}'))?;
        m.validate()?;
        Ok(m)
    }

    pub fn to_json(&self) -> Result<String, ModelError> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    pub fn validate(&self) -> Result<(), ModelError> {
        if !(1..=Self::SCHEMA_VERSION).contains(&self.schema_version) {
            return Err(ModelError::Invalid(format!(
                "unsupported schemaVersion {}",
                self.schema_version
            )));
        }
        let mut ids = std::collections::BTreeSet::new();
        for m in &self.models {
            if !ids.insert(m.id.as_str()) {
                return Err(ModelError::Invalid(format!("duplicate model id '{}'", m.id)));
            }
            if !is_valid_model_id(&m.id) {
                return Err(ModelError::Invalid(format!("bad model id '{}'", m.id)));
            }
            for f in &m.files {
                if !is_safe_relative(&f.path) {
                    return Err(ModelError::Invalid(format!(
                        "unsafe path '{}' in '{}'",
                        f.path, m.id
                    )));
                }
                // Missing hash = warning (see `warnings`); malformed hash = error.
                if let Some(sha) = &f.sha256 {
                    if sha.len() != 64 || !sha.chars().all(|c| c.is_ascii_hexdigit()) {
                        return Err(ModelError::Invalid(format!("bad sha256 for '{}'", f.path)));
                    }
                }
                if f.size == 0 {
                    return Err(ModelError::Invalid(format!("size 0 for '{}'", f.path)));
                }
                if !f.url.starts_with("https://") {
                    return Err(ModelError::Invalid(format!("non-https url for '{}'", f.path)));
                }
            }
        }
        Ok(())
    }

    pub fn get(&self, id: &str) -> Result<&ModelEntry, ModelError> {
        self.models.iter().find(|m| m.id == id).ok_or_else(|| ModelError::UnknownModel(id.into()))
    }

    pub fn for_language<'a>(&'a self, lang: &'a str) -> impl Iterator<Item = &'a ModelEntry> + 'a {
        self.models.iter().filter(move |m| m.languages.iter().any(|l| l == lang))
    }

    /// Helper models for voice-activity detection (engine `sherpaVad`, e.g.
    /// Silero VAD). Small and language-independent: the app installs them
    /// automatically together with any speech model.
    pub fn vad_models(&self) -> impl Iterator<Item = &ModelEntry> {
        self.models.iter().filter(|m| m.engine == Engine::SherpaVad)
    }

    /// Non-fatal findings, e.g. files that will be installed *unverified*.
    pub fn warnings(&self) -> Vec<String> {
        self.models
            .iter()
            .flat_map(|m| {
                m.unverified_files().map(move |f| {
                    format!(
                        "{}/{}: no sha256 in manifest (unverified, size-checked only)",
                        m.id, f.path
                    )
                })
            })
            .collect()
    }

    /// All models for `lang` (primary subtag) with `role`, preferred first:
    /// ranked models by `defaultRank`, then unranked in manifest order.
    pub fn candidates(&self, lang: &str, role: ModelRole) -> Vec<&ModelEntry> {
        let lang = primary_language(lang);
        let mut v: Vec<(usize, &ModelEntry)> = self
            .models
            .iter()
            .enumerate()
            .filter(|(_, m)| m.supports_language(&lang) && m.has_role(role))
            .collect();
        v.sort_by_key(|(i, m)| (m.default_rank.unwrap_or(u32::MAX), *i));
        v.into_iter().map(|(_, m)| m).collect()
    }

    /// Fallback chain: the user's choice first (if it fits `lang`/`role`),
    /// then the ranked candidates. Callers pick the first *installed* entry.
    pub fn fallback_chain(
        &self,
        lang: &str,
        role: ModelRole,
        preferred: Option<&str>,
    ) -> Vec<&ModelEntry> {
        let mut chain = self.candidates(lang, role);
        if let Some(pos) = preferred.and_then(|p| chain.iter().position(|m| m.id == p)) {
            let m = chain.remove(pos);
            chain.insert(0, m);
        }
        chain
    }

    /// First installed model of the fallback chain.
    /// Installed streaming models to try for `lang`, in order. If no model for
    /// `lang` is installed, falls back to installed streaming models of *other*
    /// languages (better than demo mode; the caller reports this loudly).
    /// Only entries satisfying `usable` (e.g. installed + right decoder type)
    /// are returned. Empty result = no usable streaming model at all.
    pub fn streaming_plan<'a>(
        &'a self,
        lang: &str,
        preferred: Option<&str>,
        usable: impl Fn(&ModelEntry) -> bool,
    ) -> StreamingPlan<'a> {
        let lang = primary_language(lang);
        let chain: Vec<&ModelEntry> = self
            .fallback_chain(&lang, ModelRole::Streaming, preferred)
            .into_iter()
            .filter(|m| usable(m))
            .collect();
        if !chain.is_empty() {
            return StreamingPlan { chain, language_fallback: None };
        }
        let mut other: Vec<(usize, &ModelEntry)> = self
            .models
            .iter()
            .enumerate()
            .filter(|(_, m)| m.has_role(ModelRole::Streaming) && usable(m))
            .collect();
        other.sort_by_key(|(i, m)| (m.default_rank.unwrap_or(u32::MAX), *i));
        let chain: Vec<&ModelEntry> = other.into_iter().map(|(_, m)| m).collect();
        let language_fallback = chain.first().map(|m| m.languages.join("/"));
        StreamingPlan { chain, language_fallback }
    }

    pub fn select<'a>(
        &'a self,
        lang: &str,
        role: ModelRole,
        preferred: Option<&str>,
        installed: impl Fn(&ModelEntry) -> bool,
    ) -> Option<&'a ModelEntry> {
        self.fallback_chain(lang, role, preferred).into_iter().find(|m| installed(m))
    }
}

/// Result of [`Manifest::streaming_plan`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamingPlan<'a> {
    /// Candidates in the order they should be tried.
    pub chain: Vec<&'a ModelEntry>,
    /// `Some(languages of chain[0])` when no model for the requested
    /// language was usable and another language's model is used instead.
    pub language_fallback: Option<String>,
}

/// Model ids: non-empty, `[A-Za-z0-9._-]` only.
fn is_valid_model_id(id: &str) -> bool {
    !id.is_empty() && id.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
}

/// Rejects absolute paths, `..`, backslashes and drive prefixes (zip-slip guard).
pub fn is_safe_relative(p: &str) -> bool {
    !p.is_empty()
        && !p.starts_with('/')
        && !p.contains('\\')
        && !p.contains(':')
        && p.split('/').all(|seg| !seg.is_empty() && seg != ".." && seg != ".")
}

fn io_err(path: &Path) -> impl FnOnce(io::Error) -> ModelError + '_ {
    move |source| ModelError::Io { path: path.to_path_buf(), source }
}

/// Lower-case hex SHA-256 of any reader (streamed, 64 KiB buffer).
pub fn sha256_reader(mut r: impl Read) -> io::Result<String> {
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 64 * 1024];
    loop {
        let n = r.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(to_hex(&hasher.finalize()))
}

pub fn sha256_file(path: &Path) -> Result<String, ModelError> {
    let f = File::open(path).map_err(io_err(path))?;
    sha256_reader(io::BufReader::new(f)).map_err(io_err(path))
}

fn to_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push(HEX[(b >> 4) as usize] as char);
        s.push(HEX[(b & 0xf) as usize] as char);
    }
    s
}

/// Result of a successful file check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verified {
    /// Size and manifest SHA-256 match.
    Hash,
    /// No manifest hash: size matched; `actual` is the computed SHA-256
    /// (shown in the UI so it can be pinned into the manifest).
    Unverified { actual: String },
}

/// Checks size first (cheap), then SHA-256 (if the manifest has one).
pub fn verify_file(path: &Path, expected: &ModelFile) -> Result<Verified, ModelError> {
    let size = std::fs::metadata(path).map_err(io_err(path))?.len();
    if size != expected.size {
        return Err(ModelError::Size { path: path.into(), expected: expected.size, actual: size });
    }
    let actual = sha256_file(path)?;
    match &expected.sha256 {
        Some(sha) if !actual.eq_ignore_ascii_case(sha) => {
            Err(ModelError::Checksum { path: path.into(), expected: sha.clone(), actual })
        }
        Some(_) => Ok(Verified::Hash),
        None => Ok(Verified::Unverified { actual }),
    }
}

/// Hashes computed for files the manifest could not verify.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallReport {
    /// `(relative path, computed sha256)` for every unverified file.
    pub unverified: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelStatus {
    NotInstalled,
    /// Some files present; list of missing/corrupt relative paths.
    Incomplete(Vec<String>),
    Installed,
}

/// Install layout: `<root>/<model-id>/<file.path>`; downloads go to
/// `<file>.part` and are renamed only after verification.
#[derive(Debug, Clone)]
pub struct ModelStore {
    root: PathBuf,
}

impl ModelStore {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn model_dir(&self, m: &ModelEntry) -> PathBuf {
        self.root.join(&m.id)
    }

    pub fn file_path(&self, m: &ModelEntry, f: &ModelFile) -> PathBuf {
        f.path.split('/').fold(self.model_dir(m), |p, seg| p.join(seg))
    }

    pub fn partial_path(&self, m: &ModelEntry, f: &ModelFile) -> PathBuf {
        let mut p = self.file_path(m, f).into_os_string();
        p.push(".part");
        p.into()
    }

    /// Full check (hashes every file). Use [`ModelStore::quick_status`] at startup.
    pub fn status(&self, m: &ModelEntry) -> ModelStatus {
        self.status_with(m, |path, f| verify_file(path, f).is_ok())
    }

    /// Size-only check, fast enough for every app start.
    pub fn quick_status(&self, m: &ModelEntry) -> ModelStatus {
        self.status_with(m, |path, f| {
            std::fs::metadata(path).map(|md| md.len() == f.size).unwrap_or(false)
        })
    }

    fn status_with(&self, m: &ModelEntry, ok: impl Fn(&Path, &ModelFile) -> bool) -> ModelStatus {
        let bad: Vec<String> = m
            .files
            .iter()
            .filter(|f| !ok(&self.file_path(m, f), f))
            .map(|f| f.path.clone())
            .collect();
        if bad.is_empty() {
            ModelStatus::Installed
        } else if bad.len() == m.files.len() && !self.model_dir(m).exists() {
            ModelStatus::NotInstalled
        } else {
            ModelStatus::Incomplete(bad)
        }
    }

    /// Verifies a finished `.part` download and atomically moves it into place.
    /// On mismatch the partial file is deleted.
    pub fn finalize_download(
        &self,
        m: &ModelEntry,
        f: &ModelFile,
    ) -> Result<(PathBuf, Verified), ModelError> {
        let part = self.partial_path(m, f);
        let v = match verify_file(&part, f) {
            Ok(v) => v,
            Err(e) => {
                let _ = std::fs::remove_file(&part);
                return Err(e);
            }
        };
        let dst = self.file_path(m, f);
        std::fs::rename(&part, &dst).map_err(io_err(&dst))?;
        Ok((dst, v))
    }
}

/// Download progress callback: (bytes_done, bytes_total).
pub type Progress<'a> = &'a mut dyn FnMut(u64, u64);

/// Port for the network adapter (implemented in the app with an HTTP client).
/// Must support resuming from an existing `.part` file via HTTP Range.
pub trait Downloader {
    fn download(
        &mut self,
        url: &str,
        dest_part: &Path,
        expected_size: u64,
        progress: Progress<'_>,
    ) -> Result<(), ModelError>;
}

/// Downloads all missing files of a model through `dl` and verifies them.
pub fn install(
    store: &ModelStore,
    m: &ModelEntry,
    dl: &mut dyn Downloader,
    progress: Progress<'_>,
) -> Result<InstallReport, ModelError> {
    let total = m.total_size();
    let mut done_before = 0u64;
    let mut report = InstallReport::default();
    for f in &m.files {
        let dst = store.file_path(m, f);
        if let Ok(v) = verify_file(&dst, f) {
            if let Verified::Unverified { actual } = v {
                report.unverified.push((f.path.clone(), actual));
            }
            done_before += f.size;
            progress(done_before, total);
            continue;
        }
        if let Some(parent) = dst.parent() {
            std::fs::create_dir_all(parent).map_err(io_err(parent))?;
        }
        let part = store.partial_path(m, f);
        let base = done_before;
        dl.download(&f.url, &part, f.size, &mut |d, _| progress(base + d, total))?;
        if let (_, Verified::Unverified { actual }) = store.finalize_download(m, f)? {
            report.unverified.push((f.path.clone(), actual));
        }
        done_before += f.size;
    }
    Ok(report)
}

#[cfg(test)]
mod tests;
