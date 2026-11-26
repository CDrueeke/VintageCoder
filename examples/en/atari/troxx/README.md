# Troxx (Atari 8 bit)

A self-contained arcade example for VCASM on the Atari 800XL/130XE (XL/XE, PAL and NTSC): a figure travels along the lines of a grid (8 x 5 rectangles) and colors them. When all four edges of a rectangle have been traveled, it is filled. Enemies walk on the same lines and cost a life. The rules are those of the reference version for the VIC-20 (`examples/vic20/troxx`) and of the C64 version (`examples/c64/troxx`); the Atari brings its own means: ANTIC character mode with four colors per character, display list interrupts for color gradients, player/missile graphics, POKEY with four voices. Concept: `assets/docs/Konzept-Beispiel-VC20-Gittermosaik.md` (working title "Gittermosaik", sections 15, 16 and 16.1), mockup `assets/docs/mockups/gittermosaik/gm-8-atari.png`.

**Status:** wave W3 of the concept, checked only in the emulator (`atari800` 7.2.1 with XL ROM, PAL and NTSC), not on real hardware. Manual test list for the owner: `HANDTEST.md`. Checksums of the output: `PRUEFSUMMEN.txt`.

## Operation

Start: `vcc build` in the project folder, then in the emulator `atari800 -xl -pal -nobasic -run build/main.xex` (NTSC: `-ntsc`). One joystick in port 1 or 2 (emulator with `-kbdjoy0 -kbdjoy1`: port 1 = numeric keypad 8/5/4/6 and right Ctrl as fire, port 2 = W/A/S/D and left Ctrl; details in `HANDTEST.md`). On real hardware (XL/XE) the XEX runs from disk, cassette or flash cartridge.

**Title screen:** joystick up/down selects the row, left/right changes the value, fire or the **START** key starts.

| Row | Values |
|---|---|
| PLAYERS | 1 or 2 (two players **in turns**: after each lost life the other one is up, each has their own grid, score and lives) |
| JOYSTICK | `PORT 1` (one joystick in port 1, default), `PORT 2` (one joystick in port 2), `TWO STICKS` (two players only: player 1 in port 1, player 2 in port 2) |
| SPEED | Speed 1-9 |

With **one joystick** and two players, "PASS JOYSTICK - FIRE" appears before each turn: pass the joystick on, press fire. The choice stays in RAM until power off. Below the settings is the high score list (two columns of five places each), below that the result of the disk access (`DISK OK`, `NO DRIVE`, `NO FILE`, `DISK ERR`). The target is the XL/XE with two joystick ports (the 400/800 has four, the game reads only the first two).

**Game:** the figure runs only while a direction is held; at nodes (crossings and corners) it changes direction, on a line only forward and back are possible. Fire = pause, fire again = continue. The side bar (columns 34-39) shows SCORE, HI (best score of the high score list), LEVEL, LIVES (hearts, at most six visible), SPEED, BONUS (item on the field) and, with two players, PLAYER. Messages appear in row 23, the header line in row 0.

## Rules (like the reference version)

- Grid of 8 x 5 rectangles, interior 3 x 3 characters (33 x 21 cells); 93 segments (48 horizontal, 45 vertical), 40 rectangles. Numbering H(r, c) = 8 r + c, V(r, c) = 48 + 9 r + c, B(r, c) = 8 r + c.
- Points: 1 per newly colored character, 10 per completed segment, 100 per filled rectangle, **chain reaction**: if one segment closes two rectangles at once, 200 are added (400 in total), 500 per level, bonus star 50. Extra life every 10,000 points, at most 9 lives; start with 3 lives.
- Enemies only on the lines, **decision only at nodes** (any direction except reversing): the **Runner** (red) chooses, with probability PROB (50-90 % depending on the level), the direction that reduces the distance to the figure the most, otherwise random; the **Wanderer** (orange) always random; the **Guard** (turquoise, extra feature) straight ahead, at corners the only turn: a fixed patrol around the edge. Number of enemies 1, 2, 2, 3, 3, 3, 4, ... per level. Random: 8-bit LFSR, fixed start value per level.
- **Enemies destroy open lines** (concept 16.2, `ENEMY_DESTROYS_LINES`, default 1): if an enemy that is not frozen (any type) enters an interior character of a segment that borders **no filled rectangle**, the whole segment is reset (even a completely traveled one), the points booked for it are subtracted (1 per traveled character, 10 more for a completed segment, never below 0) and `BOXCNT` of the adjacent rectangles decreases by 1. Nodes destroy nothing; filled rectangles stay. The figure keeps running, its trail is gone. Effect: the characters flash for two frames as white blocks, the row color turns white (DLI table), plus a dedicated POKEY effect "line destroyed" (rough falling tone and crackle from quickly changing noise dividers, priority 3). **Node rule:** if an enemy enters a **node**, all segments ending there (up to four, including the segment it came from and those it did not turn into) are checked by the same rule and destroyed, several in one step (`enemies.touch`; one tone, all flashes at the same time, point deduction per segment). The **figure colors nodes yellow when entering** (`grid.paint_node`, as in the C64 version, because it is a player/missile object and does not overwrite the character when leaving; the scoring does not read nodes). After every destruction `grid.nodes_free` resets yellow nodes where no yellow segment connects any more (none of the eight neighbor cells traveled or filled; state `COL_FILL` in the field state) and no adjacent rectangle is filled; a node with the figure or an enemy stays, nodes do not count for points, there is no timeout. A yellow node has its own character (crossing and T pieces; the four corners use the T piece, whose two arms stick out a little way). The game logic (`grid.seg_destroy`, `seg_is_closed`, called in `enemies.one` via `touch`) is separate from the display: `flash` and `flash_end` (from `tick`) only display. **Consequence:** Wanderers no longer skip every fourth step (as in the reference).
- Collision: same cell or swapping places. One life less, figures back to the start nodes, colors and filled rectangles stay, then a short invulnerability (blinking).
- Level complete when all 40 rectangles are filled; every two levels speed + 1 up to 9. Speed in frames per game step (PAL 12 ... 3, NTSC x1.2): the same table as in the other versions.

