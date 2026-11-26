# Handprüfung E1.3: Bauen und Starten

Stand: 2026-11-20 · Branch `claude/ui-e1-3` (aus `claude/ui-e1-2`) · App: `~/Development/abnahme-ui/e1-3/VintageCoder.app` (unsigniert: Rechtsklick ▸ Öffnen) · Beispielkopien in `beispiele/` (`c64/jumprun` mit `common/` daneben, `build/` ist **nicht** vorbereitet: Bauen legt es an).

**Zu wissen:** Dieser Stand baut und startet Programme, zeigt das Ergebnis (Ausgabe, Probleme, Statuszeile, Speicherstreifen, Zyklen je Zeile) und führt Tests aus. **Noch nicht** (E6, Debug-Sitzung): Debuggen (F6), „Ab hier starten“ (Bis hierher), Register, Flags und das Bild der laufenden Maschine, Haltepunkte im laufenden Programm.

Schritt 0: Port 1420 frei (`lsof -nP -iTCP:1420 -sTCP:LISTEN` leer), App starten, Oberfläche erscheint. Sprache Deutsch, Tastatur QWERTZ. Projekt öffnen: `…/beispiele/c64/jumprun`.

| Nr | Handlung | Erwartet | Ergebnis |
|---|---|---|---|
| 1 | Oben rechts Knopf „Bauen“ (Hammer) oder F7 | Statuszeile unten: „baut …“, danach „gebaut: n Bytes“ mit Speicherstreifen daneben; der Knopf ist während des Baus gesperrt | |
| 2 | Auf „gebaut: …“ in der Statuszeile klicken | Ablage unter dem Editor zeigt den Reiter „Ausgabe“ mit „Bauen: jumprun.asm …“, den Stufen des Baus und „Fertig: n Bytes in build/jumprun.prg“ | |
| 3 | Eine Quelle öffnen (`jumprun.asm`), nach unten scrollen | Im Rand rechts neben den Zeilennummern stehen Zyklenzahlen je Anweisung (blass), „2“ oder „4–5“; sie bleiben bei Eingaben an ihrer Zeile | |
| 4 | Maus auf den Speicherstreifen in der Statuszeile | Tooltip „Belegter Speicher: $0801–$…“ | |
| 5 | Neue Datei `kaputt.asm` mit `.target c64`, `.org $0801`, `lda nirgendwo`; im Reiter ⌘⌥F7 bzw. Befehlspalette „Bauen (aktive Datei)“ | Meldung „Bauen fehlgeschlagen (1 Fehler)“, Statuszeile „Bau: 1 Fehler“, Ablage zeigt „Probleme“ mit der Gruppe „Bauergebnis“ und der Zeile; Klick springt in die Datei an die Zeile | |
| 6 | Rechtsklick auf ein Problem | Menü: „Problem: Zur Zeile springen“, „Problem: Meldung kopieren“, „Alle Probleme kopieren“; jeder Eintrag tut, was er sagt (Einfügen in anderes Programm prüfen) | |
| 7 | Reiter „Ausgabe“, Rechtsklick in die Ausgabe | „Protokoll kopieren“ und „Protokoll leeren“ wirken | |
| 8 | F7 ohne offene Datei (alle Reiter schließen) | Baut die Hauptquelle des Projekts (`jumprun.asm`); in einem Projekt ohne `.asm`-Datei erscheint die sichtbare Meldung „Keine Quelldatei zum Bauen …“ | |
| 9 | Geänderte, ungespeicherte Datei, dann F7 | Alle Reiter werden zuerst gespeichert, dann wird gebaut | |
| 10 | Oben rechts Knopf „Starten“ (Dreieck) oder F5 | Baut und startet das Programm im Emulator des Systems (VICE für C64) oder auf der gewählten Maschine; die Ausgabe zeigt „Gestartet: …“; ein „Beenden“-Knopf erscheint | |
| 11 | Knopf „Beenden“ | Der Emulator wird beendet; Meldung „Emulator beendet.“ | |
| 12 | Emulator ist nicht eingerichtet (vorher in den Einstellungen deaktivieren oder umbenennen) und F5 | Sichtbare Fehlermeldung „Starten fehlgeschlagen: …“ und Eintrag in der Ausgabe; kein stilles Nichts | |
| 13 | Startziel-Chip (rechts oben, „C64 · VICE x64sc …“) anklicken | Menü mit dem Emulator des Systems und eingerichteten Maschinen; die Wahl bleibt (✓) und wird beim Starten benutzt | |
| 14 | Ansicht ▸ Panel (⌘⌥B) | Rechts erscheint „Maschine“: System, Startziel, Zustand, Tabelle des letzten Baus (Quelle, Programm, Größe, Start, Segmente); Register, Flags und Bild folgen mit der Debug-Sitzung (steht im Panel) | |
| 15 | Ansicht ▸ Schublade (⌘J) | Ablage unter dem Editor ein- und ausblenden | |
| 16 | In `jumprun/` Datei `jumprun.vctest` anlegen (siehe unten), dann Projekt ▸ Tests ausführen (Alt+F8) | Ablage „Tests“: „2 von 3 bestanden“, Liste mit ●, Zustand, Messwert; Wahl „Ziel“: „Nur statische Prüfungen“ oder ein Emulator | |
| 17 | Einen fehlgeschlagenen Test anklicken | Detail mit Erwartung und Ist-Wert (hier: „… Bytes zu viel“); „Testdatei öffnen“ springt in die Datei | |
| 18 | „nur Fehler“ und Filtertext | Liste zeigt nur die passenden Ergebnisse | |
| 19 | „Fehlgeschlagene“ | Nur die fehlgeschlagenen Tests laufen erneut | |
| 20 | Befehlspalette ⌘K: „bau“, „start“, „test“ tippen | Befehle erscheinen mit Kürzeln und sind nutzbar; „Debuggen“ meldet, dass es mit der Debug-Sitzung kommt | |
| 21 | Hell/Dunkel, Englisch, Pseudo | Kopfzeile, Statuszeile, Ausgabe, Probleme, Tests, Maschine wechseln mit; nichts bleibt deutsch | |
| 22 | Fenster auf 900×600 | Kopfzeile bleibt benutzbar (Knopf „Bauen“, „Starten“, Chip), nichts läuft aus dem Bild | |
| 23 | Prettyprint beim Verlassen der Zeile: in `.asm` `lda   nirgendwo      ; x` tippen und **Enter** drücken; ebenso eine fehlerfreie Zeile `lda   #1` und Pfeil nach oben/unten | Die Zeile wird gesetzt (Befehl in Spalte 9, Kommentar ab Spalte 41), **auch bei Zeilen mit Fehler** und auch beim Verlassen mit Enter; ein ⌘Z nimmt es zurück und es wird nicht erneut formatiert | |

Inhalt von `jumprun.vctest` für Punkt 16:
```json
{ "kind": "test", "target": "c64", "program": "build/jumprun.prg", "static": [ { "check": "overlap" }, { "check": "size", "max": 100000 }, { "check": "size", "max": 1 } ] }
```
