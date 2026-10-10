# zero-teleprompter

**Freier, quelloffener Teleprompter für Windows, macOS und Linux, der dir zuhört.**
zero-teleprompter läuft komplett offline. Per Voice-Tracking folgt der Text deiner Stimme (Deutsch und Englisch). Er lässt sich für Prompter-Glas spiegeln und auf einem zweiten Monitor ausgeben. Audio verlässt nie das Gerät.

> **Status: v0.1 in Entwicklung, noch nie kompiliert.**
> Der gesamte Code entstand auf einem Rechner ohne Rust-, Node- und Git-Toolchain. Er wurde weder gebaut noch getestet. Die sherpa-onnx-Anbindung ist gegen Version 1.13.8 auf docs.rs belegt (nicht kompiliert). Ungeprüft sind vor allem die APIs von cpal 0.15, ureq 2 und `tauri-plugin-global-shortcut` (`// VERIFY`). Echte Spracherkennung (Feature `sherpa`) ist standardmäßig aus. Das Modell-Manifest listet 5 Modelle ([MODELS.md](docs/research/MODELS.md)), ist aber noch nicht ladbar (SHA-256 der `tokens.txt` fehlen). Rechtliche Punkte, u. a. die ungeklärte Kroko-DE-Lizenz: [LEGAL.md](docs/legal/LEGAL.md). Ohne Modell läuft Voice im **Demo-Modus**, der das Skript simuliert „vorliest“. Siehe [Erste Schritte nach dem Klonen](#erste-schritte-nach-dem-klonen).

<!-- SCREENSHOT: Control-Fenster (Sidebar · Editor/Vorschau · Inspector · Transport), docs/book/src/img/control.png -->
<!-- SCREENSHOT: Talent-Fenster gespiegelt im Vollbild, docs/book/src/img/talent.png -->
<!-- SCREENSHOT: Voice-Statusanzeige (tracking/hold/lost), docs/book/src/img/voice.png -->
*Screenshots folgen, sobald die App zum ersten Mal gebaut ist.*

## Funktionen (v0.1, Stand des Codes)

- **Skripte:** `.txt`/`.md` importieren (Dialog oder Drag-and-Drop). Bibliothek in einem lokalen Ordner. Regieanweisungen in `[eckigen Klammern]` werden angezeigt, aber nicht gesprochen erwartet.
- **Scroll-Modi:** Manuell, Konstant (40–400 WPM), Sprachaktiviert (läuft nur, solange gesprochen wird) und Voice-Tracking.
- **Voice-Tracking:** offline mit sherpa-onnx (Feature `sherpa`). Der Abgleich nutzt ein gefenstertes Smith-Waterman-Verfahren mit Unschärfe. Zustände `idle · listening · tracking · hold · lost · resync`. Hysterese gegen Fehlsprünge. Ein Klick auf ein Wort löst „Resync hier“ aus.
- **Ausgabe:** separates Talent-Fenster, wählbarer Monitor, Vollbild, Spiegelung horizontal und vertikal.
- **Anpassung:** 6 Themes aus `tokens.json` (alle WCAG AAA, darunter Dyslexie und Low-Light). Einstellbar sind Schriftgröße (24–300 px), Schriftart, Zeilenbreite, Lesemarker (Linie oder Band), Fokuslinie, Dimmen des gelesenen Texts und eigene Farben.
- **Bedienung:** Tastatur zuerst, Befehlspalette (`Ctrl+K`), frei belegbare In-App-Tasten, globale Hotkeys `Ctrl+Shift+F6`…`F11`, Countdown.
- **Modelle:** Manifest mit SHA-256-Prüfung, Download über HTTPS mit Fortsetzen (Feature `download`).

Noch **nicht** enthalten (geplant ab v0.2): Fernsteuerung per Handy/WebSocket, MIDI/HID/Gamepad, Overlay-Modus, `.docx`-Import, Updater, Code-Signing.

## Quickstart

```bash
git clone <repo-url> zero-teleprompter
cd zero-teleprompter
cargo test                       # reine Kern-Crates, keine Systemabhängigkeiten
cd apps/desktop
npm install
npx tauri icon app-icon.png      # erzeugt alle Icons (inkl. .icns)
npm run tauri dev                # startet die App (Voice im Demo-Modus)
```

Mit echter Spracherkennung: `npm run tauri -- dev --features sherpa` <!-- VERIFY: Tauri-CLI-2-Flag -->. Dafür muss vorher ein Modell im Manifest eingetragen und installiert sein, siehe Handbuch → Voice-Tracking.

## Build-Voraussetzungen

| Was | Version | Hinweis |
|---|---|---|
| Rust über [rustup](https://rustup.rs) | stable, MSRV **1.80** | `rust-toolchain.toml` wählt stable mit `rustfmt` und `clippy` |
| Node.js | **20+** (LTS) | npm wird mitgeliefert |
| Tauri-2-Systemabhängigkeiten | – | <https://v2.tauri.app/start/prerequisites/> |
| **Windows** | Visual Studio 2022 Build Tools mit „Desktopentwicklung mit C++“ (MSVC + Windows SDK), WebView2 Runtime (ab Windows 11 vorinstalliert) | `rustup default stable-msvc` |
| **macOS** | Xcode Command Line Tools (`xcode-select --install`) | Mindestversion 11.0 |
| **Linux** | `libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf libasound2-dev` | wie in der CI (Ubuntu 22.04) |

Feature-Flags der App (`apps/desktop/src-tauri`):

| Feature | Standard | Zweck |
|---|---|---|
| `cpal` | an | Mikrofon über cpal |
| `download` | an | Modelldownload über ureq (nur HTTPS, Fortsetzen per `Range`) |
| `sherpa` | aus | echte Spracherkennung mit sherpa-onnx. Ohne dieses Feature läuft Voice im Demo-Modus (`MockEngine`) |

Linux mit NVIDIA: Ruckelt WebKitGTK, `ZT_SAFE_RENDER=1` setzen. Die App setzt dann `WEBKIT_DISABLE_DMABUF_RENDERER=1`.

## Tests

```bash
cargo test                                           # Kern-Crates inkl. zt-testkit (Szenarien, Proptests)
cd apps/desktop && npm test && npm run check         # Vitest + svelte-check
npm run build && cd ../.. && cargo test --workspace  # inklusive App (braucht apps/desktop/dist)
```

## Erste Schritte nach dem Klonen

Weil der Code ungetestet ist, sollte die erste Person mit Toolchain diese Liste abarbeiten:

1. `cargo test`, `cargo build -p zt-audio --features cpal`, `cargo build -p zt-stt --features sherpa`
2. `npm install`, danach `package-lock.json` committen und in der CI auf `npm ci` umstellen. Anschließend `npm run check` und `npm test`.
3. `npm run build`, danach `cargo clippy -p zero-teleprompter`. Alle `// VERIFY` prüfen (`rg "VERIFY"`).
4. In `apps/desktop/src-tauri/resources/models.json` die fehlenden SHA-256 der `tokens.txt` nachtragen (herunterladen und hashen) und die URLs auf Commits pinnen. Vorher `cargo fmt` und `npm run format` einmal laufen lassen.
5. Die fehlenden Commit-Hashes und Copyright-Zeilen in `THIRD_PARTY_NOTICES.md` nachtragen.

## Dokumentation

- **Handbuch (mdBook):** `docs/book/`. Bauen mit `mdbook build docs/book`, lesen mit `mdbook serve docs/book`.
- Recherche: `docs/research/RESEARCH.md` · Konzept: `docs/concept/CONCEPT.md` · Design: `docs/design/DESIGN.md`, `docs/design/tokens.json` · ADRs: `docs/adr/` (ADR-001…005) · Modelle: `docs/research/MODELS.md` · Recht: `docs/legal/LEGAL.md`
- [CONTRIBUTING.md](CONTRIBUTING.md) · [CHANGELOG.md](CHANGELOG.md)

## Repository-Aufbau

```
crates/            Reiner Rust-Kern ohne I/O (ADR-005, hexagonal)
  zt-core          Zustand, Commands, Playback, Skriptmodell, Settings/Themes aus tokens.json
  zt-align         Abgleich Sprache ↔ Skript (Smith-Waterman), Voice-Zustandsautomat
  zt-audio         Port AudioSource, cpal-Adapter (Feature cpal), Resampling auf 16 kHz mono
  zt-stt           Port SpeechEngine, sherpa-onnx (Feature sherpa), MockEngine, Hotwords
  zt-models        Modell-Manifest, SHA-256, Installationslayout, Downloader-Port
  zt-testkit       nur für Tests: Fixtures, Szenarien, Pipeline-Harness (publish = false)
apps/desktop/      Tauri-2-App
  src-tauri/       Adapter: Commands/Events, Tick-Schleife, Voice-Thread, globale Hotkeys, Persistenz, Modelle
  src/             Svelte 5 + TypeScript (Control-Fenster index.html, Talent-Fenster talent.html)
docs/              Recherche, Konzept, Design, ADRs, mdBook
```

## Lizenz

Wahlweise unter [Apache-2.0](LICENSE-APACHE) oder [MIT](LICENSE-MIT). Hinweise zu Drittanbietern stehen in [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md), Modell-Lizenzen in [THIRD_PARTY_MODELS.md](THIRD_PARTY_MODELS.md).
