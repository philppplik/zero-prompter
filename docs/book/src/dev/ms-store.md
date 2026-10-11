# Microsoft Store

As of 2026-10. Sources named in the original notes: the Tauri documentation "Microsoft Store" (https://v2.tauri.app/distribute/microsoft-store/) and Microsoft Learn "Publish your MSI or EXE app". The statements about Partner Center requirements, fees and certification below come from those notes. They were **not re-verified** when this page was translated and may have changed (marked **(unverified)** where it matters). Check the current Microsoft pages before you submit.

## Which route?

Tauri 2 does **not** produce MSIX packages, only MSI and NSIS EXE installers. The route Tauri recommends is therefore an **MSI/EXE listing**: in Partner Center you create a Store page that links to an installer hosted by you, for example a GitHub release asset.

What the listing requires and how the repository handles it:

| Requirement | Implementation in the repository |
|---|---|
| The installer must install offline (no downloads during installation) | `apps/desktop/src-tauri/tauri.microsoftstore.conf.json` sets `bundle.windows.webviewInstallMode` to `offlineInstaller` (WebView2 is embedded in the installer; the original notes estimate about +130 MB, unverified) |
| Silent installation | NSIS: `/S`. MSI: `/quiet` |
| `publisher` differs from the product name | `bundle.publisher = "zero-teleprompter contributors"` in both `tauri.conf.json` and `tauri.microsoftstore.conf.json`. Adjust it to the publisher name from Partner Center |
| A fixed, versioned download URL | Release assets `zero-teleprompter_<version>_<arch>_store-setup.exe` (`<arch>` is `x64` or `arm64`), produced by the `store` job in `release.yml` |
| **Code signature** of the installer with a certificate from a trusted CA | **Open.** For MSI/EXE listings the Store does not sign for you |
| Updates | Through a new installer URL or version in Partner Center. An in-app updater is permitted but not required. The app has no updater (see [Project status](../status.md)) |

The static C runtime (see [Build and release](build-release.md)) matters for the Store installer too: the `store` job runs `scripts/check-crt.ps1` on the built executable.

**Important about signing:** only **MSIX** packages are signed by Microsoft at publication. For MSI/EXE the installer (and the EXE inside it) must already be signed, or certification fails **(unverified)**. A low-cost option named in the original notes is **Azure Artifact Signing** (formerly "Trusted Signing", roughly 10 USD per month, identity verification required, unverified); an OV or EV certificate is the alternative. Tauri can sign through `bundle.windows.signCommand`. That is prepared conceptually but cannot be enabled until a certificate or signing service exists; nothing in the repository configures it today.

**MSIX alternative** (Microsoft signs; no certificate of your own): Tauri has no bundler for it. The built app could be packaged manually with the MSIX Packaging Tool or `makeappx`, using the *Package/Identity/Name* and *Publisher* (`CN=...`) values from Partner Center. This is not implemented because it lies outside Tauri's standard path. It is the way out if signing costs are a problem. In an MSIX build the manifest might also need the `microphone` capability for the Windows speech fallback (open question, see [ADR-008](../adr/index.md)).

## Step by step

### 1. Create a developer account

1. Open https://storedeveloper.microsoft.com and sign in with a Microsoft account.
2. Choose the account type. **Individual:** according to the original notes, registration has been free for individual developers since 2025 (identity check with ID and selfie). **Company:** a one-time fee of about 99 USD plus company verification. Fees change occasionally, so check them on Microsoft's page before registering **(unverified)**.
3. Wait for verification (hours to a few days).

### 2. Reserve the app name

1. In Partner Center choose **Apps and games**, then **New product**, then **EXE or MSI app**.
2. Reserve a name, for example "zero-teleprompter". The name must be unique in the Store.

### 3. Copy values into the repository

- Under **Product identity** or **Account settings**, read the **publisher display name**. Enter it as `bundle.publisher` in `tauri.conf.json` **and** `tauri.microsoftstore.conf.json`. It must not equal the product name.
- For a later MSIX you would also need *Package/Identity/Name* and *Publisher* (`CN=...`). They are not needed for MSI/EXE.
- Have the installer signed (see above). Create the certificate or signing-service credentials as GitHub secrets and add `signCommand` to the configuration.

### 4. Build

Run `gh workflow run release.yml` or push a tag `vX.Y.Z`. The draft release then contains, besides the regular installers:

- `zero-teleprompter_<version>_x64_store-setup.exe`
- `zero-teleprompter_<version>_arm64_store-setup.exe`

Publish the draft. Partner Center needs a publicly reachable HTTPS URL that does not change, so use a new URL for every version and never overwrite an existing file. (The `store-upload` job uses `gh release upload --clobber` on the **draft**; do not re-upload to an already published release.)

To build locally, mirror the CI step (run on Windows, from `apps/desktop`, after `npm ci` and `npx tauri icon app-icon.png`):

```bash
npx tauri build --bundles nsis --config src-tauri/tauri.microsoftstore.conf.json --features sherpa,winspeech
```

The installer is written to `target/release/bundle/nsis/`.

### 5. Submit (Partner Center)

The following form details come from the original notes and are **unverified**:

1. **Pricing and availability:** free; choose markets.
2. **Properties:** category *Productivity*. Provide a privacy policy URL. The app works offline; models are downloaded only on request from Hugging Face and GitHub.
3. **Age rating:** complete the questionnaire.
4. **Packages:** per architecture, enter the installer URL (x64 and Arm64), installer type *EXE*, silent parameter `/S`.
5. **Store listing:** description (German and English), at least one screenshot (1366x768 or larger) and logos. `npx tauri icon` generates the Store logos (`StoreLogo.png`, `Square*Logo.png`).
6. **Submit.** Certification usually takes 1 to 3 business days. Typical rejection reasons: missing signature, a non-silent installation, an installer that downloads during setup, and a missing privacy policy.

### 6. Updates

Build the new version, publish the release, enter the new URL under **Packages** in Partner Center and submit again.
