# First steps

The on-screen labels of the app are German in this version. This page gives the English meaning and, where useful, the German label in parentheses.

## Onboarding

On first launch (while `onboarded` is not set in `ui-prefs.json`) a four-step assistant opens. A built-in example script is already loaded. You can skip the assistant at any time with **Skip (Überspringen)** or `Esc`.

1. **Welcome (Willkommen).** Short introduction.
2. **Microphone (Mikrofon).** Click **Allow microphone (Mikro erlauben)** to start a microphone test. A level meter appears and, if a speech engine is running, the recognized text is shown. If no microphone is available or access is denied, the assistant tells you; you can enable voice tracking later.
3. **Speech model (Sprachmodell).** Lists the models from the manifest with a **Download (Laden)** button each. You can continue while a download runs in the background.
4. **Output (Ausgabe).** Choose the monitor for the talent window and optionally mirror horizontally. **Start rehearsal (Probelauf starten)** opens the talent window on the chosen monitor, sets the scroll mode to constant and starts playback if a script is loaded.

You can restart the assistant at any time: press `Ctrl+K` and run "Einführung erneut starten" (restart the introduction).

## A tour of the control window

```
[logo]                    [voice status] [Commands Ctrl+K | Output | Settings]  [_ [] X]
[rail] +---------------------------------------------------------------+
       | script title        Manual | Constant | Voice     Edit | Live  |
       |                                                               |
       |            editor (Edit)  or  prompter preview (Live)         |
       +---------------------------------------------------------------+
        transport: home, paragraph back, play/pause, paragraph forward, WPM, progress, time, countdown
```

### Top bar

