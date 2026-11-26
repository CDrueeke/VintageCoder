# Troxx (Plus/4)

A self-contained arcade example for VCASM on the Commodore Plus/4 (and C16-like TED machines with 64 KB): a figure travels along the lines of a grid (8 x 5 rectangles) and colors them. When all four edges of a rectangle have been traveled, it is filled. Enemies walk on the same lines and cost a life. The rules are those of the reference version for the VIC-20 (`examples/vic20/troxx`); the Plus/4 brings the **TED with 121 colors** (brightness levels, fills with gradient and bevel), a character set in RAM, figures made of characters with four animation frames (the Plus/4 has no sprites), a raster interrupt, two voices and extra features. Concept: `assets/docs/Konzept-Beispiel-VC20-Gittermosaik.md` (working title "Gittermosaik", sections 15 and 16), mockup `assets/docs/mockups/gittermosaik/gm-4-plus4.png`.

**Status:** wave W2 of the concept, checked only in the emulator (VICE `xplus4` 3.10, PAL and NTSC), not on real hardware. Manual test list for the owner: `HANDTEST.md`. Checksums of the output: `PRUEFSUMMEN.txt`.

## Operation

Start: `vcc build` in the project folder, then start **`build/disk.d64`** in the emulator (`xplus4 build/disk.d64`, autostart), so that the high score list ends up on the same disk. Without a disk (only `build/main.prg`) the game runs just the same, only without the high score list. Joystick in port 1 or 2 (emulator: `-joydev1 2` = key set 1, `-joydev2 1` = numeric keypad, fire = keypad 0 or right Alt key, depending on the emulator setting).

**Title screen:** joystick up/down selects the row, left/right changes the value, fire starts.

| Row | Values |
|---|---|
| PLAYERS | 1 or 2 (two players **in turns**: after each lost life the other one is up, each has their own grid, score and lives) |
| JOYSTICK | `PORT 2` (one joystick in port 2, default), `PORT 1` (one joystick in port 1), `TWO STICKS` (two players only: player 1 in port 1, player 2 in port 2) |
| SPEED | Speed 1-9 |

With **one joystick** and two players, "pass joystick - fire" appears before each turn: pass the joystick on, press fire. The choice stays in RAM until power off. Below the settings is the high score list, to the right of "HIGH SCORES" the result of the disk access (`DISK OK`, `NO DRIVE`, `NO FILE`, `DISK ERR`).

**Game:** the figure runs only while a direction is held; at nodes (crossings and corners) it changes direction, on a line only forward and back are possible. Fire = pause, fire again = continue. Displays as in the mockup: at the top SCORE, LEVEL and HI (row 0), below that messages (row 1); below the field the lives as hearts (row 23, with two players PLAYER n on the right) and in row 24 SPEED, BONUS (symbol and text of the item on the field) and the computer name.

**Note on the keyboard:** the Plus/4 joysticks are connected to the keyboard matrix. Do not press any key during the game (it would fake a joystick contact). The keyboard is not polled. Fire is read on both ports as bit 7 or bit 6 of `$FF08` (which bit the emulator delivers is not clarified; it worked this way in the manual test).

## Rules (like the reference version)

- Grid of 8 x 5 rectangles, interior 3 x 3 characters (33 x 21 cells); 93 segments (48 horizontal, 45 vertical), 40 rectangles. Numbering H(r, c) = 8 r + c, V(r, c) = 48 + 9 r + c, B(r, c) = 8 r + c.
- Points: 1 per newly colored character, 10 per completed segment, 100 per filled rectangle, **chain reaction**: if one segment closes two rectangles at once, 200 are added (400 in total), 500 per level, bonus star 50. Extra life every 10,000 points, at most 9 lives; start with 3 lives.
- Enemies only on the lines, **decision only at nodes** (any direction except reversing): the **Runner** (red) chooses, with probability PROB (50-90 % depending on the level), the direction that reduces the distance to the figure the most, otherwise random; the **Wanderer** (purple) always random; the **Guard** (cyan, extra feature) straight ahead, at corners the only turn: a fixed patrol around the edge. Number of enemies 1, 2, 2, 3, 3, 3, 4, ... per level. Random: 8-bit LFSR, fixed start value per level.
- **Enemies destroy lines** (concept 16.2, `ENEMY_DESTROYS_LINES`): if an enemy enters an interior character of a segment that borders **no filled rectangle** (otherwise even a completely traveled segment is not protected), the whole segment is reset (characters blue again). The points booked for it are subtracted (1 per colored character, 10 more for a completed segment, never below 0) and, for a completed segment, `BOXCNT` of the adjacent rectangles is decreased by 1; filled rectangles and their bonuses stay. **Node rule:** if an enemy enters a **node**, all segments ending there (up to four, including the segment it came from and those it did not turn into) are checked and destroyed in the same way (flash, tone and point deduction per segment). The figure colors nodes yellow **when leaving** (without points; this is also how the start node turns yellow). After every destruction, yellow nodes turn blue again if no yellow segment connects to them any more (no neighbor cell yellow) and no adjacent rectangle is filled (fill character in the neighborhood); a node with the figure or an enemy (also as the target of the current step) stays; no timeout (`nodes_free`/`node_free` in `grid.asm`, `paint_node` when leaving in `player.asm`). Frozen enemies (ice block) destroy nothing, all enemy types destroy. Feedback on the Plus/4: the colored characters flash for two frames as a white hit character, plus the effect "line destroyed" (falling tone on voice 1, crackle from noise of changing pitch on voice 2). If the figure stands on the segment, it keeps running; its trail is gone. The rule is in a few separate places: call in `enemies.one`, logic in `grid.asm` (`seg_is_closed`, `seg_destroy`, `cell_op`), `hud.bcd_sub`, effect `Fx.destroy` in `sound.asm`.
- Collision: same cell or swapping places. One life less, figures back to the start nodes, colors and filled rectangles stay, then a short invulnerability (blinking white/cyan).
- Level complete when all 40 rectangles are filled; every two levels speed + 1 up to 9. Speed in frames per game step (PAL 12 ... 3, NTSC x1.2): the same table as in the reference version.

