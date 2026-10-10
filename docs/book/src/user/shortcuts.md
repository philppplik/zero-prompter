# Tastenkürzel

Unter macOS steht `Ctrl` für `⌘`. Die App behandelt Meta wie Ctrl, sodass eine Belegung überall funktioniert. In Textfeldern lösen einfache Tasten keine Aktionen aus. `?` zeigt diese Liste in der App an.

## Umbelegbar (`Settings.hotkeys`)

| Aktion (ID) | Standard | Wirkung |
|---|---|---|
| `togglePlay` | `Space` | Play/Pause, mit Countdown beim Start |
| `stop` | `Home` | Live: Stop und zurück zum Anfang. Edit: zum Anfang |
| `faster` | `↑` | +5 WPM |
| `slower` | `↓` | −5 WPM |
| `nextParagraph` | `PgDn` | nächster Absatz |
| `prevParagraph` | `PgUp` | vorheriger Absatz |
| `toggleMirror` | `M` | horizontal spiegeln |
| `toggleFullscreen` | `F` | Vollbild des Talent-Fensters. Öffnet das Fenster bei Bedarf |
| `toggleVoice` | `V` | Voice an/aus |
| `resyncHere` | `R` | Resync an der Markerposition |
| `commandPalette` | `Ctrl+K` | Befehlspalette |
| `shortcutOverlay` | `?` | Übersicht der Tastenkürzel |

## Fest belegt (v0.1)

| Taste | Wirkung |
|---|---|
| `Shift+M` | vertikal spiegeln |
| `Ctrl+E` | Edit- und Live-Modus umschalten |
| `Ctrl+O` | Skript öffnen |
| `Ctrl+B` | Skripte & Gliederung (Drawer) ein/aus |
| `Ctrl+,` | Einstellungen öffnen/schließen |
| `Ctrl+=` / `Ctrl+-` | Schrift ±4 px |
| `Ctrl+1` … `Ctrl+6` | Theme 1–6 |
| `Ctrl+↑` | zum Anfang (Alternative für Mac-Tastaturen ohne `Home`) |
| `Esc` | schließt der Reihe nach: Palette, Übersicht, Einstellungen, Skripte-Drawer. Bricht danach den Countdown ab bzw. beendet zuletzt das Vollbild |

## Global (systemweit, auch wenn eine andere App den Fokus hat)

| Taste | Wirkung |
|---|---|
| `Ctrl+Shift+F6` | Voice an/aus |
| `Ctrl+Shift+F7` | Play/Pause |
| `Ctrl+Shift+F8` | −5 WPM |
| `Ctrl+Shift+F9` | +5 WPM |
| `Ctrl+Shift+F10` | Absatz zurück |
| `Ctrl+Shift+F11` | Absatz vor |

Die globalen Hotkeys liegen seit der Überarbeitung vom 2026-10-10 auf F-Tasten. Die früheren Kombinationen (`Ctrl+Shift+V`, `Ctrl+Shift+Space`, `Ctrl+Shift+Pfeile/PgUp/PgDn`) kollidierten mit „als Text einfügen“, Eingabemethoden und der Textauswahl in Editoren. `Ctrl+Alt` (= AltGr auf deutschen Tastaturen), `Ctrl+Shift+Esc` und `F12` werden bewusst nicht benutzt. Ob die F-Kombinationen auf allen Systemen frei sind, ist **nicht geprüft**. Die Anzeige in der App (`GLOBAL_HOTKEYS` in `src/lib/hotkeys.ts`) muss von Hand mit `src-tauri/src/hotkeys.rs` übereinstimmen.

Ist eine globale Kombination schon von einer anderen App belegt, protokolliert zero-teleprompter das und läuft ohne diese Kombination weiter. Unter Wayland funktionieren globale Hotkeys je nach Compositor gar nicht (Portal-Unterstützung ist für v0.2 geplant).
