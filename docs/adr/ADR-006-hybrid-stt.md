# ADR-006: Hybrid speech recognition – streaming zipformer + optional offline resync

> German original: [de/ADR-006-hybrid-stt.md](de/ADR-006-hybrid-stt.md)

- **Status:** Accepted
- **Date:** 2026-10-10
- **Context documents:** `docs/research/MODELS.md`, ADR-005, `crates/zt-stt`, `crates/zt-align/src/tracker.rs`

> **Editorial note (English edition):** the default model named in decision 1 (Kroko) was superseded on 2026-10-11 by Nemotron 3.5 Streaming, see the addendum in [ADR-007](ADR-007-voice-pipeline-tuning.md). The text below is kept as originally decided.

## Context

Voice tracking needs a hypothesis with p95 latency <= 350 ms. Only streaming
transducers (sherpa-onnx `OnlineRecognizer`) can deliver that, but their word
error rate on free speech is noticeably higher. Large offline models
(Parakeet TDT 0.6B v3) are considerably more accurate but only produce a
result after the end of a segment and cost about 670 MB plus several hundred
milliseconds of CPU per segment. After a slip of the tongue, an ad-lib or a
jump, the tracker should nevertheless find its way back to the right place
reliably.

## Decision

1. **The streaming model drives the cursor.** One streaming model per primary
   language: default DE `zipformer-de-kroko-2025-08-06`, default EN
   `zipformer-en-kroko-2025-08-06`, fallback EN `zipformer-en-20m-2023-02-17`
   (Apache-2.0). Order: user choice (if installed) -> `defaultRank` in the
   manifest. If construction fails, the next installed candidate is tried
   (`zt_stt::build_with_fallback`); with no model, demo mode.
2. **Resync is optional (tier L, off by default).** `HybridEngine` cuts the
   audio stream into speech segments (Silero VAD, otherwise RMS), hands them
   via `try_send` to a bounded queue and decodes them on the `zt-resync`
   thread with an `OfflineRecognizer` (`model_config.transducer`,
   `model_type = "nemo_transducer"`). If the queue is full the segment is
   dropped; the streaming path never blocks.
3. **Re-anchor instead of replace.** Results arrive as `SttEvent::Resync { text,
   age_ms }` and go to `VoiceTracker::resync`, which confirms, ignores or
   re-anchors the cursor (`ResyncOutcome`). A manual jump (`reset`) increments
   a generation counter; older results are discarded.
4. **Manifest v2:** `modelType`, `roles`, `defaultRank`, optional `sha256`.
   Files without a hash (e.g. `tokens.txt`, not in LFS) are only size-checked;
   their SHA-256 is computed after download and shown as "unverified"; the
   manifest stays loadable.
5. **Ports:** `SpeechEngine` (streaming), `OfflineEngine`, `Segmenter`, each
   with a mock (`MockEngine`, `MockOfflineEngine`, `RmsSegmenter`) for CI
   without models; the sherpa adapter sits behind the `sherpa` feature.

## Consequences

- + Low latency is preserved, and the accuracy of the re-anchor improves.
- + Fully testable with mocks (`zt-stt` unit tests, `zt-testkit/tests/hybrid_resync.rs`).
- - Additional CPU load and a ~670 MB download when resync is enabled.
- - The sherpa adapters are written against docs.rs 1.13.8; CI only checks the
  feature build (non-blocking), and runtime tests with real models are still missing.
- - Pin the SHA-256 of `tokens.txt` before a release.
