# Code-Style

## Rust

- `rustfmt` mit Standardkonfiguration. Die CI prüft `cargo fmt --all -- --check`.
- Clippy `all` als Warnung, in der CI mit `-D warnings` als Fehler.
- `unsafe_code = "forbid"` in allen Crates, in der App `deny` (wegen der Tauri-Makros).
- `missing_debug_implementations = "warn"`: Öffentliche Typen implementieren `Debug`, notfalls von Hand mit `finish_non_exhaustive()`.
- MSRV 1.80: Keine APIs, die neuer sind. Beispiel: `Option::is_none_or` gibt es erst ab 1.82, im Code steht deshalb `!is_some_and(..)`.
- Fehler: `thiserror` in den Crates. An der Tauri-Grenze werden sie zu `String`, das Frontend zeigt sie als Toast.
- Kern-Crates bleiben **frei von I/O, Uhren und Threads**. Zeit wird als Parameter übergeben.
- Serde: `rename_all = "camelCase"` für alles, was das Frontend sieht. Ausnahme: `Script`, `Token`, `Paragraph` und `Playback` haben kein Rename und erscheinen deshalb als snake_case (`is_cue`, `countdown_secs`, `ms_left`). `types.ts` spiegelt das und muss synchron bleiben.
- Kommentare auf Englisch, Doku (`docs/`) auf Deutsch. Modulköpfe (`//!`) erklären die Rolle und verweisen auf CONCEPT- oder ADR-Abschnitte.
- `// VERIFY:` markiert jede Stelle, die gegen eine nicht kompilierte externe API geschrieben wurde.

## TypeScript / Svelte

- Svelte 5 mit Runes (`$state`, `$derived`), TypeScript strict (`tsconfig.json`).
- Nur `lib/bridge.ts` kennt die Namen von Commands und Events. Komponenten importieren `api` und `events` von dort.
- Jede Nutzeraktion ist einmal in `lib/actions.ts` definiert und wird von Hotkeys, Palette und Buttons genutzt.
- Farben und Maße stammen aus `tokens.json` (über `lib/tokens.ts`). Keine hartkodierten Farbwerte in Komponenten.
- Barrierefreiheit: ARIA-Rollen und -Labels an allen Bedienelementen, Fokusring aus den Tokens, Berücksichtigung von `prefers-reduced-motion`.
