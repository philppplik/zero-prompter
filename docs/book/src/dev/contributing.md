# Contributing

Die verbindliche Fassung steht in [`CONTRIBUTING.md`](https://github.com/philppplik/zero-prompter/blob/main/CONTRIBUTING.md) im Repository-Root. Kurzfassung:

- **Beiträge** stehen unter `MIT OR Apache-2.0` (Inbound = Outbound). Es gibt kein CLA.
- **Kein GPL/AGPL/LGPL-Code.** QPrompt und Imaginary Teleprompter dienen nur als UX-Referenz. Code aus MIT- oder Apache-Projekten (z. B. OpenTeleprompter, autocue, Textream) darf übernommen werden, braucht aber
  1. einen Attributionskommentar an der Stelle im Code (Projekt, URL, Lizenz) und
  2. einen Eintrag in `THIRD_PARTY_NOTICES.md` (Lizenz, Copyright, Commit, Verwendungszweck, Fundstelle).
- Neue Modelle: Eintrag in `THIRD_PARTY_MODELS.md` mit SPDX-Lizenz und Attribution. Nur verifizierte URLs und SHA-256-Werte.
- Vor einem PR: `cargo fmt`, `cargo clippy --all-targets`, `cargo test`, `npm run check`, `npm test`.
- Unsichere APIs kapselst du hinter eigenen Traits und markierst sie mit `// VERIFY`, statt zu raten.
- Architekturänderungen bekommen ein ADR in `docs/adr/` (MADR-Format).
- Doku-Pflicht: Ändern sich Commands, Events, Settings oder Tastenkürzel, aktualisierst du die passende Seite in `docs/book/src/reference/` bzw. `user/shortcuts.md` im selben PR.
