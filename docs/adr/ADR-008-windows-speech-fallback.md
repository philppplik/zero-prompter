# ADR-008: Windows-Spracherkennung als Fallback-Engine

- Status: angenommen
- Datum: 2026-10-11
- Bezug: ADR-006 (hybride STT, Fallback-Kette), ADR-007 (Fehlerbehandlung, Latenzmetriken)

## Kontext

Die Engines aus ADR-006 (Nemotron 3.5, Kroko, Zipformer 20M via sherpa-onnx) brauchen ein heruntergeladenes Modell. Fehlt es, kann es nicht geladen werden oder schlägt die Erzeugung fehl, landete die App bisher im Demo- bzw. Fehlerzustand. Windows bringt mit `Windows.Media.SpeechRecognition` eine System-Spracherkennung mit, die ohne Download nutzbar ist, sobald das Sprachpaket installiert ist.

## Entscheidung

1. Neuer Adapter `WinSpeechEngine` (`crates/zt-stt/src/windows_speech.rs`). Er kompiliert nur unter Windows mit dem Cargo-Feature `winspeech`. Das Feature ist in den Windows-Release-Builds (x64, ARM64, Store) aktiv und auf anderen Plattformen ein No-op.
2. **Selbst aufnehmende Engine:** Windows öffnet das Standardmikrofon selbst. Der `SpeechEngine`-Trait bekommt dafür `fn self_capturing(&self) -> bool` mit dem Default `false`. Bei `true` ignoriert die Engine `accept_waveform`. Die cpal-Pipeline läuft trotzdem weiter: Sie liefert Pegel, Kanalanzeige und Mikrofon-Watchdog und erkennt den Sprechbeginn für die Latenzmessung. Unter Windows dürfen mehrere Apps im Shared Mode gleichzeitig aufnehmen. Die Latenz wird nur als *Sprechbeginn → erster Partial* (`onset_to_partial`) gemessen, weil `audio_to_partial` und `end_to_end` ohne gemeinsame Audioblöcke nichts aussagen.
3. **Ereignisse:**
   - `SpeechRecognizer.HypothesisGenerated` → `SttEvent::Partial`
   - `ContinuousRecognitionSession.ResultGenerated` → `SttEvent::Final`. Ergebnisse mit der Konfidenz `Rejected` werden verworfen.
   - `StateChanged` (`SpeechDetected` / `SoundEnded`) → `SttEvent::Vad`
   - `Completed` (die Sitzung endet von selbst, z. B. weil das Mikrofon weg ist, das Netz fehlt oder ein Timeout greift) → automatischer Neustart
   - `EndSilenceTimeout` 400 ms, `AutoStopSilenceTimeout` 24 h
4. **Threading:** Alle WinRT-Objekte leben auf einem eigenen Thread `zt-winspeech` im MTA. windows-rs tritt beim ersten Aktivieren über `CoIncrementMTAUsage` implizit dem MTA bei, deshalb ist kein `unsafe` nötig und `unsafe_code = "forbid"` bleibt bestehen. Asynchrone Operationen (`CompileConstraintsAsync`, `StartAsync`, `StopAsync`) werden dort blockierend mit `.get()` abgewartet, nie im UI-Thread. Die Event-Handler laufen im WinRT-Threadpool und schieben nur in einen `mpsc`-Kanal. Der Voice-Thread pollt diesen Kanal wie bei jeder anderen Engine. Damit muss `WinSpeechEngine` unabhängig vom Agile-Status der WinRT-Klassen nur Kanäle halten und ist `Send`.
5. **Constraints:**
   - Standard ist `SpeechRecognitionTopicConstraint(Dictation)`, also freies Diktat.
   - Scheitert das Diktat (Datenschutz-Einstellung aus oder kein Netz) und ist für die Sprache eine Grammatik-Erkennung installiert, wechselt die Engine in den **Skriptwort-Modus**: eine `SpeechRecognitionListConstraint` aus den Skriptwörtern (Hotwords, max. 2000). Sie läuft offline, erkennt aber nur genau diese Wörter bzw. Phrasen. Sie startet, sobald `set_hotwords` die Wörter liefert.
   - Topic- und List-Constraints werden nicht gemischt.
6. **Fallback-Kette** (`apps/desktop/src-tauri/src/winspeech.rs`, `voice::make_engine`):
   1. installierte Modelle (Nemotron → Kroko → 20M)
   2. Windows-Spracherkennung
   3. Fehlerstatus (`engineError`, die Meldung nennt beide Ursachen). Ist gar kein Modell installiert, bleibt es beim Demo-Modus, und die Notiz erklärt, warum Windows nicht verfügbar war.
