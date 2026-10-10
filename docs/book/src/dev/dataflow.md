# Datenfluss

## 1. Nutzeraktion → Zustand → Fenster

```
Taste / Klick / Palette (Svelte)
  └─ store.send(cmd) → api.dispatch(cmd) ─invoke("dispatch")─▶ commands::dispatch
                                                              └─ appcore::dispatch
globaler Hotkey (Ctrl+Shift+…) ─────────────────────────────────┘
  lock(Shared)
    patch = Controller::apply(cmd)
    Skript geändert oder Sprache geändert → ScriptIndex neu, tracker.set_index
    Tracker-relevante Settings geändert   → tracker.set_config (Cursor bleibt erhalten)
    Seek (Absatz, SeekToken, ResyncHere, Stop) bei aktivem Voice → TrackerEvent::ManualSeek
  unlock
  emit state://patch   (nur die geänderten Teilbäume, camelCase)
  emit scroll://frame  (wenn sich scroll geändert hat)
  Voice umgeschaltet   → voice::set_tracking(on)
  Settings geändert    → persist::save_settings (atomar: .tmp, dann rename)
```

Fehler aus `apply` (zum Beispiel `ScriptLocked`) kommen als String zurück. Das Frontend zeigt sie als Toast an.

## 2. Zeit

`zt-tick` sendet etwa alle 16 ms `Command::Tick(dt)`. Bei aktivem Voice sendet er zusätzlich `TrackerEvent::Tick`. Um IPC zu sparen, filtert der Tick:

- Während des Countdowns geht ein `playback`-Patch nur dann raus, wenn die angezeigte Sekunde wechselt.
- Das Herunterzählen von `voice.overrideMs` wird erst gemeldet, wenn es 0 erreicht.
- `scroll://frame` überträgt `position`, `target`, `velocity` (Tokens/s), `playing` und `scriptVersion`. Zwischen zwei Frames extrapoliert das Frontend (`scroller.ts`, Feder + Vorhersage) und rendert per rAF.

## 3. Voice

```
cpal-Callback ─try_send─▶ sync_channel(64) ─▶ zt-voice-Thread
   (kein Mikrofon: alle 50 ms 800 Samples Stille, damit Demo-/Mock-Engines weiterlaufen)
   ├─ alle ≥66 ms: emit voice://level { db }
   ├─ engine.accept_waveform(frame); for ev in engine.poll():
   │     Partial/Final → emit voice://transcript { text, isFinal } → TrackerEvent::Hypothesis
   │     Vad(true/false) → VadSpeech / VadSilence
   └─ (nur tracking) tracker.handle(ev, now_ms) → apply_tracker_output
          nur bei Änderung: Command::Voice(VoiceSignal) → state://patch (ohne scroll)
          Cursor bewegt    → emit voice://word { token, confidence }
```

Beim Start sendet die App einmalig `voice://engine { name, demo, microphone, error }`.

Das Resampling auf 16 kHz mono übernimmt `CpalSource` selbst, noch im Callback (`Resampler::to_16k(rate, channels)`). Der Voice-Thread bekommt deshalb bereits Frames im Engine-Format.

## 4. Persistenz

| Datei | Schreiber | Inhalt |
|---|---|---|
| `settings.json` | `persist::save_settings` nach jedem Settings-Patch | `{ "version": 1, "settings": Settings }`. Neuere Versionen werden ignoriert |
| `ui-prefs.json` | Command `set_ui_prefs` | freies JSON (`UiPrefs`), wird an alle Fenster gesendet |
| `scripts/<id>.md` | `save_script` | `id` muss `[A-Za-z0-9_-]{1,64}` entsprechen (Schutz vor Path-Traversal) |

## 5. Start

`run()` lädt die Settings (Fallback `Settings::from_tokens`), erzeugt den `Controller`, lädt ein leeres Skript, registriert `Shared`, startet die Tick-Schleife und registriert die globalen Hotkeys. Das Frontend lädt danach `lastScriptId` oder das Beispielskript.
