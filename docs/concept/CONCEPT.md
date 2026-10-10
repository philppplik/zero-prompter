# zero-teleprompter – Produktkonzept

Stand: 10.10.2026 · Grundlage: `docs/research/RESEARCH.md` · Status: Entwurf v1 · Lizenz Projekt: `MIT OR Apache-2.0`

---

## 1. Vision & Prinzipien

**Vision:** Der Teleprompter, der zuhört. zero-teleprompter ist ein plattformübergreifender (Windows, macOS, Linux), quelloffener Desktop-Prompter mit lokalem Voice-Tracking auf Wortebene, auch für Deutsch. Er folgt der Stimme, wartet bei Ad-libs und findet nach Sprüngen zurück, ohne Cloud, ohne Konto, ohne Abo.

**Positionierung:** Lücke zwischen QPrompt/Imaginary (OSS, kein Voice), Textream/NotchPrompter (Voice, nur macOS) und PromptSmart (Voice, kommerziell, nur EN).

**Prinzipien**
1. **Lokal zuerst.** Audio verlässt nie das Gerät. Kein Netzwerkzugriff außer Modell-Download, Updater und (opt-in) LAN-Fernsteuerung.
2. **Nicht raten.** Bei Unsicherheit hält der Prompter an (hold), statt falsch zu scrollen. Ein stehender Text ist besser als ein davonlaufender.
3. **Talent zuerst.** Das Ausgabefenster zeigt nur Text und Lesemarker. Alle Diagnose, Warnungen und Zustände „lost“ landen auf dem Operator-Screen.
4. **Null Reibung.** Erster Start bis lesbarer Text in ≤ 60 s. Jede Funktion per Tastatur und Command Palette erreichbar.
5. **Schnell und ruhig.** Flüssiges Scrollen (60 FPS Minimum), Startzeit < 1,5 s, Installer < 15 MB.
6. **Barrierefrei.** WCAG 2.2 AA im Chrome, Zustände über Farbe **und** Form, `prefers-reduced-motion` respektiert.
7. **Ehrlich.** Best-effort-Features (Screen-Capture-Exclusion) werden als solche gekennzeichnet; auf Linux ausgeblendet.

---

## 2. Personas & Kern-Use-Cases

| Persona | Kontext | Bedürfnis | Kritische Features |
|---|---|---|---|
| **Lena, Solo-Creatorin** (YouTube/LinkedIn) | Laptop + Webcam, ein Monitor, Notch/Overlay nah an der Kamera | Natürlich wirken, ohne Operator, Deutsch | Voice-Tracking DE, Overlay-Fenster nah an Kamera, Hotkeys, Import aus Google Docs/Markdown |
| **Tom, Video-Operator** (Agentur/Studio) | Prompter-Glas mit Spiegel, zweiter Monitor, Talent ist eine andere Person | Zuverlässig steuern, Spiegelung, Fernbedienung | Zwei-Fenster-Modell, Spiegelung H/V, Fußpedal/MIDI/Gamepad, Tempo-Feinsteuerung, Operator-Statusanzeige |
| **Dr. Weber, Speaker/Webinar** | Zoom/Teams, Screen-Share | Skript sehen, ohne dass es geteilt wird | Overlay always-on-top, Screen-Capture-Exclusion (Win), Voice-Tracking |
| **Sam, Accessibility-Nutzer** (Legasthenie, Sehschwäche) | beliebig | Gut lesbare Typografie | Dyslexia/High-Contrast-Themes, große Fonts, Fokuslinie, gedimmter gelesener Text |
| **Redaktion/Regie** (später) | Mehrere Geräte | Skript live korrigieren, Status sehen | Handy-Remote (PWA), Director-Remote-Editor (v1.0) |

**Kern-Use-Cases**
- UC1: Skript aus Datei/Zwischenablage importieren und lesbar formatieren.
- UC2: Proben mit konstantem Tempo, Tempo und Darstellung anpassen.
- UC3: Live-Aufnahme mit Voice-Tracking auf Monitor B, gespiegelt.
- UC4: Steuerung über Hotkeys, Fußpedal oder Handy.
- UC5: Overlay nahe der Kamera während eines Video-Calls.

---

## 3. Feature-Scope (priorisiert)

Legende: **M** = MVP v0.1 · **0.2** · **1.0** · **später**

