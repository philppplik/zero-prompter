# ADR-Index

Architecture Decision Records im MADR-Stil. Vorgesehener Ablageort ist `docs/adr/`.

| Nr. | Titel | Status | Fundstelle |
|---|---|---|---|
| ADR-001 | UI-Framework: Tauri 2 + Svelte 5 | angenommen | `docs/adr/ADR-001-ui-framework.md` |
| ADR-002 | Audio nativ über cpal statt getUserMedia | angenommen | `docs/adr/ADR-002-native-audio-cpal.md` |
| ADR-003 | Projektlizenz MIT OR Apache-2.0 | angenommen | `docs/adr/ADR-003-license.md` |
| ADR-004 | Updater und Distribution | angenommen (Umsetzung ab v0.2) | `docs/adr/ADR-004-updater-distribution.md` |
| ADR-005 | Code-Architektur: hexagonal (Ports & Adapters) mit reinem Kern | angenommen, 2026-10-10 | `docs/adr/ADR-005-code-architecture.md` |
| ADR-006 | Hybride Spracherkennung (Streaming + Offline-Resync) | angenommen | `docs/adr/ADR-006-hybrid-stt.md` |
| ADR-007 | Voice-Pipeline: Fehlerbehandlung, Diagnose, Latenz-Tuning | angenommen, 2026-10-11 | `docs/adr/ADR-007-voice-pipeline-tuning.md` |
| ADR-008 | Windows-Spracherkennung als Fallback-Engine | angenommen, 2026-10-11 | `docs/adr/ADR-008-windows-speech-fallback.md` |

ADR-001 bis ADR-004 sind aus `docs/research/RESEARCH.md` §4 übernommen (2026-10-10), inhaltlich unverändert.

**Kandidaten für weitere ADRs:**
- Protokoll und Sicherheit der Fernsteuerung (siehe [WS-Protokoll](../reference/ws-protocol.md))
- Signatur des Modell-Manifests (CONCEPT nennt minisign, bisher gibt es nur SHA-256)
- Eigener Resampler oder rubato
- HTTP-Client: ureq (blockierend) statt des im Konzept skizzierten reqwest
