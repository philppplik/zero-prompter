# ADR-005: Code-Architektur – Hexagonal (Ports & Adapters) mit reinem Kern

- **Status:** Angenommen
- **Datum:** 2026-10-10
- **Kontext-Dokumente:** `docs/research/RESEARCH.md`, `docs/concept/CONCEPT.md` (§6 State/Automaten, §7 NFR), `docs/design/tokens.json`

## Kontext

zero-teleprompter verbindet Echtzeit-Audio, lokale Spracherkennung (sherpa-onnx), Wort-Alignment, eine Scroll-/Playback-Logik und eine Tauri/Svelte-Oberfläche mit zwei Fenstern und optionaler Fernsteuerung. Die kritischen Anforderungen:

- Voice-Latenz p95 ≤ 350 ms, Alignment < 5 ms pro Hypothese.
- „Nicht raten“: Fehlverhalten (davonlaufender Text) ist schlimmer als Stehenbleiben. Das Verhalten in Grenzfällen (Ad-lib, Sprung, Stille, Rücksprung) muss **reproduzierbar testbar** sein.
- Mehrere Eingabequellen (UI, Hotkeys, Remote, MIDI, Voice) dürfen keinen inkonsistenten Zustand erzeugen.
- Drei Plattformen; Audio-/ONNX-Bibliotheken sind die Hauptquelle für Build- und Laufzeitprobleme.
- Das Team (und CI) muss die Kernlogik ohne Mikrofon, Modelle oder GUI testen können.

## Entscheidung

Wir strukturieren den Rust-Teil als **hexagonale Architektur (Ports & Adapters)** in einem Cargo-Workspace:

```
                ┌──────────────────── Adapter (I/O) ────────────────────┐
  Mikrofon ──▶  zt-audio::CpalSource ──┐                                │
                                        ▼  Port: AudioSource            │
                zt-stt::SherpaEngine ◀── Port: SpeechEngine ──▶ MockEngine
                        │ SttEvent                                       │
                        ▼                                                │
           ┌────────── reiner Kern (kein I/O, keine Uhr) ──────────┐     │
           │ zt-align: normalize → Smith-Waterman → VoiceTracker   │     │
           │            │ TrackerOutput → (App mappt) Command::Voice│     │
           │ zt-core:   Controller.apply(Command) → AppState+Patch │     │
           └───────────────────────────────────────────────────────┘     │
                        │ StatePatch                                     │
                        ▼                                                │
                Tauri-Adapter (App-Crate, später) → state://patch → Svelte
  Netz ──▶      zt-models::Downloader (Port) ← HTTP-Adapter in der App   │
                └────────────────────────────────────────────────────────┘
```

| Crate | Rolle | Abhängigkeiten |
|---|---|---|
| `zt-core` | Domänenkern: `AppState`, `Command`, `Controller`, Playback-Automat, Skript-Tokenisierung, Settings/Themes aus `tokens.json` | serde, serde_json, thiserror |
| `zt-align` | Reiner Algorithmus: Normalisierung DE/EN, Fuzzy-Ähnlichkeit, gefensterter Smith-Waterman, Hysterese, Voice-Automat | **keine** (serde optional) |
| `zt-audio` | Port `AudioSource`, Adapter `CpalSource` (Feature `cpal`), `MockSource`, Resampler → 16 kHz mono | thiserror, cpal (optional) |
| `zt-stt` | Port `SpeechEngine`, Adapter `SherpaEngine` (Feature `sherpa`), `MockEngine`, Hotword-Auswahl | thiserror, sherpa-onnx (optional) |
| `zt-models` | Manifest, SHA-256-Prüfung, Install-Layout, Port `Downloader` | serde, sha2, thiserror |

Regeln:

1. **Abhängigkeitsrichtung nach innen.** `zt-core` und `zt-align` kennen weder Tauri noch Audio, Dateien, Threads oder Uhren. `zt-core` hängt nicht einmal von `zt-align` ab; die App übersetzt `TrackerOutput` in `Command::Voice(VoiceSignal)`.
2. **Zeit wird injiziert.** `Command::Tick(dt_ms)` bzw. `VoiceTracker::handle(event, t_ms)`. Dadurch sind Szenarien wie „4 s ohne Match → lost“ in Mikrosekunden testbar und deterministisch.
3. **Ein Schreibpfad.** Alle Zustandsänderungen laufen über das `Command`-Enum und `Controller::apply` (CONCEPT §6.1). Der Controller liefert ein `StatePatch` (welche Teilbäume sich änderten); der Adapter sendet nur diese an die Fenster.
4. **Unsichere Fremd-APIs hinter eigenen Traits.** cpal und sherpa-onnx sind nur in je einem Adapter-Modul sichtbar, hinter Feature-Flags, und an der Stelle mit `// VERIFY` markiert. Ein API-Bruch einer Abhängigkeit betrifft genau eine Datei.
5. **Mocks sind erstklassige Adapter.** `MockSource` und `MockEngine` erlauben End-to-End-Läufe (Audio → STT → Align → Core) in CI ohne Mikrofon und Modelle sowie einen Demo-Modus.
6. **`unsafe_code = "forbid"`** workspace-weit. FFI-Unsicherheit bleibt in den Upstream-Bindings.

