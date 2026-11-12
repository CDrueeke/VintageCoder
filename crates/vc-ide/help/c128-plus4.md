# C128 und Plus/4

Zwei weitere Commodore-Targets, beide mit eigenem BASIC-Dialekt und Emulator (VICE).

## Commodore 128 (`c128`)
- **C128-Modus**, CPU `8502` (6502-Befehlssatz wie 6510, Prozessorport `$00/$01`). Der C64-Modus bleibt Target `c64`.
- BASIC 7.0 (Dialekt `basic7`), Programme ab `$1C01`; `.basicstub` erzeugt `SYS 7181` bei `.org $1c01`.
- Chips: `vic2`, `vic2e` (`$D02F` Tastatur, `$D030` 2-MHz-Modus), `sid`, `cia1`, `cia2`, `mmu128` (`$D500`), `mmu128cfg` (`$FF00`–`$FF04`), `vdc` (80 Spalten, `$D600/$D601`).
- Grafik und Klang: VIC-II- und SID-Profile des C64 (Sprites, Zeichensätze, Bildschirme, Kacheln, Tracker) sind auch im C128-Projekt verfügbar.
- F5 startet `x128 -autostart`; der Debugger spricht den Binär-Monitor von VICE.

## Commodore Plus/4 (`plus4`) und C16/116 (`c16`)
- CPU `7501` (6502-Befehlssatz, Port `$00/$01`), Chip **TED** (`ted.*`: Farben, Raster `ted.raster_lo`, Timer, zwei Tonstimmen), `acia6551`, `plus4io`.
- BASIC 3.5 (Dialekt `basic35`), Programme ab `$1001`; `.basicstub` erzeugt `SYS 4109` bei `.org $1001`.
- Der TED hat keine Sprites und keinen SID: es gibt noch keine Profile für Sprites, Zeichensätze oder Songs dieses Systems (der „Neu …“-Dialog meldet das).
- F5 startet `xplus4 -autostart`.
- **C16/C116:** eigenes Target `c16` (Beispiel `examples/c16/border`): gleiche Hardware und Chips wie der Plus/4, aber nur **16 KB RAM** (12277 Byte frei für BASIC; Anwenderspeicher nur bis `$3FFF`) und kein Benutzeranschluss. BASIC-Anfang (`$1001`), Bildschirm (`$0C00`) und Farb-RAM (`$0800`) sind wie beim Plus/4. Die Systemdateien (`plus4/hardware.asm`, `plus4/kernal.asm`) und Bibliotheken des Plus/4 gelten unverändert. F5 startet `xplus4 -model c16 -autostart`. Ein Programm, das in die 16 KB passt, läuft auch auf dem Plus/4.

## Hinweise
- Registerlisten stammen aus den Handbüchern, nicht aus einem Emulator-Dump; Programme der Vorlagen laufen im VICE (Haltepunkt-Test mit `x128` und `xplus4`).
- Wizard: `vcc new proj --target c128,plus4 --template asm-hello`.
- Nicht enthalten: Z80-Modus, VDC-Grafik, Bank-Umschaltung (MMU) als Assembler-Hilfe, Kassetten-/Modulformate dieser Rechner.
