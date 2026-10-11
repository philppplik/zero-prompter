# Security Policy

## Supported versions

zero-teleprompter is pre-release (v0.1). Only the latest commit on `main` and the latest published release, if any, receive security fixes.

## Reporting a vulnerability

Please do **not** open a public issue for security problems.

Use GitHub's private vulnerability reporting for this repository: <https://github.com/philppplik/zero-prompter/security/advisories/new>. (Unverified: this requires the feature to be enabled in the repository settings. If the link does not work, open a public issue that asks for a private contact channel, without any technical details.)

Please include the affected version or commit, your operating system, steps to reproduce, and the impact you expect. We aim to acknowledge reports within a reasonable time; this is a volunteer project and no response-time guarantee is made.

## Scope and security-relevant design

What the code does, as far as it is reflected in the repository configuration:

- **Offline by design.** Audio is captured locally and processed by sherpa-onnx on your machine. The app has no remote-control server in v0.1.
- **Network access** is limited to downloading speech models (feature `download`, ureq, HTTPS only) from the URLs in `apps/desktop/src-tauri/resources/models.json`. URLs are pinned to a commit or release asset, and every file is verified against a SHA-256 hash from the manifest.
- **Webview hardening:** the Tauri content security policy is `default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; font-src 'self' data:; connect-src ipc: http://ipc.localhost`. Window permissions are granted through the capability files in `apps/desktop/src-tauri/capabilities/` (`control.json`, `talent.json`).
- **External links** are opened through a command with a fixed list of allowed links (`open_link`).
- **Memory safety:** `unsafe_code = "forbid"` for the workspace crates; the app crate uses `deny` because of Tauri macro expansion.
- **Supply chain:** CI runs `cargo-deny` (licenses, bans, sources, advisories) and an npm license allow-list. A few advisories are explicitly ignored in `deny.toml` with a written reason (transitive dependencies of Tauri; no untrusted input reaches them according to the notes there).

## Known limitations

- Installers are unsigned unless signing secrets are configured; verify that you downloaded from this repository's Releases page.
- The auto-updater is not implemented, so you must update manually.
- Global hotkeys are registered system-wide while the app runs.

## Out of scope

Vulnerabilities in third-party models or upstream projects (Tauri, sherpa-onnx, onnxruntime) should be reported upstream; tell us as well if this project is affected.