### 3.1 Skript & Import
| Feature | Release |
|---|---|
| Plaintext (.txt), Markdown (.md, pulldown-cmark), Einfügen aus Zwischenablage | M |
| Editor (Plain/Markdown, Überschriften als Absatzmarker), Autosave | M |
| Regieanweisungen `[PAUSE]`, `[...]`, `(( … ))` – sichtbar gedimmt, vom Tracking ignoriert | M |
| Mehrere Skripte, Bibliothek (lokaler Ordner), zuletzt geöffnet | 0.2 |
| .docx (docx-rs), .rtf, .odt (später) | 0.2 / später |
| Sprungmarken/Kapitel, Outline-Navigation | 0.2 |
| Live-Inline-Edit im pausierten Prompter (Doppelklick) | 0.2 |
| Director-Remote-Editor (Bearbeitung vom zweiten Gerät) | 1.0 |

### 3.2 Anpassbarkeit
| Feature | Details | Release |
|---|---|---|
| **Font** | Atkinson Hyperlegible Next (Default, gebündelt), Inter (Fallback), Systemfonts wählbar | M (gebündelte) / 0.2 (System) |
| **Größe** | 24–300 px, Schritte per Hotkey (Ctrl +/−), Assistent „Leseabstand“ (Versalhöhe ≥ 1/150 Leseabstand) | M (Slider) / 0.2 (Assistent) |
| **Zeilenlänge/-abstand** | 25–40 Zeichen/Zeile (Spaltenbreite), Zeilenabstand 1,3–1,6, Ränder | M |
| **Farben** | Text, Hintergrund, gelesener Text (Opazität, Default 40 %), Highlight, Marker | M |
| **Themes** | JSON-Tokens (`prompter.*`, `ui.*`, `state.*`, `type`, `layout`, `motion`, `focus`). Presets: Studio Dark (Default), Broadcast Classic, Daylight, High Contrast (AAA), Low Light, Dyslexia | M: 3 Presets (Studio Dark, Broadcast Classic, High Contrast) · 0.2: alle 6 + eigene Themes + Import/Export · 1.0: Theme-Editor mit Live-Kontrastprüfung |
| **Scroll-Speed** | Konstant in WPM (Default 140) und px/s, Feinsteuerung ↑/↓ (±5 WPM), Ease-in/out beim Start/Stop | M |
| **Spiegelung** | H, V, beides – pro Fenster unabhängig (Hotkey M) | M |
| **Lesemarker** | Fokuslinie (Default 35 % Höhe, verschiebbar), Stil: Linie, Pfeil, Band, Keil; gelesenen Text dimmen; aktuelles Wort hervorheben (Voice) | M: Linie + Band + Dimmen · 0.2: alle Stile |
| Countdown vor Start (3-2-1) | | M |
| Ausrichtung links/zentriert, Großschreibung erzwingen | | 0.2 |
| Profile (Einstellungsbündel pro Setup: „Studio Glas“, „Laptop-Overlay“) | | 0.2 |

### 3.3 Scroll-Modi & Voice-Tracking
| Feature | Release |
|---|---|
| Manuell (Rad, Pfeile, PgUp/PgDn, Ziehen) | M |
| Konstant (WPM) | M |
| **Voice-Tracking** (sherpa-onnx, Kroko-de 71 MB, EN-Zipformer; Hotwords aus Skript; `zt-align`) | M |
| Zustandsanzeige tracking/hold/lost/resync (Operator), Wort-Highlight (Talent) | M |
| Klick-auf-Wort = Cursor setzen/Resync | M |
| Verhalten bei „lost“: Stop (Default) oder Fallback auf konstantes Tempo | M (Schalter) |
| Sprachaktiviert (VAD: scrollt nur, solange gesprochen wird) | 0.2 |
| Tempo-Extrapolation über Sprechtempo | M (einfach) / 0.2 (getunt) |
| Phonetische Schlüssel (Kölner Phonetik/Double Metaphone) | 0.2 |
| Parakeet-TDT Re-Sync-Pass (Tier L, on-demand 671 MB) | 1.0 |
| Vosk-Fallback für Low-End | 1.0 |
| macOS SpeechAnalyzer-Backend (feature) | 1.0 |
| Sprechcoach-Statistik (WPM, Ad-libs, Abweichungen) | später |

### 3.4 Fenster, Ausgabe, Plattform
| Feature | Release |
|---|---|
| Control-Fenster + Talent-Output-Fenster (Vollbild auf gewähltem Monitor) | M |
| Monitorwahl, Fenstermodus/Vollbild (F) | M |
| Overlay-Modus: rahmenlos, always-on-top, transparent, Größe/Position merken | 0.2 |
| Notch/Pill-Modus (macOS, an Kamera) | 1.0 |
| Screen-Capture-Exclusion (Win zuverlässig, macOS best effort, Linux ausgeblendet) | 0.2 |
| Linux-Performance-Modus (kein backdrop-filter, Pixel-Snapping) | M |
| 120-Hz-Rendering | 1.0 (best effort) |
| Auto-Updater (minisign) | 0.2 |
| NDI/Video-Out | später (SDK proprietär) |