### What the Atari additionally pushes to the limit (performance level L2)

- **ANTIC display list 40 x 24**, character set in RAM (`CHARSET`, via `CHBASE`/`CHBAS`). Grid, side bar and logo in **mode 4** (four colors per character: background, `COLPF0` untouched line, `COLPF1` traveled line, `COLPF2` row color, with bit 7 in the character code `COLPF3`), header line, message line and the texts of the title screen in **mode 2** (ROM font, copied to RAM); both modes share one character set.
- **Display list interrupts** (`dli.asm`): at the end of each text row the interrupt switches `COLBK`, `COLPF1` and `COLPF2` for the next row. This produces the color gradient of the background, **one hue per rectangle row with a brightness gradient inside the fill** and the colorful side lettering. In the blank-line block at the top and below the picture, **animated color bars** (a new `COLBK` value per raster line) travel through the border.
- **Player/missile graphics** (`pmg.asm`, single-line resolution, `PMBASE`): figure = player 0, enemies 1-3 = players 1-3 (color by type), enemy 4 = the four missiles as the "fifth player" (`PRIOR` bit 4, color `COLPF3`), all six pixels wide and eight lines high, two images per figure. The figures glide pixel by pixel between the cells (eight lines vertically, four color clocks horizontally per character); the game logic keeps calculating in character cells. Width (`PM_WIDTH_P`, `PM_WIDTH_M`) and priority (`PM_PRIOR`) are at the top of `main.asm`.
- **Fills:** frame with bevel (light at the top and left, dark at the bottom and right) and a pattern in the middle (hatching, dots, wall, checkerboard by rectangle), color by row gradient; **animated** row by row with a flash of the newest row.
- **Title screen:** large "TROXX" logo from beveled blocks with **color cycling**, scrolling text with **fine scrolling** (`HSCROL`), color bands per row, title music.
- **POKEY with four voices** (`sound.asm`): title music with melody, arpeggio, bass and noise drums (vibrato of the melody), effects with decaying "envelopes" and priorities (step, segment, rectangle, **chain reaction** as a fast rough scale on four voices with a noise burst, hit, fanfare, end, star, ice block).
- **Extra features** (each can be switched off): bonus star, ice block, Guard, high score list on disk, two players in turns with joystick choice.

## Extra features and switches

The switches are at the top of `main.asm` (`.const`, can be overridden with `-D NAME=0`). With all `EXTRA_*` set to 0 exactly the rules of the reference version apply (field size and figures stay).

| Switch | Default | Effect |
|---|---|---|
| `EXTRA_STAR` | 1 | Bonus star on a free line cell (character, 50 points), gone after ten seconds; then a pause of four to five seconds |
| `EXTRA_ICE` | 1 | Ice block: all enemies stand still for five seconds (light blue), the figure keeps running; every fourth item is an ice block |
| `EXTRA_GUARD` | 1 | Third enemy type "Guard" (from level 4 as the third enemy) |
| `EXTRA_HISCORE` | 1 | High score list (ten entries, name entry with the joystick) on disk |
| `EXTRA_2P` | 1 | Two players in turns, player choice and joystick choice on the title screen |
| `ENEMY_DESTROYS_LINES` | 1 | Rule "enemies destroy open lines" (0 = old rule) |
| `FX_DLI` | 1 | Display list interrupts (color gradients per row, border bars); 0: one color per frame (for debugging only) |
| `FX_TITLE` | 1 | Large logo, color cycling, scrolling text on the title screen |
| `FX_FLASH` | 1 | Flash when filling |
| `FX_VIB` | 1 | Vibrato of the title melody |
| `PM_PRIOR`, `PM_WIDTH_P`, `PM_WIDTH_M` | `$11`, 0, 0 | `PRIOR`, `SIZEP0..3`, `SIZEM` |

