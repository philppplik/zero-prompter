# IPC: Commands and Events

The bridge between the frontend and Rust uses Tauri `invoke` (frontend to Rust) and `listen` (Rust to frontend). On the frontend only `apps/desktop/src/lib/bridge.ts` knows the names. Sources: `apps/desktop/src-tauri/src/{commands,lib,appcore,voice,models,diag,hotkeys}.rs`.

Conventions:

- Argument names are camelCase in JS; Tauri maps them to the snake_case Rust parameters (for example `scriptVersion` -> `script_version`).
- Errors are returned as plain `string` (the Rust `Result<T, String>` error).
- Two windows exist (`tauri.conf.json`): `control` (operator UI, `index.html`) and `talent` (render-only output, `talent.html`, hidden at start). Both subscribe to the same events.
- Every state mutation goes through `appcore::dispatch`, the single mutation path shared by commands, global hotkeys, the tick loop and the voice pipeline.
- The Content Security Policy allows `connect-src ipc: http://ipc.localhost` only.

## Tauri commands

All 30 commands registered in `invoke_handler` (`lib.rs`):

| Command | Arguments | Returns | Behavior |
|---|---|---|---|
| `dispatch` | `cmd: Command` | `void` | The mutation path, see [Command](#command). |
| `get_state` | none | `AppState` | Full state snapshot (used when a window starts). |
| `import_file` | `path` | `string` | Accepts extensions `txt`, `md`, `markdown`, `text` (case-insensitive); anything else is an error. Reads via `persist::read_text_file` (limit `MAX_SCRIPT_BYTES` = 8 MiB), dispatches `loadScript`, returns the text. |
| `save_script` | `id`, `md` | `ScriptMeta` | Writes `<app_data>/scripts/<id>.md`. `id` must satisfy `persist::valid_id`. |
| `load_script` | `id` | `string` | Loads the file and dispatches `loadScript`. |
| `list_scripts` | none | `ScriptMeta[]` | Script library listing. |
| `delete_script` | `id` | `void` | Removes `<id>.md`. |
| `report_layout` | `scriptVersion`, `lines` | `bool` | Stores the talent renderer's line count; returns `false` (and ignores it) if `scriptVersion` is stale. Echoed as `layoutLines` in `scroll://frame`. |
| `get_ui_prefs` | none | `Partial<UiPrefs>` | Raw `ui-prefs.json` content. |
| `set_ui_prefs` | `prefs` | `void` | Saves and emits `prefs://changed` to all windows. |
| `list_monitors` | none | `MonitorInfo[]` | `index`, `name`, `x`, `y`, `width`, `height`, `scale`, `primary`. Requires the `control` window. |
| `open_talent` | `monitor: number or null`, `fullscreen: bool` | `void` | Optionally moves the talent window to monitor `index` (position +40/+40 px; leaves fullscreen first), shows it, sets fullscreen, emits `talent://visibility` (`true`), dispatches `setFullscreen`. |
| `close_talent` | none | `void` | Leaves fullscreen, hides the window, emits `talent://visibility` (`false`), dispatches `setFullscreen(false)`. |
| `set_talent_fullscreen` | `fullscreen` | `void` | Toggles window fullscreen and dispatches `setFullscreen`. |
| `talent_visible` | none | `bool` | Whether the talent window is visible. |
| `audio_devices` | none | `AudioDevice[]` | `id`, `name`, `isDefault`. Without the `cpal` feature this lists the mock source (empty). |
| `set_audio_device` | `id: string or null` | `void` | Stored in shared state; applies on the next voice pipeline start. |
| `mic_test` | `on` | `void` | Level and transcript only, no tracking (see `voice::set_mic_test`; behavior while tracking runs is not re-verified here). |
| `models_list` | none | `ModelRow[]` | See [Model Manifest](models-manifest.md#modelrow-ipc-payload). |
| `models_install` | `id` | `void` | Starts a background download thread `zt-model-dl`; progress via `models://progress`. Without the `download` feature it returns an error. |
| `models_remove` | `id` | `void` | Deletes the model directory. |
| `voice_models_get` | none | `VoiceModelPrefs` | See [Settings](settings.md#4-voice-model-preferences-voice-modelsjson). |
| `voice_models_set` | `prefs` | `void` | Persists the preferences, logs `zt::models`, restarts a running voice session. |
| `voice_models_selection` | `lang: string or null` | `Selection` | Which models would run for `lang` (default: the script language). |
| `build_info` | none | `BuildInfo` | `version`, `cpal`, `sherpa`, `download` (compile-time features), `globalShortcuts: [label, commandDebugName][]`. |
| `open_link` | `key` | `void` | Opens a fixed project link in the system browser. Accepted keys: `repo`, `notices`, `models`, `license`, `changelog`; others are rejected (`unknown link`). |
| `diag_snapshot` | none | `DiagReport` | Diagnostics snapshot plus `selection` (see [Logging](logging.md#diagnostics-snapshot)). |
| `diag_export_log` | `dest` | `number` | Writes a JSON snapshot header plus all log files to `dest`; returns bytes written. |
| `diag_log_dir` | none | `string or null` | Log directory path. |
| `diag_log_frontend` | `level`, `message` | `void` | Forwards frontend errors to the log (`zt::frontend`). `"error"` and `"warn"` map to those levels; anything else is `info`. |

`Selection` (`voice_models_selection`, camelCase): `language`, `streamingChain` (installed candidates in fallback order; the first is used), `streamingOptions` (all candidates for the language), `resync` (model id that would run, or null), `resyncOptions`, `sileroVad` (bool), `windowsSpeech` (bool; Windows speech recognition usable in this build, appended to the chain as a fallback with id `windows-speech`).

## Command

`zt_core::Command`, serialized with `#[serde(tag = "type", content = "value", rename_all = "camelCase")]`, for example `{ "type": "setWpm", "value": 150 }`.

| `type` | `value` | Effect |
|---|---|---|
| `loadScript` / `editScript` | string | Replaces the script text. Per earlier documentation: version +1, cursor to 0, and an error `ScriptLocked` during playback (controller behavior, not re-verified here). |
| `enterLive` / `enterEdit` | none | Mode switch. |
| `togglePlay` / `play` / `pause` | none | Playback control (with countdown). |
| `stop` | none | Stop and seek to 0. |
| `setWpm` | u16 | Clamped to 40-400. |
| `nudgeWpm` | i16 | Relative change. |
| `jumpParagraph` | i32 | Relative paragraph jump; triggers the manual override. |
| `seekToken` | TokenId | Absolute seek. |
| `resyncHere` | TokenId | Operator "resync here" / click on word: hard cursor set plus voice override. |
| `setMirror` | `{ h, v }` | Mirror flags. |
| `toggleMirrorH` | none | |
| `setFullscreen` | bool | State only; the window is controlled by `open_talent` / `set_talent_fullscreen`. |
| `setScrollMode` | `manual`, `constant`, `voiceActivated`, `voiceTracking` | Default is `constant`. |
| `toggleVoice` | none | Toggles voice tracking and starts or stops the pipeline. |
| `setTheme` | string | Error `UnknownTheme` for unknown ids. |
| `setFontSize` | u16 | Clamped to the token bounds (24-300). |
| `replaceSettings` | `Settings` | Sanitized on arrival. |
| `voice` | `VoiceSignal { phase, cursor, confidence, rateWps }` | Internal (voice thread). `phase` is `idle`, `listening`, `tracking`, `hold`, `lost` or `resync`. |
| `tick` | u32 (ms) | Internal (about 60 Hz tick loop). |

## Events (Rust to all windows)

| Event | Payload (camelCase) | Frequency / trigger |
|---|---|---|
| `state://patch` | `Partial<AppState>`: only the changed sub-trees among `script`, `playback`, `scroll`, `voice`, `output`, `settings` | On change (filtered; the countdown forwards a playback patch only when needed). |
| `scroll://frame` | `{ position, target, velocity, playing, scriptVersion, layoutLines }`. Token units; `velocity` in tokens/s; `layoutLines` is null until the talent window reported layout for the current script version. | At most about 60 Hz (tick loop sleeps 16 ms). |
| `voice://word` | `{ token, confidence }` | When the voice cursor moves. |
| `voice://level` | `{ db, rawDb, gainDb }` (dBFS after gain, dBFS before gain, applied gain in dB) | About 15 Hz while running (per earlier documentation); `-100` / `-100` / `0` at stop. |
| `voice://transcript` | `{ text, isFinal }` | Per partial or final hypothesis. |
| `voice://engine` | `EngineInfo { name, demo, reason, microphone, error, resync, notes[] }`. `reason` is one of `noModel`, `noSherpa`, `engineError` or null. | When the pipeline starts. |
| `voice://resync` | `{ text, ageMs, outcome }`, `outcome` is `ignored`, `noMatch`, `confirmed` or `reanchored` | After each resync decision. |
| `models://progress` | `{ id, done, total, finished, error, unverified: [path, sha256][] }` | At most every 100 ms during download, plus one final event with `finished: true`. Only in builds with the `download` feature. |
| `prefs://changed` | `Partial<UiPrefs>` | After `set_ui_prefs`. |
| `talent://visibility` | bool | After `open_talent`, `close_talent`, and when the talent window is closed with its title-bar button (the close handler hides it and emits `false`). |
| `diag://error` | `{ source, message }` | Every `report_error` call; the UI shows a toast. |

Note: the earlier German page stated that closing the talent window via the title bar emits no `talent://visibility`. The current `lib.rs` does emit it; this page follows the source.

## Global shortcuts

Registered by `hotkeys.rs` (plugin `tauri-plugin-global-shortcut`), always `Ctrl+Shift` plus a function key, so plain keys are never taken from other apps:

| Chord | Command |
|---|---|
| `Ctrl+Shift+F6` | `toggleVoice` |
| `Ctrl+Shift+F7` | `togglePlay` |
| `Ctrl+Shift+F8` | `nudgeWpm(-5)` |
| `Ctrl+Shift+F9` | `nudgeWpm(5)` |
| `Ctrl+Shift+F10` | `jumpParagraph(-1)` |
| `Ctrl+Shift+F11` | `jumpParagraph(1)` |

A shortcut that cannot be registered is logged (`zt::hotkeys`) and skipped.

## `AppState` serialization

camelCase, with exceptions where Rust has no rename (per earlier documentation, not re-verified in this pass): `Script`, `Token`, `Paragraph` (for example `is_cue`, `range: { start, end }`), `Playback` (`mode` is `"Edit"` or `"Live"`, `countdown_secs`), and `PlaybackState` tagged with `kind` (`stopped`, `countdown { remaining, ms_left }`, `playing`, `paused`, `ended`). The TypeScript mirror lives in `apps/desktop/src/lib/types.ts` and must be kept in sync.
