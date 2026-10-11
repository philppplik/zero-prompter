# Introduction

zero-teleprompter is a free teleprompter for Windows, macOS and Linux (license: MIT OR Apache-2.0). It is a native desktop app built with Tauri 2 (Rust core, Svelte 5 interface). Its main feature is **voice tracking**: the app listens through your microphone, recognizes your speech offline, and scrolls the script so that the word you just spoke sits at the reading marker.

It is aimed at three groups:

- **Creators** speak alone in front of a camera and want the text to follow their voice.
- **Operators** run prompter glass (a beam splitter with a mirrored image), a second monitor and a person in front of the camera.
- **Presenters** want calm, readable text, including for dyslexia or in dark rooms.

## Guiding principles

These come from the project concept document (`docs/concept/CONCEPT.md`, section 1).

1. **Local first.** Audio never leaves your device. The only network access in v0.1 is the optional download of speech models. (One exception is described in [Voice tracking](user/voice-tracking.md#windows-speech-recognition-system-fallback): the Windows system speech recognition fallback can use a Microsoft online service, depending on a Windows setting.)
2. **Do not guess.** If voice tracking is unsure, the text stays where it is (the `hold` and `lost` states) instead of running away.
3. **Keyboard first.** Every action has a key and appears in the command palette (`Ctrl+K`).

## Terminology

The manual uses these terms consistently.

| Term | Meaning |
|---|---|
| **Control window** | The operator window with the editor, the live preview, the transport bar and the settings. Closing it quits the app. |
| **Talent window** | The separate output window that shows the prompter text to the person reading. It can be placed on another monitor, shown fullscreen and mirrored. Closing it only hides it. |
| **Voice tracking** | Following the speaker's position in the script by recognizing speech and aligning it to the script text. Also the name of one of the four scroll modes. |
| **Streaming model** | A speech model that produces partial results while you speak. It drives the cursor in real time. |
| **Resync** | A correction of the cursor position. Either automatic (an optional offline model re-anchors the cursor in the background) or manual (you click a word or press `R`). |

## About the interface language

In the version this manual describes, the on-screen labels of the app are in German, and I found no language switch in the code. This manual therefore gives the English meaning of each control and, where it helps you find it, the German label in parentheses, for example "Settings (Einstellungen)".

## How the manual is organized

- **User guide:** installing and using the app. Start with [Installation](user/installation.md) and [First steps](user/first-steps.md).
- **Developer guide:** architecture, algorithms, build and tests.
- **Reference:** settings, design tokens, IPC commands and events, logging, the model manifest and the planned WebSocket protocol.
- **Decisions:** the architecture decision records (ADRs).

## Accuracy

Behavior described in this manual is behavior according to the source code, unless a page says otherwise. Statements that could not be checked against the code or on a real system are marked **(unverified)**. See [Project status](status.md) for the current maturity of the project.

To build the manual: `mdbook build docs/book` (output in `docs/book/book/`). mdBook is only a documentation tool and is not shipped with the app.
