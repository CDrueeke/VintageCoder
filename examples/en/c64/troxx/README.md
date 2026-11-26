# Troxx (C64)

Self-contained arcade example for VCASM on the Commodore 64: a figure travels along the lines of a grid (8 x 5 rectangles) and colors them. When all four edges of a rectangle have been traveled, it is filled. Enemies walk on the same lines and cost a life. The rules are those of the reference version for the VIC-20 (`examples/vic20/troxx`); the C64 offers more power and therefore hardware sprites with pixel movement, a multicolor character set, SID sound and extra features. Concept: `assets/docs/Konzept-Beispiel-VC20-Gittermosaik.md` (working title "Gittermosaik", sections 15 and 16), mockup `assets/docs/mockups/gittermosaik/gm-5-c64.png`.

**Status:** wave W2 of the concept, checked in the emulator only (VICE `x64sc`, PAL and NTSC), not on real hardware. Hand test list for the owner: `HANDTEST.md`. Checksums of the output: `PRUEFSUMMEN.txt`.

## Controls

Start: run `vcc build` in the project folder, then start **`build/disk.d64`** in the emulator (`x64sc build/disk.d64`, autostart), so that the high score list ends up on the same disk. Without a disk (only `build/main.prg`) the game runs the same, just without the high score list. Joystick in port 1 or 2 (emulator: `-joydev1 1` or `-joydev2 1` = numeric keypad, fire = keypad 0 or right Alt key).

**Title screen:** joystick up/down selects the row, left/right changes the value, fire starts.

| Row | Values |
|---|---|
| PLAYERS | 1 or 2 (two players **taking turns**: after each lost life the other one is up, each has their own grid, points and lives) |
| JOYSTICK | `PORT 2` (one joystick in port 2, default), `PORT 1` (one joystick in port 1), `TWO STICKS` (only with two players: player 1 in port 1, player 2 in port 2) |
| SPEED | speed 1-9 |

With **one joystick** and two players, "pass joystick - fire" appears before each turn: pass the joystick on, press fire. The choice stays in RAM until power off. The high score list is below the settings; to the right of "HIGH SCORES" is the result of the disk access (`DISK OK`, `NO DRIVE`, `NO FILE`, `DISK ERR`).

**Game:** the figure only moves while a direction is held; at nodes (crossings and corners) it changes direction, on a line only forward and back are possible. Fire = pause, fire again = resume. The side bar (columns 34-39) shows SCORE, HI (best points of the high score list), LEVEL, LIVES (hearts, at most six visible), SPEED, BONUS (item on the field) and, with two players, PLAYER. Messages appear in row 24.

**Note on the keyboard:** the keyboard matrix is on the same lines as the joysticks. Do not press any key during the game (it would fake a joystick contact). The keyboard is not polled.

## Rules (as in the reference version)

- Grid 8 x 5 rectangles, interior 3 x 3 characters (33 x 21 cells); 93 segments (48 horizontal, 45 vertical), 40 rectangles. Numbering H(r, c) = 8 r + c, V(r, c) = 48 + 9 r + c, B(r, c) = 8 r + c.
- Points: 1 per newly colored character, 10 per completed segment, 100 per filled rectangle, **chain reaction**: if a segment closes two rectangles at once, 200 are added (400 together), 500 per level, bonus star 50. Extra life per 10,000 points, at most 9 lives; start with 3 lives.
- **Game rule 16.2 (enemies destroy lines):** if an enemy enters an inner character of a segment that borders **no filled rectangle**, the whole segment is reset (all characters violet again, for a completed segment `BOXCNT` of the neighbors - 1); the points booked for it are deducted (1 per colored character, 10 more for a completed segment, never below 0). Nodes destroy nothing, and frozen enemies (ice block) do not either; applies to all enemy types. Feedback: the characters flash white, raster color flash in the border (red/white), score briefly red, own sound (crackle and falling tone). Switch `ENEMY_DESTROYS_LINES` (default 1, 0 = old rule). **Extension W-T1:** if an enemy reaches a **node**, all unprotected segments ending there with at least one yellow character are destroyed in the same way (per segment as above; the shared logic `enemies.touch` comes from the reference version). **Nodes:** they turn yellow when the figure enters them (`grid.paint_node`; on the VIC-20 this happens when leaving, in `erase`, same result). After each destruction `grid.nodes_free` resets free-standing yellow nodes (violet) if none of the eight neighbor cells is yellow or a fill pattern (no yellow segment connects any more, no adjacent rectangle filled); a node with the figure or an enemy stays, nodes do not count for points, no timeout. The Runner keeps the more precise pursuit on the C64 (`greedy`); the reference chooses `toward`. The Wanderer no longer skips every fourth step (as in the reference version).
- Enemies only on the lines, **decision only at nodes** (any direction except reversing): the **Runner** (red) chooses, with probability PROB (50-90 % per level), the direction that reduces the distance to the figure the most, otherwise random; the **Wanderer** (orange) always random; the **Guard** (cyan, extra feature) straight ahead, at corners the only turn: a fixed round along the edge. Number of enemies 1, 2, 2, 3, 3, 3, 4, ... per level. Random numbers: 8-bit LFSR, fixed start value per level.
- Collision: same cell or swapping places. One life less, figures back to the start nodes, colors and filled rectangles stay, then short invulnerability (blinking).
- Level complete when all 40 rectangles are filled; every two levels speed + 1 up to 9. Speed in frames per game step (PAL 12 ... 3, NTSC x 1.2): same table as in the reference version.

