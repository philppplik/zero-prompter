# FAQ and troubleshooting

## General

**Does the app need internet access?**
No. The only network access in normal use is downloading a speech model, which you start yourself. The Tauri content security policy otherwise allows only the internal IPC channel. The one exception is the optional Windows system speech fallback, which can use a Microsoft online service depending on a Windows setting (see [Voice tracking](voice-tracking.md#windows-speech-recognition-system-fallback)).

**Is my voice recorded or uploaded?**
No. Audio flows only through memory (microphone, resampler, speech recognizer) and is not stored. The log files can contain recognized text, because the `zt::stt` target logs partial and final hypotheses (partials at debug level, which the default filter enables). Review a log, or an exported log, before attaching it to a public bug report. This is based on the logging reference, not on a privacy review.

**Which languages are supported?**
German and English, for text normalization and for choosing the model. Any language code beginning with `en` counts as English; everything else is treated as German.

**Why is the interface in German?**
In this version all interface labels are German and the code contains no language switch. This manual gives the English meaning and the German label in parentheses.

## Problems

### The app does not start on Windows (VCRUNTIME140.dll was not found)

Earlier builds failed on fresh Windows systems (seen on Windows 11 ARM64) with "VCRUNTIME140.dll was not found". The MSVC runtime is now linked statically (`.cargo/config.toml`, `+crt-static`), and CI and the release workflow check with `dumpbin /dependents` (`scripts/check-crt.ps1`) that the executable imports no `VCRUNTIME*` or `MSVCP*` DLL. If you see this error you are probably running an old build; install a current one. If you build from source with your own `RUSTFLAGS` environment variable, note that it overrides the `.cargo/config.toml` setting and drops the static runtime.

### Voice moves the text although I say nothing ("Demo")

You are in **demo mode**: no streaming model is installed (Parakeet and Silero VAD alone are not enough), or the build contains no speech recognition. Click **Demo - load model (Demo · Modell laden)** next to the voice status and download "NVIDIA Nemotron 3.5 ASR Streaming 0.6B int8, 320 ms" (German and English; English only and small: "Streaming Zipformer EN 20M"). On Windows, "Windows Spracherkennung (System)" also works without a download. A running voice session picks the model up after the download. See [Voice tracking](voice-tracking.md#which-engine-runs).

### "Model error" or "speech model could not be loaded" (Modellfehler)

A model is installed but cannot be opened (damaged or missing file, not enough memory). The message names the file. Remove the model in Settings, Voice and models, and download it again. Details are in the Diagnostics tab and in the log. On Windows, the app tries the Windows speech fallback first when a model cannot be loaded.

### Log line "recognizer failed ... failed to create OnlineRecognizer"

Early test builds failed this way right after startup (about 1 ms): the adapter always set the modeling unit `bpe`, but none of the models ships a `bpe.vocab` file, and sherpa-onnx rejects that configuration. This is fixed. The app now:

1. checks all model files before starting (names and sizes appear as `preflight ok` in the log; missing files are named one by one),
2. generates the vocabulary file `zt-generated-bpe.vocab` from `tokens.txt` where hotwords need it, and
3. tries the stages `beam+hotwords`, then `beam`, then `greedy` in order. Each stage is logged as `stage ... ok` or `stage ... failed`. If every stage fails, the error message lists the reason for each stage.

If you still see the error: remove and re-download the model, and check the log for the `preflight` and `stage` lines. To see sherpa-onnx's own output, start the app from a console with the environment variable `ZT_STT_DEBUG=1` (the C++ library writes only to stderr). Paths with non-ASCII characters (such as umlauts) can cause problems on Windows; the log warns in that case.

### "Microphone delivers no data" or "only digital silence"

The device opened but delivers nothing. On Windows this is usually Settings, Privacy and security, Microphone, "Let desktop apps access your microphone" being off, or the microphone being muted. On macOS: System Settings, Privacy and Security, Microphone. Afterwards run the microphone test again.

### "Microphone very quiet (below -50 dBFS while speaking)"

The microphone delivers a signal, but far too little for speech recognition. A typical case is Surface or Snapdragon devices with a "Microphone Array ... Static Endpoints" device, with levels of -85 to -100 dBFS in pauses and hardly above -45 dBFS while speaking.

1. **Windows input volume.** Settings, System, Sound, Input, select the microphone, and set the **input volume** to 80 to 100 %. Some drivers also offer "audio enhancements" or "voice focus" there; try them.
2. **Microphone gain.** In Settings, Voice and models, keep **Microphone gain (Mikrofon-Gain)** on **Auto**. The app then amplifies digitally (up to +40 dB) toward about -20 dBFS speech level, and a limiter prevents clipping. **Manual** sets a fixed gain from -20 to +40 dB.
3. **Microphone channel.** Microphone arrays deliver several channels, and one may be empty or out of phase. **Auto** (default) takes the loudest channel, **Mix all** averages (which can cost level on arrays), and **Channel N** forces one channel. The Diagnostics tab shows the level of each channel and the chosen one.
4. In the Diagnostics tab, "Microphone raw" should be above -50 dBFS while you speak. Without a Windows-side change, digital gain also raises the noise.

### Speech does not synchronize although a model is loaded

1. Open Settings, **Diagnostics** and start voice.
2. "Engine" must show the model name, "Audio" a device, and the level should be clearly above -40 dBFS while you speak.
3. "Recognized (Erkannt)" shows what the recognizer hears. If it does not match your script, the language setting is probably wrong (Settings, Voice and models, Language).
4. If it is still unclear, use **Export log** and attach the file to a bug report.

### The text stays on "Waiting" (hold)

The app is not sure enough, or it is waiting for the second confirmation of a jump. Read a few more words, or click the word you are at (or press `R`).

### "Lost" appears often

- Check the level in the microphone test. It should be clearly above -40 dBFS while you speak.
- Set the correct language.
- Move closer to the microphone or use a headset.
- Set **When lost** to **Continue at constant speed** if you prefer that over stopping.
- Consider a different model. Nemotron 320 ms updates the cursor about every 0.3 s; 160 ms about every 0.2 s.

### A warning says the speech model is slower than real time

The streaming model needs more compute than your CPU provides (the warning appears when the real-time factor stays above 0.9 for about 10 seconds). Pick a smaller model: Nemotron 320 ms instead of 160 or 80 ms, or (English) Zipformer 20M. Also switch off background resync, which adds CPU load.

### No microphone found

- macOS: System Settings, Privacy, Microphone, allow zero-teleprompter.
- Windows: Settings, Privacy, Microphone, allow desktop apps access.
- Linux: check the ALSA or PipeWire device.

Then choose the device in Settings, Voice and models.

### Windows speech recognition does not work

See the requirements in [Voice tracking](voice-tracking.md#windows-speech-recognition-system-fallback): speech language pack installed, online speech recognition on (otherwise only the script-word mode is available) and microphone access allowed for desktop apps. The app shows specific hints for a missing language pack, for online speech recognition being off (HRESULT `0x80045509`) and for a denied microphone, each with the matching `ms-settings:` page. Remember that Windows records the default microphone itself, so the app's microphone choice does not apply.

### Global hotkeys do not work

Another app already uses the combination (the log then says "global shortcut unavailable", target `zt::hotkeys`), or you are on Wayland. The in-app keys still work while the window has focus.

### Linux with NVIDIA: stutter or a black window

Start the app with `ZT_SAFE_RENDER=1` (sets `WEBKIT_DISABLE_DMABUF_RENDERER=1`).

### Import fails

Supported: `.txt`, `.md`, `.markdown` and `.text`, up to 8 MB. `.docx` is not supported yet (planned for v0.2). If special characters look wrong, save the file as UTF-8.

### The script cannot be edited

While the text is playing it is locked. Pause or stop, then press `Ctrl+E`.

### The talent window is gone

Closing it only hides it. Use the **Output** button, or the command palette entry "Talent-Fenster öffnen". If you close the control window, the whole app quits.

### The talent window does not appear on the chosen monitor

The app moves the window to the target monitor (offset by 40 px) and then turns on fullscreen. Some window managers ignore this. Move the window by hand, then press `F`.

### Where are the logs?

In the app's log folder. The Diagnostics tab shows the path and can copy it. Files are named `zero-teleprompter.YYYY-MM-DD.log`, kept for 7 days. With `ZT_LOG=debug`, every hypothesis is logged with a timestamp and latency. Use **Export log** in the Diagnostics tab to save everything, plus a JSON snapshot, into a single file.

## Environment variables

| Variable | Effect |
|---|---|
| `ZT_LOG` | Log filter in `tracing` `EnvFilter` syntax, for example `ZT_LOG=debug`. |
| `ZT_STT_DEBUG` | Any value turns on sherpa-onnx's verbose output on stderr. |
| `ZT_SAFE_RENDER` | Linux only. Workaround for rendering problems with NVIDIA and WebKitGTK. |

## Known limitations of v0.1

- No remote control, no overlay mode, no custom themes, no auto-updater, no `.docx` import. See [Project status](../status.md).
- Installers are not signed by default, so expect operating system warnings. See [Installation](installation.md#unsigned-installers).
- Global hotkeys may not work under Wayland.
- Kroko models were removed for license reasons (`docs/legal/LEGAL.md`).
