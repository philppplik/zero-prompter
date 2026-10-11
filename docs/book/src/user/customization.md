# Customization

Open **Settings (Einstellungen)** with `Ctrl+,` (macOS `Cmd+,`) or the settings button at the top right. The tabs relevant to this page are Appearance (Darstellung), Reading marker (Lesemarker), Scroll and Shortcuts (Tastenkürzel). `Esc` or a click outside closes the dialog.

Settings live in two places, and both windows (control and talent) use them:

- **Core settings** (`settings.json`): theme, font size, font weight, line height, letter, word and paragraph spacing, alignment, uppercase, focus line, tempo, countdown, voice behavior, hotkeys, microphone gain and channel, and the whole reading-marker block. The complete list is in the [Settings reference](../reference/settings.md).
- **UI preferences** (`ui-prefs.json`): font family (including a free system font name), column width, custom colors, dimming of read text, interface appearance, onboarding state. The app broadcasts changes to both windows (event `prefs://changed`).

## Themes

There are six prompter themes, defined in `docs/design/tokens.json`. The contrast ratios below are the values stored in that file, and all are marked WCAG AAA there. Switch with `Ctrl+1` to `Ctrl+6` or by clicking a tile in Appearance.

| Key | Theme | Contrast | Notes |
|---|---|---|---|
| `Ctrl+1` | Studio Dark (default) | 16.99:1 | Warm off-white on near-black, amber marker. |
| `Ctrl+2` | Broadcast Classic | 21.0:1 | White on black, yellow marker. |
| `Ctrl+3` | Daylight | 16.40:1 | Light background, dark text. |
| `Ctrl+4` | High Contrast | 16.13:1 | Yellow text on black, white marker. |
| `Ctrl+5` | Low Light | 9.87:1 | Warm and dark, for dark rooms. |
| `Ctrl+6` | Dyslexia | 13.22:1 | Cream background; letter spacing 0.04 em, word spacing 0.16 em, line height 1.6. |

## Typography

