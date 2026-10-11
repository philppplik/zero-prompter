# Installation

You can install a pre-built package or build the app from source. Installers are produced by the release workflow (`.github/workflows/release.yml`) when a tag matching `v*` is pushed. They are attached to a **draft** GitHub release, so check the project's Releases page to see whether a version has been published. If there is none, [build from source](#build-from-source).

## Installers built by the release workflow

| Platform | Artifacts |
|---|---|
| Windows x64 and ARM64 | MSI and NSIS installer (per-user install, `installMode = currentUser`). Additional `*_store-setup.exe` installers with an embedded offline WebView2 are for Microsoft Store submission. |
| macOS (Apple Silicon and Intel) | DMG. Minimum macOS version 11.0. |
| Linux x64 | AppImage, `.deb` (depends on `libasound2`) and `.rpm`. |

Release builds include real speech recognition (feature `sherpa`, with sherpa-onnx linked statically, so no extra DLLs, dylibs or `.so` files are needed). Windows release builds also include the Windows speech fallback (feature `winspeech`).

On Windows the MSVC runtime is linked statically (`.cargo/config.toml`, `+crt-static`), so the app does not need the Visual C++ Redistributable. See [Troubleshooting](faq.md#the-app-does-not-start-on-windows-vcruntime140dll-was-not-found) if you still see that error.

### Unsigned installers

Code signing is not set up by default. It only happens if the signing secrets are configured in the repository, so assume downloaded installers are unsigned and your operating system will warn you. Install only files you downloaded from the project's own Releases page.

- **Windows (SmartScreen):** a "Windows protected your PC" dialog appears. Click **More info**, then **Run anyway**.
- **macOS (Gatekeeper):** an unsigned or non-notarized app is blocked on first launch. Control-click (right-click) the app, choose **Open**, then confirm **Open**. Alternatively allow it under **System Settings, Privacy & Security** ("Open Anyway"). macOS also asks for microphone permission the first time voice tracking uses the microphone.
- **Linux:** make the AppImage executable (`chmod +x zero-teleprompter*.AppImage`) and run it, or install the `.deb` or `.rpm` with your package manager.

## Build from source

### 1. Toolchain

| Tool | Version | Notes |
|---|---|---|
| Rust via [rustup](https://rustup.rs) | `stable` (pinned in `rust-toolchain.toml`, with `rustfmt` and `clippy`). Minimum supported Rust version (MSRV): 1.80. | |
| Node.js | 22 (used by CI and release builds) | npm is included. |
| Tauri 2 system dependencies | | See <https://v2.tauri.app/start/prerequisites/>. |

**Windows.** Install the Visual Studio 2022 Build Tools with the workload "Desktop development with C++" (MSVC and a Windows SDK), then run `rustup default stable-msvc`. The WebView2 Runtime is preinstalled on Windows 11; on Windows 10 you may need to install it from Microsoft.

**macOS.** Run `xcode-select --install`. The deployment target is 11.0.

**Linux (Debian or Ubuntu).** These are the packages CI installs on Ubuntu 22.04:

```bash
sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf libasound2-dev
```

### 2. Build and run

```bash
git clone https://github.com/philppplik/zero-prompter.git
cd zero-prompter/apps/desktop
npm ci                           # or: npm install
npx tauri icon app-icon.png      # generates all icons, including icon.icns for macOS
npm run tauri dev                # development mode
npm run tauri build              # installers under target/release/bundle/
```

Without extra flags the app has the features `cpal` (microphone) and `download` (model download), but **not** real speech recognition. In that case voice tracking runs in [demo mode](voice-tracking.md#which-engine-runs). For real speech recognition add the `sherpa` feature (the sherpa-onnx prebuilt libraries are downloaded by the build script on the first build):

```bash
npm run tauri -- dev --features sherpa
npm run tauri -- build --features sherpa
# Windows: add the system speech fallback
npm run tauri -- build --features sherpa,winspeech
```

| Feature | Default | Purpose |
|---|---|---|
| `cpal` | on | Microphone capture. |
| `download` | on | Model download (HTTPS only, resumes interrupted downloads). |
| `sherpa` | off | Real speech recognition with sherpa-onnx. |
| `winspeech` | off | Windows speech recognition as a fallback engine. Has no effect on other platforms. |

## Where the app stores data

Paths are provided by Tauri (`app_config_dir` and `app_data_dir`) for the identifier `dev.zeroteleprompter.app`. The exact folder for each operating system is **(unverified)**; on Windows these are typically below `%APPDATA%` and `%LOCALAPPDATA%`.

| File | Location | Content |
|---|---|---|
| `settings.json` | config folder | Core settings (theme, typography, tempo, voice behavior, hotkeys, reading marker). |
| `ui-prefs.json` | config folder | Interface preferences (font, colors, onboarding state, last script). |
| `voice-models.json` | config folder | Your speech model choice and the resync switch. |
| `scripts/<id>.md` | data folder | Your script library. |
| `models/<id>/...` | data folder | Downloaded speech models. An optional `models/manifest.json` replaces the bundled manifest. |
| `zero-teleprompter.YYYY-MM-DD.log` | log folder | Daily log files (see [Diagnostics](voice-tracking.md#diagnostics-tab-and-log-export)). |

To uninstall, remove the app and delete these folders.
