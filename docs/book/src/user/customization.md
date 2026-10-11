# Anpassung

Alle Einstellungen findest du im Dialog **Einstellungen**: `Ctrl+,` (macOS `⌘,`) oder das Regler-Symbol in der Aktions-Pille oben rechts. Bereiche: Darstellung, Scroll, Voice & Modelle, Tastenkürzel, Ausgabe; `Esc` oder ein Klick daneben schließt ihn. Die Skript-Bibliothek mit Gliederung öffnest du mit `Ctrl+B` (Seitenleisten-Symbol in der Pille), alle Aktionen mit `Ctrl+K`. Es gibt zwei Speicherorte:

- **Kern-Einstellungen** (`settings.json`): Theme, Schriftgröße, Typografie (Stärke, Abstände, Ausrichtung, Großschreibung), Tempo, Countdown, Voice-Verhalten und Tastenbelegung. Gilt für beide Fenster.
- **UI-Präferenzen** (`ui-prefs.json`): Schriftart (inkl. freiem Systemschrift-Namen), Zeilenbreite, Marker-Stil, Farben, Dimmen und Erscheinungsbild der Oberfläche. Die App überträgt sie per `prefs://changed` an beide Fenster.

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
| Schriftart | siehe unten (12 gebündelte Schriften, Systemschrift, freier Name) | Atkinson Hyperlegible Next |
| Zeilenbreite | 16–60 `ch` (Schieberegler) | 32 ch |
| Zeilenhöhe | 1,0–2,4 in den Einstellungen (der Kern akzeptiert 1,0–3,0). Ein Theme-Wert (Dyslexia 1,6) gilt als Minimum | 1,4 |
| Schriftstärke | 100–900. Variable Schriften nutzen ihre `wght`-Achse stufenlos und werden auf ihren Bereich begrenzt (z. B. Open Sans 300–800, IBM Plex Sans bis 700); OpenDyslexic hat nur 400/700 und springt auf das nächste Gewicht | 500 |
| Zeichenabstand | −0,05 bis 0,30 em; 0 = Wert des Themes | 0 |
| Wortabstand | −0,10 bis 1,00 em; 0 = Wert des Themes | 0 |
| Absatzabstand | 0–3 em | 0,6 em |
| Ausrichtung | Links, Zentriert, Blocksatz | Links |
| Groß-/Kleinschreibung | Normal, GROSSBUCHSTABEN (nur Anzeige, das Skript bleibt unverändert) | Normal |
| Fokus-/Leselinie | Einstellungen und Ziehen in der Vorschau: 5–70 % der Fensterhöhe. Der Kern begrenzt auf 10–90 %, Werte unter 10 % werden also auf 10 % angehoben (bekannte Inkonsistenz) | 35 % |

Die Typografie-Werte liegen in den Kern-Einstellungen (`settings.json`) und gelten damit für Vorschau und Talent-Fenster. Eine Vorschau direkt unter der Schriftauswahl zeigt jede Änderung sofort; „Typografie zurücksetzen“ stellt die Standards wieder her. Nach jeder Änderung misst der Prompter die Zeilen neu, damit Lesemarker, Dimmen und Wort-Highlight stimmen.

### Schriftarten

Alle Schriften sind lokal gebündelt (`@fontsource`, OFL-1.1) und funktionieren offline.

| Gruppe | Schriften |
|---|---|
| Sans | Atkinson Hyperlegible Next, Inter, Lexend, Source Sans 3, Open Sans, Roboto Flex, IBM Plex Sans |
| Serif | Source Serif 4, Literata, Merriweather |
| Speziell | OpenDyslexic (für Legasthenie), JetBrains Mono |
| System | Systemschrift (Standard), **Systemschrift…** mit freiem Namen einer installierten Schrift (z. B. `Verdana`). Fehlt sie, gilt die Systemschrift |

Ältere `ui-prefs.json` bleiben gültig: `serif` ist jetzt Source Serif 4 (statt Georgia), `atkinson` die Next-Variante.

## Lesemarker und Farben

- **Lesemarker** (eigener Bereich *Einstellungen → Lesemarker*, gespeichert in `settings.json` unter `marker`): Stil per Kachel mit Mini-Vorschau – Linie, Band, Pfeile, Klammern, Fokus-Fenster, Verlauf, Unterstreichung (Wort oder Satz), Seitenbalken oder kein Marker. Dazu Farbe (Vorlagen aus dem Theme-Akzent oder eigener Farbwähler), Deckkraft bzw. Abdunklung, Dicke/Höhe (0,5–3×), Position (5–70 %), Dimmen von gelesenem und ungelesenem Text, Glow, Weichzeichnen außerhalb der Lesezone (Fokus-Fenster/Verlauf) und Animation beim Weiterspringen (Aus, Gleiten, Puls). Eine Live-Vorschau zeigt das Ergebnis; Talent-Fenster und Vorschau sehen identisch aus, auch gespiegelt. Ein früher gewähltes „Band“ wird automatisch übernommen.
- **Gelesenen Text dimmen**, mit einstellbarer Opazität (0,3–1). Ohne eigenen Wert gilt der Standard des Themes.
- **Eigene Farben** für Text, Hintergrund und Marker überschreiben das Theme. Achte dabei selbst auf ausreichenden Kontrast.
- **Erscheinungsbild der Oberfläche:** System, Hell oder Dunkel. Das betrifft nur das Control-Fenster, nicht den Prompter-Text.

## Tempo und Countdown

- WPM 40–400, Standard 140. Im Modus Konstant ändert auch das Mausrad über dem Prompter das Tempo (siehe [Tastenkürzel](shortcuts.md#mausrad)).
- Countdown 0 (sofort starten) oder mehr Sekunden, Standard 3. Umschalten im Transport.

## Tastenbelegung

In den Einstellungen unter Tastenkürzel klickst du auf eine Belegung und drückst die neue Kombination. Umbelegen lassen sich nur die Aktionen in `Settings.hotkeys`, die Liste steht unter [Tastenkürzel](shortcuts.md). Die übrigen App-Tasten und die globalen Hotkeys sind in v0.1 fest.

## Bewegung reduzieren

Mit „Bewegung reduzieren“ in den Einstellungen (`reducedMotion`) oder der Systemeinstellung `prefers-reduced-motion` schaltet der Prompter weiche Übergänge wie Resync-Ease, Puls und Countdown-Animation ab oder verkürzt sie. Das gilt auch für die Animationen der Oberfläche (Dialoge, Palette, Drawer, Hinweise).

## Oberfläche (UI-Theme)

Die Bedienoberfläche nutzt das tweakcn-Theme *Minimal Neutral*: neutrale Grautöne, Radius 1rem, Schriften DM Sans und Geist Mono (lokal gebündelt). Die Werte stehen in `docs/design/tokens.json` unter `ui`. Die sechs Prompter-Themes des Talent-Fensters sind davon unabhängig.
