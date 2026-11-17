# Bauen und Starten

- **Projekt bauen (F7):** baut das Projekt, also alle Programme der Projektdatei in Baureihenfolge; hat das Projekt keine Quellen, die aktive Datei. Mit **Alt+Umschalt+F7** gilt immer die aktive Datei („Aktive Datei bauen“). Ausgabe in `build/`: Programm (`.prg`, `.xex`, `.crt`, `.car`), `.vs` (Labels für VICE), `.lst` (Listing), `.vcdbg` (Debug-Information).
- **Startprogramm starten (F5):** baut und startet das Startprogramm (★ im Projektbaum), egal welche Datei gerade offen ist. Der Kopfzeilenknopf **Starten** tut dasselbe. Mit **Alt+Umschalt+F5** startest du stattdessen die **Datei im aktiven Editor-Reiter** („Aktive Datei starten“). Beide starten auf dem Startziel des Zielsystems: im Standard-Emulator oder auf einer eingetragenen Maschine, siehe [Einstellungen](help:settings). Hat das Projekt eine Diskette und steht das Startprogramm vorn, startet VICE die Diskette (damit sich Programme nachladen lassen). Das Startprogramm legst du in den Projekt-Einstellungen fest (Reiter Quellen, Stern vor der Datei).
- **Startprogramm debuggen (F6):** startet im Debugger, siehe [Debugger](help:debugger). **Alt+Umschalt+F6** debuggt die aktive Datei.
- Alle sechs Befehle stehen im Menü **Projekt**.
- **Startziel:** Das Zielfeld in der Kopfzeile („C64 · VICE x64sc ▾“) wählt, wo das Programm läuft: im Emulator des Systems oder auf einer eingetragenen Maschine, etwa einer Ultimate 64.
- **Target:** das Zielsystem steht in `.target` im Quelltext, sonst in den Projekt-Einstellungen.
- **Sprites, Zeichensätze, Bildschirme und Songs** (`font.vcchar`, `hero.vcsprite` …) werden beim Bauen automatisch nach `build/export/<name>.asm` exportiert – einbinden mit `.include "../build/export/hero.asm"`.
- **Format:** das Standardformat des Targets (PRG, XEX); steht `.cartridge` im Quelltext, entsteht ein Modul (`.crt` / `.car`).

Auf der Kommandozeile: `vcc asm datei.asm -o ausgabe --target c64 --format prg`.

## Das Protokoll
Die Schublade hat die Reiter **Protokoll**, **Probleme** und **Tests** (in einer Debug-Sitzung auch **Debug**). Beim Bauen wechselt sie zum Reiter **Protokoll**: Zeile für Zeile steht dort, was gebaut wurde („Fertig: 34 Bytes“ mit dem Pfad des Programms, bei einer Diskette deren Name) und was beim Starten und Debuggen passiert. Eine Tabelle mit den Stufen des Baus, mit Dauer und Dateiliste, gibt es in der Oberfläche noch nicht; auf der Kommandozeile zeigt `vcc build` den Ablauf. Die erzeugten Dateien (Listing, Symboltabelle, Speicherbelegung …) liegen in `build/` und lassen sich über den Projektbaum öffnen; im Rechtsklickmenü steht **Als Hex ansehen**.

## Die Schublade beim Bauen
Treten Fehler auf, wechselt die Schublade zum Reiter **Probleme**; ein Klick auf eine Meldung springt zur Zeile, ein Rechtsklick bietet „Zur Zeile springen“, „Meldung kopieren“ und „Alle Probleme kopieren“. Im Reiter **Protokoll** bietet der Rechtsklick „Protokoll kopieren“ und „Protokoll leeren“. **F8** und **Umschalt+F8** springen zum nächsten und zum vorigen Problem. Schlägt der Bau ohne Meldung zu einer Zeile fehl, zeigt der Reiter **Protokoll** den Grund. Ist die Schublade eingeklappt, blendest du sie mit **Cmd/Strg+J** ein.

## Auf der Kommandozeile
Dasselbe geht ohne Oberfläche mit `vcc build` und `vcc asm`; alle Befehle und Optionen stehen unter [Kommandozeile](help:cli).
