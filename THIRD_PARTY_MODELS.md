# Third-Party Models

No speech model is bundled with zero-teleprompter. Models are downloaded on
demand from the URLs in `apps/desktop/src-tauri/resources/models.json`.

Every model in the manifest carries its SPDX `license` and, for CC-BY models,
an `attribution` string. Both are shown in the download dialog and the
Inspector (`requiresAttribution`).

All download URLs are pinned: Hugging Face files to a repository commit
(`/resolve/<commit>/...`), the Silero VAD to the k2-fsa/sherpa-onnx
`asr-models` release asset. Every file has a SHA-256 in the manifest. The
`tokens.txt` hashes (plain git blobs, no LFS hash) were computed from files
downloaded at the pinned commit on 2026-10-10; the Silero hash matches the
GitHub release asset digest.

| Model id | Licence | Attribution | Source URL (pinned revision) | SHA-256 (2026-10-10) |
|---|---|---|---|---|
| silero-vad | MIT (github.com/snakers4/silero-vad, LICENSE) | Silero Team | https://github.com/k2-fsa/sherpa-onnx/releases/download/asr-models/silero_vad.onnx (sherpa-onnx itself: Apache-2.0) | yes, = GitHub asset digest |
| zipformer-de-kroko-2025-08-06 | CC-BY-SA (version TODO) | Banafo / Kroko ASR | https://huggingface.co/csukuangfj/sherpa-onnx-streaming-zipformer-de-kroko-2025-08-06 @887db3d | all files |
| zipformer-en-kroko-2025-08-06 | CC-BY-SA (version TODO) | Banafo / Kroko ASR | https://huggingface.co/csukuangfj/sherpa-onnx-streaming-zipformer-en-kroko-2025-08-06 @572aaf4 | all files |
| zipformer-en-20m-2023-02-17 | Apache-2.0 (HF cardData) | k2-fsa / icefall | https://huggingface.co/csukuangfj/sherpa-onnx-streaming-zipformer-en-20M-2023-02-17 @d42f2d9 | all files |
| zipformer-en-2023-06-26-int8 | unknown (not on card) | k2-fsa / icefall | https://huggingface.co/csukuangfj/sherpa-onnx-streaming-zipformer-en-2023-06-26 | (not in manifest) |
| parakeet-tdt-0.6b-v3-int8 | CC-BY-4.0 | NVIDIA | https://huggingface.co/csukuangfj/sherpa-onnx-nemo-parakeet-tdt-0.6b-v3-int8 @2bda32e (upstream https://huggingface.co/nvidia/parakeet-tdt-0.6b-v3) | all files |

Kroko licence source: https://huggingface.co/Banafo/Kroko-ASR. That page says the
community models are licensed "under CC-BY-SA", but its metadata field reads
"test". CC-BY-SA is share-alike, so the legal implications for redistribution
need review. Bundling is not planned; models are downloaded only.

Silero VAD is a small (644 kB) helper model for speech segmentation. The app
installs it automatically together with any speech model and uses it for
resync segmentation; without it, segmentation falls back to an RMS level gate.