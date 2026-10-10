# Crates & APIs

Workspace: `Cargo.toml` im Wurzelverzeichnis, `resolver = "2"`, Edition 2021, MSRV 1.80. Die `default-members` umfassen **nur die Kern-Crates**, deshalb baut ein einfaches `cargo test` ohne WebView- oder Systemabhängigkeiten. Die App gehört zwar zum Workspace, wird aber nur mit `--workspace` oder `-p zero-teleprompter` gebaut.

Lints: `unsafe_code = "forbid"`, `missing_debug_implementations = "warn"`, `clippy::all = "warn"`. Die App selbst nutzt `unsafe_code = "deny"`, weil die Tauri-Makros `forbid` nicht zulassen.

## zt-core

Reiner Domänenkern. Abhängigkeiten: serde, serde_json, thiserror.

| Element | Zweck |
|---|---|
| `Command` | alle Zustandsänderungen, siehe [IPC](../reference/ipc.md#command) |
| `Controller::new(tokens, settings)`, `with_embedded_tokens()`, `apply(cmd) -> Result<StatePatch, CoreError>`, `state()`, `tokens()` | wendet Commands deterministisch an |
| `StatePatch { script, playback, scroll, voice, output, settings }` | welche Teilbäume sich geändert haben |
| `CoreError` | `Transition(..)`, `ScriptLocked`, `UnknownTheme(String)` |
| `AppState { script, playback, scroll, voice, output, settings }` | `current_token()` |
| `Script::parse`, `spoken_tokens()`, `paragraph_jump()`, `word_count()`, `estimated_seconds(wpm)` | Tokenisierung nach Leerraum, Absätze bei Leerzeile, Regieanweisungen in `[...]` (`is_cue`) |
| `Playback` + `PlaybackState` (`stopped`, `countdown`, `playing`, `paused`, `ended`) + `UiMode` (`Edit`, `Live`) | Zustandsautomat Wiedergabe × Modus |
| `Settings`, `Tokens`, `ThemeSet`, `LostPolicy` | Settings mit Grenzwerten (`sanitized`). `TOKENS_JSON` bindet `docs/design/tokens.json` per `include_str!` ein |

Scroll-Logik pro Tick (`Controller::on_tick`):

- `constant`: `position += wpm/60 · dt`.
- `voiceActivated`: wie `constant`, aber nur in den Phasen `tracking` oder `listening`.
- `voiceTracking`: exponentielles Annähern an `target` mit `α = 1 − e^(−dt/300 ms)`. Bei `lost` mit `LostPolicy::ConstantSpeed` läuft es konstant weiter.
- Am Skriptende folgt `PlaybackEvent::ReachedEnd`.

## zt-align

Abgleich Sprache ↔ Skript **ohne Abhängigkeiten** (serde nur über das optionale Feature `serde`). Details unter [Aligner-Algorithmus](aligner.md).

`normalize_word`, `normalize_hypothesis`, `Lang` · `ScriptIndex::build(tokens, lang)`, `token_of`, `norm_of_token` · `align(query, reference, range, expected, &SwParams) -> Option<SwHit>` · `VoiceTracker::new(index, cfg)`, `handle(TrackerEvent, t_ms) -> TrackerOutput`, `set_index`, `set_config`, `config`, `state`, `output` · `similarity::{levenshtein_ratio, jaro, jaro_winkler, word_similarity}`.

## zt-audio

| Element | Zweck |
|---|---|
| `trait AudioSource { list_devices, start(device_id, FrameSink), stop, is_running }` | Port |
| `CpalSource` (Feature `cpal`) | Adapter für cpal 0.15. Nicht auf 0.16+ aktualisieren ohne API-Anpassung (0.18 ist inkompatibel) |
| `MockSource::new(samples, chunk)` | Tests |
| `Resampler::new` / `to_16k`, `process`, `reset`, `downmix_to_mono` | Tiefpass mit Blackman-gefenstertem Sinc, danach lineare Interpolation auf 16 kHz mono |
| `rms_dbfs(frame)`, `TARGET_RATE = 16_000` | Pegel |

## zt-stt

| Element | Zweck |
|---|---|
| `trait SpeechEngine: Send { name, accept_waveform(&[f32]), poll() -> Vec<SttEvent>, set_hotwords, reset }` | Port |
| `SttEvent::{Partial(String), Final(String), Vad(bool)}` | Ereignisse |
| `MockEngine::new(events)`, `MockEngine::reading(text, start_s, wps)` | deterministische Engine für Tests und Demo |
| `SherpaEngine::new(SherpaConfig)`, `SherpaConfig::zipformer(dir, name)` (Feature `sherpa`) | gegen sherpa-onnx 1.13.8 (docs.rs) geschrieben. `SherpaConfig` hat neu `bpe_vocab`, `modeling_unit`, `max_active_paths` (Standard 4). Hotwords gehen über `hotwords_buf`, der Recognizer wird dafür neu erzeugt |
| `build_hotwords(words, min_len, max, score)` | seltene, lange Wörter als Hotwords. Die App ruft `(…, 5, 200, 1.5)` auf |

## zt-models

Abhängigkeiten: serde, serde_json, thiserror, sha2. **Kein Netzwerkcode.**

| Element | Zweck |
|---|---|
| `Manifest::parse` (validiert: nur HTTPS, sichere relative Pfade, SHA-256-Format), `get`, `for_language`, `SCHEMA_VERSION = 1` | Modellkatalog |
| `ModelEntry` (`id`, `name`, `languages`, `tier`, `license`, `attribution`, `engine`, `files`), `total_size`, `file_by_role`, `requires_attribution` | Eintrag |
| `ModelStore::new(root)`, `model_dir`, `file_path`, `partial_path`, `status` (mit Hash), `quick_status` (nur Größe), `finalize_download` | Installationslayout |
| `trait Downloader { download(url, dest_part, expected_size, progress) }`, `install(store, entry, dl, progress)` | Port. Der HTTP-Adapter `UreqDownloader` lebt in der App |
| `sha256_file`, `sha256_reader`, `verify_file` | Integrität |

## zt-testkit (`publish = false`)

Nur Testhilfen: `ControllerBuilder`, `Speaker` (Tracker mit expliziter Uhr), `SpeechTimeline` (lesbare MockEngine-Zeitleisten), `Pipeline` (MockEngine → Tracker → Controller in 50-ms-Schritten), `fixtures::SCENARIOS`. Siehe [Tests](testing.md).

## zero-teleprompter (App, `apps/desktop/src-tauri`)

Lib-Name `zero_teleprompter_lib`. Module:

| Modul | Aufgabe |
|---|---|
| `appcore` | `AppCore`, `Shared`, `dispatch`, Tick-Schleife |
| `voice` | Voice-Thread |
| `commands` | Tauri-Commands |
| `hotkeys` | globale Hotkeys |
| `persist` | Persistenz |
| `models` | Manifest, Installation, Engine-Auswahl |
