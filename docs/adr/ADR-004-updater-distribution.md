# ADR-004: Updater and distribution

> German original: [de/ADR-004-updater-distribution.md](de/ADR-004-updater-distribution.md)

- **Status:** accepted (implementation from v0.2)
- **Source:** taken over from `docs/research/RESEARCH.md` §4 (2026-10-10)

## Decision

- **Updater:** `tauri-plugin-updater` (minisign signatures) via `tauri-action`.
- **Not chosen:** cargo-dist/axoupdater, because they are poorly maintained and aimed at CLI tools.
- **Installer:** under 15 MB without models; models are downloaded at runtime.

## Addendum 2026-10-10: code signing

Research on this is in `docs/legal/LEGAL.md`: according to the report, Azure Artifact Signing is available to EU organisations (validation prerequisites and prices are unverified); OV/EV certificates are only available on hardware tokens or cloud HSMs; macOS requires the Apple Developer Program, a D-U-N-S number and notarization.
