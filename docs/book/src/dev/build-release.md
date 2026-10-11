# Build and release

Prerequisites (toolchain, system packages) are listed under [Installation](../user/installation.md). This page covers the build commands, the CI workflows, the Windows static C runtime, the sherpa-onnx prebuilt cache, lockfile handling and the release process.

## Toolchain

- Rust `stable` (`rust-toolchain.toml`: `channel = "stable"`, components `rustfmt` and `clippy`, profile `minimal`). The minimum supported Rust version is **1.80** (`workspace.package.rust-version`) and CI tests it.
- Node.js 22 (CI uses `actions/setup-node` with `node-version: 22`).
- `rustfmt.toml`: edition 2021, `max_width = 100`, `use_small_heuristics = "Max"`.
- Release profile: `lto = "thin"`, `codegen-units = 1`.

## Local commands

```bash
cargo test                                  # default-members: the six library crates incl. zt-testkit
cargo test --all-targets && cargo test --doc
cargo clippy --all-targets
cargo fmt --all -- --check
cargo build -p zt-audio --features cpal     # Linux: libasound2-dev
cargo build -p zt-stt --features sherpa     # downloads the prebuilt sherpa-onnx libraries (static)

cd apps/desktop
npm install && npx tauri icon app-icon.png
npm run check && npm test
npm run build                               # creates dist/ (required by tauri::generate_context!)
cd ../.. && cargo clippy -p zero-teleprompter --all-targets && cargo test -p zero-teleprompter
cd apps/desktop && npm run tauri build      # add: -- --features sherpa  (or sherpa,winspeech on Windows)
```

The platform icons (`icons/icon.icns` and others) are not checked in. `npx tauri icon app-icon.png` generates them, and CI and the release workflow do the same before building.

The app's Cargo features: `default = ["cpal", "download"]`; optional `sherpa` (real speech recognition) and `winspeech` (Windows speech fallback). Without `sherpa` the app runs voice tracking only in demo mode. The `build_info` command reports `cpal`, `sherpa` and `download` at runtime (not `winspeech`).

## CI (`.github/workflows/ci.yml`)

Triggers: push to `main`, pull requests, manual dispatch, and a weekly schedule (`17 3 * * 1`) for the extended property tests. Concurrency cancels superseded runs. Global `RUSTFLAGS: -D warnings`.

| Job | Runs on | Content |
|---|---|---|
| `core` | ubuntu-22.04, windows-latest, macos-latest | `cargo fmt --all && git diff --exit-code` (Linux only), `cargo clippy --all-targets`, `cargo test --all-targets`, `cargo test --doc` |
| `msrv` | ubuntu-22.04 | Installs 1.80 and runs `cargo +1.80 test --all-targets --locked` |
| `proptest-extended` | ubuntu-22.04, schedule and manual only | `PROPTEST_CASES=8192`, `cargo test --release -p zt-align -p zt-core -p zt-testkit`. Failing seeds (`**/proptest-regressions/**`) are uploaded as an artifact |
| `adapters` | ubuntu-22.04, windows-latest, macos-latest | `cargo build -p zt-audio --features cpal` and `cargo build -p zt-stt --features sherpa` (Linux installs `libasound2-dev`) |
| `stt-smoke` | ubuntu-22.04, windows-latest, windows-11-arm | End-to-end speech smoke test with real models, see [Testing](testing.md) |
| `frontend` | ubuntu-22.04 | `npm ci`, `npm run check`, `npm test`, `npm run build`; uploads `dist` as an artifact |
| `app` | ubuntu-22.04, windows-latest, macos-latest | Needs `frontend`. Icons, `clippy -p zero-teleprompter` (default features, `--features sherpa`, and `--features sherpa,winspeech`), `cargo test -p zero-teleprompter`, a release build with `--features sherpa` (informational size listing), and on Windows the `check-crt.ps1` runtime-import check |
| `licenses` | ubuntu-22.04 | `cargo-deny` (`licenses bans sources advisories`) and an npm license allow-list that excludes GPL, AGPL and LGPL |
| `winspeech` | windows-latest, windows-11-arm | clippy and tests of `zt-stt` with feature `winspeech`, a list of installed speech languages (informational), and `examples/winspeech_probe` for `de` and `en-US` (see [ADR-008](../adr/index.md)) |

`deny.toml` forbids GPL-family licenses by omission from the allow-list, bans the `sherpa-rs` crate, denies unknown registries and git sources, and lists documented ignores for transitive advisories in the Tauri dependency tree.

## Windows: static C runtime

`.cargo/config.toml` sets `-C target-feature=+crt-static` for `cfg(all(windows, target_env = "msvc"))` (x64 and ARM64). Reasons, as documented in the file:

- Without it the app needs `VCRUNTIME140.dll` and `MSVCP140.dll`, which are missing on fresh Windows installations (seen on Windows 11 ARM64).
- It is required by sherpa-onnx: the `sherpa-onnx-sys` Windows prebuilt libraries are only shipped as `static-MT` (static CRT), and mixing them with the default dynamic CRT fails to link.

**Pitfall:** an explicit `RUSTFLAGS` environment variable **overrides** the `rustflags` in `.cargo/config.toml`. Because CI sets `RUSTFLAGS=-D warnings`, every Windows job re-adds the flag in a first step ("Static MSVC CRT") by writing `RUSTFLAGS=-D warnings -C target-feature=+crt-static` to `GITHUB_ENV`. If you set `RUSTFLAGS` locally on Windows, add `-C target-feature=+crt-static` yourself.

