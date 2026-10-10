# Erste Schritte

## Onboarding

Beim ersten Start (solange `onboarded` in `ui-prefs.json` nicht gesetzt ist) führt ein Assistent durch vier Schritte:

1. **Willkommen**
2. **Mikrofon:** Freigabe und Pegelanzeige über den Mikrofontest.
3. **Sprachmodell:** Status und Download, sofern das Manifest Einträge hat.
4. **Ausgabe:** Monitor für das Talent-Fenster wählen.

Den Assistenten startest du jederzeit erneut über die Befehlspalette (`Ctrl+K` → „Einführung erneut starten“).

## Das Control-Fenster

```
┌ Sidebar ─────┬ Editor / Vorschau ─────────────┬ Inspector ─────┐
│ Skripte      │                                │ Darstellung    │
│ Gliederung   │                                │ Scroll, Marker │
│              │                                │ Voice, Ausgabe │
├──────────────┴─── Transport: ⏮ ◀¶ ▶ ¶▶ · WPM · Position · Zeit · Countdown ┤
```

- **Sidebar:** Skriptbibliothek (neu, öffnen, löschen) und eine Gliederung nach Absätzen. Ein Klick auf einen Absatz springt dorthin.
- **Editor / Vorschau:** Im Edit-Modus schreibst du Text, Markdown ist erlaubt. Im Live-Modus siehst du die Prompter-Vorschau. Umschalten mit `Ctrl+E`.
- **Inspector:** Einstellungen, siehe [Anpassung](customization.md).
- **Transport:** Zum Anfang, Absatz zurück/vor, Play/Pause, Tempo, Positionsleiste, Restzeit/Gesamtdauer und Countdown-Dauer.

## Skript laden

- **Öffnen:** `Ctrl+O` oder Ordner-Symbol in der Sidebar. Erlaubt sind `.txt`, `.md`, `.markdown` und `.text`, höchstens 8 MB. Dateien in UTF-8 (auch mit BOM) werden gelesen. Andere Kodierungen interpretiert die App als Latin-1, statt abzubrechen.
- **Drag-and-Drop** einer Datei auf das Fenster.
- **Neu:** „+“ in der Sidebar.

Importierte Skripte kopiert die App als `<name>-<zeitstempel>.md` in die Bibliothek.

### Skriptformat

- **Absätze** trennst du durch eine Leerzeile. `PgUp`/`PgDn` springen absatzweise.
- **Regieanweisungen** stehen in eckigen Klammern, z. B. `[PAUSE]` oder `[Blick in Kamera]`. Sie werden angezeigt, zählen aber nicht als gesprochene Wörter. Das Voice-Tracking überspringt sie, und sie fließen nicht in Wortzahl und Dauer ein.
- Die erste nicht leere Zeile (ohne führende `#`) wird zum Titel in der Bibliothek.

## Proben und live gehen

1. `Ctrl+E` wechselt in den **Live-Modus**. Während der Wiedergabe ist der Text gesperrt.
2. Tempo einstellen: `↑`/`↓` je ±5 WPM, Bereich 40–400.
3. `Leertaste` startet den Countdown (Standard 3 s, im Transport umschaltbar). `Esc` bricht ihn ab.
4. **Talent-Fenster:** Befehlspalette → „Talent-Fenster öffnen“ oder Monitor im Inspector wählen. `F` schaltet Vollbild.
5. **Spiegeln:** `M` horizontal, `Shift+M` vertikal.
6. `Home` springt zum Anfang (im Live-Modus: Stop).

Am Skriptende wechselt die Wiedergabe in den Zustand „Ende“.

## Befehlspalette

`Ctrl+K` (macOS `⌘K`) öffnet eine unscharfe Suche über alle Aktionen, gruppiert nach Wiedergabe, Darstellung, Themes, Ausgabe, Voice, Skripte und Hilfe. `?` zeigt alle Tastenkürzel an.
