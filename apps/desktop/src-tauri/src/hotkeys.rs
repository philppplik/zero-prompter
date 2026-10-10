//! System-wide hotkeys (tauri-plugin-global-shortcut) so the operator can
//! drive the prompter while another app (OBS, slides, Zoom) has focus.
//!
//! In-app keys (Space, arrows, PgUp/PgDn, M, V, …) are handled in the
//! frontend; global ones always use Ctrl+Shift+F-keys so they never steal plain keys
//! from other applications. Handler: Fn(&AppHandle, &Shortcut, ShortcutEvent) (plugin 2.x).

use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
use zt_core::Command;

use crate::appcore::{self, Shared};

fn bindings() -> Vec<(Shortcut, &'static str, Command)> {
    let m = Some(Modifiers::CONTROL | Modifiers::SHIFT);
    // Ctrl+Shift+F6..F11: function-key chords are rarely bound system-wide.
    // Avoided on purpose: Ctrl+Shift+V (paste as plain text), Ctrl+Shift+Arrows
    // (select word/line in editors), Ctrl+Shift+Space (IME / Office), Ctrl+Alt+*
    // (= AltGr on German layouts), Ctrl+Shift+Esc (Task Manager), F12 (devtools).
    vec![
        (Shortcut::new(m, Code::F7), "Ctrl+Shift+F7", Command::TogglePlay),
        (Shortcut::new(m, Code::F8), "Ctrl+Shift+F8", Command::NudgeWpm(-5)),
        (Shortcut::new(m, Code::F9), "Ctrl+Shift+F9", Command::NudgeWpm(5)),
        (Shortcut::new(m, Code::F10), "Ctrl+Shift+F10", Command::JumpParagraph(-1)),
        (Shortcut::new(m, Code::F11), "Ctrl+Shift+F11", Command::JumpParagraph(1)),
        (Shortcut::new(m, Code::F6), "Ctrl+Shift+F6", Command::ToggleVoice),
    ]
}

/// (label, command name) pairs for the shortcut overlay / about page.
pub fn describe() -> Vec<(String, String)> {
    bindings().into_iter().map(|(_, label, cmd)| (label.to_string(), format!("{cmd:?}"))).collect()
}

/// Builds the plugin with a handler that maps shortcuts to commands.
pub fn plugin() -> tauri::plugin::TauriPlugin<tauri::Wry> {
    tauri_plugin_global_shortcut::Builder::new()
        .with_handler(|app: &AppHandle, shortcut: &Shortcut, event| {
            if event.state() != ShortcutState::Pressed {
                return;
            }
            let Some(cmd) =
                bindings().into_iter().find(|(s, _, _)| *s == *shortcut).map(|(_, _, c)| c)
            else {
                return;
            };
            let shared = app.state::<Shared>();
            if let Err(e) = appcore::dispatch(app, &shared, cmd) {
                tracing::warn!(target: "zt::hotkeys", error = %e, "global shortcut ignored");
            }
        })
        .build()
}

/// Registers all bindings; failures (key taken by another app) are logged, not fatal.
pub fn register_all(app: &AppHandle) {
    for (s, label, _) in bindings() {
        if let Err(e) = app.global_shortcut().register(s) {
            tracing::warn!(target: "zt::hotkeys", %label, error = %e, "global shortcut unavailable");
        }
    }
}
