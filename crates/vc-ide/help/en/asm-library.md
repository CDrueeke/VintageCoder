# Assembler: System Library

This page belongs to the [assembler manual](help:assembler).

A **system library** saves you from typing addresses and routines. With `#include <path>` you include ready-made lists of constants for the target system: hardware registers, operating system entry points, system variables and vectors. The files are in `data/sys/`; a file of the same name in the project takes precedence. `//` counts as a comment like `;`.

```asm
#include <c64/hardware.asm>
#include <c64/kernal.asm>

    lda #15              // also // is a comment
    sta VIC_BORDERCOLOR
    jsr CHROUT
```

| File | Contents |
|---|---|
| `c64/hardware.asm` | VIC-II, SID, CIA 1/2, color RAM, color names `COL_*` |
| `c64/kernal.asm` | KERNAL jump table (`CHROUT`, `GETIN` ...), vectors, zero page, RAM vectors |
| `c64/basic.asm` | BASIC pointers, floating-point registers, ROM routines (`STROUT` ...) |
| `c128/hardware.asm` `kernal.asm` | VIC-IIe, MMU, VDC (80 columns), KERNAL extensions (`JSRFAR`, `INDFET` ...), macro `mmu_config` |
| `plus4/hardware.asm` `kernal.asm` | TED, ACIA, ROM bank selection, colors `TED_COL_*`, KERNAL jump table (also for the C16) |
| `mega65/hardware.asm` | like C64, plus VIC-III/IV (`VIC4_*`), DMAgic (`DMA_*`), hypervisor/system (`M65_*`) |
| `mega65/gs4502.asm` | macros `mega65_enable_vic4`, `mega65_fast`, `ld32` |
| `mega65/kernal.asm` | jump table (the same in all ROM versions) |
| `mega65/rom_920418.asm` | system variables of the closed ROM version 920418 |
| `mega65/rom_open.asm` | OpenROMs: only the jump table is guaranteed |
| `atari8bit/antic.asm` `gtia.asm` `pokey.asm` | chip registers with the standard Atari names (`DMACTL`, `COLPF0`, `AUDF1` ...) |
| `atari8bit/os.asm` | PIA, zero page, shadow registers (`SDMCTL`, `COLOR0` ...), IOCB, `CIOV`, `SETVBV` |
| `vcs2600/tia.asm` | TIA, RIOT, memory areas, macros `CLEAN_START`, `VERTICAL_SYNC`, `WAIT_LINES` |
| `common/types.asm` | bit masks `BIT0`-`BIT7`, status bits `P_*`, macros `set_bits`, `ld16`, `ptr_set`, `mov16`, `mov8`/`movx8`/`movy8` (copy a byte), `add8`/`sub8` ... |
| `common/math.asm` | 16-bit macros (`add16`, `sub16`, `cmp16`, `inc16` ...) and the subroutines `mul16`/`div16` (insert them with `math_routines`; zero page selectable with `MATH_ZP`) |
| `c64/tiles.asm` (with `common/tiles.asm`) | tile library: `tiles_use`, `tile_draw`, `tile_draw_color`, `tile_fill`, `screen_select`, `screen_draw` (macros) and, with `tiles_runtime`, the subroutines `tile_put`, `screen_draw_all`, `screen_draw_rows`, `screen_map_get`, `tile_prop`; also restoring the background and filling rows: `tile_erase` (X, Y), `tile_restore` (macro, rectangle from the map) with `tile_restore_a`, `tile_row` (macro) with `tile_row_a`; with the switch `TILE_RLE` the maps are run-length encoded (export option `"rle": true` of the screen bank) and `screen_select` unpacks them to `TILE_MAPRAM` (`screen_load_rle`); any tile size W x H, switches `TILE_SMC`, `TILE_ZP`, `TILE_SCREEN`, `TILE_MAPRAM`, `TILE_RLE`. Example `examples/c64/tiles_demo`, commands at the top of the file |
| `atari8bit/tiles.asm`, `plus4/tiles.asm`, `vic20/tiles.asm`, `c128/tiles.asm`, `mega65/tiles.asm` | the same tile library (`common/tiles.asm`) for the other systems, same commands and switches; only the system parts differ (see below). Examples `examples/<system>/tiles_demo` |
| `c64/notes.inc`, `c128/notes.inc`, `mega65/notes.inc`, `plus4/notes.inc`, `vic20/notes.inc`, `atari8bit/notes.inc` | note constants `NOTE_C0` ... `NOTE_B7` and the frequency table of the sound chip (PAL/NTSC), see the section "Notes and frequency tables" below |

