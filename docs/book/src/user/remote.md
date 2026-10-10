# Fernsteuerung

> **In v0.1 nicht implementiert.** Es gibt im Code keinen Netzwerkserver, keinen WebSocket, kein MIDI/HID und keine Gamepad-Unterstützung.

## Was in v0.1 geht

- **Globale Hotkeys** (`Ctrl+Shift+…`, siehe [Tastenkürzel](shortcuts.md)). Damit steuerst du den Prompter, während OBS, Folien oder eine Videokonferenz im Vordergrund sind.
- **Presenter-Clicker und Fußpedale**, die sich als Tastatur ausgeben: Wenn das Gerät `PgUp`/`PgDn`, Pfeiltasten oder `Space` sendet, funktioniert es bei fokussiertem Control-Fenster ohne weitere Einrichtung. Bei Bedarf belegst du die Aktionen im Inspector passend um.

## Geplant

| Version | Funktion (laut CONCEPT §3.5, §5 F5) |
|---|---|
| 0.2 | Handy-Fernbedienung als PWA im LAN: QR-Code, 6-stellige PIN, Session-Token. Play/Pause, Tempo, Absatz ±, Spiegeln, Modus, Status und Mini-Vorschau. Bricht die Verbindung ab, läuft der Prompter unverändert weiter |
| 0.2 | MIDI (midir), HID (hidapi), Gamepad (gilrs) mit „Learn-Key“ |
| 1.0 | mDNS-Discovery, OSC, OBS-WebSocket, Director-Remote-Editor |

Den Protokollentwurf findest du unter [WebSocket-Protokoll (geplant)](../reference/ws-protocol.md). Alle Fernsteuerungsquellen sollen dieselben `Command`s erzeugen wie die Tastatur. Die Architektur ist dafür vorbereitet, siehe [Datenfluss](../dev/dataflow.md).