### Improvements over the VIC-20

- **TED colors**: color RAM byte = brightness (0-7) * 16 + hue. The lines are blue, traveled lines light yellow. The four fill patterns (hatching, dots, wall, checkerboard) are multicolor characters with **nine layers in the 3 x 3 block** (highlight at the top and left, shadow at the bottom and right = bevel, `charset.asm`, generated by `tools/gen_charset.py`); the hue changes per rectangle row (green, cyan, purple, red, yellow), the **brightness rises per row of the block** (color gradient), and a raster interrupt sets the two multicolor background colors (shadow and highlight) to the matching dark and light tone for each rectangle row.
- **Figures as characters** (`figs.asm`): player, up to four enemies and the item are characters with **four animation frames** (a change every eight frames) and color changes (invulnerability white/cyan, ice block light blue, item blinks). The grid has a **shadow copy** (character and color per cell, `grid.asm`); a figure covers only the screen, when it moves on the cell is restored from the shadow copy. The game logic and the shadow copy know no figures.
- **Second field layout from level 5**: six rectangles are already filled at the start (gray wall blocks, edges yellow), the counters (`BOXCNT`, `BOXES_DONE`) are recounted from them, the level still ends at 40 rectangles. `LAYOUT2_LEVEL` sets the level.
- **Raster interrupt** (`raster.asm`, via the KERNAL vector `$0314`, KERNAL ROM stays switched in): frame clock (line 204), **moving color bars** in the upper and lower border (gradient blue, brightness 0-7 and back; one table per TV standard, because PAL and NTSC show different numbers of border lines), color bands of the fills, on the title screen background bands and a **smooth scroll** (row 6, X offset in `$FF07`, 38-column mode only in this row); disk accesses block it.
- **Title screen** with a large logo made of beveled multicolor blocks (5 x 5 per letter) and a color gradient in hue and brightness.
- **Fill animation**: row by row with a flash of the newest row (color RAM only, the logic stays untouched).
- **TED sound** (two voices, `sound.asm`): title music (melody on voice 1, bass alternating with the chord on voice 2), effects step (noise on voice 2), segment, rectangle, chain (two voices), hit (low tone and falling noise), level (fanfare), end, bonus star, ice block, each with priorities. The TED has no envelope and only one volume for both voices; tones below 108 Hz do not exist, so the bass is one octave higher than on the C64.
- **Extra features** (each can be switched off, see below): bonus star, ice block, Guard, second layout, high score list on disk, two players in turns with joystick choice.

## Extra features and switches

The switches are at the top of `main.asm` (`.const`, can be overridden with `-D NAME=0`). With all `EXTRA_*` set to 0 exactly the rules of the reference version apply (field size stays).

