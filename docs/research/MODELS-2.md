# MODELS-2: ASR-Modelle für zero-prompter (Stand 2026-10-11)

Rahmen: sherpa-onnx 1.13.8 (Rust), Latenz Wort→Scroll < 300 ms, CPU-only (inkl. Windows ARM64), DE + EN, kommerzielle Nutzung.
Bisheriger Katalog: Kroko DE/EN (Lizenz unklar), zipformer-en-20M (Apache-2.0), Parakeet TDT 0.6B v3 int8 (offline resync, CC-BY-4.0), Silero VAD.

Legende: **verifiziert** = Lizenz/Zahl auf der Primärquelle (HF-Modellkarte des Herstellers bzw. Herstellerseite) gelesen. **ungeprüft** = nur aus Sekundärquelle oder Schätzung. CPU-Latenzen sind Schätzungen (eigene Messung auf x64 und ARM64 nötig).

## Tabelle

| Modell | Typ | Sprachen | Größe | Lizenz | WER | CPU-Einschätzung | sherpa-onnx | Teleprompter-Eignung |
|---|---|---|---|---|---|---|---|---|
| **NVIDIA Nemotron 3.5 ASR Streaming 0.6B** | Streaming, cache-aware FastConformer-RNNT; Chunk 80/160/320/560/1120 ms | ~40 Locales in einem Checkpoint, u. a. de-DE (Tier "transcription-ready"), en; Sprach-Prompt oder "auto" | int8: Encoder 658 MB + Decoder 15 MB + Joiner 9.5 MB (~685 MB) | **OpenMDW-1.1**, laut Karte "ready for commercial use" (verifiziert) | FLEURS DE: 9.81 (80 ms) / 9.21 (160 ms) / 8.83 (320 ms) / 8.42 (560 ms); EN: 9.43 / 8.88 / 8.27 / 7.99 (verifiziert) | 600M Parameter: auf aktuellen x64- und Apple-CPUs laut Drittbericht echtzeitfähig; auf Snapdragon-X-ARM64 vermutlich RTF ~0.2-0.4 (ungeprüft). 160-ms-Chunk ergibt eine algorithmische Latenz von ~160 ms plus Rechenzeit, was knapp unter 300 ms liegen dürfte. | Ja: `csukuangfj2/sherpa-onnx-nemotron-3.5-asr-streaming-0.6b-{80,160,320,560,1120}ms[-int8]-2026-06-11` (HF). Benötigt sherpa-onnx >= 1.13.4 (Drittquelle), 1.13.8 passt. | **Neuer DE-Default.** Einziges lizenzsauberes, gutes DE-Streaming-Modell; zugleich sehr gutes EN. Nachteil ist die Größe. |
| NVIDIA Nemotron Speech Streaming EN 0.6B | Streaming, cache-aware RNNT, 80 ms-1.12 s | EN | int8 ~632 MB | NVIDIA Open Model License, kommerziell erlaubt (verifiziert) | OpenASR-Mittel: 8.43 (80 ms) / 7.67 (160 ms) / 7.07 (560 ms) / 6.93 (1.12 s) (verifiziert) | wie oben | `csukuangfj/sherpa-onnx-nemotron-speech-streaming-en-0.6b[-int8]-2026-01-14`; Varianten je Chunk: `csukuangfj2/sherpa-onnx-nemotron-speech-streaming-en-0.6b-{80,160,560,1120}ms[-int8]-2026-04-25` | Option "EN hohe Qualität". Durch 3.5 multilingual weitgehend ersetzt (spart einen Download). |
| NeMo Parakeet unified EN 0.6B (streaming) | Streaming, 240/560/1120 ms | EN | ~0.6B, int8 ~600 MB (geschätzt) | Lizenz auf der NVIDIA-Karte nicht geprüft (ungeprüft) | – | 240 ms Mindest-Chunk ist für < 300 ms zu knapp | `csukuangfj2/sherpa-onnx-nemo-parakeet-unified-en-0.6b[-int8]-streaming-{240,560,1120}ms` | Nein: Nemotron ist besser und hat kleinere Chunks. |
| NeMo streaming FastConformer EN (CTC/Transducer, ~114M) | Streaming, 80/480/1040 ms | EN | int8 ~120 MB (geschätzt) | NVIDIA-NeMo-Modelle meist CC-BY-4.0 (ungeprüft) | LibriSpeech ~5-6 (ungeprüft) | Leicht, RTF klein | `csukuangfj/sherpa-onnx-nemo-streaming-fast-conformer-{ctc,transducer}-en-{80ms,480ms,1040ms}[-int8]` | **Leichtes EN-Streaming** als Mittelklasse zwischen zipformer-20M und Nemotron. |
| Kroko DE / EN (streaming zipformer) | Streaming | DE / EN | ~71 MB | Community-Modelle **CC-BY-SA**, kommerziell ist eine separate "Commercial & OEM"-Lizenz vorgesehen (verifiziert, Banafo/Kroko-ASR) | – | Sehr leicht, < 100 ms | `sherpa-onnx-streaming-zipformer-de-kroko-2025-08-06` | Technisch ideal, lizenzrechtlich problematisch: Share-Alike auf ein Modell, das in einer kommerziellen App mitgeliefert wird, ist riskant. Nur optional als Download durch den Nutzer anbieten oder eine OEM-Lizenz kaufen. |
| sherpa-onnx-streaming-zipformer-en-20M-2023-02-17 | Streaming | EN | ~20M Parameter, < 100 MB | Apache-2.0 (bereits im Katalog) | schwächer | sehr leicht | vorhanden | Bleibt als **Low-End-Fallback für EN**. |
| Vosk small-de-0.15 / small-en-us-0.15 (Kaldi) | Streaming (Kaldi, eigene Engine, **nicht sherpa-onnx**) | DE / EN | 45 MB / 40 MB | Apache-2.0 (verifiziert, alphacephei.com) | DE: Tuda 13.75, Podcast 30.67; EN: LS-clean 9.85 | Sehr leicht. Rust-Bindings (`vosk` crate) brauchen libvosk; ein ARM64-Windows-Build ist fraglich (ungeprüft). | Nein (nur neue Vosk-Zipformer für bn/ru in sherpa) | Einziger lizenzsauberer **leichter** DE-Streaming-Fallback, bringt aber eine zweite Engine mit. vosk-model-de-0.21 (1.9 GB, Apache-2.0, Tuda 9.83) ist zu groß. |
| NeMo FastConformer Transducer/CTC multilingual (en-de-es-fr-14288, 10-lang-20k) | **Offline** (nicht cache-aware) | DE, EN u. a. | int8 ~100-150 MB (geschätzt) | NVIDIA, vermutlich CC-BY-4.0 (ungeprüft) | – | schnell | `csukuangfj/sherpa-onnx-nemo-fast-conformer-transducer-en-de-es-fr-14288-int8` u. a. | Leichter **Offline-Resync für DE**, falls Parakeet v3 auf ARM64 zu langsam ist. |
| stt_de_fastconformer_hybrid_large_pc | Offline | DE | ~115M | NVIDIA, CC-BY-4.0 (ungeprüft) | MCV-DE ~5-6 (ungeprüft) | schnell | `csukuangfj/sherpa-onnx-nemo-transducer-stt_de_fastconformer_hybrid_large_pc[-int8]` | Guter leichter DE-Resync. |
| Canary 180M Flash (en/de/es/fr) | Offline, AED | EN, DE, ES, FR | int8 ~200 MB | CC-BY-4.0 (ungeprüft) | – | mittel | `csukuangfj/sherpa-onnx-nemo-canary-180m-flash-en-es-de-fr-int8` | Resync-Alternative, kein Streaming. |
| Parakeet TDT 0.6B v3 int8 | Offline | 25 EU-Sprachen | ~640 MB | CC-BY-4.0 | – | – | vorhanden | Resync behalten, oder durch Nemotron 3.5 ersetzen (eine einzige Datei für Streaming und Resync). |
| Moonshine Streaming tiny/small/medium | Streaming (80 ms Lookahead, autoregressiver Decoder) | **nur EN** | 34M / 123M / 245M | MIT (verifiziert) | LS-clean 4.49 / 2.49 / 2.08 (Sekundärquelle) | gut; die Ausgabelatenz wächst aber mit der Textlänge | Nicht als Streaming-Modell in sherpa (sherpa unterstützt Moonshine v1 offline) | Für DE nutzlos. Für EN durch Nemotron/FastConformer abgedeckt. |
| Whisper (tiny…large-v3-turbo, distil) | Offline (30-s-Fenster). "whisper-streaming" ist Re-Decoding mit LocalAgreement, Latenz ≥ 1-2 s | multilingual inkl. DE | 39M-809M | MIT | gut | auf CPU zu langsam für < 300 ms | sherpa-onnx `sherpa-onnx-whisper-*` (offline) | Nicht für Tracking geeignet; höchstens für Resync. |
| Kyutai STT 1B en_fr / 2.6B en | Streaming (0.5 s bzw. 2.5 s Delay) | EN+FR / EN, **kein DE** | 1B / 2.6B | CC-BY-4.0 (verifiziert) | – | 1B auf CPU grenzwertig. Rust via candle möglich. | Nein | Nein (kein DE, Delay 500 ms). |
| Mistral Voxtral Realtime (Mini 4B) | Streaming, Delay konfigurierbar (Hersteller: < 200 ms möglich; Parität mit Whisper bei 480 ms) | 13 Sprachen inkl. DE | 4B | Apache-2.0 (Herstellerangabe, Sekundärquelle) | – | 4B auf CPU nicht echtzeitfähig unter 300 ms | Nein | Nein (zu groß für CPU). |
| Cohere Transcribe 14-lang int8 (2026-04-01) | Offline | 14 Sprachen | – | ungeprüft | – | – | `sherpa-onnx-cohere-transcribe-14-lang-int8-2026-04-01` (GitHub Release `asr-models`) | Nur Resync-Kandidat; Lizenz vorher prüfen. |

