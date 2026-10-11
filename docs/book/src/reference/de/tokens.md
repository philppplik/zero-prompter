# Design-Tokens

Quelle: `docs/design/tokens.json` (Format nach der Design Tokens Community Group, `meta.version` 1.0.0, Kontrast nach WCAG 2.2). Die Datei ist die **einzige Quelle** für Rust und UI:

- Rust bindet sie zur Compile-Zeit ein (`zt_core::settings::TOKENS_JSON`, `include_str!`) und liest daraus `state`, `prompter`, `type.prompter` und `layout.focusLineDefault`. Unbekannte Schlüssel werden ignoriert. Ein Unit-Test stellt sicher, dass die Datei gültig ist.
- Das Frontend importiert sie in `lib/tokens.ts` (Themes, UI-Farben, CSS-Variablen).

Begründung und Gestaltung: `docs/design/DESIGN.md`.

## Gruppen

| Schlüssel | Inhalt |
|---|---|
| `ui.light`, `ui.dark` | Farben der Oberfläche: `bg`, `surface*`, `border*`, `text*`, `accent*`, `focusRing`, `overlay` |
| `state.<phase>` | `color`, `shape`, `label` für `idle`, `listening`, `tracking`, `hold`, `lost`, `resync` |
| `prompter.<theme>` | `bg`, `text`, `readOpacity`, `marker`, `wordHighlight`, `direction`, `contrast`, `wcag`, optional `letterSpacing`, `wordSpacing`, `lineHeight` |
| `type` | `fontUi`, `fontPrompter`, `fontMono`, `scale.*` (UI-Schriftgrade), `prompter` (`sizeMin` 24, `sizeDefault` 72, `sizeMax` 300, `lineHeight` 1,4, `measureCh` 32, `weight` 500) |
| `space` | 0–11 → 0–64 px |
| `radius` | xs 4 · sm 6 · md 8 · lg 12 · xl 16 · pill 999 |
| `elevation` | 0–4 (Schatten) |
| `motion` | `duration` (fast 120, base 200, slow 300, countdown 1000, resync 300, playEase 600 ms), `easing` (inkl. Feder: stiffness 170, damping 26, mass 1), `reducedMotion` |
| `layout` | sidebar 240, inspector 300, toolbar 44, transport 56, titlebar 38, `focusLineDefault` 0,35, `minWindow` [960, 600] |
| `focus` | `ringWidth` 2, `ringOffset` 2 |

## Prompter-Themes

| ID | bg | text | marker | Kontrast | WCAG |
|---|---|---|---|---|---|
| `studioDark` | #0E0F12 | #F4F1EA | #FFB224 | 16,99 | AAA |
| `broadcastClassic` | #000000 | #FFFFFF | #FFE14D | 21,0 | AAA |
| `daylight` | #FAF8F3 | #1A1A1A | #C2410C | 16,40 | AAA |
| `highContrast` | #000000 | #FFE14D | #FFFFFF | 16,13 | AAA |
| `lowLight` | #120D08 | #C9B79A | #B5542C | 9,87 | AAA |
| `dyslexia` | #FDF6E3 | #1F2A44 | #2563EB | 13,22 | AAA |

Die Reihenfolge in der JSON-Datei bestimmt die Belegung von `Ctrl+1` bis `Ctrl+6`.

## Ändern

1. `tokens.json` bearbeiten. Den Kontrast in `contrast` neu berechnen. `tokens.test.ts` prüft ihn mit `contrastRatio`.
2. Neue Prompter-Themes brauchen alle Pflichtfelder von `PrompterTheme`, sonst schlägt `Tokens::embedded()` beim Start fehl.
3. Rust **und** Frontend neu bauen, denn die Tokens sind an beiden Stellen zur Compile-Zeit eingebunden.
