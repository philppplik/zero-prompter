# Changelog

All notable changes. Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), versioning follows [SemVer](https://semver.org/).

> The previous German text is kept as [CHANGELOG.de.md](CHANGELOG.de.md).

## [Unreleased]

## [0.2.0] – 2026-10-13

### Fixed (2026-10-13, smoother scrolling)
- **Smoother scrolling in all modes** (report: judder on Windows 11 ARM64 / Snapdragon). The frontend now owns the clock: each rAF frame integrates the speed with the real frame time (identical distance at 30/60/90/120/144 Hz); Rust positions only feed a capped, forward-only correction instead of a re-extrapolated target per IPC frame. Constant / voice-activated: WPM/60 with eased speed changes (mouse wheel). Voice tracking: speaking rate estimated from the cursor updates, glide at that rate, no stop-and-go between words, no backward jitter, smooth stop on hold. Seeks glide with a critically damped spring.
- Rendering: one sub-pixel `translate3d` layer (no pixel rounding), no per-frame layout reads, read-dimming overlay moved by transform only, marker "soft focus" as a static veil instead of `backdrop-filter`, HUD without backdrop blur.
- Rust tick loop: sub-millisecond remainders are carried (timer truncation made the core clock run slow and uneven); scroll frames report the exact nominal speed in clock modes.

### Added (2026-10-13)
- Diagnostics: **scroll smoothness** (rAF frame time p50/p95/max, dropped frames, position jitter in px). The frame-time overlay shows the same values in the talent window.

### Changed (2026-10-13, settings de-duplication)
- The reading-marker **color** is set only under Settings > Lesemarker; **text dimming** (read / unread) only under Settings > Darstellung. Migration on first start: a marker color from Darstellung moves into `marker.color` (if unset), a marker read-dimming value moves into `readOpacity` (if unset). `marker.dimRead` is legacy (still read, no longer written when empty).

## [0.1.0] – 2026-10-11

> Historical note: this entry was written before the first CI builds. See [Project Status](docs/book/src/status.md) for the current state.

### Removed (2026-10-11, license-clean model stack)
- **Kroko DE/EN** (`zipformer-de-kroko-2025-08-06`, `zipformer-en-kroko-2025-08-06`, Banafo, CC-BY-SA community license with a separate commercial license) removed from the manifest, the default/fallback order, CI `stt-smoke` and the tests. The model stack is now: Nemotron 3.5 Streaming (OpenMDW-1.1; 320 ms default, 160/80 ms optional), Zipformer EN 20M (Apache-2.0, light English option, now `defaultRank 1`), Parakeet TDT 0.6B v3 (CC-BY-4.0, resync), Silero VAD (MIT), Windows speech recognition (OS).
- Existing installations: Kroko folders are ignored (can be deleted by hand), and a saved Kroko choice falls back to Nemotron or the default chain. There is no light German model any more: German without Nemotron uses Windows speech recognition on Windows instead of an English model, otherwise demo mode with "Load model".

### Added (2026-10-11, Windows speech recognition)
- **Windows speech recognition as a fallback engine** (ADR-008, Windows only, feature `winspeech`, enabled in the Windows release builds): `Windows.Media.SpeechRecognition` with dictation (partials via `HypothesisGenerated`, finals via `ResultGenerated`, VAD via `StateChanged`). Without online speech recognition, an offline script-word mode (list constraint) runs instead. The engine captures audio itself (new trait method `SpeechEngine::self_capturing`) and runs on its own MTA thread.
- Fallback chain: models (Nemotron/20M) -> Windows speech recognition -> error state or demo. "Windows Speech Recognition (System)" can also be selected explicitly in the model picker, with no download.
- Precise error hints: language pack missing, online speech recognition off (`0x80045509`), microphone denied, each with the path and an `ms-settings:` link. Logging under `zt::engine` with initialization time. Latency is measured as speech onset -> partial.
- CI job `winspeech` (windows-latest + windows-11-arm): build/clippy/tests with the feature and an initialization probe (`examples/winspeech_probe`).

### Added / Changed (2026-10-11, Nemotron voice tracking)
- **NVIDIA Nemotron 3.5 ASR Streaming 0.6B int8** (OpenMDW-1.1) is the default streaming model for German and English, in the **320 ms** variant (682 MB). 80 ms and 160 ms are selectable. Kroko DE/EN and Zipformer EN 20M remain selectable and are used while Nemotron is not installed. The language is set per stream (`de`/`en`, otherwise `auto`); output is cased and punctuated. Nemotron has no hotwords (sherpa-onnx 1.13.8: `greedy_search` only).
- Reason for the switch: Kroko only delivers new words about every 1.3 s because the model decodes in 1.28 s blocks (`decode_chunk_len = 128`), not because of our code. Nemotron 320 ms: about every 0.3 s (CI measurement, table in `docs/research/MODELS.md` and ADR-007).
- Warning when the speech model runs slower than real time on the machine (RTF > 0.9 over 10 s), with a recommendation to use a smaller model.
- CI `stt-smoke`: Nemotron 160/320 ms in addition to Kroko and 20M on Ubuntu, Windows x64 and Windows ARM64; comparison table (load time, RTF, chunk latency p95, partial cadence, time to first word, peak RAM) in the log and the job summary.

### Fixed (2026-10-11, latency metric)
- "Speech onset -> first hypothesis" showed nonsensical values (p95 46 s): the speech onset was a fixed level after the AGC, which raised the noise above it on a quiet microphone, and it was only reset at the end of a sentence. It is now relative to the continuously estimated noise floor (+12 dB, 60 ms), restarting after 0.5 s of silence.

### Added (2026-10-11, typography and mouse wheel)
- **Fonts:** 12 locally bundled prompter fonts (OFL-1.1, offline, mostly variable): Atkinson Hyperlegible Next, Inter, Lexend, Source Sans 3, Open Sans, Roboto Flex, IBM Plex Sans, Source Serif 4, Literata, Merriweather, OpenDyslexic, JetBrains Mono, plus "System font..." with a free-form name. `serif` is now Source Serif 4 instead of Georgia; the talent window now loads all fonts (JetBrains Mono was missing before).
- **Typography** (core settings, `sanitized()` + tests): font weight 100-900 (bound to the `wght` axis, falling back to available weights), letter, word and paragraph spacing, alignment (left/centered/justified), UPPERCASE. Live preview in the Appearance tab; the prompter re-measures the layout map on every typography change and after fonts finish loading. The theme line height is now treated as a minimum instead of overriding the slider.
- **Mouse wheel:** Constant = speed +/-5 WPM per notch (`Shift` +/-1) with a short "140 WPM" overlay; Manual = position; `Ctrl`/`Alt` swaps position and speed. Voice mode unchanged (short manual override). Now also works over the talent window. Documented in `shortcuts.md` and in the `?` overlay.
- **Sample script** about ten times longer (~1,250 words): numbers, names, technical terms, short/long sentences, fast-speech paragraphs, repetitions and stage directions for testing voice tracking.

### Added (2026-10-11, reading marker)
- Fully customizable reading marker (Settings -> **Reading marker**): nine styles (line, band, arrows, brackets, focus window, gradient, word/sentence underline, side bar, no marker) with mini preview tiles and a live preview; color with theme presets, opacity, thickness/height, position, dimming of read/unread text, glow, blur, animation (glide/pulse, respects "reduce motion").
- New core setting `Settings.marker` (`zt_core::marker::MarkerSettings`, sanitized + tests); the former UI pref `markerStyle` is migrated once. Rendering in `ReadingMarker.svelte`, GPU-friendly (transform/opacity).

### Changed (2026-10-11, windows and navigation)
- Windows/Linux: control window without the system title bar (`decorations: false`, `shadow: true`, giving rounded corners and a shadow via DWM on Windows 11). Custom window controls at the top right (minimize, maximize/restore, close = quit app); the top bar is a drag area, double-click maximizes. macOS keeps the native traffic-light buttons (`tauri.macos.conf.json`, `titleBarStyle: Overlay`).
- Scripts drawer: the toggle now sits in a narrow animated strip on the left instead of the top action bar (`Ctrl+B` unchanged).
- Capabilities `control`: `core:window:allow-minimize`, `allow-toggle-maximize`, `allow-internal-toggle-maximize`, `allow-is-maximized`, `allow-start-dragging`.

### Fixed (2026-10-11, microphone level)
- Very quiet microphone on Surface/Snapdragon (microphone array, 48 kHz, 2 channels, -85 to -100 dBFS): the downmix averaged all channels; an empty channel cost 6 dB, and out-of-phase channels cancelled each other. New: channel selection **Auto** (loudest channel, smoothed, 3 dB hysteresis), "Mix all" or a fixed channel (setting "Microphone channel").
- Digital preamplification: **Microphone gain** Auto (AGC to about -20 dBFS speech, max +40 dB, adapts only above -75 dBFS, soft limiter against clipping) or manual (-20 to +40 dB). Takes effect live, also while voice tracking is running.
- Diagnostics: raw level before gain, current gain, level per channel and active channel (Diagnostics tab and `zt::audio` log). Warning "Microphone very quiet (< -50 dBFS while speaking)" with a hint about the Windows input volume.
- 48k->16k resampler checked: scaling and filter gain are correct (test: a 1 kHz sine keeps its amplitude within +/-1 dB).

### Added (2026-10-11)
- Settings -> **About**: version (`getVersion`), build commit, license, links to the repo, third-party notices, model licenses and changelog (command `open_link` with a fixed link list), credits.

### Fixed (2026-10-11, voice tracking does not start)
- "failed to create OnlineRecognizer" after 1 ms with Kroko DE/EN and Zipformer EN 20M: the adapter always set `modeling_unit = "bpe"`, but none of the models has a `bpe.vocab`; sherpa-onnx `OnlineModelConfig::Validate` rejects that (`bpe_vocab: '' does not exist`). `modeling_unit`/`bpe_vocab` are now only set together; for SentencePiece models, `zt-generated-bpe.vocab` is generated from `tokens.txt` so hotwords work (casing matched to the model: Kroko mixed case, LibriSpeech upper case).
- Creation now runs as a ladder `beam+hotwords` -> `beam` -> `greedy`, each step logged. Up-front check of all model files (existence, size) and a Rust re-implementation of the sherpa validation rules (`zt_stt::model_check`) for precise error messages. `ZT_STT_DEBUG=1` turns on sherpa `debug`.
- CI: new job `stt-smoke` (Ubuntu, Windows x64, Windows ARM64) downloads the models from `models.json` (SHA-checked, cached), decodes test WAVs (`cargo run -p zt-stt --example smoke --features sherpa`), checks keywords and the hotword step, and reproduces the old error (`--legacy-check`). RTF and chunk latency appear in the job summary.

### Fixed (2026-10-11, voice)
- "Syncing speech does not work, UI shows Demo": if an installed model could not be loaded, the app silently fell back to demo mode (the mock engine reads the script aloud by itself). Now: error message + log, no self-movement. If only a model for another language is installed, it is used (with a notice) instead of demo. After a model download, a running voice tracking session restarts. "Demo" now only appears without a streaming model, with a "Download model" button (ADR-007).
- Microphone watchdog: reports "no data" or "digital silence only" (Windows microphone privacy) after 3 s.
- Crashes of the voice thread are reported instead of being swallowed.

### Added (2026-10-11, diagnostics)
- Structured logging with `tracing` to a daily rotating file in the app log folder (`ZT_LOG` for the filter), with events for audio, VAD, STT partials/finals, tracker, engine selection, model downloads, frontend errors.
- Settings -> **Diagnostics**: level, engine/model, audio format, frames/s, tick rate, tracker state, latencies p50/p95 (speech onset->hypothesis, audio->hypothesis, hypothesis->cursor, audio->cursor), last errors, log export, microphone test.

### Changed (2026-10-11, latency)
- Scroll ease in voice tracking 300 -> 150 ms.
- Tracker near path: the first matching word after a seek/jump moves the cursor immediately (idea from ShayneP/local-teleprompter, MIT). Rationale and latency chain in ADR-007.

### Fixed (2026-10-11)
- Windows (x64/ARM64): the app did not start on fresh systems ("VCRUNTIME140.dll was not found"). The MSVC runtime is now linked statically (`.cargo/config.toml`, `+crt-static`); CI/release verify via `dumpbin /dependents` (`scripts/check-crt.ps1`) that no `VCRUNTIME*/MSVCP*` DLL is imported.

### Changed (2026-10-11)
- Release builds on all platforms include real voice tracking (feature `sherpa`, sherpa-onnx 1.13.8, statically linked including onnxruntime, so no extra DLLs/dylibs/.so). Without an installed model, the app still starts in demo mode.
- `repository` URL: https://github.com/philppplik/zero-prompter

### Added (2026-10-10, release preparation)
- Silero VAD (MIT) as model `silero-vad` in the catalog. It is downloaded automatically with every speech model and used for resync segmentation; without it, the app segments by level (RMS). The Inspector shows "Segmentation: Silero VAD active".
- All model URLs are pinned to commits or release assets, and all files have a SHA-256 (including `tokens.txt`).
- `Cargo.lock` and `package-lock.json` are checked in (workflow `lockfiles.yml`). CI uses `npm ci` and `--locked` (MSRV).
- Release: Windows ARM64 (`aarch64-pc-windows-msvc`) and Store installer `*_store-setup.exe` with offline WebView2 (`tauri.microsoftstore.conf.json`), guide in `docs/book/src/dev/ms-store.md`.
- Actions updated (checkout/setup-node/upload-artifact v7, download-artifact v8, tauri-action v1, Node 22), no more Node 20 warnings. `thiserror` 2.

### Added
- Hybrid speech recognition (ADR-006): a streaming model per language (default Kroko DE/EN, fallback Zipformer EN 20M) with an automatic fallback chain, plus optional offline resync with Parakeet TDT 0.6B v3 (`OfflineRecognizer`, `nemo_transducer`) on its own thread that re-anchors the tracker. Inspector: model choice per language and a resync switch.
- Manifest v2: `modelType`, `roles`, `defaultRank`, optional `sha256`; files without a hash are hashed after download and shown as unverified.
- Cargo workspace with the pure core crates `zt-core`, `zt-align`, `zt-audio`, `zt-stt`, `zt-models` and the test crate `zt-testkit` (ADR-005, hexagonal).
- `zt-core`: script model (paragraphs, stage directions in `[...]`), playback x mode state machine with countdown, scroll modes Manual, Constant, Voice-activated and Voice tracking, lost policy, manual override, settings and themes from `docs/design/tokens.json`.
- `zt-align`: normalization for German and English (umlaut folding, numbers to words, filler words), word similarity (Levenshtein, Jaro-Winkler), windowed Smith-Waterman, `VoiceTracker` with hysteresis and the states idle, listening, tracking, hold, lost, resync.
- `zt-audio`: `AudioSource` port, cpal adapter (feature `cpal`), own resampler to 16 kHz mono.
- `zt-stt`: `SpeechEngine` port, `MockEngine` (demo and tests), sherpa-onnx adapter (feature `sherpa`, unverified), hotword selection.
- `zt-models`: manifest with validation (HTTPS only, safe paths), SHA-256 check, install layout, downloader port.
- Tauri 2 app with control window and talent window, 22 commands, 9 events, ~60 Hz tick loop, voice thread, global hotkeys (`Ctrl+Shift+...`), atomic persistence, model download via ureq with resume.
- Svelte 5 frontend: sidebar and library, editor, prompter preview with word highlighting, Inspector, transport, command palette, shortcut overview, onboarding, drag-and-drop import.
- Tests: unit, property (proptest), table and scenario tests (Rust), Vitest (frontend).
- CI (fmt, clippy, tests on three operating systems, MSRV 1.80, adapters, app, licenses) and release workflow (draft release with installers).
- Documentation: README, mdBook under `docs/book/`, CONTRIBUTING, THIRD_PARTY_NOTICES, THIRD_PARTY_MODELS.

### Changed (revision 2026-10-10, still not compiled)
- `zt-stt`: sherpa-onnx adapter rewritten against **sherpa-onnx =1.13.8** (backed by docs.rs). `get_result` returns `Option`, no more `unsafe impl Send`, all config values explicit (including `max_active_paths = 4`), hotwords via `hotwords_buf`, new `SherpaConfig` fields `bpe_vocab`, `modeling_unit`, `max_active_paths`. `set_hotwords` reports `Unsupported` when `bpe.vocab` is missing for BPE models.
- Global hotkeys moved to `Ctrl+Shift+F6` ... `F11` (voice, play/pause, -/+5 WPM, paragraph back/forward). The old combinations collided with, among others, "paste as text" (`Ctrl+Shift+V`).
- `zt-audio`: deliberately stays on cpal 0.15 (0.18.2 is API-incompatible).
- Model catalog `resources/models.json` with 5 models, see `docs/research/MODELS.md`.
- Formatting: `rustfmt.toml`, Prettier configuration and `format`/`format:check` scripts. `zt-models` tests moved to `src/tests.rs`. `Inspector.svelte` split into `HotkeyEditor.svelte` and `ModelList.svelte`; keyboard capture is only attached to the window while "Shortcuts" is expanded.
- `scroll://frame` now contains `layoutLines` (only for a matching script version; the frontend does not use it yet).
- ADR-001 to ADR-004 as separate files in `docs/adr/`. Legal assessment in `docs/legal/LEGAL.md`.

### Fixed
- Focus line: a single range of 5-70 % in Rust and frontend (`FOCUS_LINE_MIN/MAX`); NaN/infinity falls back to the default.
- Talent window: closing via the title bar leaves fullscreen and sends `talent://visibility = false`.
- `report_layout` is evaluated instead of being discarded.
- The voice cursor is clamped to the end of the script.
- Empty script: playback ends immediately at position 0, in every scroll mode.
- Play during the countdown is ignored instead of being rejected as an error.
- In the tick in which the countdown ends, only the remaining time is scrolled (`Playback::tick_overflow`).

### Known issues
- Model manifest not yet loadable: SHA-256 values for `tokens.txt` missing, URLs not pinned to commits. Without `sherpa` and without a model, voice tracking runs in demo mode.
- Patent situation US 9,953,646 not reviewed by a lawyer, see `docs/legal/LEGAL.md`.
- `cargo fmt --check`, `clippy -D warnings`, Prettier never run.
- No `package-lock.json`. Icons are generated via `npx tauri icon`.
- Remote control (WebSocket/PWA), MIDI/HID/gamepad, overlay, `.docx`, updater and signing follow from v0.2.

> Note: several entries in "Known issues" and "Added" above predate later entries (for example the lockfiles and pinned model hashes were addressed on 2026-10-10 in the release-preparation entry). Entries are kept as originally written.
