# Troxx (MEGA65)

Self-contained arcade example for VCASM on the MEGA65: a figure travels along the lines of a grid (14 x 7 rectangles) and colors them. When all four edges of a rectangle have been traveled, it is filled. Enemies walk on the same lines and cost a life; **rule 16.2**: when an enemy walks over an inner character of a segment that does not border a filled rectangle, the segment is destroyed. The rules are those of the reference version for the VIC-20 (`examples/vic20/troxx`) and of the C64 example; the MEGA65 offers more power: 80 x 50 characters with full-color tiles (VIC-IV), a 256-color palette, 16-color sprites with pixel movement, a fill animation by DMA, 40 MHz, multi-voice music over four SID chips, extra features and **two players**, taking turns or simultaneously. Concept: section 15 of the concept "Gittermosaik" (project documents, in German, not part of the example folder).

**Status:** checked in the Xemu emulator only (PAL and NTSC), not on real hardware. Hand test list for the owner: `HANDTEST.md`. Screenshots from Xemu: `bilder/`.

## Starting

Run `vcc build` in the project folder, then start `build/main.prg` in the emulator or load it on the MEGA65:

```
xmega65 -prg build/main.prg -prgmode 65
```

The program is a BASIC 65 start (`SYS 8205`) and runs without additional files. The high score list (`TROXX.HS`) is on drive 8 (real MEGA65: SD card, Xemu: directory "XEMU EXTERNAL"); without a drive or file the game continues with the default list and reports this after the entry ("not saved"). Joysticks on both ports (Xemu: joystick emulation via the keyboard, see the Xemu settings; real joysticks on the MEGA65).

## Controls

**Title screen:** joystick up/down selects the row, left/right changes the value, fire starts. The high score list ("rekord") is shown below. The on-screen texts are lower case without umlauts and in German, as in the concept.

| Row | Values |
|---|---|
| players | 1 or 2 |
| flow | only with two players: taking turns (after each hit the other player is up, each has their own grid, points and lives) or simultaneous (both figures in the same grid, yellow and cyan) |
| joystick | one joystick on port 2 (default), one joystick on port 1, two joysticks (player 1 on port 1, player 2 on port 2; always so in the simultaneous game) |
| speed | 1-9 |

With **one joystick** and two players taking turns, a prompt to pass the joystick on and press fire appears before every turn.

**Game:** the figure only moves while a direction is held; at nodes (crossings and corners) it can change direction, on a line only forward and back are possible. Fire = pause, fire again = continue. Below the field (rows 43-49) three panels show player 1, player 2 (points, lives as hearts) and the level (record, filled rectangles, progress bar); messages appear in the bottom row.

**Simultaneous:** both figures color in the same grid in the color of their player (yellow: player 1, cyan: player 2). The points for a character, a segment and a rectangle go to the player whose figure colors the last character. An item (star, ice block) is picked up by the figure that enters its cell; the points go to its player. A hit costs only one life of the hit player; when the last one is used up, that figure drops out, and the game ends when both have dropped out.

## Rules

