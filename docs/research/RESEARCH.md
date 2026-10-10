# zero-teleprompter – Research-Synthese

Stand: 10.10.2026 · Grundlage: 5 Research-Agents, 2 Runden plus Austauschphase · Kennzeichnung: **[V]** live verifiziert, **[U]** unverifiziert (Modellwissen oder Sekundärquelle)

---

## 1. Executive Summary

- **Marktlücke:** Kein großes Open-Source-Projekt bietet plattformübergreifendes, lokales Voice-Tracking auf Wortebene. QPrompt und Imaginary haben keins. Textream und NotchPrompter laufen nur auf macOS. OpenTeleprompter erkennt nur Sprachaktivität (VAD). Die Kommerz-Referenz ist PromptSmart (VoiceTrack, laut App Store nur Englisch).
- **Stack:** Tauri 2.12.x + Svelte 5 + TypeScript als UI, dazu ein Rust-Cargo-Workspace für Audio, STT, Alignment, Input und Remote. Die Installer erzeugt der Tauri-Bundler: NSIS .exe/.msi, .dmg, AppImage/.deb/.rpm.
- **STT:** sherpa-onnx (Apache-2.0) über die **offiziellen** Rust-Bindings (`sherpa-onnx` 1.13.x). Standardmodell für Deutsch ist `streaming-zipformer-de-kroko-2025-08-06` (~71 MB) mit Hotwords aus dem Skript. Parakeet-TDT-0.6b-v3 (CC-BY-4.0, ~671 MB) dient optional als Re-Sync-Pass. Whisper ist nicht für das Live-Tracking vorgesehen.
- **Alignment:** eine reine Rust-Crate `zt-align`. Sie normalisiert die Wörter, gleicht sie per Smith-Waterman im Fenster mit Fuzzy-Wortkosten ab, prüft Monotonie und Sprung-Hysterese, pausiert bei Stille (VAD) und schaltet über einen Zustandsautomaten (idle/listening/tracking/hold/lost/resync).
- **Lizenz:** `MIT OR Apache-2.0`. GPL-Projekte (QPrompt, Imaginary) dienen nur als UX-Referenz (Clean-Room). Code aus MIT-Projekten (OpenTeleprompter, autocue, Textream) übernehmen wir mit Attribution.
- **Größte Risiken:** WebKitGTK-Scrollperformance unter Linux, Screen-Capture-Exclusion (unter Linux nicht machbar), Windows-Code-Signing für Einzelpersonen in der EU, Lizenz des Kroko-Modells (vermutlich CC-BY-SA), Patentlage (US 9,953,646, laut Google verfallen).

---

## 2. Markt- & OSS-Landschaft

### 2.1 Feature-Matrix Open Source

| Projekt | Repo | Lizenz | Stars | Stack | Voice | Nutzung für uns |
|---|---|---|---|---|---|---|
| Textream | https://github.com/f/textream | MIT [V] | ~4,0k [V] | Swift/macOS | Word-Tracking, Classic, Voice-Activated; Remote per QR | UX- und Portierungsreferenz |
| OpenTeleprompter V3 | https://github.com/ArunNGun/openTeleprompt | MIT [V, README] | 76 [V] | **Tauri + Rust**, React | nur VAD (85–3400 Hz) | Code-Referenz: Overlay/Notch/Content-Protection in `src-tauri/src/lib.rs` |
| EdNutting/autocue | https://github.com/EdNutting/autocue | MIT [V] | 2 | Python, Vosk/sherpa-onnx, rapidfuzz | Wort-Tracking mit Backtrack- und Forward-Jump-Erkennung | Vorlage für Algorithmus und Tests |
| NotchPrompter | https://github.com/jpomykala/NotchPrompter | LICENSE vorhanden, Typ unklar (laut korben.info MIT) [U] | 650 | Swift | – | nur Referenz, bis die Lizenz geklärt ist |
| notchprompt | https://gittrend.io/repo/saif0200/notchprompt | [U] | ~759 | Swift | Word-Tracking v2.0 | nur Referenz |
| madebysan/cue | https://awesome.ecosyste.ms/projects/github.com%2Fmadebysan%2Fcue | MIT [U] | ~0 | SwiftUI | ja | gering |
| QPrompt | https://github.com/Cuperino/QPrompt-Teleprompter | **GPL-3.0** [V] | 610 | Qt/KDE | nein | nur UX (Marker, Spiegelung) |
| Imaginary Teleprompter | https://github.com/ImaginarySense/Imaginary-Teleprompter | **GPL-3.0** [V] | 298 | Electron | nein | nur UX |
| smart-teleprompter | https://github.com/amulyagarimella/smart-teleprompter | unklar | klein | Web + Google Cloud STT | ja, aber langsam | Gegenbeispiel: Cloud-STT ist zu langsam |
| VoicePrompter | – | – | – | – | – | kein Repo auffindbar |

