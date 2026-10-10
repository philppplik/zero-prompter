# Rechtliche Einschätzung: zero-teleprompter

> **Keine Rechtsberatung.** Das ist eine Recherche-Zusammenfassung (Stand 2026-10-10). Punkte mit **[ANWALT]** muss ein Patent- oder IT-Anwalt bestätigen, bevor das Produkt verkauft wird.
> Legende: 🟢 geringes Risiko · 🟡 offen bzw. Handlungsbedarf · 🔴 blockiert den Release, bis geklärt

## Übersicht

| # | Thema | Ampel | Kurzfazit |
|---|-------|-------|-----------|
| 1 | Patent US 9,953,646 (PromptSmart/VoiceTrack) | 🟡 | Laut Google Patents „Expired - Fee Related“. Den Status selbst nicht geprüft, USPTO-Bestätigung fehlt |
| 1b | Weitere Patente (Microsoft US11902690B2 / EP4423748B1) | 🟡 | Ansprüche nicht gelesen |
| 2 | Modelllizenzen | 🟡/🔴 | Parakeet CC-BY-4.0 ok (Attribution nötig). Kroko-de: Lizenz unklar, evtl. nur nicht-kommerziell → 🔴 |
| 3 | Lizenzen der Abhängigkeiten | 🟢/🟡 | MIT/Apache/MPL-2.0 sind kompatibel; Audit mit cargo-deny fehlt |
| 4 | Code Signing (Windows/macOS) | 🟢 | Azure Artifact Signing ist für EU-Organisationen verfügbar; Apple Developer Program |
| 5 | DSGVO | 🟢 | Rein lokale Verarbeitung ohne Telemetrie. Kaum Pflichten, Datenschutzhinweis trotzdem empfohlen |

## 1. Patente