- Grid 14 x 7 rectangles, interior 4 x 5 characters (71 x 43 cells, gx 0-70, gy 0-42); 217 segments (112 horizontal, 105 vertical), 98 rectangles. Numbering H(r, c) = 14 r + c, V(r, c) = 112 + 15 r + c, B(r, c) = 14 r + c. The field fills rows 0-42 and columns 4-74 of the screen, the panels are below.
- Points: 1 per newly colored character, 10 per completed segment, 100 per filled rectangle, **chain reaction**: if a segment closes two rectangles at once, 200 more are added (400 together), 500 per level, bonus star 50. Extra life per 10,000 points, at most 9 lives; start with 3 lives. When an enemy destroys a segment, the points booked for it are deducted (never below 0).
- Enemies only on the lines, **decision only at nodes** (any direction except reversing): the **runner** (red) chooses with the probability PROB (50-90 % depending on the level) the direction that reduces the distance to the nearest figure the most, otherwise random; the **wanderer** (orange) always random, every fourth step is skipped; the **guard** (extra feature) straight ahead, at corners the only turn: a fixed round along the border. Number of enemies 2, 2, 3, 3, 4, 4, 5, 5, 5 per level, starting at five nodes (never at the bottom, where the figures start). Random numbers: 8-bit LFSR, fixed seed per level.
- **Enemies destroy lines** (rule 16.2, `ENEMY_DESTROYS_LINES`): when an enemy enters an inner character of a segment that does **not border a filled rectangle**, the whole segment is reset (it flashes white for two frames and becomes blue again); the points booked are deducted (1 per colored character, 10 more for a completed segment, never below 0, each player loses the points of their own characters) and, for a completed segment, `BOXCNT` of the adjacent rectangles is decreased by 1. **Node rule:** when an enemy enters a **node**, all segments ending there (up to four, including the segment it came from) are checked and destroyed in the same way (`enemies.touch`). The figure colors nodes **when entering** them in its color (no points, `grid.paint_node`). After every destruction colored nodes become blue again if no traveled segment connects to them any more (none of the eight neighbor cells traveled) and no adjacent rectangle is filled; a node with a figure or an enemy on it stays (`nodes_free`/`node_free` in `grid.asm`, as in the reference version from commit 9fa111ad). Frozen enemies (ice block) destroy nothing.
- Collision: same cell. One life less, figures back to the start nodes (bottom left and bottom right), enemies to their start nodes, colors and filled rectangles remain, then short invulnerability.
- Level completed when all 98 rectangles are filled; every two levels speed + 1 up to 9. Speed in frames per game step (PAL 12 ... 3, NTSC x 1.2).

## What the MEGA65 does here

- **VIC-IV, 80 x 50 characters** (`$D031 = $C8`, 640 x 400 pixels): 16-bit character numbers, screen at `$0001D000`, color RAM at `$0FF80000`. Lines, nodes, fill patterns, hearts, panels and the logo are **full-color tiles** (FCM, 8 x 8 pixels, one byte per pixel = palette index) from `assets.inc`; the font comes from the character ROM (four themes for panels of different colors).
- **256-color palette** (`$D100/$D200/$D300`): base colors, gradients for the fill patterns, 64 rainbow colors for the title screen (palette color cycling without redrawing).
- **16-color sprites, 16 x 16** (`$D06B`, true pixels): two figures, five enemies, one item, with pixel movement: the game logic still calculates in character cells, the sprite glides to the next cell in eight pixels during the step (`sprites.asm`).
- **DMA** (DMAgic, `vic4.asm`): copy tiles and sprites, clear the screen, save and restore the grid when the player changes, **fill animation**: a rectangle fills line by line, the newest line flashes.
- **40 MHz** (`$D054` VFAST); frame timing by raster polling (no interrupt).
- **Sound** (`sound.asm`): four SID chips, twelve voices; title music with six voices (SID 1 and 2) with pulse width modulation, effects (step, segment, rectangle, chain, hit, level, end, star, ice, destruction) on SID 3 and 4 with priorities.
- **High score list** (`hiscore.asm`): ten entries on the SD card via the KERNAL, name entry with the joystick, CRC-16 checksum; errors (no drive, no file, wrong checksum, write protected) lead to the default list and a message.

## Extra features and switches

The switches are at the top of `main.asm` (`.const`, can be overridden with `-D NAME=0`). With all of them at 0 exactly the rules of the reference version apply (field size and sprites remain).

| Switch | Default | Effect |
|---|---|---|
| `EXTRA_STAR` | 1 | bonus star on a free line cell (sprite, 50 points), gone after ten seconds; then four to five seconds pause |
| `EXTRA_ICE` | 1 | ice block: all enemies stand still for five seconds, the figures keep moving; every fourth item is an ice block |
| `EXTRA_GUARD` | 1 | third enemy type "guard" |
| `EXTRA_HISCORE` | 1 | high score list on the SD card |
| `EXTRA_2P` | 1 | two players (taking turns or simultaneously), joystick choice on the title screen |
| `ENEMY_DESTROYS_LINES` | 1 | rule 16.2: enemies destroy segments that are not closed |
| `FX_FILL` | 1 | fill animation with a gloss line |
| `FX_TITLE` | 1 | large logo, palette color cycling, pulse width modulation of the title music |

