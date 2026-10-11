# ADR-007: Voice pipeline – error handling, diagnostics and latency tuning

> German original: [de/ADR-007-voice-pipeline-tuning.md](de/ADR-007-voice-pipeline-tuning.md)

- Status: accepted
- Date: 2026-10-11
- Related: ADR-002 (cpal), ADR-006 (hybrid STT), user feedback on v0.1.0 (Windows 11 ARM64, `--features sherpa`): "syncing speech does not work, the UI says Demo"

## Context

### Finding: "Demo" / sync does not work

The release builds contain sherpa (CI log Windows ARM64: `tauri build --features sherpa`, `zt-stt` built with the feature; the feature is passed through correctly via `apps/desktop/src-tauri/Cargo.toml` -> `zt-stt/sherpa`). The bug is in the engine selection in `voice.rs`/`models.rs` of v0.1.0:

1. **Silent demo fallback on load errors.** `make_engine` turned *every* `Err` from `build_engine` into demo mode (`MockEngine::reading`). The demo engine reads the script aloud by itself at 2.3 words/s. If an installed model could not be loaded, the text therefore scrolled independently of the voice, which for users is exactly "sync does not work" plus the badge "Demo". The error appeared only as a short toast and was logged nowhere (`eprintln!` is invisible in a Windows GUI app).
2. **Language binding without fallback.** Only models of the script language (`settings.language`, default `de`) were searched. Anyone who had only Zipformer EN 20M (Apache-2.0) or only Parakeet (role `resync`, not streaming) installed got `Ok(None)` -> demo, although a model was on disk.
3. **Stale engine after a model download.** The engine is built when voice or the microphone test starts. In onboarding the microphone test runs *before* the model download; afterwards `store.engine.demo = true` stayed set until voice was restarted.
4. **No diagnostics.** No log, no latency or level values, no detection of "microphone opened but delivers nothing" (the Windows privacy setting "Let desktop apps access your microphone" yields silence instead of an error), and panics in the voice thread vanished with the thread.

Checked and found **not** to be the cause: feature propagation in the workspace; model paths (file names come from the manifest roles `encoder/decoder/joiner/tokens`); sample rate (cpal -> own resampler -> 16 kHz mono, sherpa `feat_config` 16 kHz/80 fbank per the crate default); Silero (only used for resync segmentation, RMS fallback); hotwords (none of the shipped models has `bpe.vocab` -> `set_hotwords` returns `Unsupported`, the recognizer stays unchanged -- expected, now logged at info level).

## Decision

### Engine selection without silent fallbacks

| Situation | Engine | UI |
|---|---|---|
| Streaming model for the script language installed | HybridEngine (as in ADR-006) | Model name |
| Only a streaming model of another language installed | that model (`Manifest::streaming_plan`) | Notice "No model for language ... - using ..." |
| No streaming model installed | Demo (`reason = noModel`) | Badge "Demo · load model" -> Settings/Voice |
| Build without `sherpa` | Demo (`reason = noSherpa`) | Badge + notice |
| Model installed, loading fails | `NullEngine` (`reason = engineError`), **not** demo | Error toast, badge "Model error · details" -> Diagnostics |

After a successful model download, a running voice session is restarted.

### Error path and logging

- `tracing` + `tracing-subscriber` (EnvFilter, variable `ZT_LOG`) + `tracing-appender` (daily rotation, 7 files) in the app log directory (`app_log_dir`, on Windows `%LOCALAPPDATA%\<identifier>\logs`).
- A single error path: `diag::report_error` = log + ring buffer (50) + event `diag://error` -> toast. A panic hook logs panics from all threads; the voice thread runs under `catch_unwind` and reports crashes.
- Structured events (targets): `zt::voice` start/stop, `zt::engine` plan/selection/load time/fallback reasons, `zt::audio` device + format, frames/s, RMS, `zt::vad` segment boundaries, `zt::stt` partial (debug, with timestamp and latency) / final (info), `zt::tracker` state changes, confidence, cursor, `zt::resync`, `zt::models` download/hash, `zt::frontend` unhandled JS errors.
- Microphone watchdog: 3 s without a callback or only exact zeros -> error with a hint about the privacy settings.
- Diagnostics tab (`DiagnosticsPanel.svelte`): level, engine/model/chain, audio format, frames/s, tick rate, tracker state, latencies p50/p95, last errors, "Export log" (save dialog, logs + JSON snapshot), microphone test.

### Latency

Measured in the voice thread (rolling, last 200 values, `zt_core::metrics::LatencyWindow`, nearest-rank):

