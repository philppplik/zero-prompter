# Project status

**Version 0.1.0, pre-release.** This page describes what is in the repository and what is still open. It replaces an earlier version of this page that said the code had never been compiled; that is no longer the case according to the repository's CI configuration and changelog.

> A note on evidence. The statements below come from the repository contents (`CHANGELOG.md`, `README.md`, `.github/workflows/*.yml`, source files). This page does not report the result of any particular CI run, and the documentation author did not run the app. Where something depends on a real system, it is marked **(unverified)**.

## What the repository contains

| Area | Status |
|---|---|
| Core crates `zt-core`, `zt-align`, `zt-models`, `zt-testkit` | Implemented with unit, property-based and scenario tests. |
| `zt-audio` (microphone capture via `cpal`, gain, channel selection, resampling to 16 kHz mono) | Implemented. Deliberately pinned to cpal 0.15 because cpal 0.18 has an incompatible API. Some cpal 0.15 API details are marked `// VERIFY` in the source. |
| `zt-stt` (speech engines) | sherpa-onnx adapters (pinned to `=1.13.8`), hybrid orchestrator (streaming model plus optional offline resync), Silero VAD segmentation, a mock engine for demo mode and tests, and a Windows speech engine (feature `winspeech`). |
| Desktop app (Tauri 2, Svelte 5) | Control window, talent window, command palette, settings modal, onboarding, diagnostics, model download. |
| Model manifest | `apps/desktop/src-tauri/resources/models.json` (schema version 2): 8 entries, all URLs pinned to commits or release assets and every file carries a SHA-256 hash. |
| Continuous integration | `.github/workflows/ci.yml` defines jobs for the Rust core (three operating systems), MSRV 1.80, extended property tests, adapters, a speech-recognition smoke test with real models, the frontend (check, test, build), the Tauri app, license checks and the Windows speech fallback. |
| Release workflow | `.github/workflows/release.yml` builds installers when a `v*` tag is pushed and attaches them to a **draft** GitHub release. |
| Documentation | This manual, the ADRs, research notes in `docs/research/`, and the legal notes in `docs/legal/LEGAL.md`. Parts of the project documentation are still German. |

## Known limitations and open items

- **No published installer is guaranteed.** Installers exist only as drafts produced by the release workflow. See [Installation](user/installation.md).
- **No code signing or notarization by default.** Signing happens only if the repository has the signing secrets configured. Expect SmartScreen (Windows) and Gatekeeper (macOS) warnings.
- **No auto-updater.**
- **Not implemented yet:** remote control (phone, WebSocket, MIDI, HID, gamepad), overlay mode, `.docx` import, custom themes. See [Remote control](user/remote.md).
- **Unverified external APIs.** Parts of cpal 0.15, `ureq` 2 (model download) and `tauri-plugin-global-shortcut` are marked `// VERIFY` in the source. Find them with `rg "VERIFY"`.
- **Global hotkeys.** Whether `Ctrl+Shift+F6` to `F11` is free on every system is **(unverified)**. They may not work under Wayland, depending on the compositor.
- **Licensing is not final for every model.** The license of the German Kroko model is flagged as unresolved in [`docs/legal/LEGAL.md`](../../legal/LEGAL.md), and a patent note needs legal review. This is not legal advice.
- **Hardware requirements.** The Nemotron 160 ms and 80 ms variants need a fast CPU. The app warns when the speech model decodes slower than real time.
- **Voice tracking quality** depends on microphone, room, script and model. Latency figures in this manual come from CI measurements in `docs/research/MODELS.md`, not guarantees.
- **Screenshots** in the README are placeholders and not yet added.
- **Monitor placement of the talent window** is done by moving the window and then toggling fullscreen. Some window managers ignore this. See [Troubleshooting](user/faq.md).

## Where to look for details

| Topic | Page |
|---|---|
| Changes per version | `CHANGELOG.md` in the repository root (currently still German) |
| Architecture decisions | [ADR index](adr/index.md) |
| Model research and measurements | `docs/research/MODELS.md` |
| Legal notes | `docs/legal/LEGAL.md`, `THIRD_PARTY_NOTICES.md`, `THIRD_PARTY_MODELS.md` |
