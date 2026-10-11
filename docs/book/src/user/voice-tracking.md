# Voice-Tracking

Beim Voice-Tracking hört die App über das Mikrofon zu, erkennt die gesprochenen Wörter **lokal** und scrollt den Text so, dass das zuletzt gesprochene Wort an der Leselinie steht. Audio verlässt das Gerät nicht.

## Voraussetzungen

| Baustein | Woher |
|---|---|
| Mikrofon | Build-Feature `cpal` (Standard) |
| Spracherkennung | Build-Feature `sherpa` (in allen Release-Builds enthalten) und ein installiertes Streaming-Modell (Zipformer-Transducer) |

Welche Engine läuft, entscheidet die App beim Start von Voice oder Mikrofontest ([ADR-007](../../../adr/ADR-007-voice-pipeline-tuning.md)):

| Situation | Was passiert |
|---|---|
| Streaming-Modell für die eingestellte Sprache installiert | echte Erkennung mit diesem Modell |
| nur ein Streaming-Modell einer **anderen** Sprache installiert | dieses Modell wird genutzt, ein Hinweis erscheint |
| **kein** Streaming-Modell installiert (Parakeet und Silero VAD zählen nicht, sie sind Hilfsmodelle) | **Demo-Modus**: eine Mock-Engine „liest“ das Skript mit ca. 2,3 Wörtern/s. Neben dem Status erscheint „Demo · Modell laden“ und führt direkt zu Voice & Modelle |
| Build ohne Spracherkennung (Feature `sherpa`) | Demo-Modus mit Hinweis |
| Modell installiert, lässt sich aber nicht laden | **kein** Demo: Fehlermeldung, Badge „Modellfehler · Details“ (öffnet den Diagnose-Tab), der Text bewegt sich nicht von selbst |

Nach einem Modell-Download startet eine laufende Voice-Sitzung automatisch neu und nutzt das neue Modell.
> `resources/models.json` enthält vier Modelle (Kroko DE/EN, Zipformer EN 20M als Fallback, Parakeet TDT 0.6B v3 für Resync), Details und Quellen in [`docs/research/MODELS.md`](../../../research/MODELS.md). Für `tokens.txt` steht kein SHA-256 im Manifest: Diese Dateien werden nur auf Größe geprüft, ihr Hash wird nach dem Download berechnet und in den Einstellungen als „unverifiziert“ angezeigt. Die URLs zeigen auf `main` und sind noch nicht auf Commits gepinnt. Das Kroko-DE-Modell hat eine ungeklärte Lizenz (siehe [LEGAL.md](../../../legal/LEGAL.md)) und darf bis zur Klärung nicht ausgeliefert werden. Alternativ lässt sich ein eigenes Manifest als `<Datenordner>/models/manifest.json` ablegen. Erlaubt sind nur HTTPS-URLs mit SHA-256-Prüfung.
>
> Hotwords (Skriptbegriffe) wirken nur mit `modified_beam_search` und Transducer-Modellen. Bei BPE-Modellen braucht es zusätzlich `bpe.vocab` (nicht `bpe.model`), sonst meldet `set_hotwords` `Unsupported`.

## Modellwahl und Resync

In den Einstellungen (`Ctrl+,`) unter Voice & Modelle → Spracherkennung wählst du pro Sprache das Streaming-Modell oder „Automatisch“ (Standard: Kroko DE bzw. Kroko EN; lässt sich ein Modell nicht laden, springt die App auf das nächste installierte, z. B. Zipformer EN 20M). Hinweise dazu erscheinen unter der Auswahl.

Der Schalter **Resync im Hintergrund** (Stufe L, standardmäßig aus) lädt zusätzlich Parakeet TDT 0.6B v3. Es dekodiert abgeschlossene Sprachsegmente im Hintergrund und verankert den Cursor neu, wenn das Streaming-Modell danebenlag. Das Scrollen wird dadurch nie verzögert. Segmentiert wird mit Silero VAD, wenn `silero_vad.onnx` im Modellordner liegt, sonst über den Pegel. Änderungen gelten beim nächsten Start von Voice. Hintergrund: [ADR-006](../../../adr/ADR-006-hybrid-stt.md).

