# Bauen und Starten

- **Bauen (F7):** baut das Projekt, also alle Programme der Projektdatei in Baureihenfolge; hat das Projekt keine Quellen, die aktive Datei. Mit Umschalt gilt immer die aktive Datei. Ausgabe in `build/`: Programm (`.prg`, `.xex`, `.crt`, `.car`), `.vs` (Labels für VICE), `.lst` (Listing), `.vcdbg` (Debug-Information).
- **Starten (F5):** baut und startet auf dem Startziel des Zielsystems: im Standard-Emulator oder auf einer eingetragenen Maschine, siehe [Einstellungen](help:settings). Hat das Projekt eine Diskette und steht das Startprogramm vorn, startet VICE die Diskette (damit sich Programme nachladen lassen).
- **Debuggen (F6):** startet im Debugger, siehe [Debugger](help:debugger). Das Feld „Auf:“ in der Befehlsleiste wählt statt des Standards eine Maschine, etwa eine Ultimate 64.
- **Target:** „automatisch“ nimmt `.target` aus dem Quelltext; sonst die Auswahl im Feld „Auf:“ der Befehlsleiste (Abschnitt „Ziel“).
- Das Ergebnis je Quelle und als letzten Build zeigt die [Projekt-Übersicht](help:overview).
- **Sprites, Zeichensätze, Bildschirme und Songs** (`font.vcchar`, `hero.vcsprite` …) werden beim Bauen automatisch nach `build/export/<name>.asm` exportiert – einbinden mit `.include "../build/export/hero.asm"`.
- **Format:** das Standardformat des Targets (PRG, XEX); steht `.cartridge` im Quelltext, entsteht ein Modul (`.crt` / `.car`).

Auf der Kommandozeile: `vcc asm datei.asm -o ausgabe --target c64 --format prg`.

## Bauprotokoll
Nach jedem Bau öffnet sich unten das Panel **Bauprotokoll**. Es zeigt je Programm die Stufen des Baus mit Dauer und Meldungen und führt alle beteiligten Dateien auf:

| Stufe | Assembler | BASIC |
|---|---|---|
| Vorbereitung | Quelle, Zielsystem, CPU, Include-Suche | Quelle, Zielsystem, Dialekt |
| Medien exportieren | veraltete Sprites, Zeichensätze, Songs nach `build/export/` | – |
| Quelltext und Includes | – | Includes eingesetzt, Zeilen je Datei |
| Assemblieren / Übersetzen | Dateien, Makros, Durchläufe, Symbole, Segmente, Startadresse, Zyklenbudgets, Warnungen | Strukturen auflösen, Zeilennummern, Variablen, Labels |
| Ausgabe schreiben | Format und Größe des Programms | Größe, Ladeadresse |
| Zwischendateien | Quelltext nach Includes (`.expanded.asm`), Listing (`.lst`), Symboltabelle (`.sym.txt`), Speicherbelegung (`.map.txt`), VICE-Labels (`.vs`), Debuginformation (`.vcdbg`) | BASIC mit Zeilennummern (`.classic.txt`), Variablen (`.vars.txt`), Labels (`.labels.txt`) |

Jede Datei lässt sich öffnen: Textdateien mit einem Klick auf den Namen im Editor, jede Datei mit **Hex** als Hexdump und mit **Zeigen** im Dateimanager. Bei Fehlern steht die fehlgeschlagene Stufe mit den Meldungen im Protokoll.

## Das Fußpanel beim Bauen
Treten Fehler auf, öffnet sich das Fußpanel mit dem Reiter **Probleme** (auch wenn du es eingeklappt hattest); ein Klick auf eine Meldung springt zur Zeile. Ein fehlerfreier Bau zeigt das **Bauprotokoll**, aber nur, wenn das Fußpanel ohnehin offen ist: hast du es bewusst eingeklappt, bleibt es zu. Schlägt der Bau ohne Meldung zu einer Zeile fehl, zeigt der Reiter **Ausgabe** den Grund.

## Auf der Kommandozeile
Dasselbe geht ohne Oberfläche mit `vcc build` und `vcc asm`; alle Befehle und Optionen stehen unter [Kommandozeile](help:cli).
