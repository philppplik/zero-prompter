# Legal notes (summary)

> **Not legal advice.** This is an English summary of a research document written on 2026-10-10 by a non-lawyer. Items marked **[LAWYER]** must be confirmed by a patent or IT lawyer before the product is sold. The full German original is [LEGAL.md](LEGAL.md); where this summary and the original differ, the original governs.

Traffic-light legend: green = low risk, yellow = open or action needed, red = blocks the release until resolved.

| # | Topic | Status | Summary |
|---|---|---|---|
| 1 | US 9,953,646 (PromptSmart/VoiceTrack) | yellow | Google Patents shows "Expired - Fee Related". The fee status was not checked against the USPTO; reinstatement after unintentional delay (37 CFR 1.378) may still be possible. **[LAWYER]** |
| 1b | Other patents (Microsoft US11902690B2 / EP4423748B1, Google WO2020050820A1) | yellow | Claims not read; the EP member may matter in Germany/EU. **[LAWYER]** |
| 2 | Model licenses | yellow / red | Parakeet TDT 0.6B v3: CC-BY-4.0, attribution required, no share-alike. Kroko (Banafo): license unclear, possibly non-commercial only, so release-critical. |
| 3 | Dependency licenses | green / yellow | MIT/Apache/BSD/ISC/Zlib/Unicode-3.0 compatible; MPL-2.0 is file-level copyleft. The actual dependency tree was not audited at the time (`cargo deny` pending). |
| 4 | Code signing | green | Windows: Azure Artifact Signing (organizations in EU/UK) or OV/EV certificates; macOS: Apple Developer Program with notarization. Prices were not verified. |
| 5 | GDPR | green | Audio is processed locally only, no telemetry. A privacy notice is still recommended (model downloads transmit the IP address to the host). |

## Design consequence recorded in the original

To stay clear of claim 1 of US 9,953,646, the alignment window size should not be coupled to font size or screen orientation (text-based or fixed window only). This is a technical observation, not a legal opinion.

## Prioritized to-dos (from the original)

1. Resolve the Kroko DE license or use Parakeet / an Apache-licensed Zipformer instead.
2. **[LAWYER]** Freedom-to-operate review (US, EP/DE).
3. Document the alignment-window design decision.
4. Add `cargo deny`, `cargo about` and an npm license report to CI; ship third-party licenses and a model attribution dialog.
5. Start Azure Artifact Signing validation and the Apple Developer Program (D-U-N-S) early.
6. Privacy notice, imprint, EULA **[LAWYER]**.

Note: since this assessment, the project has moved on (for example Nemotron 3.5 under OpenMDW-1.1 is now the default model, and a third-party notices file exists). The original has not been updated for this; check `THIRD_PARTY_MODELS` and [../research/MODELS.md](../research/MODELS.md) for current model licenses.
