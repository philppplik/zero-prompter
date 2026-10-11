# ADR-008: Windows speech recognition as a fallback engine

> German original: [de/ADR-008-windows-speech-fallback.md](de/ADR-008-windows-speech-fallback.md)

- Status: accepted
- Date: 2026-10-11
- Related: ADR-006 (hybrid STT, fallback chain), ADR-007 (error handling, latency metrics)

## Context

The engines from ADR-006 (Nemotron 3.5, Kroko, Zipformer 20M via sherpa-onnx) need a downloaded model. If it is missing, cannot be loaded, or construction fails, the app previously ended up in the demo or error state. Windows ships a system speech recognizer, `Windows.Media.SpeechRecognition`, which is usable without a download as soon as the language pack is installed.

## Decision

1. New adapter `WinSpeechEngine` (`crates/zt-stt/src/windows_speech.rs`). It compiles only on Windows with the Cargo feature `winspeech`. The feature is active in the Windows release builds (x64, ARM64, Store) and is a no-op on other platforms.
2. **Self-capturing engine:** Windows opens the default microphone itself. For this the `SpeechEngine` trait gains `fn self_capturing(&self) -> bool` with the default `false`. When `true`, the engine ignores `accept_waveform`. The cpal pipeline keeps running anyway: it provides levels, the channel display and the microphone watchdog, and detects speech onset for latency measurement. On Windows several apps may record simultaneously in shared mode. Latency is measured only as *speech onset -> first partial* (`onset_to_partial`), because `audio_to_partial` and `end_to_end` say nothing without shared audio blocks.
3. **Events:**
   - `SpeechRecognizer.HypothesisGenerated` -> `SttEvent::Partial`
   - `ContinuousRecognitionSession.ResultGenerated` -> `SttEvent::Final`. Results with confidence `Rejected` are discarded.
   - `StateChanged` (`SpeechDetected` / `SoundEnded`) -> `SttEvent::Vad`
   - `Completed` (the session ends on its own, e.g. because the microphone is gone, the network is missing or a timeout hits) -> automatic restart
   - `EndSilenceTimeout` 400 ms, `AutoStopSilenceTimeout` 24 h
4. **Threading:** All WinRT objects live on a dedicated `zt-winspeech` thread in the MTA. windows-rs implicitly joins the MTA on first activation via `CoIncrementMTAUsage`, so no `unsafe` is needed and `unsafe_code = "forbid"` stays in force. Asynchronous operations (`CompileConstraintsAsync`, `StartAsync`, `StopAsync`) are awaited there in a blocking fashion with `.get()`, never on the UI thread. The event handlers run in the WinRT thread pool and only push into an `mpsc` channel. The voice thread polls that channel like for any other engine. As a result, `WinSpeechEngine` holds only channels, independent of the agile status of the WinRT classes, and is `Send`.
5. **Constraints:**
   - The default is `SpeechRecognitionTopicConstraint(Dictation)`, i.e. free dictation.
   - If dictation fails (privacy setting off or no network) and a grammar recognizer is installed for the language, the engine switches to **script-word mode**: a `SpeechRecognitionListConstraint` built from the script words (hotwords, max. 2000). It runs offline but recognizes only exactly those words or phrases. It starts as soon as `set_hotwords` supplies the words.
   - Topic and list constraints are not mixed.
6. **Fallback chain** (`apps/desktop/src-tauri/src/winspeech.rs`, `voice::make_engine`):
   1. installed models (Nemotron -> Kroko -> 20M)
   2. Windows speech recognition
   3. error status (`engineError`, the message names both causes). If no model is installed at all, it stays in demo mode, and the note explains why Windows was not available.
7. **Manual choice:** Under Settings -> Voice, "Windows Spracherkennung (System)" (id `windows-speech`) appears in the model selection (the label is a UI string and is currently German). There is nothing to download. `Selection.windowsSpeech` reports whether the build contains the engine.
8. **Crate:** `windows = "=0.61.3"` is pinned with the features `Foundation`, `Foundation_Collections`, `Globalization` and `Media_SpeechRecognition`. 0.62.x requires Rust 1.82, the workspace MSRV is 1.80. Tauri pulls 0.61.3 into the lockfile anyway, so no additional copy is created.

