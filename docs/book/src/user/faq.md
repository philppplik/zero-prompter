# FAQ & Fehlerbehebung

## Allgemein

**Braucht die App Internet?**
Nein. Netzwerkzugriff gibt es nur, wenn du ein Sprachmodell herunterlädst. Die Tauri-CSP erlaubt sonst nur IPC.

**Wird meine Stimme aufgezeichnet oder hochgeladen?**
Nein. Die Audiodaten fließen nur durch den Arbeitsspeicher (Mikrofon → Resampler → Spracherkennung) und werden nicht gespeichert.

**Welche Sprachen gibt es?**
Deutsch und Englisch, für die Normalisierung und die Wahl des Modells. Jeder Sprachcode, der mit `en` beginnt, gilt als Englisch, alles andere als Deutsch.

## Probleme

**Voice bewegt den Text, obwohl ich nichts sage („Demo“).**
Dann läuft der **Demo-Modus**: Es ist kein Streaming-Modell installiert (Parakeet und Silero VAD allein reichen nicht), oder der Build enthält keine Spracherkennung. Klicke auf „Demo · Modell laden“ neben dem Voice-Status und lade z. B. „Kroko Streaming Zipformer DE“. Eine laufende Voice-Sitzung übernimmt das Modell nach dem Download automatisch. Siehe [Voice-Tracking](voice-tracking.md).

**„Modellfehler“ oder „Sprachmodell konnte nicht geladen werden“.**
Ein Modell ist installiert, lässt sich aber nicht öffnen (beschädigte oder fehlende Datei, zu wenig Speicher). Die Meldung nennt die Datei. Entferne das Modell unter Voice & Modelle und lade es neu. Details stehen im Diagnose-Tab und im Log.

**Log: „recognizer failed … failed to create OnlineRecognizer“ (nach ~1 ms).**
Bis einschließlich der ersten Testbuilds lag das an der Konfiguration, nicht am Modell: sherpa-onnx verlangt bei `modeling_unit = bpe` eine Datei `bpe.vocab`, die keines der angebotenen Modelle mitbringt. Behoben: Die App prüft vor dem Start alle Modelldateien (Name + Größe stehen als `preflight ok` im Log; fehlende Dateien werden einzeln genannt), erzeugt die für Hotwords nötige Vokabeldatei `zt-generated-bpe.vocab` aus `tokens.txt` und probiert der Reihe nach `beam+hotwords` → `beam` → `greedy`. Jede Stufe steht im Log (`stage … ok` / `stage … failed`). Scheitern alle Stufen, enthält die Meldung die Gründe je Stufe. Für die interne Ausgabe von sherpa-onnx die App aus einer Konsole mit `ZT_STT_DEBUG=1` starten (die C++-Bibliothek schreibt nur nach stderr). Pfade mit Umlauten können unter Windows Probleme machen; das Log warnt dann.

**„Mikrofon liefert keine Daten“ / „nur digitale Stille“.**
Das Gerät wurde geöffnet, liefert aber nichts. Unter Windows meist: Einstellungen → Datenschutz und Sicherheit → Mikrofon → „Desktop-Apps den Zugriff auf das Mikrofon erlauben“ ist aus, oder das Mikrofon ist stummgeschaltet. Auf macOS: Systemeinstellungen → Datenschutz → Mikrofon. Danach Mikrofontest erneut starten.

**„Mikrofon sehr leise (< −50 dBFS beim Sprechen)“.**
Das Mikrofon liefert Signal, aber viel zu wenig für die Spracherkennung (typisch: Surface/Snapdragon-Geräte mit „Microphone Array … Static Endpoints“, Pegel −85 bis −100 dBFS in Pausen, kaum über −45 dBFS beim Sprechen).
1. Windows: Einstellungen → System → Sound → Eingabe → das Mikrofon wählen → **Eingangslautstärke** auf 80–100 % stellen. Bei manchen Treibern gibt es dort zusätzlich „Audioverbesserungen“ bzw. „Sprachfokus“; ausprobieren.
2. Einstellungen → Voice & Modelle → **Mikrofon-Gain** auf „Auto“ lassen. Die App verstärkt dann digital (bis +40 dB) auf ca. −20 dBFS Sprachpegel; ein Limiter verhindert Übersteuern. „Manuell“ setzt eine feste Verstärkung von −20 bis +40 dB.
3. **Mikrofon-Kanal**: Mikrofon-Arrays liefern mehrere Kanäle, von denen einer leer oder gegenphasig sein kann. „Auto“ nimmt den lautesten Kanal (Standard), „Alle mischen“ mittelt (kann bei Arrays Pegel kosten), „Kanal N“ erzwingt einen Kanal. Der Diagnose-Tab zeigt den Pegel jedes Kanals und den gewählten Kanal.
4. Im Diagnose-Tab sollte „Mikrofon roh“ beim Sprechen über −50 dBFS liegen; ohne Windows-Anpassung hebt nur die digitale Verstärkung auch das Rauschen an.

