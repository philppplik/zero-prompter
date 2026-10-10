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

Teleprompter tracking matches against a known script, so a moderate WER is
acceptable. Latency and robustness matter more.

## Open points (not verified)
- SHA-256 of all `tokens.txt` files (not LFS). The manifest will not validate until they are filled.
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