### US 9,953,646 B2: „Method and system for dynamic speech recognition and tracking of prewritten script“
Quelle: https://patents.google.com/patent/US9953646B2/en
- Inhaber: Belleau Technologies LLC (PromptSmart). Priorität 2014-09-02, Anmeldung 2015-09-02, Erteilung 2018-04-24.
- **Status laut Google Patents: „Expired - Fee Related“.** Daneben steht ein angepasstes Ablaufdatum 2036-03-25 – das ist das theoretische Datum *bei* gezahlten Gebühren. Google kennzeichnet seinen Status ausdrücklich als Annahme, nicht als Rechtsfeststellung. Die konkreten Lapse-Daten in den Legal Events habe ich **nicht** gesehen. Fällig wären die Jahresgebühren in Jahr 3,5 (≈2021-10), 7,5 (≈2025-10) und 11,5. „Fee Related“ deutet auf eine nicht gezahlte Gebühr hin.
- Eine Wiedereinsetzung wegen unabsichtlicher Säumnis (37 CFR 1.378) ist grundsätzlich auch später möglich. → **[ANWALT]**: Fee-Status und Wiedereinsetzungsrisiko im USPTO Patent Center prüfen (https://patentcenter.uspto.gov, Anmeldung 14/843,923).
- Familie: Gefunden habe ich nur US20160062970A1. Ein EP- oder DE-Familienmitglied war **nicht** sichtbar. In DE/EU besteht daraus also vermutlich kein Schutz (nicht vollständig verifiziert, z. B. über Espacenet).
- Es gibt Rechtsstreitigkeiten (S.D.N.Y. 1:18-cv-06319, 1:21-cv-04607). Der Inhaber setzt das Patent also aktiv durch bzw. hat es durchgesetzt.
- Anspruch 1 grob (paraphrasiert): Text anzeigen → Sprache in „Hypothesewörter“ umwandeln → eine *vorbestimmte Anzahl* Hypothesewörter mit einer *vorbestimmten Anzahl* Skriptwörter vergleichen → Match-Position bestimmen, wenn eine bestimmte Zahl übereinstimmt → Anzeige aktualisieren. **Außerdem:** Schriftgröße und Bildschirmausrichtung ermitteln und die Anzahl der verglichenen Skriptwörter ändern.
- **Abgrenzung (technische Einschätzung, keine Rechtsmeinung):** Unser Ansatz aus lokalem STT und Fuzzy-Alignment über ein Fenster ähnelt dem Kern. Die Kombination mit „Schriftgröße/Ausrichtung bestimmen und davon abhängig die Vergleichsfensterlänge ändern“ sollten wir **bewusst vermeiden**: feste bzw. nur textbasierte Fenstergröße, keine Kopplung an Font oder Orientierung. Ein Fuzzy- bzw. phonetisches Alignment ist kein exakter Wortanzahl-Match. Das kann für die Abgrenzung helfen, entscheidet sie aber nicht. **[ANWALT] für die US-Vermarktung, falls das Patent wieder in Kraft ist.**

### Weitere Patente (zitieren US 9,953,646)
- Microsoft „Machine learning driven teleprompter“: US11902690B2 / **EP4423748B1** (EP, also in DE potenziell relevant!) – https://patents.google.com/patent/US11902690B2/en → Ansprüche **nicht gelesen**. 🟡 **[ANWALT]**
- Google WO2020050820A1 „Reading progress estimation based on phonetic fuzzy matching“ – thematisch sehr nah an Fuzzy-Alignment. Ob daraus Patente erteilt wurden, ist nicht geprüft. 🟡
- Amazon US10042880B1 (Startposition beim Vorlesen eines E-Books), RU2862260C1 (Anzeige auf dem Teleprompter-Bildschirm): wahrscheinlich nur Randthemen.
- **To-do:** Freedom-to-Operate-Recherche (DE/EP/US) durch einen Patentanwalt. Kosten typisch im niedrigen vierstelligen Bereich (Schätzung, nicht belegt).

## 2. Modelllizenzen

| Modell | Lizenz | Kommerziell | Pflichten | Quelle |
|---|---|---|---|---|
| NVIDIA Parakeet TDT 0.6B v3 (25 Sprachen inkl. DE) | CC-BY-4.0 | ja | Namensnennung, Lizenzlink, Änderungen kenntlich machen | https://huggingface.co/nvidia/parakeet-tdt-0.6b-v3 |
| Kroko-ASR (Banafo), DE-Streaming-Zipformer | **unklar**. Ein Drittanbieter nennt „Dual-license (Free for non-commercial use)“ | **nicht belegt** | – | https://www.promptlayer.com/models/kroko-asr , https://huggingface.co/csukuangfj/sherpa-onnx-streaming-zipformer-de-kroko-2025-08-06 |
| Whisper (OpenAI) | MIT (laut Wissensstand, in dieser Sitzung nicht neu geprüft) | ja | Copyright- und Lizenzhinweis beilegen | https://github.com/openai/whisper/blob/main/LICENSE |
| Andere sherpa-onnx-Zipformer | je nach Modell (oft Apache-2.0), pro Modell prüfen | – | – | Model Cards |

- **CC-BY-4.0 hat kein ShareAlike.** Unsere App wird also nicht „infiziert“. Ein separater Download ist unproblematisch, solange in der App bzw. im Download-Dialog Folgendes steht: „Parakeet TDT 0.6B v3 © NVIDIA, CC BY 4.0, Link, ggf. ‚konvertiert nach ONNX/int8‘“. Erst bei CC-BY-**SA**-Modellen wären *Änderungen am Modell* (z. B. Quantisierung) wieder unter SA zu stellen. Die App selbst bleibt auch dann frei, weil sie nur ein separates Werk nutzt (Einschätzung).
- **🔴 Kroko-de:** Nicht ausliefern und keinen Download anbieten, bevor die Lizenz im offiziellen Repo (Banafo/Kroko-ASR) gelesen bzw. eine kommerzielle Lizenz bei Banafo eingeholt ist.

## 3. Lizenzen der Abhängigkeiten
- MIT, Apache-2.0 (bzw. Dual), BSD, ISC, Zlib und Unicode-3.0 sind untereinander und für proprietären Vertrieb kompatibel. Pflicht: Lizenztexte und NOTICE-Dateien (Apache §4) mit ausliefern.
- MPL-2.0 (z. B. in Tauri-Abhängigkeiten wie `cssparser`/`selectors`) gilt dateibezogen als Copyleft: Nur *geänderte* MPL-Dateien müssen offengelegt werden, die Nutzung als Bibliothek ist frei. Ein Hinweis auf den Quellcode-Ort ist nötig.
- **Nicht** geprüft: der konkrete Abhängigkeitsbaum (nichts kompilierbar). To-do: `cargo deny check licenses` mit Allowlist [MIT, Apache-2.0, BSD-2/3, ISC, Zlib, Unicode-3.0, MPL-2.0], ausdrücklich ohne GPL/AGPL/LGPL-statisch. Zusätzlich `cargo about` für THIRD_PARTY_LICENSES.html und `license-checker` für npm. Native Libs wie onnxruntime (MIT) und sherpa-onnx (Apache-2.0) gesondert aufführen.

## 4. Code Signing
- **Windows / Azure Artifact Signing** (früher Trusted Signing): GA laut devclass 2026-01 in USA, Kanada und Europa, für **Organisationen in EU/UK**. Einzelentwickler sind nur in USA/Kanada zugelassen. Der Zertifikatsname entspricht der validierten juristischen Person; für eine GmbH vermutlich der Handelsregisterauszug (nicht belegt). Zur Wahl stehen ein Basic- und ein Premium-SKU. Die Preise habe ich in dieser Sitzung nicht verifiziert (früher ca. 9,99 bzw. 99,99 USD/Monat). Quellen: https://learn.microsoft.com/en-us/azure/artifact-signing/overview , https://devclass.com/2026/01/14/code-signing-windows-apps-may-be-easier-and-more-secure-with-new-azure-artifact-service/ . Nutzer berichten von abgelehnten EU-Validierungen: https://learn.microsoft.com/en-us/answers/questions/5934347/
- Alternative OV/EV bei Sectigo, DigiCert, SSL.com, Certum: Seit 2023 nur noch mit Hardware-Token oder Cloud-HSM. Kosten ca. 200–700 €/Jahr (Schätzung, nicht belegt). EV bringt bei SmartScreen **keinen** Sofort-Reputationsvorteil mehr. Die Reputation baut sich über Downloads auf.
- **macOS:** Apple Developer Program (99 USD/Jahr, Stand Wissensbasis). Als Organisation braucht man eine D-U-N-S-Nummer. Developer-ID-Signatur und Notarisierung (`notarytool`) sind in Tauri integriert.
- To-do: D-U-N-S beantragen, Azure-Identitätsvalidierung früh starten (Dauer: Tage bis Wochen).

## 5. DSGVO
- Mikrofon-Audio wird nur lokal verarbeitet, ohne Übertragung und ohne Telemetrie. Damit gibt es beim Anbieter **keine Verarbeitung personenbezogener Daten**, denn wir sind nicht Verantwortlicher für die lokale Nutzung. Das ist das günstigste Szenario.
- Trotzdem nötig: ein Datenschutzhinweis (Website und App). Er muss sagen, dass keine Daten übertragen werden und wohin die Modell-Downloads gehen (Hugging Face o. ä.). Der Abruf übermittelt die IP-Adresse an den Drittanbieter, das gehört in den Hinweis oder lässt sich über einen eigenen Mirror vermeiden. Gleiches gilt für Update-Checks und Crash-Reports: nur opt-in.
- Audio und Transkripte nicht auf Platte speichern bzw. nur mit opt-in. Auf macOS ist `NSMicrophoneUsageDescription` Pflicht.
- Impressum (§ 5 DDG) auf der Website, AGB/EULA für den Verkauf. **[ANWALT]** für EULA und Haftungsausschluss.

## To-dos (priorisiert)
1. 🔴 Kroko-de-Lizenz klären oder auf Parakeet bzw. Apache-Zipformer ausweichen.
2. 🟡 **[ANWALT]** FTO: US 9,953,646 (Fee-Status/Wiedereinsetzung), EP4423748B1, Google WO2020050820.
3. 🟡 Fenstergröße des Alignments nicht an Font oder Orientierung koppeln (Designentscheidung dokumentieren).
4. 🟡 `cargo deny` + `cargo about` + npm-Lizenzreport in die CI; THIRD_PARTY_LICENSES und ein Modell-Attributionsdialog in die App.
5. 🟢 Azure Artifact Signing (Firmenvalidierung) und Apple Developer Program mit D-U-N-S beantragen.
6. 🟢 Datenschutzhinweis, Impressum, EULA (**[ANWALT]**).

## Was ein Anwalt zwingend bestätigen muss
- Patentstatus und FTO (US, EP/DE), insbesondere Wiedereinsetzbarkeit von US 9,953,646 und die Microsoft-EP-Ansprüche.
- Kommerzielle Nutzbarkeit der Kroko-Gewichte.
- EULA, Haftung, Gewährleistung, Datenschutzhinweis.
