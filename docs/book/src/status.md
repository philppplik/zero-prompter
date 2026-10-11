# Projektstatus

> **Ehrlicher Stand: Der Code wurde noch nie kompiliert oder ausgeführt.**

Das gesamte Repository (Rust, Svelte/TypeScript, CI) entstand auf einem Rechner ohne Rust, Node und Git. Daraus folgt:

| Bereich | Stand |
|---|---|
| Kern-Crates `zt-core`, `zt-align`, `zt-models`, `zt-testkit` | vollständig geschrieben, mit Unit-, Property- und Szenario-Tests. Nur Standardbibliothek plus serde, thiserror und sha2, deshalb ist das Risiko hier am geringsten. **Nicht ausgeführt.** |
| `zt-audio` mit Feature `cpal` | bewusst auf cpal 0.15 gehalten. Die aktuelle cpal 0.18.2 hat eine inkompatible API (`build_input_stream` nimmt `StreamConfig` per Wert, `name()` → `description()`/`id()`), belegt auf docs.rs. Die 0.15-API selbst ist nur aus dem Gedächtnis, offen: ob `Stream` `Send` ist |
| `zt-stt` mit Feature `sherpa` | neu geschrieben gegen **sherpa-onnx =1.13.8** (im Root-`Cargo.toml` gepinnt). Alle Typen und Signaturen auf docs.rs belegt, `unsafe impl Send` entfällt (Recognizer und Stream sind `Send + Sync`). Offen: Modell-Dateinamen, Windows-Download-Build |
| Tauri-App | geschrieben gegen Tauri 2. Ungeprüft: Handler-Signatur von `tauri-plugin-global-shortcut`, Permission-Namen, Plugin-Versionen, ureq 2 (`set`, `into_reader`; aktuell wäre ureq 3.4.2) |
| Frontend | Svelte 5 (Runes), Vitest-Tests für Hotkeys, Palette, Scroller und Tokens. Prettier-Konfiguration neu, aber noch nie gelaufen. **Kein `package-lock.json`.** |
| Modell-Manifest | 5 Modelle mit echten Größen und SHA-256 der `.onnx`-Dateien (Hugging-Face-API). **Noch nicht ladbar**, weil die `tokens.txt`-Hashes fehlen, siehe [`docs/research/MODELS.md`](../../research/MODELS.md) |
| Formatierung | `rustfmt.toml` (max_width 100, `use_small_heuristics = "Max"`), von Hand formatiert. `cargo fmt --check` und `clippy -D warnings` nie ausgeführt |
| Recht | [`docs/legal/LEGAL.md`](../../legal/LEGAL.md): Kroko-Modelle entfernt (lizenzsauberer Stack), Patent US 9,953,646 Status nur auf Google Patents gesehen, Anwaltsprüfung nötig |
| Doku | dieses Buch, abgeglichen mit dem Code-Stand vom 2026-10-10 |

Alle offenen Prüfpunkte findest du mit `rg "VERIFY"`. Die Checkliste für die erste Person mit Toolchain steht im README unter „Erste Schritte nach dem Klonen“.

Wo dieses Buch Verhalten beschreibt, ist das Verhalten **laut Code** gemeint, nicht beobachtetes Verhalten.