| Switch | Default | Effect |
|---|---|---|
| `EXTRA_STAR` | 1 | Bonus star on a free line cell (character, 50 points), gone after ten seconds; then a pause of four to five seconds |
| `EXTRA_ICE` | 1 | Ice block: all enemies stand still for five seconds (light blue), the figure keeps running; every fourth item is an ice block |
| `EXTRA_GUARD` | 1 | Third enemy type "Guard" (from level 4 as the third enemy) |
| `EXTRA_HISCORE` | 1 | High score list (ten entries, name entry with the joystick) on disk |
| `EXTRA_2P` | 1 | Two players in turns, player choice on the title screen |
| `ENEMY_DESTROYS_LINES` | 1 | Rule 16.2: enemies destroy lines (0 = old rule) |
| `EXTRA_LAYOUT` | 1 | Second field layout from level `LAYOUT2_LEVEL` (5) |
| `FX_RASTER` | 1 | Border bars, color bands, scrolling (without them only the frame clock interrupt remains) |
| `FX_TITLE` | 1 | Large logo, color gradient, scrolling on the title screen (needs `FX_RASTER`) |
| `FX_FLASH` | 1 | Flash when filling |
| `FX_SHADE` | 1 | Brightness gradient of the fills and color band per rectangle row |

Test and endurance switches: `AUTOPLAY=1` (the computer plays by itself, no title screen), `AUTO_PLAYERS=2` (with two players), `AUTOGOD=1` (invulnerable), `TESTDEMO=1` (counters in RAM from `$5030` and in hexadecimal in row 1: frames, games, highest level), `LIVES0=n`, `START_SPEED=n`, `LAYOUT2_LEVEL=n`, `TESTSCORE=$nn` (start score), `SCRIPT=1` or `2` (joystick inputs from `tools/script*.inc`), `TESTPAD=1` (wrappers for the short test routines). Building a variant:

```bash
vcc asm main.asm --target plus4 --cpu 7501 --illegals deny -o troxx_auto.prg -D AUTOPLAY=1 -D AUTOGOD=1 -D TESTDEMO=1
```

## High score list (disk)

File `TROXX.HS` on drive 8, 77 bytes: ID "TXHS" and version 1, ten entries of 7 bytes (name 3 screen codes, score BCD low/middle/high, level BCD), checksum (sum and sum of the bytes combined with `$5A`). It is saved with "@0:" (replaces the old file) and then verified by **reading back and comparing** (the KERNAL does not report write protection). If the drive or the file is missing or the checksum is wrong, the default list ("TRX", 10,000 down to 1,000 points) applies; every error is caught and displayed, the game keeps running. Name entry: up/down letter (A-Z, space), left/right position, fire done.

During a disk access the raster interrupt is off and the KERNAL interrupt vector is set again (`raster.pause`/`resume`). Observation in the emulator: an `OPEN` on the error channel 15 after saving crashed the computer (cause not clarified); therefore the error channel is not read.

## Memory map

| Area | Use |
|---|---|
| `$0002`-`$007F` | Zero page: game state (map at the top of `main.asm`); `$90` and higher stays with the KERNAL (disk) |
| `$0800`-`$0BE7`, `$0C00`-`$0FE7` | Color RAM, screen 40 x 25 |
| `$1001`-`$3FFF` | Program (segment `PROGRAM`, currently about 8.4 KB) |
| `$4000`-`$47FF` | Character set RAM: codes 0-127 copy of the character ROM (text), 128-255 own graphics (69 characters) |
| `$4800`-`$4AB4`, `$4B00`-`$4DB4` | Shadow copy of the grid: characters, colors (33 x 21) |
| `$5000`-`$5FFF` | Game variables, high score list, state and grid of the second player |

The KERNAL ROM stays switched in (disk, character ROM); this leaves `$1001`-`$7FFF` available (about 28 KB). The memory expansion to 60 KB (ROM off) is not needed and not used.

TED: screen `$0C00`, color RAM `$0800` (`$FF14` = `$08`), character set `$4000` from RAM (`$FF12` bit 2 = 0, `$FF13` = `$40`), `$FF06` = `$1B`, `$FF07` = `$98` (characters 128-255 separate, multicolor possible, 40 columns). Cells (text, lines, figures) use only the hues 0-7: bit 3 of the hue is the cell's multicolor bit in multicolor mode (fills and logo blocks set it); background, border and the multicolor backgrounds 1 and 2 (`$FF16`, `$FF17`) can have all 121 colors. Multicolor bit pairs: 00 background 0, 01 background 1 (dark tone of the row), 10 background 2 (light tone), 11 color of the cell.

## Files

