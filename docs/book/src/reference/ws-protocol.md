# WebSocket Protocol (Planned)

> **Not implemented.** As of v0.1 there is no `zt-remote` crate, no server and no WebSocket. This page records the **draft** from `docs/concept/CONCEPT.md` (section 8.2 and feature F5) so later work has a starting point. Everything here may change.

## Framework (planned for v0.2)

- Crate `zt-remote`: axum, WebSocket, PWA assets, QR code. Opt-in under the remote-control settings, LAN only.
- Pairing: a QR code carrying the LAN address, plus a 6-digit PIN. After the PIN is verified the server issues a session token.
- If the connection drops, the prompter continues unchanged and the operator sees a notice.
- Every remote action becomes a `zt_core::Command` and runs through the same `appcore::dispatch` path as keyboard and UI input.

## Messages v1 (JSON, draft)

Client to server:

```json
{ "t": "hello", "pin": "123456" }
{ "t": "cmd", "cmd": { "type": "togglePlay" } }
{ "t": "cmd", "cmd": { "type": "nudgeWpm", "value": 5 } }
```

Server to client:

```json
{ "t": "state", "patch": { "playback": { "state": { "kind": "playing" } } } }
{ "t": "err", "...": "..." }
```

`cmd` would use the same format as the [Tauri bridge commands](ipc.md#command); `patch` would use the same format as `state://patch`.

## Open questions

- Error format (`err`) and versioning (for example a protocol version in `hello`).
- Token validity, reconnection, and rate-limiting of PIN attempts.
- Which commands remote clients may send. Probably not `loadScript` or `replaceSettings`; an allow-list is likely.
- TLS on the LAN, or deliberately plain HTTP with a PIN.

The decision will be recorded as an ADR before implementation starts.