## Empfehlung: neue Default-Strategie

**Grundidee:** Ein lizenzsauberes, multilinguales Streaming-Modell als Default. Leichte Fallbacks nur dort, wo die Lizenz klar ist.

1. **DE-Default (Streaming): Nemotron 3.5 ASR Streaming 0.6B int8, 160-ms-Variante** (`csukuangfj2/sherpa-onnx-nemotron-3.5-asr-streaming-0.6b-160ms-int8-2026-06-11`), Sprach-Prompt fest auf `de`, nicht `auto`. Lizenz OpenMDW-1.1, kommerziell ok. Für Teleprompter-Tracking reicht eine WER von ~9 %, weil wir gegen bekannten Skripttext matchen. Wenn ARM64 zu langsam ist, auf die 320-ms-Variante wechseln (DE-WER 8.83), dann aber mit Wort-Vorhersage aus dem Skript, um die Latenz zu kaschieren.
2. **EN-Default:** dasselbe Modell mit Prompt `en` (ein einziger Download für beide Sprachen). Wer höchste EN-Qualität will, nimmt Nemotron Speech Streaming EN 0.6B (NVIDIA Open Model License).
3. **Leicht-Modus / Low-End-CPU:** EN mit NeMo streaming FastConformer transducer 80ms int8, sonst zipformer-en-20M. Für DE gibt es leider **kein lizenzsauberes leichtes sherpa-onnx-Streaming-Modell**. Optionen:
   (a) Kroko DE nur als optionaler Nutzer-Download mit Hinweis auf CC-BY-SA, oder Kroko-OEM-Lizenz kaufen;
   (b) Vosk small-de-0.15 (Apache-2.0, 45 MB) als zweite Engine, wobei ARM64-Windows geprüft werden muss.
