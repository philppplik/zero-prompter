# Third-Party Models

No speech model is bundled with zero-teleprompter. Models are downloaded on
demand from the URLs in `apps/desktop/src-tauri/resources/models.json`.

Every model in the manifest carries its SPDX `license` and, for CC-BY and
OpenMDW models, an `attribution` string. Both are shown in the download dialog and the
Inspector (`requiresAttribution`).

All download URLs are pinned: Hugging Face files to a repository commit
(`/resolve/<commit>/...`), the Silero VAD to the k2-fsa/sherpa-onnx
`asr-models` release asset. Every file has a SHA-256 in the manifest. The
`tokens.txt` hashes (plain git blobs, no LFS hash) were computed from files
downloaded at the pinned commit on 2026-10-10; the Silero hash matches the
GitHub release asset digest.

| Model id | Licence | Attribution | Source URL (pinned revision) | SHA-256 (2026-10-10) |
|---|---|---|---|---|
| nemotron-3.5-streaming-0.6b-160ms-int8 | OpenMDW-1.1 (NVIDIA model card `license_name: openmdw-1.1`, https://openmdw.ai/license/1-1/; verified 2026-10-11 at nvidia/nemotron-3.5-asr-streaming-0.6b @ea30d66) | NVIDIA (Nemotron 3.5 ASR Streaming 0.6B); ONNX int8 export k2-fsa/sherpa-onnx | https://huggingface.co/csukuangfj2/sherpa-onnx-nemotron-3.5-asr-streaming-0.6b-160ms-int8-2026-06-11 @b3a4dbd | all files (tokens.txt hashed 2026-10-11) |
| nemotron-3.5-streaming-0.6b-80ms-int8 | OpenMDW-1.1 | as above | https://huggingface.co/csukuangfj2/sherpa-onnx-nemotron-3.5-asr-streaming-0.6b-80ms-int8-2026-06-11 @2ac5952 | all files |
| nemotron-3.5-streaming-0.6b-320ms-int8 (default DE/EN) | OpenMDW-1.1 | as above | https://huggingface.co/csukuangfj2/sherpa-onnx-nemotron-3.5-asr-streaming-0.6b-320ms-int8-2026-06-11 @424ce58 | all files |
| silero-vad | MIT (github.com/snakers4/silero-vad, LICENSE) | Silero Team | https://github.com/k2-fsa/sherpa-onnx/releases/download/asr-models/silero_vad.onnx (sherpa-onnx itself: Apache-2.0) | yes, = GitHub asset digest |
| zipformer-en-20m-2023-02-17 | Apache-2.0 (HF cardData) | k2-fsa / icefall | https://huggingface.co/csukuangfj/sherpa-onnx-streaming-zipformer-en-20M-2023-02-17 @d42f2d9 | all files |
| zipformer-en-2023-06-26-int8 | unknown (not on card) | k2-fsa / icefall | https://huggingface.co/csukuangfj/sherpa-onnx-streaming-zipformer-en-2023-06-26 | (not in manifest) |
| parakeet-tdt-0.6b-v3-int8 | CC-BY-4.0 | NVIDIA | https://huggingface.co/csukuangfj/sherpa-onnx-nemo-parakeet-tdt-0.6b-v3-int8 @2bda32e (upstream https://huggingface.co/nvidia/parakeet-tdt-0.6b-v3) | all files |

Resulting stack for the public release: Nemotron 3.5 streaming (OpenMDW-1.1,
default 320 ms, 160/80 ms optional), Zipformer EN 20M (Apache-2.0, light
English option), Parakeet TDT 0.6B v3 (CC-BY-4.0, optional resync), Silero VAD
(MIT) and the Windows speech recognition of the OS (no redistribution).

Removed (2026-10-11): the Kroko DE/EN community models
(`zipformer-{de,en}-kroko-2025-08-06`, Banafo, CC-BY-SA with a separate
commercial/OEM licence; https://huggingface.co/Banafo/Kroko-ASR). They are no
longer offered for download. Folders left over from earlier versions are
ignored by the app and can be deleted by hand from `<data>/models/`.

NVIDIA Nemotron 3.5 ASR Streaming 0.6B: the OpenMDW-1.1 licence is a
permissive model licence (use, modification and redistribution allowed; keep
the licence and notices with the materials). The app only downloads the model
and shows the attribution "Nemotron 3.5 ASR Streaming 0.6B by NVIDIA,
OpenMDW-1.1" in the model list. The same `decoder.int8.onnx`, `joiner.int8.onnx`
and `tokens.txt` are shared by all chunk variants (identical SHA-256); only the
encoder differs.

Silero VAD is a small (644 kB) helper model for speech segmentation. The app
installs it automatically together with any speech model and uses it for
resync segmentation; without it, segmentation falls back to an RMS level gate.