# Testing

> This page describes the tests that exist in the repository and how CI runs them. It does not report results. The documentation author did not run the suite; see [Project status](../status.md) for what is and is not verified.

## Strategy

The pure crates (`zt-core`, `zt-align`, `zt-models`, parts of `zt-audio` and `zt-stt`) take all I/O and time as parameters, so almost everything is testable with ordinary unit tests, table-driven tests, property tests and simulated clocks. Adapters that need real hardware or models (microphone, sherpa-onnx, Windows speech) are covered by build checks plus an end-to-end smoke job in CI, not by unit tests.

| Layer | Technique | Needs |
|---|---|---|
| Unit | `#[cfg(test)]` modules next to the code | nothing |
| Property | `proptest` (version `1`, `default-features = false`, feature `std`) | nothing |
| Table-driven | State machines and alignment scenarios | nothing |
| Pipeline integration | `zt-testkit` mock engine to tracker to controller on simulated time | nothing |
| STT smoke | Real sherpa-onnx, real models, real audio (`scripts/stt-smoke.py`, `examples/smoke.rs`) | network, CI only |
| Windows speech probe | `examples/winspeech_probe.rs` | Windows |
| Frontend | Vitest | Node.js 22 |
| Manual acceptance | Checklist below | a real machine |

## Rust tests

| Location | Kind | Content |
|---|---|---|
| `crates/*/src/**` (`#[cfg(test)]`) | unit | Script tokenization, settings clamping, embedded tokens, marker clamping, latency percentiles, controller scroll modes and countdown, Smith-Waterman, similarity, normalization and number words, the tracker resync and near-cursor tests, AGC and soft limiter, onset detector, resampler, channel selection, hotword selection, fallback chain, mock engines, RMS segmenter, `model_check`, manifest validation and SHA-256 handling (`zt-models/src/tests.rs`), and in the app `persist::valid_id`, `current_layout_lines` and the watchdog types in `voice.rs` |
| `crates/zt-align/tests/prop_normalize.rs` | proptest | Normalization never panics, output words are clean, idempotence, case insensitivity, filler-free and stable hypotheses, number expansion for German and English, long digit strings spelled digit by digit, thousands separators, punctuation ignored, Levenshtein is a metric, similarities bounded, and a table of slip-of-the-tongue similarities |
| `crates/zt-align/tests/prop_align.rs` | proptest | An exact slice is found, hits respect window and bounds, insertions are tolerated, and tracker outputs are well formed for arbitrary event sequences |
| `crates/zt-core/tests/playback_table.rs` | table-driven | Playback x mode state machine |
| `crates/zt-core/tests/script_props.rs` | proptest | Script model (byte ranges, paragraphs) |
| `crates/zt-testkit/tests/tracker_scenarios.rs` | scenarios | Runs `fixtures::SCENARIOS` |
| `crates/zt-testkit/tests/pipeline.rs` | integration | `Pipeline` with mock engines over simulated time |
| `crates/zt-testkit/tests/controller.rs` | integration | Controller behavior (scroll modes, lost policy, override, countdown) |
| `crates/zt-testkit/tests/hybrid_resync.rs` | integration | `HybridEngine` with a mock offline engine and the tracker |

Run everything that CI runs for the library crates:

```bash
cargo test --all-targets
cargo test --doc
```

Property tests run with the default case count on every push. The weekly job `proptest-extended` (and manual dispatch) runs `PROPTEST_CASES=8192` in release mode for `zt-align`, `zt-core` and `zt-testkit`, and uploads failing seeds from `proptest-regressions/` as an artifact. The workspace `Cargo.toml` carries a `// VERIFY` note: if a new proptest 1.x release raises its MSRV above 1.80, pin the version (for example `=1.5.0`).

### Scenario tests (`zt-testkit::fixtures`)

`SCRIPT_DE` is a 60-word German reference script (four sentences) in which every whitespace token normalizes to exactly one word, so token id equals normalized index equals word position. A `Scenario` is a name, a `Category` and a list of `Step`s:

- `Step::Say(text)`: growing partials 250 ms apart plus a final.
- `Step::Silence(ms)`: VAD silence, then time passes.
- `Step::Tick(ms)`, `Step::Seek(token)`.
- `Step::Expect(state, cursor)`: asserts the tracker state and cursor at that point.

Categories: `Normal`, `Versprecher` (slips of the tongue, STT typos), `Zurueckspringen` (backtracking), `Ueberspringen` (skipping), `Stille` (silence), `AdLib`, `Manual`, `Lost`. The category names are German identifiers in the source. The table has 13 scenarios in this revision, for example "small backtrack accepted immediately", "far forward jump is confirmed, then eased", "very long silence is never `lost`", and "unrelated speech -> lost, global search recovers". Categories follow EdNutting/autocue (MIT); texts and code are the project's own.

