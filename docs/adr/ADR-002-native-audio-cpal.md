# ADR-002: Native audio via cpal instead of getUserMedia

> German original: [de/ADR-002-native-audio-cpal.md](de/ADR-002-native-audio-cpal.md)

- **Status:** accepted
- **Source:** taken over from `docs/research/RESEARCH.md` §4 (2026-10-10)

## Decision

Microphone audio is captured natively in Rust via `cpal`, not in the WebView via `getUserMedia`.

## Rationale

- Only one code path; the audio data stays in the Rust thread that feeds sherpa-onnx.
- No dependency on the WebViews' permission handling ([tauri#2338](https://github.com/tauri-apps/tauri/issues/2338)).
- Conflict resolution: the GUI draft had left getUserMedia open; cpal is the decision.

## Platform obligations

- **macOS:** `NSMicrophoneUsageDescription` in `src-tauri/Info.plist` and the entitlement `com.apple.security.device.audio-input`; for the Mac App Store additionally `device.microphone`. Without the usage string, cpal silently delivers only zeros ([soloterm/solo#114](https://github.com/soloterm/solo/issues/114)).
- **Windows:** no capability is needed when unpackaged; for MSIX it is ([MS Learn](https://learn.microsoft.com/en-us/windows/apps/develop/devices-sensors/enable-device-capabilities)).

## Addendum 2026-10-10: version

The adapter stays on **cpal 0.15**. The current cpal 0.18.2 is API-incompatible (`build_input_stream` takes `StreamConfig` by value, the error callback takes `Error`, `name()` is replaced by `description()`/`id()`; [docs.rs](https://docs.rs/cpal/latest/cpal/traits/trait.DeviceTrait.html)). An upgrade is a separate task. The 0.15 API usage in the code has not been checked against the 0.15 documentation.
