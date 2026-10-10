# Third-Party Models

No speech model is bundled with zero-teleprompter. Models are downloaded on
demand from the URLs in `apps/desktop/src-tauri/resources/models.json`.

Every model in the manifest carries its SPDX `license` and, for CC-BY models,
an `attribution` string. Both are shown in the download dialog and the
Inspector (`requiresAttribution`).

**Open blocker:** the `tokens.txt` files are plain git blobs on Hugging Face, so
the HF API gives no SHA-256 for them. Their `sha256` is empty and
`Manifest::validate` will reject the manifest until someone hashes the
downloaded files. URLs still point to `main` and must be pinned to a commit.

| Model id | Licence | Attribution | Source URL | Verified (2026-10-10, HF API lfs.sha256) |
|---|---|---|---|---|
| zipformer-de-kroko-2025-08-06 | CC-BY-SA (version TODO) | Banafo / Kroko ASR | https://huggingface.co/csukuangfj/sherpa-onnx-streaming-zipformer-de-kroko-2025-08-06 | onnx files yes, tokens.txt no |
| zipformer-en-kroko-2025-08-06 | CC-BY-SA (version TODO) | Banafo / Kroko ASR | https://huggingface.co/csukuangfj/sherpa-onnx-streaming-zipformer-en-kroko-2025-08-06 | onnx files yes, tokens.txt no |
| zipformer-en-20m-2023-02-17 | Apache-2.0 (HF cardData) | k2-fsa / icefall | https://huggingface.co/csukuangfj/sherpa-onnx-streaming-zipformer-en-20M-2023-02-17 | onnx files yes, tokens.txt no |
| zipformer-en-2023-06-26-int8 | unknown (not on card) | k2-fsa / icefall | https://huggingface.co/csukuangfj/sherpa-onnx-streaming-zipformer-en-2023-06-26 | onnx files yes, tokens.txt no |
| parakeet-tdt-0.6b-v3-int8 | CC-BY-4.0 | NVIDIA | https://huggingface.co/csukuangfj/sherpa-onnx-nemo-parakeet-tdt-0.6b-v3-int8 (upstream https://huggingface.co/nvidia/parakeet-tdt-0.6b-v3) | onnx files yes, tokens.txt no |

Kroko licence source: https://huggingface.co/Banafo/Kroko-ASR. That page says the
community models are licensed "under CC-BY-SA", but its metadata field reads
"test". CC-BY-SA is share-alike, so the legal implications for redistribution
need review. Bundling is not planned; models are downloaded only.
