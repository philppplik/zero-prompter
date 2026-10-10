//! Tauri commands = the inbound side of the bridge (CONCEPT §8.3).
//! Every state mutation is funnelled through `appcore::dispatch`.

use serde::Serialize;
use serde_json::Value;
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, State};
use zt_audio::AudioSource;
use zt_core::{AppState, Command};

use crate::appcore::{self, Shared};
use crate::{models, persist, voice};

pub const EV_PREFS: &str = "prefs://changed";
pub const EV_TALENT: &str = "talent://visibility";

type CmdResult<T> = Result<T, String>;

#[tauri::command]
pub fn dispatch(app: AppHandle, shared: State<'_, Shared>, cmd: Command) -> CmdResult<()> {
    appcore::dispatch(&app, &shared, cmd)
}

#[tauri::command]
pub fn get_state(shared: State<'_, Shared>) -> AppState {
    shared.lock().ctl.state().clone()
}

/// Imports a .txt/.md file and loads it. Returns the text for the editor.
#[tauri::command]
pub fn import_file(app: AppHandle, shared: State<'_, Shared>, path: String) -> CmdResult<String> {
    let p = std::path::Path::new(&path);
    let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase();
    if !matches!(ext.as_str(), "txt" | "md" | "markdown" | "text") {
        return Err(format!("Format .{ext} wird (noch) nicht unterstützt – bitte .txt oder .md"));
    }
    let text = persist::read_text_file(p)?;
    appcore::dispatch(&app, &shared, Command::LoadScript(text.clone()))?;
    Ok(text)
}

#[tauri::command]
pub fn save_script(app: AppHandle, id: String, md: String) -> CmdResult<persist::ScriptMeta> {
    persist::save_script(&app, &id, &md)
}

#[tauri::command]
pub fn load_script(app: AppHandle, shared: State<'_, Shared>, id: String) -> CmdResult<String> {
    let text = persist::load_script(&app, &id)?;
    appcore::dispatch(&app, &shared, Command::LoadScript(text.clone()))?;
    Ok(text)
}

#[tauri::command]
pub fn list_scripts(app: AppHandle) -> CmdResult<Vec<persist::ScriptMeta>> {
    persist::list_scripts(&app)
}

#[tauri::command]
pub fn delete_script(app: AppHandle, id: String) -> CmdResult<()> {
    persist::delete_script(&app, &id)
}

/// Layout ownership lies with the frontend; Rust only keeps the latest
/// report for the current script version and rejects stale ones.
#[tauri::command]
pub fn report_layout(shared: State<'_, Shared>, script_version: u64, lines: u32) -> bool {
    let mut g = shared.lock();
    if g.ctl.state().script.version != script_version {
        return false;
    }
    g.layout = Some((script_version, lines));
    true
}

#[tauri::command]
pub fn get_ui_prefs(app: AppHandle) -> Value {
    persist::load_ui_prefs(&app)
}

/// Saves frontend-only preferences and broadcasts them to all windows
/// (keeps the talent window in sync without its own logic).
#[tauri::command]
pub fn set_ui_prefs(app: AppHandle, prefs: Value) -> CmdResult<()> {
    persist::save_ui_prefs(&app, &prefs)?;
    app.emit(EV_PREFS, prefs).map_err(|e| e.to_string())
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitorInfo {
    pub index: usize,
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub scale: f64,
    pub primary: bool,
}

#[tauri::command]
pub fn list_monitors(app: AppHandle) -> CmdResult<Vec<MonitorInfo>> {
    let win = app.get_webview_window("control").ok_or("control window missing")?;
    let primary = win.primary_monitor().ok().flatten().and_then(|m| m.name().cloned());
    let mons = win.available_monitors().map_err(|e| e.to_string())?;
    Ok(mons
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let name = m.name().cloned().unwrap_or_else(|| format!("Monitor {}", i + 1));
            MonitorInfo {
                index: i,
                primary: primary.as_deref() == Some(name.as_str()),
                name,
                x: m.position().x,
                y: m.position().y,
                width: m.size().width,
                height: m.size().height,
                scale: m.scale_factor(),
            }
        })
        .collect())
}