### 3.5 Eingabe & Fernsteuerung
| Feature | Release |
|---|---|
| App-Hotkeys (Space, ↑/↓, PgUp/PgDn, Home, M, V, F, Ctrl/⌘K, ?) | M |
| Remapping + Shortcut-Recorder | M (Tastatur) |
| Globale Hotkeys (Win/mac/X11), Wayland via ashpd-Portal | 0.2 |
| Fußpedal/Presenter (HID als Tastatur) | M (implizit über Tastatur) |
| HID (hidapi), MIDI (midir), Gamepad (gilrs), Learn-Key | 0.2 |
| Handy-Remote: LAN-Webserver (axum + WS), PWA, QR-Code, Pairing-PIN | 0.2 |
| mDNS-Discovery, OSC (rosc), OBS (obws) | 1.0 |

### 3.6 Onboarding, Hilfe, Qualität
| Feature | Release |
|---|---|
| Onboarding ≤ 60 s (Beispielskript, Mikro-Erklärung + Prompt, Mikrotest, Modelldownload, Monitorwahl) | M (kompakt) / 0.2 (Leseabstand) |
| Command Palette (Raycast-Muster) | M |
| Shortcut-Overlay (`?`) | M |
| Modell-Manager (Download, Resume, SHA-256, Löschen, Lizenzen) | M |
| Lokalisierung DE/EN | M |
| Signierte Installer (macOS notarisiert; Windows Signing je nach Klärung) | 0.2 |

---

## 4. Informationsarchitektur & Fenster-Modell

### 4.1 Fenster
```
┌──────────── Control-Fenster (Monitor A, ungespiegelt) ───────────────┐
│ Titlebar (Mica/Vibrancy)  [Skriptname]   [Mode: Edit|Live]  [⌘K]      │
├────────────┬───────────────────────────────────────┬─────────────────┤
│ Sidebar    │  Hauptbereich                         │ Inspector       │
│ - Skripte  │  Edit-Modus: Editor                   │ - Darstellung   │
│ - Outline  │  Live-Modus: Vorschau (1:1 Talent,    │ - Scroll/Tempo  │
│            │   verkleinert) + Wort-Cursor           │ - Voice         │
│            │                                       │ - Ausgabe       │
├────────────┴───────────────────────────────────────┴─────────────────┤
│ Transportleiste: ▶/⏸  Tempo  Modus(Manual/Const/Voice)  Status-Pill  │
│  Mikro-Pegel  Position (Wort 312/1840, 4:12 verbleibend)              │
└───────────────────────────────────────────────────────────────────────┘

┌──────── Talent-Output-Fenster (Monitor B, Vollbild, kein Chrome) ─────┐
│  gelesener Text (40 %)                                                │
│ ▶────────────── Fokuslinie 35 % ───────────────── aktuelles Wort      │
│  kommender Text                                                       │
│  (hold: dezenter gelber Ring an der Fokuslinie, sonst nichts)         │
└───────────────────────────────────────────────────────────────────────┘
```
- **Einzel-Monitor:** Talent-Fenster als zweites, frei positionierbares Fenster oder Vorschau im Control-Fenster maximiert (Taste F).
- **Overlay (0.2):** Talent-Fenster rahmenlos, always-on-top, halbtransparent; ein Pill-Fenster mit Play/Pause/Status.
- Beide Fenster abonnieren denselben Rust-State; die Talent-Ansicht ist eine reine Render-Route (`/talent`), ohne eigene Logik.

### 4.2 Navigationsstruktur
- **Bibliothek** → Skript → **Edit** ⇄ **Live** (Modes with a seam)
- **Einstellungen** (eigenes Sheet): Allgemein · Darstellung/Themes · Scroll · Voice & Modelle · Eingabe/Hotkeys · Ausgabe/Monitore · Fernsteuerung · Über/Lizenzen
- **Command Palette** als universeller Einstieg: Aktionen, Skripte, Einstellungen, Themes.

---

## 5. User Flows

### F1 – Skript importieren
1. Ctrl/⌘O, Drag&Drop auf Fenster oder Ctrl/⌘V im leeren Zustand.
2. `zt-core::import` erkennt Format (Endung + Sniffing), parst in `Script` (Blöcke, Tokens, Regieanweisungen).
3. Fehler (Encoding, defekte Datei) → Toast mit Ursache, Datei bleibt unverändert; Rohtext-Fallback angeboten.
4. Skript öffnet im Edit-Modus, Vorschau rechts zeigt Zeilenumbruch in Talent-Geometrie, Hinweis bei Zeilen > 40 Zeichen.
5. Autosave in Bibliothek (`<data_dir>/scripts/*.md` + Metadaten-JSON).

