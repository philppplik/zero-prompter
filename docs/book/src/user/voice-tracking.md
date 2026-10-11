# Voice-Tracking

Beim Voice-Tracking hört die App über das Mikrofon zu, erkennt die gesprochenen Wörter **lokal** und scrollt den Text so, dass das zuletzt gesprochene Wort an der Leselinie steht. Audio verlässt das Gerät nicht.

## Voraussetzungen

| Baustein | Woher |
|---|---|
| Mikrofon | Build-Feature `cpal` (Standard) |
| Spracherkennung | Build-Feature `sherpa` (in allen Release-Builds enthalten) und ein installiertes Streaming-Modell (NVIDIA Nemotron 3.5 Streaming, Kroko- oder Zipformer-Transducer) |

Welche Engine läuft, entscheidet die App beim Start von Voice oder Mikrofontest ([ADR-007](../../../adr/ADR-007-voice-pipeline-tuning.md)):

| Situation | Was passiert |
|---|---|
| Streaming-Modell für die eingestellte Sprache installiert | echte Erkennung mit diesem Modell |
| nur ein Streaming-Modell einer **anderen** Sprache installiert | dieses Modell wird genutzt, ein Hinweis erscheint |
| **kein** Streaming-Modell installiert (Parakeet und Silero VAD zählen nicht, sie sind Hilfsmodelle) | **Demo-Modus**: eine Mock-Engine „liest“ das Skript mit ca. 2,3 Wörtern/s. Neben dem Status erscheint „Demo · Modell laden“ und führt direkt zu Voice & Modelle |
| Build ohne Spracherkennung (Feature `sherpa`) | Demo-Modus mit Hinweis |
| Modell installiert, lässt sich aber nicht laden | **kein** Demo: Fehlermeldung, Badge „Modellfehler · Details“ (öffnet den Diagnose-Tab), der Text bewegt sich nicht von selbst |

Nach einem Modell-Download startet eine laufende Voice-Sitzung automatisch neu und nutzt das neue Modell.
> `resources/models.json` enthält Nemotron 3.5 Streaming (80/160/320 ms), Kroko DE/EN, Zipformer EN 20M als Fallback, Parakeet TDT 0.6B v3 für Resync und Silero VAD, alle URLs auf Commits gepinnt und mit SHA-256; Details und Quellen in [`docs/research/MODELS.md`](../../../research/MODELS.md). Das Kroko-DE-Modell hat eine ungeklärte Lizenz (siehe [LEGAL.md](../../../legal/LEGAL.md)) und darf bis zur Klärung nicht ausgeliefert werden. Alternativ lässt sich ein eigenes Manifest als `<Datenordner>/models/manifest.json` ablegen. Erlaubt sind nur HTTPS-URLs mit SHA-256-Prüfung.
>
> Hotwords (Skriptbegriffe) wirken nur mit `modified_beam_search` (Kroko, Zipformer). Nemotron läuft in sherpa-onnx 1.13.8 nur mit `greedy_search` und damit ohne Hotwords; `set_hotwords` meldet dann `Unsupported`.

## Modellwahl und Resync

In den Einstellungen (`Ctrl+,`) unter Voice & Modelle → Spracherkennung wählst du pro Sprache das Streaming-Modell oder „Automatisch“. Hinweise dazu erscheinen unter der Auswahl.

| Modell | Download | Cursor bewegt sich | CPU | Hinweis |
|---|---|---|---|---|
| **Nemotron 3.5 Streaming, 320 ms** (Standard DE + EN) | 682 MB, ca. 0,8–1 GB RAM | etwa alle 0,3 s | mittel (CI: RTF 0,3 Arm64, 0,6 x86) | Groß-/Kleinschreibung und Satzzeichen, keine Hotwords |
| Nemotron 3.5 Streaming, 160 ms / 80 ms | je 682 MB | etwa alle 0,2 s | hoch (CI x86: RTF über 1 bei 160 ms) | nur für starke CPUs |
| Kroko DE / EN | 71 MB | etwa alle 1,3 s (Modell-Chunk 1,28 s) | sehr gering | Hotwords |
| Zipformer EN 20M | 44 MB | etwa alle 0,3 s | sehr gering | nur Großbuchstaben, schwächer bei freier Rede |

„Automatisch“ nimmt das erste **installierte** Modell der Reihenfolge Nemotron 320 ms → Kroko → (EN) Zipformer 20M; wer Nemotron nicht lädt, behält Kroko. Lässt sich ein Modell nicht laden, springt die App auf das nächste installierte. Rechnet ein Modell auf dem Rechner langsamer als Echtzeit (der Text würde immer weiter hinterherhinken), erscheint nach etwa 10 s eine Warnung mit der Empfehlung, ein kleineres Modell zu wählen. Messwerte: [`docs/research/MODELS.md`](../../../research/MODELS.md).

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
| Sprechbeginn → erste Hypothese | Sprechbeginn bis zur ersten Partial der Äußerung (enthält Modell-Chunk und Lookahead). Sprechbeginn = Pegel mindestens 12 dB über dem laufend geschätzten Rauschboden für 60 ms; nach 0,5 s Stille wird neu gemessen. Von der AGC angehobenes Rauschen zählt so nicht mehr als Sprechbeginn |
| Audio-Chunk → Hypothese | Aufnahmezeit des Chunks bis Partial (Warteschlange + Dekodieren) |
| Hypothese → Cursor | Abgleich im Tracker |
| Audio → Cursor (gesamt) | Aufnahme bis Cursor-Event; danach startet die Scroll-Bewegung im nächsten Tick (≤ 16 ms) mit einer Zeitkonstante von 150 ms |

Wie oft sich der Cursor bewegen kann, bestimmt vor allem der Decoder-Takt des Modells: Kroko rechnet in Blöcken von 1,28 s Audio und liefert deshalb nur etwa alle 1,3 s neue Wörter (meist 1–3 auf einmal), auch wenn „Audio-Chunk → Hypothese“ und „Hypothese → Cursor“ klein sind. Nemotron 320 ms liefert etwa alle 0,3 s (CI-Messung, [MODELS.md](../../../research/MODELS.md)). Die Werte auf deinem Rechner zeigt der Diagnose-Tab.