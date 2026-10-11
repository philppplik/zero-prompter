# Model Manifest

Sources: `crates/zt-models/src/lib.rs`, `apps/desktop/src-tauri/resources/models.json`, `apps/desktop/src-tauri/src/models.rs`.

The manifest lists every speech model the app knows, where to download it, and how to verify it. The `zt-models` crate has no network code: downloads go through the `Downloader` trait, implemented in the app (`ureq`, HTTPS only, resumable with HTTP `Range`).

## Lookup order

1. `<app_data>/models/manifest.json` (user-supplied or updated manifest), if readable.
2. Otherwise the bundled `resources/models.json` (embedded with `include_str!`).

The manifest is validated on every load. Warnings (see below) are logged at `warn` level under target `zt::models`.

## Schema (`schemaVersion` 2)

`Manifest::SCHEMA_VERSION` is `2`; versions `1` and `2` are accepted, anything else is rejected. Version 2 added `modelType`, `roles`, `defaultRank` and optional `sha256`. Keys starting with `_` (for example `_note` in the bundled file) are not part of the schema; unknown keys are ignored by serde.

```json
{
  "schemaVersion": 2,
  "models": [ { "...": "ModelEntry" } ]
}
```

### `ModelEntry`

| Field | Type | Required | Description |
|---|---|---|---|
| `id` | string | yes | Unique; only `[A-Za-z0-9._-]`, non-empty. Also the install directory name. |
| `name` | string | yes | Display name (the bundled names are still partly German). |
| `languages` | string[] | yes | Primary language subtags, for example `["de", "en"]`. |
| `tier` | `"s"` \| `"m"` \| `"l"` | yes | Size/quality tier. |
| `engine` | `"sherpaOnlineTransducer"` \| `"sherpaOfflineTransducer"` \| `"sherpaVad"` | yes | Engine family. |
| `modelType` | `"online_transducer"` \| `"online_nemo_transducer"` \| `"offline_nemo_transducer"` | no | Explicit decoder type. If absent: `sherpaOnlineTransducer` -> `online_transducer`, `sherpaOfflineTransducer` -> `offline_nemo_transducer`, `sherpaVad` -> none. |
| `roles` | (`"streaming"` \| `"resync"`)[] | no | Pipeline roles. If empty: online types -> `streaming`, offline type -> `resync`, VAD -> none. |
| `defaultRank` | u32 | no | Lower = preferred. Absent = never auto-picked ahead of ranked models (sorted last, in manifest order). |
| `license` | string | yes | SPDX id or `"unknown"`. |
| `attribution` | string | no | Attribution text. Must be shown when `license` starts with `CC-BY` or `OpenMDW` (`requiresAttribution`). |
| `files` | `ModelFile[]` | yes | Files of the model. |

Model types: `online_transducer` is a streaming Zipformer transducer (sherpa `OnlineRecognizer`); `online_nemo_transducer` is a NeMo cache-aware streaming transducer (Nemotron 3.5) decoded by the `OnlineRecognizer` and taking a per-stream `language` option (`"de"`, `"en"`, or `"auto"`); `offline_nemo_transducer` is NeMo Parakeet TDT decoded by the `OfflineRecognizer` with sherpa `model_type = "nemo_transducer"`.

### `ModelFile`

| Field | Type | Required | Description |
|---|---|---|---|
| `path` | string | yes | Relative path inside the model directory. No leading `/`, no `\`, no `:`, no empty, `.` or `..` segments (zip-slip guard). |
| `url` | string | yes | Must start with `https://`. |
| `sha256` | string | no | 64 hex characters (stored lower-case). `""`, `null` or absent means unverified. |
| `size` | u64 | yes | Exact byte size, must be greater than 0. |
| `role` | string | no | Engine role of the file: `encoder`, `decoder`, `joiner`, `tokens`, `vad`. Used to find real file names. |

### Validation rules

`Manifest::validate` fails the whole manifest on: unsupported `schemaVersion`; duplicate or malformed model id; unsafe file path; malformed `sha256` (wrong length or non-hex); `size` of 0; non-HTTPS URL. A missing `sha256` is not an error but a warning, because Hugging Face exposes SHA-256 only for LFS objects. Such files (for example small `tokens.txt` files) are checked by exact size plus HTTPS, and the computed hash is reported after download.

Warning format: `<model id>/<path>: no sha256 in manifest (unverified, size-checked only)`.

## Install layout and integrity

