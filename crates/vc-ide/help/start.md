# VintageCoder: Überblick

VintageCoder ist eine Entwicklungsumgebung für 6502-Systeme: **C64**, **C128**, **C16** und **Plus/4**, **VC-20**, **Atari 8-Bit** und **MEGA65**, mit eigenem Assembler, BASIC, Emulator-Anbindung und Studios für Grafik und Klang.

## Der schnelle Einstieg
1. Auf dem Tab **Willkommen** ein Projekt öffnen, ein Beispiel öffnen (Ordner `examples/`) oder mit **Neues Projekt** den [Wizard](help:wizard) starten. Danach zeigt die [Projekt-Übersicht](help:overview), was das Projekt enthält.
2. Im Projektbaum eine `.asm`- oder `.bas`-Datei öffnen.
3. **F7** baut, **F5** baut und startet auf dem Startziel (Emulator oder eingetragene Hardware, siehe [Einstellungen](help:settings)), **F6** startet den Debugger.
4. **F1** öffnet diese Hilfe; steht der Cursor im Editor auf einem Wort, zeigt sie dazu den Eintrag (Befehl, Direktive, Register, BASIC-Wort). Im Hilfe-Panel führen das Inhaltsverzeichnis und die Suche zu den Themen.

## Wo finde ich was?
- Quelltext schreiben: [Editor](help:editor), [Assembler](help:assembler), [BASIC](help:basic)
- Bauen und starten: [Bauen und Starten](help:build)
- Fehler suchen: [Debugger](help:debugger), [Haltepunkte](help:breakpoints), [Memory Map](help:memmap), [Tests](help:tests)
- Grafik und Klang: [Sprites, Zeichensätze und Songs](help:graphics), [Export](help:export), [Tracker](help:tracker), [Hüllkurve](help:envelope)
- Module und Disketten: [Segmente, Bänke, Module](help:cartridges), [Disk-Studio](help:disks)
- Neues System: [MEGA65](help:mega65)
- Fehler melden: [Feedback senden](help:feedback)
- Bedienung: [Einstellungen](help:settings), [Projekt-Übersicht](help:overview), [Tastenkürzel](help:shortcuts), [Oberfläche anpassen](help:layout)

Tipp: Die Suche oben im Hilfe-Panel findet Themen, Befehle, Direktiven und Chip-Register.


**Projekt bauen:** F7 baut alle Programme laut Projektdatei, F5/F6 starten bzw. debuggen das Startprogramm (★ im Projektbaum); mit Umschalt gilt die aktive Datei. Die Projekt-Einstellungen (Menü Projekt) bearbeiten Quellen, Startprogramm und Diskette grafisch.
