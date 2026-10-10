//! zero-teleprompter desktop app: Tauri 2 adapter layer (ADR-005, hexagonal).
//!
//! - `appcore`  : Controller + VoiceTracker behind one mutex, the single `dispatch` path, tick loop
//! - `voice`    : mic -> STT -> tracker thread
//! - `commands` : `#[tauri::command]`s (CONCEPT Â§8.3)
//! - `hotkeys`  : global shortcuts
//! - `persist`  : settings / ui prefs / script library
//! - `models`   : model manifest / install / remove
//!
//! Windows (tauri.conf.json): `control` (operator UI, index.html) and
//! `talent` (render-only output, talent.html), both subscribe to the same events.

mod commands;
mod appcore;
mod hotkeys;
mod models;
mod persist;
mod voice;

use tauri::{Emitter, Manager, WindowEvent};
use zt_core::{Command, Controller, Settings, Tokens};

use crate::appcore::{AppCore, Shared};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Linux + NVIDIA: WebKitGTK's DMA-BUF renderer can stutter or render
    // blank; opt-in workaround (CONCEPT M1). Set before any webview exists.
    #[cfg(target_os = "linux")]
    if std::env::var_os("ZT_SAFE_RENDER").is_some()
        && std::env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_none()
    {
        std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(hotkeys::plugin())
        .setup(|app| {
            let handle = app.handle().clone();
            let tokens = Tokens::embedded();
            let settings =
                persist::load_settings(&handle).unwrap_or_else(|| Settings::from_tokens(&tokens));
            let mut ctl = Controller::new(tokens, settings);
            // Start with an empty script; the frontend loads the last one / the example.
            let _ = ctl.apply(Command::LoadScript(String::new()));
            let mut core = AppCore::new(ctl);
            core.voice_models = models::load_prefs(&handle);
            let shared = Shared::new(core);
            app.manage(shared.clone());
            appcore::spawn_tick_loop(handle.clone(), shared);
            hotkeys::register_all(&handle);
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                match window.label() {
                    // The talent window is only hidden so it can be reopened instantly.
                    "talent" => {
                        api.prevent_close();
                        let _ = window.set_fullscreen(false);
                        let _ = window.hide();
                        // Same contract as `commands::close_talent`: the control
                        // window's toggle listens to this event.
                        let _ = window.app_handle().emit(commands::EV_TALENT, false);
                        let shared = window.app_handle().state::<Shared>();
                        let _ = appcore::dispatch(
                            window.app_handle(),
                            &shared,
                            Command::SetFullscreen(false),
                        );
                    }
                    // Closing the operator window quits the app (talent is hidden, not closed).
                    "control" => {
                        voice::stop(&window.app_handle().state::<Shared>());
                        window.app_handle().exit(0);
                    }
                    _ => {}
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::dispatch,
            commands::get_state,
            commands::import_file,
            commands::save_script,
            commands::load_script,
            commands::list_scripts,
            commands::delete_script,
            commands::report_layout,
            commands::get_ui_prefs,
            commands::set_ui_prefs,
            commands::list_monitors,
            commands::open_talent,
            commands::close_talent,
            commands::set_talent_fullscreen,
            commands::talent_visible,
            commands::audio_devices,
            commands::set_audio_device,
            commands::mic_test,
            commands::models_list,
            commands::models_install,
            commands::models_remove,
            commands::voice_models_get,
            commands::voice_models_set,
            commands::voice_models_selection,
            commands::build_info,
        ])
        .run(tauri::generate_context!())
        .expect("error while running zero-teleprompter");
}