- Layout: `<app_data>/models/<model-id>/<file.path>`.
- Downloads go to `<file>.part`, are verified (size first, then SHA-256 if present) and atomically renamed. On mismatch the `.part` file is deleted. Existing valid files are skipped.
- Status: `ModelStore::status` hashes every file; `quick_status` (used at startup and by `models_list`) checks sizes only. States: `NotInstalled`, `Incomplete(missing paths)`, `Installed`.
- For unverified files the computed hashes are written to `<model dir>/unverified-sha256.json` (a `path -> sha256` map) and shown in the UI so they can be pinned in the manifest.
- Installing any speech model also installs the VAD helper models (`engine: sherpaVad`, i.e. Silero VAD) if missing. A helper failure is reported via `diag::report_error` but does not fail the main install.
- After a successful install, a running voice session is restarted to pick up the new model.
- A legacy manually copied `<models>/silero_vad.onnx` is still honored when no `sherpaVad` manifest entry is installed.

## Model selection

- `Manifest::candidates(lang, role)`: all models whose `languages` contain the primary subtag of `lang` (`de-DE`, `en_US`, `DE` -> `de`/`en`) and that have the role, sorted by `defaultRank` (unranked last) then manifest order.
- `fallback_chain`: the user's preferred model first (if it fits), then the ranked candidates.
- `streaming_plan`: the chain filtered to usable (installed, online) models. If none exists for the language, installed streaming models of other languages are used and a note is shown (`language_fallback`); if there are none at all the app runs the demo engine.
- Engine construction (`build_engine`, `sherpa` feature) tries chain entries in order and skips those that fail to load (`zt_stt::build_with_fallback`); skipped models and fallbacks appear in `EngineInfo.notes`.
- Resync is added when `VoiceModelPrefs.resync` is true and an installed `offline_nemo_transducer` model exists; otherwise the engine is streaming-only and a note is added. Segmentation uses Silero VAD if available, else RMS segmentation. Resync problems never disable streaming.
- On Windows builds with Windows speech support, the id `windows-speech` is appended to the streaming chain as the last fallback (or used exclusively if it is the user's preference).

## Bundled models (`resources/models.json`)

Sizes are the sums of the `size` fields of each entry. All entries pin their URLs (Hugging Face commit hashes, or a `k2-fsa/sherpa-onnx` release asset) and carry a SHA-256 for every file.

| Id | Languages | Tier | Type / role | `defaultRank` | License | Total size (bytes) |
|---|---|---|---|---|---|---|
| `silero-vad` | de, en | s | `sherpaVad` helper | 0 | MIT | 643,854 |
| `nemotron-3.5-streaming-0.6b-320ms-int8` | de, en | l | `online_nemo_transducer`, streaming | 0 | OpenMDW-1.1 | 682,215,471 |
| `nemotron-3.5-streaming-0.6b-160ms-int8` | de, en | l | `online_nemo_transducer`, streaming | none | OpenMDW-1.1 | 682,215,471 |
| `nemotron-3.5-streaming-0.6b-80ms-int8` | de, en | l | `online_nemo_transducer`, streaming | none | OpenMDW-1.1 | 682,215,469 |
| `zipformer-de-kroko-2025-08-06` | de | s | `online_transducer`, streaming | 1 | CC-BY-SA-4.0 | 71,051,469 |
| `zipformer-en-kroko-2025-08-06` | en | s | `online_transducer`, streaming | 1 | CC-BY-SA-4.0 | 71,053,214 |
| `zipformer-en-20m-2023-02-17` | en | s | `online_transducer`, streaming | 2 | Apache-2.0 | 43,649,301 |
| `parakeet-tdt-0.6b-v3-int8` | de, en | l | `offline_nemo_transducer`, resync | 0 | CC-BY-4.0 | 670,478,772 |

The 320 ms Nemotron variant is the default streaming model (rank 0) and is labeled "recommended" in the manifest; the 160 ms and 80 ms variants react faster but need a stronger CPU, and are not auto-selected. Note: the Kroko attribution text in the manifest still contains a `TODO: confirm exact CC-BY-SA version`. See `THIRD_PARTY_MODELS.md` in the repository for license notices.

## `ModelRow` (IPC payload)

Returned by `models_list` (camelCase):

| Field | Description |
|---|---|
| `entry` | The full `ModelEntry`. |
| `status` | `"installed"`, `"notInstalled"` or `"incomplete"` (size check). |
| `missing` | Relative paths that are missing or have a wrong size (when incomplete). |
| `totalSize` | Sum of file sizes. |
| `requiresAttribution` | `license` starts with `CC-BY` or `OpenMDW`. |
| `modelType` | Effective model type (explicit or derived), or null. |
| `roles` | Effective roles. |
| `unverified` | `[path, sha256 or null][]` for files without a manifest hash; the second element is the locally computed hash if known. |

## Errors (`ModelError`)

`Io { path, source }`, `Manifest` (JSON parse), `Invalid` (validation), `Checksum { path, expected, actual }`, `Size { path, expected, actual }`, `UnknownModel(id)`, `Download(message)`.