### F2 – Proben
1. Wechsel zu Live (Tab/Ctrl+Enter). Text wird gesperrt (read-only), Layout eingefroren.
2. Modus „Konstant“, Space → Countdown 3-2-1 → Scroll mit WPM.
3. ↑/↓ ändern Tempo (Toast „150 WPM“), PgUp/PgDn springen Absätze, Home an den Anfang.
4. Pause → Doppelklick auf Text → Inline-Edit (0.2) bzw. zurück zu Edit (M); Position bleibt per Token-Anker erhalten.
5. Ende des Skripts → Stop, „Ende“-Marker, Probendauer angezeigt.

### F3 – Live gehen
1. Monitor wählen (Inspector „Ausgabe“), Talent-Fenster öffnet Vollbild; Spiegelung H/V setzen.
2. Optional „Profil laden“ (0.2).
3. Preflight-Check in der Transportleiste: Mikro ok, Modell geladen, Monitor aktiv, Spiegelung. Rote Punkte sind anklickbar.
4. Space → Countdown → Start im gewählten Modus.
5. Ad-hoc-Eingriffe über Hotkeys/Remote immer möglich; manuelle Eingabe übersteuert Voice kurzzeitig (Override 2 s, danach Resync).

### F4 – Voice-Tracking
1. Erstnutzung: Erklärung → OS-Mikrofonberechtigung → bei Ablehnung Deeplink in Systemeinstellungen.
2. Modell fehlt → Modell-Manager-Dialog (Größe, Lizenz, Quelle) → Download mit Fortschritt/Resume → SHA-256-Prüfung.
3. Mikrotest: Pegel + Live-Transkript, Zustand „listening“.
4. V oder Moduswahl „Voice“ → Hotwords aus Skript werden gebaut, Recognizer startet (Zustand `listening`).
5. Erstes Match → `tracking`: Scroll folgt, aktuelles Wort hervorgehoben.
6. Stille/Unsicherheit → `hold` (Text steht, gelber Ring). Ad-lib → `hold`, kein Scrollen.
7. > 4 s ohne Match → `lost`: nur Operator sieht Warnung; Verhalten gemäß Einstellung (Stop | Konstant).
8. Sprung im Text (Sprecher überspringt Absatz) → nach 2 bestätigten Hypothesen `resync` mit Ease ≤ 300 ms.
9. Klick auf Wort / Hotkey „Resync hier“ setzt Cursor hart.

### F5 – Fernsteuerung
- **Tastatur/Pedal (M):** Presenter/Pedal senden Tasten; Mapping im Shortcut-Recorder.
- **Handy (0.2):** Einstellungen → Fernsteuerung aktivieren → lokaler Server startet auf LAN-IP → QR-Code + 6-stellige PIN → PWA öffnet, Pairing → Token (Session) → Bedienelemente: Play/Pause, Tempo, Absatz ±, Spiegeln, Modus, Status + Mini-Vorschau. Verbindungsabbruch → Prompter läuft unverändert weiter, Operator-Hinweis.
- **MIDI/HID/Gamepad (0.2):** Learn-Key: Aktion wählen → „Taste drücken…“ → Gerät + Event gespeichert.

---

## 6. Zustandsmodell

### 6.1 Applikationszustand (Rust, Single Source of Truth)
```rust
struct AppState {
    script: ScriptState,          // Dokument, Version, Layout-Map (Token → Zeile/Y)
    mode: UiMode,                 // Edit | Live
    playback: PlaybackState,      // Stopped | Countdown{n} | Playing | Paused | Ended
    scroll: ScrollState,          // mode: Manual|Constant|VoiceActivated|VoiceTracking,
                                  // position_px, target_px, velocity, wpm
    voice: VoiceState,            // siehe 6.2 + confidence, cursor_token, rate_wps
    output: OutputState,          // monitor_id, fullscreen, mirror_h, mirror_v, overlay
    settings: Settings,           // Theme, Typografie, Marker, Hotkeys, Profile
    remote: RemoteState,          // Off | Listening{addr,pin} | Connected{clients}
    models: ModelsState,          // installiert, Downloads mit Fortschritt
}
```
Änderungen nur über `Command`-Enum (`Play`, `Pause`, `SetWpm(u16)`, `JumpParagraph(i32)`, `SeekToken(usize)`, `SetMirror{h,v}`, `SetScrollMode(..)`, `ToggleVoice`, …). Jede Quelle (UI, Hotkey, Remote, MIDI) erzeugt Commands → `zt-core::Controller` → neuer State → Event `state://patch` an alle Fenster/Clients.

