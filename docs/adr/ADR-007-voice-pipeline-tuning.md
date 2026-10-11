# ADR-007: Voice-Pipeline – Fehlerbehandlung, Diagnose und Latenz-Tuning

- Status: angenommen
- Datum: 2026-10-11
- Bezug: ADR-002 (cpal), ADR-006 (hybride STT), Nutzerfeedback zu v0.1.0 (Windows 11 ARM64, `--features sherpa`): „Sprache synchronisieren klappt nicht, in der UI steht Demo“

## Kontext

### Befund „Demo“ / Sync klappt nicht

Die Release-Builds enthalten sherpa (CI-Log Windows ARM64: `tauri build --features sherpa`, `zt-stt` mit Feature gebaut; das Feature wird korrekt über `apps/desktop/src-tauri/Cargo.toml` → `zt-stt/sherpa` weitergereicht). Der Fehler liegt in der Engine-Auswahl in `voice.rs`/`models.rs` von v0.1.0:

1. **Stiller Demo-Fallback bei Ladefehlern.** `make_engine` machte aus *jedem* `Err` von `build_engine` den Demo-Modus (`MockEngine::reading`). Die Demo-Engine liest das Skript mit 2,3 Wörtern/s selbst vor. Konnte ein installiertes Modell nicht geladen werden, scrollte der Text also unabhängig von der Stimme – für Nutzer genau „Sync klappt nicht“ plus Badge „Demo“. Der Fehler kam nur als kurzer Toast und wurde nirgends geloggt (`eprintln!` ist in einer Windows-GUI-App unsichtbar).
2. **Sprachbindung ohne Ausweichen.** Gesucht wurde nur unter Modellen der Skriptsprache (`settings.language`, Standard `de`). Wer z. B. nur Zipformer EN 20M (Apache-2.0) oder nur Parakeet (Rolle `resync`, kein Streaming) installiert hatte, bekam `Ok(None)` → Demo, obwohl ein Modell auf der Platte lag.
3. **Veraltete Engine nach Modell-Download.** Die Engine wird beim Start von Voice/Mikrofontest gebaut. Im Onboarding läuft der Mikrofontest *vor* dem Modell-Download; danach blieb `store.engine.demo = true` stehen, bis Voice neu gestartet wurde.
4. **Keine Diagnose.** Kein Log, keine Latenz- oder Pegelwerte, keine Erkennung von „Mikrofon geöffnet, liefert aber nichts“ (Windows-Datenschutz „Mikrofonzugriff für Desktop-Apps“ liefert Stille statt eines Fehlers), Panics im Voice-Thread verschwanden mit dem Thread.

Geprüft und **nicht** ursächlich: Feature-Weitergabe im Workspace; Modellpfade (Dateinamen kommen aus den Manifest-Rollen `encoder/decoder/joiner/tokens`); Sample-Rate (cpal → eigener Resampler → 16 kHz mono, sherpa `feat_config` 16 kHz/80 fbank laut Crate-Default); Silero (nur für Resync-Segmentierung, Fallback RMS); Hotwords (keines der mitgelieferten Modelle hat `bpe.vocab` → `set_hotwords` liefert `Unsupported`, der Recognizer bleibt unverändert – erwartbar, jetzt als Info geloggt).

## Entscheidung

### Engine-Auswahl ohne stille Fallbacks

| Situation | Engine | UI |
|---|---|---|
| Streaming-Modell der Skriptsprache installiert | HybridEngine (wie ADR-006) | Modellname |
| Nur Streaming-Modell anderer Sprache installiert | dieses Modell (`Manifest::streaming_plan`) | Hinweis „Kein Modell für Sprache … – nutze …“ |
| Kein Streaming-Modell installiert | Demo (`reason = noModel`) | Badge „Demo · Modell laden“ → Einstellungen/Voice |
| Build ohne `sherpa` | Demo (`reason = noSherpa`) | Badge + Hinweis |
| Modell installiert, Laden scheitert | `NullEngine` (`reason = engineError`), **kein** Demo | Fehler-Toast, Badge „Modellfehler · Details“ → Diagnose |

Nach erfolgreichem Modell-Download wird eine laufende Voice-Session neu gestartet.

### Fehlerpfad und Logging