**Sprache synchronisiert nicht, obwohl ein Modell geladen ist.**
1. Einstellungen → **Diagnose** öffnen und Voice starten.
2. „Engine“ muss den Modellnamen zeigen, „Audio“ ein Gerät, der Pegel beim Sprechen deutlich über −40 dBFS liegen.
3. „Erkannt“ zeigt, was die Spracherkennung hört. Passt das nicht zum Skript, stimmt die Sprache nicht (Voice & Modelle → Sprache).
4. Bleibt es unklar: **Log exportieren** und die Datei dem Fehlerbericht anhängen.

**Wo liegen die Logs?**
Im App-Log-Ordner (Pfad steht im Diagnose-Tab), Dateien `zero-teleprompter.JJJJ-MM-TT.log`, 7 Tage. Mit der Umgebungsvariable `ZT_LOG=debug` wird jede Hypothese mit Zeitstempel und Latenz protokolliert.
**Die Modellliste ist leer.**
Das ist so gewollt. Das mitgelieferte Manifest enthält in v0.1 keine Einträge, weil keine ungeprüften URLs oder Hashes ausgeliefert werden sollen.

**Der Text bleibt auf „Wartet“ (`hold`) stehen.**
Die App ist nicht sicher genug oder wartet auf die zweite Bestätigung eines Sprungs. Lies ein paar Wörter weiter, oder klicke auf das Wort, an dem du bist.

**„Verloren“ (`lost`) erscheint häufig.**
- Prüfe den Pegel im Mikrofontest. Er sollte beim Sprechen deutlich über −40 dBFS liegen.
- Stelle die richtige Sprache ein.
- Gehe näher ans Mikrofon oder nimm ein Headset.
- Setze bei Bedarf „Bei Verlust“ auf „konstant weiter“.

**Kein Mikrofon gefunden.**
- macOS: Systemeinstellungen → Datenschutz → Mikrofon → zero-teleprompter erlauben.
- Windows: Einstellungen → Datenschutz → Mikrofon → „Desktop-Apps den Zugriff erlauben“.
- Linux: ALSA- bzw. PipeWire-Gerät prüfen.

Dann in den Einstellungen unter Voice & Modelle das Gerät wählen.

**Globale Hotkeys gehen nicht.**
Die Kombination ist schon belegt (das steht dann im Log `zt: global shortcut … unavailable`), oder du nutzt Wayland. Bei fokussiertem Fenster funktionieren die App-Tasten weiterhin.

**Linux mit NVIDIA: Ruckeln oder schwarzes Fenster.**
Starte die App mit `ZT_SAFE_RENDER=1`.

**Import schlägt fehl.**
Unterstützt sind `.txt`, `.md`, `.markdown` und `.text` bis 8 MB. `.docx` kommt in v0.2. Sonderzeichen sehen falsch aus? Speichere die Datei als UTF-8.

**Das Skript lässt sich nicht bearbeiten.**
Während der Wiedergabe ist der Text gesperrt. Pausiere oder stoppe, dann `Ctrl+E`.

**Das Talent-Fenster ist weg.**
Beim Schließen wird es nur ausgeblendet. Über die Befehlspalette → „Talent-Fenster öffnen“ kommt es zurück. Schließt du das Control-Fenster, beendet sich die App.

## Bekannte Einschränkungen von v0.1

- Der Code ist noch nie kompiliert worden, siehe [Projektstatus](../status.md).
- Keine Fernsteuerung, kein Overlay, keine eigenen Themes, kein Updater.
- Den Monitor für das Talent-Fenster setzt die App per Fensterposition: Sie verschiebt das Fenster um 40 px versetzt auf den Zielmonitor und schaltet dann Vollbild ein. Manche Fenstermanager ignorieren das.