| Setting | Range / values | Default |
|---|---|---|
| Font size (Größe) | 24 to 300 px. `Ctrl+=` and `Ctrl+-` change it by 4 px. | 72 px |
| Font family (Schriftart) | See [Fonts](#fonts). | Atkinson Hyperlegible Next |
| Column width (Spaltenbreite) | 16 to 60 `ch`. The UI recommends 25 to 40 characters per line. | 32 ch |
| Line height (Zeilenabstand) | 1.0 to 2.4 in the settings slider (the core accepts 1.0 to 3.0). A theme's own value (Dyslexia: 1.6) acts as a minimum. | 1.4 |
| Font weight (Schriftstärke) | 100 to 900. See below. | 500 |
| Letter spacing (Zeichenabstand) | -0.05 to 0.30 em. 0 means "use the theme's value". | 0 |
| Word spacing (Wortabstand) | -0.10 to 1.00 em. 0 means "use the theme's value". | 0 |
| Paragraph spacing (Absatzabstand) | 0 to 3 em | 0.6 em |
| Alignment (Ausrichtung) | Left, Center, Justify | Left |
| Case (Groß-/Kleinschreibung) | Normal, UPPERCASE (display only; the script text is unchanged) | Normal |
| Focus line / reading position | 5 % to 70 % of the window height; set in the Reading marker tab (Position) or by dragging in the preview. | 35 % |

A live preview under the font selector shows every change immediately. **Reset typography (Typografie zurücksetzen)** restores the weight, line height, spacing, alignment and case defaults. After each change the prompter re-measures its lines so that the marker, dimming and word highlight stay correct.

**Font weight.** Variable fonts use their `wght` axis continuously and are clamped to their range (for example Open Sans 300 to 800, IBM Plex Sans up to 700). OpenDyslexic has only the weights 400 and 700, so the value snaps to the nearest one. The settings page shows the supported range and, if different from your setting, the weight actually rendered.

### Fonts

The prompter fonts are bundled with the app (via `@fontsource`, SIL OFL 1.1 license) and work offline.

| Group | Fonts |
|---|---|
| Sans | Atkinson Hyperlegible Next, Inter, Lexend, Source Sans 3, Open Sans, Roboto Flex, IBM Plex Sans |
| Serif | Source Serif 4, Literata, Merriweather |
| Special (Speziell) | OpenDyslexic, JetBrains Mono |
| System | **System font (Systemschrift (Standard))**, and **System font... (Systemschrift...)** where you type the name of an installed font (for example `Verdana`). If that font is not installed, the system font is used. The name is limited to 64 characters and to letters, digits, spaces, `_`, `.` and `-`. |

Older preference files stay valid: the former `serif` choice now maps to Source Serif 4 (instead of Georgia), and `atkinson` maps to the Next variant.

## Colors and dimming

- **Custom colors** for text, background and marker override the theme. A readout shows the contrast ratio between your text and background colors (AAA at 7:1 or more, AA between 4.5:1 and 7:1, "too low" below 4.5:1). Check it yourself, since the app does not stop you. **Reset (Zurücksetzen)** returns to the theme colors.
- **Dim read text (Gelesenen Text dimmen):** fades the words you have already read. The opacity slider covers 0.3 to 1; without a value the theme's default applies.
- **Interface appearance (Oberfläche):** System, Light or Dark. This affects only the control window, not the prompter text. The interface theme is a neutral gray design with the DM Sans and Geist Mono fonts, bundled locally. Optionally enable a frame-time display.

## Reading marker styles

The **Reading marker (Lesemarker)** tab sets how the current reading position looks. It is stored in `settings.json` under `marker`, so the talent window and the preview look identical, also when mirrored. A live preview shows the result.

| Style (German label) | Appearance |
|---|---|
| Line (Linie) | Thin reading line across the column width. Default. |
| Band (Band) | Semi-transparent line with edges. |
| Arrows (Pfeile) | Triangles at the left and right edges. |
| Brackets (Klammern) | Square brackets around the reading line. |
| Spotlight (Fokus-Fenster) | Everything except the reading zone is darkened. |
| Gradient (Verlauf) | Soft band; text above and below fades out. |
| Underline (Unterstreichung) | Underlines the current word or the current sentence (choose Word or Sentence). |
| Side bar (Seitenbalken) | Vertical accent bar to the left of the line. |
| No marker (Kein Marker) | Position only, nothing visible. |

Further controls:

| Control | Range | Notes |
|---|---|---|
| Color | any `#RRGGBB`, or theme color | Presets from the theme accent or a custom picker. |
| Opacity / Darkening | 0.1 to 1.0 | Labeled "darkening" (Abdunklung) for Spotlight and Gradient. |
| Thickness / height (Dicke / Höhe) | 0.5x to 3.0x | |
| Position | 5 % to 70 % | Same as the focus line. |
| Dim read text | 0 to 0.9 | "Theme" resets to the theme default. |
| Dim unread text | 0 to 0.9 | Text below the reading zone. Default 0 (off). |
| Glow | 0 to 1 | 0 is off. |
| Soft focus outside the zone (Weichzeichnen außerhalb) | 0 to 8 px | Applies to Spotlight and Gradient. |
| Animation | None, Glide (Gleiten), Pulse (Puls) | Motion when the marker moves. |

A command palette entry ("Lesemarker: ...") cycles to the next style. An older `markerStyle` preference is migrated once to the new setting.

## Tempo and countdown

- Tempo is 40 to 400 WPM, default 140. It applies to constant mode and as the fallback when voice tracking is lost with the policy "continue at constant speed". In Constant mode the mouse wheel also changes it (see [Keyboard shortcuts](shortcuts.md#mouse-wheel)).
- **Countdown (Countdown):** Off (0), 3 s (default) or 5 s. Set it in Settings, Scroll, or by clicking the countdown button in the transport bar.

## Scroll tab

| Control | Notes |
|---|---|
| Mode (Modus) | Manual, Constant, Voice-activated, Voice tracking. See [Scroll modes](first-steps.md#scroll-modes). |
| Tempo | Slider, 40 to 400 WPM in steps of 5. |
| Countdown | Off, 3 s, 5 s. |
| Reduce motion (Bewegung reduzieren) | Also applies to interface animations. |

## Reduce motion

**Reduce motion** (core setting `reducedMotion`), or the system setting `prefers-reduced-motion`, turns off or shortens soft transitions such as the resync ease, the marker pulse and the countdown animation. It also covers interface animations (dialogs, palette, drawer, notices).

## Remapping shortcuts

In Settings, Shortcuts, click a binding and press the new key combination. Only the actions in `Settings.hotkeys` can be remapped; they are listed in [Keyboard shortcuts](shortcuts.md#remappable-shortcuts). The other in-app keys and the global hotkeys are fixed in v0.1.

## Output tab

Choose the monitor for the talent window (numbered tiles with resolution), open or close the talent window, toggle fullscreen, and set horizontal and vertical mirroring. **Reload monitors and devices (Monitore & Geräte neu laden)** refreshes the lists after you plug in hardware. The app places the talent window by moving it onto the target monitor and then switching to fullscreen; some window managers ignore this.
