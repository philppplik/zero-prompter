# IPC: Commands & Events

Die Brücke zwischen Frontend und Rust verläuft über Tauri `invoke` und `listen`. Auf Frontend-Seite kennt nur `apps/desktop/src/lib/bridge.ts` die Namen. Die Argumentnamen sind auf JS-Seite camelCase, Tauri bildet sie auf die snake_case-Parameter in Rust ab (z. B. `scriptVersion` → `script_version`). Fehler kommen als `string` zurück.

## Tauri-Commands

| Command | Argumente | Rückgabe | Wirkung |
|---|---|---|---|
| `dispatch` | `cmd: Command` | `void` | einziger Mutationspfad, siehe unten |
| `get_state` | – | `AppState` | vollständiger Zustand (beim Start eines Fensters) |
| `import_file` | `path` | `string` | `.txt`/`.md`/`.markdown`/`.text` ≤ 8 MB, wird als `LoadScript` geladen |
| `save_script` | `id`, `md` | `ScriptMeta` | `<data>/scripts/<id>.md`, `id` muss `[A-Za-z0-9_-]{1,64}` entsprechen |
| `load_script` | `id` | `string` | lädt und sendet `LoadScript` |
| `list_scripts` | – | `ScriptMeta[]` | absteigend nach Änderungsdatum |
| `delete_script` | `id` | `void` | |
| `report_layout` | `scriptVersion`, `lines` | `bool` | `false` bei veralteter Version. Wird in v0.1 nur gespeichert |
| `get_ui_prefs` | – | `Partial<UiPrefs>` | |
| `set_ui_prefs` | `prefs` | `void` | speichert und sendet `prefs://changed` |
| `list_monitors` | – | `MonitorInfo[]` | `index, name, x, y, width, height, scale, primary` |
| `open_talent` | `monitor: number\|null`, `fullscreen` | `void` | zeigt das Talent-Fenster, sendet `talent://visibility` und `SetFullscreen` |
| `close_talent` | – | `void` | blendet das Fenster aus |
| `set_talent_fullscreen` | `fullscreen` | `void` | |
| `talent_visible` | – | `bool` | |
| `audio_devices` | – | `AudioDevice[]` | `id, name, isDefault`. Ohne `cpal` eine leere Liste |
| `set_audio_device` | `id: string\|null` | `void` | gilt ab dem nächsten Start der Voice-Pipeline |
| `mic_test` | `on` | `void` | nur Pegel und Transkript. Wird ignoriert, solange Tracking läuft |
| `models_list` | – | `ModelRow[]` | `entry, status (installed\|notInstalled\|incomplete), missing, totalSize, requiresAttribution` |
| `models_install` | `id` | `void` | startet den Download im Hintergrund. Ohne `download` ein Fehler |
| `models_remove` | `id` | `void` | löscht das Modellverzeichnis |
| `build_info` | – | `{ version, cpal, sherpa, download, globalShortcuts: [label, cmd][] }` | |

## Command

`zt_core::Command`, serialisiert mit `#[serde(tag = "type", content = "value", rename_all = "camelCase")]`:

| `type` | `value` | Wirkung |
|---|---|---|
| `loadScript` / `editScript` | string | Text ersetzen, Version +1, Cursor auf 0. Fehler `ScriptLocked` während der Wiedergabe |
| `enterLive` / `enterEdit` | – | Moduswechsel (Edit nur aus Stopped oder Paused) |
| `togglePlay` / `play` / `pause` | – | mit Countdown |
| `stop` | – | Stop und Seek auf 0 |
| `setWpm` | u16 | auf 40–400 begrenzt |
| `nudgeWpm` | i16 | relativ |
| `jumpParagraph` | i32 | relativ, begrenzt. Löst den manuellen Override aus |
| `seekToken` / `resyncHere` | TokenId | absolut. Löst den manuellen Override aus |
| `setMirror` | `{ h, v }` | |
| `toggleMirrorH` | – | |
| `setFullscreen` | bool | nur Zustand. Das Fenster steuern `open_talent` und `set_talent_fullscreen` |
| `setScrollMode` | `manual` \| `constant` \| `voiceActivated` \| `voiceTracking` | |
| `toggleVoice` | – | setzt den Modus auf `voiceTracking` bzw. `constant` und startet oder stoppt die Pipeline |
| `setTheme` | string | Fehler `UnknownTheme` |
| `setFontSize` | u16 | auf 24–300 begrenzt |
| `replaceSettings` | `Settings` | wird begrenzt (`sanitized`) |
| `voice` | `VoiceSignal { phase, cursor, confidence, rateWps }` | **intern** (Voice-Thread). Nicht in `types.ts` |
| `tick` | u32 ms | **intern** (Tick-Schleife). Nicht in `types.ts` |

## Events (Rust → alle Fenster)

| Event | Payload | Frequenz |
|---|---|---|
| `state://patch` | `Partial<AppState>`: nur die geänderten Teilbäume `script`, `playback`, `scroll`, `voice`, `output`, `settings` | bei jeder Änderung (gefiltert, siehe [Datenfluss](../dev/dataflow.md#2-zeit)) |
| `scroll://frame` | `{ position, target, velocity, playing, scriptVersion }` (Token-Einheiten, velocity in Tokens/s) | ≤ ~60 Hz |
| `voice://word` | `{ token, confidence }` | wenn der Voice-Cursor sich bewegt |
| `voice://level` | `{ db }` (dBFS, −100 beim Stoppen) | ~15 Hz |
| `voice://transcript` | `{ text, isFinal }` | pro Partial oder Final der Engine |
| `voice://engine` | `{ name, demo, microphone, error }` | beim Start der Pipeline |
| `models://progress` | `{ id, done, total, finished, error }` | ≤ 10 Hz und einmal mit `finished` |
| `prefs://changed` | `Partial<UiPrefs>` | nach `set_ui_prefs` |
| `talent://visibility` | bool | `open_talent` / `close_talent` |

Hinweis: Wird das Talent-Fenster über die Titelleiste geschlossen, sendet die App **kein** `talent://visibility` (nur `SetFullscreen(false)`). Das Frontend fragt den Status gegebenenfalls über `talent_visible` ab.

## Serialisierung von `AppState`

camelCase mit diesen Ausnahmen (kein Rename in Rust):
- `Script`, `Token`, `Paragraph`: z. B. `is_cue`, `range: { start, end }`
- `Playback`: `mode` (`"Edit"` \| `"Live"`), `countdown_secs`
- `PlaybackState` ist getaggt mit `kind`: `stopped`, `countdown { remaining, ms_left }`, `playing`, `paused`, `ended`

Die TypeScript-Spiegelung steht in `apps/desktop/src/lib/types.ts` und muss synchron gehalten werden.