**Konfliktauflösung Imaginary:** Die Angabe „GFDL“ bei AlternativeTo ist falsch. Das README sagt eindeutig „GPL3“.

### 2.2 Kommerzielle Anbieter

| Produkt | USP | Voice | Preis 2026 |
|---|---|---|---|
| PromptSmart | VoiceTrack: stoppt bei Ad-libs, offline, ohne Kalibrierung | ja (nur EN) | Pakete Starter, Teams, Enterprise. Preise nicht öffentlich ([promptsmart.com/pricing](https://promptsmart.com/pricing)) |
| Speakflow | Browser, Overlay über anderen Fenstern | Flow (Free eingeschränkt) | Free / Plus $15/Monat / Studio $99/Monat, jeweils jährlich [V] ([speakflow.com/pricing](https://www.speakflow.com/pricing)) |
| Teleprompter.com | Mobile, Voice-Scroll, 4K | ja | Pro [U] |
| BIGVU | Video-Suite | ja | ab ~$24.99/Monat [U]. Spiegelung nur im Pro-Plan |
| Teleprompter Premium | einfach, Remote | nein | Free + Abo [U] |

### 2.3 Übernahmeliste (Attribution in `THIRD_PARTY_NOTICES.md` mit Repo-URL, Commit-Hash und Copyright)
1. **OpenTeleprompter** `src-tauri/src/lib.rs`: Overlay- und Pill-Fenster, Always-on-top, Notch-Modus, Unsichtbarkeit beim Screen-Share.
2. **autocue** `src/autocue/` und `tests/`: Logik für Backtrack und Forward-Jump, Testfälle.
3. **Textream**: `MarqueeTextView.swift` (Wort-Highlighting), `NotchOverlayController.swift`, `ExternalDisplayController.swift`. Diese Dateien portieren wir als Konzept nach Rust/Svelte, wir kopieren sie nicht.

---

## 3. Speech-to-Text & Voice-Tracking

### 3.1 Engines

| Engine | Status | Streaming | DE | Lizenz | Rolle |
|---|---|---|---|---|---|
| **sherpa-onnx** | 15,2k Stars. Crate 1.13.8 vom 11.09.2026 [V]. Offizielle Rust-API | ja (Zipformer-Transducer) | Kroko-de, 71 MB [V] | Apache-2.0 | **Primär** |
| sherpa-rs (thewh1teagle) | archiviert/deprecated [V] | – | – | MIT | **nicht verwenden** |
| Parakeet-TDT-0.6b-v3 | WER-de 5,04 % (FLEURS) [V] | in sherpa nur offline | ja | CC-BY-4.0 | Tier L: Re-Sync auf VAD-Segmenten |
| whisper-rs | 0.16.0 vom 12.03.2026, **Unlicense** [V], Codeberg | nein (30-s-Fenster) | sehr gut | Unlicense / whisper.cpp MIT | nur Offline-Feature |
| Vosk small-de | ~45 MB [U] | ja | ja | Apache-2.0 | Low-End-Fallback |
| Moonshine | 11,2k Stars [V] | ja | **kein DE-Modell** belegt. Ältere Nicht-EN-Modelle stehen unter einer nicht-kommerziellen Lizenz | MIT (Ausnahmen) | verwerfen für DE |
| Silero VAD | [U] | – | sprachunabhängig | MIT | VAD (in sherpa-onnx integriert) |
| macOS SpeechAnalyzer | ab macOS 26, on-device, DE [V] | ja | ja | System | optionales natives Backend |
| Windows Live Captions | **keine öffentliche API** [V] | – | – | – | nicht nutzbar. Höchstens das alte `Windows.Media.SpeechRecognition` |

**Konfliktauflösung Engine:** Die Agents schlugen unterschiedliche Engines vor: whisper.cpp, Vosk, Moonshine (UX-Agent) sowie sherpa-onnx (STT- und Arch-Agent). Wir entscheiden uns für **sherpa-onnx**. Whisper streamt nicht. Moonshine hat kein deutsches Modell. Vosk ist schwächer und dient nur als Fallback. Nur sherpa-onnx liefert echtes Streaming, Hotword-Biasing ([Doku](https://k2-fsa.github.io/sherpa/onnx/hotwords/index.html), nur Transducer mit `modified_beam_search`), integriertes VAD und mehrere Modellfamilien hinter einer API.

**Konfliktauflösung whisper-rs-Lizenz:** Laut crates.io-Metadaten steht whisper-rs unter Unlicense. Das ist mit MIT kompatibel.

### 3.2 Modell-Tiers

| Tier | Modell | Größe | Auslieferung |
|---|---|---|---|
| S (Standard DE) | streaming-zipformer-de-kroko-2025-08-06 + Hotwords | ~71 MB | Download beim ersten Start (wegen CC-BY-SA nicht im Bundle) |
| S (EN) | offizielles sherpa-onnx-Streaming-Zipformer EN | ~20–70 MB [U] | Download, optional gebündelt |
| L (Re-Sync/Genauigkeit) | Parakeet-TDT-0.6b-v3 int8 | ~671 MB, ≥ 2 GB RAM | on-demand |
| Fallback | Vosk small-de/en | ~45 MB [U] | on-demand |

Lizenzpflichten: Für CC-BY-4.0 (Parakeet) braucht es einen Eintrag in About, in `THIRD_PARTY_MODELS.md` und im Download-Dialog. Die Lizenz von Kroko-de muss vor jeder Bündelung geklärt werden ([HF](https://huggingface.co/csukuangfj/sherpa-onnx-streaming-zipformer-de-kroko-2025-08-06)). Lädt der Nutzer das Modell separat herunter, ist ShareAlike unkritisch, weil wir das Modell nicht verändern.

### 3.3 Alignment-Algorithmus (`zt-align`, pure Rust, ohne IO)
1. **Vorverarbeitung:** Regieanweisungen, `[PAUSE]` und Markup aus dem Tokenstrom entfernen. Eine Positions-Map führt jeden Token auf Layout-Zeile und Y-Position zurück.
2. **Normalisierung:** Kleinschreibung, Interpunktion entfernen, Zahlen ausschreiben, Umlaut-Folding. Optional ein phonetischer Schlüssel (Kölner Phonetik für DE, Double Metaphone für EN).
3. **Suchfenster:** −10 bis +60 Wörter um den Cursor.
4. **Matching:** Die letzten k = 5–8 Hypothesenwörter laufen per Smith-Waterman (lokales Alignment) gegen das Fenster. Die Wortkosten ergeben sich aus der Jaro-Winkler- bzw. Levenshtein-Ähnlichkeit (`strsim`). Finals wiegen mehr als Partials. Der Aufwand ist O(k·w), also unter 5 ms.
5. **Hysterese:** Kleine Rückwärtsschritte sind erlaubt, das ist die Backtrack-Erkennung nach dem autocue-Muster. Sprünge über mehr als N Zeilen gelten erst nach 2 bestätigten Hypothesen (resync). Bei Verlust synchronisiert ein globaler Abgleich längerer N-Gramme neu.
6. **Glättung:** Ein kritisch gedämpfter Filter (Feder) bewegt die Zielposition. Eine Schätzung des Sprechtempos (Wörter/s) überbrückt die STT-Latenz durch Extrapolation.
7. **Zustände:** idle → listening → tracking ⇄ hold (VAD-Stille oder Konfidenz unter Schwelle: nicht raten, nicht scrollen) → lost (X s ohne Match) → resync.
8. **Biasing:** Die Skriptwörter gehen als Hotwords an das Modell (`hotwords-score` 1.5–2.0).

### 3.4 Latenzbudget (Ziel ≤ 250 ms, realistisch 200–350 ms; Werte noch per Benchmark zu prüfen)
| Stufe | Budget |
|---|---|
| cpal-Puffer | 10–20 ms |
| rubato auf 16 kHz | < 5 ms |
| VAD | 30-ms-Frames |
| Zipformer-Chunk | 160–320 ms [U] |
| Alignment | < 5 ms |
| Render | 8–16 ms |

### 3.5 Pipeline
`cpal` → Ringbuffer → `rubato` → VAD → sherpa-onnx OnlineRecognizer (eigener Thread) → Partial/Final-Events → `zt-align` → Scroll-Controller (Rust) → Tauri-Event → Svelte rendert per rAF.

---

## 4. Tech-Stack (ADR-Stil)

### ADR-001: UI-Framework – Tauri 2 + Svelte 5
- **Status:** akzeptiert
- **Kontext:** Gefordert sind eine native .exe sowie macOS und Linux, Polish auf Apple-Niveau, Rust-Geschwindigkeit, Installer und Updates.
- **Optionen:**
  - Tauri 2 (111,7k Stars, v2.12.2 vom 09.10.2026 [V])
  - Slint (24,1k [V])
  - egui, iced, Dioxus, GPUI (gpui-component 16,7k [V], Lizenz NOASSERTION), Makepad, Floem/Xilem
- **Entscheidung:** Tauri 2.12.x + Svelte 5 (5.57.2 [V]) + TypeScript + Vite. Für die Komponenten nehmen wir Bits UI bzw. shadcn-svelte.
- **Begründung:**
  - Ein Web-Stack ist der schnellste Weg zu Premium-Polish.
  - Text-Rendering und Fonts sind sehr gut.
  - Tauri bringt Bundler für alle drei Plattformen, Updater und Signing mit.
  - Die Lizenz ist permissiv.
  - Svelte statt Solid: Solid hat seit v1.9.0 (09/2024) kein GitHub-Release mehr veröffentlicht [V]. Svelte hat das größere Ökosystem (Command Palette, Transitions, reduced-motion).
- **Konsequenzen:**
  - Linux/WebKitGTK ist ein Risiko, siehe den Fallback-Plan unten.
  - Die Scroll-Engine liegt in Rust, die UI interpoliert nur. Damit ist ein nativer Ausgabe-Renderer jederzeit nachrüstbar.

### ADR-002: Audio nativ über cpal statt getUserMedia
- **Begründung:**
  - Es gibt nur einen Code-Pfad, und die Audiodaten bleiben im Rust-Thread bei sherpa-onnx.
  - Wir sind nicht vom Permission-Handling der WebViews abhängig ([tauri#2338](https://github.com/tauri-apps/tauri/issues/2338)).
- **macOS:** Pflicht sind `NSMicrophoneUsageDescription` in `src-tauri/Info.plist` und das Entitlement `com.apple.security.device.audio-input`. Für einen Mac-App-Store-Build kommt `device.microphone` dazu. Fehlt der Usage-String, liefert cpal stillschweigend nur Nullen ([soloterm/solo#114](https://github.com/soloterm/solo/issues/114)).
- **Windows:** Für unpackaged-Apps ist keine Capability nötig, bei MSIX schon ([MS Learn](https://learn.microsoft.com/en-us/windows/apps/develop/devices-sensors/enable-device-capabilities)).
- **Konfliktauflösung:** Der GUI-Agent hatte getUserMedia offengelassen. Wir entscheiden uns für cpal.

### ADR-003: Projektlizenz MIT OR Apache-2.0
- Ermöglicht die Übernahme von MIT-Code und bleibt geschäftlich offen.
- GPL-Code ist ausgeschlossen. `cargo-deny` und npm `license-checker` prüfen das im CI.
- **Slint-Konflikt aufgelöst:** Slint steht unter Royalty-free-2.0. Diese Lizenz erlaubt proprietäre Desktop-Nutzung, verlangt eine Attribution (AboutSlint) und schließt Embedded aus ([Lizenztext](https://unpkg.com/slint-ui@1.12.0/LICENSES/LicenseRef-Slint-Royalty-free-2.0.md)). Die Angabe „nur GPL“ aus Runde 1 war falsch. Für uns ist Slint nur noch ein Notfall-Renderer.
- **NDI:** Das SDK ist proprietär, deshalb nicht im MVP.

### ADR-004: Updater und Distribution
- **Updater:** `tauri-plugin-updater` (minisign) über `tauri-action`. cargo-dist/axoupdater nutzen wir nicht, weil die Wartung schwach ist und sie für CLI-Tools gedacht sind.
- **Installer:** unter 15 MB ohne Modelle.

### Linux-Fallback-Plan
1. WebKitGTK. Bei NVIDIA setzt die App automatisch `WEBKIT_DISABLE_DMABUF_RENDERER=1` ([Tauri Linux Graphics](https://v2.tauri.app/develop/debug/linux-graphics/)).
2. Ein Performance-Modus ohne `backdrop-filter`, mit Pixel-Snapping und 60-Hz-Ziel.
3. Ein CEF-Runtime-Build, sobald er stabil ist [U].
4. Notfall: ein nativer Talent-Renderer (wgpu/egui oder Slint) nur für das Ausgabefenster.

Servo/Verso planen wir nicht ein, das Projekt stagniert [U]. Bekannte Bugs: [WebKit 250988](https://bugs.webkit.org/show_bug.cgi?id=250988), [WebKit 310814](https://bugs.webkit.org/show_bug.cgi?id=310814).

---

## 5. UX- & Design-Prinzipien

- **Zwei Fenster:**
  - Control (Editor, Palette, Einstellungen, ungespiegelt, Mica/Vibrancy über [window-vibrancy](https://github.com/tauri-apps/window-vibrancy) nur im Chrome)
  - Talent-Output (Vollbild auf Monitor B, Spiegelung H/V/beides pro Fenster, kein Chrome)
- **Konfliktauflösung Editor vs. Direktmanipulation – Hybrid „Modes with a seam“:**
  - Es gibt einen Edit-Modus und einen Live-Modus.
  - Inline-Edit per Doppelklick ist nur im pausierten Prompter möglich.
  - Während des Voice-Trackings ist der Text gesperrt, weil eine Änderung die Wortindizes verschieben würde. Erlaubt bleibt Klick-auf-Wort, um dorthin zu springen.
- **Vier Scroll-Modi:** manuell, konstant (Anzeige in WPM), sprachaktiviert und Voice-Tracking.
- **Lesbarkeit:**
  - Autocue und Datavideo empfehlen 1 ft Leseabstand pro Zoll Bildschirmdiagonale [V] ([Autocue](https://www.autocue.com/education/tutorials/what-size-of-prompter-do-i-need/)).
  - Eine herstellerbelegte Fontregel gibt es nicht. Designannahme: Versalhöhe ≥ 1/150 des Leseabstands. Das muss per Nutzertest kalibriert werden.
  - 25–40 Zeichen pro Zeile, Zeilenabstand 1,3–1,6, Fokuslinie bei ~35 %, gelesener Text auf 40 % Opazität.
- **Font:** Atkinson Hyperlegible Next (SIL OFL 1.1 [V]; das Repo ist seit 13.08.2026 archiviert), Fallback Inter.
- **Voice-Zustände:** codiert über Farbe **und** Form (WCAG 1.4.1).
  - tracking: grün, Wort-Highlight
  - hold: gelber Ring, Text eingefroren
  - lost: Hinweis nur auf dem Operator-Screen
  - resync: Ease ≤ 300 ms
- **Command Palette** nach dem Raycast-Muster ([Action Panel](https://manual.raycast.com/action-panel)): Ctrl/⌘K, Fuzzy-Suche, Sektionen, ↵ für die Primäraktion, Inline-Shortcut-Recorder, `?` für das Shortcut-Overlay.
- **Hotkeys:**
  - Space für Play/Pause, ↑/↓ für das Tempo, PgUp/PgDn für Absätze, Home für den Anfang, M für Spiegeln, V für Voice, F für Vollbild.
  - Alle Tasten sind remappbar. Learn-Key funktioniert für Tastatur, HID, MIDI und Gamepad.
- **Themes als JSON-Tokens:** Namespaces `prompter.*`, `ui.*`, `state.*`, `type`, `layout`, `motion`, `focus`. Presets: Studio Dark (Default), Broadcast Classic, Daylight, High Contrast (AAA), Low Light, Dyslexia. Ein CI-Check prüft die Kontraste (Text ≥ 4,5:1, Zustandsfarben ≥ 3:1).
- **Accessibility:**
  - WCAG 2.2 AA im Chrome
  - Fokus-Ring 2 px mit ≥ 3:1 Kontrast ([2.4.13](https://www.w3.org/WAI/WCAG22/Understanding/focus-appearance.html))
  - Zielgröße 24×24 px
  - `prefers-reduced-motion` wird respektiert
- **Onboarding (≤ 60 s):**
  1. Ein Beispielskript läuft bereits.
  2. Setup des Leseabstands.
  3. Erklärung vor der Berechtigungsabfrage, dann der OS-Prompt mit Deeplink, falls abgelehnt.
  4. Mikrofontest mit Live-Anzeige des Zustands.
  5. Wahl des Ausgabe-Monitors.
- **Motion:** 150–250 ms mit Spring-Easing. Scroll über rAF + `translate3d`, berechnet aus px/s × dt.
- **P2 / best effort:** Screen-Capture-Exclusion (Windows zuverlässig, macOS unsicher wegen ScreenCaptureKit, Linux nicht angeboten), 120 Hz, Director-Remote-Editor.

---

## 6. Architektur, Tooling, CI, Testing, Docs

### 6.1 Workspace
```
zero-teleprompter/
  Cargo.toml            # workspace, MIT OR Apache-2.0
  crates/
    zt-core/            # Skriptmodell, Import (pulldown-cmark, docx-rs), Settings/Themes, Scroll-Engine
    zt-align/           # Aligner, pure Logik
    zt-audio/           # cpal + rubato + VAD
    zt-stt/             # trait SpeechEngine; sherpa-onnx (default), whisper-rs (feature), macOS SpeechAnalyzer (feature)
    zt-input/           # global-hotkey (Win/mac/X11) + ashpd-Portal (Wayland), midir, hidapi, gilrs, Learn-Key
    zt-remote/          # axum + WebSocket, PWA-Assets, QR, mDNS; später obws (OBS), rosc (OSC)
    zt-models/          # signiertes models.json, Download mit Range-Resume, SHA-256, atomic rename
  apps/desktop/         # Tauri 2 + Svelte 5 (dünne src-tauri)
  apps/remote-ui/       # Handy-PWA
  docs/                 # mdBook, ADRs (MADR)
  fixtures/audio/       # WAV + erwartete Wortindex-Zeitpunkte (Git LFS)
```

### 6.2 Plattform-Features
- **Screen-Capture-Exclusion:** über Tauri `Window::set_content_protected` ([docs.rs](https://docs.rs/tauri/latest/tauri/window/struct.Window.html)).
  - Windows: `WDA_EXCLUDEFROMCAPTURE` ab Win10 2004. Microsoft garantiert ausdrücklich nicht, dass jede Aufnahme verhindert wird.
  - macOS: `sharingType = .none`, wird von ScreenCaptureKit teils umgangen.
  - Linux: unmöglich.
- **Globale Hotkeys:** `tauri-plugin-global-shortcut` 2.3.2 unterstützt unter Linux nur X11 [V]. Unter Wayland läuft es über das xdg-desktop-portal GlobalShortcuts (`ashpd`).

### 6.3 Modelle
- **Hosting:** HuggingFace-Repo, GitHub Releases als Fallback.
- **Manifest:** mit minisign signiert, Felder `id, url[], sha256, size, license, attribution, min_app_version`.
- **Download:** über `reqwest` mit Range-Resume.

### 6.4 CI (GitHub Actions)
- **Matrix:** `windows-latest`, `macos-latest` (aarch64 und x86_64 getrennt; Universal per lipo wird später geprüft), `ubuntu-22.04` (WebKitGTK 4.1, libasound2-dev).
- **Caching:** `Swatinem/rust-cache` und `actions/cache` für die sherpa-onnx-Prebuilt-Archive.
- **Gates:** `cargo fmt --check`, `clippy -D warnings`, `cargo-deny`, `license-checker`, `cargo nextest`, Vitest, Playwright, Kontrast-Check.
- **Release:** `tauri-action` bei `v*`-Tags.
- **Native Libraries:**
  - Windows: `onnxruntime.dll` neben der exe
  - macOS: in `Frameworks/` mit @rpath, mitsigniert
  - Linux: über die Bundle-Resources
- **Signing:**
  - macOS: Developer ID ($99/Jahr), notarytool über `APPLE_API_*`.
  - Windows: Azure **Artifact** Signing (seit 01/2026 GA, $9,99/Monat). In der EU gilt es **nur für Organisationen** [V] ([devclass](https://devclass.com/2026/01/14/code-signing-windows-apps-may-be-easier-and-more-secure-with-new-azure-artifact-service/)). Alternative ist ein OV-Zertifikat mit Cloud-HSM.

### 6.5 Testing
- **Unit-Tests** in UI-freien Crates.
- **`proptest`:** Monotonie beim Vorwärtslesen, Robustheit gegen eingefügte und ausgelassene Wörter, Parser stürzt nie ab.
- **`insta`:** Snapshots für Parser und Themes.
- **`cargo-fuzz`:** für die Importer.
- **Aligner-Fixtures:**
  1. Synthetische Token-Streams laufen deterministisch in jedem PR.
  2. 5–10 eigene CC0-WAVs mit Erwartungs-YAML (±2 Wörter) laufen nightly, weil dafür ein Modell-Download nötig ist.
- **E2E:** Playwright für die Web-UI, `tauri-driver` auf Windows und Linux.

### 6.6 Docs
- mdBook für Nutzer- und Entwicklerhandbuch, veröffentlicht über GitHub Pages
- `cargo doc`
- ADRs im MADR-Format unter `docs/adr`
- `THIRD_PARTY_NOTICES.md` und `THIRD_PARTY_MODELS.md`

---

## 7. Risiken & offene Fragen

| # | Risiko / Frage | Maßnahme |
|---|---|---|
| 1 | WebKitGTK: Ruckeln und kein Beleg für 120 Hz | Scroll-Prototyp in Sprint 1 auf allen drei OS bauen, Fallback-Plan aus §4 |
| 2 | Lizenz von Kroko-de (CC-BY-SA?) und fehlende WER-Angabe | LICENSE bei Banafo prüfen, eigener FLEURS-de-Benchmark |
| 3 | Patent US 9,953,646 (Belleau/PromptSmart). Laut Google „Expired – Fee Related“, das ist nicht rechtlich bestätigt. Es gab Klagen vor dem S.D.N.Y. Dazu kommen weitere Patente (US11902690, US12167159, US12718807) | Status im USPTO Patent Center prüfen, Freedom-to-Operate-Prüfung durch einen Anwalt vor kommerziellem Release |
| 4 | Windows-Signing für Einzelpersonen in der EU | Klären, ob die yes-investmedia GmbH die Kriterien erfüllt (3 Jahre). Sonst OV-Zertifikat |
| 5 | Screen-Capture-Exclusion ist nur Abschreckung | Als best effort kommunizieren, unter Linux ausblenden |
| 6 | Wayland: Hotkeys nur über das Portal | ashpd-Integration testen (GNOME, KDE) |
| 7 | Tatsächliche Latenz und CPU-Last auf Low-End-Geräten | Benchmark mit `decode_chunk_len`, Mindesthardware festlegen |
| 8 | Größe von onnxruntime, Universal-Build für macOS | im CI messen |
| 9 | Ungeprüfte Crate-Versionen (cpal, rubato, midir, hidapi, gilrs, axum, obws, rosc, docx-rs, mdBook, insta, proptest, nextest, vosk-rs, ort) | vor der Implementierung mit `cargo search` prüfen, Renovate einrichten |
| 10 | Lizenz von NotchPrompter und gpui-component, CEF-Runtime-Status, Status von Verso | Nachprüfung, für die MVP-Entscheidung nicht blockierend |
| 11 | Produktentscheidung: Default im Zustand „lost“ (Stop oder Classic-Scroll) | Vorschlag: Stop, umschaltbar |
| 12 | Grenze für die Installergröße | Vorschlag: unter 15 MB ohne Modelle, Modell-Download beim Onboarding |

---

## 8. Quellen (Auswahl)

- Repos: https://github.com/f/textream · https://github.com/ArunNGun/openTeleprompt · https://github.com/EdNutting/autocue · https://github.com/jpomykala/NotchPrompter · https://github.com/Cuperino/QPrompt-Teleprompter · https://github.com/ImaginarySense/Imaginary-Teleprompter · https://github.com/k2-fsa/sherpa-onnx · https://github.com/thewh1teagle/sherpa-rs · https://github.com/moonshine-ai/moonshine · https://github.com/tauri-apps/tauri · https://github.com/tauri-apps/window-vibrancy · https://github.com/googlefonts/atkinson-hyperlegible-next
- Crates und Modelle: https://crates.io/crates/sherpa-onnx · https://crates.io/crates/whisper-rs · https://huggingface.co/nvidia/parakeet-tdt-0.6b-v3 · https://huggingface.co/csukuangfj/sherpa-onnx-nemo-parakeet-tdt-0.6b-v3-int8 · https://huggingface.co/csukuangfj/sherpa-onnx-streaming-zipformer-de-kroko-2025-08-06 · https://k2-fsa.github.io/sherpa/onnx/hotwords/index.html
- Plattform: https://developer.apple.com/documentation/speech/speechanalyzer · https://developer.apple.com/videos/play/wwdc2025/277/ · https://support.microsoft.com/windows/b52da59c-14b8-4031-aeeb-f6a47e6055df · https://v2.tauri.app/develop/debug/linux-graphics/ · https://v2.tauri.app/release/global-shortcut/v2.3.2/ · https://tauri.app/distribute/sign/macos · https://www.meziantou.net/how-to-exclude-your-windows-app-from-screen-capture-and-recall.htm · https://developer.apple.com/forums/thread/808016
- Lizenzen und Recht: https://slint.dev/pricing · https://unpkg.com/slint-ui@1.12.0/LICENSES/LicenseRef-Slint-Royalty-free-2.0.md · https://patents.google.com/patent/US9953646
- UX: https://www.autocue.com/education/tutorials/what-size-of-prompter-do-i-need/ · https://datavideo.com/article/590/how-to-choose-the-right-teleprompter · https://www.w3.org/WAI/WCAG22/Understanding/focus-appearance.html · https://manual.raycast.com/action-panel · https://apps.apple.com/app/id894811756
- Markt: https://www.speakflow.com/pricing · https://promptsmart.com/pricing · https://www.teleprompter.com/compare/teleprompter-vs-promptsmart
- Signing: https://devclass.com/2026/01/14/code-signing-windows-apps-may-be-easier-and-more-secure-with-new-azure-artifact-service/ · https://learn.microsoft.com/en-us/answers/questions/2243504/trusted-signing-for-other-countries
