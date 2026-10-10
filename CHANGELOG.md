# Changelog

Alle nennenswerten Änderungen. Format nach [Keep a Changelog](https://keepachangelog.com/de/1.1.0/), Versionierung nach [SemVer](https://semver.org/lang/de/).

## [Unreleased] – 0.1.0

> Noch nie kompiliert oder ausgeführt (Erstellung ohne Toolchain). Siehe README → „Erste Schritte nach dem Klonen“.

### Hinzugefügt
- Cargo-Workspace mit den reinen Kern-Crates `zt-core`, `zt-align`, `zt-audio`, `zt-stt`, `zt-models` und dem Test-Crate `zt-testkit` (ADR-005, hexagonal).
- `zt-core`: Skriptmodell (Absätze, Regieanweisungen in `[...]`), Automat Wiedergabe × Modus mit Countdown, Scroll-Modi Manuell, Konstant, Sprachaktiviert und Voice-Tracking, Lost-Policy, manueller Override, Settings und Themes aus `docs/design/tokens.json`.
- `zt-align`: Normalisierung für Deutsch und Englisch (Umlaut-Folding, Zahlen zu Wörtern, Füllwörter), Wortähnlichkeit (Levenshtein, Jaro-Winkler), gefenstertes Smith-Waterman, `VoiceTracker` mit Hysterese und den Zuständen idle, listening, tracking, hold, lost, resync.
- `zt-audio`: Port `AudioSource`, cpal-Adapter (Feature `cpal`), eigener Resampler auf 16 kHz mono.
- `zt-stt`: Port `SpeechEngine`, `MockEngine` (Demo und Tests), sherpa-onnx-Adapter (Feature `sherpa`, ungeprüft), Hotword-Auswahl.
- `zt-models`: Manifest mit Validierung (nur HTTPS, sichere Pfade), SHA-256-Prüfung, Installationslayout, Downloader-Port.
- Tauri-2-App mit Control- und Talent-Fenster, 22 Commands, 9 Events, Tick-Schleife mit ~60 Hz, Voice-Thread, globale Hotkeys (`Ctrl+Shift+…`), atomare Persistenz, Modelldownload über ureq mit Fortsetzen.
- Svelte-5-Frontend: Sidebar und Bibliothek, Editor, Prompter-Vorschau mit Wort-Hervorhebung, Inspector, Transport, Befehlspalette, Übersicht der Tastenkürzel, Onboarding, Drag-and-Drop-Import.
- Tests: Unit-, Property- (proptest), Tabellen- und Szenariotests (Rust), Vitest (Frontend).
- CI (fmt, clippy, Tests auf drei Betriebssystemen, MSRV 1.80, Adapter, App, Lizenzen) und Release-Workflow (Draft-Release mit Installern).
- Dokumentation: README, mdBook unter `docs/book/`, CONTRIBUTING, THIRD_PARTY_NOTICES, THIRD_PARTY_MODELS.

### Geändert (Überarbeitung 2026-10-10, weiterhin nicht kompiliert)
- `zt-stt`: sherpa-onnx-Adapter neu gegen **sherpa-onnx =1.13.8** (docs.rs-belegt). `get_result` liefert `Option`, kein `unsafe impl Send` mehr, alle Config-Werte explizit (u. a. `max_active_paths = 4`), Hotwords über `hotwords_buf`, neue `SherpaConfig`-Felder `bpe_vocab`, `modeling_unit`, `max_active_paths`. `set_hotwords` meldet `Unsupported`, wenn bei BPE-Modellen `bpe.vocab` fehlt.
- Globale Hotkeys auf `Ctrl+Shift+F6` … `F11` verlegt (Voice, Play/Pause, −/+5 WPM, Absatz zurück/vor). Die alten Kombinationen kollidierten u. a. mit „als Text einfügen“ (`Ctrl+Shift+V`).
- `zt-audio`: bewusst bei cpal 0.15 (0.18.2 ist API-inkompatibel).
- Modellkatalog `resources/models.json` mit 5 Modellen, siehe `docs/research/MODELS.md`.
- Formatierung: `rustfmt.toml`, Prettier-Konfiguration und Skripte `format`/`format:check`. Tests von `zt-models` in `src/tests.rs`. `Inspector.svelte` aufgeteilt in `HotkeyEditor.svelte` und `ModelList.svelte`; die Tastatur-Aufnahme hängt nur noch am Fenster, solange „Tastenkürzel“ aufgeklappt ist.
- `scroll://frame` enthält neu `layoutLines` (nur bei passender Skriptversion; Frontend wertet es noch nicht aus).
- ADR-001 bis ADR-004 als eigene Dateien in `docs/adr/`. Rechtliche Einschätzung in `docs/legal/LEGAL.md`.

### Behoben
- Fokuslinie: ein einheitlicher Bereich 5–70 % in Rust und Frontend (`FOCUS_LINE_MIN/MAX`); NaN/∞ fällt auf den Standard zurück.
- Talent-Fenster: Schließen per Titelleiste beendet Vollbild und sendet `talent://visibility = false`.
- `report_layout` wird ausgewertet statt verworfen.
- Voice-Cursor wird auf das Skriptende begrenzt.
- Leeres Skript: Wiedergabe endet sofort bei Position 0, in jedem Scroll-Modus.
- Play während des Countdowns wird ignoriert statt als Fehler abgelehnt.
- Im Tick, in dem der Countdown endet, wird nur die Restzeit gescrollt (`Playback::tick_overflow`).

### Bekannt
- Modell-Manifest noch nicht ladbar: SHA-256 der `tokens.txt` fehlen, URLs nicht auf Commits gepinnt. Ohne `sherpa` und ohne Modell läuft Voice im Demo-Modus.
- Lizenz von Kroko-DE ungeklärt (release-kritisch), Patentlage US 9,953,646 nicht anwaltlich geprüft, siehe `docs/legal/LEGAL.md`.
- `cargo fmt --check`, `clippy -D warnings`, Prettier nie ausgeführt.
- Kein `package-lock.json`. Die Icons werden über `npx tauri icon` erzeugt.
- Fernsteuerung (WebSocket/PWA), MIDI/HID/Gamepad, Overlay, `.docx`, Updater und Signierung folgen erst ab v0.2.
