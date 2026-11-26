# Bauen und Starten

- **Projekt bauen (F7):** baut das Projekt, also alle Programme der Projektdatei in Baureihenfolge; hat das Projekt keine Quellen, die aktive Datei. Mit **Alt+Umschalt+F7** gilt immer die aktive Datei („Aktive Datei bauen“). Ausgabe in `build/`: Programm (`.prg`, `.xex`, `.crt`, `.car`), `.vs` (Labels für VICE), `.lst` (Listing), `.vcdbg` (Debug-Information).
- **Startprogramm starten (F5):** baut und startet das Startprogramm (★ im Bereich **Projekt** auf der Seite **Quellen**), egal welche Datei gerade offen ist. Der Knopf **Starten** in der Kopfzeile tut dasselbe. Mit **Alt+Umschalt+F5** startest du stattdessen die **Datei im aktiven Reiter** („Aktive Datei starten“). Beide starten auf dem Startziel des Zielsystems, siehe unten. Hat das Projekt eine Diskette und steht das Startprogramm vorn, startet VICE die Diskette (damit sich Programme nachladen lassen).
- **Beenden (Umschalt+F5):** beendet den laufenden Emulator. Der Knopf **Beenden** in der Kopfzeile erscheint, solange der Emulator läuft.
- **Debuggen (F6):** baut das Startprogramm, startet es im Emulator mit einer Debug-Sitzung und öffnet den Hauptbereich **Debug** (siehe [Debugger](help:debugger)). **Alt+Umschalt+F6** debuggt die aktive Datei. Ein Symbol in der Kopfzeile gibt es dafür nicht; im Bereich **Debug** startet der Knopf **Debuggen starten** die Sitzung. **Umschalt+F5** beendet sie. Bei Turbo BASIC XL gibt es das Debuggen noch nicht.
- Bauen und Starten stehen im Menü **Projekt**. Vor dem Bauen speichert VintageCoder alle offenen Dateien.
- **Startziel:** Der Knopf **Startziel** in der Kopfzeile („C64 · VICE x64sc ▾“) zeigt, wo das Programm läuft. Ein Klick öffnet die Auswahl. Das Startziel wählt nur, *wo* das Programm läuft; das Zielsystem steht in `.target` im Quelltext, sonst im Bereich **Projekt** auf der Seite **Ziele**. Emulator-Pfade und echte Hardware trägst du unter **Einstellungen ▸ Ziele und Hardware** ein, siehe [Einstellungen](help:settings).
- **Sprites, Zeichensätze, Bildschirme und Songs** (`font.vcchar`, `hero.vcsprite` …) bindest du mit der Datei selbst ein, zum Beispiel `.include "graphics/hero.vcsprite"`; beim Bauen werden sie umgewandelt (Meldung in der Ausgabe: „Medium umgewandelt“).
- **Format:** das Standardformat des Targets (PRG, XEX); steht `.cartridge` im Quelltext, entsteht ein Modul (`.crt` / `.car`).

Auf der Kommandozeile: `vcc asm datei.asm -o ausgabe --target c64 --format prg`.

## Die Ausgabe
Die Ablage unter dem Editor (**Cmd/Strg+J**) hat die Reiter **Probleme**, **Ausgabe**, **Tests**, **Suche** und **Gliederung**. Beim Bauen und Starten steht im Reiter **Ausgabe** Zeile für Zeile, was geschehen ist („Fertig: 34 Bytes“ mit dem Pfad des Programms, bei einer Diskette deren Name). Die Statusleiste zeigt „baut …“, danach „gebaut: N Bytes“ oder „Bau: N Fehler“; ein Klick darauf öffnet die Ablage. Die Ausgabe gliedert den Bau in Stufen (Zeilen mit „▸“); eine Tabelle mit Dauer und Dateiliste gibt es in der Oberfläche nicht, auf der Kommandozeile zeigt `vcc build` den Ablauf. Die erzeugten Dateien (Listing, Symboltabelle, Speicherbelegung …) liegen in `build/` und lassen sich über die Seitenleiste öffnen.

## Probleme
Treten Fehler auf, wechselt die Ablage zum Reiter **Probleme**; ein Klick auf eine Meldung springt zur Zeile, ein Rechtsklick bietet „Problem: Zur Zeile springen“, „Problem: Meldung kopieren“ und „Alle Probleme kopieren“. Im Reiter **Ausgabe** bieten die Knöpfe **Kopieren** und **Leeren** (und der Rechtsklick) „Protokoll kopieren“ und „Protokoll leeren (Ausgabe leeren)“. **F8** und **Umschalt+F8** springen zum nächsten und zum vorigen Problem. Schlägt der Bau ohne Meldung zu einer Zeile fehl, zeigt der Reiter **Ausgabe** den Grund. Ist die Ablage eingeklappt, blendest du sie mit **Cmd/Strg+J** ein.

## Das Panel Maschine
Das Panel **Maschine** (Bereich Code, **Cmd/Strg+Alt+B**) zeigt System, Startziel und Zustand („bereit“, „baut …“, „läuft“) und darunter den letzten Bau: Quelle, Programm, Größe, Start (SYS) und eine Tabelle der Segmente. Zur genauen Belegung siehe [Memory Map](help:memmap). Läuft eine Debug-Sitzung, zeigt es zusätzlich den Abschnitt **Debug-Sitzung** mit PC, A X Y, SP, Flags und, wenn der Emulator eines liefert, sein Bild (Knopf **Bild aktualisieren**).

## Auf der Kommandozeile
Dasselbe geht ohne Oberfläche mit `vcc build` und `vcc asm`; alle Befehle und Optionen stehen unter [Kommandozeile](help:cli).
