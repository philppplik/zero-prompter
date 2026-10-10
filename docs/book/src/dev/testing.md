# Tests

> Keiner der hier beschriebenen Tests ist bisher gelaufen, siehe [Projektstatus](../status.md).

## Rust

| Ort | Art | Inhalt |
|---|---|---|
| `crates/*/src/**` `#[cfg(test)]` | Unit | Tokenisierung, Grenzwerte der Settings, Laden der eingebetteten Tokens, Smith-Waterman, Ähnlichkeit (Referenzwerte MARTHA/MARHTA 0,9611), Resampler (Länge, Durchlass- und Sperrband), SHA-256-Testvektoren, Manifest-Validierung, `persist::valid_id` |
| `crates/zt-align/tests/prop_normalize.rs`, `prop_align.rs` | proptest | Invarianten der Normalisierung und Ausrichtung |
| `crates/zt-core/tests/playback_table.rs` | tabellengetrieben | Automat Wiedergabe × Modus |
| `crates/zt-core/tests/script_props.rs` | proptest | Skriptmodell (Byte-Bereiche, Absätze) |
| `crates/zt-testkit/tests/tracker_scenarios.rs` | Szenarien | `fixtures::SCENARIOS`: Versprecher, Rücksprung, Vorwärtssprung, Stille, Ad-lib. Kategorien nach EdNutting/autocue (MIT) |
| `crates/zt-testkit/tests/pipeline.rs` | Integration | MockEngine → Tracker → Controller, simuliert in 50-ms-Schritten |
| `crates/zt-testkit/tests/controller.rs` | Integration | Controller-Verhalten (Scroll-Modi, Lost-Policy, Override) |

Hilfen aus `zt-testkit`:

```rust
use zt_testkit::{Speaker, SpeechTimeline, Pipeline, ControllerBuilder};

let mut s = Speaker::default_de();          // Tracker auf SCRIPT_DE, explizite Uhr
s.say("Willkommen zu meinem Kanal");        // wachsende Partials, dann Final
assert_eq!(s.state(), zt_align::VoiceState::Tracking);

let engine = SpeechTimeline::new(2.0).say("Heute zeige ich euch").pause(1.5).build();
let mut p = Pipeline::new(zt_testkit::SCRIPT_DE, engine);
p.run_secs(5.0);
```

(Das Beispiel dient nur zur Veranschaulichung und wurde nicht ausgeführt. Signaturen: siehe `crates/zt-testkit/src/`.)

Property-Tests laufen in der CI mit der Standardanzahl von Fällen, wöchentlich mit `PROPTEST_CASES=8192`. Die proptest-Version ist `1` ohne Default-Features. Erhöht eine neue 1.x-Version die MSRV über 1.80, muss die Version gepinnt werden (`// VERIFY` im Workspace-`Cargo.toml`).

## Frontend (Vitest)

`apps/desktop/src/lib/*.test.ts`:

| Datei | Prüft |
|---|---|
| `hotkeys.test.ts` | `chordOf`, `normalizeChord`, `bindingMap` |
| `palette.test.ts` | unscharfe Suche, Hervorhebung, Aktionssuche |
| `scroller.test.ts` | `buildLayout`, `yOfPosition`, `stepSpring`, `predict` |
| `tokens.test.ts` | Theme-Presets aus `tokens.json`, `contrastRatio`, `prompterVars`, `applyUiTokens` |

Gemeinsame Helfer stehen in `src/test/helpers.ts`. Ausführen mit `npm test`, Typprüfung mit `npm run check`.

## Manuelle Abnahme (vor jedem Release)

1. Das Beispielskript konstant mit 140 WPM abspielen. Am Ende muss der Zustand `ended` erreicht werden.
2. Voice im Demo-Modus einschalten. Die Statusanzeige muss `listening → tracking` zeigen.
3. Mit echtem Modell: Ad-lib, Rücksprung, Absatz überspringen, 5 s schweigen. Erwartet werden `hold`, `resync` und `lost` wie dokumentiert.
4. Talent-Fenster auf einem zweiten Monitor öffnen, Vollbild, Spiegeln H/V.
5. Globale Hotkeys mit fokussierter Fremd-App ausprobieren.
