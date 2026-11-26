# Handprüfung E6 Debug (mit echtem Emulator)

Start: `start-debug-c64.command` (jumprun, öffnet den Bereich Debug). Voraussetzung: VICE ist eingerichtet. Die Ablauftests prüfen die Oberfläche gegen eine Attrappe; hier geht es um den echten Emulator.

| # | Schritte | Erwartet | Ergebnis |
|---|---|---|---|
| 1 | Bereich **Debug** → „Debuggen starten“ (oder Befehl „Startprogramm debuggen“) | Baut, startet VICE mit Debug-Sitzung; Status „Angehalten bei $… jumprun.asm:…“ oder „Läuft“ | |
| 2 | Im Code-Bereich am linken Rand einen Haltepunkt setzen (Klick oder F9) auf eine ausgeführte Zeile | Er wird in die Sitzung übernommen (Liste links „Editor jumprun.asm:n“); das Programm hält dort | |
| 3 | Weiter (▶ oder Alt+F5), Pause (Alt+F6) | Läuft/hält; Register, Disassembler, Speicher erscheinen nur im Halt | |
| 4 | Schritt über (F10), hinein (F11), hinaus (Umschalt+F11) | PC folgt; geänderte Register umrandet, „Schritt über“ überspringt JSR | |
| 5 | Haltepunkt im Disassembler per Klick auf den Kreis links; zweiter Klick löscht | Roter Punkt; Eintrag in der Liste | |
| 6 | Haltepunkt mit Bedingung (z. B. `a == 5`) und Ignorierzähler; Fehlerhafte Bedingung | Zählt Treffer; fehlerhafte Bedingung: Meldung | |
| 7 | Wachpunkt (Lesen/Schreiben/Zugriff) auf `$D020` (nur wo der Adapter es kann: Ultimate, MEGA65) | Hält beim Zugriff; bei VICE ist die Auswahl ausgegraut | |
| 8 | Beobachten: `a`, `$d020`, `d020`, ein Symbol | Wert hex · dezimal im Halt; `d020` → „Meinst du $d020?“ | |
| 9 | Speicher: Adresse `D020`, `$0400`, ein Symbol; ungültig `xyz` | 256 Bytes mit ASCII; geänderte Bytes hervorgehoben; Meldung mit Beispielen | |
| 10 | Bis hierher ausführen (Cursorzeile im Code-Bereich, Befehl „Debuggen: Bis hierher ausführen“) | Läuft bis dorthin und hält | |
| 11 | Im Halt: Momentaufnahme → „Erkennen“ → Optionen → „Als Dateien sichern“ | Neue Medien im Projekt (Bildschirm, Zeichensatz, Sprites) | |
| 12 | Bildschirm (rechts): Bild aktualisieren, „bei jedem Halt“, „Live (Zeitlupe)“; Text an den Emulator senden | Bild des Emulators; Text erscheint in der Maschine | |
| 13 | Debuggen beenden (Umschalt+F5 oder ■) | Sitzung weg, **kein VICE-Prozess bleibt zurück**, Ausführungszeile im Editor verschwindet | |
| 14 | Emulator von außen beenden, während die Sitzung läuft | Verständliche Meldung („Verbindung zum Emulator ist verloren“), Sitzung endet sauber | |
| 15 | MEGA65 per Ethernet als Ziel, Debuggen | Hinweis „Debuggen geht nur über USB“ | |
| 16 | **VoiceOver:** Symbolleiste, Disassembler-Kreise, Register, Haltepunkt-Formular | Knöpfe mit Namen („Haltepunkt bei $080A, Schalter“), Statuszeile wird gelesen | |

Noch nicht: „Diesen Test debuggen“ aus der Testliste (kommt mit dem Test-Bereich), eigene Chip-Fenster (waren in der bisherigen Oberfläche nicht angeschlossen).

## E6-Rest: Chip-Fenster, PC in der Statusleiste, Maschine-Panel, Test debuggen

Bilder: `~/Development/abnahme-ui/e6-reste/bilder` (`debug-11-chips`, `debug-12-panel`, `debug-13-test-sitzung`). Die Ablauftests laufen gegen die Attrappe; **Punkte 17 und 18 brauchen einen echten Emulator (VICE) und sind nur von Hand prüfbar.**

| # | Schritte | Erwartet | Ergebnis |
|---|---|---|---|
| 17 | Debug: links bei „Chips“ VIC-II anhaken, im Halt schauen; weiterlaufen lassen, wieder anhalten | Register des Chips mit Namen und Wert ($xx); beim nächsten Halt ändern sich die Werte (z. B. `$D020` Rahmenfarbe); SID, CIA nach Wahl | |
| 18 | Tests-Panel: Test wählen, „Diesen Test debuggen“ (VICE nötig) | Bereich Debug, Kennzeichen „Test-Sitzung“, Emulator wartet am Anfang der Falle; Schritte und Haltepunkte wie sonst | |
| 19 | Während einer Sitzung: Statusleiste unten | „Debug: angehalten bei $xxxx“, im Lauf „läuft“; Klick wechselt in den Debug-Bereich | |
| 20 | Rechts das Maschine-Panel öffnen (Bereich Code) | Abschnitt „Debug-Sitzung“ mit PC, A X Y, SP, Flags; „Bild aktualisieren“ holt ein Einzelbild | |
