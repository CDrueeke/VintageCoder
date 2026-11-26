# PRALLBALL (VIC-20, 8 KB expansion) - prototype

Ball-and-paddle game in assembler for the VIC-20 (PAL) with **at least 8 KB expansion** ($2000-$3FFF; 3.5 KB internal + 8 KB = about 11.5 KB). The VIC-I
has no sprites and only 22 x 23 characters; the game logic is therefore a **slim rewrite** (not the code of the other systems). Source code commented in detail.

- **Build:** `vcc build .` (or IDE F7; project setting `memory: 8k`). Start: `xvic -memory 8k -autostart build/disk.d64` (disk; do **not** load the PRG alone, otherwise
  the file access of the high score list stays without a drive).
- **Controls:** joystick (only port) or keyboard: Z left, X right, space fire, P pause, RUN/STOP back to the title. Name entry: type, RETURN (30 s auto-confirm).
- **Display:** 22 x 23 characters (176 x 184 dots): screen `$1000`, color RAM `$9400`, **own character set in RAM at `$1C00`** (the VIC-I only sees memory below `$2000`; the text characters 0..63
  are copied by the program from the character ROM). Bricks and walls are **multicolor characters** (light = auxiliary color white, shadow = border color black), text and figures single-color.
  Ball (5 x 5), paddle (16/24/32 dots wide) and coin are **character sprites**: the cells under them get pool characters (codes 80..99), whose bit pattern the program composes for each frame
  (`blit`: shift image lines by X mod 8, OR into two cells); the background is computed from the brick map on restore (`tsp_restore`). No raster interrupt:
  the interrupts stay off, the frame timing comes from the raster register `$9004` (figures are rebuilt below the display).
- **Status:** title (logo with color change, top 5, music, demo after 30 s), **22 levels** (20 regular + 2 bonus with 45 s time and no ball loss), 10 brick columns (derived from the 18 columns of the other
  systems), brick types a-h, silver, gold, indestructible, explosive, surprise, bounce angle by hit position with movement influence, 8 speed levels, extras as falling
  coins (B wider, S narrower, L slower, F faster, + extra ball, P 500 points), high score list with name entry on the start drive, pause.
- **Deliberately left out (memory/computing time, as far as the target allows):** boss level, multiball, laser, pierce, ticker and scrolling text, paddle, joystick name entry.
- **Memory:** program currently `$1201-$1BDC` (limit `$1BFF`) and `$2000-$34BC` (together about 7.8 KB with levels, song and tables; the PRG file is 8894 bytes because the gap `$1BDD-$1FFF` is filled with zeros); `$1C00-$1FFF` character set RAM (empty in the program image); brick map `$0340` (cassette buffer), pool cell list `$03B8-$03DF`.
  Reserve up to `$3FFF`: about 2.9 KB. `.org $2000` in the source separates the two program parts; the test runner checks that part 1 ends before `$1C00`.
- **High score list:** `PRALL.HS` (127 bytes, format as on the C64) on the drive from which the game was started. The serial bus shares port A of VIA 1: the input must not change `$9113`.
- **Sound (VIC-I, 3 tones + noise, shared volume):** title song (16 bars from the C64 song, melody soprano, bass bass, arpeggio alto, drums as noise burst), effects on alto/noise.
- **Not yet / unchecked:** sound not heard (only register values checked); keyboard and joystick not typed in the emulator (matrix checked against the KERNAL decode table `$EC5E`); NTSC not tested;
  behavior without expansion (does not run).
- **Libraries:** `lib/{input,hiscore,rand,sound}.asm` (contract: see the header comments of the files). Zero page: random `$02`, music `$04`, high score list `$08`, game `$10-$62`, input `$8C`.
- **Tests:** `python3 tools/run_tests.py` (after `vcc build .`): starts xvic with `-memory 8k` and the text monitor, calls subroutines one by one. Reason: `vcc test --backend vice` starts xvic without
  memory expansion. The path to `xvic` is at the top of `tools/run_tests.py` (`EXE`, a macOS path there) and may need to be adjusted. 81 checks: load sizes and limits, BCD points, random, speed table, paddle bounce, brick collision, brick types incl. explosion, coins (all 6 kinds), all 22 levels, character sprites with
  independently computed bit patterns, high score list, sound values.
- **Test versions (`-D`):** `AUTOPLAY` (the computer plays from the start), `STARTLEVEL=n` (0-based), `TESTLOSE` (loses on purpose, 5000 points: game over/name entry), `TESTCAP` (a coin falls from every destroyed brick). Build:
  `vcc asm prallball.asm --target vic20 --memory 8k -D AUTOPLAY=1 -o x.prg`; to start, build a disk (`vcc disk d64 -o x.d64 --add x.prg:PRALLBALL`).
- **Tools:** `tools/gen.py` (generates `lib/*.inc`: direction table, levels reduced to 10 columns from `levels/levels.txt`, multicolor characters, logo, shortened song from the C64 song `examples/c64/prallball/lib/music.inc`), `tools/make_levels.py` (copy of the C64 script, generates `levels/levels.txt`), `tools/run_tests.py`.

## Settings and demo player (new)

At the start of `prallball.asm` there is the block **EINSTELLUNGEN** (settings; can also be set with `-D NAME=value` when assembling):
`EXTRA_RATE` (48: chance in 256 per destroyed brick for coins; about twice as many as before), `DEMO_REAKTION_MIN`/`DEMO_REAKTION_MAX` (4/12 frames of reaction time after the ball changes direction), `DEMO_FEHLER_RATE`, `DEMO_FEHLER_FERN`, `DEMO_UNGENAU`, `DEMO_VERFEHLEN_RATE`, `DEMO_EXTRA_LUST`, `DEMO_EXTRA_FERN`.
The demo player thus acts more human (reaction time, short mistakes, imprecise hitting, rare deliberate misses) and catches good falling coins as long as the ball is far away.
The test version `-D AUTOPLAY=1 -D TESTDEMO=1` counts events of the demo player in RAM (see header comment).

**Angle randomness (new):** On each paddle hit the bounce angle is shifted by exactly one zone (17 degrees) to the left or right with the chance `WINKEL_ZUFALL` (default 64 of 256, about every fourth hit). This applies to the human player and to the demo player; the ball always stays within the eight zones, and at the edges it is not pushed beyond them. It prevents endless, identical ball paths. `0` switches it off (`-D WINKEL_ZUFALL=0`). With `-D AUTOPLAY=1` the computer now plays all levels without end (before: only the first five); `-D TESTDEMO=1` additionally counts the angle shifts, the paddle hits, the completed levels, the destroyed bricks and the levels ended by the time limit (addresses in the header of `prallball.asm`).

**Demo time limit (new):** If the demo player hits no brick for `DEMO_STEIN_LIMIT` seconds (default 40), the level counts as completed and the next one begins, so an unreachable or tough last brick cannot hold the demo up endlessly. Only in the demo (and with `AUTOPLAY`), never in the game for humans; `0` switches it off.