### Improvements over the VIC-20

- **Multicolor sprites** (outline black, highlight white, body color per sprite, two images per figure; `tools/gen_sprites.py`) for the figure, up to four enemies and the item: the game logic still works in character cells (decisions only at nodes, collision at character level); the sprite glides to the next cell in eight dots during the step (`sprites.asm`). Frame timing by raster polling (line 251, below the picture); the raster interrupt only changes the border.
- **Multicolor character set**: lines and text single-color, the four fill patterns (hatching, dots, wall, checkerboard) as multicolor characters; color by rectangle row.
- **Raster interrupt** (`raster.asm`): animated color bars in the upper and lower border, on the title screen background bands (black at top and bottom) and a **smooth scroll** (row 6, fine offset in `$D016`, 38-column mode only in this row); disk accesses block it.
- **Title screen** with a large logo made of beveled multicolor blocks (5 x 5 per letter) and a color cycle.
- **Fill animation**: row by row with a flash of the newest row (color memory only, the logic stays untouched).
- **SID**: title music (three voices) with low-pass filter (resonance, cutoff frequency sweeps back and forth) and effects step, segment, rectangle, chain, hit, level, end, bonus star, ice block, each with priorities.
- **Extra features** (each can be switched off, see below): bonus star, ice block, Guard, high score list on disk, two players taking turns with joystick choice.

## Extra features and switches

The switches are at the top of `main.asm` (`.const`, can be overridden with `-D NAME=0`). With all of them at 0, exactly the rules of the reference version apply (field size and sprites remain).

| Switch | Default | Effect |
|---|---|---|
| `EXTRA_STAR` | 1 | bonus star on a free line cell (sprite, 50 points), gone after ten seconds; then a pause of four to five seconds |
| `EXTRA_ICE` | 1 | ice block: all enemies stand still for five seconds (light blue), the figure keeps moving; every fourth item is an ice block |
| `EXTRA_GUARD` | 1 | third enemy type "Guard" (from level 4 as the third enemy) |
| `EXTRA_HISCORE` | 1 | high score list (ten entries, name entry with the joystick) on disk |
| `ENEMY_DESTROYS_LINES` | 1 | game rule 16.2: enemies destroy unfinished lines |
| `EXTRA_2P` | 1 | two players taking turns, player choice on the title screen |
| `FX_RASTER` | 1 | raster interrupt (border bars, background bands, scroll) |
| `FX_TITLE` | 1 | large logo, color cycle, scroll on the title screen (needs `FX_RASTER`) |
| `FX_FLASH` | 1 | flash when filling |
| `FX_PWM` | 1 | pulse width modulation and filter sweep of the title music |

Test and endurance switches: `AUTOPLAY=1` (the computer plays itself, no title screen), `AUTO_PLAYERS=2` (with two players), `AUTOGOD=1` (invulnerable), `TESTDEMO=1` (counters in RAM from `$C020` and in hexadecimal in row 24: frames, games, highest level), `LIVES0=n`, `TESTSCORE=$nn` (start points), `SCRIPT=1` or `2` (joystick input from `tools/script*.inc`, see below), `TESTPAD=1` (wrappers for the short test routines). Building a variant:

```bash
vcc asm main.asm --target c64 --cpu 6510 --illegals deny -o /tmp/troxx_auto.prg -D AUTOPLAY=1 -D AUTOGOD=1 -D TESTDEMO=1
```

## High score list (disk)

File `TROXX.HS` on drive 8, 77 bytes: identifier "TXHS" and version 1, ten entries of 7 bytes (name 3 screen codes, points BCD low/middle/high, level BCD), checksum (sum and sum of the bytes combined with `$5A`). It is saved with "@0:" (replaces the old file). If the drive or the file is missing or the checksum does not match, the default list applies ("TRX", 10,000 down to 1,000 points); every error is caught and displayed, the game keeps running. Name entry: up/down letter (A-Z, space), left/right position, fire done.

