# PRALLBALL (Commodore Plus/4) - Prototype

Ball-and-paddle game in assembler for the Commodore Plus/4 (PAL, 64 KB; **C16/116 are not supported**, they have only 16 KB). Own implementation for the TED.
The game logic (ball, bricks, extras, bonus/boss levels) is a 6502 program and similar to the C64 version; display, figures,
sound and input are Plus/4-specific. The source is commented in detail.

- **Build:** `vcc build .` (or IDE F7); start: `xplus4 -autostart build/prallball.prg` (or `build/disk.d64`: high score list on the same drive).
- **Controls:** joystick (both ports) or keyboard (Z left, X right, space bar fire, RUN/STOP abort; the input library does report the P key as pause, but the game does not evaluate it); name entry: type + RETURN (or joystick: up/down character, left/right cursor, fire).
- **Display (TED):** multicolor text mode 40 x 25 with its own character set in RAM (`$F000`). The TED has **no sprites**: ball (up to 3), paddle (16/32/48 pixels), coin and
  laser shots are **character sprites** (compositor `tsp_render`): at each frame change the cells under the sprites are filled with pool characters (codes 128..191) whose bit patterns are built from
  background and pre-shifted sprite image (`(background AND mask) OR image`, strips per fine position from `tools/gen.py`). The background is additionally kept in shadow copies
  (`$C400` characters, `$C800` colors), from which the cells are restored before every frame. Sprites are positioned to an accuracy of 2 pixels (one multicolor pixel).
  Sky gradient and background bands: **raster interrupt sets `$FF15` 22 times per frame**; colors: tables with C64 color numbers, `tcol`/`tedbg` convert them to TED color values (brightness * 16 + hue).
- **State:** title (block logo with color change, top 5, music, **ticker** with pixel-wise transitions, **scrolling text** with fine scrolling, bouncing balls, demo after 30 s), 24 levels
  (incl. 2 bonus, 2 boss), bounce angle by hit position with influence of movement and side impact, extras as rotating color-coded coins (B, S, R, L, F, K, +, P, multiball, laser, penetration),
  high score list with name entry (30 s auto-confirm), saving to the device the game was started from (KERNAL cell `FA` = `$AE`).
- **Plus/4 specifics:** the game runs **without ROM** (`$FF3F`, own interrupt vector in `$FFFE`); for the high score list `hs_io_on` switches the KERNAL on briefly (TED interrupts off,
  system messages off). All buffers that the KERNAL touches are located below `$8000`. Coin colors: multicolor cells know only the hues 0..7, so "orange" is a muted yellow ("dark yellow").
- **Sound (TED, 2 voices, no envelopes):** title music = melody (voice 1) + bass (voice 2), drums as a short noise burst on voice 2 (the bass pauses), the third note track
  (arpeggio) is omitted; sound effects use voice 2. Frequency table for PAL, low tones folded up by octaves.
- **Not yet / unchecked:** sound unheard (only structure and register tests); joystick via `$FF08` (selection `%11111011`/`%11111101`, fire bit 6 or 7, as in the cc65 driver `plus4-stdjoy`; both ports are read; not checked on real hardware); keyboard input only checked against the KERNAL decode table, not typed in the emulator; paddle (not provided on the Plus/4); NTSC not tested. In the worst case
  (3 balls + laser + coin) the main loop misses individual frame changes (about 3 %, test version `TESTFPS`).
- **Libraries:** `lib/{input,hiscore,rand,sound}.asm` (contract: see the header comments of the files). Zero page: `$02-$0D` random/music/high score list, `$10-$8B` game, `$8C-$8F` input,
  `$90-$AC` character sprites (the KERNAL uses `$90` and upwards only during the high score list).
- **Tests:** `python3 tools/gen_tests.py && vcc build . && vcc test . --backend vice` (93 emulator tests in `xplus4` with trap `$7F00`, ROM switch-off in the setup, plus static checks:
  BCD score, direction table, paddle bounce, brick collision, penetration, all 24 levels, high score list with CRC and device selection, compositor with independently computed bit patterns,
  sound effects and music steps). Cycle measurement (`cycles`) does not exist for `xplus4`.
- **Test versions (`-D`):** `AUTOPLAY` (computer plays), `STARTLEVEL=n` (0-based), `TESTMULTI`, `TESTLASER`, `TESTLOSE` (loses on purpose: game end/name entry), `TESTCAP` (every brick drops a capsule), `NOTICKER` (no ticker in the demo), `TESTFPS` (counters
  in line 24: loops, frame changes, missed frames). Build: `vcc asm prallball.asm --target plus4 --cpu 7501 -D AUTOPLAY=1 -o x.prg`.
- **Tools:** `tools/gen.py` (direction table, levels, characters, character sprites), `tools/make_levels.py` (level drafts), `tools/compose.py` (title music, same notes as C64).

## Settings and demo player (new)

At the start of `prallball.asm` there is the block **EINSTELLUNGEN** (settings; can also be set with `-D NAME=value` when assembling):
`EXTRA_RATE` (40: chance in 256 per destroyed brick for capsules; about twice as many as before), `DEMO_REAKTION_MIN`/`DEMO_REAKTION_MAX` (4/12 frames reaction time after a change of direction of the ball), `DEMO_FEHLER_RATE`, `DEMO_FEHLER_FERN`, `DEMO_UNGENAU`, `DEMO_VERFEHLEN_RATE`, `DEMO_EXTRA_LUST`, `DEMO_EXTRA_FERN`.
The demo player thus seems more human (reaction time, short mistakes, inaccurate hitting, rare deliberate misses) and catches good falling capsules as long as the ball is far away.
Test version `-D AUTOPLAY=1 -D TESTDEMO=1` counts events of the demo player in RAM (see header comment).

**Angle randomness (new):** On each paddle hit the bounce angle is shifted by exactly one zone (17 degrees) to the left or right with the chance `WINKEL_ZUFALL` (default 64 of 256, about every fourth hit). This applies to the human player and to the demo player; the ball always stays within the eight zones, and at the edges it is not pushed beyond them. It prevents endless, identical ball paths. `0` switches it off (`-D WINKEL_ZUFALL=0`). With `-D AUTOPLAY=1` the computer now plays all levels without end (before: only the first five); `-D TESTDEMO=1` additionally counts the angle shifts, the paddle hits, the completed levels, the destroyed bricks and the levels ended by the time limit (addresses in the header of `prallball.asm`).

**Demo time limit (new):** If the demo player hits no brick for `DEMO_STEIN_LIMIT` seconds (default 40), the level counts as completed and the next one begins, so an unreachable or tough last brick cannot hold the demo up endlessly. Only in the demo (and with `AUTOPLAY`), never in the game for humans; `0` switches it off.
