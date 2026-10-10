# Aligner-Algorithmus

Code: `crates/zt-align`. Der Aligner ist rein und deterministisch: Er macht kein I/O, liest keine Uhr und hat keine Abhängigkeiten.

## Pipeline

```
STT-Hypothese ─▶ normalize_hypothesis ─▶ letzte k Wörter (Query)
   ─▶ Smith-Waterman im Fenster um den Cursor ─▶ (sonst) globale Suche
   ─▶ Hysterese + Zustandsautomat (VoiceTracker) ─▶ cursor_token ─▶ zt-core
```

## 1. Normalisierung (`normalize.rs`)

- Kleinschreibung und Umlaut-Folding (`ä→ae`, `ß→ss`, `é→e`), sodass „Straße“ und „Strasse“ gleich sind.
- Apostrophe und Bindestriche werden zusammengezogen (`geht's→gehts`, `E-Mail→email`). Andere Satzzeichen trennen Wörter.
- Ziffern werden zu Zahlwörtern der gewählten Sprache (`2026` → `zweitausendsechsundzwanzig` bzw. `two thousand twenty six`). Zahlen ab 1 000 000 werden Ziffer für Ziffer gesprochen. Tausendertrennzeichen werden erkannt.
- Füllwörter (`äh`, `ähm`, `uh`, `um`) entfernt die Normalisierung nur aus **Hypothesen**, nicht aus dem Skript.

`ScriptIndex` speichert die normalisierten Skriptwörter (`words`) und ordnet jedes Wort seinem Token zu (`owner`). Ein Token kann mehrere normalisierte Wörter erzeugen (Zahlen), Regieanweisungen erzeugen keine. Für eine Regieanweisung liefert `norm_of_token` das nächste gesprochene Wort.

## 2. Wortähnlichkeit (`similarity.rs`)

`word_similarity(a, b)` liefert einen Wert in [0, 1]:
- Bei Gleichheit 1.
- Ist eines der Wörter kürzer als 4 Zeichen **oder** unterscheidet sich der Anfangsbuchstabe, gilt nur die Levenshtein-Ratio (`1 − dist/max_len`). So gelten „und“/„uns“ und „haus“/„maus“ nicht als Treffer.
- Sonst gilt `max(Levenshtein-Ratio, Jaro-Winkler)`. Das ist robust gegen Endungsfehler der Spracherkennung („teleprompta“).

## 3. Gefenstertes Smith-Waterman (`sw.rs`)

Die lokale Ausrichtung der Query (k = 6 Wörter, Finals mit Gewicht 1,0, Partials mit 0,8) gegen `reference[range]`:

| Parameter (`SwParams`) | Standard | Bedeutung |
|---|---|---|
| `sim_threshold` | 0,8 | ab hier zählt ein Wortpaar als Treffer |
| `match_score` | 2,0 | Treffer: `match_score · sim · weight` |
| `mismatch` | −1,5 | |
| `gap_script` | −1,0 | Skriptwort übersprungen |
| `gap_hyp` | −1,0 | zusätzliches Wort (Ad-lib, Einfügung der Spracherkennung) |
| `distance_penalty` | 0,01 | Tie-Break: pro Wort Abstand zur erwarteten Position |
| `max_trailing` | 1 | die Ausrichtung muss auf einem der letzten `1 + max_trailing` Query-Wörter enden |

Gültige Endzellen sind nur Treffer-Zellen. `SwHit.end` ist das Skriptwort, das zuletzt gesprochen wurde. `confidence = score / max_possible`, gerechnet über das Query-Suffix ab Beginn der Ausrichtung. Ältere Kontextwörter verwässern einen frischen Treffer also nicht, nicht passende Wörter am Ende dagegen schon. Der Aufwand ist O(k·w), bei k = 6 und w = 70 sind das etwa 420 Zellen.

## 4. Tracker und Hysterese (`tracker.rs`)

| `TrackerConfig` | Standard |
|---|---|
| Fenster `window_before` / `window_after` | 10 / 60 Wörter um `expected = cursor + 1` |
| `k` | 6 |
| `theta` / `min_matches` (lokal) | 0,55 / 2 |
| `theta_global` / `min_matches_global` | 0,7 / 4 |
| `jump_words` (Vorwärtssprung) | 18 |
| `backtrack_words` (Rücksprung ohne Bestätigung) | 10 |
| `jump_confirmations` / `jump_tolerance` | 2 / 4 |
| `lost_timeout_ms` (Sprechzeit ohne Treffer) | 4000, aus den Settings |
| `resync_ease_ms` | 300 |
| `manual_override_ms` | 2000, aus den Settings |

Ablauf `on_hypothesis`:

1. Hypothese normalisieren. Ein Final wird in den Puffer übernommen (höchstens `3k` Wörter), ein Partial ersetzt den vorherigen Partial. Die Query besteht aus den letzten `k` Wörtern. Ist die Query unverändert, passiert nichts.
2. **Lokale Suche** im Fenster. Bei einem Treffer gilt: Liegt er höchstens `backtrack_words` zurück und höchstens `jump_words` voraus, wird er **sofort akzeptiert**. Sonst (oder im Zustand `lost`) ist er ein **Sprungkandidat**.
3. Ohne lokalen Treffer folgt die **globale Suche** über das ganze Skript, mit strengeren Schwellen. Ein Treffer dort wird zum Sprungkandidaten.
4. **Sprung:** Er wird erst ausgeführt, wenn `jump_confirmations` aufeinanderfolgende Kandidaten konsistent sind (innerhalb von `jump_tolerance` bzw. `2·jump_tolerance` Wörtern). Dann wechselt der Zustand für 300 ms nach `resync`, danach nach `tracking`. Bis dahin steht der Zustand auf `hold`.
5. Die Sprechrate (`rate_wps`) ist ein EMA (0,8/0,2) aus Wortfortschritt pro Zeit und wird über Sprünge hinweg nicht berechnet.

## Zustandsautomat

| Von | Ereignis | Nach |
|---|---|---|
| idle | Start | listening |
| listening | Treffer (conf ≥ θ) | tracking |
| tracking | Treffer | tracking |
| tracking | VAD-Stille / kein Treffer | hold |
| hold | Treffer nahe Cursor | tracking |
| hold/tracking | Sprechzeit ohne Treffer > `lost_timeout_ms` | lost |
| tracking/hold/lost | Sprung, 2× bestätigt | resync |
| resync | nach 300 ms | tracking |
| * | Stop | idle |
| * | ManualSeek | hold (Stimme wird `manual_override_ms` ignoriert) |

## Herkunft

Die Behandlung von Rücksprüngen und Vorwärtssprüngen (kleine Rücksprünge sofort, große nur nach Bestätigung) und die Kategorien der Szenariotests folgen dem Ansatz von **EdNutting/autocue** (MIT). Die Implementierung ist anhand der Beschreibung neu geschrieben, es wurde kein Code kopiert, siehe `THIRD_PARTY_NOTICES.md`. Levenshtein, Jaro-Winkler und Smith-Waterman sind nach den Lehrbuchdefinitionen implementiert.

## Tuning

Die erwarteten Werte in `zt-testkit/src/fixtures.rs` sind von Hand aus dem Algorithmus abgeleitet. Bricht eine Parameteränderung ein Szenario, leite den Erwartungswert neu her, statt ihn einfach anzupassen.