## Memory map

| Area | Use |
|---|---|
| `$0002`-`$007B` | zero page: game state (map in the header of `main.asm`); `$90` and higher is left to the KERNAL (disk) |
| `$0400`-`$07E7`, `$07F8` | screen 40 x 25, sprite pointers |
| `$0801`-`$2FFF` | program (segment `PROGRAM`, currently about 7.4 KB) |
| `$3000`-`$37FF` | character set RAM: codes 0-127 copy of the character ROM (text), 128-255 own graphics |
| `$3800`-`$3AFF` | sprite data (11 sprites of 64 bytes, block number `$E0` ff.) |
| `$C000`-`$CFFF` | game variables, high score list, game state and grid of the second player |

VIC-II: bank 0, `$D018` = `$1C`, `$D016` = `$18` (multicolor characters possible), background 0 blue, 1 black, 2 dark gray, border light blue. Lines and text have color memory bit 3 = 0 (single-color, colors 0-7), fills bit 3 = 1. The grid starts at screen column 1 and row 2 (column 0 stays free so that a sprite at the edge does not disappear under the border).

## Files

| File | Content |
|---|---|
| `main.asm` | switches, constants, memory map, start, state machine (title screen, player change, game over) |
| `vic.asm` | screen init, PAL/NTSC detection, set up character set and sprites, `wait_frame` |
| `raster.asm` | raster interrupt, border bars |
| `sprites.asm` | sprites: position with pixel movement, images, colors; sprite data (`sprites.inc`, generated by `tools/gen_sprites.py`) |
| `grid.asm` | data model, drawing, segments, rectangles, filling, save and load the grid |
| `player.asm`, `enemies.asm` | movement, self-play, random numbers, decision, collision |
| `items.asm` | bonus star and ice block |
| `input.asm` | joystick via CIA 1, port choice |
| `sound.asm` | sound module (SID), effects, title music |
| `hud.asm` | side bar, messages, title screen, BCD points |
| `hiscore.asm`, `disk.asm` | high score list and disk access via the KERNAL |
| `data.inc`, `charset.asm` | tables, graphic characters |
| `KOPIEN.txt` | list of the routines copied from the reference version |
| `troxx.vctest`, `troxx_kurz.vctest` | tests |
| `tools/` | `gen_sprites.py`, `gen_tests.py`, `build_pad.sh`, `testpad.inc`, input scripts |

The texts on the screen are English and short (as in the mockup); they are in one place in `hud.asm`.

## Shared logic and copies

The project is self-contained (only the assembler's system library is outside). The system-independent logic (numbering, counters, random numbers, BCD points, AI decision, tables) comes from the reference version; each file names its origin in the header ("Herkunft: examples/vic20/troxx/..., Stand Commit ..."). `KOPIEN.txt` lists which routines count as copies and which are deliberately adapted. Comparison (in the repository folder):

```bash
python3 tools/troxx_kopien.py examples/c64/troxx
```

The tool compares the copies by checksum with the VIC-20 project after removing comments and blank lines and reports differences (exit status 1). Changes to the shared logic belong in the reference version and from there into all projects.

## Tests

```bash
vcc build                                   # builds build/main.prg and build/disk.d64
python3 tools/gen_tests.py                  # regenerates troxx.vctest and troxx_kurz.vctest (only needed after changes to the zero page addresses)
vcc test --backend vice                     # troxx.vctest: static checks (size, overlap, zero page) and routines in the emulator
tools/build_pad.sh vcc                      # builds build/pad.prg (with wrappers, see tools/testpad.inc)
vcc test troxx_kurz.vctest --backend vice   # very short routines
tools/build_regel0.sh vcc                   # builds build/regel0.prg (old rule)
vcc test troxx_regel0.vctest --backend vice # test with ENEMY_DESTROYS_LINES = 0
```

The shared logic passes the same test cases as in the reference version (`bcd_add`, `legal_mask`, `greedy`, `neighbours` for all segments, `paint` with chain reaction, `fill_box`, `player.step`), plus tests of the C64 parts (sprite position, input, title menu, high score list, sound state, items, player change). Result: `troxx.vctest` 87 of 87 (3 static checks, plus three tests for the extra life at 10000 points), `troxx_kurz.vctest` 166 of 166, `troxx_regel0.vctest` 1 of 1 (old rule). Endurance run (`AUTOPLAY`, `TESTDEMO`) in the emulator without a crash: PAL and NTSC 10 minutes each (one player and two players, with and without `AUTOGOD`), up to 20 minutes with disk (17 games in a row, high score list saved each time), also with all extra features off.

Checked in the emulator only, not on real hardware; the real joystick lines can only be checked with the hand test list.
