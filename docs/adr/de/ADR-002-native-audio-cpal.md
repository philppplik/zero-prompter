# ADR-002: Audio nativ über cpal statt getUserMedia

- **Status:** angenommen
- **Quelle:** übernommen aus `docs/research/RESEARCH.md` §4 (2026-10-10)

## Entscheidung

Mikrofon-Audio wird nativ in Rust über `cpal` aufgenommen, nicht im WebView über `getUserMedia`.

## Begründung

- Nur ein Code-Pfad; die Audiodaten bleiben im Rust-Thread bei sherpa-onnx.
- Keine Abhängigkeit vom Permission-Handling der WebViews ([tauri#2338](https://github.com/tauri-apps/tauri/issues/2338)).
- Konfliktauflösung: Der GUI-Entwurf hatte getUserMedia offengelassen; entschieden ist cpal.

## Plattformpflichten

- **macOS:** `NSMicrophoneUsageDescription` in `src-tauri/Info.plist` und Entitlement `com.apple.security.device.audio-input`; für den Mac App Store zusätzlich `device.microphone`. Ohne Usage-String liefert cpal stillschweigend nur Nullen ([soloterm/solo#114](https://github.com/soloterm/solo/issues/114)).
- **Windows:** unpackaged keine Capability nötig, bei MSIX schon ([MS Learn](https://learn.microsoft.com/en-us/windows/apps/develop/devices-sensors/enable-device-capabilities)).

## Nachtrag 2026-10-10: Version

Der Adapter bleibt auf **cpal 0.15**. Die aktuelle cpal 0.18.2 ist API-inkompatibel (`build_input_stream` nimmt `StreamConfig` per Wert, Fehler-Callback mit `Error`, `name()` ersetzt durch `description()`/`id()`; [docs.rs](https://docs.rs/cpal/latest/cpal/traits/trait.DeviceTrait.html)). Ein Upgrade ist eine eigene Aufgabe. Die 0.15-API im Code ist nicht gegen die 0.15-Doku geprüft.
