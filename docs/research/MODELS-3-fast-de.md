# MODELS-3: Schnelle deutsche Streaming-ASR (Stand 2026-10-11)

Ziel: DE, erstes Wort < 300 ms, Partial-Takt <= 200 ms, robust bei 200+ WPM, klein, kommerziell, CPU inkl. Win ARM64, sherpa-onnx 1.13.8 (Rust).

Hinweis zur Belegqualität: Recherche per Websuche; die offizielle HF-Modellkarte von Nemotron 3.5 war nicht direkt abrufbar, Angaben stammen teils aus Mirrors/Reviews. Felder mit "?" sind unbestätigt und vor Entscheidung zu verifizieren.

## Tabelle

| Modell | Streaming | Chunk/Latenz | Sprachen | Größe | Lizenz | WER DE | sherpa-onnx | Schnellsprecher |
|---|---|---|---|---|---|---|---|---|
| NVIDIA Nemotron 3.5 ASR Streaming 0.6B | ja, cache-aware FastConformer-RNNT | 80/160/320/560/1120 ms wählbar (Laufzeit-Konfig) | 40 Locales trainiert, DE "ready" | 600M (~0,6-1,2 GB fp32, int8 ~ 0,6 GB) | OpenMDW-1.1 (kommerziell ok) | ? (Karte prüfen) | wird integriert (NeMo-Transducer-Export) | gut: Cache-aware, keine Fensterüberlappung; 80 ms möglich, leicht schlechtere WER |
| Kroko DE streaming Zipformer | ja | effektiv ~1,3 s Partials (gemessen) | DE | klein (~70-150 MB) | Kroko-Lizenz prüfen | ? | ja, seit 1.12.10 | schlecht wegen grobem Takt |
| NVIDIA FastConformer hybrid streaming 80/480/1040 ms | ja | 0/80/480/1040 ms | nur EN gefunden; kein `stt_de_..._streaming` nachweisbar | 114M | CC-BY-4.0 | – | ja (EN) | n/a für DE |
| Zipformer DE kleine Chunks (chunk-16/32) | ja | 320-640 ms | DE nur via Kroko/Community | klein | ? | ? | nur wenn exportiert | ok, aber kein offizielles DE-Modell gefunden |
| Paraformer / SenseVoice / Dolphin | Paraformer streaming nur ZH/EN; SenseVoice offline (kein echtes Streaming); Dolphin asiatische Sprachen | – | kein DE-Fokus | – | div. | – | ja | ungeeignet |
| Moonshine v2 | ja (EN), weitere Sprachen nur einzelne | – | kein DE bestätigt | klein | – | – | teilw. | verworfen |
| Picovoice Cheetah / Cheetah Fast | ja | "ultra-low latency" (Fast); kein DE-Zahlenwert publiziert | EN, FR, DE, IT, PT, ES | klein | proprietär, AccessKey, kostenpflichtig | Benchmark selbst laufen lassen | nein; Rust-Crate `pv_cheetah` (2.0.2, Feb 2024) | gut laut Hersteller; ARM64-Windows prüfen |
| Vosk/Kaldi small-de | ja | ~ Wortgranular, schnelle Partials | DE | ~45 MB | Apache-2.0 | ~ 13-20 % | nein | brauchbar, alt |
| Windows.Media.SpeechRecognition (de-DE) | ja (Continuous Dictation) | Hypothesen ~ 200-500 ms (erfahrungsgemäß) | DE | OS | OS-Bestandteil | mittel | nein (WinRT, Rust via `windows` crate) | mittel; ARM64 nativ; Fallback-Option |
| sherpa-onnx KWS (Zipformer KWS) mit Skriptwörtern | ja | sehr klein (Frame-Ebene) | nur EN/ZH-Modelle gefunden | ~3-15 MB | Apache-2.0 | n/a | ja | Idee stark, aber ohne DE-KWS-Modell Tokenizer-Problem |
| Azure Embedded Speech, Speechmatics/Deepgram on-device, Apple SpeechAnalyzer | ja | niedrig | DE | – | proprietär/Vertrag bzw. nur Apple | – | nein | Apple irrelevant für Windows; Rest Lizenzhürde |

## Tricks
- Nemotron-Chunk auf 80 ms stellen: halbierter Takt ggü. 160 ms; Rechenlast pro Sekunde steigt, auf Snapdragon RTF messen.
- Greedy-Decoding + int8-Encoder; Partial-Diff nur auf neue Tokens.
- Statt voller Wortgenauigkeit: Fuzzy/phonetisches Alignment (Kölner Phonetik) der Partials gegen das Skript; erlaubt kleineres/schlechteres Modell.
- Hotwords/Contextual Biasing (sherpa-onnx `hotwords_file` für Transducer, modified_beam_search) mit Skriptwörtern.

## Empfehlung Top 3
1. **Nemotron 3.5 Streaming 0.6B, 80 ms** (Fallback 160 ms) + Skript-Hotwords + phonetisches Alignment. Einziges offenes, kommerzielles DE-Streaming mit kleinem Chunk.
2. **Windows.Media.SpeechRecognition de-DE** als leichtgewichtiger, ARM64-nativer Fallback (kein Download, Latenz messen).
3. **Picovoice Cheetah Fast DE** nur falls kommerzielle Lizenz akzeptabel; Rust-SDK vorhanden.
(Kroko bleibt Low-Resource-Fallback; KWS lohnt erst bei verfügbarem DE-/multilingualem KWS-Modell.)

## Quellen
- https://huggingface.co/nvidia/nemotron-3.5-asr-streaming-0.6b
- https://wavect.io/blog/nvidia-nemotron-3-5-asr-production-review.md
- https://deepinfra.com/nvidia/Nemotron-3.5-ASR-Streaming-Multilingual-0.6b
- https://catalog.ngc.nvidia.com/orgs/nvidia/teams/nemo/models/stt_en_fastconformer_hybrid_large_streaming_80ms
- https://huggingface.co/nvidia/stt_en_fastconformer_hybrid_large_streaming_multi
- https://github.com/k2-fsa/sherpa-onnx/issues/3144
- https://k2-fsa.github.io/sherpa/onnx/pretrained_models/index.html
- https://modulellm-tutorial.readthedocs.io/en/latest/model_pages/kws.html
- https://picovoice.ai/docs/cheetah
- https://picovoice.ai/docs/quick-start/cheetah-rust
- https://picovoice.ai/blog/speech-to-text-latency/