- **Logo** on the left.
- **Voice status** pill and, when applicable, a **Demo** or **model error** badge. See [Voice tracking](voice-tracking.md#status-indicator).
- **Action pill** with three buttons: **Commands (Befehle)** opens the command palette (`Ctrl+K`); **Output** opens or closes the talent window (a dot shows it is active); **Settings (Einstellungen)** opens the settings dialog (`Ctrl+,`).
- **Exit live (Live beenden)** button, visible only in Live mode.
- **Window controls.** On Windows and Linux the control window has no system title bar. The top bar is the drag area (double-click maximizes or restores), and on the right are minimize, maximize or restore, and close. **Close quits the whole app.** On macOS the native traffic-light buttons stay at the top left (overlay title bar) and no custom controls are drawn.

### Rail and scripts drawer

The narrow **rail** on the left has one button that toggles the **scripts drawer** (`Ctrl+B`). The drawer is hidden by default and contains:

- the **script library** (new script, open file, delete) with a word count tooltip per script, and
- an **outline** by paragraph. Click a paragraph to jump there.

You cannot switch scripts while the text is playing.

### Main card: edit or live

- **Edit mode:** a Markdown text editor with autosave to the library. The status line shows the word count and an estimated duration at the current WPM, and "locked" while playback runs (the text cannot be edited during playback).
- **Live mode:** a scaled 1:1 preview of the talent window. Click a word to trigger [resync](voice-tracking.md#manual-intervention). Drag the reading position in the preview to move the focus line.
- Toggle between the modes with `Ctrl+E` or the **Edit | Live** switch. Switching to Edit is disabled while the text is running.
- The **Manual | Constant | Voice** switch in the card header selects the [scroll mode](#scroll-modes). The fourth mode, voice-activated, is available in settings and the command palette.
- In Live mode a banner appears if the talent window is not open ("output not connected"), and another one if voice tracking has been lost for more than 4 seconds.

### Transport bar

Home (to start), paragraph back, play or pause, paragraph forward; tempo in WPM with minus and plus buttons and a number field (40 to 400); a position bar you can click (and move with the arrow keys in steps of 10 words); remaining and total time; "word N/M"; and the countdown button, which cycles **0, 3 and 5 seconds** (a start delay; the default is 3).

### Settings dialog

Open with `Ctrl+,` or the settings button. It is a modal dialog with a vertical tab list (use the arrow keys, `Home` and `End` to move between tabs). `Esc` or a click outside closes it.

| Tab (German label) | What it contains |
|---|---|
| **Appearance (Darstellung)** | Prompter theme, font, weight, size, column width, line height, letter, word and paragraph spacing, alignment, uppercase, dimming of read text, custom colors with a contrast readout, interface theme (system, light, dark), frame-time display. See [Customization](customization.md). |
| **Reading marker (Lesemarker)** | Marker style, color, shape, dimming and effects, with a live preview. |
| **Scroll** | Scroll mode, tempo, countdown, reduce motion. |
| **Voice and models (Voice & Modelle)** | Language, microphone, gain, channel, microphone test, lost behavior, speech model choice, resync switch and the model download list. See [Voice tracking](voice-tracking.md). |
| **Shortcuts (Tastenkürzel)** | Remap the in-app shortcuts. See [Keyboard shortcuts](shortcuts.md). |
| **Output (Ausgabe)** | Monitor selection, open or close the talent window, fullscreen, horizontal and vertical mirroring, reload monitors and devices. |
| **Diagnostics (Diagnose)** | Live engine, audio and latency information, errors and log export. |
| **About (Über)** | Version, build, license (MIT OR Apache-2.0) and links to the source repository, third-party notices, model licenses and the changelog. |

## Loading a script

- **Open:** `Ctrl+O` or the file icon in the scripts drawer. Accepted extensions: `.txt`, `.md`, `.markdown` and `.text` (the file dialog itself lists `.txt`, `.md` and `.markdown`). Maximum size 8 MB. UTF-8 files (with or without a byte-order mark) are read as is; other encodings are interpreted as Latin-1 instead of failing.
- **Drag and drop** a file onto the window.
- **Paste:** with an empty script and focus outside a text field, `Ctrl+V` loads the clipboard text as the script. In the empty editor you can also paste directly.
- **New:** the **+** button in the scripts drawer.

Imported files are saved into the library as `<name>-<timestamp>.md`. The editor autosaves your changes.

### Script format

- Separate **paragraphs** with a blank line. `PgUp` and `PgDn` jump by paragraph.
- **Stage directions** go in square brackets, for example `[PAUSE]` or `[look at camera]`. They are shown but are not counted as spoken words: voice tracking skips them, and they are not part of the word count or duration.
- The first non-empty line (leading `#` characters removed) becomes the script's title in the library.
- Markdown headings are allowed.
- You can write numbers as digits ("2026"); the app compares them with the spoken form.

## Rehearse and go live

1. Press `Ctrl+E` to enter **Live** mode. While the text is playing it is locked.
2. Set the tempo: `Up` and `Down` change it by 5 WPM (range 40 to 400).
3. Press `Space` to start. With a countdown set, the countdown runs first; `Esc` cancels it.
4. Open the **talent window** with the **Output** button (or the command palette entry "Talent-Fenster öffnen") or by picking a monitor in Settings, Output. Press `F` to toggle fullscreen; if the talent window is hidden, `F` opens it fullscreen.
5. Mirror the output with `M` (horizontal) and `Shift+M` (vertical).
6. `Home` stops and jumps to the start in Live mode (in Edit mode it jumps to the start).

At the end of the script, playback enters the "end" state.

### Scroll modes

| Mode | Behavior |
|---|---|
| **Manual (Manuell)** | The text moves only when you move it: mouse wheel, paragraph jumps, position bar, clicking a word. |
| **Constant (Konstant)** | The text scrolls at the WPM rate. The mouse wheel changes the tempo. |
| **Voice-activated (Sprachaktiviert)** | Uses the voice pipeline but scrolls at the constant WPM rate only while speech is detected. |
| **Voice tracking (Voice-Tracking)** | The text follows the recognized words. See [Voice tracking](voice-tracking.md). |

Selecting a voice mode turns the voice pipeline on if it was off; selecting manual or constant turns it off. `V` toggles voice and, as a side effect, switches the scroll mode to voice tracking (on) or constant (off). Mouse wheel behavior per mode is in [Keyboard shortcuts](shortcuts.md#mouse-wheel).

## Command palette

`Ctrl+K` (macOS `Cmd+K`) opens a fuzzy search over all actions, grouped as playback, appearance, themes, output, voice, scripts and help. `?` shows all keyboard shortcuts.
