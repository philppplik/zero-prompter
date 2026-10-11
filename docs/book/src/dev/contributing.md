# Contributing

The authoritative version is [`CONTRIBUTING.md`](https://github.com/philppplik/zero-prompter/blob/main/CONTRIBUTING.md) in the repository root; read it together with the Code of Conduct. This page is a summary.

## Good first contributions

- Build the app on your platform, run the tests, and report what breaks.
- Resolve open `// VERIFY` markers (`rg "VERIFY"`) by checking the upstream API or behavior and replacing the marker with a verified comment.
- Improve the documentation or add missing screenshots.

## Licensing

- Contributions are licensed under `MIT OR Apache-2.0` (inbound = outbound). There is no CLA. Submit only code you have the right to submit.
- **Forbidden:** code under GPL, AGPL or LGPL. QPrompt and Imaginary Teleprompter serve as UX references only. `cargo-deny` and the npm license check in CI enforce this.
- **Allowed with attribution:** code or ideas from MIT or Apache-2.0 projects (for example OpenTeleprompter, EdNutting/autocue, Textream). This requires
  1. a comment at the adopted spot naming the project, URL, license and whether it is code or an idea, and
  2. an entry in `THIRD_PARTY_NOTICES.md` (license, copyright line, commit, purpose, files).
- **Models:** add an entry to `THIRD_PARTY_MODELS.md` and to the manifest (`apps/desktop/src-tauri/resources/models.json`) with an SPDX `license` and, for CC-BY and similar licenses, an `attribution`. Use only verified URLs (pinned to a commit or release asset), sizes and SHA-256 values.

## Workflow

1. For larger changes, open an issue first. Architecture decisions get an ADR in `docs/adr/` (MADR format).
2. Create a branch and keep the change small and focused.
3. Check locally (the same as CI):
   ```bash
   cargo fmt --all
   cargo clippy --all-targets
   cargo test --all-targets
   cargo test --doc
   cd apps/desktop && npm run check && npm test
   ```
   For app changes also run `npm run build` followed by `cargo clippy -p zero-teleprompter --all-targets` (the app crate needs `apps/desktop/dist`). If you touch speech code, also build with `--features sherpa` (and `--features sherpa,winspeech` on Windows).
4. Update the documentation in the same pull request. New or changed commands, events, settings or shortcuts belong in `docs/book/src/reference/` or `docs/book/src/user/shortcuts.md`. Add an entry under "Unreleased" in `CHANGELOG.md`.
5. Open the pull request and describe what changed, why, and how you tested it.

CI must pass. It builds with `RUSTFLAGS=-D warnings`, so warnings fail the build. See [Build and release](build-release.md) for the job list.

## Toolchain

- Rust `stable` (`rust-toolchain.toml`), MSRV 1.80, tested in CI with `--locked`. Do not raise dependency versions in a way that breaks the MSRV; the lockfile is generated MSRV-aware by the `lockfiles.yml` workflow.
- Node.js 22.
- Formatting: `rustfmt.toml` for Rust, Prettier for the frontend (`npm run format`).

## Code rules

- Core crates (`zt-core`, `zt-align`, `zt-models`) stay free of I/O, clocks and threads. Time is passed in as a parameter.
- Wrap unreliable external APIs behind your own trait and mark them `// VERIFY: ...` until verified.
- `unsafe` is forbidden in the workspace crates; the app crate uses `deny` because of Tauri macro output.
- In the frontend, only `lib/bridge.ts` knows IPC names. Actions live in `lib/actions.ts`; colors and sizes come from `tokens.json`.
- Terminology in docs and UI text: "control window", "talent window", "voice tracking", "streaming model", "resync".
- Details: [Code style](code-style.md).

## Adding things: where to touch

| Change | Touch |
|---|---|
| New command | `zt_core::Command` and `Controller::apply`; the IPC reference; `lib/types.ts`; tests in `zt-testkit` or `zt-core` |
| New setting | `zt_core::Settings` (default and `sanitized`), `lib/types.ts`, the settings reference page, and the UI |
| New speech engine | An adapter implementing `SpeechEngine` in `zt-stt` behind a feature; wire it into `models::build_engine` or `voice::make_engine`; a mock-based test; a CI build job |
| New model | `resources/models.json` (schema 2: `modelType`, `roles`, `defaultRank`, files with URL, size, SHA-256 where available), `THIRD_PARTY_MODELS.md`, and a case in `scripts/stt-smoke.py` |
| Tracker tuning | `TrackerConfig` or `SwParams` defaults, the scenario table in `zt-testkit`, and the [Aligner algorithm](aligner.md) page |

## Reporting bugs

Include the operating system and version, the app version and features (command `build_info`), the engine information from the inspector (demo or sherpa, which model), and steps to reproduce. For voice problems, attach the log export from **Settings, Diagnostics** and a short script sample. Do not attach recordings of other people without their consent.

Security problems: do not open a public issue; see `SECURITY.md`.
