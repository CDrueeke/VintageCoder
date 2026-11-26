# VintageCoder: Überblick

VintageCoder ist eine Entwicklungsumgebung für 6502-Systeme: **C64**, **C128**, **C16** und **Plus/4**, **VC-20**, **Atari 8-Bit** und **MEGA65**, mit eigenem Assembler, BASIC, Emulator-Anbindung und Editoren für Grafik und Klang.

## Der schnelle Einstieg
1. Auf der **Startseite** (Bereich **Code**, solange kein Projekt offen ist) ein Projekt aus der Liste **Zuletzt geöffnete Projekte** öffnen, **Projekt öffnen** oder **Beispiel öffnen** wählen (Liste aus dem Ordner `examples/`) oder mit **Neues Projekt** den Dialog **Neues Projekt erstellen** öffnen (siehe [Startseite, Konfigurationsassistent und neues Projekt](help:wizard)). Danach zeigt der Projektbaum, was das Projekt enthält; die [Projekt-Übersicht](help:overview) fasst den Zustand zusammen.
2. Im Projektbaum eine `.asm`- oder `.bas`-Datei öffnen.
3. **F7** baut, **F5** baut und startet das Startprogramm auf dem Startziel (Emulator, siehe [Einstellungen](help:settings)). **F6** startet das Programm mit einer Debug-Sitzung im Bereich **Debug** (siehe [Debugger](help:debugger)).
4. **Cmd/Strg+K** öffnet die Befehlspalette: Sie findet alle Befehle und Dateien des Projekts.

## Wo finde ich was?
- Quelltext schreiben: [Editor](help:editor), [Assembler](help:assembler), [BASIC](help:basic)
- Bauen und starten: [Bauen und Starten](help:build)
- Fehler suchen: [Debugger](help:debugger), [Haltepunkte](help:breakpoints), [Memory Map](help:memmap), [Tests](help:tests)
- Grafik und Klang: [Sprites, Zeichensätze und Songs](help:graphics), [Export](help:export), [Tracker](help:tracker), [Hüllkurve](help:envelope)
- Module und Disketten: [Segmente, Bänke, Module](help:cartridges), [Disk-Editor](help:disks)
- Neues System: [MEGA65](help:mega65)
- Fehler melden: [Feedback senden](help:feedback)
- Neue Version suchen: [Nach Updates suchen](help:updates)
- Weitere Themen: [Pfadfelder](help:paths), [Markdown](help:markdown), [Zahlenwerkzeuge](help:numbers), [BASIC in Assembler](help:basic2asm), [Konfiguration sichern](help:config)
- Bedienung: [Einstellungen](help:settings), [Takt in der Statusleiste](help:clock), [Projekt-Übersicht](help:overview), [Tastenkürzel](help:shortcuts), [Oberfläche anpassen](help:layout), [Dateibaum](help:filetree)

**Projekt bauen:** F7 baut alle Programme laut Projektdatei, F5 startet das Startprogramm; mit **Alt+Umschalt+F7** und **Alt+Umschalt+F5** gilt die aktive Datei. Quellen, Startprogramm und Zielsysteme siehst und änderst du im Hauptbereich **Projekt** (Cmd/Strg+Alt+0).
