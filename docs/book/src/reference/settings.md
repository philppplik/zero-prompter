# Settings

## Kern-Settings (`zt_core::Settings`, Datei `settings.json`)

Dateiformat: `{ "version": 1, "settings": { … } }`. Felder in camelCase. Fehlende Felder bekommen den Standardwert (`#[serde(default)]`). Nach dem Laden begrenzt `sanitized()` die Werte auf gültige Bereiche.

| Feld | Typ | Standard | Gültigkeit / Wirkung |
|---|---|---|---|
| `theme` | string | `"studioDark"` | muss in `tokens.json → prompter` existieren, sonst gilt der Standard |
| `fontSizePx` | u16 | 72 (`type.prompter.sizeDefault`) | 24–300 |
| `lineHeight` | f32 | 1,4 | 1,0–3,0 |
| `focusLine` | f32 | 0,35 (`layout.focusLineDefault`) | 0,1–0,9 (Anteil der Fensterhöhe) |
| `wpm` | u16 | 140 | 40–400 (`WPM_MIN`/`WPM_MAX`) |
| `countdownSecs` | u8 | 3 | 0 = sofort starten |
| `lostPolicy` | `"stop"` \| `"constantSpeed"` | `"stop"` | Verhalten bei `lost` im Modus Voice-Tracking |
| `lostTimeoutMs` | u32 | 4000 | Sprechzeit ohne Treffer bis `lost` (geht an `TrackerConfig`) |
| `manualOverrideMs` | u32 | 2000 | Zeit nach einem manuellen Seek, in der die Stimme ignoriert wird |
| `language` | string | `"de"` | beginnt mit `en` → Englisch, sonst Deutsch. Wirkt auf Normalisierung und Modellwahl |
| `reducedMotion` | bool | false | |
| `hotkeys` | map Aktion → Tastenkombination | siehe [Tastenkürzel](../user/shortcuts.md) | Format `Ctrl+K`, `Shift+M`, `ArrowUp`, `Space`, `?` |

Änderungen laufen über `Command::ReplaceSettings`, `SetTheme` oder `SetFontSize`. Nach jedem Settings-Patch speichert die App die Datei atomar.

## UI-Präferenzen (`UiPrefs`, Datei `ui-prefs.json`)

Nur für das Frontend. Die App speichert die Datei per `set_ui_prefs` und sendet sie als `prefs://changed` an alle Fenster.

| Feld | Typ | Standard |
|---|---|---|
| `fontFamily` | `atkinson` \| `inter` \| `system` \| `serif` \| `mono` | `atkinson` |
| `measureCh` | number (16–60) | 32 |
| `markerStyle` | `line` \| `band` | `line` |
| `dimRead` | bool | true |
| `readOpacity` | number \| null (null = Theme-Wert) | null |
| `textColor`, `bgColor`, `markerColor` | string \| null | null |
| `uiTheme` | `system` \| `light` \| `dark` | `system` |
| `onboarded` | bool | false |
| `lastScriptId` | string \| null | null |
| `inspectorOpen` | Abschnitt → bool | `appearance`, `scroll` offen |
| `showFrameStats` | bool | false |

## Umgebungsvariablen

| Variable | Wirkung |
|---|---|
| `ZT_SAFE_RENDER` | nur Linux: setzt `WEBKIT_DISABLE_DMABUF_RENDERER=1`, sofern nicht schon gesetzt |
| `PROPTEST_CASES` | nur Tests: Anzahl der proptest-Fälle |
