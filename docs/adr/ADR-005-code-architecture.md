# ADR-005: Code architecture – hexagonal (ports and adapters) with a pure core

> German original: [de/ADR-005-code-architecture.md](de/ADR-005-code-architecture.md)

- **Status:** Accepted
- **Date:** 2026-10-10
- **Context documents:** `docs/research/RESEARCH.md`, `docs/concept/CONCEPT.md` (§6 state/state machines, §7 NFRs), `docs/design/tokens.json`

## Context

zero-teleprompter combines real-time audio, local speech recognition (sherpa-onnx), word alignment, scroll/playback logic and a Tauri/Svelte user interface with two windows (control window and talent window) and optional remote control. The critical requirements:

- Voice latency p95 <= 350 ms, alignment < 5 ms per hypothesis.
- "Do not guess": misbehavior (text running away) is worse than standing still. Behavior in edge cases (ad-lib, jump, silence, jump back) must be **reproducibly testable**.
- Multiple input sources (UI, hotkeys, remote, MIDI, voice) must not produce inconsistent state.
- Three platforms; audio and ONNX libraries are the main source of build and runtime problems.
- The team (and CI) must be able to test the core logic without a microphone, models or a GUI.

## Decision

We structure the Rust part as a **hexagonal architecture (ports and adapters)** in a Cargo workspace:

```
                ┌──────────────────── Adapters (I/O) ────────────────────┐
  Microphone ─▶ zt-audio::CpalSource ──┐                                 │
                                        ▼  Port: AudioSource             │
                zt-stt::SherpaEngine ◀── Port: SpeechEngine ──▶ MockEngine
                        │ SttEvent                                       │
                        ▼                                                │
           ┌────────── pure core (no I/O, no clock) ───────────────┐     │
           │ zt-align: normalize → Smith-Waterman → VoiceTracker   │     │
           │            │ TrackerOutput → (app maps) Command::Voice │     │
           │ zt-core:   Controller.apply(Command) → AppState+Patch │     │
           └───────────────────────────────────────────────────────┘     │
                        │ StatePatch                                     │
                        ▼                                                │
                Tauri adapter (app crate, later) → state://patch → Svelte
  Network ──▶   zt-models::Downloader (port) ← HTTP adapter in the app   │
                └────────────────────────────────────────────────────────┘
```

| Crate | Role | Dependencies |
|---|---|---|
| `zt-core` | Domain core: `AppState`, `Command`, `Controller`, playback state machine, script tokenization, settings/themes from `tokens.json` | serde, serde_json, thiserror |
| `zt-align` | Pure algorithm: DE/EN normalization, fuzzy similarity, windowed Smith-Waterman, hysteresis, voice state machine | **none** (serde optional) |
| `zt-audio` | Port `AudioSource`, adapter `CpalSource` (feature `cpal`), `MockSource`, resampler to 16 kHz mono | thiserror, cpal (optional) |
| `zt-stt` | Port `SpeechEngine`, adapter `SherpaEngine` (feature `sherpa`), `MockEngine`, hotword selection | thiserror, sherpa-onnx (optional) |
| `zt-models` | Manifest, SHA-256 verification, install layout, port `Downloader` | serde, sha2, thiserror |

Rules:

1. **Dependencies point inward.** `zt-core` and `zt-align` know nothing about Tauri, audio, files, threads or clocks. `zt-core` does not even depend on `zt-align`; the app translates `TrackerOutput` into `Command::Voice(VoiceSignal)`.
2. **Time is injected.** `Command::Tick(dt_ms)` and `VoiceTracker::handle(event, t_ms)` respectively. This makes scenarios such as "4 s without a match -> lost" testable in microseconds and deterministic.
3. **A single write path.** All state changes go through the `Command` enum and `Controller::apply` (CONCEPT §6.1). The controller returns a `StatePatch` (which subtrees changed); the adapter sends only those to the windows.
4. **Unsafe third-party APIs sit behind our own traits.** cpal and sherpa-onnx are visible only in one adapter module each, behind feature flags, and are marked with `// VERIFY` at the relevant spots. An API break in a dependency affects exactly one file.
5. **Mocks are first-class adapters.** `MockSource` and `MockEngine` allow end-to-end runs (audio -> STT -> align -> core) in CI without a microphone and models, as well as a demo mode.
6. **`unsafe_code = "forbid"`** workspace-wide. FFI unsafety stays in the upstream bindings.

## Why no framework for the core

Among others, an actor framework (actix/ractor), an ECS (bevy_ecs), Redux-style store crates and an async runtime (tokio) as the backbone of the core were considered.

- **Problem size:** The core is a reducer (`State x Command -> State`) plus two small state machines. That can be expressed clearly in a few hundred lines; a framework brings more concepts than it solves problems.
- **Determinism and testability:** Actor and async frameworks introduce scheduling nondeterminism and hidden clocks. The core must be synchronous and reproducible (fixture tests, fuzzing, criterion benchmarks).
- **Latency budget:** Alignment runs per hypothesis in < 5 ms on the STT thread. Message-passing overhead and framework allocations are undesirable there; pure functions on slices are fastest and easiest to benchmark.
- **Licence and supply chain:** Every dependency must pass `cargo-deny` (MIT/Apache-compatible). `zt-align` has zero runtime dependencies.
- **Replaceability:** Tauri is our shell decision, but not part of the core. A later headless mode (remote-only), a CLI for fixture runs, or a different UI stack use the same core unchanged.
- **Concurrency belongs at the edge:** Threads (audio callback, STT thread, UI event loop) and channels live in the app layer. Tauri's own tokio runtime is present there anyway; the core does not need it.

## Consequences

**Positive**
- Edge cases of voice tracking are covered by fast unit and scenario tests (`crates/zt-testkit/tests/tracker_scenarios.rs`, fixtures in `crates/zt-testkit/src/fixtures.rs`; property tests in `crates/zt-align/tests/prop_*.rs`).
- CI builds and tests the core without ALSA, onnxruntime or models (`--no-default-features`); the adapters run in their own jobs.
- API changes in cpal or sherpa-onnx are locally contained.

**Negative / risks**
- Some boilerplate: mirror enums (`VoicePhase` in `zt-core` <-> `VoiceState` in `zt-align`) and a translation step in the app.
- Own implementations instead of crates (Levenshtein/Jaro-Winkler instead of `strsim`, FIR + interpolation instead of `rubato`). They are small and tested; switching to the crates is a local change if benchmarks or WER measurements justify it.
- The sherpa-onnx and cpal adapters were written without a toolchain and are marked with `// VERIFY`. Before the first release they must be compiled against the published crate versions and checked with fixture WAVs.

## Alternatives

| Alternative | Rejected because |
|---|---|
| Logic in the Svelte frontend | Two windows plus remote need one source of truth; worse latency and testability |
| One large app crate | Core cannot be built without audio/Tauri; longer build times; coupling |
| Actor framework in the core | Nondeterminism, overhead, more concepts than necessary |
| async/tokio in the core | No I/O in the core; async would only introduce complexity and clocks |
