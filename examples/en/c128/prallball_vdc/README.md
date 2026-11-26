# PRALLBALL (Commodore 128, VDC version, 80 columns) - prototype

Second C128 version next to the 40-column version `examples/c128/prallball` (VIC-II). The **VDC** (8563/8568) can only do text: no sprites, no raster interrupt, no
multicolor characters; but it has 80 columns, two character sets and one color per cell. The game logic is that of the C64/Plus/4 version; the display is new. The source is extensively commented.

- **Build:** `vcc build .`; start in **80-column mode**: `x128 -80col -autostart build/prallball.prg` (or `build/disk.d64`). On a real C128, press the 40/80 DISPLAY key before starting
  (the monitor must be connected to the VDC output).
- **Controls:** joystick port 2 or keyboard (Z left, X right, space fire, P pause, RUN/STOP abort); name entry: type + RETURN.
- **Display:** every logical cell (40 x 25 as on the C64) consists of **two VDC cells**: the character is stretched to 16 x 8 pixels (the left four pixels on the left, the right four pixels on the right, each doubled; left
  character set 1, right character set 2 via attribute bit 7). All texts and the layout stay unchanged. The screen is in the VDC ($0000 characters, $0800 attributes, $2000/$3000
  character sets); main memory holds mirrors, shadow copies of the background and the character set (1 bit/pixel).
- **Character sprites** (ball up to 3, paddle, coin, laser shots): as on the Plus/4, `tsp_render` assembles pool characters (128..191) every frame from background AND mask OR image
  (single color, 8 fine positions, accurate to 1 logical pixel) and loads them **stretched into the VDC**. Cells that are also occupied in the next frame are not cleared briefly (no flicker).
- **Differences from the VIC-II version:** single-color bricks (color per cell instead of light edge/shadow), a dark background color per level (VDC register 26) instead of the raster bars,
  the ticker runs through changed character bit patterns (only differences are loaded), the scroller moves by a whole character every 6 frames (fine scrolling of one row is not possible on the VDC),
  short background flashes (brick hits) are omitted. The VIC-II is dark and only supplies the frame clock (raster interrupt line 251); the processor runs at **2 MHz**, for the high score list
  at 1 MHz.
- **Memory / MMU:** like the VIC-II version (configuration `$3E`, all RAM, own interrupt vector); high score list and `copy_font` below `$4000`. The high score list is read **before** switching
  to 2 MHz and the dark VIC (afterwards the KERNAL got stuck in the fast serial transfer); saving at the game end works (1 MHz meanwhile).
- **State:** title (logo, top 5, ticker, scroller, demo after 30 s), 24 levels, extras, multiball/laser, bonus/boss levels, high score list on the start drive. Checked in `x128 -80col`.
- **Not yet / unchecked:** real C128 (emulator only; VDC access times and flicker on the device unknown), sound not listened to, keyboard/joystick not typed, 64 KB VDC not needed (16 KB are enough),
  NTSC not tested. In the worst case (3 balls + laser + coin) the main loop misses about 2-3 % of the frame changes (`TESTFPS`).
- **Tests:** `python3 tools/gen_tests.py && vcc build . && vcc test . --backend vice` (x128, trap `$BF00`, `"$ff00": 62`): 103 emulator tests; the VDC contents are read back with `vdc_get`
  (screen, attributes, stretched pool characters, ticker), plus logic tests, compositor with independently computed bit patterns, high score list, sound steps taken over as on the C64.
- **Test versions (`-D`):** `AUTOPLAY`, `STARTLEVEL=n`, `TESTMULTI`, `TESTLASER`, `TESTLOSE`, `TESTFPS`, `TESTCAP`; build: `vcc asm prallball.asm --target c128 --cpu 8502 -D AUTOPLAY=1 -o x.prg`.
- **Tools:** `tools/gen.py` (direction table, levels, characters, sprite strips, stretch tables), `tools/make_levels.py` (level drafts 4-24 to `levels/levels.txt`),
  `tools/compose.py` (title music to `lib/music.inc`), `tools/gen_tests.py` (generates `prallball.vctest`). Further switch: `NOTICKER` (without demo ticker); overview in the header of `prallball.asm`.

## Settings and demo player (new)

At the beginning of `prallball.asm` is the **EINSTELLUNGEN** block (can also be set with `-D NAME=value` when assembling):
`EXTRA_RATE` (40: chance in 256 per destroyed brick for capsules; about twice as many as before), `DEMO_REAKTION_MIN`/`DEMO_REAKTION_MAX` (4/12 frames of reaction time after a change of direction of the ball), `DEMO_FEHLER_RATE`, `DEMO_FEHLER_FERN`, `DEMO_UNGENAU`, `DEMO_VERFEHLEN_RATE`, `DEMO_EXTRA_LUST`, `DEMO_EXTRA_FERN`.
The demo player thus seems more human (reaction time, short mistakes, inaccurate hits, rare deliberate misses) and catches good falling capsules as long as the ball is far away.
Test version `-D AUTOPLAY=1 -D TESTDEMO=1` counts events of the demo player in RAM (see the header comment).

**Angle randomness (new):** On each paddle hit the bounce angle is shifted by exactly one zone (17 degrees) to the left or right with the chance `WINKEL_ZUFALL` (default 64 of 256, about every fourth hit). This applies to the human player and to the demo player; the ball always stays within the eight zones, and at the edges it is not pushed beyond them. It prevents endless, identical ball paths. `0` switches it off (`-D WINKEL_ZUFALL=0`). With `-D AUTOPLAY=1` the computer now plays all levels without end (before: only the first five); `-D TESTDEMO=1` additionally counts the angle shifts, the paddle hits, the completed levels, the destroyed bricks and the levels ended by the time limit (addresses in the header of `prallball.asm`).

**Demo time limit (new):** If the demo player hits no brick for `DEMO_STEIN_LIMIT` seconds (default 40), the level counts as completed and the next one begins, so an unreachable or tough last brick cannot hold the demo up endlessly. Only in the demo (and with `AUTOPLAY`), never in the game for humans; `0` switches it off.

