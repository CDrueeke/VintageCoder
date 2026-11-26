# Memory Map

The **Memory map** (main section **Data > Memory**) shows how the address space is used after the last build.

## Map
The map shows the address space as 16x16 pages of 256 bytes each; on the left is the start address of each row. Each page is colored by the type of memory and its purpose (for example zero page, stack, screen, sprites, character set, music, I/O, ROM, BASIC, free), as defined by the selected **bank switching** of the target. The program from the **last build** lies on top: segments, banks and symbols come from the `.vcdbg` next to the built program. A bar in the page shows how much of it is used. Without a build, it says "Nothing built yet. The map comes from the last build. Build first (F7)."

- **Filter:** **All**, **Program**, **RAM**, **ROM** or **I/O** hide the rest (the other pages turn pale).
- **Select a page:** click or arrow keys. On the right, **Selected page** shows the address, the type of memory, how many bytes the program uses there, the symbols on the page and the source lines (a click jumps to the editor).
- **Areas** (left): one entry per type with color and start address; a click selects the first page of this type. Below are the bytes used by the program and the free RAM.
- **Checks** (right): notes with a sign for the severity (✓, ·, ⚠, ✗), for example the number of overlaps, whether the stack is free and how much of the zero page the program uses.

### Bank switching
For C64, C128 and Atari 800 XL/XE, the target (`data/targets/*.target.json`, section `banking`) provides presets for the register that switches the memory:

| System | Register | Bits |
|---|---|---|
| C64 | Processor port `$0001` | LORAM, HIRAM, CHAREN |
| C128 | MMU configuration register `$FF00` | IO, ROM_LOW, ROM_MID0/1, ROM_HIGH0/1, BANK |
| Atari 800 XL/XE | PORTB `$D301` | OSROM, BASIC, BANK0/1, CPUBANK, ANTICBANK, SELFTEST |

At the bottom right you see the register with its address and the list of presets (for example "Standard (BASIC, I/O, KERNAL)" on the C64 with `$37` or "All RAM" with `$34`); a click chooses the view. A note about the preset appears below. A preset can be limited to one model with the note "130XE only". If a target has no presets (VIC-20, C16/Plus/4, MEGA65), the map shows only the segments of the program.
