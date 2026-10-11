# Keyboard shortcuts

On macOS, `Ctrl` in this list means `Cmd`: the app treats the Meta key like Ctrl, so one binding works everywhere. In text fields, plain keys (without Ctrl or Cmd) type text and do not trigger actions; `Esc` still works. Inside text fields the usual edit chords (`Ctrl+A`, `C`, `V`, `X`, `Z`, `Y`, `Shift+Z`, `Up` and `Down`) are left to the field. Press `?` in the app to see this list.

Sources: `apps/desktop/src/lib/hotkeys.ts` (in-app and display of global keys), `apps/desktop/src-tauri/src/hotkeys.rs` (global keys), and the default hotkey table in `crates/zt-core/src/settings.rs`.

## Remappable shortcuts

These are stored in `Settings.hotkeys` and can be changed in Settings, Shortcuts (Tastenkürzel).

| Action ID | Default | Effect |
|---|---|---|
| `togglePlay` | `Space` | Play or pause. A countdown runs first when starting (if enabled). |
| `stop` | `Home` | Live: stop and return to the start. Edit: jump to the start. |
| `faster` | `Up` | +5 WPM (shows a brief "N WPM" notice). |
| `slower` | `Down` | -5 WPM. |
| `nextParagraph` | `PgDn` | Next paragraph. |
| `prevParagraph` | `PgUp` | Previous paragraph. |
| `toggleMirror` | `M` | Mirror horizontally. |
| `toggleFullscreen` | `F` | Toggle fullscreen of the talent window. Opens the window if it is hidden. |
| `toggleVoice` | `V` | Voice tracking on or off. |
| `resyncHere` | `R` | Resync at the current marker position. |
| `commandPalette` | `Ctrl+K` | Command palette. |
| `shortcutOverlay` | `?` | Shortcut overview. |

## Fixed in-app shortcuts

| Key | Effect |
|---|---|
| `Shift+M` | Mirror vertically. |
| `Ctrl+E` | Toggle Edit and Live mode. |
| `Ctrl+O` | Open a script file. |
| `Ctrl+B` | Show or hide the scripts drawer (also the rail button). |
| `Ctrl+,` | Open or close settings. |
| `Ctrl+=` and `Ctrl+-` | Font size +4 px or -4 px. |
| `Ctrl+1` to `Ctrl+6` | Select theme 1 to 6. |
| `Ctrl+Up` | Jump to the start (an alternative for Mac keyboards without a `Home` key). |
| `Esc` | Closes, in this order: command palette, shortcut overview, settings, scripts drawer. If none is open, it cancels a running countdown, or else exits talent fullscreen. |

## Mouse wheel

The wheel works over the preview in the control window and over the talent window. The effect depends on the scroll mode. Voice tracking and voice-activated mode count as "voice on" only while the voice pipeline is actually enabled.

| Mode | Wheel | `Shift` + wheel | `Ctrl` or `Alt` (macOS `Cmd` or `Opt`) + wheel |
|---|---|---|---|
| Constant | Tempo +/-5 WPM per notch (wheel up is faster). | Tempo +/-1 WPM. | Moves the position. |
| Manual | Moves the position, like scrolling a document. | (no special effect) | Tempo +/-5 WPM, with `Shift` +/-1. This prepares the tempo for a later switch to Constant. |
| Voice-activated, or voice tracking with voice on | Short manual override: moves the position, and voice is paused for `manualOverrideMs` (2 s by default). | | The modifier has no effect; the wheel still moves the position. |

When the tempo changes, the prompter briefly shows a subtle "140 WPM" label at the bottom center. Touchpads are accumulated: one full notch of distance produces one step.

## Global shortcuts (system-wide)

These work even when another app has focus (for example OBS, slides or a video call). They are registered by the Rust side with `tauri-plugin-global-shortcut`.

| Key | Effect |
|---|---|
| `Ctrl+Shift+F6` | Voice on or off. |
| `Ctrl+Shift+F7` | Play or pause. |
| `Ctrl+Shift+F8` | -5 WPM. |
| `Ctrl+Shift+F9` | +5 WPM. |
| `Ctrl+Shift+F10` | Paragraph back. |
| `Ctrl+Shift+F11` | Paragraph forward. |

The global shortcuts are on function keys because the earlier combinations collided with common shortcuts such as "paste as plain text" (`Ctrl+Shift+V`), input methods and text selection in editors. `Ctrl+Alt` (which is AltGr on German keyboards), `Ctrl+Shift+Esc` and `F12` are deliberately avoided. Whether the F-key combinations are free on every system is **unverified**.

If another application already owns a combination, zero-teleprompter logs a warning (target `zt::hotkeys`, message "global shortcut unavailable") and keeps running without it. Under Wayland, global hotkeys may not work at all, depending on the compositor. The list shown in the app (`GLOBAL_HOTKEYS` in `hotkeys.ts`) must be kept in sync by hand with `hotkeys.rs`.