## Windows-Spracherkennung (System-Fallback)

Windows-Builds können die Spracherkennung des Betriebssystems nutzen ([ADR-008](../../../adr/ADR-008-windows-speech-fallback.md)). Dafür muss nichts heruntergeladen werden.

- **Automatisch als Fallback:** Ist kein Streaming-Modell installiert oder lässt sich keins laden, versucht die App die Windows-Spracherkennung, bevor sie in den Demo-Modus bzw. auf „Modellfehler“ geht. Unter der Modellauswahl steht dann z. B. „Nemotron → Kroko → Windows Spracherkennung (System)“.
- **Manuell:** In Voice & Modelle → Spracherkennung „Windows Spracherkennung (System)“ wählen.
- **Voraussetzungen:**
  - Das Sprachpaket mit Spracherkennung ist installiert: Einstellungen → Zeit und Sprache → Sprache und Region → Deutsch → Sprachoptionen → „Spracherkennung“ (`ms-settings:regionlanguage`). Die Sprache wählst du unter Zeit und Sprache → Spracherkennung (`ms-settings:speech`).
  - **Online-Spracherkennung** ist eingeschaltet: Datenschutz und Sicherheit → Spracherkennung (`ms-settings:privacy-speech`). Das freie Diktat ist ein Microsoft-Clouddienst, das Audio verlässt dann das Gerät.
  - Mikrofonzugriff für Desktop-Apps ist erlaubt (`ms-settings:privacy-microphone`).
- **Ohne Online-Spracherkennung** wechselt die Engine in den *Skriptwort-Modus* (offline, sofern die Grammatik-Erkennung für die Sprache installiert ist). Erkannt werden dann nur Wörter aus dem Skript. Das reicht als Notbetrieb, ist aber deutlich ungenauer.
- Windows nimmt das **Standard-Mikrofon** selbst auf. Die Mikrofonwahl, der Gain und die Kanalwahl der App wirken nicht auf die Erkennung. Pegelanzeige und Watchdog laufen weiter über die App-Aufnahme.
- Die Latenz misst die Diagnose als *Sprechbeginn → erster Partial*. „Audio → Partial“ und „Ende-zu-Ende“ bleiben für diese Engine leer.

## Einschalten
- `V` (in der App) oder `Ctrl+Shift+F6` (global) schaltet Voice an und aus.
- Beim Einschalten springt der Scroll-Modus auf **Voice-Tracking**, beim Ausschalten auf **Konstant**.
- Alternativ wählst du den Modus in der Befehlspalette („Scroll-Modus: …“). „Sprachaktiviert“ nutzt dieselbe Pipeline, scrollt aber mit konstantem Tempo, solange Sprache erkannt wird.
- Sprache (Deutsch oder Englisch) und Mikrofon stellst du in den Einstellungen unter Voice & Modelle ein. Dort gibt es auch einen **Mikrofontest** mit Pegelanzeige. Während Voice-Tracking aktiv ist, lässt er sich nicht starten.

## Statusanzeige

| Zustand | Farbe / Form | Bedeutung |
|---|---|---|
| `idle` „Voice aus“ | grau, Kreis-Umriss | Voice ist aus |
| `listening` „Hört zu“ | blau, pulsierend | wartet auf den ersten sicheren Treffer |
| `tracking` „Folgt“ | grün, gefüllt | der Text folgt der Stimme |
| `hold` „Wartet“ | orange, Pausenbalken | Stille, unsicherer Treffer oder ein möglicher Sprung wird noch bestätigt. Der Text bleibt stehen |
| `lost` „Verloren“ | rot, Dreieck | länger als `lostTimeoutMs` (Standard 4 s) gesprochen, ohne dass etwas zum Skript passte |
| `resync` „Springt“ | violett | ein bestätigter Sprung wird sanft ausgeführt (300 ms) |

## Verhalten in Grenzfällen