## Error cases and UI hints

| Case | Detection | Message / link |
|---|---|---|
| Language pack missing | Language not in `SupportedTopicLanguages` or `SupportedGrammarLanguages`, or `SpeechRecognizer(Language)` fails | "Windows language pack for 'de' missing ..." -> `ms-settings:regionlanguage` (language options -> speech recognition), `ms-settings:speech` |
| Online speech recognition off | HRESULT `0x80045509` (SPERR_SPEECH_PRIVACY_POLICY_NOT_ACCEPTED) | -> `ms-settings:privacy-speech`. Error without a grammar language, otherwise script-word mode |
| Microphone denied | `0x80070005` | -> `ms-settings:privacy-microphone` |
| No network / dictation service | Compile status `NetworkFailure` | Hint "Internet required" |
| Language mismatch | `TopicLanguageNotSupported` / `GrammarLanguageMismatch` | Hint about the language pack |

## Limitations (honest)

- **Dictation is an online service.** On Windows 10/11, `SpeechRecognitionScenario.Dictation` requires the "Online speech recognition" setting and an internet connection. Audio goes to Microsoft. This contradicts the app's offline claim, which is why the engine is only a **fallback** or a deliberate manual choice.
- **Script-word mode is coarse:** It recognizes only words from the list, each result is short, and there are no partials in word order. For tracking it suffices only as an emergency mode.
- **Unpackaged desktop app:** According to Microsoft documentation and community reports, `SpeechRecognizer` and `ContinuousRecognitionSession` also work without package identity. Microphone access is then governed by "Let desktop apps access your microphone" instead of an app capability. **Not yet verified** on real hardware (CI has no microphone). For the Store build (MSIX), the `microphone` capability in the manifest might be required; that is open.
- **No hotword biasing in dictation:** The topic constraint accepts only a hint string, not script words.
- **Second microphone open:** Windows uses the *default* input device, not the one selected in the app. The app's gain and channel selection do not affect recognition.
- **ARM64:** WinRT APIs are natively available on ARM64. CI builds and initializes on `windows-11-arm`; recognition with real speech is untested.
- **Newer Windows AI APIs** (Windows App SDK / "Windows AI Foundry", Phi Silica, Copilot+ NPU models) offer, as of 2026-10, no generally available streaming STT API for unpackaged Win32 apps with German. They would also require the Windows App SDK as a runtime and in part Copilot+ hardware. Therefore rejected. Re-evaluate once a streaming ASR API is stable.
- The old SAPI 5 desktop recognition (`Win32_Media_Speech`, legacy "Windows Speech Recognition") is deprecated in Windows 11 (replaced by Voice Access) and has no usable German dictation. Therefore rejected.

## Consequences

- Without a download, voice tracking is immediately possible on Windows (with online speech recognition enabled).
- The release builds additionally depend on the `windows` crate (MIT/Apache-2.0, already present via Tauri).
- CI job `winspeech` (windows-latest and windows-11-arm): clippy and tests with the feature, plus `examples/winspeech_probe` (list languages, create the recognizer, `CompileConstraintsAsync` without a microphone). If the runner lacks the language pack or consent, the probe reports `SKIP`/`FAILED` as information. It turns red only if a grammar language is installed and the offline list constraint still fails to compile.

## Evidence (CI, 2026-10-11)

`examples/winspeech_probe` ran on `windows-latest` (x64) and `windows-11-arm` (ARM64), unpackaged as a normal process. Both runners have only `en-US` (token `MS-1033-110-WINMO-DNN`) and no German language pack. The probe therefore reports "SKIP ... probing system language en-US" for `de`, which corresponds to the expected failure path "language pack missing". For `en-US` everything succeeded: `SpeechRecognizer` was created, and `CompileConstraintsAsync` returned `Success` for both Dictation and ScriptList (~115 ms warm, ~1.1-1.8 s on the first call). So the dictation constraint compiles even without online consent. The privacy check (`0x80045509`) only triggers at `StartAsync`, which the adapter handles the same way. Not verified, because CI has no microphone: `StartAsync`, partials/finals with real speech, latency and recognition quality.