**Tile library per system.** Everything shared is in `common/tiles.asm`; the system file only sets `TILE_COLS`, `TILE_ROWS`, `TILE_SCREEN` and `TILE_COLORRAM` (all can be overridden before including, e.g. with `vcc asm -D TILE_SCREEN=$0800`).

| System | Screen | Color memory | Special feature |
|---|---|---|---|
| C64 | 40x25 at `$0400` | `$D800`, 4 bits | bit 3 = multicolor |
| Atari 8-bit | 40x24 at `$4000` | none (`TILE_NOCOLOR`) | color choice in multicolor mode 4 via character bit 7 (cell flag `sel` in the export); the screen must lie in a 4 KB block (the file checks this); `tile_draw_color` draws only the character |
| Plus/4, C16 | 40x25 at `$0C00` | `$0800` (before the screen, distance -`$0400`) | attribute byte = brightness*16 + hue, `col` in the export is the finished byte |
| VIC-20 | 22x23, basic machine `$1E00` (from +8 KB `$1000`) | `$9600` (from +8 KB `$9400`), 4 bits | addresses from `vic20/hardware.asm` according to the memory expansion of the project; tile width 3 gives 7 tiles per row |
| C128 | 40x25 (VIC-IIe) at `$0400` | `$D800` | the VDC (80 columns) is not supported; `TILE_MAPRAM` must not lie in a ROM window (`$4000`-`$BFFF` shows BASIC ROM) |
| MEGA65 | 40x25 at `$0800` | `$D800` | VIC-II behavior of the VIC-IV; cycle budgets with `TILE_CYC_LDA = 5` (40 MHz table of the 45GS02) |

Tested (`vcc test`, emulator): C64, Plus/4, VIC-20 and C128 with VICE (trap mode), Atari 8-bit with atari800 (mailbox mode); MEGA65 only built and checked statically (Xemu did not start here), the cycles come from the `.cycles` block. The VIC-20 example is built for the basic machine. The zero page group `TILE_ZP` (`$F7`-`$FE`) has not yet been checked against the KERNAL usage on Plus/4, C128, VIC-20 and MEGA65 (the examples run with the I flag set and without KERNAL).


**Including and names.** `#include <...>` includes each file only once per namespace, so libraries can include each other. The same names can have different addresses on different systems (e.g. `SPIN_SPOUT` on the C128 and on the MEGA65). So include only the library of your target system, otherwise the assembler reports "symbol defined twice". If you want several systems in one file, give each one a namespace: `#include <c128/kernal.asm> as c128` and then `jsr c128.SPIN_SPOUT`.

**MEGA65 ROMs.** The jump table (`mega65/kernal.asm`) is the same in all ROM versions. Variables and internal addresses depend on the ROM version: include exactly one of the files `mega65/rom_*.asm` (two at the same time give an error). In the source you can query the version with `.if MEGA65_ROM == 920418`; for the OpenROMs the value is 1.

## Example: using constants and macros

```asm
#include <c64/hardware.asm>
#include <c64/kernal.asm>
#include <common/types.asm>

        lda #COL_RED                 ; color name instead of number
        sta VIC_BORDERCOLOR          ; register with standard name
        ld16 $fb, $0400              ; 16-bit value to $fb/$fc
        set_bits $d015, %00000011    ; turn on sprites 0 and 1
        jsr CHROUT
;=* a9 02 8d 20 d0 a9 00 85 fb a9 04 85 fc
```