- `tracing` + `tracing-subscriber` (EnvFilter, Variable `ZT_LOG`) + `tracing-appender` (tägliche Rotation, 7 Dateien) im App-Log-Verzeichnis (`app_log_dir`, Windows `%LOCALAPPDATA%\<identifier>\logs`).
- Ein Fehlerpfad: `diag::report_error` = Log + Ringpuffer (50) + Event `diag://error` → Toast. Panic-Hook loggt Panics aller Threads, der Voice-Thread läuft unter `catch_unwind` und meldet Abstürze.
- Strukturierte Events (Targets): `zt::voice` Start/Stop, `zt::engine` Plan/Auswahl/Ladezeit/Fallback-Gründe, `zt::audio` Gerät+Format, Frames/s, RMS, `zt::vad` Segmentgrenzen, `zt::stt` Partial (debug, mit Zeitstempel und Latenz) / Final (info), `zt::tracker` Zustandswechsel, Konfidenz, Cursor, `zt::resync`, `zt::models` Download/Hash, `zt::frontend` unbehandelte JS-Fehler.
- Mikrofon-Watchdog: 3 s ohne Callback oder nur exakte Nullen → Fehler mit Hinweis auf die Datenschutzeinstellungen.
- Diagnose-Tab (`DiagnosticsPanel.svelte`): Pegel, Engine/Modell/Kette, Audioformat, Frames/s, Tick-Rate, Tracker-Zustand, Latenzen p50/p95, letzte Fehler, „Log exportieren“ (Speichern-Dialog, Logs + JSON-Snapshot), Mikrofontest.

### Latenz

Gemessen wird im Voice-Thread (rollierend, letzte 200 Werte, `zt_core::metrics::LatencyWindow`, Nearest-Rank):

| Metrik | Start | Ende |
|---|---|---|
| `onsetToPartial` | Sprechbeginn relativ zum Rauschboden (`OnsetDetector`, siehe Nachtrag 2026-10-11; vorher fester Wert −45 dBFS) | erste Partial-Hypothese |
| `audioToPartial` | Aufnahmezeit des Chunks, der eine neue Partial auslöste | Partial emittiert |
| `hypToCursor` | Hypothese an Tracker | Cursor-Wechsel |
| `endToEnd` | Aufnahmezeit des Chunks | `voice://word` emittiert |

Latenzkette (Ziel: Wort erkannt → Scroll < 300 ms p50, hart < 1 s). **Geschätzt** = aus Konfiguration/Literatur abgeleitet, noch nicht auf Zielhardware gemessen; die gemessenen Werte liefert ab dieser Version der Diagnose-Tab.

| Stufe | Wert | Art |
|---|---|---|
| Audio-Callback (WASAPI shared, cpal-Standardpuffer) | ~10 ms Periode | geschätzt (Windows-Standard) |
| Kanal → Voice-Thread | blockierendes `recv`, kein Polling-Takt (`recv_timeout` 50 ms nur als Abbruch-Check) | Code |
| Resampling auf 16 kHz | < 1 ms | geschätzt |
| `accept_waveform` + Decode | pro Callback-Chunk, `decode` solange `is_ready` | Code |
| Zipformer-Chunk (Modell-Lookahead) | Partials kommen pro Encoder-Chunk (typisch 320 ms bei Streaming-Zipformer) → Ø ~160 ms Warten + Rechenzeit | geschätzt |
| Rechenzeit pro Chunk (ARM64, 2 Threads, int8/fp32) | 20–60 ms | geschätzt |
| Tracker (Smith-Waterman k=6 × w=70) | < 1 ms | geschätzt, misst `hypToCursor` |
| Tick 60 Hz (`sleep(16 ms)`, Rust std auf Windows mit High-Res-Timer) | ≤ 16 ms | Code, misst „Tick Hz“ |
| Scroll-Ease (vorher τ = 300 ms) | **τ = 150 ms**: 63 % eines Wortschritts nach 150 ms sichtbar, Bewegung beginnt im nächsten Tick | Code + Test |
| Endpoint-Regeln (2,4 s / 0,8 s / 20 s) | betreffen nur `Final`, der Tracker folgt den Partials | Code |

Erwartung Wort → sichtbarer Scrollbeginn ≈ 10 + ~160 + ~40 + 1 + ≤16 ms ≈ **230 ms p50** (geschätzt), p95 durch Chunk-Grenze und CPU ≈ 450 ms – innerhalb der harten Grenze. Bestätigung über `endToEnd` im Diagnose-Tab (Scroll-Animation kommt ab Tick hinzu).

Weitere getunte Defaults:
- Tracker-Nahpfad (`TrackerConfig::near_words = 2`, siehe unten) für das erste Wort nach Seek/Re-Anchor.
- Hotwords bleiben für Modelle ohne `bpe.vocab` aus; der Recognizer wird dann nicht neu erzeugt (keine doppelte Ladezeit).

