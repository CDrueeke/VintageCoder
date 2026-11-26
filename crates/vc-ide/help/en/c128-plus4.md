# C128 and Plus/4

Two more Commodore targets, both with their own BASIC dialect and emulator (VICE).

## Commodore 128 (`c128`)
- **C128 mode**, CPU `8502` (6502 instruction set like the 6510, processor port `$00/$01`). C64 mode stays the target `c64`.
- BASIC 7.0 (dialect `basic7`), programs start at `$1C01`; `.basicstub` creates `SYS 7181` at `.org $1c01`.
- Chips: `vic2`, `vic2e` (`$D02F` keyboard, `$D030` 2 MHz mode), `sid`, `cia1`, `cia2`, `mmu128` (`$D500`), `mmu128cfg` (`$FF00`-`$FF04`), `vdc` (80 columns, `$D600/$D601`).
- Graphics and sound: the VIC-II and SID profiles of the C64 (sprites, character sets, screens, tiles, tracker) are also available in a C128 project.
- F5 starts `x128 -autostart`; the debugger talks to the binary monitor of VICE.

## Commodore Plus/4 (`plus4`) and C16/116 (`c16`)
- CPU `7501` (6502 instruction set, port `$00/$01`), chip **TED** (`ted.*`: colors, raster `ted.raster_lo`, timers, two voices), `acia6551`, `plus4io`.
- BASIC 3.5 (dialect `basic35`), programs start at `$1001`; `.basicstub` creates `SYS 4109` at `.org $1001`.
- The TED has no sprites and no SID: there are no profiles yet for sprites, character sets or songs of this system.
- F5 starts `xplus4 -autostart`.
- **C16/C116:** its own target `c16` (example `examples/c16/border`): the same hardware and chips as the Plus/4, but only **16 KB RAM** (12277 bytes free for BASIC; user memory only up to `$3FFF`) and no user port. The start of BASIC (`$1001`), the screen (`$0C00`) and the color RAM (`$0800`) are the same as on the Plus/4. The system files (`plus4/hardware.asm`, `plus4/kernal.asm`) and libraries of the Plus/4 apply unchanged. F5 starts `xplus4 -model c16 -autostart`. A program that fits into 16 KB also runs on the Plus/4.

## Notes
- The register lists come from the manuals, not from an emulator dump; the programs of the templates run in VICE (breakpoint test with `x128` and `xplus4`).
- Dialog **Create new project** in the user interface or on the command line: `vcc new proj --target c128,plus4 --template asm-hello`.
- Not included: Z80 mode, VDC graphics, bank switching (MMU) as an assembler aid, cassette and cartridge formats of these computers.