### 6.2 Playback × Mode
```
Edit ──(Live)──▶ Live/Stopped ──Space──▶ Countdown ──▶ Playing ⇄ Paused
                      ▲                                 │
                      └──────────── Stop/Home ◀── Ended ◀┘
Edit nur aus Stopped/Paused erreichbar; im Playing ist der Text gesperrt.
```

### 6.3 Voice-Automat (`zt-align`)
| Von | Ereignis | Nach | Wirkung |
|---|---|---|---|
| idle | Voice an | listening | Recognizer startet |
| listening | Match (Score ≥ θ) | tracking | Ziel = Match-Position |
| tracking | Match | tracking | Ziel nachführen, Extrapolation |
| tracking | VAD-Stille oder Score < θ | hold | Ziel einfrieren |
| hold | Match nahe Cursor | tracking | weiter |
| hold/tracking | t_nomatch > 4 s (konfig.) | lost | Operator-Warnung, Policy Stop/Konstant |
| tracking/hold/lost | Sprung > N Zeilen, 2× bestätigt | resync | Ease ≤ 300 ms → tracking |
| lost | globaler N-Gramm-Match | resync | |
| * | Voice aus / Stop | idle | Recognizer stoppt |
| * | manueller Eingriff | hold (Override 2 s) | danach Matching ab neuem Cursor |

Parameter (Defaults, per Fixture getunt): Fenster −10/+60 Wörter, k = 6, θ = 0.55, N = 3 Zeilen, Hotwords-Score 1.75.

---

## 7. Nicht-funktionale Anforderungen

| Bereich | Anforderung | Messung |
|---|---|---|
| Voice-Latenz (Sprechen → Scroll-Ziel) | p50 ≤ 250 ms, p95 ≤ 350 ms | Fixture-WAVs mit Zeitstempeln, Nightly |
| Input-Latenz (Taste/Remote → sichtbar) | ≤ 50 ms lokal, ≤ 100 ms WLAN | Playwright + Log-Timestamps |
| Alignment-Rechenzeit | < 5 ms pro Hypothese | criterion-Benchmark |
| Scroll-FPS | 60 FPS stabil (≤ 1 % Frames > 20 ms) auf Win/mac; Linux ≥ 50 FPS im Perf-Modus; 120 Hz best effort | rAF-Frametime-Logger (Dev-Overlay) |
| CPU | Voice aktiv ≤ 25 % eines Kerns auf Referenz (4-Kern, 2019) | Benchmark |
| RAM | ≤ 250 MB mit Tier S | Messung |
| Kaltstart | ≤ 1,5 s bis Control-Fenster interaktiv | CI-Messung |
| Installer | < 15 MB ohne Modelle (onnxruntime inklusive, Ziel; im CI überwacht) | CI-Größenbudget, Fail bei Überschreitung + 10 % |
| Frontend-Bundle | JS ≤ 250 KB gz, CSS ≤ 40 KB gz, Fonts nur gebündelte Subsets | Vite-Budget im CI |
| Offline | 100 % funktionsfähig nach Modell-Download; kein Telemetrie-Default | E2E im Offline-Modus |
| Datenschutz | Kein Audio auf Platte (außer explizitem Debug-Recording); Remote nur LAN, PIN, opt-in | Review |
| Robustheit | Parser crasht nie (fuzz); Mikro-Abriss → hold + Hinweis, kein Absturz | cargo-fuzz, Tests |
| Barrierefreiheit | WCAG 2.2 AA Chrome, Themes Kontrast ≥ 4,5:1, Zustandsfarben ≥ 3:1 | CI-Kontrastcheck, axe |
| Plattformen | Win 10 2004+, macOS 13+ (aarch64/x86_64), Ubuntu 22.04+ (X11/Wayland) | CI-Matrix |

---

## 8. Architekturübersicht

### 8.1 Schichten
```
apps/desktop (Svelte 5 UI: Control + /talent + Palette)
        │  invoke(Command) / listen(state://patch, voice://word, scroll://frame)
apps/desktop/src-tauri (dünn: Fenster, Plugins, Command-Bridge)
        │
zt-core::Controller  ◀── zt-input (Hotkeys/MIDI/HID/Gamepad → Command)
   │   │            ◀── zt-remote (WS → Command, State → WS)
   │   └── ScrollEngine (Feder, Tick 120 Hz) 
   │
   └── VoiceSession ── zt-audio ─▶ zt-stt ─▶ zt-align ─▶ ScrollEngine.target
                                     ▲
                                 zt-models (Pfade, Download, Verify)
```

### 8.2 Crates & Schnittstellen (verbindliche Skizze)

