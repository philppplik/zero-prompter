//! Persistence adapter (CONCEPT §8.4):
//! - `<config_dir>/settings.json`   core `Settings` (versioned envelope)
//! - `<config_dir>/ui-prefs.json`   frontend-only preferences (font, colors, panels, onboarding…)
//! - `<data_dir>/scripts/<id>.md`   script library
//!
//! Writes go to a temp file + rename so a crash never leaves half a file.

use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{AppHandle, Manager};
use zt_core::Settings;

const SETTINGS_VERSION: u32 = 1;
const MAX_SCRIPT_BYTES: u64 = 8 * 1024 * 1024;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SettingsFile {
    version: u32,
    settings: Settings,
}

fn config_dir(app: &AppHandle) -> Option<PathBuf> {
    app.path().app_config_dir().ok()
}

fn scripts_dir(app: &AppHandle) -> Result<PathBuf, String> {
    app.path().app_data_dir().map(|d| d.join("scripts")).map_err(|e| e.to_string())
}

pub fn write_atomic(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    if let Some(p) = path.parent() {
        std::fs::create_dir_all(p)?;
    }
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(".tmp");
    let tmp = PathBuf::from(tmp);
    std::fs::write(&tmp, bytes)?;
    std::fs::rename(&tmp, path)
}

pub fn load_settings(app: &AppHandle) -> Option<Settings> {
    let path = config_dir(app)?.join("settings.json");
    let text = std::fs::read_to_string(path).ok()?;
    let file: SettingsFile = serde_json::from_str(text.trim_start_matches('\u{feff}')).ok()?;
    // Future migrations: match file.version { 1 => ..., }
    (file.version <= SETTINGS_VERSION).then_some(file.settings)
}

pub fn save_settings(app: &AppHandle, s: &Settings) {
    let Some(dir) = config_dir(app) else { return };
    let file = SettingsFile { version: SETTINGS_VERSION, settings: s.clone() };
    if let Ok(json) = serde_json::to_vec_pretty(&file) {
        if let Err(e) = write_atomic(&dir.join("settings.json"), &json) {
            eprintln!("zt: saving settings failed: {e}");
        }
    }
}

pub fn load_ui_prefs(app: &AppHandle) -> Value {
    config_dir(app)
        .and_then(|d| std::fs::read_to_string(d.join("ui-prefs.json")).ok())
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or(Value::Object(Default::default()))
}

pub fn save_ui_prefs(app: &AppHandle, prefs: &Value) -> Result<(), String> {
    let dir = config_dir(app).ok_or("no config dir")?;
    let json = serde_json::to_vec_pretty(prefs).map_err(|e| e.to_string())?;
    write_atomic(&dir.join("ui-prefs.json"), &json).map_err(|e| e.to_string())
}

/// Library ids are file stems: `[A-Za-z0-9_-]{1,64}` only (no path traversal).
pub fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScriptMeta {
    pub id: String,
    pub title: String,
    pub modified_ms: u64,
    pub words: usize,
}

fn title_of(text: &str) -> String {
    let first = text.lines().map(str::trim).find(|l| !l.is_empty()).unwrap_or("Ohne Titel");
    let t: String = first.trim_start_matches('#').trim().chars().take(60).collect();
    if t.is_empty() { "Ohne Titel".into() } else { t }
}

pub fn save_script(app: &AppHandle, id: &str, text: &str) -> Result<ScriptMeta, String> {
    if !valid_id(id) {
        return Err(format!("invalid script id '{id}'"));
    }
    let path = scripts_dir(app)?.join(format!("{id}.md"));
    write_atomic(&path, text.as_bytes()).map_err(|e| e.to_string())?;
    Ok(ScriptMeta {
        id: id.into(),
        title: title_of(text),
        modified_ms: now_ms(),
        words: text.split_whitespace().count(),
    })
}

pub fn load_script(app: &AppHandle, id: &str) -> Result<String, String> {
    if !valid_id(id) {
        return Err(format!("invalid script id '{id}'"));
    }
    read_text_file(&scripts_dir(app)?.join(format!("{id}.md")))
}

pub fn delete_script(app: &AppHandle, id: &str) -> Result<(), String> {
    if !valid_id(id) {
        return Err(format!("invalid script id '{id}'"));
    }
    std::fs::remove_file(scripts_dir(app)?.join(format!("{id}.md"))).map_err(|e| e.to_string())
}

pub fn list_scripts(app: &AppHandle) -> Result<Vec<ScriptMeta>, String> {
    let dir = scripts_dir(app)?;
    let Ok(rd) = std::fs::read_dir(&dir) else { return Ok(vec![]) };
    let mut out = Vec::new();
    for e in rd.flatten() {
        let p = e.path();
        if p.extension().and_then(|x| x.to_str()) != Some("md") {
            continue;
        }
        let Some(id) = p.file_stem().and_then(|s| s.to_str()).map(str::to_string) else { continue };
        if !valid_id(&id) {
            continue;
        }
        let text = read_text_file(&p).unwrap_or_default();
        let modified_ms = e
            .metadata()
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        out.push(ScriptMeta {
            id,
            title: title_of(&text),
            modified_ms,
            words: text.split_whitespace().count(),
        });
    }
    out.sort_by(|a, b| b.modified_ms.cmp(&a.modified_ms));
    Ok(out)
}

/// Reads .txt/.md as UTF-8 (BOM stripped). Falls back to Latin-1 for legacy
/// Windows files instead of failing (CONCEPT F1 "Rohtext-Fallback").
pub fn read_text_file(path: &Path) -> Result<String, String> {
    let meta = std::fs::metadata(path).map_err(|e| format!("{}: {e}", path.display()))?;
    if meta.len() > MAX_SCRIPT_BYTES {
        return Err(format!("file too large ({} MB, max 8 MB)", meta.len() / 1_000_000));
    }
    let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let text = match String::from_utf8(bytes) {
        Ok(s) => s,
        Err(e) => e.into_bytes().iter().map(|&b| b as char).collect(),
    };
    Ok(text.trim_start_matches('\u{feff}').replace("\r\n", "\n"))
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_safe() {
        assert!(valid_id("keynote_q4-2026"));
        assert!(!valid_id("../etc"));
        assert!(!valid_id(""));
        assert!(!valid_id("a/b"));
    }

    #[test]
    fn titles() {
        assert_eq!(title_of("\n# Keynote Q4\nText"), "Keynote Q4");
        assert_eq!(title_of("   "), "Ohne Titel");
    }
}
