# Remote control

> **Not implemented in v0.1.** The code contains no network server, no WebSocket, no MIDI or HID support and no gamepad support.

## What works in v0.1

- **Global hotkeys.** `Ctrl+Shift+F6` to `Ctrl+Shift+F11` control voice, play or pause, tempo and paragraph jumps while OBS, slides or a video call have focus. See [Keyboard shortcuts](shortcuts.md#global-shortcuts-system-wide).
- **Presenter clickers and foot pedals that act as a keyboard.** If the device sends `PgUp`, `PgDn`, arrow keys or `Space`, it works when the control window has focus, with no setup. Remap the actions in Settings, Shortcuts if needed. Note that these keys act on the focused control window; only the F-key combinations above work when another app has focus.
- **Second monitor output.** The talent window is a separate window that you can place on another monitor, which is the usual way to hand the reading view to a person while you operate from the control window.

## Planned

The plan comes from the project concept (`docs/concept/CONCEPT.md`, sections 3.5 and 5, F5). It is not a commitment, and none of it exists in the code yet.

| Version | Feature |
|---|---|
| 0.2 | Phone remote as a web app on the local network: QR code, 6-digit PIN, session token. Play or pause, tempo, paragraph jumps, mirroring, mode, status and a mini preview. If the connection drops, the prompter keeps running unchanged. |
| 0.2 | MIDI (`midir`), HID (`hidapi`), gamepad (`gilrs`) with a "learn key" function. |
| 1.0 | mDNS discovery, OSC, OBS WebSocket, a director remote editor. |

The draft protocol is on the page [WebSocket protocol (planned)](../reference/ws-protocol.md). All remote sources are intended to produce the same `Command` values as the keyboard; the architecture is prepared for that, see [Data flow](../dev/dataflow.md).
