# WebSocket-Protokoll (geplant)

> **Nicht implementiert.** In v0.1 gibt es weder die Crate `zt-remote` noch einen Server oder WebSocket. Diese Seite hält den **Entwurf** aus `docs/concept/CONCEPT.md` §8.2 und §5 F5 fest, damit spätere Arbeit daran anknüpfen kann. Alles hier kann sich noch ändern.

## Rahmen (geplant für v0.2)

- Crate `zt-remote`: axum, WebSocket, PWA-Assets, QR-Code. Opt-in in den Einstellungen unter Fernsteuerung, nur im LAN.
- Pairing: QR-Code mit LAN-Adresse, dazu eine 6-stellige PIN. Nach erfolgreicher Prüfung gibt es einen Session-Token.
- Fällt die Verbindung aus, läuft der Prompter unverändert weiter. Der Operator sieht einen Hinweis.
- Jede Remote-Aktion wird zu einem `zt_core::Command` und läuft über denselben Pfad `appcore::dispatch` wie Tastatur und UI.

## Nachrichten v1 (JSON, Entwurf)

Client → Server:

```json
{ "t": "hello", "pin": "123456" }
{ "t": "cmd", "cmd": { "type": "togglePlay" } }
{ "t": "cmd", "cmd": { "type": "nudgeWpm", "value": 5 } }
```

Server → Client:

```json
{ "t": "state", "patch": { "playback": { "state": { "kind": "playing" } } } }
{ "t": "err", "…": "…" }
```

`cmd` hätte dasselbe Format wie die [Commands der Tauri-Bridge](ipc.md#command), `patch` dasselbe wie `state://patch`.

## Offene Punkte

- Fehlerformat (`err`) und Versionierung (`hello` mit Protokollversion?).
- Gültigkeit des Tokens, Wiederverbinden und Begrenzung der PIN-Versuche.
- Welche Commands darf die Fernsteuerung senden? Vermutlich kein `loadScript`/`replaceSettings`, sondern eine Positivliste.
- TLS im LAN, oder bewusst nur HTTP mit PIN.

Die Entscheidung wird als ADR festgehalten, bevor die Implementierung beginnt.
