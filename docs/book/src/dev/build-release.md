# Build & Release

Die Voraussetzungen stehen unter [Installation](../user/installation.md#1-toolchain).

## Lokale Befehle

```bash
cargo test                                  # default-members: alle Kern-Crates inkl. zt-testkit
cargo clippy --all-targets
cargo fmt --all -- --check
cargo build -p zt-audio --features cpal     # Linux: libasound2-dev
cargo build -p zt-stt --features sherpa     # lädt prebuilt sherpa-onnx-Libs (statisch)

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
| `msrv` | Ubuntu | `cargo +1.80 test --all-targets --locked` |
| `proptest-extended` | Ubuntu, wöchentlich und manuell | `PROPTEST_CASES=8192`. Fehlschlagende Seeds werden als Artefakt gesichert |
| `adapters` | alle drei | Build mit `cpal`, Build mit `sherpa` (blockierend) |
| `frontend` | Ubuntu | `npm ci` (Lockfile eingecheckt), check, test, build. `dist` als Artefakt |
| `app` | alle drei | Icons, clippy (auch `--features sherpa`), Test, Release-Build mit `sherpa`; Windows: `dumpbin`-Prüfung |
| `licenses` | Ubuntu | `cargo-deny` (licenses, bans, sources, advisories) und npm-Lizenz-Allowlist ohne GPL, AGPL oder LGPL |

## Release (`.github/workflows/release.yml`)

Auslöser: ein Tag `v*` oder manueller Start. `tauri-apps/tauri-action@v1` baut:
- macOS arm64 und x64,
- Linux x64,
- Windows x64 und ARM64 (nativer Runner `windows-11-arm`),
- zusätzlich Store-Installer `*_store-setup.exe` (x64, ARM64) mit Offline-WebView2, siehe [Microsoft Store](ms-store.md),

und legt alles als **Draft-Release** ab. Alle Release-Builds nutzen `--features sherpa`.

## Windows: statische C-Laufzeit

`.cargo/config.toml` setzt für alle `*-pc-windows-msvc`-Targets `-C target-feature=+crt-static`. Damit braucht die App weder `VCRUNTIME140.dll` noch `MSVCP140.dll` (fehlen auf frischen Windows-Installationen, v. a. ARM64). Das ist zugleich Pflicht für sherpa-onnx: `sherpa-onnx-sys` liefert für Windows (x64 und ARM64) nur `static-MT`-Archive (statische CRT). Achtung: eine gesetzte Umgebungsvariable `RUSTFLAGS` überschreibt die Datei; CI setzt das Flag auf Windows daher zusätzlich. `scripts/check-crt.ps1` prüft per `dumpbin /dependents`, dass keine VC++-Laufzeit importiert wird.

Signierung: Die Secrets `APPLE_*` und `TAURI_SIGNING_PRIVATE_KEY[_PASSWORD]` sind optional. Ohne sie entstehen unsignierte Pakete.

Checkliste für ein Release:
1. Version in `Cargo.toml` (`workspace.package`), `apps/desktop/package.json` und `tauri.conf.json` angleichen.
2. `CHANGELOG.md`: den Abschnitt „Unreleased“ datieren.
3. `THIRD_PARTY_NOTICES.md` und `THIRD_PARTY_MODELS.md` prüfen und `cargo deny check` ausführen.
4. Tag `vX.Y.Z` pushen. Den Draft prüfen, dann veröffentlichen.

## Lockfiles

`Cargo.lock` und `apps/desktop/package-lock.json` sind eingecheckt. Nach Änderungen an Abhängigkeiten: `gh workflow run lockfiles.yml`. Der Workflow erzeugt beide Dateien MSRV-bewusst (`CARGO_RESOLVER_INCOMPATIBLE_RUST_VERSIONS=fallback`) und committet sie. Die npm-Pakete `@tauri-apps/api` und `@tauri-apps/plugin-dialog` sind per `~` auf dieselbe Minor-Version gepinnt wie die Rust-Crates `tauri` und `tauri-plugin-dialog` im Lockfile, sonst bricht `tauri build` mit einem Versionskonflikt ab.

## Feature-Matrix der App

`default = ["cpal", "download"]`, optional `sherpa`. Die tatsächlich gebauten Features meldet der Command `build_info` zur Laufzeit.