## Vergleich mit ShayneP/local-teleprompter

Quelle: https://github.com/ShayneP/local-teleprompter (Lizenz **MIT**, geprüft im README/Repository). Gelesen: `frontend/lib/teleprompter/position-tracker.ts`, README.

| Aspekt | local-teleprompter | zero-teleprompter | Übernahme |
|---|---|---|---|
| STT | NVIDIA Nemotron Streaming 0.6B (FastConformer-RNNT, cache-aware) als Python/FastAPI-Server, LiveKit, GPU/MPS bevorzugt; README nennt ~80 ms bis erste Wörter auf Apple Silicon/GPU | sherpa-onnx in-process, Zipformer (Kroko) auf CPU, keine Python-/Server-Abhängigkeit | nein: Server + GPU widerspricht Offline-Einzelbinary (ADR-002/006); Modell als Kandidat in MODELS.md beobachten |
| Matching | vorwärts, Nahfenster Cursor..+2 sofort; Fernfenster bis +17 nur mit Bigramm-Bestätigung; Fuzzy nur ≥ 5 Zeichen, Längendifferenz ≤ 2, Levenshtein ≤ 1; Re-Anchor nach 4 Fehlschlägen über 6 Wörter, ≥ 3 Treffer | gefenstertes Smith-Waterman (k=6, −10/+60), Hysterese (2× Bestätigung ab 18 Wörtern), Rücksprung bis 10 Wörter, globale Suche bei `lost`, Offline-Resync | **teilweise**: Nahpfad + strikte Fuzzy-Regel |
| Partials | verarbeitet nur neue Wörter (`consumedSpokenCount`), keine Gewichtung | ersetzt Partial, gewichtet Finals höher | nein: SW-Alignment robuster gegen Revisionen der Hypothese |
| Rückwärts | nur vorwärts (manuell per Doppelklick) | Rücksprünge erlaubt | nein |

Übernommen (eigene Implementierung, kein Code kopiert, Attribution in THIRD_PARTY_NOTICES.md): **Nahpfad** – wenn das SW-Alignment zu wenig Evidenz hat (`min_matches = 2`), darf das *neueste* Wort den Cursor trotzdem bewegen, sofern es eines der nächsten drei Skriptwörter trifft (exakt oder nach der strikten Fuzzy-Regel). Nur mit vorhandenem Anker (`tracking`/`hold`/`resync`), nie in `listening`/`lost`. Begründung „nachweislich besser“: Nach manuellem Seek, Re-Anchor oder Sprung ist der Hypothesenpuffer leer; bisher brauchte die erste Bewegung zwei passende Wörter (≈ 1 Wort ≈ 250–400 ms zusätzliche Latenz bei 2,5–4 Wörtern/s). Mit Nahpfad bewegt sich der Cursor beim ersten Wort (Test `near_fast_path_moves_on_first_word_after_seek`). Die strikte Fuzzy-Regel verhindert, dass kurze Wörter („und“/„uns“) den Cursor ziehen (Test `near_word_match_rules`); die bestehende Szenario-Tabelle (u. a. „ein Wort reicht in `listening` nicht“) bleibt unverändert grün.

Nicht übernommen, weil nicht besser belegt: das Bigramm-Gate fürs Fernfenster (unsere 2×-Sprungbestätigung erfüllt denselben Zweck mit Zeit-Hysterese) und der Re-Anchor über 6 Wörter (bei uns globale SW-Suche + Offline-Resync).

## Nachtrag 2026-10-11: Partial-Kadenz, Onset-Metrik, Nemotron

**Befund (Nutzer-Log, Win11 ARM64, Kroko DE):** Partials nur alle ~1,27 s
(t_ms 19180 → 20430 → 21747 → 23020), je 1–3 Wörter; `audioToPartial` 80 ms,
`hypToCursor` 0,3 ms, der Scroll ruckelt im 1,3-s-Takt. `onsetToPartial`
p50 2649 ms / p95 46154 ms.

**Ursache Kadenz: Modell, nicht Code.** Encoder-Metadaten der Kroko-Modelle:
`decode_chunk_len = 128`, `T = 141` (10-ms-Frames) → ein Decoder-Schritt pro
1,28 s Audio. Unsere Schleife dekodiert nach jedem Audio-Block
(`while is_ready { decode }`) und meldet jede Textänderung; Endpoint-Regeln
betreffen nur Finals. Gegenprobe im CI mit identischem Code: Zipformer EN 20M
0,3 s, Nemotron 160 ms 0,2 s, Nemotron 320 ms 0,3 s, Kroko 1,30 s (p50 = p95).
Die Annahme oben („typisch 320 ms bei Streaming-Zipformer“) gilt für Kroko
nicht. Kein Code-Fix möglich; Abhilfe ist ein Modell mit kleinerem Chunk.