- **Versprecher, Füllwörter, Ad-libs:** Unschärfe beim Wortvergleich und Lückenkosten fangen sie ab. Füllwörter („äh“, „ähm“, „uh“, „um“) werden verworfen.
- **Wiederholen (kurzer Rücksprung, bis 10 Wörter):** wird sofort akzeptiert.
- **Weiter vorne einsetzen (über 18 Wörter):** Der Sprung wird erst nach **zwei** übereinstimmenden Erkennungen ausgeführt. Bis dahin bleibt der Text in `hold`.
- **Verloren:** Was dann passiert, legt die Einstellung „Bei Verlust“ fest (`lostPolicy`): **Stoppen** (Standard) oder **mit konstantem Tempo weiter**. Die App sucht dann im ganzen Skript, bis sie dich wiederfindet.

## Manuell eingreifen

- **Klick auf ein Wort** in der Vorschau löst „Resync hier“ aus: Der Cursor springt dorthin.
- `R` setzt den Resync an die aktuelle Markerposition.
- Nach Klick, Absatzsprung (`PgUp`/`PgDn`), Positionsleiste oder Stop ignoriert die App die Stimme für `manualOverrideMs` (Standard 2 s). Danach übernimmt die Stimme wieder.

## Tipps

- Ein Headset oder Richtmikrofon nah am Mund verbessert die Erkennung deutlich.
- Seltene Wörter aus dem Skript (Namen, Fachbegriffe) gibt die App der Spracherkennung als Hotwords mit, sofern die Engine das unterstützt.
- Zahlen kannst du als Ziffern schreiben („2026“). Die App vergleicht sie mit der gesprochenen Form.

## Diagnose und Logs

Einstellungen (`Ctrl+,`) → **Diagnose** zeigt live:

- Pegel und **Mikrofontest**, Audiogerät mit nativem Format (z. B. `48000 Hz · 1 ch · F32 -> 16 kHz mono`), Frames/s und die Tick-Rate (Soll ~60 Hz);
- aktive Engine, installierte Streaming-Modelle in Fallback-Reihenfolge, Tracker-Zustand und Konfidenz, letzte Hypothese;
- **Latenzen p50/p95** über die letzten 200 Messungen;
- die letzten Fehler und Hinweise;
- **Log exportieren**: speichert alle Logdateien plus einen JSON-Schnappschuss in eine Datei, die du einem Fehlerbericht anhängen kannst.

Logdateien liegen im App-Log-Ordner (Windows `%LOCALAPPDATA%\<App-ID>\logs`, macOS `~/Library/Logs/<App-ID>`, Linux `~/.local/share/<App-ID>/logs`), Name `zero-teleprompter.JJJJ-MM-TT.log`, täglich rotiert, 7 Tage. Mehr Details: Umgebungsvariable `ZT_LOG=debug` (oder z. B. `ZT_LOG=info,zt::stt=debug` für jede Partial-Hypothese).

### Latenzkette

Ziel: gesprochenes Wort → Scrollbeginn unter 300 ms (p50), höchstens 1 s.

| Messpunkt im Diagnose-Tab | misst |
|---|---|
| Sprechbeginn → erste Hypothese | erster lauter Audio-Chunk bis zur ersten Partial (enthält den Modell-Lookahead) |
| Audio-Chunk → Hypothese | Aufnahmezeit des Chunks bis Partial (Warteschlange + Dekodieren) |
| Hypothese → Cursor | Abgleich im Tracker |
| Audio → Cursor (gesamt) | Aufnahme bis Cursor-Event; danach startet die Scroll-Bewegung im nächsten Tick (≤ 16 ms) mit einer Zeitkonstante von 150 ms |

Geschätzte Kette (noch nicht auf Zielhardware gemessen): Audio-Callback ~10 ms → Modell-Chunk Ø ~160 ms + Rechnen 20–60 ms → Tracker < 1 ms → Tick ≤ 16 ms ≈ 230 ms p50. Die gemessenen Werte zeigt der Diagnose-Tab.