**zt-core** – Skriptmodell, Import, Settings, Themes, Controller, Scroll-Engine
```rust
pub struct Script { pub id: ScriptId, pub blocks: Vec<Block>, pub tokens: Vec<Token>, pub version: u64 }
pub struct Token { pub idx: usize, pub text: String, pub norm: String, pub block: usize, pub kind: TokenKind /* Word | Cue */ }
pub fn import(bytes: &[u8], hint: Format) -> Result<Script, ImportError>;
pub fn export_markdown(s: &Script) -> String;

pub struct LayoutMap { pub token_y: Vec<f32>, pub line_of: Vec<u32>, pub paragraph_y: Vec<f32> } // vom Frontend gemessen
pub struct ScrollEngine { /* pos, vel, target, mode */ }
impl ScrollEngine {
    pub fn tick(&mut self, dt: Duration) -> ScrollFrame;          // kritisch gedämpfte Feder
    pub fn set_target_token(&mut self, idx: usize, layout: &LayoutMap);
    pub fn set_constant(&mut self, px_per_s: f32);
}
pub enum Command { Play, Pause, Stop, SetWpm(u16), Nudge(i32), JumpParagraph(i32), SeekToken(usize),
                   SetScrollMode(ScrollMode), ToggleVoice, SetMirror{ h: bool, v: bool }, ApplySettings(SettingsPatch), /* … */ }
pub struct Controller { /* AppState + Kanäle */ }
impl Controller {
    pub fn dispatch(&self, cmd: Command, src: Source) -> Result<(), CmdError>;
    pub fn subscribe(&self) -> broadcast::Receiver<StatePatch>;
}
pub struct Theme { /* Tokens, serde JSON */ }  pub fn contrast_check(t: &Theme) -> Vec<ContrastIssue>;
```

**zt-align** – pure Logik, keine IO, `no_std`-fähig angestrebt
```rust
pub struct AlignConfig { pub back: usize, pub fwd: usize, pub k: usize, pub theta: f32, pub jump_lines: u32, pub lost_after: Duration }
pub struct Aligner { /* Skript-Normform, Cursor, Automat */ }
impl Aligner {
    pub fn new(words: &[NormWord], cfg: AlignConfig) -> Self;
    pub fn on_hypothesis(&mut self, h: &Hypothesis, now: Instant) -> AlignEvent;
    pub fn on_vad(&mut self, speaking: bool, now: Instant) -> AlignEvent;
    pub fn seek(&mut self, word_idx: usize);
    pub fn state(&self) -> TrackState; // Idle|Listening|Tracking|Hold|Lost|Resync
}
pub struct Hypothesis { pub words: Vec<String>, pub is_final: bool, pub t: Instant }
pub enum AlignEvent { None, Advance{ idx: usize, confidence: f32, rate_wps: f32 }, State(TrackState), Jump{ idx: usize } }
pub fn normalize(text: &str, lang: Lang) -> Vec<NormWord>; // Lowercase, Satzzeichen, Zahlen→Wörter, Umlaut-Folding
```

**zt-audio** – cpal + Ringbuffer + rubato (16 kHz mono) + VAD-Frames
```rust
pub fn list_inputs() -> Vec<InputDevice>;
pub struct AudioStream; impl AudioStream { pub fn start(dev: Option<DeviceId>, tx: Sender<AudioFrame>) -> Result<Self, AudioError>; pub fn level(&self) -> f32; }
pub struct AudioFrame { pub samples: Arc<[f32]>, pub t: Instant } // 16 kHz, 30 ms
```

**zt-stt** – Engine-Abstraktion
```rust
pub trait SpeechEngine: Send {
    fn start(&mut self, cfg: &SttConfig /* model path, lang, hotwords */) -> Result<(), SttError>;
    fn accept(&mut self, f: &AudioFrame);
    fn poll(&mut self) -> Option<SttEvent>; // Partial/Final(Hypothesis), Vad(bool)
    fn stop(&mut self);
}
// impls: SherpaOnline (default), feature "whisper", feature "macos-speech", feature "vosk"
```

**zt-models** – Manifest (minisign), Download (reqwest Range-Resume), SHA-256, atomarer Rename
```rust
pub async fn manifest() -> Result<Manifest, ModelError>;
pub async fn install(id: &str, progress: impl Fn(u64, u64)) -> Result<PathBuf, ModelError>;
pub fn installed() -> Vec<InstalledModel>; pub fn remove(id: &str) -> io::Result<()>;
```

