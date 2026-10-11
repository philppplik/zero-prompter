# ADR-006: Hybride Spracherkennung – Streaming-Zipformer + optionaler Offline-Resync

- **Status:** Angenommen
- **Datum:** 2026-10-10
- **Kontext-Dokumente:** `docs/research/MODELS.md`, ADR-005, `crates/zt-stt`, `crates/zt-align/src/tracker.rs`

## Kontext

Voice-Tracking braucht eine Hypothese mit p95 ≤ 350 ms Latenz. Das leisten nur
Streaming-Transducer (sherpa-onnx `OnlineRecognizer`), deren Wortfehlerrate auf
freier Rede aber spürbar höher ist. Große Offline-Modelle (Parakeet TDT 0.6B v3)
sind deutlich genauer, liefern aber erst nach Segmentende ein Ergebnis und
kosten ~670 MB und mehrere hundert Millisekunden CPU pro Segment. Nach einem
Versprecher, Ad-lib oder Sprung soll der Tracker trotzdem zuverlässig wieder
an die richtige Stelle finden.

## Entscheidung

1. **Streaming bestimmt den Cursor.** Pro Primärsprache ein Streaming-Modell:
   Standard DE `zipformer-de-kroko-2025-08-06`, Standard EN
   `zipformer-en-kroko-2025-08-06`, Fallback EN `zipformer-en-20m-2023-02-17`
   (Apache-2.0). Reihenfolge: Nutzerwahl (falls installiert) → `defaultRank`
   im Manifest. Scheitert der Aufbau, wird der nächste installierte Kandidat
   versucht (`zt_stt::build_with_fallback`); ohne Modell Demo-Modus.
2. **Resync ist optional (Stufe L, standardmäßig aus).** `HybridEngine`
   schneidet den Audiostrom in Sprachsegmente (Silero VAD, sonst RMS),
   übergibt sie per `try_send` an eine begrenzte Queue und dekodiert sie auf
   dem Thread `zt-resync` mit `OfflineRecognizer`
   (`model_config.transducer`, `model_type = "nemo_transducer"`). Ist die
   Queue voll, wird das Segment verworfen – der Streaming-Pfad blockiert nie.
3. **Re-Anker statt Ersatz.** Ergebnisse kommen als `SttEvent::Resync { text,
   age_ms }` und gehen an `VoiceTracker::resync`, das bestätigt, ignoriert oder
   den Cursor neu verankert (`ResyncOutcome`). Ein manueller Sprung (`reset`)
   erhöht eine Generation; ältere Ergebnisse werden verworfen.
4. **Manifest v2:** `modelType`, `roles`, `defaultRank`, `sha256` optional.
   Dateien ohne Hash (z. B. `tokens.txt`, kein LFS) werden nur größengeprüft,
   ihr SHA-256 nach dem Download berechnet und als „unverifiziert“ angezeigt;
   das Manifest bleibt ladbar.
5. **Ports:** `SpeechEngine` (Streaming), `OfflineEngine`, `Segmenter`, jeweils
   mit Mock (`MockEngine`, `MockOfflineEngine`, `RmsSegmenter`) für CI ohne
   Modelle; sherpa-Adapter hinter dem Feature `sherpa`.

## Konsequenzen

- + Niedrige Latenz bleibt erhalten, Genauigkeit des Re-Ankers steigt.
- + Vollständig mit Mocks testbar (`zt-stt` Unit-Tests, `zt-testkit/tests/hybrid_resync.rs`).
- − Zusätzliche CPU-Last und ~670 MB Download bei aktiviertem Resync.
- − Die sherpa-Adapter sind gegen docs.rs 1.13.8 geschrieben; in CI wird nur
  der Feature-Build geprüft (nicht blockierend), Laufzeittests mit echten
  Modellen fehlen noch.
- − SHA-256 der `tokens.txt` vor einem Release pinnen.
