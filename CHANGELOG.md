# Changelog

Alle nennenswerten Änderungen. Format nach [Keep a Changelog](https://keepachangelog.com/de/1.1.0/), Versionierung nach [SemVer](https://semver.org/lang/de/).

## [Unreleased] – 0.1.0

> Noch nie kompiliert oder ausgeführt (Erstellung ohne Toolchain). Siehe README → „Erste Schritte nach dem Klonen“.

### Hinzugefügt (2026-10-11, Windows-Spracherkennung)
- **Windows-Spracherkennung als Fallback-Engine** (ADR-008, nur Windows, Feature `winspeech`, in den Windows-Release-Builds aktiv): `Windows.Media.SpeechRecognition` mit Diktat (Partials über `HypothesisGenerated`, Finals über `ResultGenerated`, VAD über `StateChanged`). Ohne Online-Spracherkennung läuft ein Offline-Skriptwort-Modus (List-Constraint). Die Engine nimmt selbst auf (neue Trait-Methode `SpeechEngine::self_capturing`) und läuft auf einem eigenen MTA-Thread.
- Fallback-Kette: Modelle (Nemotron/Kroko/20M) → Windows-Spracherkennung → Fehlerstatus bzw. Demo. Zusätzlich in der Modellauswahl wählbar: „Windows Spracherkennung (System)“, ohne Download.
- Präzise Fehlerhinweise: Sprachpaket fehlt, Online-Spracherkennung aus (`0x80045509`), Mikrofon verweigert, jeweils mit Pfad und `ms-settings:`-Link. Logging unter `zt::engine` mit Initialisierungszeit. Latenz als Sprechbeginn → Partial.
- CI-Job `winspeech` (windows-latest + windows-11-arm): Build/clippy/Tests mit Feature und Initialisierungs-Probe (`examples/winspeech_probe`).
### Hinzugefügt (2026-10-11, Typografie & Mausrad)
- **Schriftarten:** 12 lokal gebündelte Prompter-Schriften (OFL-1.1, offline, überwiegend variabel): Atkinson Hyperlegible Next, Inter, Lexend, Source Sans 3, Open Sans, Roboto Flex, IBM Plex Sans, Source Serif 4, Literata, Merriweather, OpenDyslexic, JetBrains Mono – plus „Systemschrift…“ mit freiem Namen. `serif` ist jetzt Source Serif 4 statt Georgia; das Talent-Fenster lädt jetzt alle Schriften (vorher fehlte JetBrains Mono).
- **Typografie** (Kern-Settings, `sanitized()` + Tests): Schriftstärke 100–900 (an die `wght`-Achse gebunden, Fallback auf vorhandene Gewichte), Zeichen-, Wort- und Absatzabstand, Ausrichtung (links/zentriert/Blocksatz), GROSSBUCHSTABEN. Live-Vorschau im Tab Darstellung; der Prompter misst die Layout-Map bei jeder Typo-Änderung und nach dem Nachladen von Schriften neu. Theme-Zeilenhöhe gilt jetzt als Minimum statt den Regler zu überstimmen.
- **Mausrad:** Konstant = Tempo ±5 WPM pro Raste (`Shift` ±1) mit kurzem „140 WPM“-Overlay; Manuell = Position; `Ctrl`/`Alt` vertauscht Position und Tempo. Voice-Modus unverändert (kurzer manueller Override). Funktioniert jetzt auch über dem Talent-Fenster. Doku in `shortcuts.md` und im `?`-Overlay.
- **Beispielskript** rund zehnmal länger (~1.250 Wörter): Zahlen, Namen, Fachwörter, kurze/lange Sätze, Schnellsprech-Absätze, Wiederholungen und Regieanweisungen zum Testen des Voice-Trackings.

### Hinzugefügt (2026-10-11, Lesemarker)
- Lesemarker vollständig anpassbar (Einstellungen → **Lesemarker**): neun Stile (Linie, Band, Pfeile, Klammern, Fokus-Fenster, Verlauf, Unterstreichung von Wort/Satz, Seitenbalken, kein Marker) mit Mini-Vorschau-Kacheln und Live-Vorschau; Farbe mit Theme-Vorlagen, Deckkraft, Dicke/Höhe, Position, Dimmen gelesen/ungelesen, Glow, Weichzeichnen, Animation (Gleiten/Puls, respektiert „Bewegung reduzieren“).
- Neue Kern-Einstellung `Settings.marker` (`zt_core::marker::MarkerSettings`, sanitized + Tests); der bisherige UI-Pref `markerStyle` wird einmalig migriert. Rendering in `ReadingMarker.svelte`, GPU-freundlich (transform/opacity).

### Geändert (2026-10-11, Fenster & Navigation)
- Windows/Linux: Control-Fenster ohne System-Titelleiste (`decorations: false`, `shadow: true` → unter Windows 11 abgerundete Ecken und Schatten über DWM). Eigene Fenstersteuerung oben rechts (Minimieren, Maximieren/Wiederherstellen, Schließen = App beenden); die Top-Bar ist Ziehbereich, Doppelklick maximiert. macOS behält die nativen Ampel-Knöpfe (`tauri.macos.conf.json`, `titleBarStyle: Overlay`).
- Skripte-Drawer: Umschalter in einer schmalen, animierten Leiste links statt in der oberen Aktionsleiste (`Ctrl+B` unverändert).
- Capabilities `control`: `core:window:allow-minimize`, `allow-toggle-maximize`, `allow-internal-toggle-maximize`, `allow-is-maximized`, `allow-start-dragging`.

### Behoben (2026-10-11, Mikrofonpegel)
- Sehr leises Mikrofon auf Surface/Snapdragon (Mikrofon-Array, 48 kHz, 2 Kanäle, −85 bis −100 dBFS): Der Downmix mittelte alle Kanäle; ein leerer Kanal kostete 6 dB, gegenphasige Kanäle löschten sich aus. Neu: Kanalwahl **Auto** (lautester Kanal, geglättet, 3 dB Hysterese), „Alle mischen“ oder fester Kanal (Einstellung „Mikrofon-Kanal“).
- Digitale Vorverstärkung: **Mikrofon-Gain** Auto (AGC auf ca. −20 dBFS Sprache, max. +40 dB, passt sich nur über −75 dBFS an, Soft-Limiter gegen Clipping) oder manuell (−20 bis +40 dB). Wirkt live, auch während Voice läuft.
- Diagnose: Rohpegel vor Gain, aktuelle Verstärkung, Pegel pro Kanal und aktiver Kanal (Diagnose-Tab und Log `zt::audio`). Warnung „Mikrofon sehr leise (< −50 dBFS beim Sprechen)“ mit Hinweis auf die Windows-Eingangslautstärke.
- Resampler 48k→16k geprüft: Skalierung und Filterverstärkung korrekt (Test: 1-kHz-Sinus behält die Amplitude ±1 dB).

### Hinzugefügt (2026-10-11)
- Einstellungen → **Über**: Version (`getVersion`), Build-Commit, Lizenz, Links zu Repo, Third-Party-Notices, Modelllizenzen und Changelog (Befehl `open_link` mit fester Link-Liste), Credits.

### Behoben (2026-10-11, Spracherkennung startet nicht)
- „failed to create OnlineRecognizer“ nach 1 ms mit Kroko DE/EN und Zipformer EN 20M: Der Adapter setzte immer `modeling_unit = "bpe"`, aber keines der Modelle hat `bpe.vocab`; sherpa-onnx `OnlineModelConfig::Validate` lehnt das ab (`bpe_vocab: '' does not exist`). `modeling_unit`/`bpe_vocab` werden jetzt nur gemeinsam gesetzt; für SentencePiece-Modelle wird `zt-generated-bpe.vocab` aus `tokens.txt` erzeugt, damit Hotwords funktionieren (Schreibweise passend zum Modell: Kroko gemischt, LibriSpeech groß).
- Erzeugung als Stufenleiter `beam+hotwords` → `beam` → `greedy`, jede Stufe geloggt. Vorab-Prüfung aller Modelldateien (Existenz, Größe) und Nachbau der sherpa-Validierungsregeln in Rust (`zt_stt::model_check`) für präzise Fehlertexte. `ZT_STT_DEBUG=1` schaltet sherpa `debug` ein.
- CI: neuer Job `stt-smoke` (Ubuntu, Windows x64, Windows ARM64) lädt die Modelle aus `models.json` (SHA-geprüft, Cache), dekodiert Test-WAVs (`cargo run -p zt-stt --example smoke --features sherpa`), prüft Schlüsselwörter, Hotword-Stufe und reproduziert den alten Fehler (`--legacy-check`). RTF und Chunk-Latenz stehen in der Job-Zusammenfassung.

### Behoben (2026-10-11, Voice)
- „Sprache synchronisieren klappt nicht, UI zeigt Demo“: Konnte ein installiertes Modell nicht geladen werden, fiel die App still in den Demo-Modus (Mock-Engine liest das Skript selbst vor). Jetzt: Fehlermeldung + Log, keine Selbstbewegung. Ist nur ein Modell einer anderen Sprache installiert, wird es genutzt (mit Hinweis) statt Demo. Nach einem Modell-Download startet eine laufende Voice-Sitzung neu. „Demo“ erscheint nur noch ohne Streaming-Modell, mit Button „Modell laden“ (ADR-007).
- Mikrofon-Watchdog: meldet „keine Daten“ bzw. „nur digitale Stille“ (Windows-Mikrofon-Datenschutz) nach 3 s.
- Abstürze des Voice-Threads werden gemeldet statt verschluckt.

### Hinzugefügt (2026-10-11, Diagnose)
- Strukturiertes Logging mit `tracing` in eine täglich rotierende Datei im App-Log-Ordner (`ZT_LOG` für den Filter), Events für Audio, VAD, STT-Partials/Finals, Tracker, Engine-Auswahl, Modell-Downloads, Frontend-Fehler.
- Einstellungen → **Diagnose**: Pegel, Engine/Modell, Audioformat, Frames/s, Tick-Rate, Tracker-Zustand, Latenzen p50/p95 (Sprechbeginn→Hypothese, Audio→Hypothese, Hypothese→Cursor, Audio→Cursor), letzte Fehler, Log-Export, Mikrofontest.

### Geändert (2026-10-11, Latenz)
- Scroll-Ease im Voice-Tracking 300 → 150 ms.
- Tracker-Nahpfad: das erste passende Wort nach Seek/Sprung bewegt den Cursor sofort (Idee aus ShayneP/local-teleprompter, MIT). Begründung und Latenzkette in ADR-007.
### Behoben (2026-10-11)
- Windows (x64/ARM64): Die App startete auf frischen Systemen nicht („VCRUNTIME140.dll was not found“). Die MSVC-Laufzeit wird jetzt statisch gelinkt (`.cargo/config.toml`, `+crt-static`); CI/Release prüfen per `dumpbin /dependents` (`scripts/check-crt.ps1`), dass keine `VCRUNTIME*/MSVCP*`-DLL importiert wird.

### Geändert (2026-10-11)
- Release-Builds aller Plattformen enthalten echte Spracherkennung (Feature `sherpa`, sherpa-onnx 1.13.8, statisch gelinkt inkl. onnxruntime – keine zusätzlichen DLLs/dylibs/.so). Ohne installiertes Modell startet die App weiterhin im Demo-Modus.
- `repository`-URL: https://github.com/philppplik/zero-prompter

### Hinzugefügt (2026-10-10, Release-Vorbereitung)
- Silero VAD (MIT) als Modell `silero-vad` im Katalog. Es wird mit jedem Sprachmodell automatisch mitgeladen und für die Resync-Segmentierung genutzt; ohne das Modell segmentiert die App über den Pegel (RMS). Der Inspector zeigt „Segmentierung: Silero VAD aktiv“.
- Alle Modell-URLs sind auf Commits bzw. Release-Assets gepinnt, und alle Dateien haben einen SHA-256 (auch `tokens.txt`).
- `Cargo.lock` und `package-lock.json` sind eingecheckt (Workflow `lockfiles.yml`). CI nutzt `npm ci` und `--locked` (MSRV).
- Release: Windows ARM64 (`aarch64-pc-windows-msvc`) und Store-Installer `*_store-setup.exe` mit Offline-WebView2 (`tauri.microsoftstore.conf.json`), Anleitung in `docs/book/src/dev/ms-store.md`.
- Actions aktualisiert (checkout/setup-node/upload-artifact v7, download-artifact v8, tauri-action v1, Node 22), keine Node-20-Warnungen mehr. `thiserror` 2.
### Hinzugefügt
- Hybride Spracherkennung (ADR-006): Streaming-Modell pro Sprache (Standard Kroko DE/EN, Fallback Zipformer EN 20M) mit automatischer Fallback-Kette, optionaler Offline-Resync mit Parakeet TDT 0.6B v3 (`OfflineRecognizer`, `nemo_transducer`) auf eigenem Thread, der den Tracker neu verankert. Inspector: Modellwahl pro Sprache und Resync-Schalter.
- Manifest v2: `modelType`, `roles`, `defaultRank`, optionaler `sha256`; Dateien ohne Hash werden nach dem Download gehasht und als unverifiziert angezeigt.
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
