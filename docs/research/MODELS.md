# STT models for voice tracking (sherpa-onnx)

Research date: 2026-10-10. Sizes and SHA-256 values come from the Hugging Face tree API
(`/api/models/<repo>/tree/main?recursive=1`, field `lfs.oid`). Nothing has been
downloaded or run locally.

## Comparison

| Id | Type | Languages | Download (MB, roughly) | Licence | WER (source) | Files |
|---|---|---|---|---|---|---|
| zipformer-de-kroko-2025-08-06 | streaming zipformer transducer | de | 71 | CC-BY-SA (Banafo) | not published | encoder.onnx, decoder.onnx, joiner.onnx, tokens.txt |
| zipformer-en-kroko-2025-08-06 | streaming zipformer transducer | en | 71 | CC-BY-SA (Banafo) | not published | encoder.onnx, decoder.onnx, joiner.onnx, tokens.txt |
| zipformer-en-20m-2023-02-17 (int8) | streaming zipformer transducer, 20M params | en | 44 | Apache-2.0 | not on card; LibriSpeech-only | encoder/decoder/joiner-epoch-99-avg-1.int8.onnx, tokens.txt |
| zipformer-en-2023-06-26 (int8, chunk16-left128) | streaming zipformer transducer | en | 73 | unknown | not on card; LibriSpeech-only | encoder/decoder/joiner-epoch-99-avg-1-chunk-16-left-128.int8.onnx, tokens.txt |
| parakeet-tdt-0.6b-v3-int8 | **offline** NeMo TDT transducer (resync) | 25 EU languages incl. de, en | 670 | CC-BY-4.0 | FLEURS en 4.85 / de 5.04; CoVoST en 6.80 / de 4.84; Open ASR avg (en) 6.34 | encoder.int8.onnx, decoder.int8.onnx, joiner.int8.onnx, tokens.txt |

Archive alternatives (GitHub, `asr-models` release):
- https://github.com/k2-fsa/sherpa-onnx/releases/download/asr-models/sherpa-onnx-streaming-zipformer-en-20M-2023-02-17.tar.bz2
- https://github.com/k2-fsa/sherpa-onnx/releases/download/asr-models/sherpa-onnx-streaming-zipformer-en-2023-06-26.tar.bz2

The manifest uses per-file HF URLs instead, because the schema needs a SHA-256
for each file.

## Recommendation

- **Default DE:** `zipformer-de-kroko-2025-08-06`. It is the only verified German *streaming*
  transducer for sherpa-onnx (added in v1.12.10, PR #2502). The download is about 71 MB.
  Caveats: CC-BY-SA, attribution required, and no published WER.
- **Default EN:** `zipformer-en-kroko-2025-08-06`. As a strictly permissive fallback, use `zipformer-en-20m-2023-02-17`
  (Apache-2.0, 44 MB, very fast, but LibriSpeech-only, so expect weaker results on free speech).
- **Resync (optional, tier L):** `parakeet-tdt-0.6b-v3-int8` decodes VAD segments offline and
  re-anchors the position after a slip. It is large (about 670 MB) and needs
  `model_type = "nemo_transducer"` in sherpa-onnx.

### Integration in the app (ADR-006)

| Role | Engine (sherpa-onnx 1.13.8) | Manifest |
|---|---|---|
| Streaming DE / EN | `OnlineRecognizer`, transducer (encoder/decoder/joiner/tokens) | `modelType: online_transducer`, `roles: ["streaming"]`, `defaultRank` 0 = default |
| Fallback EN | same, tried automatically when the preferred model fails to load | `defaultRank: 1` |
| Resync | `OfflineRecognizer`, `model_config.transducer` + `model_type = "nemo_transducer"` | `modelType: offline_nemo_transducer`, `roles: ["resync"]` |

The user picks a streaming model per primary language (or "automatic") and
switches resync on in the Inspector (`voice-models.json`). Resync segments
come from Silero VAD if `<data>/models/silero_vad.onnx` exists, otherwise from
the model-free RMS segmenter.

Teleprompter tracking matches against a known script, so a moderate WER is
acceptable. Latency and robustness matter more.

## Windows speech recognition (system fallback, ADR-008)

Not a downloadable model: `Windows.Media.SpeechRecognition` (WinRT), id `windows-speech`, Windows builds with feature `winspeech` only (`windows` crate `=0.61.3`).

| Aspect | Value |
|---|---|
| Dictation (`SpeechRecognitionTopicConstraint`, scenario Dictation) | cloud service; needs language pack + privacy setting "Online speech recognition" (HRESULT `0x80045509` otherwise) + network |
| Offline (`SpeechRecognitionListConstraint`) | script words only, needs the grammar language installed; coarse |
| Partials / finals | `HypothesisGenerated` / `ContinuousRecognitionSession.ResultGenerated` |
| Audio | captured by Windows itself (default device), not by cpal |
| Biasing | none for dictation (topic hint only); list mode = script words |
| Platforms | Windows 10/11 x64 + ARM64; unpackaged Win32 use documented, not yet verified on hardware |
| Licence | OS component, no redistribution |

Role: fallback after Nemotron / Kroko / Zipformer 20M and explicit choice "Windows Spracherkennung (System)". Accuracy and latency were not measured (CI has no microphone or consent). Newer Windows AI APIs (Windows App SDK) offer no general streaming ASR for unpackaged apps as of 2026-10. SAPI 5 desktop recognition is deprecated in Windows 11.

## Open points (not verified)- SHA-256 of all `tokens.txt` files (not LFS). Since manifest schema v2 the hash is
  optional: such files are size-checked, their SHA-256 is computed after download and
  stored as `unverified-sha256.json` in the model folder, and the UI marks them as
  unverified. Pin the reported hashes before a release.
- The exact CC-BY-SA version for Kroko, and whether the share-alike terms are acceptable.
- The licence of `zipformer-en-2023-06-26`.
- Measured latency/RTF on the target CPU.

## Sources
- https://huggingface.co/csukuangfj/sherpa-onnx-streaming-zipformer-de-kroko-2025-08-06
- https://huggingface.co/csukuangfj/sherpa-onnx-streaming-zipformer-en-kroko-2025-08-06
- https://huggingface.co/Banafo/Kroko-ASR
- https://huggingface.co/csukuangfj/sherpa-onnx-streaming-zipformer-en-20M-2023-02-17
- https://huggingface.co/csukuangfj/sherpa-onnx-streaming-zipformer-en-2023-06-26
- https://huggingface.co/csukuangfj/sherpa-onnx-nemo-parakeet-tdt-0.6b-v3-int8
- https://huggingface.co/nvidia/parakeet-tdt-0.6b-v3
- https://k2-fsa.github.io/sherpa/onnx/pretrained_models/online-transducer/zipformer-transducer-models.html
