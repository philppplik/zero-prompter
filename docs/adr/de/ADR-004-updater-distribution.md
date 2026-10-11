# ADR-004: Updater und Distribution

- **Status:** angenommen (Umsetzung ab v0.2)
- **Quelle:** übernommen aus `docs/research/RESEARCH.md` §4 (2026-10-10)

## Entscheidung

- **Updater:** `tauri-plugin-updater` (minisign-Signaturen) über `tauri-action`.
- **Nicht gewählt:** cargo-dist/axoupdater, weil schwach gewartet und auf CLI-Tools ausgerichtet.
- **Installer:** unter 15 MB ohne Modelle; Modelle werden zur Laufzeit geladen.

## Nachtrag 2026-10-10: Code-Signing

Recherche dazu in `docs/legal/LEGAL.md`: Azure Artifact Signing ist laut Bericht für EU-Organisationen verfügbar (Validierungsvoraussetzungen und Preise unbelegt); OV/EV-Zertifikate nur noch über Hardware-Token/Cloud-HSM; macOS braucht Apple Developer Program, D-U-N-S-Nummer und Notarisierung.
