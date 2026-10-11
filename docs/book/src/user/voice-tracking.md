# Voice tracking

With voice tracking, the app listens through the microphone, recognizes the words you speak **locally**, and scrolls the text so that the word you spoke last sits at the reading marker. Audio is processed in memory (microphone, resampler, speech recognizer) and is not stored or uploaded. There is one exception, the optional Windows system speech fallback; see [Windows speech recognition](#windows-speech-recognition-system-fallback).

Terminology: the model that produces results while you speak is the **streaming model**. An optional second, slower model corrects the position in the background; that correction is called **resync**.

## Requirements

| Building block | Where it comes from |
|---|---|
| Microphone | Build feature `cpal` (on by default). |
| Speech recognition | Build feature `sherpa` (included in all release builds) and an installed streaming model: NVIDIA Nemotron 3.5 Streaming or, English only, Zipformer EN 20M, or Windows speech recognition. |

### Which engine runs

When you start voice tracking or the microphone test, the app picks the engine as follows (see ADR-007 and ADR-008 in the repository).

| Situation | What happens |
|---|---|
| A streaming model for the script language is installed | Real recognition with that model. |
| Only a streaming model for **another** language is installed (for example German, but only Zipformer EN 20M) | On Windows: Windows speech recognition in the right language; otherwise the other model is used and a notice appears. For German, download Nemotron. |
| **No** streaming model is installed (Parakeet and Silero VAD do not count; they are helper models) | **Demo mode**: a mock engine "reads" the script by itself at about 2.3 words per second. A **Demo - load model (Demo · Modell laden)** badge appears next to the status and leads to Settings, Voice and models. On Windows builds with the `winspeech` feature, the Windows speech fallback is tried first. |
| The build does not include speech recognition (no `sherpa` feature) | Demo mode with a notice. |
| A model is installed but cannot be loaded | **No demo.** An error message appears, plus a **Model error - details (Modellfehler · Details)** badge that opens the Diagnostics tab. The text does not move by itself. On Windows, the Windows speech fallback is tried first. |

After a model download finishes, a running voice session restarts automatically and uses the new model. Demo mode is not real speech recognition and is always labeled as such.

## Speech models

The model catalog is `apps/desktop/src-tauri/resources/models.json` (8 entries). Every file URL is pinned to a commit or release asset and has a SHA-256 hash. Sizes below are the sum of the manifest file sizes in decimal MB, as shown in the app. Licenses are listed in `THIRD_PARTY_MODELS.md`; some carry conditions (CC-BY-SA, CC-BY, OpenMDW), so read them before redistributing.

| Model | Role | Languages | Download | License | Notes |
|---|---|---|---|---|---|
| **Nemotron 3.5 ASR Streaming 0.6B int8, 320 ms** (default) | streaming | de, en | 682 MB | OpenMDW-1.1 | Recommended. Cursor can move about every 0.3 s. Output with capitalization and punctuation. No hotwords. |
| Nemotron 3.5, 160 ms | streaming | de, en | 682 MB | OpenMDW-1.1 | Faster reaction, needs a strong CPU. |
| Nemotron 3.5, 80 ms | streaming | de, en | 682 MB | OpenMDW-1.1 | Fastest reaction, needs a very strong CPU. |

| Streaming Zipformer EN 20M int8 (2023-02-17) | streaming | en | 44 MB | Apache-2.0 | Smallest. English only. |
| NVIDIA Parakeet TDT 0.6B v3 int8 | resync (offline) | de, en | 670 MB | CC-BY-4.0 | Used only for [resync](#resync-in-the-background). |
| Silero VAD | helper (segmentation) | - | 0.6 MB | MIT | Installed automatically together with any speech model. |

Timing figures (cursor cadence, real-time factor on CI machines) come from `docs/research/MODELS.md` and ADR-007; they depend on your CPU. Hotwords (words from your script handed to the recognizer) work only with `modified_beam_search` decoding, which is the case for Zipformer EN 20M. Nemotron runs only with greedy search in the pinned sherpa-onnx 1.13.8 and therefore has no hotwords.

### Downloading and removing models

Open Settings, Voice and models (Voice & Modelle) and scroll to **Speech models (Sprachmodelle)**. Each row shows name, languages, size, role and license. Click **Download (Laden)**; interrupted downloads show **Resume (Fortsetzen)**. A progress bar is shown while downloading. **Remove (Entfernen)** deletes an installed model. Downloads use HTTPS only and each file is checked against its SHA-256 hash. If a file has no hash in the manifest, it is hashed after the download and the row says that it is unverified. This download is the only network access of the app in normal use.

You can supply your own manifest as `<data folder>/models/manifest.json`; it replaces the bundled one. Only HTTPS URLs are accepted.

### Choosing a model

Under **Speech recognition (Spracherkennung)** pick a streaming model per language (German and English) or **Automatic (Automatisch)**. "Not downloaded" is appended to models that are not installed.

Automatic picks the first **installed** model in manifest order: Nemotron 320 ms, then (English only) Zipformer 20M, then Windows speech recognition. There is no light German model any more (Kroko was removed for license reasons); German without Nemotron uses Windows speech recognition, otherwise the app shows demo mode with a download prompt. Folders of previously downloaded Kroko models are ignored (delete them by hand from `<data folder>/models/`), and a saved Kroko choice falls back to Automatic (Nemotron). If a model cannot be loaded, the app falls back to the next installed one. A line under the selection shows the active chain for the script language. Changes take effect the next time voice starts; changing the choice while voice runs restarts the voice session.

If a model decodes more slowly than real time on your computer (text would lag further and further behind), a warning appears after about 10 seconds with the recommendation to choose a smaller model (the threshold is a real-time factor above 0.9).

### Resync in the background

The switch **Resync in the background (Resync im Hintergrund)** (off by default) additionally loads Parakeet TDT 0.6B v3. It decodes finished speech segments in the background and re-anchors the cursor if the streaming model was wrong. Scrolling is never delayed by it. Segments are cut with Silero VAD when its model is installed; otherwise by audio level (RMS). The panel shows "Segmentation: Silero VAD active" or "level (RMS)", and "Resync model not installed, streaming only" if Parakeet is missing. It costs extra CPU and about 670 MB of disk. Background: ADR-006.

## Windows speech recognition (system fallback)

On Windows builds that include the `winspeech` feature (the official Windows release builds do), the app can use the speech recognition built into Windows (`Windows.Media.SpeechRecognition`). No download is needed (ADR-008).

- **Automatic fallback.** If no streaming model is installed, or none can be loaded, the app tries Windows speech recognition before it falls back to demo mode or shows a model error. The note under the model selection then reads like "Nemotron -> Windows speech recognition (system)".
- **Manual choice.** Select **Windows Spracherkennung (System)** (Windows speech recognition, system) in the model dropdown. It is offered only where available.
- **Requirements:**
  - The speech language pack is installed: Settings, Time and language, Language and region, your language, Language options, "Speech recognition" (`ms-settings:regionlanguage`). Choose the language under Time and language, Speech (`ms-settings:speech`).
  - **Online speech recognition** is turned on: Privacy and security, Speech (`ms-settings:privacy-speech`). Free dictation is a Microsoft cloud service, so **audio leaves your device** in this mode.
  - Microphone access for desktop apps is allowed (`ms-settings:privacy-microphone`).
- **Without online speech recognition**, the engine switches to a *script-word mode* (offline, if the grammar recognition for the language is installed). It recognizes only words from your script. That is enough for an emergency, but clearly less accurate.
- **Microphone handling.** Windows captures the **default microphone** itself. The microphone choice, gain and channel settings of the app do not affect recognition. The level meter and the silence watchdog still use the app's own capture.
- **Latency measurement.** The Diagnostics tab measures *speech onset to first partial*; "audio to partial" and "end to end" stay empty for this engine.

## Turning voice tracking on

- `V` (in the app) or `Ctrl+Shift+F6` (global) toggles voice.
- Turning it on sets the scroll mode to **Voice tracking**; turning it off sets it to **Constant**.
- You can also choose the mode in the command palette ("Scroll-Modus: ...") or in Settings, Scroll. **Voice-activated** uses the same pipeline but scrolls at the constant WPM rate only while speech is detected.
- In Settings, Voice and models, choose the **language** (German or English) and the **microphone**. The language selects the model and the text normalization. Any language code beginning with `en` counts as English; everything else is treated as German.

### Microphone, gain and channel

| Setting (German label) | Options |
|---|---|
| Microphone (Mikrofon) | System default, or a specific device. |
| Microphone gain (Mikrofon-Gain) | **Auto** (default): digital automatic gain control targeting about -20 dBFS speech level, up to +40 dB, adapting only above -75 dBFS, with a soft limiter against clipping. **Manual**: a fixed gain from -20 to +40 dB. Takes effect live, also while voice runs. |
| Microphone channel (Mikrofon-Kanal) | **Auto** (loudest channel, smoothed, with 3 dB hysteresis; default), **Mix all (Alle mischen)** (average), or channel 1 to 4. |
| Microphone test (Mikro testen) | Starts the capture with a level meter and shows the recognized text. Disabled while voice tracking is on. |
| When lost (Wenn verloren) | **Stop (Anhalten)** (default) or **Continue at constant speed (Konstant weiter)**. |

Microphone arrays often deliver several channels, one of which may be empty or out of phase. "Auto" avoids averaging in an empty channel.

### Microphone watchdog

If the microphone opens but delivers nothing, the app reports "Microphone delivers no data" or "only digital silence" after about 3 seconds (typically a Windows microphone privacy setting or a muted device). See [Troubleshooting](faq.md).

## Status indicator

The pill at the top shows the tracker state by color, shape and text (not by color alone). The talent never sees it.

| State | Label (German) | Appearance | Meaning |
|---|---|---|---|
| `idle` | Voice aus (voice off) | gray, circle outline | Voice is off. |
| `listening` | Hört zu... (listening) | blue, pulsing | Waiting for the first confident match. |
| `tracking` | Folgt (following), with WPM and a confidence bar | green, filled | The text follows your voice. |
| `hold` | Wartet (waiting) | orange, pause bars | Silence, an uncertain match, or a possible jump awaiting confirmation. The text stays put. |
| `lost` | Verloren - klicke ein Wort (lost, click a word) | red, triangle | You spoke for longer than `lostTimeoutMs` (default 4 s of speech) without anything matching the script. |
| `resync` | Springt... (jumping) | violet | A confirmed jump is being performed smoothly (about 300 ms). |

In Live mode, if the state stays `lost` for more than 4 seconds, a banner asks you to click the word you are at, or press `R`.

## Behavior in edge cases

These values are the tracker defaults in `crates/zt-align/src/tracker.rs`.

- **Slips, filler words, ad-libs:** fuzzy word matching and gap costs absorb them. Filler words such as "aeh", "aehm", "uh", "um", "hm" are dropped from what you say.
- **Repeating a passage (short jump back, up to 10 words):** accepted immediately.
- **Starting further ahead (more than 18 words):** the jump is executed only after **two** consistent recognitions (candidates within 4 words of each other count as the same jump). Until then the text stays in `hold`.
- **First word after a manual seek:** a near-cursor fast path lets the newest spoken word advance the cursor at once if it matches one of the next 2 script words (exact, or with edit distance 1 for words of at least 5 letters).
- **Lost:** what happens is set by "When lost": stop, or continue at constant speed. The app searches the whole script until it finds you again.

## Manual intervention

- **Click a word** in the live preview to trigger "Resync here": the cursor jumps there.
- `R` triggers resync at the current marker position.
- After a click, a paragraph jump (`PgUp`, `PgDn`), a click on the position bar or a stop, the app ignores your voice for `manualOverrideMs` (default 2 s) and then voice takes over again.

## Tips

- A headset or directional microphone close to your mouth improves recognition considerably.
- Rare words from your script (names, technical terms) are passed to the recognizer as hotwords if the engine supports them (Zipformer EN 20M does; Nemotron does not).
- Write numbers as digits ("2026"); the app compares them with the spoken form.

## Diagnostics tab and log export

Open Settings, **Diagnostics (Diagnose)**. It polls the app about every 500 ms and shows:

- **Engine:** a headline (active model, "Resync on", "no microphone", or the demo or error text); build version, operating system and architecture, whether the build has `sherpa-onnx` and `cpal`; the installed streaming models in fallback order and the language; the audio device with its native format (for example `48000 Hz · 1 ch · F32 -> 16 kHz mono`); frames per second and the tick rate (target about 60 Hz); the tracker state and confidence; and the last recognized text ("Erkannt").
- **Level:** a meter and the **Microphone test** button (not available while voice tracking runs). While voice runs it also shows the raw microphone level before gain, the current gain, and the level of each channel with the active one highlighted. If the raw level at speaking volume (90th percentile over the last 30 s) is below -50 dBFS, a "microphone very quiet" hint appears.
- **Latency (rolling, last 200 measurements):** p50 and p95 per measuring point (table below).
- **Last errors:** up to the 8 most recent errors and notices.
- **Log:** **Export log (Log exportieren)** saves one text file containing a JSON snapshot plus all log files, which you can attach to a bug report. The log folder path is shown, and clicking it copies the path.

The target is under 300 ms from spoken word to scroll start (p50), at most 1 s. This is a design target, not a measured guarantee.

| Measuring point | Measures |
|---|---|
| Speech onset to first hypothesis | From onset to the first partial result of an utterance (includes the model's chunk and look-ahead). Onset is a level at least 12 dB above the running noise-floor estimate for 60 ms; measurement restarts after 0.5 s of silence. |
| Audio chunk to hypothesis | From the capture time of a chunk to its partial result (queue plus decoding). |
| Hypothesis to cursor | Alignment cost in the tracker. |
| Audio to cursor (total) | From capture to the cursor event; the scroll animation starts at the next tick (at most 16 ms later) with a time constant of 150 ms. |

How often the cursor can move is mostly determined by the model's decoder cadence: Nemotron 320 ms delivers new words about every 0.3 s and 160 ms about every 0.2 s (CI measurement). Your machine's numbers are in the Diagnostics tab.

### Log files

Log files are named `zero-teleprompter.YYYY-MM-DD.log`, rotated daily, 7 files kept, in the app's log folder. The Diagnostics tab shows the exact path; the OS-specific default location is **unverified** here. For more detail start the app with the environment variable `ZT_LOG=debug`, or for example `ZT_LOG=info,zt::stt=debug` to log every partial hypothesis. The default filter is `info,zero_teleprompter_lib=debug,zt=debug`. See the [logging reference](../reference/logging.md).