`scripts/check-crt.ps1 <exe>` locates `dumpbin` through `vswhere`, prints the machine type and `/dependents`, and fails if any import matches `vcruntime` or `msvcp` followed by digits and `.dll`. CI runs it on `target/release/zero-teleprompter.exe` in the `app` job (Windows), in the release `build` job and in the `store` job.

## sherpa-onnx prebuilt libraries and the cache

The `sherpa-onnx` crate (pinned to `=1.13.8`) links statically by default. Without `SHERPA_ONNX_LIB_DIR` its build script downloads a prebuilt archive and unpacks it to `target/sherpa-onnx-prebuilt`.

All Rust jobs use `Swatinem/rust-cache@v2` with `prefix-key: v1-rust` (or a job-specific key) and an extra `cache-directories: target/sherpa-onnx-prebuilt`. The extra directory matters: a restored build-script output would otherwise point at a prebuilt directory that was not cached.

**Stale cache guard.** A partially restored cache (for example after a lockfile change) can leave `target/sherpa-onnx-prebuilt/<archive>/lib` without the libraries, and the build script trusts an existing `lib` directory. The jobs `stt-smoke`, `app` and the release `build` therefore run a shell step that deletes any prebuilt directory without a `*sherpa-onnx-c-api*` file in `lib`, and removes cached `sherpa-onnx-sys-*` release build-script output so the libraries are downloaded again. Keep this step if you edit the workflows.

## Lockfiles

`Cargo.lock` and `apps/desktop/package-lock.json` are committed. After any dependency change run:

```bash
gh workflow run lockfiles.yml
```

The `lockfiles.yml` workflow (manual dispatch, `contents: write`) runs `cargo generate-lockfile` with `CARGO_RESOLVER_INCOMPATIBLE_RUST_VERSIONS=fallback` so that every dependency compatible with Rust 1.80 is chosen where one exists, runs `npm install --package-lock-only`, and commits both files to the current branch if they changed.

The npm packages `@tauri-apps/api` and `@tauri-apps/plugin-dialog` are pinned with `~` (currently `~2.11.0` and `~2.7.0`) to the same minor version as the Rust crates `tauri` and `tauri-plugin-dialog` in the lockfile. A mismatch makes `tauri build` fail with a version conflict. Check `package.json` against `Cargo.lock` when you update either side.

## Release (`.github/workflows/release.yml`)

Trigger: a pushed tag matching `v*`, or manual dispatch. Permission: `contents: write`.

**Job `build`** (matrix, `fail-fast: false`), using `tauri-apps/tauri-action@v1` with `projectPath: apps/desktop`, `tagName: v__VERSION__`, `releaseName: 'zero-teleprompter v__VERSION__'`, `releaseDraft: true`, `prerelease: false`:

| Name | Runner | Extra arguments |
|---|---|---|
| macOS arm64 | macos-latest | `--target aarch64-apple-darwin --features sherpa` |
| macOS x64 | macos-latest | `--target x86_64-apple-darwin --features sherpa` |
| Linux x64 | ubuntu-22.04 | `--features sherpa` |
| Windows x64 | windows-latest | `--features sherpa,winspeech` |
| Windows ARM64 | windows-11-arm (native runner) | `--features sherpa,winspeech` |

The native ARM64 runner avoids cross-compiling C dependencies such as `ring` and `cpal` from x64. After the build, Windows jobs run `check-crt.ps1`. The tag name comes from the app version, so a manual run on a branch does not create a tag named after the branch. The release body text in the workflow is currently German.

**Job `store`** (needs `build`; x64 on windows-latest, ARM64 on windows-11-arm): builds an NSIS installer with `npx tauri build --bundles nsis --config src-tauri/tauri.microsoftstore.conf.json --features sherpa,winspeech`, verifies the static CRT, and renames the installer to `zero-teleprompter_<version>_<arch>_store-setup.exe`. See [Microsoft Store](ms-store.md).

**Job `store-upload`** (needs `store`): downloads the `store-*` artifacts and runs `gh release upload "v<version>" store-out/* --clobber` to attach them to the draft.

**Signing is optional.** A step exports the `APPLE_*` and `TAURI_SIGNING_PRIVATE_KEY[_PASSWORD]` secrets into the environment only when they are non-empty, because empty `APPLE_*` variables make the bundler try to import an empty certificate and fail. Without the secrets, the packages are unsigned. Windows code signing is not configured (see the Store page for the consequence).

Bundler settings in `tauri.conf.json`: identifier `dev.zeroteleprompter.app`, category Productivity, targets `all`, NSIS `installMode: currentUser`, macOS minimum system version 11.0 with `entitlements.plist`, Debian package dependency `libasound2`.

### Release checklist

1. Align the version in `Cargo.toml` (`workspace.package`), `apps/desktop/package.json` and `apps/desktop/src-tauri/tauri.conf.json`.
2. In `CHANGELOG.md`, date the "Unreleased" section.
3. Review `THIRD_PARTY_NOTICES.md` and `THIRD_PARTY_MODELS.md`, and run `cargo deny check`.
4. Push the tag `vX.Y.Z`. Inspect the draft release, then publish it.
5. For the Store, see the steps on the [Microsoft Store](ms-store.md) page.
