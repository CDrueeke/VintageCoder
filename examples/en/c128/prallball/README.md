# PRALLBALL (Commodore 128, VIC-II version) - Prototype

Ball-and-paddle game in assembler for the C128 in **40-column mode (VIC-II)**, PAL. The technique matches the C64 version (multicolor text mode with its own character set,
hardware sprites, raster color bands, SID); own code, because memory management (MMU), start and file access are different. An additional **VDC version (80 columns)** follows in
`examples/c128/prallball_vdc`. The source is commented in detail.

- **Build:** `vcc build .` (or IDE F7); start: `x128 -40col -autostart build/prallball.prg` (or `build/disk.d64`: high score list on the same drive). Start in 40-column mode
  (key 40/80 DISPLAY or `-40col`), otherwise the monitor shows only the VDC screen.
- **Controls:** joystick port 2 or keyboard (Z left, X right, space bar fire, RUN/STOP abort; the input library reports the P key as pause, the game does not evaluate it); name entry: type + RETURN (or joystick up/down/left/right or Q/A/Z/X, fire confirms).
- **Memory:** the game sets the MMU configuration `$3E` (**all RAM of bank 0 plus I/O**, no system ROMs; own interrupt vector in `$FFFE`). Program from `$1C01` (to
  `$47DF`, then a gap up to `$7FFF`), **VIC bank 2** (`$8000-$BFFF`): sprite data `$8000` (pointers `$00-$0A`), screen `$8400`, character set `$8800` (copied from the character ROM, configuration `$01`),
  levels/song from `$9400` (the VIC sees only the character ROM at `$9000-$9FFF`, so nothing graphical may be located there), brick map `$C000-$C0D7`, ball/ticker variables `$C100-$C17F`. 1 MHz (the 2 MHz mode
  disturbs the VIC-II and stays off).
- **High score list:** file `PRALL.HS` (127 bytes, format as on the C64) on the drive the game was started from (`$BA`, remembered at start). For the KERNAL calls `hs_io_on` briefly switches to the
  standard configuration `$00` (system ROMs), switches off the VIC interrupt, suppresses the system messages and selects bank 0 with `SETBNK`; **the switching routines and `copy_font`
  are therefore located below `$4000`** (there is always RAM).
- **State:** as on the C64: title (logo, top 5, music, ticker, scrolling text, bouncing balls, demo after 30 s), 24 levels (bonus/boss), bounce angle by hit position with influence of movement and
  side impact, extras as rotating coins (multiball, laser, penetration), high score list with name entry. Checked in `x128` (title, demo, game, saving/loading of the high score list).
- **Not yet / unchecked:** 2 MHz operation outside the visible picture (not needed, deliberately left out); sound unheard (SID as on the C64, only structure checked); keyboard and
  joystick not typed in the emulator (query as on the C64: CIA1); the additional C128 keys (`$D02F`) are not used; NTSC not tested.
- **Libraries:** `lib/{input,hiscore,rand,sound}.asm` (contract: see the header comments of the files; zero page: random `$02`, music `$04`, high score list `$08`, game `$10-$8B`, input `$8C`, the KERNAL uses `$90` and upwards).
- **Tests:** `python3 tools/gen_tests.py && vcc build . && vcc test . --backend vice` (x128, trap `$BF00`, in the setup `"$ff00": 62` = MMU configuration `$3E`: 68 emulator tests and static checks:
  BCD score, direction table, paddle bounce, brick collision, penetration, all 24 levels, high score list, character set copy, VIC setup, screen helpers).
- **Test versions (`-D`):** `AUTOPLAY`, `STARTLEVEL=n` (0-based), `TESTMULTI`, `TESTLASER`, `TESTLOSE`, `TESTFPS`, `TESTCAP`, `NOTICKER` (details in the file header of `prallball.asm`). Build: `vcc asm prallball.asm --target c128 --cpu 8502 -D AUTOPLAY=1 -o x.prg`.
- **Tools:** `tools/gen.py`, `tools/make_levels.py`, `tools/compose.py` (as C64).

## Settings and demo player (new)

At the start of `prallball.asm` there is the block **EINSTELLUNGEN** (settings; can also be set with `-D NAME=value` when assembling):
`EXTRA_RATE` (40: chance in 256 per destroyed brick for capsules; about twice as many as before), `DEMO_REAKTION_MIN`/`DEMO_REAKTION_MAX` (4/12 frames reaction time after a change of direction of the ball), `DEMO_FEHLER_RATE`, `DEMO_FEHLER_FERN`, `DEMO_UNGENAU`, `DEMO_VERFEHLEN_RATE`, `DEMO_EXTRA_LUST`, `DEMO_EXTRA_FERN`.
The demo player thus seems more human (reaction time, short mistakes, inaccurate hitting, rare deliberate misses) and catches good falling capsules as long as the ball is far away.
Test version `-D AUTOPLAY=1 -D TESTDEMO=1` counts events of the demo player in RAM (see header comment).

**Angle randomness (new):** On each paddle hit the bounce angle is shifted by exactly one zone (17 degrees) to the left or right with the chance `WINKEL_ZUFALL` (default 64 of 256, about every fourth hit). This applies to the human player and to the demo player; the ball always stays within the eight zones, and at the edges it is not pushed beyond them. It prevents endless, identical ball paths. `0` switches it off (`-D WINKEL_ZUFALL=0`). With `-D AUTOPLAY=1` the computer now plays all levels without end (before: only the first five); `-D TESTDEMO=1` additionally counts the angle shifts, the paddle hits, the completed levels, the destroyed bricks and the levels ended by the time limit (addresses in the header of `prallball.asm`).

**Demo time limit (new):** If the demo player hits no brick for `DEMO_STEIN_LIMIT` seconds (default 40), the level counts as completed and the next one begins, so an unreachable or tough last brick cannot hold the demo up endlessly. Only in the demo (and with `AUTOPLAY`), never in the game for humans; `0` switches it off.
