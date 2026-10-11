# STT models for voice tracking (sherpa-onnx)

Research date: 2026-10-10. Sizes and SHA-256 values come from the Hugging Face tree API
(`/api/models/<repo>/tree/main?recursive=1`, field `lfs.oid`). Nothing has been
downloaded or run locally.

## Measured comparison (CI `stt-smoke`, 2026-10-11)

Source: GitHub Actions run 38132699828 (`scripts/stt-smoke.py`, sherpa-onnx
1.13.8, real models, 100 ms chunks, 500 ms lead silence, 2 s tail). Runners:
`ubuntu-22.04` / `windows-latest` (x86-64, 4 vCPU) and `windows-11-arm`
(Arm64, 4 vCPU). Test audio: `de-kroko-0` ("Alles hat ein Ende, nur die Wurst
hat zwei", 2.75 s) and `en-librispeech-0` (6.6 s). Threads: Nemotron 4,
Zipformers 2.

- **RTF** = decode time / audio time (incl. tail; > 1 = slower than real time).
- **Cadence** = audio time between two text changes (partials) = how often
  the prompter cursor can move. **First word** = speech onset in the WAV
  (noise-floor detector) to the first partial.

| Model | Platform | RTF DE / EN | chunk p95 ms | cadence p50 / p95 s | first word s (DE / EN) | load ms | peak RSS MB | text |
|---|---|---|---|---|---|---|---|---|
| Nemotron 3.5 **320 ms** int8 | win-arm64 | 0.31 / 0.32 | 111–119 | 0.30 / 0.40 | 0.59 / 0.90 | ~1500 | 774 | DE exact; EN "broffels" |
| Nemotron 3.5 320 ms int8 | linux-x64 | 0.59 / 0.59 | 214 | 0.30 / 0.40 | 0.59 / 0.90 | ~1800 | 966 | as above |
| Nemotron 3.5 320 ms int8 | win-x64 | 0.59 / 0.59 | 212–214 | 0.30 / 0.40 | 0.59 / 0.90 | ~1700 | 758 | as above |
| Nemotron 3.5 160 ms int8 | win-arm64 | 0.53 / 0.60 | 107–126 | 0.20 / 0.30–0.40 | 0.59 / 0.90 | ~1500 | 773 | DE + EN exact (cased, punctuated) |
| Nemotron 3.5 160 ms int8 | linux-x64 | 1.23 / 1.13 | 214–281 | 0.20 / 0.30–0.50 | 0.59 / 0.90 | ~1800 | 966 | as above |
| Nemotron 3.5 160 ms int8 | win-x64 | 1.05 / 1.11 | 204–222 | 0.20 / 0.30–0.40 | 0.59 / 0.90 | ~1600 | 760 | as above |
| Kroko DE | win-arm64 | 0.030 / – | 0.2 | **1.30 / 1.30** | 0.99 / – | ~2800 | 249 | exact |
| Kroko DE | linux-x64 | 0.029 / – | 0.3 | 1.30 / 1.30 | 0.99 / – | ~2400 | 255 | exact |
| Kroko EN | win-arm64 | – / 0.044 | 51 | 1.30 / 1.30 | – / 1.00 | ~2800 | 249 | exact |
| Kroko EN | linux-x64 | – / 0.041 | 50 | 1.30 / 1.30 | – / 2.30 | ~2600 | 257 | exact |
| Zipformer EN 20M int8 | win-arm64 | – / 0.031 | 10 | 0.30 / 0.40 | – / 1.20 | ~620 | 158 | upper case, no punctuation |
| Zipformer EN 20M int8 | linux-x64 | – / 0.038 | 12 | 0.30 / 0.40 | – / 1.20 | ~530 | 150 | as above |

Run-to-run noise on the shared runners is large: the same 160 ms model
measured RTF 0.6–0.7 on linux-x64 and 2.3–2.4 on win-x64 in run 38132359791.
Peak RSS is sampled with psutil every 50 ms (Windows: peak working set).

Findings:
- **Kroko's 1.3 s cadence is the model, not our code.** The Kroko encoders are
  exported with `decode_chunk_len = 128` feature frames (encoder metadata,
  `T = 141` incl. padding) = 1.28 s of audio per decoder step. Our loop
  already decodes on every 100 ms block (`while is_ready { decode }`) and emits
  a partial on every text change; there is nothing to emit in between. The
  20M Zipformer (chunk 32 frames) shows 0.3 s with the same code.
- **Nemotron** gives 0.2 s (160 ms) / 0.3 s (320 ms) cadence, the first word
  ~0.4–0.9 s earlier than Kroko, casing + punctuation, but costs 0.6 GB RAM,
  a 0.66 GB download and ~10–20x the CPU of Kroko.
- 160 ms is **not real time on the 4-vCPU x86 runners** (RTF 1.05–1.23);
  320 ms is (0.59), with half the CPU of 160 ms everywhere. Hence the default is
  **320 ms**; 160 ms / 80 ms are selectable for strong CPUs (Arm64 runner:
  160 ms at RTF 0.53–0.60). The app warns when decoding stays slower than real
  time (`RealtimeWatch` in `voice.rs`, RTF > 0.9 for two 5 s windows).
- Without lead silence (first measurement) Nemotron dropped the first word
  ("Alles") and the 20M model the first two; with 500 ms of silence before
  speech (the normal case with a running microphone) all models got it.
- sherpa-onnx 1.13.8 implements only `greedy_search` for streaming NeMo
  transducers. `modified_beam_search` makes the C++ side `exit(-1)` (CI log:
  `online-recognizer-transducer-nemo-impl.h:59 Unsupported decoding method`),
  so Nemotron runs without hotwords; the ladder never tries beam for it.

## Comparison (desk research)

| Id | Type | Languages | Download (MB, roughly) | Licence | WER (source) | Files |
|---|---|---|---|---|---|---|
| nemotron-3.5-streaming-0.6b-{80,160,320}ms-int8 | streaming NeMo cache-aware FastConformer-RNNT, language prompt | 40 locales incl. de, en (app: de, en) | 682 | OpenMDW-1.1 | see NVIDIA card (FLEURS per chunk size) | encoder.int8.onnx, decoder.int8.onnx, joiner.int8.onnx, tokens.txt |
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

- **Default DE + EN (since 2026-10-11):** `nemotron-3.5-streaming-0.6b-320ms-int8`
  (NVIDIA, OpenMDW-1.1, 682 MB, sherpa-onnx `csukuangfj2/...-320ms-int8-2026-06-11`
  @424ce58). Language prompt per stream (`de`/`en`, else `auto`), greedy only.
  80 ms and 160 ms exports are selectable. See the measured comparison above.
- **Kroko DE / EN** (`zipformer-*-kroko-2025-08-06`) stay as the light
  alternative (71 MB, RTF ~0.03) and as fallback when Nemotron is not
  installed; their 1.28 s decoder chunk limits the cursor to ~1.3 s steps.
  Caveats: CC-BY-SA, attribution required, and no published WER.
- **Permissive EN fallback:** `zipformer-en-20m-2023-02-17`
  (Apache-2.0, 44 MB, very fast, but LibriSpeech-only, so expect weaker results on free speech).
- **Resync (optional, tier L):** `parakeet-tdt-0.6b-v3-int8` decodes VAD segments offline and
  re-anchors the position after a slip. It is large (about 670 MB) and needs
  `model_type = "nemo_transducer"` in sherpa-onnx.

### Integration in the app (ADR-006)

| Role | Engine (sherpa-onnx 1.13.8) | Manifest |
|---|---|---|
| Streaming DE / EN (Nemotron) | `OnlineRecognizer`, transducer fields (sherpa detects NeMo from the encoder metadata), `greedy_search`, `stream.set_option("language", "de")` after create and every reset, 4 threads | `modelType: online_nemo_transducer`, `roles: ["streaming"]`, `defaultRank: 0` (320 ms) |
| Streaming DE / EN (Kroko) | `OnlineRecognizer`, transducer (encoder/decoder/joiner/tokens) | `modelType: online_transducer`, `defaultRank: 1` |
| Fallback EN | same, tried automatically when the preferred model fails to load | `defaultRank: 2` |
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
- Measured latency/RTF on the target CPU (Snapdragon X): CI covers Arm64 and
  x86-64 runners only; the app's Diagnose tab and the `RealtimeWatch` warning
  show the real figures.
- Nemotron WER on free German speech (only short test clips measured).
- Hotwords for Nemotron need a sherpa-onnx release with `modified_beam_search`
  for streaming NeMo (present on master, not in 1.13.8).

## Sources
- https://huggingface.co/nvidia/nemotron-3.5-asr-streaming-0.6b (licence OpenMDW-1.1, @ea30d66)
- https://huggingface.co/csukuangfj2/sherpa-onnx-nemotron-3.5-asr-streaming-0.6b-320ms-int8-2026-06-11 (@424ce58; 160 ms @b3a4dbd, 80 ms @2ac5952)
- https://github.com/k2-fsa/sherpa-onnx/tree/master/scripts/nemo/nemotron-3.5-asr-streaming-0.6b
- https://huggingface.co/csukuangfj/sherpa-onnx-streaming-zipformer-de-kroko-2025-08-06
- https://huggingface.co/csukuangfj/sherpa-onnx-streaming-zipformer-en-kroko-2025-08-06
- https://huggingface.co/Banafo/Kroko-ASR
- https://huggingface.co/csukuangfj/sherpa-onnx-streaming-zipformer-en-20M-2023-02-17
- https://huggingface.co/csukuangfj/sherpa-onnx-streaming-zipformer-en-2023-06-26
- https://huggingface.co/csukuangfj/sherpa-onnx-nemo-parakeet-tdt-0.6b-v3-int8
- https://huggingface.co/nvidia/parakeet-tdt-0.6b-v3
- https://k2-fsa.github.io/sherpa/onnx/pretrained_models/online-transducer/zipformer-transducer-models.html