## Warum kein Framework für den Kern

Geprüft wurden u. a. ein Actor-Framework (actix/ractor), ein ECS (bevy_ecs), Redux-artige Store-Crates und eine async-Runtime (tokio) als Rückgrat des Kerns.

- **Problemgröße:** Der Kern ist ein Reducer (`State × Command → State`) plus zwei kleine Automaten. Das lässt sich in wenigen hundert Zeilen klar ausdrücken; ein Framework bringt mehr Konzepte als Probleme löst.
- **Determinismus und Testbarkeit:** Actor-/async-Frameworks führen Scheduling-Nichtdeterminismus und versteckte Uhren ein. Der Kern muss synchron und reproduzierbar sein (Fixture-Tests, Fuzzing, criterion-Benchmarks).
- **Latenzbudget:** Das Alignment läuft pro Hypothese in < 5 ms im STT-Thread. Message-Passing-Overhead und Allokationen eines Frameworks sind dort unerwünscht; reine Funktionen auf Slices sind am schnellsten und am leichtesten zu benchmarken.
- **Lizenz und Lieferkette:** Jede Abhängigkeit muss durch `cargo-deny` (MIT/Apache-kompatibel). `zt-align` hat null Laufzeit-Abhängigkeiten.
- **Austauschbarkeit:** Tauri ist unsere Shell-Entscheidung, aber nicht Teil des Kerns. Ein späterer Headless-Modus (Remote-only), ein CLI für Fixture-Läufe oder ein anderer UI-Stack nutzen denselben Kern unverändert.
- **Nebenläufigkeit gehört an den Rand:** Threads (Audio-Callback, STT-Thread, UI-Event-Loop) und Kanäle leben in der App-Schicht. Dort ist Tauris eigene tokio-Runtime ohnehin vorhanden; der Kern braucht sie nicht.

## Konsequenzen

**Positiv**
- Grenzfälle des Voice-Trackings sind als schnelle Unit- und Szenario-Tests abgedeckt (`crates/zt-testkit/tests/tracker_scenarios.rs`, Fixtures in `crates/zt-testkit/src/fixtures.rs`; Property-Tests in `crates/zt-align/tests/prop_*.rs`).
- CI baut und testet den Kern ohne ALSA, onnxruntime oder Modelle (`--no-default-features`); die Adapter laufen in eigenen Jobs.
- API-Änderungen in cpal oder sherpa-onnx sind lokal begrenzt.

**Negativ / Risiken**
- Etwas Boilerplate: Spiegel-Enums (`VoicePhase` in `zt-core` ↔ `VoiceState` in `zt-align`) und eine Übersetzung in der App.
- Eigene Implementierungen statt Crates (Levenshtein/Jaro-Winkler statt `strsim`, FIR+Interpolation statt `rubato`). Sie sind klein und getestet; der Wechsel auf die Crates ist eine lokale Änderung, falls Benchmarks oder WER-Messungen es rechtfertigen.
- Die sherpa-onnx- und cpal-Adapter wurden ohne Toolchain geschrieben und sind mit `// VERIFY` markiert. Vor dem ersten Release müssen sie gegen die veröffentlichten Crate-Versionen kompiliert und mit Fixture-WAVs geprüft werden.

## Alternativen

| Alternative | Verworfen, weil |
|---|---|
| Logik im Svelte-Frontend | Zwei Fenster + Remote brauchen eine Quelle der Wahrheit; Latenz und Testbarkeit schlechter |
| Ein großes App-Crate | Kern nicht ohne Audio/Tauri baubar; längere Build-Zeiten; Kopplung |
| Actor-Framework im Kern | Nichtdeterminismus, Overhead, mehr Konzepte als nötig |
| async/tokio im Kern | Keine I/O im Kern; async würde nur Komplexität und Uhren einführen |
