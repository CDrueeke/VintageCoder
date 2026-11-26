# VIC-20

Target `vic20`: CPU 6502 (`nmos6502`), video chip **VIC-I** (6560/6561), two **VIA 6522**, BASIC V2 (dialect `cbm_v2`), emulator VICE (`xvic`).

## Memory expansion
The VIC-20 comes in its basic configuration with 5 KB and with cartridge expansions. BASIC start, screen RAM and color RAM depend on it. The expansion is stored in the project file at the target (`"memory": "8k"`). You can choose it when you create the project in the dialog **Create new project** (field **Memory expansion**) or by hand in the project file; the templates use the basic machine.

| Expansion | BASIC from | Screen | Color RAM | VICE `-memory` |
|---|---|---|---|---|
| `unexpanded` (basic machine, 5 KB) | `$1001` | `$1E00` | `$9600` | `none` |
| `3k` (+3 KB, `$0400-$0FFF`) | `$0401` | `$1E00` | `$9600` | `3k` |
| `8k`, `16k`, `24k`, `35k` | `$1201` | `$1000` | `$9400` | `8k`, `16k`, `24k`, `all` |

- **Assembler:** When building, VintageCoder defines `MEM_SCREEN_RAM`, `MEM_COLOR_RAM` and `MEM_BASIC_START`. The system file `#include <vic20/hardware.asm>` (or `vic20/kernal.asm`) turns them into `SCREEN_RAM`, `COLOR_RAM` and `BASIC_START`. Without a setting, the basic machine applies. On the command line: `vcc asm prog.asm --target vic20 --memory 8k`.
- **`.org` and `.basicstub`:** `.org BASIC_START` and `.basicstub start` adapt to the expansion. A fixed `.org $1001` applies only to the basic machine and the templates.
- **BASIC:** `vcc basic prog.bas --target vic20 --memory 8k` (and the normal build with the project setting) sets the start address; the program must fit into the free memory.
- **Run:** F5 starts `xvic` with the matching `-memory` argument. VintageCoder searches for the path to the emulator automatically; you enter it by hand under **Settings > Targets and hardware**. **F6** starts `xvic` with a debug session.

## Cartridge
With `.cartridge "a0", "NAME"` the source code becomes a raw 8 KB image; the file extension names the memory block: `.a0` (`$A000`-`$BFFF`), `.20`, `.40`, `.60` (`$2000`, `$4000`, `$6000`). Unused bytes are `$FF`. A cartridge at `$A000` is started by the system itself if the cold and warm start addresses are at `$A000` and `$A002` and the signature `A0CBM` is at `$A004` (`.byte $41,$30,$c3,$c2,$cd`); if it is missing, the assembler reports an error. The system is not yet initialized at start: the cartridge calls `RAMTAS`, `RESTOR`, `IOINIT` and `CINT` (and `CLRCHN`) itself. The template *Cartridge* shows this. On the command line: `vcc asm modul.asm --target vic20` (the extension follows the block) or `--format a0|20|40|60`. F5 inserts the image as a cartridge (`-cartA`, `-cart2` ...).

## Chips
- `vic1` (`$9000-$900F`): screen origin, columns/rows, memory pointer (`$9005`), raster, three tone voices and noise (`$900A-$900D`), volume/auxiliary color, border/background (`$900F`). Example: `sta vic1.color`.
- `via1` (`$9110`: NMI, user port, joystick, serial bus) and `via2` (`$9120`: IRQ timer, keyboard).
- Constants: `#include <vic20/hardware.asm>` (registers, memory areas, colors `COL_*`) and `#include <vic20/kernal.asm>` (KERNAL jump table as on the C64, but only from `RESTOR` (`$FF8A`); `RAMTAS`, `IOINIT` and `CINT` are direct ROM addresses; plus vectors and system variables).

## Character set, PETSCII
- The character sets of the VIC-20 (uppercase/graphics and upper/lower case, per character ROM revision) come from your ROM (help "ROM files") and can be loaded as a template in the [Character set editor](help:charset); the profile `vic20.char.hires` is 8x8, single color. There is also `vic20.char.multicolor` (4x8, four pixel values: background, border, character color, auxiliary color; the colors are in `$900F`, in color RAM and in `$900E`) and `vic20.char.hires16` (8x16 for `$9003` bit 0). Both start empty; character and color fields work with the VIC-I palette.
- `.petscii`, `.screencode` and the `{clr}` notation are the same as on the C64.

## Screen editor
You edit the screen in the main area **Graphics** under **Screen** (see [Screen editor](help:screen)). Profile `vic20.screen.text` (22x23 characters, with color RAM; border, background and character color per character). The editor delivers only the data (`chars` and `attrs` as export or `.include`); they must be copied at run time to `SCREEN_RAM` and `COLOR_RAM`, whose addresses depend on the memory expansion (see above). For color RAM only colors 0-7 count; from 8 on the VIC-I switches the character to multicolor (with the color n - 8), the color fields of the editor do not show this difference. Border: only colors 0-7, background and auxiliary color 0-15.

## Tiles and tracker
You find the tile editor under **Graphics > Tiles**, the tracker in the main area **Music**.
- **Tiles:** profile `vic20.tilemap.2x2` (11x11 tiles of 2x2 characters each = 22x22 of the 22x23 screen, with color RAM). The export is like on the other systems; it is copied at run time to `SCREEN_RAM` and `COLOR_RAM`.
- **Tracker:** profile `vic20.vic.song` (4 voices: bass `$900A`, alto `$900B`, soprano `$900C`, noise `$900D`; 32 rows, at most 8 instruments). Example with a player routine: `examples/vic20/music_demo` (`lib/music.inc`, frequency tables `lib/vic_freq.inc`, generated by `tools/gen_vic_freq.py`). The beat comes from the KERNAL IRQ (PAL 50 Hz, NTSC 60 Hz); so the speed of the song depends on the television standard.
- **Limits of the chip:** The VIC-I has only **one shared volume** (`$900E`, bits 0-3). The volume envelope of an instrument therefore only switches the voice on (value > 0) or off (0), the volume stays 15; the distortion is ignored. The pitch is stepped in 7 bits (`f = clock * m / (256 * (127 - N))`, m = 1, 2, 4 for bass, alto, soprano, measured in VICE); notes outside the range of a voice are moved by octaves, high notes are inaccurate. The noise has no fixed pitch. The preview in the tracker does not know this stepping and sounds cleaner than the chip.

## Limits
- Cartridges: only 8 KB images without banks (no Mega-Cart or Final Expansion).
- When importing from disk images, BASIC from `$1201` and `$0401` is detected as VIC-20; `$1001` stays assigned to the Plus/4 (the line texts are the same).
