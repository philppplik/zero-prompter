# Design Tokens

Source: `docs/design/tokens.json` (Design Tokens Community Group format; `meta.version` `1.0.0`, `meta.date` `2026-10-10`, contrast standard WCAG 2.2). This file is the single source of truth for Rust and the UI:

- Rust embeds it at compile time (`zt_core::settings::TOKENS_JSON`, via `include_str!`). It reads only `state`, `prompter`, `type.prompter` and `layout.focusLineDefault`; unknown keys are ignored. A unit test guards that the file parses. A leading UTF-8 BOM is tolerated.
- The frontend imports it in `apps/desktop/src/lib/tokens.ts` (themes, UI colors, CSS variables). `app.css` is the stylesheet that consumes the resulting variables.

Rationale and visual design: `docs/design/DESIGN.md`.

## Groups

| Key | Content |
|---|---|
| `meta` | `name`, `version`, `date`, `contrastStandard` |
| `ui.light`, `ui.dark` | Control-window colors (below). `ui.$source` records the origin: the tweakcn "Minimal Neutral" theme, converted from oklch to sRGB hex. |
| `state.<phase>` | Voice-state indicator: `color`, `shape`, `label` |
| `prompter.<themeId>` | Talent-window themes |
| `type` | Font stacks, UI type scale, prompter type scale |
| `space` | Spacing scale |
| `radius` | Corner radii |
| `elevation` | Shadows |
| `motion` | Durations, easings, reduced-motion behavior |
| `layout` | Fixed UI dimensions |
| `focus` | Focus ring |

## Control-window colors (`ui`)

| Token | Light | Dark |
|---|---|---|
| `bg` | `#FFFFFF` | `#171717` |
| `surface` | `#FDFDFD` | `#0E0E0E` |
| `surfaceRaised` | `#FFFFFF` | `#1F1F1F` |
| `surfaceSunken` | `#F5F5F5` | `#262626` |
| `border` | `#EEEEEE` | `#262626` |
| `borderStrong` | `#D4D4D4` | `#404040` |
| `text` | `#0A0A0A` | `#FAFAFA` |
| `textMuted` | `#737373` | `#A3A3A3` |
| `textOnAccent` | `#FAFAFA` | `#171717` |
| `accent` | `#171717` | `#E5E5E5` |
| `accentHover` | `#404040` | `#FFFFFF` |
| `accentSubtle` | `#EEEEEE` | `#404040` |
| `focusRing` | `#A3A3A3` | `#737373` |
| `overlay` | `rgba(10,10,10,0.28)` | `rgba(0,0,0,0.55)` |
| `cardFrom` | `#FFFFFF` | `#1F1F1F` |
| `cardTo` | `#F4F4F5` | `#121212` |
| `danger` | `#E7000B` | `#FF6467` |

## Voice states (`state`)

The six keys map one-to-one to `zt_core::VoicePhase` (`Idle`, `Listening`, `Tracking`, `Hold`, `Lost`, `Resync`). `Tokens` requires all keys of the map to deserialize; a unit test asserts exactly these six. Each state has a color and a shape so that state is never conveyed by color alone.

| Phase | Color | Shape | Label (as stored, German) |
|---|---|---|---|
| `idle` | `#8A8F98` | `circle-outline` | "Voice aus" |
| `listening` | `#4CC2FF` | `circle-pulse` | "Hört zu" |
| `tracking` | `#3DD68C` | `circle-filled` | "Folgt" |
| `hold` | `#FFB224` | `pause-bars` | "Wartet" |
| `lost` | `#F2555A` | `triangle` | "Verloren" |
| `resync` | `#8E7BFF` | `arc-rotate` | "Springt" |

Note: the `label` strings in `tokens.json` are still German. Localizing them is not part of this documentation task.

## Prompter themes

`prompter.<id>` entries deserialize into `PrompterTheme` (required: `bg`, `text`, `readOpacity`, `marker`, `wordHighlight`, `direction`, `contrast`, `wcag`; optional: `letterSpacing`, `wordSpacing`, `lineHeight`). The default theme is `studioDark` (`ThemeSet::DEFAULT`). JSON order determines the theme shortcuts `Ctrl+1` to `Ctrl+6` (as documented previously; the binding itself lives in the frontend and was not re-verified).