Test and endurance switches: `AUTOPLAY=1` (the computer plays by itself, no title screen), `AUTO_PLAYERS=2` (with two players), `AUTOGOD=1` (invulnerable), `TESTDEMO=1` (counters in RAM from `$8520` and in hexadecimal in row 23: frames low/high, games; to the left of it the frames per second), `LIVES0=n`, `TESTSCORE=$nn` (start score), `SCRIPT=1` to `4` (joystick inputs from `tools/script*.inc`), `STARTLEVEL=$07` (start level, BCD), `FLASH_HOLD=1` (the flash stays, for a screenshot), `TESTPIA=1` (test aid for `troxx_pia.vctest`), `HS_HOSTTEST=1` (high score list via the emulator's host device `H1:`). Building a variant:

```bash
vcc asm main.asm --target atari8 --cpu sally -o /tmp/troxx_auto.xex -D AUTOPLAY=1 -D AUTOGOD=1 -D TESTDEMO=1
```

## High score list (disk)

File `TROXX.HS` in the same format as in the C64 version (77 bytes): ID "TXHS" and version 1, ten entries of 7 bytes (name 3 characters in C64 screen code, score BCD low/middle/high, level BCD), checksum (sum and sum of the bytes combined with `$5A`). Access goes through the **operating system's CIO** (`D1:TROXX.HS`, the drive digit is taken from `DUNIT`): if the game runs under a DOS (DOS 2.5, MyDOS, SpartaDOS ...), the file lies on the boot disk and is replaced when saving (`OPEN` for writing). **Reason:** direct SIO sectors would destroy files on a foreign disk; a boot area without file management does not help an XEX. If the game runs **without DOS** (emulator with `-run`, cassette, cartridge), the CIO reports "device missing": the title screen shows `NO DRIVE`, the default list ("TRX", 10,000 down to 1,000 points) stays and is kept up to date during the session; there is no crash. Missing file: `NO FILE`; wrong checksum, disk full or write-protected: `DISK ERR`. During the access display, DLI and sound are off (the serial bus is time-critical). Test in the emulator without DOS: `-D HS_HOSTTEST=1` and `atari800 -hreadwrite -H1 <folder>`.

Name entry: up/down letter (A-Z, space), left/right position, fire done.

## Memory map

| Area | Use |
|---|---|
| `$0080`-`$00FF` | Zero page: game state (map at the top of `main.asm`; the addresses are those of the C64 version plus `$7E`), `$F8` ff. pointers of the sound, the DLI and the player/missiles |
| `$2000`-`$6FFF` | Program (segment from `.org $2000`, currently about 9.5 KB) |
| `$7000`-`$73FF` | Character set RAM: codes 0-63 copy of the ROM font (mode 2), codes 64-127 own graphics and side lettering (mode 4, `gfx.inc`) |
| `$7800`-`$7FFF` | Player/missile memory (`PMBASE` `$78`, single lines: missiles `$7B00`, players 0-3 `$7C00`-`$7F00`) |
| `$8000`-`$83EF` | Screen 40 x 24 (960 bytes) and scrolling text buffer |
| `$8400`-`$84FF` | Display lists (game, title) and color tables per row |
| `$8500`-`$86FF` | Game variables, sound, high score list |
| `$8700`-`$89B4` | Field state `GRIDST` (33 x 21 bytes, the Atari has no color RAM) |
| `$8A00`-`$94D3` | Saved grids of the two players; `$9600` color bar table |

The operating system stays switched on (timer `RTCLOK`, shadow registers, CIO). BASIC is off (`-nobasic`; the source does not touch `$A000`-`$BFFF`).

## Files

| File | Content |
|---|---|
| `main.asm` | Switches, constants, memory map, start, state machine (title screen, player change, game over) |
| `hw.inc` | Register addresses of ANTIC, GTIA, POKEY, PIA and operating system shadows |
| `antic.asm` | Display lists, character set, color templates, TV standard, `wait_frame` |
| `dli.asm` | Display list interrupt, vertical blank interrupt, color bars |
| `pmg.asm` | Player/missile graphics: position with pixel movement, images, colors; figure templates |
| `grid.asm` | Data model, drawing, segments, rectangles, filling, saving and loading grids |
| `player.asm`, `enemies.asm` | Movement, self-play, random, decision, collision |
| `items.asm` | Bonus star and ice block |
| `input.asm` | Joystick via PIA and `TRIG0`/`TRIG1`, port choice |
| `sound.asm` | Sound module (POKEY), effects, title music |
| `hud.asm` | Side bar, messages, title screen, BCD scores |
| `hiscore.asm`, `disk.asm` | High score list and disk access via the CIO |
| `data.inc`, `gfx.inc` | Tables; character set (generated by `tools/gen_gfx.py`) |
| `KOPIEN.txt` | List of the routines copied from the reference version |
| `troxx.vctest` | Tests |
| `troxx_regel0.vctest` | Test with the rule switched off (with `tools/build_regel0.sh`) |
| `troxx_pia.vctest` | Tests of the joystick directions via the PIA (with `tools/build_pia.sh`) |
| `tools/` | `gen_gfx.py`, `gen_tests.py`, `build_pia.sh`, input scripts `script1.inc`-`script4.inc` |

The texts on the screen are English and short (as in the mockup); they are in one place in `hud.asm`.

## Shared logic and copies

The project is self-contained (only the assembler's system library lies outside). The system-independent logic (numbering, counters, random, BCD scores, AI decision, tables) comes from the reference version; each file names its origin in its header ("Herkunft: examples/vic20/troxx/..., Stand Commit ..."). `KOPIEN.txt` lists which routines count as copies and which are deliberately adapted. Comparison (in the repository folder):

```bash
python3 tools/troxx_kopien.py examples/atari/troxx
```

The tool compares the copies with the VIC-20 project by checksum after removing comments and blank lines and reports deviations (exit status 1). Changes to the shared logic belong in the reference version and from there into all projects.

## Tests

```bash
vcc build                                       # builds build/main.xex
python3 tools/gen_tests.py                      # regenerates troxx.vctest and troxx_pia.vctest (only needed after changes to zero page addresses, characters or tests)
vcc test troxx.vctest --backend atari800        # static checks and routines in the emulator (no wrapper program needed)
tools/build_pia.sh vcc                          # builds build/pia.xex with the test aid pia_read (-D TESTPIA=1)
vcc test troxx_pia.vctest --backend atari800    # joystick directions of both ports via the PIA
tools/build_regel0.sh vcc                       # builds build/regel0.xex (-D ENEMY_DESTROYS_LINES=0)
vcc test troxx_regel0.vctest --backend atari800 # enemies destroy nothing with the rule switched off
```

The shared logic passes the same test cases as in the reference version (`bcd_add`, `legal_mask`, `greedy`, `neighbours` for all segments, `paint` with chain reaction, `fill_box`, `player.step`), plus tests of the Atari parts (`grid.show`, frame and pattern of the fill, position of the figures, title menu, port choice, high score list, POKEY sound state, items, player change). `troxx_pia.vctest` briefly switches the PIA lines to output and so checks reading the directions on both ports; the fire lines `TRIG0`/`TRIG1` cannot be set this way.

## Test status (emulator, 2026-11-22)

- `troxx.vctest`: 260 of 260 (257 routine tests, 3 static checks; among them extra life at 10,000 points: 9999 + 1 -> +1 life, 9998 + 1 -> same, 9 lives stay 9; among them rule 16.2 and the node rule: `paint_node`, yellow nodes in `show`, `node_free` in all cases, `nodes_free`, `enemies.touch` at a node with two segments, with protection and with partial protection); `troxx_regel0.vctest` 1 of 1; `troxx_pia.vctest`: 14 of 14.
- `python3 tools/troxx_kopien.py examples/atari/troxx`: 32 equal, 30 adapted, 0 deviations.
- Endurance run with `AUTOPLAY=1 AUTOGOD=1 TESTDEMO=1` in `atari800` (XL ROM): PAL and NTSC eight minutes each (one player, up to level 4, about 28,000 frames per run), then, after the last changes, four minutes each again for PAL (two players), NTSC (one player) and PAL without `AUTOGOD` with two players (two games played to the end: game over, name `CPU`, new game); after adding rule 16.2 and the node rule, four minutes each again with `ENEMY_DESTROYS_LINES=1` (PAL two players, NTSC without `AUTOGOD`) and `=0` (PAL); all without a crash. High score list written and read again via `H1:` (77 bytes, `DISK OK`, name entry via `SCRIPT=3`); pause via `SCRIPT=4`; four enemies (level 7) via `STARTLEVEL=$07`.
- Screenshots (PAL, compared with the mockup): `~/Development/abnahme-troxx/atari/bilder/`.

Checked only in the emulator, not on real hardware; the real joystick lines (fire), the START key, the sound and the disk access under DOS can only be checked with the manual test list.
