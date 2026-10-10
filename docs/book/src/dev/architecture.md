# Architektur

Grundlage ist [ADR-005](../adr/index.md): eine **hexagonale Architektur (Ports & Adapters)** mit einem reinen, I/O-freien Kern.

## Schichten

```
              ┌──────────────────── Adapter (I/O) ────────────────────┐
 Mikrofon ──▶ zt-audio::CpalSource ──┐                                │
                                      ▼ Port: AudioSource             │
              zt-stt::SherpaEngine / MockEngine ◀── Port: SpeechEngine│
                      │ SttEvent                                      │
                      ▼                                               │
        ┌───────── reiner Kern (kein I/O, keine Uhr) ─────────┐       │
        │ zt-align: normalize → Smith-Waterman → VoiceTracker  │       │
        │           TrackerOutput ──(App mappt)──▶ Command::Voice│      │
        │ zt-core:  Controller::apply(Command) → AppState + StatePatch │
        └──────────────────────────────────────────────────────┘       │
                      │ StatePatch                                    │
                      ▼                                               │
 apps/desktop/src-tauri (Tauri-Adapter) ── Events ──▶ Svelte (2 Fenster)
 Netz ──▶ zt-models::Downloader (Port) ◀── UreqDownloader in der App   │
              └───────────────────────────────────────────────────────┘
```

## Regeln

1. **`zt-core` und `zt-align` haben keine Abhängigkeiten zu I/O, Threads oder Uhren.** Zeit kommt nur explizit herein: `Command::Tick(dt_ms)` bzw. `t_ms` in `VoiceTracker::handle`. Dadurch sind alle Grenzfälle deterministisch testbar.
2. **`zt-core` kennt `zt-align` nicht.** Die App übersetzt `TrackerOutput` in `VoiceSignal` (`appcore::apply_tracker_output`).
3. **Ein einziger Mutationspfad:** Jede Quelle (UI, In-App-Hotkey, globaler Hotkey, Tick-Schleife, Voice-Thread) läuft über `appcore::dispatch` bzw. `Controller::apply` hinter **einem** Mutex (`Shared`).
4. **Adapter hinter Features:** `cpal`, `sherpa` und `download` sind optional. Ohne sie bleiben Mock-Implementierungen. Die CI und die Kern-Tests brauchen weder Mikrofon noch Modell noch GUI.
5. **Rust ist die Single Source of Truth.** Das Frontend hält eine Kopie des `AppState`, die es über `state://patch` aktualisiert. Eigene Wahrheit hält es nur für reine UI-Präferenzen (`ui-prefs.json`) und das Layout (Pixel).
6. **Layout gehört dem Frontend.** Der Kern rechnet in Token-Einheiten (`scroll.position` als gebrochener Token-Index). Das Frontend bildet Tokens auf Pixel ab und meldet per `report_layout` die Zeilenzahl samt Skriptversion. In v0.1 wird dieser Wert nur gespeichert.

## Threads in der App

| Thread | Aufgabe |
|---|---|
| Tauri-Hauptthread | Fenster, Commands |
| `zt-tick` | ~60 Hz (Schlaf 16 ms, `dt` auf 1–250 ms begrenzt). Ruft `Command::Tick` und `TrackerEvent::Tick` auf und sendet `scroll://frame` |
| `zt-voice` | erzeugt den cpal-Stream **im eigenen Thread** (`cpal::Stream` ist nicht `Send`) und betreibt Engine → Tracker |
| cpal-Callback | schiebt Frames per `try_send` in einen begrenzten Kanal (Kapazität 64) und blockiert nie |
| `zt-model-dl` | Modelldownload, Fortschritt höchstens alle 100 ms |

## Fenster

| Label | Datei | Rolle | Capability |
|---|---|---|---|
| `control` | `index.html` → `App.svelte` | Operator-UI | `core:default`, Fenstertitel, `dialog:allow-open`, `dialog:allow-ask` |
| `talent` | `talent.html` → `Talent.svelte` | reine Darstellung. Schließen blendet nur aus | `core:default` |

Beide Fenster abonnieren dieselben Events. Die CSP erlaubt nur `self` und IPC.