The expected values were derived by hand from the algorithm. When a parameter change breaks a scenario, re-derive the expected value rather than editing it to pass (see [Aligner algorithm](aligner.md)).

### testkit helpers

The examples below illustrate the API; they were not compiled for this page. Signatures are in `crates/zt-testkit/src/`.

```rust
use zt_testkit::{Pipeline, Speaker, SpeechTimeline};

// Tracker over SCRIPT_DE with an explicit clock, already started (state Listening).
let mut s = Speaker::default_de();
s.say("Willkommen zu meinem Kanal");        // growing partials, then a final
assert_eq!(s.state(), zt_align::VoiceState::Tracking);

// Mock engine from a readable timeline, run through the whole pipeline in 50 ms steps.
let engine = SpeechTimeline::new(2.0).say("Willkommen zu meinem Kanal").pause(1.5).build();
let mut p = Pipeline::new(zt_testkit::SCRIPT_DE, engine);
p.run_secs(5.0);
```

`Pipeline::new` builds a controller with the script loaded, voice enabled and playback running, starts the tracker, and applies hotwords from the script like the app does. Each `step()` feeds 50 ms of silence samples (the mock engine only counts them), forwards engine events to the tracker, forwards a changed tracker output to the controller as `Command::Voice`, sends a tracker tick, and applies `Command::Tick(50)`.

## STT smoke test (CI job `stt-smoke`)

Matrix: ubuntu-22.04, windows-latest, windows-11-arm. The job builds `examples/smoke.rs` in release mode with `--features sherpa`, then runs `scripts/stt-smoke.py stt-cache <model ids...>`:

- Reads the model list from `apps/desktop/src-tauri/resources/models.json`, downloads the files with SHA-256 verification into a cached directory (`actions/cache`, keyed by the hash of `models.json` and the script).
- Downloads pinned test WAVs (hash-checked): a German clip (`de-kroko-0`) and English clips.
- Runs the smoke example per model and case. The example creates the recognizer (the fallback ladder is logged), optionally applies hotwords, streams the WAV in 100 ms chunks, prints text, RTF, per-chunk latency, partial cadence, and the time from speech onset (`OnsetDetector`) to the first partial, and fails if an expected keyword is missing. `--legacy-check` reproduces the 0.x bug: the old configuration (`modeling_unit = "bpe"` without `bpe_vocab`) must make `OnlineRecognizer::create` return `None`.
- Models in the CI invocation: Nemotron 3.5 160 ms and 320 ms, Kroko DE, Kroko EN, Zipformer EN 20M. Nemotron cases run without hotwords (greedy search only).
- Optionally samples peak RSS with `psutil` and prints a comparison table to the log and the step summary.

The measured values recorded in `docs/research/MODELS.md` come from this job; see the latency section of [Data flow](dataflow.md).

## Windows speech probe (CI job `winspeech`)

On windows-latest and windows-11-arm: clippy and tests of `zt-stt` with `--features winspeech`, and `examples/winspeech_probe` for `de` and `en-US`. The probe lists languages, creates the recognizer and runs `CompileConstraintsAsync` without a microphone. If the runner lacks the language pack or consent, the probe reports a skip or failure as information; it turns the job red only if a grammar language is installed and the offline list constraint still fails to compile ([ADR-008](../adr/index.md)). Recognition with real speech is not tested in CI.

## Frontend (Vitest)

`npm test` runs `vitest run` over `apps/desktop/src/**/*.test.ts`. Current test files in `src/lib/`: `about`, `diagnostics`, `example`, `fonts`, `hotkeys`, `marker`, `motion`, `palette`, `scroller`, `tokens` and `wheel`. Shared helpers are in `src/test/helpers.ts`. Type checking: `npm run check` (`svelte-check`). Formatting: `npm run format:check` (Prettier).

## Manual acceptance (before each release)

1. Play the example script at a constant 140 WPM. The state must reach `ended` at the end.
2. Enable voice in demo mode (no model). The status indicator must show `listening`, then `tracking`.
3. With a real model: ad-lib, go back to a previous sentence, skip a paragraph, stay silent for 5 s. Expect `hold`, `resync` and `lost` as described in [Voice tracking](../user/voice-tracking.md).
4. Open the talent window on a second monitor, in fullscreen, with horizontal and vertical mirroring.
5. Try the global hotkeys (`Ctrl+Shift+F6` to `F11`) while another application has focus.
6. Check the diagnostics tab: engine and model, audio format, frames per second, tick rate near 60, latency percentiles, and log export.
7. On Windows: confirm the installer does not report a missing `VCRUNTIME140.dll` on a clean machine.
