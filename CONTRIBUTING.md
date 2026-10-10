# Mitwirken an zero-teleprompter

Danke für dein Interesse! Ausführliche Entwicklerdoku: `docs/book/` (Teil „Entwicklerhandbuch“).

> **Hinweis:** Der Code wurde bisher **nie kompiliert**. Als erster Beitrag hilft am meisten: bauen, testen und die `// VERIFY`-Stellen klären. Die Checkliste dafür steht in der README.

## Lizenz deiner Beiträge

Mit einem Pull Request stellst du deinen Beitrag unter `MIT OR Apache-2.0` (Inbound = Outbound, kein CLA). Bitte nur Code einreichen, an dem du die Rechte hast.

## Fremdcode und Ideen

- **Verboten:** Code unter GPL, AGPL oder LGPL. QPrompt und Imaginary Teleprompter sind nur UX-Referenzen. `cargo-deny` und die npm-Lizenzprüfung in der CI erzwingen das.
- **Erlaubt mit Attribution:** MIT- oder Apache-2.0-Projekte wie OpenTeleprompter (ArunNGun/openTeleprompt), EdNutting/autocue oder f/textream. Dazu gehören
  1. ein Kommentar an der übernommenen Stelle: Projekt, URL, Lizenz, ob Code oder nur Idee;
  2. ein Eintrag in `THIRD_PARTY_NOTICES.md`: Lizenz, Copyright-Zeile, Commit-Hash, Verwendungszweck, Dateien.
- **Modelle:** Eintrag in `THIRD_PARTY_MODELS.md` und im Manifest mit SPDX-`license` und, bei CC-BY, `attribution`. Nur verifizierte URLs, Größen und SHA-256-Werte.

## Ablauf

1. Für größere Änderungen erst ein Issue eröffnen. Architekturentscheidungen bekommen ein ADR in `docs/adr/`.
2. Branch anlegen, klein und fokussiert ändern.
3. Lokal prüfen:
   ```bash
   cargo fmt --all
   cargo clippy --all-targets
   cargo test
   cd apps/desktop && npm run check && npm test
   ```
   Bei Änderungen an der App zusätzlich: `npm run build && cargo clippy -p zero-teleprompter --all-targets`.
4. Doku im selben PR anpassen. Neue oder geänderte Commands, Events, Settings oder Tastenkürzel gehören in `docs/book/src/reference/` bzw. `docs/book/src/user/shortcuts.md`. Den Eintrag unter „Unreleased“ in `CHANGELOG.md` ergänzen.
5. PR mit Beschreibung öffnen: was, warum, wie getestet.

## Regeln für den Code

- Kern-Crates (`zt-core`, `zt-align`, `zt-models`) bleiben frei von I/O, Uhren und Threads. Zeit wird als Parameter übergeben.
- Unsichere externe APIs kapselst du hinter einem eigenen Trait und markierst sie mit `// VERIFY: …`.
- `unsafe` ist verboten. MSRV ist 1.80.
- Im Frontend kennt nur `lib/bridge.ts` IPC-Namen. Aktionen stehen in `lib/actions.ts`, Farben und Maße kommen aus `tokens.json`.
- Details: `docs/book/src/dev/code-style.md`.

## Fehler melden

Bitte angeben: Betriebssystem und Version, App-Version und Features (Command `build_info`), Engine-Info aus dem Inspector (Demo oder sherpa), Schritte zum Reproduzieren. Bei Voice-Problemen hilft zusätzlich ein kurzes Skript-Beispiel.
