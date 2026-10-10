# Voice-Tracking

Beim Voice-Tracking hört die App über das Mikrofon zu, erkennt die gesprochenen Wörter **lokal** und scrollt den Text so, dass das zuletzt gesprochene Wort an der Leselinie steht. Audio verlässt das Gerät nicht.

## Voraussetzungen

| Baustein | Woher |
|---|---|
| Mikrofon | Build-Feature `cpal` (Standard) |
| Spracherkennung | Build-Feature `sherpa` (Standard **aus**) und ein installiertes Streaming-Modell (Zipformer-Transducer) für die eingestellte Sprache |

Fehlt eines davon, läuft Voice im **Demo-Modus**. Eine Mock-Engine „liest“ dann das Skript mit etwa 2,3 Wörtern pro Sekunde. So lässt sich die Oberfläche ohne Modell ausprobieren. Der Inspector zeigt per Event `voice://engine` an, welche Engine läuft und ob ein Mikrofon aktiv ist.

> `resources/models.json` enthält vier Modelle (Kroko DE/EN, Zipformer EN 20M als Fallback, Parakeet TDT 0.6B v3 für Resync), Details und Quellen in [`docs/research/MODELS.md`](../../../research/MODELS.md). Für `tokens.txt` steht kein SHA-256 im Manifest: Diese Dateien werden nur auf Größe geprüft, ihr Hash wird nach dem Download berechnet und im Inspector als „unverifiziert“ angezeigt. Die URLs zeigen auf `main` und sind noch nicht auf Commits gepinnt. Das Kroko-DE-Modell hat eine ungeklärte Lizenz (siehe [LEGAL.md](../../../legal/LEGAL.md)) und darf bis zur Klärung nicht ausgeliefert werden. Alternativ lässt sich ein eigenes Manifest als `<Datenordner>/models/manifest.json` ablegen. Erlaubt sind nur HTTPS-URLs mit SHA-256-Prüfung.
>
> Hotwords (Skriptbegriffe) wirken nur mit `modified_beam_search` und Transducer-Modellen. Bei BPE-Modellen braucht es zusätzlich `bpe.vocab` (nicht `bpe.model`), sonst meldet `set_hotwords` `Unsupported`.

## Modellwahl und Resync

Im Inspector unter Voice → Spracherkennung wählst du pro Sprache das Streaming-Modell oder „Automatisch“ (Standard: Kroko DE bzw. Kroko EN; lässt sich ein Modell nicht laden, springt die App auf das nächste installierte, z. B. Zipformer EN 20M). Hinweise dazu erscheinen unter der Auswahl.

Der Schalter **Resync im Hintergrund** (Stufe L, standardmäßig aus) lädt zusätzlich Parakeet TDT 0.6B v3. Es dekodiert abgeschlossene Sprachsegmente im Hintergrund und verankert den Cursor neu, wenn das Streaming-Modell danebenlag. Das Scrollen wird dadurch nie verzögert. Segmentiert wird mit Silero VAD, wenn `silero_vad.onnx` im Modellordner liegt, sonst über den Pegel. Änderungen gelten beim nächsten Start von Voice. Hintergrund: [ADR-006](../../../adr/ADR-006-hybrid-stt.md).

## Einschalten

- `V` (in der App) oder `Ctrl+Shift+F6` (global) schaltet Voice an und aus.
- Beim Einschalten springt der Scroll-Modus auf **Voice-Tracking**, beim Ausschalten auf **Konstant**.
- Alternativ wählst du den Modus in der Befehlspalette („Scroll-Modus: …“). „Sprachaktiviert“ nutzt dieselbe Pipeline, scrollt aber mit konstantem Tempo, solange Sprache erkannt wird.
- Sprache (Deutsch oder Englisch) und Mikrofon stellst du im Inspector unter Voice ein. Dort gibt es auch einen **Mikrofontest** mit Pegelanzeige. Während Voice-Tracking aktiv ist, lässt er sich nicht starten.

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
