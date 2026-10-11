# Settings Reference

zero-prompter keeps three independent groups of persisted preferences:

| Group | Owner | File | Location |
|---|---|---|---|
| Core settings (`zt_core::Settings`, including the nested `marker` block) | Rust (`zt-core`) | `settings.json` | app config directory |
| UI preferences (`UiPrefs`) | Frontend (opaque JSON to Rust) | `ui-prefs.json` | app config directory |
| Voice model preferences (`VoiceModelPrefs`) | Rust (`src-tauri/src/models.rs`) | `voice-models.json` | app config directory |

"App config directory" is Tauri's `app_config_dir()` for the identifier `dev.zeroteleprompter.app`; its concrete path is OS-specific and is not verified here. Scripts are stored separately as `<app_data_dir>/scripts/<id>.md`, and models under `<app_data_dir>/models/`.

All writes use an atomic write helper (`persist::write_atomic`).

## 1. Core settings (`zt_core::Settings`)

Source: `crates/zt-core/src/settings.rs`.

### File format

```json
{ "version": 1, "settings": { "theme": "studioDark", "wpm": 140 } }
```

- Field names are camelCase.
- The struct is annotated `#[serde(default)]`, so any missing field takes its default. Old files without newer keys (typography, marker, microphone) still load.
- Unknown enum values (for example `"textAlign": "diagonal"`) are a deserialization error.
- A file whose `version` is greater than the supported version (`SETTINGS_VERSION = 1`) is ignored and defaults are used. No migrations exist yet.
- After loading, `Settings::sanitized(&Tokens)` clamps all values into the ranges below. Non-finite floats (NaN, infinity) fall back to a default instead of being clamped.
- The default for a fresh install is `Settings::from_tokens(&Tokens::embedded())`: it takes `fontSizePx`, `lineHeight`, `fontWeight` and `focusLine` from `docs/design/tokens.json` and everything else from the built-in base values.

### Fields

