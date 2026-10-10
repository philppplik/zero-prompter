# Third-Party Notices

zero-teleprompter is licensed under `MIT OR Apache-2.0`. This file lists
third-party projects whose ideas, test categories or code influenced this
repository, plus runtime dependencies with notable licences.

> Commit hashes marked `TODO` must be filled in when the referenced revision
> is pinned (the authoring machine had no git/network access to verify them).

## Ideas / algorithms (re-implemented, no code copied)

### EdNutting/autocue
- URL: https://github.com/EdNutting/autocue
- Licence: MIT
- Copyright: see upstream LICENSE (TODO: copy exact copyright line)
- Commit: TODO
- Used for: approach to backtrack and forward-jump detection in word tracking
  (small backward steps accepted immediately, large jumps require
  confirmation) and the categories of scenario tests (ad-lib, backtrack,
  forward jump).
- Where: `crates/zt-align/src/tracker.rs`, `crates/zt-testkit/src/lib.rs`,
  `crates/zt-testkit/src/fixtures.rs`, `crates/zt-testkit/tests/tracker_scenarios.rs`
  (attribution comments in all of them; the scenario tests moved there from
  `crates/zt-align/tests/`). Our implementation (Smith-Waterman window,
  hysteresis counters, state machine), fixture texts and test code are original.

### f/textream
- URL: https://github.com/f/textream
- Licence: MIT
- Copyright: see upstream LICENSE (TODO: copy exact copyright line)
- Commit: TODO
- Used for: UX reference for word highlighting and voice-tracking modes
  (concept only, no code copied). Where: the current-word highlight in
  `apps/desktop/src/components/Prompter.svelte` (attribution comment there);
  our implementation (layout map, single moving highlight layer) is original.

### ArunNGun/openTeleprompt (OpenTeleprompter V3)
- URL: https://github.com/ArunNGun/openTeleprompt
- Licence: MIT
- Copyright: see upstream LICENSE (TODO: copy exact copyright line)
- Commit: TODO
- Used for: planned overlay/pill window and content protection in the Tauri
  app layer (v0.2). Not used in v0.1 (neither crates nor `apps/desktop`).

## Runtime dependencies with licence notes

| Crate / component | Licence | Notes |
|---|---|---|
| sherpa-onnx (k2-fsa), pinned `=1.13.8` (+ `sherpa-onnx-sys =1.13.8`) | Apache-2.0 | optional, feature `sherpa` in `zt-stt`; default build statically links a prebuilt archive downloaded by the build script unless `SHERPA_ONNX_LIB_DIR` is set |
| onnxruntime (via sherpa-onnx) | MIT | shipped with the app when `sherpa` is enabled |
| cpal 0.15 | Apache-2.0 | optional, feature `cpal` in `zt-audio` (intentionally not 0.18, API-incompatible) |
| serde, serde_json | MIT OR Apache-2.0 | |
| thiserror | MIT OR Apache-2.0 | |
| sha2 (RustCrypto) | MIT OR Apache-2.0 | |
| tauri, tauri-build, tauri-plugin-global-shortcut, tauri-plugin-dialog | MIT OR Apache-2.0 | desktop app shell |
| ureq | MIT OR Apache-2.0 | model downloads (feature `download`); pulls rustls/ring/webpki-roots (see `deny.toml`) |
| WebView deps (cssparser/selectors via tauri-utils) | MPL-2.0 | unmodified, file-level copyleft only |
| proptest | MIT OR Apache-2.0 | dev-dependency only (property tests), never shipped |

The test-only crate `crates/zt-testkit` (`publish = false`) contains no
third-party code; it is not a dependency of the shipped app.

### Frontend (npm, `apps/desktop`)

| Package | Licence | Notes |
|---|---|---|
| svelte, @sveltejs/vite-plugin-svelte, vite, vitest, typescript | MIT / Apache-2.0 | build/test tooling |
| @tauri-apps/api, @tauri-apps/plugin-dialog, @tauri-apps/cli | MIT OR Apache-2.0 | |
| prettier, prettier-plugin-svelte | MIT | dev-only formatting, never shipped (licence from memory, not re-checked) |
| @fontsource/atkinson-hyperlegible (font: Braille Institute) | OFL-1.1 (font), MIT (package) | bundled prompter font |
| @fontsource-variable/inter (font: Rasmus Andersson) | OFL-1.1 (font), MIT (package) | UI font |
| @fontsource-variable/jetbrains-mono (font: JetBrains) | OFL-1.1 (font), MIT (package) | numbers/timecodes |

Icons in `apps/desktop/src/components/Icon.svelte` and `app-icon.png` are
original drawings (no icon library copied).

Model licences (Kroko DE/EN CC-BY-SA per model page, unresolved; Zipformer
EN 20M Apache-2.0; Zipformer EN 2023-06-26 unknown; Parakeet TDT 0.6B v3
CC-BY-4.0) are tracked separately in `THIRD_PARTY_MODELS.md`,
`docs/research/MODELS.md` and in the `license`/`attribution` fields of the
model manifest (`zt-models`). Legal assessment: `docs/legal/LEGAL.md`
(Kroko-DE must not ship until its licence is clarified).

### Documentation tooling (not shipped)

| Tool | Licence | Notes |
|---|---|---|
| mdBook (rust-lang/mdBook) | MPL-2.0 | builds `docs/book`; not a dependency of any crate or of the app |

## Textbook algorithms (no third-party code)

Levenshtein distance, Jaro-Winkler similarity, Smith-Waterman local
alignment and the Blackman-windowed-sinc low-pass filter are implemented
from their standard published definitions.