| File | Content |
|---|---|
| `main.asm` | Switches, constants, memory map, start, state machine (title screen, player change, game over), fill animation |
| `ted.asm` | Screen init, PAL/NTSC detection, setting up the character set, `wait_frame` |
| `raster.asm` | Raster interrupt: frame clock, border bars, color bands, scrolling |
| `figs.asm` | Figures as characters: drawing, restoring from the shadow copy, images, colors |
| `grid.asm` | Data model with shadow copy, drawing, segments, rectangles, filling, second layout, saving and loading grids |
| `player.asm`, `enemies.asm` | Movement, self-play, random, decision, collision |
| `items.asm` | Bonus star and ice block |
| `input.asm` | Joystick via the keyboard matrix (`$FF08`), port choice |
| `sound.asm` | Sound module (TED), effects, title music |
| `hud.asm` | Displays, messages, title screen, BCD scores |
| `hiscore.asm`, `disk.asm` | High score list and disk access via the KERNAL |
| `data.inc`, `charset.asm` | Tables, graphic characters (generated) |
| `KOPIEN.txt` | List of the routines copied from the reference version |
| `troxx.vctest`, `troxx_kurz.vctest`, `troxx_regel0.vctest` | Tests (the last file with `ENEMY_DESTROYS_LINES=0`, built by `tools/build_regel0.sh`) |
| `tools/` | `gen_charset.py`, `gen_tests.py`, `build_pad.sh`, `build_regel0.sh`, `testpad.inc`, input scripts |

The texts on the screen are English and short (as in the mockup); they are in one place in `hud.asm`.

## Shared logic and copies

The project is self-contained (only the assembler's system library lies outside). The system-independent logic (numbering, counters, random, BCD scores, AI decision, tables) comes from the reference version; each file names its origin in its header ("Herkunft: examples/vic20/troxx/..., Stand Commit ..."). `KOPIEN.txt` lists which routines count as copies and which are deliberately adapted (on the Plus/4: shadow copy, figures as characters, colors, grid dimensions). Comparison (in the repository folder):

```bash
python3 tools/troxx_kopien.py examples/plus4/troxx
```

The tool compares the copies with the VIC-20 project by checksum after removing comments and blank lines and reports deviations (exit status 1). Changes to the shared logic belong in the reference version and from there into all projects.

## Tests

```bash
vcc build                                   # builds build/main.prg and build/disk.d64
python3 tools/gen_tests.py                  # regenerates troxx.vctest and troxx_kurz.vctest (only needed after changes to the addresses in main.asm)
vcc test --backend vice                     # troxx.vctest: static checks (size, overlap, zero page) and routines in the emulator (xplus4)
tools/build_pad.sh vcc                      # builds build/pad.prg (with wrappers, see tools/testpad.inc)
vcc test troxx_kurz.vctest --backend vice   # very short routines
tools/build_regel0.sh vcc                   # builds build0/main.prg with ENEMY_DESTROYS_LINES=0
vcc test troxx_regel0.vctest --backend vice # the same cases of rule 16.2 with the rule switched off
```

The shared logic passes the same test cases as in the reference version (`bcd_add`, `legal_mask`, `greedy`, `neighbours` for all segments, `paint` with chain reaction, `fill_box`, `player.step`), plus tests of the Plus/4 parts (shadow copy, fills with gradient, second layout, title menu, high score list, sound state, items, player change). The joystick lines cannot be set in a test run (keyboard matrix); only the port assignment is tested. Result (`xplus4`, PAL): `troxx.vctest` 59 of 59 (3 static checks, 42 routine tests, 9 tests of rule 16.2 and the node rule: segments partly or fully colored, protected, edge, two neighbor rectangles, point lower limit, uncolored, enemy at a node destroys three segments, frozen enemy, and 7 tests `nodes_free`: free-standing node, yellow neighbor cell, filled neighbor rectangle, figure, enemy, corner node, edge node), `troxx_kurz.vctest` 161 of 161 (among them `nb_boxes` for all 93 segments), `troxx_regel0.vctest` 9 of 9 (the same cases with the rule switched off: nothing changes).

Checked only in the emulator, not on real hardware; the real joystick lines of both ports can only be checked with the manual test list.

**Endurance runs** (`AUTOPLAY`, `AUTOGOD`, `TESTDEMO`, speed 9, emulator in warp mode), with rule 16.2 (default): 1.5 billion cycles (about 28 minutes of emulator time) in PAL with one player (level 6) and in NTSC with two players (level 5), 800 million cycles NTSC with one player; with `ENEMY_DESTROYS_LINES=0`: 800 million cycles each NTSC with one and PAL with two players; all without a crash (the levels come more slowly with the rule, because enemies destroy lines). All enemy types, items, the second layout, player changes and level changes occur; for a screenshot with a destroyed line see the acceptance folder. With a disk and `LIVES0=1`: game over, name entry (self-play "CPU" and input script `SCRIPT=2` with the name "BC "), saving and reloading the high score list several times in a row (checked before rule 16.2, the disk access stayed unchanged); a restart from the same disk shows the saved list. Variants with all `EXTRA_*` at 0 and with all `FX_*` at 0 ran before the rule.
