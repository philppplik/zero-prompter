# Logging and Diagnostics

Source: `apps/desktop/src-tauri/src/diag.rs` (plus the call sites listed below). Logging uses the `tracing` ecosystem (`tracing-subscriber`, `tracing-appender`).

## Log files

| Property | Value |
|---|---|
| Directory | Tauri `app_log_dir()` (OS-specific, not verified here). Created at startup. |
| File name | `zero-teleprompter.YYYY-MM-DD.log` |
| Rotation | Daily |
| Retention | 7 files (`max_log_files(7)`) |
| Writer | Non-blocking (`tracing_appender::non_blocking`); the worker guard is kept for the process lifetime. |
| Formatting | Plain text (`fmt` layer, no ANSI colors), thread names included in the file. |
| Also written to | stderr (same filter, no ANSI). |

The exact line layout is the default `tracing_subscriber::fmt` format (timestamp, level, thread name, target, message and fields). It is not customized in the source.

## Filtering

Environment variable `ZT_LOG` accepts `EnvFilter` syntax (for example `ZT_LOG=info,zt::stt=debug`). If unset or invalid, the default is:

```
info,zero_teleprompter_lib=debug,zt=debug
```

`ZT_STT_DEBUG` (any value) additionally switches on sherpa-onnx's own verbose output on stderr.

## Startup line

At startup one `info` event records `version`, `os`, `arch`, `cpal`, `sherpa`, `download` (compile-time features) and `log_dir`, with the message `zero-teleprompter starting`.

## Tracing targets

| Target | Typical events (level) |
|---|---|
| `zt::error` | `pipeline error` with fields `source`, `message` (error). Emitted by `report_error`. |
| `zt::warn` | `pipeline warning` with `source`, `message` (warn). Emitted by `record_warning`. |
| `zt::panic` | `panic` with `thread` and panic info (error). Installed panic hook; the previous hook still runs. |
| `zt::frontend` | Messages forwarded from the webview through `diag_log_frontend` (error / warn / info). |
| `zt::models` | Manifest warnings, voice model prefs changes, download start, helper installs, "installed UNVERIFIED (no manifest hash)" with `sha256`, download complete or failed. |
| `zt::engine` | Streaming model plan (`language`, `preferred`, `chain`, `language_fallback`, `root`), recognizer creation (`encoder`, `tokens`, `decoding`, `threads`, `language`), `recognizer ready` with `ms`, `recognizer failed`, engine selection, fallback to Windows speech, demo mode notice. Also sherpa preflight/stage lines via the config log sink. |
| `zt::voice` | Voice start (`tracking`, `lang`, ...) and `voice stop` with `secs`. |
| `zt::audio` | Stream description and gain, `mic gain changed`, `mic channel mode changed`, per-interval `frames_per_s`, `rms_dbfs`, `raw_dbfs` (debug). |
| `zt::stt` | Partial/final hypotheses with `t_ms` (debug/info), onset latency, hotword application, decode real-time factor `rtf` (debug). |
| `zt::vad` | `speech start` / `speech end` (debug). |
| `zt::tracker` | Cursor updates (debug) and voice state transitions `from` -> `to` (info). |
| `zt::resync` | `resync` with `age_ms`, `outcome`, `text` (info). |
| `zt::hotkeys` | Global shortcut registration problems (warn). |
| `zt::persist` | `saving settings failed` (error). |

Default filter note: `zt=debug` enables debug output for every `zt::*` target, so the default log is fairly verbose for `zt::audio` and `zt::stt`.

## Error path

`diag::report_error(app, source, message)` is the single error path. It (1) logs at error level on `zt::error`, (2) appends an `ErrorEntry { atMs, source, message }` to an in-memory ring buffer limited to 50 entries (`MAX_ERRORS`, oldest dropped), and (3) emits the `diag://error` event for a UI toast. `record_warning` logs on `zt::warn` and also pushes into the ring buffer, with `source` suffixed by ` (Hinweis)` (the suffix is German in the current source). Typical `source` values: `audio`, `stt`, `models`, `voice-thread`.

## Diagnostics snapshot

`diag_snapshot` returns `DiagReport` (camelCase): all `Snapshot` fields flattened plus `selection` (installed streaming models usable for the script language, or null on error).

| Field | Description |
|---|---|
| `version`, `os`, `arch` | Build and platform. |
| `sherpa`, `cpal` | Compile-time features. |
| `running`, `tracking` | Voice pipeline state. |
| `engine` | `EngineInfo` (see [IPC](ipc.md#events-rust-to-all-windows)) or null. |
| `audio` | Description of the active audio stream, or null. |
| `levelDb`, `rawDb`, `gainDb` | Level after gain, level before gain (`rawLevelDb`), applied gain. The JSON key names follow the struct: `levelDb`, `rawLevelDb`, `gainDb`. |
| `channelDb`, `activeChannel` | Smoothed per-channel levels (dBFS) and the channel used for mono (null = mix). |
| `framesPerS`, `tickHz` | Measured audio frame rate and tick rate (target about 60 Hz). |
| `voiceState`, `confidence`, `lastPartial` | Last tracker state, confidence, last partial transcript. |
| `latency` | Four `LatencySummary` objects, see below. |
| `errors` | The ring buffer contents, oldest first. |
| `logDir` | Log directory path or null. |

`LatencySummary` fields: `count` (samples in window), `total` (samples ever recorded), `p50Ms`, `p95Ms`, `maxMs`, `lastMs` (each may be null).

| Latency | Measured from -> to |
|---|---|
| `audioToPartial` | Capture time of an audio chunk -> partial hypothesis emitted (queue, resample, decode; excludes the model's internal look-ahead). |
| `onsetToPartial` | First voiced chunk of an utterance (RMS onset) -> first partial (includes the model's chunk look-ahead). |
| `hypToCursor` | Hypothesis received -> tracker cursor moved (alignment cost). |
| `endToEnd` | Capture time of the chunk -> `voice://word` emitted (end to end inside Rust). |

## Log export

`diag_export_log(dest)` writes one file containing the header line `# zero-teleprompter diagnostics snapshot`, the pretty-printed JSON snapshot, then every log file (oldest first), each preceded by `# ---- <path>`. It returns the byte count and logs `log exported`. It fails with an error if no log directory is known.