The chip registers are also available without `#include` as `chip.register` (for example `vic2.border`), as long as the target system knows them; the description appears when you point at it with the mouse.

Back to: [Assembler manual](help:assembler), [Macros](help:asm-macros), [Cheat sheet](help:asm-index).

## Notes and frequency tables

Every system with sound generation has a file `notes.inc` with named note constants and the matching frequency table of the sound chip:

```
#include <c64/notes.inc>        ; or c128, mega65, plus4, vic20, atari8bit
```

The files are generated by `tools/gen_notes.py` from the physical formulas (equal temperament, A4 = 440 Hz) and are not changed by hand; `python3 tools/gen_notes.py --check` checks whether they still match the script.

**Constants.** The note value counts as in the tracker: `NOTE_C0 = 1`, `NOTE_A4 = 58` (440 Hz), `NOTE_B7 = 96`. The names are English: `NOTE_CS4` is C sharp (C#), `NOTE_DB4` is D flat - the same pitch as `NOTE_CS4`; `NOTE_B4` is B natural (called H in German), `NOTE_BB4` is B flat (called B in German, the same pitch as `NOTE_AS4`). With calculation you get intervals: `NOTE_C4 + 7` is the fifth, `NOTE_C4 + 12` the next octave. Also `NOTE_HOLD = 0` (hold the note), `NOTE_REST = 96` (rest), `NOTE_FIRST`, `NOTE_LAST`. In the song format of the player routines (`sound.asm`, `music.inc`), 96 is the rest: so as a song note you can write `NOTE_C0` to `NOTE_AS7` (1..95), and `NOTE_B7` only as a table index.

**Tables.** They are indexed with the note value (entry 0 = no note). PAL is the default; `vcc asm -D NTSC=1` (or `.const NTSC = 1` before the include) selects NTSC. `NOTES_NO_TABLE` (defined before) includes only the constants. Including more than once is harmless.

| System | Tables | Value range and limits |
|---|---|---|
| C64, C128, MEGA65 | `note_sid_lo`, `note_sid_hi` | 16-bit frequency counter `Fn = f * 2^24 / clock` (PAL 985248 Hz, NTSC 1022727 Hz). `NOTE_TABLE_HIGHEST`: PAL `NOTE_AS7`, NTSC `NOTE_B7`; above that the counter is full (65535) |
| Plus/4 | `note_ted_lo`, `note_ted_hi` | 10-bit tone value `N = 1024 - clock/8 / f`. Below about 108 Hz the TED runs out of range: notes below `NOTE_TABLE_LOWEST` (A2) sound whole octaves higher |
| VIC-20 | `note_vic_sopr`, `note_vic_alto`, `note_vic_bass` | register values `$80 \| N` for `$900C`, `$900B`, `$900A`; each voice has about seven octaves, notes outside are moved by octaves |
| Atari 8-bit | `note_pokey8`, `note_pokey16_lo`, `note_pokey16_hi` | 8 bits (64 kHz): from `NOTE_POKEY8_LOWEST` (PAL C3, NTSC C#3), below that `$FF`; 16 bits (linked channels): all 96 notes |

Example, a C major scale on SID voice 1 (C64): get the pitch and write it to `$D400`/`$D401`.

```
#include <c64/notes.inc>

scale:  ldy #0
@next:  ldx major,y             ; note value
        lda note_sid_lo,x
        sta $d400
        lda note_sid_hi,x
        sta $d401
        ; ... set gate and wait ...
        iny
        cpy #8
        bne @next
        rts
major:  .byte NOTE_C4, NOTE_D4, NOTE_E4, NOTE_F4, NOTE_G4, NOTE_A4, NOTE_B4, NOTE_C5
```
