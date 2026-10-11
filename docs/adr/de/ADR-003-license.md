# ADR-003: Projektlizenz MIT OR Apache-2.0

- **Status:** angenommen
- **Quelle:** übernommen aus `docs/research/RESEARCH.md` §4 (2026-10-10)

## Entscheidung

zero-teleprompter steht wahlweise unter MIT oder Apache-2.0.

## Begründung und Folgen

- Erlaubt die Übernahme von MIT-Code und bleibt geschäftlich offen.
- GPL-Code ist ausgeschlossen. `cargo-deny` und ein npm-Lizenzcheck prüfen das in der CI.
- **Slint:** steht unter Royalty-free-2.0 (proprietäre Desktop-Nutzung erlaubt, Attribution nötig, Embedded ausgeschlossen; [Lizenztext](https://unpkg.com/slint-ui@1.12.0/LICENSES/LicenseRef-Slint-Royalty-free-2.0.md)). Für uns nur Notfall-Renderer.
- **NDI:** proprietäres SDK, nicht im MVP.

## Nachtrag 2026-10-10: Modelle

Modelle werden separat heruntergeladen und fallen nicht unter die Projektlizenz. Die Lizenzlage pro Modell steht in `THIRD_PARTY_MODELS.md`, `docs/research/MODELS.md` und `docs/legal/LEGAL.md`. Insbesondere ist die Lizenz von Kroko-DE (CC-BY-SA laut Modellseite vs. „non-commercial“ laut Drittquelle) ungeklärt.