7. **Manuelle Wahl:** Unter Einstellungen → Voice steht „Windows Spracherkennung (System)“ (id `windows-speech`) in der Modellauswahl. Es gibt nichts herunterzuladen. `Selection.windowsSpeech` meldet, ob der Build die Engine enthält.
8. **Crate:** Gepinnt ist `windows = "=0.61.3"` mit den Features `Foundation`, `Foundation_Collections`, `Globalization` und `Media_SpeechRecognition`. 0.62.x setzt Rust 1.82 voraus, die Workspace-MSRV ist 1.80. Tauri zieht 0.61.3 ohnehin in den Lockfile, es entsteht also keine zusätzliche Kopie.

## Fehlerfälle und Hinweise im UI

| Fall | Erkennung | Meldung / Link |
|---|---|---|
| Sprachpaket fehlt | Sprache nicht in `SupportedTopicLanguages` bzw. `SupportedGrammarLanguages`, oder `SpeechRecognizer(Language)` schlägt fehl | „Windows-Sprachpaket für „de“ fehlt …“ → `ms-settings:regionlanguage` (Sprachoptionen → Spracherkennung), `ms-settings:speech` |
| Online-Spracherkennung aus | HRESULT `0x80045509` (SPERR_SPEECH_PRIVACY_POLICY_NOT_ACCEPTED) | → `ms-settings:privacy-speech`. Ohne Grammatik-Sprache Fehler, sonst Skriptwort-Modus |
| Mikrofon verweigert | `0x80070005` | → `ms-settings:privacy-microphone` |
| Kein Netz / Diktierdienst | Compile-Status `NetworkFailure` | Hinweis „Internet nötig“ |
| Sprache passt nicht | `TopicLanguageNotSupported` / `GrammarLanguageMismatch` | Hinweis auf das Sprachpaket |

## Einschränkungen (ehrlich)

- **Diktat ist ein Online-Dienst.** Unter Windows 10/11 braucht `SpeechRecognitionScenario.Dictation` die Einstellung „Online-Spracherkennung“ und eine Internetverbindung. Audio geht dabei an Microsoft. Das widerspricht dem Offline-Anspruch der App, deshalb ist die Engine nur **Fallback** bzw. eine bewusste manuelle Wahl.
- **Skriptwort-Modus ist grob:** Er erkennt nur Wörter aus der Liste, jedes Ergebnis ist kurz, und es gibt keine Partials in Wortfolge. Für das Tracking reicht das nur als Notbetrieb.
- **Unpackaged Desktop-App:** `SpeechRecognizer` und `ContinuousRecognitionSession` funktionieren laut Microsoft-Doku und Community-Berichten auch ohne Package Identity. Die Mikrofonfreigabe läuft dann über „Desktop-Apps Zugriff auf das Mikrofon erlauben“ statt über eine App-Capability. Auf echter Hardware **noch nicht verifiziert** (CI hat kein Mikrofon). Für den Store-Build (MSIX) wäre ggf. die Capability `microphone` im Manifest nötig, das ist offen.
- **Kein Hotword-Biasing im Diktat:** Die Topic-Constraint akzeptiert nur einen Hinweis-String, keine Skriptwörter.
- **Zweite Mikrofonöffnung:** Windows nimmt das *Standard*-Eingabegerät, nicht das in der App gewählte. Gain und Kanalwahl der App wirken nicht auf die Erkennung.
- **ARM64:** WinRT-APIs sind auf ARM64 nativ vorhanden. Gebaut und initialisiert wird in CI auf `windows-11-arm`, Erkennung mit echter Sprache ist nicht getestet.
- **Neuere Windows-AI-APIs** (Windows App SDK / „Windows AI Foundry“, Phi Silica, Copilot+-NPU-Modelle) bieten Stand 2026-10 keine allgemein verfügbare Streaming-STT-API für unpackaged Win32-Apps mit Deutsch. Außerdem bräuchten sie das Windows App SDK als Laufzeit und teils Copilot+-Hardware. Deshalb verworfen. Neu prüfen, sobald eine Streaming-ASR-API stabil ist.
- Die alte SAPI-5-Desktop-Erkennung (`Win32_Media_Speech`, „Windows-Spracherkennung“ Legacy) ist in Windows 11 abgekündigt (durch Voice Access ersetzt) und hat kein brauchbares deutsches Diktat. Deshalb verworfen.

## Konsequenzen

- Ohne Download ist unter Windows sofort Voice-Tracking möglich (bei aktivierter Online-Spracherkennung).
- Die Release-Builds hängen zusätzlich vom `windows`-Crate ab (MIT/Apache-2.0, ohnehin über Tauri vorhanden).
- CI-Job `winspeech` (windows-latest und windows-11-arm): clippy und Tests mit Feature, außerdem `examples/winspeech_probe` (Sprachen auflisten, Recognizer erzeugen, `CompileConstraintsAsync` ohne Mikrofon). Fehlt auf dem Runner das Sprachpaket oder die Einwilligung, meldet der Probe `SKIP`/`FAILED` als Information. Rot wird er nur, wenn eine Grammatik-Sprache installiert ist und die Offline-List-Constraint trotzdem nicht kompiliert.