**Ursache Onset-Metrik:** fester Schwellwert −45 dBFS *nach* AGC. Bei leisem
Mikrofon hebt die AGC das Rauschen über −45 dBFS → „Onset“ auf Rauschen,
zurückgesetzt nur bei Endpoint/Final → Sekunden bis Minuten. Neu:
`zt_audio::OnsetDetector` – Schwelle = Rauschboden + 12 dB (Boden folgt
schnell nach unten, langsam nach oben mit 3 dB/s, eingefroren während
Sprache), mindestens 60 ms laut, Neu-Scharfschaltung nach 0,5 s Stille,
unbeanspruchter Onset bleibt 2,5 s für eine späte erste Hypothese gültig.
Derselbe Detektor misst im CI „Zeit bis erstes Wort“.

**Entscheidung Modell:** NVIDIA Nemotron 3.5 ASR Streaming 0.6B int8
(OpenMDW-1.1, sherpa-onnx-Export `csukuangfj2/…-2026-06-11`) wird Standard für
DE und EN, als **320-ms-Variante**; 80/160 ms wählbar, Kroko und 20M bleiben
wählbar und sind Fallback, solange Nemotron nicht installiert ist. Umsetzung:
`ModelType::OnlineNemoTransducer`, `SherpaConfig::nemotron` (Transducer-Felder,
`greedy_search`, 4 Threads, `stream.set_option("language", "de"|"en")` nach
`create_stream` und jedem `reset`, sonst `"auto"`). sherpa-onnx 1.13.8 kennt
für Streaming-NeMo nur `greedy_search`; `modified_beam_search` beendet den
Prozess (`exit(-1)`), daher keine Beam-Stufe und keine Hotwords.

Gemessen (CI-Lauf 38132699828, 4-vCPU-Runner, 100-ms-Blöcke, 500 ms
Vorlauf-Stille; vollständige Tabelle in `docs/research/MODELS.md`):

| Modell | RTF Arm64 / x86 | Kadenz p50 / p95 | erstes Wort DE / EN | Laden | RSS |
|---|---|---|---|---|---|
| Nemotron 320 ms | 0,31 / 0,59 | 0,3 / 0,4 s | 0,59 / 0,90 s | ~1,5–1,8 s | 0,76–0,97 GB |
| Nemotron 160 ms | 0,53–0,60 / **1,05–1,23** | 0,2 / 0,3–0,5 s | 0,59 / 0,90 s | ~1,5–1,8 s | 0,76–0,97 GB |
| Kroko DE / EN | 0,03–0,04 | **1,3 / 1,3 s** | 0,99 / 1,0–2,3 s | ~2,4–2,8 s | 0,25 GB |
| Zipformer EN 20M | 0,03–0,04 | 0,3 / 0,4 s | – / 1,2 s | ~0,6 s | 0,15 GB |

Warum 320 statt 160 ms: 160 ms ist auf den x86-Runnern nicht echtzeitfähig
(in einem zweiten Lauf sogar RTF 2,4 auf windows-latest), 320 ms braucht überall
die Hälfte der Rechenzeit bei 0,3 s Kadenz (Kroko: 1,3 s). Damit ein zu
langsames Modell nicht unbemerkt hinterherläuft, misst `RealtimeWatch`
(`voice.rs`) die Dekodierzeit je 5 s Audio und warnt einmal, wenn RTF > 0,9 in
zwei Fenstern hintereinander.

## Konsequenzen

- „Demo“ erscheint nur noch, wenn wirklich kein Streaming-Modell installiert ist oder der Build kein STT enthält, und dann immer mit „Modell laden“.
- Ein kaputtes Modell führt nicht mehr zu scheinbarem Tracking, sondern zu einer sichtbaren Fehlermeldung mit Log.
- Neue Abhängigkeiten: `tracing`, `tracing-subscriber`, `tracing-appender` (alle MIT). `Cargo.lock` per `lockfiles.yml` regeneriert.
- Offene Punkte: Latenzwerte auf Windows-ARM64-Zielhardware messen und hier nachtragen; Encoder-Chunkgröße der Kroko-Modelle aus den Modell-Metadaten verifizieren; `num_threads` (2) für ARM64 evaluieren; cpal-Stream-Fehler (`err_fn`) laufen noch über `eprintln!` in `zt-audio` statt über `tracing`.