| Field | Type | Default | Range / valid values | Notes |
|---|---|---|---|---|
| `theme` | string | `"studioDark"` | any key of `tokens.json` -> `prompter` | Unknown id resets to `studioDark`. See [Design Tokens](tokens.md#prompter-themes). |
| `fontSizePx` | u16 | `72` (`type.prompter.sizeDefault`) | `24`-`300` (`type.prompter.sizeMin`/`sizeMax`) | Clamped against the token bounds, not hard-coded. |
| `lineHeight` | f32 | `1.4` (`type.prompter.lineHeight`) | `1.0`-`3.0` | Non-finite -> token default. Whether a theme's own `lineHeight` acts as a minimum is a frontend behavior and not verified here. |
| `fontWeight` | u16 | `500` (`type.prompter.weight`) | `100`-`900` | Bound to the `wght` axis of variable fonts; the frontend snaps static fonts to the nearest weight. |
| `letterSpacingEm` | f32 | `0.0` | `-0.05`-`0.3` | In em. `0` = theme default. Non-finite -> `0`. |
| `wordSpacingEm` | f32 | `0.0` | `-0.1`-`1.0` | In em. `0` = theme default. Non-finite -> `0`. |
| `paragraphSpacingEm` | f32 | `0.6` | `0.0`-`3.0` | Space after each paragraph, in em. Non-finite -> `0.6`. |
| `textAlign` | enum | `"left"` | `"left"`, `"center"`, `"justify"` | |
| `textTransform` | enum | `"none"` | `"none"`, `"uppercase"` | Display only; the script text is unchanged. |
| `focusLine` | f32 | `0.35` (`layout.focusLineDefault`) | `0.05`-`0.70` | Fraction of viewport height. Constants `FOCUS_LINE_MIN`/`FOCUS_LINE_MAX`. Non-finite -> token default (clamped). |
| `wpm` | u16 | `140` | `40`-`400` | Constant-speed rate in words per minute. Constants `WPM_MIN`/`WPM_MAX`. |
| `countdownSecs` | u8 | `3` | not clamped by `sanitized()` | Countdown before playback starts. |
| `lostPolicy` | enum | `"stop"` | `"stop"`, `"constantSpeed"` | What happens when voice tracking enters the `lost` phase. |
| `lostTimeoutMs` | u32 | `4000` | not clamped | Time in `lost` handling before the policy applies (exact semantics live in the controller; not re-verified here). |
| `manualOverrideMs` | u32 | `2000` | not clamped | Duration of the manual override after the operator jumps or resyncs manually. |
| `language` | string | `"de"` | free string (BCP-47 style, for example `"de"`, `"en-US"`) | Script language. Selects the voice model via its primary subtag (`de-DE`, `en_US` -> `de`, `en`). |
| `reducedMotion` | bool | `false` | | Disables decorative motion. |
| `hotkeys` | map string -> string | see below | not validated | Action id -> key chord. |
| `micGainAuto` | bool | `true` | | `true`: automatic gain control targeting about -20 dBFS. |
| `micGainDb` | f32 | `0.0` | `-20.0`-`40.0` | Used when `micGainAuto` is `false`. Non-finite -> `0`. |
| `micChannel` | string | `"auto"` | `"auto"`, `"mix"`, or a 1-based channel number (`"1"`, `"2"`, ...) | Multi-channel input to mono. `auto` = loudest channel, `mix` = average. Input is trimmed and lower-cased; invalid values (including `"0"`) become `"auto"`. |
| `marker` | object | see section 2 | | Reading-marker look. |

### Default hotkeys (`hotkeys`)

| Action id | Default chord |
|---|---|
| `togglePlay` | `Space` |
| `stop` | `Home` |
| `faster` | `ArrowUp` |
| `slower` | `ArrowDown` |
| `nextParagraph` | `PageDown` |
| `prevParagraph` | `PageUp` |
| `toggleMirror` | `M` |
| `toggleFullscreen` | `F` |
| `toggleVoice` | `V` |
| `resyncHere` | `R` |
| `commandPalette` | `Ctrl+K` |
| `shortcutOverlay` | `?` |

These are in-app chords handled by the frontend. System-wide shortcuts are registered separately by the Rust side (see [IPC](ipc.md#global-shortcuts)).

### Scroll mode

`ScrollMode` is not part of `Settings`; it is runtime state set through the `setScrollMode` command. Values: `manual`, `constant` (default), `voiceActivated`, `voiceTracking`.

## 2. Reading marker (`settings.marker`)

Source: `crates/zt-core/src/marker.rs`. The frontend renderer (`ReadingMarker.svelte`, `lib/marker.ts`) must use the same bounds. Also `#[serde(default)]`, camelCase.

| Field | Type | Default | Range / valid values | Notes |
|---|---|---|---|---|
| `style` | enum | `"line"` | `line`, `band`, `arrows`, `brackets`, `spotlight`, `gradient`, `underline`, `sidebar`, `none` | |
| `color` | string or null | `null` | `#RGB` or `#RRGGBB` (case-insensitive) | Normalized to lowercase `#rrggbb`; anything else becomes `null`. `null` = UI pref or theme marker color. |
| `opacity` | f32 | `1.0` | `0.1`-`1.0` | For `spotlight`/`gradient` it is the strength of the darkening. Non-finite -> `1.0`. |
| `size` | f32 | `1.0` | `0.5`-`3.0` | Thickness/height multiplier. Non-finite -> `1.0`. |
| `dimRead` | f32 or null | `null` | `0.0`-`0.9` | Dimming of already-read text. `null` = theme `readOpacity`. A non-finite value becomes `null`. |
| `dimUnread` | f32 | `0.0` | `0.0`-`0.9` | Dimming of not-yet-read text below the reading zone. |
| `glow` | f32 | `0.0` | `0.0`-`1.0` | Glow around the marker; `0` = off. |
| `blur` | f32 | `0.0` | `0.0`-`8.0` | Soft-focus blur in px outside the reading zone (spotlight/gradient). |
| `animation` | enum | `"glide"` | `none`, `glide`, `pulse` | Motion when the marker moves. |
| `underlineScope` | enum | `"word"` | `word`, `sentence` | What the `underline` style underlines. |

## 3. UI preferences (`ui-prefs.json`)

Source of truth: `UiPrefs` in `apps/desktop/src/lib/types.ts`. Rust treats this file as opaque JSON: `get_ui_prefs` returns it, `set_ui_prefs` stores it and broadcasts `prefs://changed` so the talent window stays in sync.

| Field | Type | Default | Notes |
|---|---|---|---|
| `fontFamily` | `FontChoice` | `"atkinson"` | Other `FontChoice` values are defined in `types.ts` (not enumerated here). |
| `customFont` | string | `""` | Installed system font name, used when `fontFamily = "custom"`. |
| `measureCh` | number | `32` | Text column width in `ch`. |
| `markerStyle` | `MarkerStyle` | `"line"` | UI-side marker style preference. |
| `dimRead` | boolean | `true` | |
| `readOpacity` | number or null | `null` | `null` = theme default. |
| `textColor` | string or null | `null` | `null` = theme default. |
| `bgColor` | string or null | `null` | `null` = theme default. |
| `markerColor` | string or null | `null` | `null` = theme default. |
| `uiTheme` | `"system"` \| `"light"` \| `"dark"` | `"system"` | Control-window appearance. |
| `onboarded` | boolean | `false` | |
| `lastScriptId` | string or null | `null` | Script reopened at start. |
| `inspectorOpen` | map string -> boolean | (see `DEFAULT_PREFS`) | Collapsed/open inspector sections. |
| `showFrameStats` | boolean | (see `DEFAULT_PREFS`) | Frame statistics overlay. |

The last two defaults are not shown in the excerpt inspected for this page; check `DEFAULT_PREFS` in `types.ts`.

## 4. Voice model preferences (`voice-models.json`)

Source: `VoiceModelPrefs` in `apps/desktop/src-tauri/src/models.rs`. camelCase, `#[serde(default)]`. A missing or unparsable file yields defaults.

| Field | Type | Default | Notes |
|---|---|---|---|
| `streaming` | map string -> string | `{}` | Primary language (`"de"`, `"en"`) -> preferred streaming model id. Absent = automatic (manifest `defaultRank` plus fallback chain). The special id `windows-speech` selects the Windows speech fallback where available. |
| `resync` | bool | `false` | Offline resync on/off. Off by default (tier L, about 670 MB, extra CPU). |
| `resyncModel` | string or null | `null` | Preferred resync model id; `null` = manifest default. |

Changing these through `voice_models_set` restarts a running voice session so the new engine takes effect immediately.

## 5. Environment variables

| Variable | Effect |
|---|---|
| `ZT_LOG` | `tracing` `EnvFilter` expression overriding the default log filter. See [Logging](logging.md). |
| `ZT_STT_DEBUG` | If set (any value), enables sherpa-onnx verbose output on stderr. |
| `ZT_SAFE_RENDER` | Linux only. If set and `WEBKIT_DISABLE_DMABUF_RENDERER` is unset, the app sets `WEBKIT_DISABLE_DMABUF_RENDERER=1` before any webview exists (workaround for NVIDIA/WebKitGTK rendering issues). |
