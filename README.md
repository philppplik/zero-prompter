# zero-teleprompter

**A free, open-source teleprompter for Windows, macOS and Linux that listens to you.**

zero-teleprompter runs entirely offline. With voice tracking, the script follows your voice (German and English). It can be mirrored for beam-splitter glass and sent to a second monitor. Audio never leaves your device.

[![CI](https://github.com/philppplik/zero-prompter/actions/workflows/ci.yml/badge.svg)](https://github.com/philppplik/zero-prompter/actions/workflows/ci.yml)
License: MIT OR Apache-2.0

> **Status: v0.1, pre-release.** The CI builds and tests the Rust crates and the Tauri app on Windows, macOS and Linux, and runs end-to-end speech-recognition smoke tests with real models. Installers are produced by the release workflow as a **draft** GitHub release. Some parts are still unverified or unfinished; see [Status and limitations](#status-and-limitations).

<!-- SCREENSHOT: control window (sidebar, editor/preview, inspector, transport) -> docs/book/src/img/control.png -->
<!-- SCREENSHOT: talent window, mirrored, fullscreen -> docs/book/src/img/talent.png -->
<!-- SCREENSHOT: voice status indicator (tracking / hold / lost) -> docs/book/src/img/voice.png -->
*Screenshots are placeholders and will be added.*

## Features

- **Scripts:** import `.txt` / `.md` (dialog or drag and drop). Library in a local folder. Stage directions in `[square brackets]` are shown but not expected to be spoken.
- **Scroll modes:** manual, constant speed (40-400 WPM), voice-activated (scrolls only while you speak), and voice tracking.
- **Voice tracking:** fully offline speech recognition with [sherpa-onnx](https://github.com/k2-fsa/sherpa-onnx). The script position is found with a windowed, fuzzy Smith-Waterman alignment. Tracker states: `idle`, `listening`, `tracking`, `hold`, `lost`, `resync`. Hysteresis guards against false jumps. Click a word to trigger "Resync here".
- **Hybrid recognition:** a *streaming model* provides low-latency partial results; an optional offline model (NVIDIA Parakeet TDT) is used for *resync*, driven by Silero VAD speech segmentation.
- **Windows speech fallback:** on Windows, builds with the `winspeech` feature can fall back to the built-in Windows speech recognition (`Windows.Media.SpeechRecognition`) when no model is usable (ADR-008). Availability depends on the installed speech language pack and the Windows online speech recognition setting.
- **Output:** a separate talent window, selectable monitor, fullscreen, horizontal and vertical mirroring.
- **Customization:** 6 themes from `tokens.json` (WCAG AAA, including dyslexia-friendly and low-light), 12 bundled offline fonts plus a system font option, adjustable font size, weight, spacing, alignment, nine reading-marker styles, dimming of already-read text, and custom colors.
- **Control:** keyboard first, command palette (`Ctrl+K`), remappable in-app shortcuts, system-wide global hotkeys (`Ctrl+Shift+F6` to `F11`), start countdown, mouse wheel for speed or position.
- **Microphone handling:** automatic channel selection, automatic or manual gain, a silence/no-data watchdog, and a diagnostics tab with levels, latency percentiles and log export.
- **Models:** manifest with pinned URLs and SHA-256 verification; download over HTTPS with resume (feature `download`).

Not included yet (planned for v0.2 or later): remote control from a phone / WebSocket, MIDI / HID / gamepad, overlay mode, `.docx` import, auto-updater, code signing. See the [changelog](CHANGELOG.md) and [project status](docs/book/src/status.md).

## Download and install

Installers are built by the [release workflow](.github/workflows/release.yml) when a `v*` tag is pushed, and are attached to a **draft** release. Check the [Releases page](https://github.com/philppplik/zero-prompter/releases) for published versions. If no release is published yet, [build from source](#build-from-source).

| Platform | Artifacts built by the workflow |
|---|---|
| Windows x64 and ARM64 | MSI and NSIS installer (per-user install); additional `_store` NSIS installers with offline WebView2 for Microsoft Store submission |
| macOS (Apple Silicon and Intel) | DMG; minimum macOS 11.0 |
| Linux x64 | AppImage, `.deb` (depends on `libasound2`), `.rpm` |

Release builds enable the `sherpa` feature (real speech recognition, sherpa-onnx statically linked) on all platforms, plus `winspeech` on Windows.

### Unsigned installers

Code signing is **not** set up by default. Signing only happens if the signing secrets are configured in the repository, so assume that downloaded installers are unsigned and your OS will warn you. Only install files you downloaded from this repository's Releases page.

- **Windows (SmartScreen):** "Windows protected your PC" appears. Click **More info**, then **Run anyway**.
- **macOS (Gatekeeper):** an unsigned or un-notarized app is blocked on first launch. Control-click (right-click) the app, choose **Open**, then confirm **Open**. Alternatively, allow it under **System Settings, Privacy & Security** ("Open Anyway"). macOS also asks for microphone permission on first use of voice tracking.
- **Linux:** `chmod +x zero-teleprompter*.AppImage` and run it, or install the `.deb` / `.rpm` with your package manager.

## Quickstart

1. Start the app. The **control window** opens; the **talent window** (the mirrored, fullscreen-capable prompter output) is opened on demand.
2. Import a script (`Ctrl+O` or drag a `.txt` / `.md` file onto the window), or use the built-in example script.
3. Press `Space` to play. Use `Up` / `Down` to change speed (+/-5 WPM) and `M` to mirror.
4. Press `F` to send the talent window fullscreen to the selected monitor.

See [Shortcuts](docs/book/src/user/shortcuts.md) for the full list.

### Voice-tracking quickstart

1. Open **Settings** and the model list, and install a *streaming model*. The default recommendation is **Nemotron 3.5 Streaming 0.6B int8, 320 ms** (German and English, about 682 MB). The small Silero VAD helper model is installed automatically with any speech model.
2. Choose voice tracking as the scroll mode and press `V` (or `Ctrl+Shift+F6` globally) to turn voice on.
3. Read the script aloud. The tracker moves through `listening`, `tracking`, `hold` and `lost`. If it loses your place, click the word you are at or press `R` to *resync*.
4. For problems, open **Settings, Diagnostics** (levels, engine and model, tracker state, latencies, log export).

Without a usable streaming model, voice runs in **demo mode** (a mock engine that "reads" the script on its own). Demo mode is shown explicitly in the UI and is not real speech recognition.

Details: [Voice tracking](docs/book/src/user/voice-tracking.md).

## Models

The manifest is [`apps/desktop/src-tauri/resources/models.json`](apps/desktop/src-tauri/resources/models.json). Sizes below are the sum of the files in the manifest (binary MB). Model licenses are listed in [THIRD_PARTY_MODELS.md](THIRD_PARTY_MODELS.md); some licenses carry conditions (CC-BY-SA, CC-BY, OpenMDW), so read them before redistributing.

| Model ID | Role | Languages | Size | License | Notes |
|---|---|---|---|---|---|
| `nemotron-3.5-streaming-0.6b-320ms-int8` | streaming (default) | de, en | ~651 MiB | OpenMDW-1.1 | Recommended. Partial results about every 0.3 s (CI measurement, see `docs/research/MODELS.md`) |
| `nemotron-3.5-streaming-0.6b-160ms-int8` | streaming | de, en | ~651 MiB | OpenMDW-1.1 | Faster reaction, needs a strong CPU |
| `nemotron-3.5-streaming-0.6b-80ms-int8` | streaming | de, en | ~651 MiB | OpenMDW-1.1 | Fastest reaction, needs a very strong CPU |
| `zipformer-de-kroko-2025-08-06` | streaming | de | ~68 MiB | CC-BY-SA-4.0 | Small; decodes in 1.28 s blocks, so new words arrive roughly every 1.3 s. German license status is flagged in [LEGAL.md](docs/legal/LEGAL.md) |
| `zipformer-en-kroko-2025-08-06` | streaming | en | ~68 MiB | CC-BY-SA-4.0 | Small, same block behavior |
| `zipformer-en-20m-2023-02-17` | streaming | en | ~42 MiB | Apache-2.0 | Smallest; English only |
| `parakeet-tdt-0.6b-v3-int8` | resync (offline) | de, en | ~639 MiB | CC-BY-4.0 | Offline transducer used to correct the position after segments |
| `silero-vad` | VAD helper | - | ~0.6 MiB | MIT | Installed automatically; drives segmentation for resync |
| Windows speech recognition | system fallback | system languages | none | OS component | Windows only, `winspeech` feature, selectable without download |

If Nemotron is not installed, the app uses another installed streaming model (Kroko / Zipformer) automatically. Hotwords are not available for Nemotron (the pinned sherpa-onnx 1.13.8 supports only greedy search there). All figures come from `models.json` and `CHANGELOG.md`; latency figures depend on your CPU.

## Build from source

### Prerequisites

| Tool | Version | Notes |
|---|---|---|
| Rust via [rustup](https://rustup.rs) | `stable` (`rust-toolchain.toml`), MSRV **1.80** (`workspace.package.rust-version`) | Components `rustfmt` and `clippy` are selected automatically. CI also runs the tests on 1.80 |
| Node.js | **22** (used in CI and release builds) | npm is included. The README previously said 20+; CI uses 22, so use 22 |
| Tauri 2 system dependencies | - | <https://v2.tauri.app/start/prerequisites/> |
| **Windows** | Visual Studio 2022 Build Tools, workload "Desktop development with C++" (MSVC + Windows SDK) | `rustup default stable-msvc`. WebView2 Runtime (preinstalled on Windows 11). The MSVC runtime is linked statically (`.cargo/config.toml`, `+crt-static`), so end users need no VC++ redistributable |
| **macOS** | Xcode Command Line Tools (`xcode-select --install`) | Deployment target 11.0 |
| **Linux** | `libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf libasound2-dev` | Exactly the packages CI installs on Ubuntu 22.04 |

### Steps

```bash
git clone https://github.com/philppplik/zero-prompter.git
cd zero-prompter
cargo test                       # pure core crates, no system dependencies

cd apps/desktop
npm ci                           # or: npm install
npx tauri icon app-icon.png      # generates all icons (incl. .icns)
npm run tauri dev                # run the app in development mode
```

With real speech recognition (the sherpa-onnx prebuilt libraries are downloaded by the build script on first build):

```bash
npm run tauri -- dev --features sherpa
npm run tauri -- build --features sherpa            # installers under target/release/bundle/
# Windows additionally: --features sherpa,winspeech
```

Without `--features sherpa`, voice tracking falls back to the demo mode described above. `cargo build --workspace` or `cargo test --workspace` of the app crate needs `apps/desktop/dist` (run `npm run build` first) because the frontend is embedded at compile time.

### Feature flags

App crate `zero-teleprompter` (`apps/desktop/src-tauri`):

| Feature | Default | Purpose |
|---|---|---|
| `cpal` | on | Microphone capture via cpal (forwards to `zt-audio/cpal`) |
| `download` | on | Model download via ureq (HTTPS only, resume via `Range`) |
| `sherpa` | off | Real speech recognition with sherpa-onnx (forwards to `zt-stt/sherpa`). Without it, voice runs in demo mode (`MockEngine`) |
| `winspeech` | off | Windows speech recognition as a fallback engine (forwards to `zt-stt/winspeech`); a no-op on non-Windows platforms |

`zt-audio` (`cpal`) and `zt-stt` (`sherpa`, `winspeech`) have the same feature names and are off by default.

### Environment variables

| Variable | Effect |
|---|---|
| `ZT_LOG` | Log filter for the rotating log file (tracing env-filter syntax) |
| `ZT_STT_DEBUG` | `1` turns on sherpa-onnx debug output |
| `ZT_SAFE_RENDER` | `1` sets `WEBKIT_DISABLE_DMABUF_RENDERER=1` (Linux, e.g. NVIDIA stutter in WebKitGTK) |

## Testing

```bash
cargo fmt --all
cargo clippy --all-targets
cargo test --all-targets                     # core crates incl. zt-testkit: unit, property and scenario tests
cargo test --doc

cd apps/desktop
npm run check                                # svelte-check
npm test                                     # Vitest
npm run build
cd ../..
cargo clippy -p zero-teleprompter --all-targets --features sherpa
cargo test -p zero-teleprompter              # needs apps/desktop/dist
```

CI (`.github/workflows/ci.yml`) additionally runs: MSRV tests on Rust 1.80 (`--locked`), weekly extended property tests (`PROPTEST_CASES=8192`), adapter builds (cpal, sherpa), an end-to-end STT smoke test with real models (`scripts/stt-smoke.py`, Ubuntu, Windows x64, Windows ARM64), the Windows speech probe, license checks (`cargo-deny` and an npm allow-list), and a check that the Windows executable imports no VC++ runtime DLLs (`scripts/check-crt.ps1`). See [Testing](docs/book/src/dev/testing.md).

## Project structure

```
crates/            Pure Rust core, no I/O (ADR-005, hexagonal architecture)
  zt-core          State, commands, playback, script model, settings/themes from tokens.json
  zt-align         Speech-to-script alignment (Smith-Waterman), voice state machine
  zt-audio         AudioSource port, cpal adapter (feature cpal), gain, resampling to 16 kHz mono
  zt-stt           SpeechEngine port, sherpa-onnx adapters (feature sherpa), hybrid orchestrator,
                   VAD segmentation, MockEngine, Windows speech (feature winspeech)
  zt-models        Model manifest, SHA-256, install layout, downloader port
  zt-testkit       Test-only: fixtures, scenarios, pipeline harness (publish = false)
apps/desktop/      Tauri 2 app
  src-tauri/       Adapter: commands/events, tick loop, voice thread, global hotkeys, persistence, models
  src/             Svelte 5 + TypeScript (control window index.html, talent window talent.html)
docs/              Research, concept, design, ADRs, mdBook user and developer manual
scripts/           CI helpers (stt-smoke.py, check-crt.ps1)
```

## Documentation

- **Manual (mdBook):** `docs/book/`. Build with `mdbook build docs/book`, read with `mdbook serve docs/book`.
- [Installation](docs/book/src/user/installation.md) | [Voice tracking](docs/book/src/user/voice-tracking.md) | [Shortcuts](docs/book/src/user/shortcuts.md) | [Architecture](docs/book/src/dev/architecture.md) | [Testing](docs/book/src/dev/testing.md) | [Build and release](docs/book/src/dev/build-release.md)
- Architecture decisions: [`docs/adr/`](docs/adr/) (ADR-001 to ADR-008). Models research: [`docs/research/MODELS.md`](docs/research/MODELS.md). Legal notes: [`docs/legal/LEGAL.md`](docs/legal/LEGAL.md).
- [CONTRIBUTING.md](CONTRIBUTING.md) | [SECURITY.md](SECURITY.md) | [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) | [CHANGELOG.md](CHANGELOG.md)

Parts of the documentation are still German; English versions are being added.

## Status and limitations

- Pre-release v0.1. Behavior described in the manual is behavior according to the code unless stated otherwise.
- Some external APIs are marked `// VERIFY` in the source (for example parts of cpal 0.15, ureq 2 and `tauri-plugin-global-shortcut`). Find them with `rg "VERIFY"`.
- **Licensing is not final for every model.** The German Kroko model's license is flagged as unresolved in [LEGAL.md](docs/legal/LEGAL.md), and a patent note requires legal review. This is not legal advice.
- No code signing or notarization by default; no auto-updater.
- Global hotkeys may not work under Wayland, depending on the compositor; whether `Ctrl+Shift+F6` to `F11` is free on every system is not verified.
- Nemotron 160 ms and 80 ms variants need a fast CPU. The app warns when the speech model runs slower than real time (RTF above 0.9 over 10 seconds).
- Voice tracking quality depends on microphone, room and script. Latency numbers in the docs are CI measurements, not guarantees.

## License

Dual-licensed under [Apache-2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT), at your option. Third-party software notices: [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md). Model licenses: [THIRD_PARTY_MODELS.md](THIRD_PARTY_MODELS.md).