/// Shows the talent output window, optionally on `monitor` and fullscreen.
#[tauri::command]
pub fn open_talent(
    app: AppHandle,
    shared: State<'_, Shared>,
    monitor: Option<usize>,
    fullscreen: bool,
) -> CmdResult<()> {
    let win = app.get_webview_window("talent").ok_or("talent window missing")?;
    if let Some(i) = monitor {
        let mons = win.available_monitors().map_err(|e| e.to_string())?;
        if let Some(m) = mons.get(i) {
            // Leave fullscreen first, otherwise some WMs ignore the move.
            let _ = win.set_fullscreen(false);
            win.set_position(PhysicalPosition::new(m.position().x + 40, m.position().y + 40))
                .map_err(|e| e.to_string())?;
        }
    }
    win.show().map_err(|e| e.to_string())?;
    win.set_fullscreen(fullscreen).map_err(|e| e.to_string())?;
    let _ = app.emit(EV_TALENT, true);
    appcore::dispatch(&app, &shared, Command::SetFullscreen(fullscreen))
}

#[tauri::command]
pub fn close_talent(app: AppHandle, shared: State<'_, Shared>) -> CmdResult<()> {
    if let Some(win) = app.get_webview_window("talent") {
        let _ = win.set_fullscreen(false);
        win.hide().map_err(|e| e.to_string())?;
    }
    let _ = app.emit(EV_TALENT, false);
    appcore::dispatch(&app, &shared, Command::SetFullscreen(false))
}

#[tauri::command]
pub fn set_talent_fullscreen(
    app: AppHandle,
    shared: State<'_, Shared>,
    fullscreen: bool,
) -> CmdResult<()> {
    if let Some(win) = app.get_webview_window("talent") {
        win.set_fullscreen(fullscreen).map_err(|e| e.to_string())?;
    }
    appcore::dispatch(&app, &shared, Command::SetFullscreen(fullscreen))
}

#[tauri::command]
pub fn talent_visible(app: AppHandle) -> bool {
    app.get_webview_window("talent").and_then(|w| w.is_visible().ok()).unwrap_or(false)
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioDevice {
    pub id: String,
    pub name: String,
    pub is_default: bool,
}

#[tauri::command]
pub fn audio_devices() -> CmdResult<Vec<AudioDevice>> {
    #[cfg(feature = "cpal")]
    let src = zt_audio::CpalSource::new();
    #[cfg(not(feature = "cpal"))]
    let src = zt_audio::MockSource::new(vec![], 1);
    let list = src.list_devices().map_err(|e| e.to_string())?;
    Ok(list
        .into_iter()
        .map(|d| AudioDevice { id: d.id, name: d.name, is_default: d.is_default })
        .collect())
}

#[tauri::command]
pub fn set_audio_device(shared: State<'_, Shared>, id: Option<String>) {
    shared.lock().audio_device = id;
}

#[tauri::command]
pub fn mic_test(app: AppHandle, shared: State<'_, Shared>, on: bool) {
    voice::set_mic_test(&app, &shared, on);
}

#[tauri::command]
pub fn models_list(app: AppHandle) -> CmdResult<Vec<models::ModelRow>> {
    models::list(&app)
}

#[tauri::command]
pub fn models_install(app: AppHandle, id: String) -> CmdResult<()> {
    models::install(&app, &id)
}

#[tauri::command]
pub fn models_remove(app: AppHandle, id: String) -> CmdResult<()> {
    models::remove(&app, &id)
}

#[tauri::command]
pub fn voice_models_get(shared: State<'_, Shared>) -> models::VoiceModelPrefs {
    shared.lock().voice_models.clone()
}

/// Stores the model choice; restarts a running voice session so the new
/// engine (or resync on/off) takes effect immediately.
#[tauri::command]
pub fn voice_models_set(
    app: AppHandle,
    shared: State<'_, Shared>,
    prefs: models::VoiceModelPrefs,
) -> CmdResult<()> {
    models::save_prefs(&app, &prefs)?;
    tracing::info!(target: "zt::models", prefs = ?prefs, "voice model prefs changed");
    shared.lock().voice_models = prefs;
    voice::restart_if_running(&app, &shared);
    Ok(())
}

/// Diagnostics snapshot for the "Diagnose" settings tab (polled while open).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagReport {
    #[serde(flatten)]
    pub snapshot: crate::diag::Snapshot,
    /// Installed streaming models usable for the script language, in order.
    pub selection: Option<models::Selection>,
}

