# Microsoft Store

Stand: 2026-10. Quelle: Tauri-Doku „Microsoft Store“ (https://v2.tauri.app/distribute/microsoft-store/) und Microsoft Learn „Publish your MSI or EXE app“.

## Welcher Weg?

Tauri 2 erzeugt **keine MSIX-Pakete**, nur MSI und NSIS-EXE. Der von Tauri empfohlene Weg ist deshalb ein **MSI/EXE-Listing** im Store: Im Partner Center wird eine Store-Seite angelegt, die auf einen von uns gehosteten Installer verlinkt (z. B. das GitHub-Release-Asset).

Für dieses Listing gilt:

| Anforderung | Umsetzung im Repo |
|---|---|
| Installer muss offline installieren (keine Downloads während der Installation) | `src-tauri/tauri.microsoftstore.conf.json` setzt `webviewInstallMode = offlineInstaller` (WebView2 im Installer, +~130 MB) |
| Stille Installation | NSIS: Parameter `/S`. MSI: `/quiet` |
| `publisher` ≠ Produktname | `bundle.publisher = "zero-teleprompter contributors"` (an den Partner-Center-Herausgebernamen anpassen) |
| Feste, versionierte Download-URL | Release-Assets `zero-teleprompter_<version>_<x64|arm64>_store-setup.exe` (Job `store` in `release.yml`) |
| **Code-Signatur** des Installers mit einem Zertifikat einer vertrauenswürdigen CA | **offen**: Bei MSI/EXE übernimmt der Store die Signatur **nicht**. |
| Updates | Über neue Installer-URL/Version im Partner Center. Ein In-App-Updater ist erlaubt, aber nicht nötig |

**Wichtig zur Signatur:** Nur bei **MSIX**-Paketen signiert Microsoft das Paket bei der Veröffentlichung selbst. Bei MSI/EXE muss der Installer (und die EXE darin) vorher signiert sein, sonst schlägt die Zertifizierung fehl. Günstige Option: **Azure Artifact Signing** (früher „Trusted Signing“, ca. 10 USD/Monat, Identitätsprüfung nötig). Alternativ ein OV/EV-Zertifikat. Tauri kann über `bundle.windows.signCommand` signieren. Das ist vorbereitet, aber erst mit Zertifikat aktivierbar.

**Alternative MSIX** (Store signiert, keine eigene Signatur nötig): Tauri hat dafür keinen Bundler. Möglich wäre, die gebaute App mit dem MSIX Packaging Tool oder `makeappx` manuell zu verpacken. Dafür braucht man die Werte *Package/Identity/Name* und *Publisher* (`CN=…`) aus dem Partner Center. Nicht umgesetzt, weil es außerhalb des Tauri-Standardwegs liegt. Falls die Signaturkosten ein Problem sind, ist das der Ausweg.

## Schritt für Schritt

### 1. Entwicklerkonto anlegen

1. https://storedeveloper.microsoft.com öffnen und mit einem Microsoft-Konto anmelden.
2. Kontotyp wählen. **Einzelperson**: Seit 2025 ist die Registrierung für Einzelentwickler kostenlos (Identitätsprüfung mit Ausweis und Selfie). **Unternehmen**: einmalig ca. 99 USD, dazu eine Verifizierung des Unternehmens. Kosten vor der Anmeldung auf der Microsoft-Seite prüfen, sie ändern sich gelegentlich.
3. Verifizierung abwarten (Stunden bis wenige Tage).

### 2. App-Namen reservieren

1. Partner Center → **Apps und Spiele** → **Neues Produkt** → **EXE- oder MSI-App**.
2. Namen reservieren, z. B. „zero-teleprompter“. Der Name muss im Store eindeutig sein.

### 3. Werte ins Repo übernehmen

- Unter **Produktidentität / Kontoeinstellungen** den **Herausgeber-Anzeigenamen** ablesen. Diesen Wert in `bundle.publisher` eintragen, und zwar in `tauri.conf.json` **und** in `tauri.microsoftstore.conf.json`. Er darf nicht dem Produktnamen entsprechen.
- Für ein späteres MSIX bräuchte man zusätzlich *Package/Identity/Name* und *Publisher* (`CN=…`). Für MSI/EXE werden sie nicht benötigt.
- Den Installer signieren lassen (siehe oben). Dazu das Zertifikat bzw. die Artifact-Signing-Zugangsdaten als GitHub-Secrets anlegen und `signCommand` ergänzen.

### 4. Build

`gh workflow run release.yml` oder einen Tag `vX.Y.Z` pushen. Der Draft enthält danach zusätzlich:

- `zero-teleprompter_<v>_x64_store-setup.exe`
- `zero-teleprompter_<v>_arm64_store-setup.exe`

Den Draft veröffentlichen. Das Partner Center braucht eine öffentlich erreichbare HTTPS-URL, die sich nicht ändert. Für jede Version also eine neue URL verwenden, nie eine bestehende Datei überschreiben.

Lokal geht es auch so:

```bash
cd apps/desktop
npm run tauri build -- --no-bundle
npm run tauri bundle -- --bundles nsis --config src-tauri/tauri.microsoftstore.conf.json
```

### 5. Einreichung (Partner Center)

1. **Preise und Verfügbarkeit**: kostenlos, Märkte wählen.
2. **Eigenschaften**: Kategorie *Produktivität*. Datenschutzrichtlinie-URL angeben. Die App ist offline, Modelle werden nur auf Anforderung von Hugging Face und GitHub geladen.
3. **Altersfreigabe**: Fragebogen ausfüllen.
4. **Pakete**: Pro Architektur die Installer-URL (x64 und Arm64) angeben, Installer-Typ *EXE*, stiller Parameter `/S`.
5. **Store-Eintrag**: Beschreibung (DE/EN), mindestens ein Screenshot (1366×768 oder größer) und Logos. `npx tauri icon` erzeugt die Store-Logos (`StoreLogo.png`, `Square*Logo.png`).
6. **Einreichen.** Die Zertifizierung dauert meist 1–3 Werktage. Typische Ablehnungsgründe sind fehlende Signatur, eine nicht stille Installation, ein Installer, der nachlädt, und fehlende Datenschutzrichtlinie.

### 6. Updates

Neue Version bauen, Release veröffentlichen und im Partner Center unter **Pakete** die neue URL eintragen und erneut einreichen.
