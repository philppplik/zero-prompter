# Build & Release

Die Voraussetzungen stehen unter [Installation](../user/installation.md#1-toolchain).

## Lokale Befehle

```bash
cargo test                                  # default-members: alle Kern-Crates inkl. zt-testkit
cargo clippy --all-targets
cargo fmt --all -- --check
cargo build -p zt-audio --features cpal     # Linux: libasound2-dev
cargo build -p zt-stt --features sherpa     # VERIFY: sherpa-onnx-Binding

cd apps/desktop
npm install && npx tauri icon app-icon.png
npm run check && npm test
npm run build                               # erzeugt dist/ (für generate_context! nötig)
cd ../.. && cargo clippy -p zero-teleprompter --all-targets && cargo test -p zero-teleprompter
cd apps/desktop && npm run tauri build
```

Die Icons (`icons/icon.icns` u. a.) sind nicht eingecheckt. `npx tauri icon` erzeugt sie aus `app-icon.png`.

## CI (`.github/workflows/ci.yml`)

| Job | Plattformen | Inhalt |
|---|---|---|
| `core` | Ubuntu 22.04, Windows, macOS | fmt (nur Linux), clippy, `cargo test --all-targets`, Doctests. `RUSTFLAGS=-D warnings` |
| `msrv` | Ubuntu | `cargo +1.80 test --all-targets` |
| `proptest-extended` | Ubuntu, wöchentlich und manuell | `PROPTEST_CASES=8192`. Fehlschlagende Seeds werden als Artefakt gesichert |
| `adapters` | alle drei | Build mit `cpal`, Build mit `sherpa` (**continue-on-error**) |
| `frontend` | Ubuntu | `npm install` (noch kein Lockfile), check, test, build. `dist` als Artefakt |
| `app` | alle drei | Icons, clippy, Test und Release-Build der App |
| `licenses` | Ubuntu | `cargo-deny` (licenses, bans, sources, advisories) und npm-Lizenz-Allowlist ohne GPL, AGPL oder LGPL |

## Release (`.github/workflows/release.yml`)

Auslöser: ein Tag `v*` oder manueller Start. `tauri-apps/tauri-action@v0` baut:
- macOS arm64 und x64,
- Linux x64,
- Windows x64,

und legt alles als **Draft-Release** ab.

Signierung: Die Secrets `APPLE_*` und `TAURI_SIGNING_PRIVATE_KEY[_PASSWORD]` sind optional. Ohne sie entstehen unsignierte Pakete.

Checkliste für ein Release:
1. Version in `Cargo.toml` (`workspace.package`), `apps/desktop/package.json` und `tauri.conf.json` angleichen.
2. `CHANGELOG.md`: den Abschnitt „Unreleased“ datieren.
3. `THIRD_PARTY_NOTICES.md` und `THIRD_PARTY_MODELS.md` prüfen und `cargo deny check` ausführen.
4. Tag `vX.Y.Z` pushen. Den Draft prüfen, dann veröffentlichen.

## Feature-Matrix der App

`default = ["cpal", "download"]`, optional `sherpa`. Die tatsächlich gebauten Features meldet der Command `build_info` zur Laufzeit.
