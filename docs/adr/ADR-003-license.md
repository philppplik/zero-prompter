# ADR-003: Project licence MIT OR Apache-2.0

> German original: [de/ADR-003-license.md](de/ADR-003-license.md)

- **Status:** accepted
- **Source:** taken over from `docs/research/RESEARCH.md` §4 (2026-10-10)

## Decision

zero-teleprompter is licensed under either MIT or Apache-2.0, at the user's option.

## Rationale and consequences

- Allows adopting MIT-licensed code and keeps the project commercially open.
- GPL code is excluded. `cargo-deny` and an npm licence check enforce this in CI.
- **Slint:** is under Royalty-free-2.0 (proprietary desktop use allowed, attribution required, embedded excluded; [licence text](https://unpkg.com/slint-ui@1.12.0/LICENSES/LicenseRef-Slint-Royalty-free-2.0.md)). For us it is only an emergency renderer.
- **NDI:** proprietary SDK, not part of the MVP.

## Addendum 2026-10-10: models

Models are downloaded separately and are not covered by the project licence. The licence status per model is listed in `THIRD_PARTY_MODELS.md`, `docs/research/MODELS.md` and `docs/legal/LEGAL.md`. The Kroko models (CC-BY-SA community license, commercial use under a separate license) were removed on 2026-10-11; all offered models are permissive, CC-BY-4.0 or OpenMDW-1.1.
