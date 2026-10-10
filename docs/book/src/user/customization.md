# Anpassung

Alle Einstellungen findest du im **Inspector** auf der rechten Seite. Es gibt zwei Speicherorte:

- **Kern-Einstellungen** (`settings.json`): Theme, Schriftgröße, Tempo, Countdown, Voice-Verhalten und Tastenbelegung. Gilt für beide Fenster.
- **UI-Präferenzen** (`ui-prefs.json`): Schriftart, Zeilenbreite, Marker-Stil, Farben, Dimmen und Erscheinungsbild der Oberfläche. Die App überträgt sie per `prefs://changed` an beide Fenster.

## Themes

Es gibt sechs Prompter-Themes aus `docs/design/tokens.json`. Alle erreichen WCAG AAA. Umschalten mit `Ctrl+1` bis `Ctrl+6`:

| Taste | Theme | Kontrast | Besonderheit |
|---|---|---|---|
| `Ctrl+1` | Studio Dark (Standard) | 16,99:1 | |
| `Ctrl+2` | Broadcast Classic | 21,0:1 | Schwarz/Weiß |
| `Ctrl+3` | Daylight | 16,40:1 | heller Hintergrund |
| `Ctrl+4` | High Contrast | 16,13:1 | Gelb auf Schwarz |
| `Ctrl+5` | Low Light | 9,87:1 | warm und dunkel für dunkle Räume |
| `Ctrl+6` | Dyslexia | 13,22:1 | Buchstabenabstand 0,04 em, Wortabstand 0,16 em, Zeilenhöhe 1,6 |

## Schrift und Layout

| Einstellung | Bereich / Werte | Standard |
|---|---|---|
| Schriftgröße | 24–300 px (`Ctrl+=` / `Ctrl+-` je ±4) | 72 px |
| Schriftart | Atkinson Hyperlegible, Inter, System, Serif, Mono | Atkinson |
| Zeilenbreite | 16–60 `ch` (Schieberegler) | 32 ch |
| Zeilenhöhe | 1,0–2,4 im Inspector (der Kern akzeptiert 1,0–3,0) | 1,4 |
| Fokus-/Leselinie | Inspector und Ziehen in der Vorschau: 5–70 % der Fensterhöhe. Der Kern begrenzt auf 10–90 %, Werte unter 10 % werden also auf 10 % angehoben (bekannte Inkonsistenz) | 35 % |

## Lesemarker und Farben

- **Marker-Stil:** Linie oder Band.
- **Gelesenen Text dimmen**, mit einstellbarer Opazität (0,3–1). Ohne eigenen Wert gilt der Standard des Themes.
- **Eigene Farben** für Text, Hintergrund und Marker überschreiben das Theme. Achte dabei selbst auf ausreichenden Kontrast.
- **Erscheinungsbild der Oberfläche:** System, Hell oder Dunkel. Das betrifft nur das Control-Fenster, nicht den Prompter-Text.

## Tempo und Countdown

- WPM 40–400, Standard 140.
- Countdown 0 (sofort starten) oder mehr Sekunden, Standard 3. Umschalten im Transport.

## Tastenbelegung

Im Inspector unter Tasten klickst du auf eine Belegung und drückst die neue Kombination. Umbelegen lassen sich nur die Aktionen in `Settings.hotkeys`, die Liste steht unter [Tastenkürzel](shortcuts.md). Die übrigen App-Tasten und die globalen Hotkeys sind in v0.1 fest.

## Bewegung reduzieren

Mit „Bewegung reduzieren“ im Inspector (`reducedMotion`) oder der Systemeinstellung `prefers-reduced-motion` schaltet der Prompter weiche Übergänge wie Resync-Ease, Puls und Countdown-Animation ab oder verkürzt sie.