| Metric | Start | End |
|---|---|---|
| `onsetToPartial` | Speech onset relative to the noise floor (`OnsetDetector`, see addendum 2026-10-11; previously a fixed -45 dBFS) | first partial hypothesis |
| `audioToPartial` | Capture time of the chunk that triggered a new partial | partial emitted |
| `hypToCursor` | Hypothesis handed to the tracker | cursor change |
| `endToEnd` | Capture time of the chunk | `voice://word` emitted |

Latency chain (target: word recognized -> scroll < 300 ms p50, hard limit < 1 s). **Estimated** = derived from configuration/literature, not yet measured on target hardware; from this version on, the Diagnostics tab provides the measured values.

| Stage | Value | Kind |
|---|---|---|
| Audio callback (WASAPI shared, cpal default buffer) | ~10 ms period | estimated (Windows default) |
| Channel -> voice thread | blocking `recv`, no polling interval (`recv_timeout` of 50 ms only as an abort check) | code |
| Resampling to 16 kHz | < 1 ms | estimated |
| `accept_waveform` + decode | per callback chunk, `decode` while `is_ready` | code |
| Zipformer chunk (model lookahead) | Partials arrive per encoder chunk (typically 320 ms for a streaming zipformer) -> on average ~160 ms of waiting plus compute time | estimated |
| Compute time per chunk (ARM64, 2 threads, int8/fp32) | 20-60 ms | estimated |
| Tracker (Smith-Waterman k=6 x w=70) | < 1 ms | estimated, measured by `hypToCursor` |
| 60 Hz tick (`sleep(16 ms)`, Rust std on Windows with high-resolution timer) | <= 16 ms | code, measured as "Tick Hz" |
| Scroll ease (previously tau = 300 ms) | **tau = 150 ms**: 63 % of a word step visible after 150 ms, movement begins on the next tick | code + test |
| Endpoint rules (2.4 s / 0.8 s / 20 s) | only affect `Final`; the tracker follows the partials | code |

Expected word -> visible scroll start ~ 10 + ~160 + ~40 + 1 + <= 16 ms ~ **230 ms p50** (estimated); p95 ~ 450 ms due to chunk boundaries and CPU, which is within the hard limit. To be confirmed through `endToEnd` in the Diagnostics tab (the scroll animation is added from the tick onward).

Other tuned defaults:
- Tracker near path (`TrackerConfig::near_words = 2`, see below) for the first word after a seek/re-anchor.
- Hotwords stay off for models without `bpe.vocab`; the recognizer is then not recreated (no double load time).

## Comparison with ShayneP/local-teleprompter

Source: https://github.com/ShayneP/local-teleprompter (licence **MIT**, checked in the README/repository). Read: `frontend/lib/teleprompter/position-tracker.ts`, README.

| Aspect | local-teleprompter | zero-teleprompter | Adopted |
|---|---|---|---|
| STT | NVIDIA Nemotron Streaming 0.6B (FastConformer-RNNT, cache-aware) as a Python/FastAPI server, LiveKit, GPU/MPS preferred; README mentions ~80 ms to first words on Apple Silicon/GPU | sherpa-onnx in-process, Zipformer (Kroko) on CPU, no Python/server dependency | no: server + GPU contradicts the offline single binary (ADR-002/006); watch the model as a candidate in MODELS.md |
| Matching | forward, near window cursor..+2 immediately; far window up to +17 only with bigram confirmation; fuzzy only for >= 5 characters, length difference <= 2, Levenshtein <= 1; re-anchor after 4 failures over 6 words, >= 3 hits | windowed Smith-Waterman (k=6, -10/+60), hysteresis (2x confirmation from 18 words), jump back up to 10 words, global search when `lost`, offline resync | **partially**: near path + strict fuzzy rule |
| Partials | processes only new words (`consumedSpokenCount`), no weighting | replaces the partial, weights finals higher | no: SW alignment is more robust against hypothesis revisions |
| Backward | forward only (manual via double-click) | jumps back allowed | no |

Adopted (own implementation, no code copied, attribution in THIRD_PARTY_NOTICES.md): **near path** -- when the SW alignment has too little evidence (`min_matches = 2`), the *newest* word may still move the cursor if it hits one of the next three script words (exactly or via the strict fuzzy rule). Only with an existing anchor (`tracking`/`hold`/`resync`), never in `listening`/`lost`. Rationale for "demonstrably better": after a manual seek, re-anchor or jump the hypothesis buffer is empty; previously the first movement required two matching words (~ 1 word ~ 250-400 ms of additional latency at 2.5-4 words/s). With the near path the cursor moves on the first word (test `near_fast_path_moves_on_first_word_after_seek`). The strict fuzzy rule prevents short words ("und"/"uns") from pulling the cursor (test `near_word_match_rules`); the existing scenario table (among others "one word is not enough in `listening`") stays green and unchanged.

