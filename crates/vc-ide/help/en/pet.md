# PET/CBM (not included in this version)

PET/CBM is **not included** in the packages of this version (deselected in the release selection, hidden). There is no entry in the project assistant, no emulator setting, no start with F5 and no debugger. The following information describes the state of the data in the source code (build from source, `vcc`). It comes without any promise, and parts that have not been tested are marked as such.

## PET/CBM (`pet`)
- Target `pet`: CPU 6502 (`nmos6502`, **no** port at `$00/$01`), text screen at `$8000` (40 columns: 1000 bytes, 80 columns: 2000 bytes, **no color, no sprites**), BASIC 2 or 4 (dialect `cbm_v2`), programs from `$0401`. `.basicstub` creates `SYS 1037` at `.org $0401`.
- Chips: `petpia1` (`$E810`, keyboard, cassette), `petpia2` (`$E820`, IEEE-488), `petvia` (`$E840`, user port, sound via the shift register), `crtc` (`$E880/$E881`, only 8032 and larger). Constants: `#include <pet/hardware.asm>` and `#include <pet/kernal.asm>` (KERNAL jump table `$FFC0-$FFEA`).
- Build: `vcc asm prog.asm --target pet -o prog.prg`, BASIC: `vcc basic prog.bas --target pet`, project: `vcc build folder` (project file with `"system": "pet"`). The templates `asm-hello` and `basic-hello` exist for `pet` (`vcc new proj --target pet --template asm-hello`), examples are in `examples/pet`.
- Text: The PET models 2001 to 4032 start in **uppercase/graphics mode**. Write text in the source code in **lowercase** for this reason (`.petscii "hello"` appears as `HELLO`). The 8032 starts with uppercase and lowercase. `CHR$(147)` clears the screen.
- Manual start: `xpet -model 4032 -autostart prog.prg` (also `2001`, `8032` and all other models from `xpet -help`).
- **Tested** (VICE 3.10, macOS): the assembler template runs on 2001, 4032 and 8032, and the BASIC example on 4032. **Not tested:** all register addresses of the chip files (written from memory), the cycle counts (the file only contains the clock rate), SuperPET, 8096/8296 with bank switching.

## C16/C116
Belongs to the target `plus4` (see *C128 and Plus/4*): `xplus4 -model c16`.
