# PET/CBM und CBM-II (Spike, noch nicht in der Oberfläche)

Stand: **Datenstand für `vcc`**. PET/CBM und CBM-II sind noch **keine** Zielsysteme der Oberfläche (kein Eintrag im Wizard-Dialog der App, keine Emulator-Einstellung, kein Start mit F5, kein Debugger). Was schon geht, ist Bauen mit `vcc` und der Start von Hand im installierten VICE. Ungeprüftes ist als solches gekennzeichnet.

## PET/CBM (`pet`)
- Target `pet`: CPU 6502 (`nmos6502`, **kein** Port bei `$00/$01`), Textbildschirm bei `$8000` (40 Spalten: 1000 Byte, 80 Spalten: 2000 Byte, **keine Farbe, keine Sprites**), BASIC 2 oder 4 (Dialekt `cbm_v2`), Programme ab `$0401`; `.basicstub` erzeugt `SYS 1037` bei `.org $0401`.
- Chips: `petpia1` (`$E810`, Tastatur, Kassette), `petpia2` (`$E820`, IEEE-488), `petvia` (`$E840`, Benutzeranschluss, Ton über das Schieberegister), `crtc` (`$E880/$E881`, nur 8032 und größer). Konstanten: `#include <pet/hardware.asm>` und `#include <pet/kernal.asm>` (KERNAL-Sprungtabelle `$FFC0–$FFEA`).
- Bauen: `vcc asm prog.asm --target pet -o prog.prg`, BASIC: `vcc basic prog.bas --target pet`, Projekt: `vcc build ordner` (Projektdatei mit `"system": "pet"`). Vorlagen `asm-hello` und `basic-hello` gibt es für `pet` (`vcc new proj --target pet --template asm-hello`), Beispiele unter `examples/pet`.
- Text: Die PET-Modelle 2001 bis 4032 starten im **Groß-/Grafikmodus**; schreibe Text im Quelltext deshalb **klein** (`.petscii "hallo"` erscheint als `HALLO`). Der 8032 startet mit Groß-/Kleinschreibung. `CHR$(147)` löscht den Bildschirm.
- Start von Hand: `xpet -model 4032 -autostart prog.prg` (auch `2001`, `8032` und alle weiteren Modelle aus `xpet -help`).
- **Geprüft** (VICE 3.10, macOS): die Assembler-Vorlage läuft auf 2001, 4032 und 8032; das BASIC-Beispiel auf 4032. **Ungeprüft:** alle Registeradressen der Chip-Dateien (aus dem Gedächtnis), die Zyklenangaben (nur der Takt steht in der Datei), SuperPET, 8096/8296 mit Bankumschaltung.

## CBM-II (`cbm2`, 6x0/7x0, nur BASIC)
- Target `cbm2`: der 6509 hat ein **Bankregister bei `$00/$01`**. BASIC-Programme liegen in Bank 1 ab `$0003` (Ladeadresse `$0003`, vom Dialekt `cbm_v2` passend gebaut), das System in Bank 15. Chips: `cbm2crtc` (`$D800`), `cbm2sid` (`$DA00`), `cia1` (`$DC00`), `cbm2tpi1`/`cbm2tpi2` (`$DE00`/`$DF00`). Registeradressen **ungeprüft**.
- Bauen und Start: `vcc basic prog.bas --target cbm2 -o prog.prg`, dann `xcbm2 -model 610 -autostart prog.prg`. **Geprüft** (VICE 3.10): das BASIC-Beispiel (`examples/cbm2`) läuft auf dem 610.
- **Assembler gibt es für das CBM-II noch nicht:** `SYS` führt Code in der eingestellten Bank aus (Standard Bank 15), das geladene Programm liegt aber in Bank 1. Dafür braucht es Bankbehandlung im Assembler (Bank als Teil der Adresse, Lader, Zugriffe über das Indirektregister) und eine Bankansicht im Debugger. Die 5x0-Reihe (`xcbm5x0`: VIC-II statt CRTC) ist nur als Katalog `data/args/vice-xcbm5x0.json` vorbereitet.

## C16/C116
Gehört zum Target `plus4` (siehe *C128 und Plus/4*): `xplus4 -model c16`.