Not adopted because not demonstrably better: the bigram gate for the far window (our 2x jump confirmation serves the same purpose with time hysteresis) and re-anchoring over 6 words (we use a global SW search + offline resync).

## Addendum 2026-10-11: partial cadence, onset metric, Nemotron

**Finding (user log, Win11 ARM64, Kroko DE):** Partials only every ~1.27 s
(t_ms 19180 -> 20430 -> 21747 -> 23020), 1-3 words each; `audioToPartial` 80 ms,
`hypToCursor` 0.3 ms, the scroll stutters at a 1.3 s beat. `onsetToPartial`
p50 2649 ms / p95 46154 ms.

**Cause of the cadence: the model, not the code.** Encoder metadata of the Kroko
models: `decode_chunk_len = 128`, `T = 141` (10 ms frames) -> one decoder step
per 1.28 s of audio. Our loop decodes after every audio block
(`while is_ready { decode }`) and reports every text change; endpoint rules only
affect finals. Cross-check in CI with identical code: Zipformer EN 20M
0.3 s, Nemotron 160 ms 0.2 s, Nemotron 320 ms 0.3 s, Kroko 1.30 s (p50 = p95).
The assumption above ("typically 320 ms for a streaming zipformer") does not
hold for Kroko. No code fix is possible; the remedy is a model with a smaller chunk.

**Cause of the onset metric:** a fixed threshold of -45 dBFS *after* AGC. With a quiet
microphone the AGC lifts the noise above -45 dBFS -> "onset" on noise,
reset only on endpoint/final -> seconds to minutes. New:
`zt_audio::OnsetDetector` -- threshold = noise floor + 12 dB (the floor follows
quickly downward, slowly upward at 3 dB/s, frozen during speech), at least
60 ms loud, re-armed after 0.5 s of silence; an unclaimed onset stays valid for
2.5 s for a late first hypothesis. The same detector measures "time to first word" in CI.

**Model decision:** NVIDIA Nemotron 3.5 ASR Streaming 0.6B int8
(OpenMDW-1.1, sherpa-onnx export `csukuangfj2/...-2026-06-11`) becomes the default for
DE and EN, as the **320 ms variant**; 80/160 ms are selectable, Kroko and 20M remain
selectable and act as a fallback as long as Nemotron is not installed. Implementation:
`ModelType::OnlineNemoTransducer`, `SherpaConfig::nemotron` (transducer fields,
`greedy_search`, 4 threads, `stream.set_option("language", "de"|"en")` after
`create_stream` and after every `reset`, otherwise `"auto"`). sherpa-onnx 1.13.8 only
supports `greedy_search` for streaming NeMo; `modified_beam_search` terminates the
process (`exit(-1)`), hence no beam stage and no hotwords.

Measured (CI run 38132699828, 4-vCPU runner, 100 ms blocks, 500 ms
lead silence; full table in `docs/research/MODELS.md`):

| Model | RTF Arm64 / x86 | Cadence p50 / p95 | First word DE / EN | Load | RSS |
|---|---|---|---|---|---|
| Nemotron 320 ms | 0.31 / 0.59 | 0.3 / 0.4 s | 0.59 / 0.90 s | ~1.5-1.8 s | 0.76-0.97 GB |
| Nemotron 160 ms | 0.53-0.60 / **1.05-1.23** | 0.2 / 0.3-0.5 s | 0.59 / 0.90 s | ~1.5-1.8 s | 0.76-0.97 GB |
| Kroko DE / EN | 0.03-0.04 | **1.3 / 1.3 s** | 0.99 / 1.0-2.3 s | ~2.4-2.8 s | 0.25 GB |
| Zipformer EN 20M | 0.03-0.04 | 0.3 / 0.4 s | - / 1.2 s | ~0.6 s | 0.15 GB |

Why 320 rather than 160 ms: 160 ms is not real-time capable on the x86 runners
(in a second run even RTF 2.4 on windows-latest), whereas 320 ms needs
half the compute everywhere at 0.3 s cadence (Kroko: 1.3 s). So that a model that
is too slow does not silently fall behind, `RealtimeWatch`
(`voice.rs`) measures the decode time per 5 s of audio and warns once if RTF > 0.9 in
two consecutive windows.

## Consequences

- "Demo" only appears when no streaming model is really installed or the build contains no STT, and then always with "Load model".
- A broken model no longer leads to apparent tracking but to a visible error message with a log.
- New dependencies: `tracing`, `tracing-subscriber`, `tracing-appender` (all MIT). `Cargo.lock` regenerated via `lockfiles.yml`.
- Open points: measure latency values on Windows ARM64 target hardware and add them here; verify the encoder chunk size of the Kroko models from the model metadata; evaluate `num_threads` (2) for ARM64; cpal stream errors (`err_fn`) still go through `eprintln!` in `zt-audio` instead of `tracing`.
