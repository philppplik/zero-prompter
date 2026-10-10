# FAQ & Fehlerbehebung

## Allgemein

**Braucht die App Internet?**
Nein. Netzwerkzugriff gibt es nur, wenn du ein Sprachmodell herunterlädst. Die Tauri-CSP erlaubt sonst nur IPC.

**Wird meine Stimme aufgezeichnet oder hochgeladen?**
Nein. Die Audiodaten fließen nur durch den Arbeitsspeicher (Mikrofon → Resampler → Spracherkennung) und werden nicht gespeichert.

**Welche Sprachen gibt es?**
Deutsch und Englisch, für die Normalisierung und die Wahl des Modells. Jeder Sprachcode, der mit `en` beginnt, gilt als Englisch, alles andere als Deutsch.

## Probleme

**Voice bewegt den Text, obwohl ich nichts sage.**
Dann läuft der **Demo-Modus**: Die App wurde ohne Feature `sherpa` gebaut, oder es ist kein Modell für die eingestellte Sprache installiert. Der Inspector zeigt das an. Siehe [Voice-Tracking](voice-tracking.md).

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

Dann im Inspector unter Voice das Gerät wählen.

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