4. **Resync (offline):** Parakeet TDT v3 behalten. Für schwache CPUs leichter: `sherpa-onnx-nemo-transducer-stt_de_fastconformer_hybrid_large_pc-int8` (DE) oder die en-de-es-fr-FastConformer.
5. **Kroko nicht mehr als Default ausliefern**, solange keine kommerzielle Lizenz vorliegt.

**Vor dem Umstieg zu messen:** RTF und Latenz von Nemotron 3.5 int8 (80/160/320 ms) auf Windows ARM64 (Snapdragon X) und x64 mit 2-4 Threads; RAM-Bedarf (~1 GB erwartet); Startzeit beim Laden von 685 MB. Außerdem die Lizenz der NeMo-Offline-Modelle auf den NVIDIA-Karten nachprüfen (hier als ungeprüft markiert).

## Quellen

- https://huggingface.co/nvidia/nemotron-3.5-asr-streaming-0.6b (Lizenz OpenMDW-1.1, Sprachen, WER-Tabelle, Chunks)
- https://huggingface.co/csukuangfj2/sherpa-onnx-nemotron-3.5-asr-streaming-0.6b-320ms-int8-2026-06-11 (Dateigrößen)
- https://huggingface.co/nvidia/nemotron-speech-streaming-en-0.6b (NVIDIA Open Model License, WER)
- https://openwhispr.com/blog/local-streaming-speech-to-text (Nemotron auf CPU, sherpa-onnx >= 1.13.4)
- https://huggingface.co/api/models?search=sherpa-onnx-nemo-streaming und ?search=sherpa-onnx%20de (Modellnamen)
- https://k2-fsa.github.io/sherpa/onnx/pretrained_models/online-transducer/index.html
- https://huggingface.co/Banafo/Kroko-ASR (CC-BY-SA vs. Commercial/OEM)
- https://alphacephei.com/vosk/models (Vosk-Größen, WER, Lizenzen)
- https://huggingface.co/moonshine-ai/moonshine-streaming-small (MIT, nur EN)
- https://huggingface.co/kyutai/stt-1b-en_fr (CC-BY-4.0, EN/FR, 0.5 s)
- https://mistral.ai/news/voxtral-transcribe-2 (Voxtral Realtime, Apache-2.0, 13 Sprachen)
- https://github.com/k2-fsa/sherpa-onnx/releases/tag/asr-models
