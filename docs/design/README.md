# Design

| File | Content |
|---|---|
| [DESIGN.md](DESIGN.md) | Design specification (German, dated 2026-10-10): principles, token system, components, ASCII wireframes, interactions and shortcuts, accessibility |
| [tokens.json](tokens.json) | Canonical design tokens (English keys, W3C design-tokens style, version 1.0.0) |
| [mockup.html](mockup.html) | Static HTML mockup |

Product concept (German): [../concept/CONCEPT.md](../concept/CONCEPT.md).

## Principles (summary of DESIGN.md section 1)

1. The text is the interface: the talent window shows only text, the reading marker and, in voice tracking, a word highlight.
2. Calm over reaction: motion only where it carries information; uncertainty means standing still.
3. The operator sees everything, the talent nothing unnecessary: diagnostics live in the control window.
4. Keyboard first: every action has a shortcut and appears in the command palette.
5. State is shown by color, shape and text, never by color alone (WCAG 1.4.1).
6. Native restraint: system-like UI font, a single accent, otherwise grayscale.
7. Measurably readable: prompter themes carry their contrast ratio.

## Tokens

Token groups: `ui.*` (chrome, light and dark; derived from the tweakcn "Minimal Neutral" theme), `state.*` (voice states idle, listening, tracking, hold, lost, resync, each with color and shape), `prompter.*` (theme presets), `type.*`, and further groups listed in DESIGN.md. `tokens.json` is canonical; the app consumes the prompter themes through `zt-core`.

Prompter themes in `tokens.json`: studioDark (default), broadcastClassic, daylight, highContrast, lowLight, dyslexia. Contrast values are stated in the file.

Voice states are shown as: idle (outlined circle), listening (pulsing circle), tracking (filled circle), hold (pause bars), lost (triangle), resync (rotating arc).

## Status

DESIGN.md predates later UI work recorded in the [CHANGELOG](../../CHANGELOG.md) (custom window controls, customizable reading marker, typography settings, bundled fonts); it was not updated for these. Treat the code and the CHANGELOG as authoritative where they differ. Not verified against the code in this overview: component list and wireframes.
