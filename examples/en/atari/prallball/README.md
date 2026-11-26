# PRALLBALL (Atari 8 bit) - prototype

Ball-and-paddle game in assembler for the Atari 800XL/130XE (PAL), own implementation for the Atari hardware.
The pure game logic (ball, bricks, extras, bonus/boss levels) is a 6502 program and similar to the C64 version; the display (ANTIC/GTIA), figures (player/missile),
sound (POKEY) and input are Atari-specific. The source code is commented in detail.

- **Build:** `vcc build .` (or IDE F7); start: `atari800 -xl -pal -nobasic -run build/prallball.xex`.
- **Controls:** joystick port 1 or keyboard (Z left, X right, space fire, ESC abort); name entry: type + RETURN.
- **Display:** display list with text mode 2 (text) and 4 (playfield); **the display list interrupt sets the colors per text line** (sky gradient, rainbow bricks,
  highlight edges, metal); paddle = players 0-2 (16/32/48 dots), capsule = player 3 (rotation), balls = missiles 0-2, laser shot = missile 3.
- **Status:** title (block logo with color change, top 5, music, **ticker** with pixel-wise transitions, **scrolling text** with fine scrolling (HSCROL), bouncing missile balls, demo after 30 s with ticker in the header line), highlight (the highlight edge of the brick rows flashes from top to bottom), 24 levels (incl. bonus/boss), bounce angle by hit position with movement influence and side impact,
  extras as coins (B, S, R, L, F, K, +, P, multiball, laser, pierce; recognizable by color), high score list with name entry.
- **Atari specials (fully used):** **raster line kernel** (the DLI before line 18 sets the sky color and the player colors for each raster line: smooth sky gradient and metal gloss in the paddle), **wave logo** (fine scrolling HSCROL per text line through the DLI table, own display list with LMS per logo line), high score list on the boot drive (DUNIT).
- **Not yet:** second laser shot (only one missile free), paddle (query prepared, untested), high score list on disk tested only with DOS (without DOS D: reports an error, the default list stays), sound not heard (only structure and tables checked); the pixel-line alignment of the paddle gloss is tuned by eye to the emulator (`kpm`).
- **Libraries:** `lib/{input,hiscore,rand,sound}.asm` (contract: see the header comments of the files; random from the POKEY hardware register, music on all 4 channels
  with 16-bit bass, high score list through CIO `D:PRALL.HSC`).
- **Tests:** `python3 tools/gen_tests.py && vcc build . && vcc test . --backend atari800` (68 emulator tests + 3 static checks: BCD points, direction table, paddle bounce,
  brick collision, pierce, all 24 levels, high score list with CRC).
- **Testing the high score list without DOS:** `-D HS_HOSTTEST=1` uses the host device `H1:` of the emulator: `atari800 -xl -pal -nobasic -hreadwrite -H1 <folder> -run ...`; with `-D TESTHS=1` the title writes and reads the list at once and shows the CIO status codes (1 = success). Checked: `PRALL.HSC` has 127 bytes in the same format as on the C64.
- **Test versions (`-D`):** `AUTOPLAY` (the computer plays; `DEMOFLAG=0` with header line), `TESTLOSE` (loses on purpose: game over/name entry), `NOHS` (no disk access at start), `STARTLEVEL=n` (start with level n, 0-based), `TESTMULTI` (multiball at once), `TESTCAP` (every brick drops a capsule), `TESTLASER` (laser always on), `TESTKEY` (feeds key presses D, A, Z into the OS buffer), `TESTPM` (ball Y fixed, to compare sprite and playfield lines), `NOTICKER` (without ticker); high score list: `HS_HOSTTEST`, `TESTHS` (see above).
- **Tools:** `tools/make_levels.py` (level drafts), `tools/compose.py` (title music, same notes as C64, Atari player in `sound.asm`), `tools/gen.py` (playfield characters `lib/gfx.inc.chars`), `tools/gen_tests.py` (generates `prallball.vctest`). `lib/dirtable.inc` and `lib/levels.inc` are unchanged copies of the C64 version (`examples/c64/prallball`, generated there by `tools/gen.py` from `levels/levels.txt`); there is no separate generation for the Atari here.

## Settings and demo player (new)

At the start of `prallball.asm` there is the block **EINSTELLUNGEN** (settings; can also be set with `-D NAME=value` when assembling):
`EXTRA_RATE` (40: chance in 256 per destroyed brick for capsules; about twice as many as before), `DEMO_REAKTION_MIN`/`DEMO_REAKTION_MAX` (4/12 frames of reaction time after the ball changes direction), `DEMO_FEHLER_RATE`, `DEMO_FEHLER_FERN`, `DEMO_UNGENAU`, `DEMO_VERFEHLEN_RATE`, `DEMO_EXTRA_LUST`, `DEMO_EXTRA_FERN`.
The demo player thus acts more human (reaction time, short mistakes, imprecise hitting, rare deliberate misses) and catches good falling capsules as long as the ball is far away.
The test version `-D AUTOPLAY=1 -D TESTDEMO=1` counts events of the demo player in RAM (see header comment).

**Angle randomness (new):** On each paddle hit the bounce angle is shifted by exactly one zone (17 degrees) to the left or right with the chance `WINKEL_ZUFALL` (default 64 of 256, about every fourth hit). This applies to the human player and to the demo player; the ball always stays within the eight zones, and at the edges it is not pushed beyond them. It prevents endless, identical ball paths. `0` switches it off (`-D WINKEL_ZUFALL=0`). With `-D AUTOPLAY=1` the computer now plays all levels without end (before: only the first five); `-D TESTDEMO=1` additionally counts the angle shifts, the paddle hits, the completed levels, the destroyed bricks and the levels ended by the time limit (addresses in the header of `prallball.asm`).

**Demo time limit (new):** If the demo player hits no brick for `DEMO_STEIN_LIMIT` seconds (default 40), the level counts as completed and the next one begins, so an unreachable or tough last brick cannot hold the demo up endlessly. Only in the demo (and with `AUTOPLAY`), never in the game for humans; `0` switches it off.