**zt-input** – Hotkeys/Geräte → `Command`
```rust
pub struct Binding { pub trigger: Trigger /* Key|Midi|Hid|Gamepad */, pub action: ActionId, pub global: bool }
pub fn start(bindings: Vec<Binding>, tx: Sender<Command>) -> InputHandle;
pub fn learn(timeout: Duration) -> Result<Trigger, LearnError>;
```

**zt-remote** – axum + WebSocket + PWA-Assets + QR
```rust
pub async fn serve(cfg: RemoteConfig, ctrl: ControllerHandle) -> Result<RemoteHandle, RemoteError>;
// WS-Protokoll v1 (JSON): C→S {"t":"hello","pin":"123456"} | {"t":"cmd","cmd":{...}} ; S→C {"t":"state","patch":{...}} | {"t":"err",...}
```

### 8.3 Tauri-Bridge (Frontend ↔ Rust)
- Commands: `dispatch(cmd)`, `get_state()`, `import_file(path)`, `save_script(id, md)`, `report_layout(script_version, LayoutMap)`, `list_monitors()`, `open_talent(monitor)`, `models_list/install/remove`, `audio_devices()`.
- Events: `state://patch` (bei Änderung), `scroll://frame` (Ziel-Position + Geschwindigkeit, max. 60 Hz – Frontend interpoliert per rAF mit `translate3d`), `voice://word` (Cursor-Token + Konfidenz), `voice://level` (15 Hz), `models://progress`.
- **Layout-Ownership:** Das Frontend misst Token-Positionen (Range-Rects) nach jedem Layout-Wechsel und meldet `LayoutMap` mit `script_version`; Rust verwirft veraltete Maps.

### 8.4 Persistenz
- `<config_dir>/zero-teleprompter/settings.json` (versioniert, Migrationen), `themes/*.json`, `bindings.json`.
- `<data_dir>/zero-teleprompter/scripts/*.md` + `library.json`; `models/<id>/`.

---

## 9. MVP-Implementierungsplan (v0.1)

Annahme: 1 Senior-Dev, ~10 Wochen. Jeder Meilenstein endet mit lauffähigem Build in der CI-Matrix (Win/mac/Linux) und Definition of Done (DoD).