#[tauri::command]
pub fn diag_snapshot(app: AppHandle, shared: State<'_, Shared>) -> DiagReport {
    let (prefs, lang) = {
        let g = shared.lock();
        (g.voice_models.clone(), g.ctl.state().settings.language.clone())
    };
    DiagReport {
        snapshot: crate::diag::snapshot(),
        selection: models::selection(&app, &lang, &prefs).ok(),
    }
}

/// Writes all log files (plus a JSON snapshot) to `dest` (chosen via the
/// save dialog). Returns the number of bytes written.
#[tauri::command]
pub fn diag_export_log(dest: String) -> CmdResult<u64> {
    crate::diag::export_logs(std::path::Path::new(&dest))
}

/// Log directory path (shown in the panel so users can attach the files).
#[tauri::command]
pub fn diag_log_dir() -> Option<String> {
    crate::diag::snapshot().log_dir
}

/// Errors reported by the frontend (unhandled rejections etc.) go to the same log.
#[tauri::command]
pub fn diag_log_frontend(level: String, message: String) {
    match level.as_str() {
        "error" => tracing::error!(target: "zt::frontend", %message),
        "warn" => tracing::warn!(target: "zt::frontend", %message),
        _ => tracing::info!(target: "zt::frontend", %message),
    }
}

/// Which models would run for `lang` (default: the script language).
#[tauri::command]
pub fn voice_models_selection(
    app: AppHandle,
    shared: State<'_, Shared>,
    lang: Option<String>,
) -> CmdResult<models::Selection> {
    let (prefs, script_lang) = {
        let g = shared.lock();
        (g.voice_models.clone(), g.ctl.state().settings.language.clone())
    };
    models::selection(&app, lang.as_deref().unwrap_or(&script_lang), &prefs)
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildInfo {
    pub version: &'static str,
    pub cpal: bool,
    pub sherpa: bool,
    pub download: bool,
    pub global_shortcuts: Vec<(String, String)>,
}

#[tauri::command]
pub fn build_info() -> BuildInfo {
    BuildInfo {
        version: env!("CARGO_PKG_VERSION"),
        cpal: cfg!(feature = "cpal"),
        sherpa: cfg!(feature = "sherpa"),
        download: cfg!(feature = "download"),
        global_shortcuts: crate::hotkeys::describe(),
    }
}

/// Opens one of a fixed set of project links in the system browser ("Über"
/// tab). Only known keys are accepted, so the webview cannot launch arbitrary
/// URLs or programs. Uses the OS launcher directly (no opener plugin needed).
#[tauri::command]
pub fn open_link(key: String) -> CmdResult<()> {
    const REPO: &str = "https://github.com/philppplik/zero-prompter";
    let url = match key.as_str() {
        "repo" => REPO.to_string(),
        "notices" => format!("{REPO}/blob/main/THIRD_PARTY_NOTICES.md"),
        "models" => format!("{REPO}/blob/main/THIRD_PARTY_MODELS.md"),
        "license" => format!("{REPO}#license"),
        "changelog" => format!("{REPO}/blob/main/CHANGELOG.md"),
        other => return Err(format!("unknown link: {other}")),
    };
    #[cfg(target_os = "windows")]
    let mut cmd = std::process::Command::new("explorer.exe");
    #[cfg(target_os = "macos")]
    let mut cmd = std::process::Command::new("open");
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let mut cmd = std::process::Command::new("xdg-open");
    cmd.arg(&url).spawn().map(|_| ()).map_err(|e| e.to_string())
}
