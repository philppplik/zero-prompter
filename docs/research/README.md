# Research

Background research behind zero-prompter. Most files are German research artifacts and are kept as written; this page is the English overview. Figures quoted here come from those files and were not re-verified.

| File | Language | Content |
|---|---|---|
| [MODELS.md](MODELS.md) | English | Speech models for voice tracking (sherpa-onnx): CI measurements (`stt-smoke`, 2026-10-11), desk-research comparison, recommendation, Windows speech fallback, open points |
| [MODELS-2.md](MODELS-2.md) | German | ASR model candidates and a proposed default strategy (as of 2026-10-11) |
| [MODELS-3-fast-de.md](MODELS-3-fast-de.md) | German | Fast German streaming ASR candidates (as of 2026-10-11); notes that some entries are unconfirmed ("?") |
| [RESEARCH.md](RESEARCH.md) | German | Research synthesis from 2026-10-10 |

## RESEARCH.md at a glance

Sections: (1) executive summary, (2) market and open-source landscape (feature matrix, commercial vendors, list of ideas taken over with attribution), (3) speech-to-text and voice tracking (engines, model tiers, alignment algorithm in `zt-align`, latency budget with a target of at most 250 ms and a realistic 200-350 ms still to be benchmarked, pipeline), (4) tech stack in ADR style, (5) UX and design principles, (6) architecture, tooling, CI, testing, docs, (7) risks and open questions, (8) sources.

Section 4 became ADR-001 to ADR-004; see the [ADR index](../book/src/adr/index.md).

## Where the research ended up

- Model decisions: [MODELS.md](MODELS.md), `docs/adr/ADR-006-hybrid-stt.md`, `docs/adr/ADR-007-voice-pipeline-tuning.md`, `docs/adr/ADR-008-windows-speech-fallback.md`.
- Product concept (German): [../concept/CONCEPT.md](../concept/CONCEPT.md) (vision, personas, feature scope, window model, user flows, state model, non-functional requirements, MVP plan v0.1, open product decisions).
- Design: [../design/README.md](../design/README.md).
- Legal: [../legal/README.md](../legal/README.md).

## Terminology

The documentation uses: control window, talent window, voice tracking, streaming model, resync.
