# ADR Index

Architecture Decision Records in MADR style. The canonical location is `docs/adr/`. German originals are kept under `docs/adr/de/`.

| No. | Title | Status | Location |
|---|---|---|---|
| ADR-001 | UI framework: Tauri 2 + Svelte 5 | accepted | `docs/adr/ADR-001-ui-framework.md` |
| ADR-002 | Native audio via cpal instead of getUserMedia | accepted | `docs/adr/ADR-002-native-audio-cpal.md` |
| ADR-003 | Project license MIT OR Apache-2.0 | accepted | `docs/adr/ADR-003-license.md` |
| ADR-004 | Updater and distribution | accepted (implementation from v0.2) | `docs/adr/ADR-004-updater-distribution.md` |
| ADR-005 | Code architecture: hexagonal (ports and adapters) with a pure core | accepted, 2026-10-10 | `docs/adr/ADR-005-code-architecture.md` |
| ADR-006 | Hybrid speech recognition (streaming + offline resync) | accepted | `docs/adr/ADR-006-hybrid-stt.md` |
| ADR-007 | Voice pipeline: error handling, diagnostics, latency tuning | accepted, 2026-10-11 | `docs/adr/ADR-007-voice-pipeline-tuning.md` |
| ADR-008 | Windows speech recognition as a fallback engine | accepted, 2026-10-11 | `docs/adr/ADR-008-windows-speech-fallback.md` |

ADR-001 to ADR-004 were taken over from `docs/research/RESEARCH.md` section 4 (2026-10-10), with unchanged content.

**Candidates for further ADRs:**
- Protocol and security of the remote control (see [WebSocket protocol](../reference/ws-protocol.md))
- Signing of the model manifest (CONCEPT mentions minisign; so far there is only SHA-256)
- Custom resampler or rubato
- HTTP client: ureq (blocking) instead of the reqwest sketched in the concept