Test and endurance switches: `AUTOPLAY=1` (the computer plays itself, no title screen), `AUTO_PLAYERS=2` (with two players), `AUTO_SIMUL=1` (simultaneous), `AUTOGOD=1` (invulnerable), `TESTDEMO=1` (counters in RAM from `$CE84`: frames, games, highest level, hits, rectangles), `HOLD_DESTROY=n` (test only: hold the flash of destroyed segments for n frames, finished versions in `testfassungen/`), `LIVES0=n`, `TESTSCORE=$nn` (start points), `NOSAVE=1` (the high score list stays untouched), `SCRIPT=1` or `2` (joystick input from `tools/script*.inc`: operate the title screen and play without a joystick). To build a variant, put the `.const` lines before the program text.

## Tests

```
vcc test --backend xemu            # in the project folder, after vcc build
```

`troxx.vctest` (generated by `tools/gen_tests.py`) contains static checks (size, overlap, memory map) and 93 routine tests of the shared logic (`rnd`, `level_idx`, `set_row`, `cell_type`, `nb_boxes`, `legal_mask`, `box_filled`, `seg_is_closed`, `bcd_add`/`bcd_sub`, `greedy`, `place_all`, `find`, `player.step`, node rule: `node_free`, `paint_node`, `seg_destroy`, `touch`, high score list), in the Xemu emulator. **Note on Xemu:** the test runner starts Xemu with the original ROM, whose BASIC lies over `$2000-$7FFF`; the first test ("Vorbereitung") hides the ROMs, after that the others run. It must therefore come first, and after a timeout (restart of the emulator) the following tests fail until it runs again.

Shared logic: `KOPIEN.txt` lists which routines are copies of the reference version and which were adapted; `python3 tools/troxx_kopien.py examples/mega65/troxx` (in the repository folder) checks this.

**Endurance run:** `AUTOPLAY=1 AUTOGOD=1 TESTDEMO=1` in Xemu, PAL and NTSC, several minutes, once simultaneous with two players, once taking turns (results in `HANDTEST.md`, section "Dauerlauf").

## Pictures

Screenshots from Xemu (`bilder/`): `titelbild.png`, `spielstand.png` (one player), `zweispieler-gleichzeitig.png`, `zweispieler-treffer.png`, `zerstoerte-linie.png` (a segment flashes because an enemy destroys it), `spielende-rekord.png` (name entry). Differences from the mockup of the concept: the game shows the panels without the system line ("VIC-IV 80x50 ...") and fills the rectangles with a gradient per row (color per rectangle row); the figures are 16 x 16 sprites with a face, the enemies runner (red), wanderer (violet) and guard.

## Files

| File | Content |
|---|---|
| `main.asm` | constants, memory map, start, state machine (title, level start, game, fill, hit, level end, game over), title screen |
| `vic4.asm` | VIC-IV: setup, tiles, palette, font, DMA, frame timing, put characters |
| `grid.asm` | shadow grid, drawing, segments, rectangles, coloring (`paint`), destroying (`seg_destroy`), filling |
| `player.asm`, `enemies.asm`, `items.asm` | figures (one or two), enemies, star and ice block |
| `sprites.asm` | sprites: placement, pixel movement, hit image |
| `hud.asm` | panels, texts, points (BCD), hearts, progress |
| `input.asm`, `sound.asm`, `hiscore.asm` | joysticks, four SIDs, high score list |
| `data.inc`, `assets.inc` | tables; palette, tiles, sprites (generated by `tools/gen_assets.py`) |
| `KOPIEN.txt`, `HANDTEST.md`, `XEMU-TESTS.md` | list of copies, hand test list, instructions for the Xemu runs (German) |
| `testfassungen/LIESMICH.md` | test versions for screenshots of the destruction (`zerstoerung-hold`, `szenario-destroy`, `szenario-node`): build instructions (German), not part of the standard build |

## Memory map

`$0002-$00BF` and `$00E0-$00F7` zero page (map in the head of `main.asm`), `$2001-$B7FF` program, `$B800-$BFFF` sprite data and pointers (VIC bank 2), `$C000-$CD6F` shadow grid (43 rows of 80 bytes), `$CE00-$CFFF` game variables, `$E000-$FFEF` saved grids of the two players; in chip RAM `$0001D000` screen, `$00012000` tiles, `$00014000` font, `$0FF80000` color RAM. All ROMs are hidden during the game; the high score list only brings the KERNAL back for the duration of an access.
