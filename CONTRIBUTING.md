# Contributing to zero-teleprompter

Thank you for your interest. Detailed developer documentation lives in `docs/book/` (the "developer" part of the manual). Please also read the [Code of Conduct](CODE_OF_CONDUCT.md).

## Good first contributions

- Build the app on your platform, run the tests, and report what breaks.
- Resolve open `// VERIFY` markers in the source (`rg "VERIFY"`) by checking the upstream API or behavior and replacing the marker with a verified comment.
- Improve the English documentation or add missing screenshots.

## License of your contributions

By opening a pull request you license your contribution under `MIT OR Apache-2.0` (inbound = outbound, no CLA). Only submit code you have the right to submit.

## Third-party code and ideas

- **Forbidden:** code under GPL, AGPL or LGPL. QPrompt and Imaginary Teleprompter are UX references only. `cargo-deny` and the npm license check in CI enforce this.
- **Allowed with attribution:** MIT or Apache-2.0 projects such as OpenTeleprompter (ArunNGun/openTeleprompt), EdNutting/autocue or f/textream. This requires
  1. a comment at the adopted spot: project, URL, license, and whether it is code or only an idea;
  2. an entry in `THIRD_PARTY_NOTICES.md`: license, copyright line, commit hash, purpose, files.
- **Models:** add an entry to `THIRD_PARTY_MODELS.md` and to the manifest (`apps/desktop/src-tauri/resources/models.json`) with an SPDX `license` and, for CC-BY, an `attribution`. Use only verified URLs (pinned to a commit or release asset), sizes and SHA-256 values.

## Workflow

1. For larger changes, open an issue first. Architecture decisions get an ADR in `docs/adr/`.
2. Create a branch and keep the change small and focused.
3. Check locally (same as CI):
   ```bash
   cargo fmt --all
   cargo clippy --all-targets
   cargo test --all-targets
   cargo test --doc
   cd apps/desktop && npm run check && npm test
   ```
   For app changes also run `npm run build` followed by `cargo clippy -p zero-teleprompter --all-targets` (the app crate needs `apps/desktop/dist`). If you touch speech code, also build with `--features sherpa` (and `--features sherpa,winspeech` on Windows).
4. Update the docs in the same pull request. New or changed commands, events, settings or shortcuts belong in `docs/book/src/reference/` or `docs/book/src/user/shortcuts.md`. Add an entry under "Unreleased" in `CHANGELOG.md`.
5. Open the pull request with a description: what, why, how it was tested.

CI must pass. It builds with `RUSTFLAGS=-D warnings`, so warnings fail the build.

## Toolchain

- Rust: `stable` per `rust-toolchain.toml`; the MSRV is 1.80 and CI tests it with `--locked`. Do not raise dependency versions in a way that breaks MSRV (the lockfile is generated MSRV-aware, see `.github/workflows/lockfiles.yml`).
- Node.js 22 (as in CI).
- Formatting: `rustfmt.toml` for Rust, Prettier for the frontend (`npm run format`).

## Code rules

- Core crates (`zt-core`, `zt-align`, `zt-models`) stay free of I/O, clocks and threads. Time is passed in as a parameter.
- Unreliable external APIs are wrapped behind your own trait and marked `// VERIFY: ...` until verified.
- `unsafe` is forbidden in the workspace crates (`unsafe_code = "forbid"`); the app crate uses `deny` because of Tauri macro output.
- In the frontend only `lib/bridge.ts` knows IPC names. Actions live in `lib/actions.ts`; colors and sizes come from `tokens.json`.
- Terminology in docs and UI text: "control window", "talent window", "voice tracking", "streaming model", "resync".
- Details: `docs/book/src/dev/code-style.md`.

## Reporting bugs

Please include: operating system and version, app version and features (command `build_info`), engine info from the inspector (demo or sherpa, which model), and steps to reproduce. For voice problems, attach an export of the log from **Settings, Diagnostics** and a short script sample. Do not attach recordings of other people without their consent.

Security problems: do not open a public issue; see [SECURITY.md](SECURITY.md).
