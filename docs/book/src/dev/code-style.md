# Code style

## Rust

- **Formatting:** `rustfmt` with the repository's `rustfmt.toml` (stable options only: edition 2021, `max_width = 100`, `use_small_heuristics = "Max"`). CI runs `cargo fmt --all && git diff --exit-code` on Linux and fails on any difference.
- **Clippy:** `clippy::all` is a warning in the workspace lints. CI builds with `RUSTFLAGS=-D warnings`, which turns warnings into errors.
- **`unsafe`:** `unsafe_code = "forbid"` in all workspace crates. The app crate uses `deny` because code expanded from Tauri macros (`generate_context!`, `generate_handler!`) cannot satisfy `forbid`. The app crate therefore does not inherit `[workspace.lints]`.
- **Debug:** `missing_debug_implementations = "warn"`. Public types implement `Debug`, by hand with `finish_non_exhaustive()` when a field is not `Debug`.
- **MSRV 1.80:** do not use newer standard-library APIs. Example: `Option::is_none_or` needs Rust 1.82, so the code writes `!x.is_some_and(...)` instead (see `sw.rs`).
- **Errors:** `thiserror` in the library crates. At the Tauri boundary errors become `String`, and the frontend shows them as toasts. Pipeline problems go through `diag::report_error` (log, ring buffer, toast) rather than `eprintln!`.
- **Purity:** the core crates stay **free of I/O, clocks and threads**. Time is passed in as a parameter (`Command::Tick`, `t_ms`). Adapters live behind traits and Cargo features.
- **Serde and the frontend contract:** `rename_all = "camelCase"` for everything the frontend sees. Exceptions without a rename, which therefore appear in snake_case: `Script`, `Token`, `Paragraph` and `Playback` (`is_cue`, `countdown_secs`, `ms_left`). `UiMode` has no rename either and serializes as `Edit` and `Live`. `apps/desktop/src/lib/types.ts` mirrors this and must stay in sync with the Rust types.
- **Language:** code comments, identifiers and the book are in English. A few identifiers and UI strings are still German (for example the scenario categories in `zt-testkit`, `WINSPEECH_NAME`, and some log and error texts in the app). Do not mix languages within a new comment or message.
- **Module headers:** `//!` comments explain the role of the module and refer to the relevant ADR or concept section.
- **`// VERIFY:`** marks every spot written against an external API that has not been checked by compiling or running (find them with `rg "VERIFY"`). Replace the marker with a verified comment once you have checked it.
- **Logging:** use `tracing` with a `target` of the form `zt::<area>` (`zt::voice`, `zt::engine`, `zt::stt`, ...). Debug-level events may be frequent; info-level events should be rare.
- **Hot paths:** the audio callback must never block or allocate large buffers; use `try_lock`, `try_send` and bounded channels. The streaming speech path must never wait for the offline resync worker.

## TypeScript and Svelte

- Svelte 5 with runes (`$state`, `$derived`), TypeScript in strict mode (`tsconfig.json`).
- Only `src/lib/bridge.ts` knows the names of commands and events. Components import `api` and `events` from there.
- Every user action is defined once in `src/lib/actions.ts` and reused by hotkeys, the command palette and buttons.
- Colors and sizes come from `docs/design/tokens.json` (through `src/lib/tokens.ts`). No hard-coded color values in components.
- Accessibility: ARIA roles and labels on controls, focus ring from the tokens, and respect for `prefers-reduced-motion` (plus the `reducedMotion` setting).
- Formatting: Prettier with `prettier-plugin-svelte` (`npm run format`, `npm run format:check`). Type checking: `npm run check`.

## Documentation

- The handbook lives in `docs/book/src/` and is written in US English. Use the terms **control window**, **talent window**, **voice tracking**, **streaming model** and **resync** consistently.
- Mark statements that you could not verify against the code or a real system as **(unverified)** rather than guessing.
- Changes to commands, events, settings or shortcuts require updating the matching reference page in the same change (see [Contributing](contributing.md)).
