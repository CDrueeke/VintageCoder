# PET/CBM (in dieser Version nicht enthalten)

PET/CBM ist in den Paketen dieser Version **nicht enthalten** (in der Release-Auswahl abgewählt, ausgeblendet). Es gibt keinen Eintrag im Projekt-Assistenten, keine Emulator-Einstellung, keinen Start mit F5 und keinen Debugger. Die folgenden Angaben beschreiben den Datenstand im Quelltext (Bau aus dem Quelltext, `vcc`); sie sind ohne Zusage und ungeprüfte Teile sind als solche gekennzeichnet.

## PET/CBM (`pet`)
- Target `pet`: CPU 6502 (`nmos6502`, **kein** Port bei `$00/$01`), Textbildschirm bei `$8000` (40 Spalten: 1000 Byte, 80 Spalten: 2000 Byte, **keine Farbe, keine Sprites**), BASIC 2 oder 4 (Dialekt `cbm_v2`), Programme ab `$0401`; `.basicstub` erzeugt `SYS 1037` bei `.org $0401`.
- Chips: `petpia1` (`$E810`, Tastatur, Kassette), `petpia2` (`$E820`, IEEE-488), `petvia` (`$E840`, Benutzeranschluss, Ton über das Schieberegister), `crtc` (`$E880/$E881`, nur 8032 und größer). Konstanten: `#include <pet/hardware.asm>` und `#include <pet/kernal.asm>` (KERNAL-Sprungtabelle `$FFC0–$FFEA`).
- Bauen: `vcc asm prog.asm --target pet -o prog.prg`, BASIC: `vcc basic prog.bas --target pet`, Projekt: `vcc build ordner` (Projektdatei mit `"system": "pet"`). Vorlagen `asm-hello` und `basic-hello` gibt es für `pet` (`vcc new proj --target pet --template asm-hello`), Beispiele unter `examples/pet`.
- Text: Die PET-Modelle 2001 bis 4032 starten im **Groß-/Grafikmodus**; schreibe Text im Quelltext deshalb **klein** (`.petscii "hallo"` erscheint als `HALLO`). Der 8032 startet mit Groß-/Kleinschreibung. `CHR$(147)` löscht den Bildschirm.
- Start von Hand: `xpet -model 4032 -autostart prog.prg` (auch `2001`, `8032` und alle weiteren Modelle aus `xpet -help`).
- **Geprüft** (VICE 3.10, macOS): die Assembler-Vorlage läuft auf 2001, 4032 und 8032; das BASIC-Beispiel auf 4032. **Ungeprüft:** alle Registeradressen der Chip-Dateien (aus dem Gedächtnis), die Zyklenangaben (nur der Takt steht in der Datei), SuperPET, 8096/8296 mit Bankumschaltung.

## C16/C116
Gehört zum Target `plus4` (siehe *C128 und Plus/4*): `xplus4 -model c16`.
