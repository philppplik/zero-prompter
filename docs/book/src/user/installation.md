# Installation

> Bisher gibt es **keine fertigen Installer**. Der Release-Workflow (`.github/workflows/release.yml`) erzeugt sie erst, wenn ein Tag `v*` gepusht wird, und wurde noch nie ausgeführt. Bis dahin musst du die App aus dem Quellcode bauen.

## Geplante Installer

Der Workflow baut diese Pakete als Entwurfs-Release auf GitHub:

| Plattform | Format |
|---|---|
| Windows x64 | MSI und NSIS (`bundle.targets = "all"`) |
| macOS arm64 und x64 | DMG, Mindestversion macOS 11 |
| Linux x64 | AppImage, deb (hängt von `libasound2` ab), rpm |

Signierung (Apple, Tauri-Updater-Schlüssel) ist vorbereitet, aber nur aktiv, wenn die entsprechenden Secrets gesetzt sind. Unsignierte Builds lösen Warnungen aus: SmartScreen unter Windows, Gatekeeper unter macOS (Rechtsklick → Öffnen).

## Aus dem Quellcode bauen

### 1. Toolchain

**Alle Plattformen**
- Rust über [rustup](https://rustup.rs). Das Repo pinnt `stable` mit `rustfmt` und `clippy` (`rust-toolchain.toml`). Die MSRV ist 1.80.
- Node.js 20 oder neuer.

**Windows**
1. „Visual Studio 2022 Build Tools“ installieren, Workload **Desktopentwicklung mit C++** (MSVC v143 und Windows 10/11 SDK).
2. Die WebView2 Runtime ist unter Windows 11 vorinstalliert. Unter Windows 10 lädst du sie gegebenenfalls von Microsoft nach.
3. `rustup default stable-msvc`

**macOS:** `xcode-select --install`

**Linux (Debian/Ubuntu):**
```bash
sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf libasound2-dev
```

Die vollständige Liste steht unter <https://v2.tauri.app/start/prerequisites/>.

### 2. Bauen

```bash
cd apps/desktop
npm install
npx tauri icon app-icon.png    # erzeugt die Icons, einschließlich icon.icns für macOS
npm run tauri dev              # Entwicklung
npm run tauri build            # Installer unter target/release/bundle/
```

Ohne weitere Flags enthält der Build die Features `cpal` (Mikrofon) und `download` (Modelldownload). Mit echter Spracherkennung baust du so:

```bash
npm run tauri -- build --features sherpa   # echte Spracherkennung (Release-Builds haben das immer)
```

## Speicherorte

Die Pfade kommen von Tauri (`app_config_dir` bzw. `app_data_dir`) und richten sich nach dem Identifier `dev.zeroteleprompter.app`:

| Datei | Ort |
|---|---|
| `settings.json` (Kern-Einstellungen, versioniert) | Konfigurationsordner |
| `ui-prefs.json` (Darstellung, Onboarding-Status) | Konfigurationsordner |
| `scripts/<id>.md` (Skriptbibliothek) | Datenordner |
| `models/<id>/…`, optional `models/manifest.json` | Datenordner |

Zum Deinstallieren entfernst du die App und löschst diese beiden Ordner.
