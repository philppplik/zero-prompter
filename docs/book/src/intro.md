# Einleitung

zero-teleprompter ist ein freier Teleprompter für Windows, macOS und Linux (Lizenz MIT OR Apache-2.0). Er richtet sich an drei Gruppen:

- **Creator** sprechen allein vor der Kamera und wollen, dass der Text ihrer Stimme folgt.
- **Operatoren** betreiben Prompter-Glas mit Spiegelung, einen zweiten Monitor und eine Person vor der Kamera.
- **Vortragende** wollen ruhig und gut lesbar ablesen, auch bei Legasthenie oder in dunklen Räumen.

Leitprinzipien (aus `docs/concept/CONCEPT.md` §1):

1. **Lokal zuerst.** Audio verlässt nie das Gerät. Netzwerkzugriff gibt es in v0.1 nur für den optionalen Modelldownload.
2. **Nicht raten.** Wenn das Voice-Tracking unsicher ist, bleibt der Text lieber stehen (`hold`, `lost`), statt wegzulaufen.
3. **Tastatur zuerst.** Jede Aktion ist per Taste und über die Befehlspalette erreichbar.

Das Buch ist in vier Teile gegliedert:

- **Nutzerhandbuch:** Bedienung der App.
- **Entwicklerhandbuch:** Architektur, Algorithmen, Build und Tests.
- **Referenz:** Settings, Tokens, IPC und das geplante WebSocket-Protokoll.
- **ADR-Index:** Architekturentscheidungen.

Bauen: `mdbook build docs/book` (Ausgabe in `docs/book/book/`). mdBook ist nur ein Werkzeug für die Dokumentation und wird nicht mit der App ausgeliefert.