| Id | `bg` | `text` | `marker` | `wordHighlight` | `readOpacity` | `direction` | `contrast` | `wcag` |
|---|---|---|---|---|---|---|---|---|
| `studioDark` | `#0E0F12` | `#F4F1EA` | `#FFB224` | `rgba(255,178,36,0.22)` | 0.40 | `#8A8F98` | 16.99 | AAA |
| `broadcastClassic` | `#000000` | `#FFFFFF` | `#FFE14D` | `rgba(255,225,77,0.25)` | 0.40 | `#9AA0A6` | 21.0 | AAA |
| `daylight` | `#FAF8F3` | `#1A1A1A` | `#C2410C` | `rgba(194,65,12,0.14)` | 0.45 | `#5C6370` | 16.40 | AAA |
| `highContrast` | `#000000` | `#FFE14D` | `#FFFFFF` | `rgba(255,255,255,0.30)` | 0.55 | `#FFFFFF` | 16.13 | AAA |
| `lowLight` | `#120D08` | `#C9B79A` | `#B5542C` | `rgba(181,84,44,0.25)` | 0.45 | `#7D6F5C` | 9.87 | AAA |
| `dyslexia` | `#FDF6E3` | `#1F2A44` | `#2563EB` | `rgba(37,99,235,0.14)` | 0.45 | `#5C6370` | 13.22 | AAA |

`dyslexia` additionally sets `letterSpacing: "0.04em"`, `wordSpacing: "0.16em"`, `lineHeight: 1.6`.

## Typography (`type`)

| Token | Value |
|---|---|
| `fontUi` | `'DM Sans Variable', 'DM Sans', ui-sans-serif, system-ui, sans-serif` |
| `fontPrompter` | `'Atkinson Hyperlegible Next', Inter, system-ui, sans-serif` |
| `fontMono` | `'Geist Mono Variable', 'Geist Mono', ui-monospace, monospace` |

UI scale (`type.scale.*`, px):

| Name | size | line | weight |
|---|---|---|---|
| `caption` | 11 | 16 | 500 |
| `footnote` | 12 | 16 | 400 |
| `body` | 13 | 20 | 400 |
| `bodyStrong` | 13 | 20 | 600 |
| `callout` | 15 | 22 | 500 |
| `title3` | 17 | 24 | 600 |
| `title2` | 22 | 28 | 600 |
| `title1` | 28 | 34 | 700 |
| `display` | 96 | 96 | 700 |

Prompter scale (`type.prompter`, deserialized into `PrompterTypeScale`): `sizeMin` 24, `sizeDefault` 72, `sizeMax` 300, `lineHeight` 1.4, `measureCh` 32, `weight` 500.

## Space, radius, elevation

Spacing (`space`, px): `0`:0, `1`:2, `2`:4, `3`:8, `4`:12, `5`:16, `6`:20, `7`:24, `8`:32, `9`:40, `10`:48, `11`:64.

Radius (px): `xs` 4, `sm` 8, `md` 10, `lg` 16, `xl` 20, `card` 24, `pill` 999.

Elevation (`elevation.0` to `elevation.4`): `0` is `none`; `1` to `4` are CSS `box-shadow` strings with increasing blur (see the JSON for exact values).

## Motion

| Duration token | ms |
|---|---|
| `instant` | 0 |
| `fast` | 120 |
| `base` | 200 |
| `slow` | 300 |
| `panel` | 260 |
| `countdown` | 1000 |
| `resync` | 300 |
| `playEase` | 600 |

Easings: `standard` `cubic-bezier(0.2, 0, 0, 1)`, `enter` `cubic-bezier(0, 0, 0, 1)`, `exit` `cubic-bezier(0.3, 0, 1, 1)`, `emphasized` `cubic-bezier(0.32, 0.72, 0, 1)`, `scrollStartStop` `cubic-bezier(0.45, 0, 0.55, 1)`, and a spring (`stiffness` 170, `damping` 26, `mass` 1).

Reduced motion (`motion.reducedMotion`): `resync` 0, `pulse` `none`, `countdown` `crossfade-only`.

## Layout and focus

`layout`: `sidebar` 240, `inspector` 300, `toolbar` 44, `transport` 56, `titlebar` 38, `focusLineDefault` 0.35, `minWindow` `[960, 600]` (matches `minWidth`/`minHeight` of the control window in `tauri.conf.json`). `focus`: `ringWidth` 2, `ringOffset` 2.

## Changing tokens

1. Edit `tokens.json`. Recompute `contrast` for any changed prompter theme; `apps/desktop/src/lib/tokens.test.ts` checks it with `contrastRatio`.
2. New prompter themes need all required `PrompterTheme` fields, otherwise `Tokens::embedded()` panics at startup.
3. Rebuild both Rust and the frontend, because both embed the file at build time.