### M0 – Fundament (Woche 1)
- Cargo-Workspace + Crates (leer, mit Traits), `apps/desktop` (Tauri 2.12.x, Svelte 5, TS, Vite), Bits UI.
- Crate-Versionen per `cargo search` prüfen (Risiko #9), Renovate einrichten.
- CI: fmt, clippy -D warnings, cargo-deny (GPL-Blocker), license-checker, nextest, Vitest; Build-Artefakte für 3 OS; Größenbudget-Check.
- `LICENSE-MIT`, `LICENSE-APACHE`, `THIRD_PARTY_NOTICES.md`, `THIRD_PARTY_MODELS.md`, ADR-001–004 nach `docs/adr`.
- **DoD:** Leeres Fenster startet auf 3 OS aus CI-Artefakt.

### M1 – Scroll-Spike & Talent-Renderer (Woche 2) – **Risiko-Gate #1**
- `ScrollEngine` (Feder, konstantes Tempo) in `zt-core` mit Unit-Tests.
- Zweites Fenster `/talent`, rAF + `translate3d`, Spiegelung via CSS-Transform, Fokuslinie, Dimmen des gelesenen Texts.
- Frametime-Logger-Overlay; Linux: `WEBKIT_DISABLE_DMABUF_RENDERER=1` bei NVIDIA, Performance-Modus.
- **DoD:** 10.000-Wort-Skript scrollt mit 60 FPS (Win/mac), ≥ 50 FPS Linux. Wenn nicht: Fallback-Entscheidung (natives Talent-Fenster) vor M2 dokumentieren.

### M2 – Skriptmodell, Import, Editor (Woche 3)
- `Script`/`Token`/`Block`, Importer txt/md/Zwischenablage, Regieanweisungen, `normalize()` (DE/EN, Zahlen ausschreiben).
- Editor (CodeMirror 6 oder textarea-basiert, Markdown), Autosave, Bibliothek minimal (zuletzt geöffnet).
- LayoutMap-Messung im Frontend + `report_layout`.
- Tests: insta-Snapshots Parser, proptest „Parser crasht nie“, cargo-fuzz-Target angelegt.
- **DoD:** Datei öffnen → editieren → Live → scrollen; Neustart behält Skript.

### M3 – Controller, Live-Modus, Hotkeys, Anpassbarkeit (Woche 4–5)
- `Controller` + `Command` + `state://patch`; Playback-Automat inkl. Countdown, Edit/Live-Seam (Sperre).
- App-Hotkeys + Remapping + Shortcut-Recorder, Command Palette, `?`-Overlay.
- Inspector: Font (gebündelt), Größe, Zeilenbreite/-abstand, Farben, Marker (Linie/Band), Spiegelung H/V pro Fenster, WPM.
- Themes als JSON-Tokens, 3 Presets, Kontrast-Check im CI; Settings-Persistenz mit Version.
- Monitorwahl, Vollbild, Transportleiste.
- **DoD:** Kompletter Proben- und Live-Flow (F1–F3) ohne Voice; alle Hotkeys remappbar; Playwright-E2E für F1/F2.

### M4 – Audio, Modelle, STT (Woche 6–7)
- `zt-audio`: cpal-Geräteliste, Stream, rubato 16 kHz, Pegel; macOS Info.plist + Entitlement.
- `zt-models`: signiertes Manifest, Download mit Resume, SHA-256, atomic rename, Modell-Manager-UI mit Lizenzanzeige. Kroko-de-Lizenz klären (Risiko #2) – bis dahin nur Download, nicht bündeln.
- `zt-stt`: `SherpaOnline` (sherpa-onnx 1.13.x), Hotwords aus Skript, `modified_beam_search`, eigener Thread, integriertes VAD; onnxruntime-Bundling pro OS.
- Onboarding: Erklärung → Berechtigung → Mikrotest mit Live-Transkript.
- **DoD:** Live-Transkript im Mikrotest auf 3 OS; Latenzmessung Audio → Partial geloggt.

### M5 – Aligner & Voice-Tracking (Woche 7–9) – **Kern**
- `zt-align`: Fenster-Smith-Waterman mit `strsim`-Kosten, Final > Partial-Gewichtung, Hysterese, Backtrack (autocue-Muster, Attribution), Automat (6.3), Tempo-Schätzung.
- Kopplung VoiceSession → `ScrollEngine.set_target_token`; Wort-Highlight; Statusanzeige Operator (Farbe + Form); Policy „lost“ (Stop | Konstant); Klick-auf-Wort → `seek`; manueller Override.
- Tests: synthetische Token-Streams (Einfügen, Auslassen, Ad-lib, Absatzsprung, Rücksprung) in jedem PR; proptest Monotonie; criterion < 5 ms; 5–10 CC0-WAV-Fixtures (DE+EN) nightly mit ±2 Wörter-Toleranz.
- **DoD:** Fixtures grün; p50-Latenz ≤ 250 ms auf Referenzhardware gemessen; Ad-lib von 10 s erzeugt kein Scrollen.

### M6 – Härtung & Release v0.1 (Woche 10)
- Fehlerpfade: Mikro-Abriss, Modell defekt, Monitor getrennt, Berechtigung entzogen.
- Accessibility-Pass (Fokus-Ring, Zielgrößen, reduced-motion, Screenreader-Labels im Chrome), DE/EN-Strings.
- NFR-Messung (Kaltstart, RAM, CPU, Installergröße) und Dokumentation; mdBook „Erste Schritte“.
- Release über `tauri-action` bei `v0.1.0`; macOS notarisiert, sofern Developer-ID vorhanden; Windows unsigniert mit Hinweis (Signing in 0.2, Risiko #4).
- **DoD:** Installer < 15 MB, alle CI-Gates grün, Release-Notes, bekannte Einschränkungen dokumentiert.

### Parallel/blockierend außerhalb der Entwicklung
- Kroko-de-Lizenz (vor M4), FTO/Patentprüfung US 9,953,646 ff. (vor kommerziellem Release), Windows-Signing-Weg (vor 0.2), Referenzhardware festlegen (vor M5).

### Ausblick
- **v0.2 (~6 Wochen):** Overlay-Modus, Screen-Capture-Exclusion, globale Hotkeys + Wayland-Portal, Handy-Remote (PWA/QR/PIN), MIDI/HID/Gamepad, .docx, alle Themes + eigene Themes, Profile, Sprachaktiviert-Modus, Inline-Edit, Leseabstand-Assistent, Updater, Signing.
- **v1.0:** Parakeet-Re-Sync, Vosk-Fallback, macOS SpeechAnalyzer, Notch/Pill, mDNS/OSC/OBS, Director-Remote-Editor, Theme-Editor, 120 Hz best effort, mdBook komplett.

---

## 10. Offene Produktentscheidungen
1. Default bei „lost“: **Stop** (Vorschlag aus Research #11), umschaltbar – bestätigt für MVP.
2. Editor-Komponente: CodeMirror 6 (Bundle +~120 KB gz) vs. eigenes contenteditable – Entscheidung in M2 per Spike.
3. Telemetrie: keine im MVP; ggf. opt-in Crash-Reports ab 0.2.
4. Mindesthardware für Voice-Tracking: nach M5-Benchmark festlegen.
