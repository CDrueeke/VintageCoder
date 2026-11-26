# PRALLBALL for the Commodore PET (4000 series, 40 columns)

Breakout clone, version for the PET with BASIC 4 (4032) and 32 KB RAM. Simpler than the other versions: **no color, no sprites, no sound chip.**

## Playing
Build with `vcc build examples/pet/prallball`; start with the disk `build/disk.d64`:

```bash
xpet -model 4032 -8 build/disk.d64 -keybuf 'load"*",8\nrun\n'
```

| Key | Action |
|---|---|
| Z or , | Paddle left |
| X or . | Paddle right |
| Space | Launch ball / leave title |
| P | Pause |
| RUN/STOP | End (restart of the computer) |

22 levels (the boss levels of the C64 version are omitted), brick types as in the other versions: brick a to h (10 to 80 points, alternating full block and grid), silver (2 hits), gold (3 hits), indestructible, explosive (`XX`, destroys the eight neighbors), surprise (`??`: wide paddle, one extra life or slower ball). The ball gets faster over time (eight steps). High-score table (10 entries) on the disk in `PRALL.HS`.

**Not included** (compared with C64/C128): multi-ball, laser, falling capsules, music, title animation, joystick.

## Technology
- **Picture:** 40 x 25 characters. Drawing uses the quarter-block characters of the PET character set (2 x 2 sub-cells per character = 80 x 50 sub-cells); the ball is one sub-cell, the paddle a row of sub-cells (lower half of row 23). The mapping mask -> screen code (`qtab`) is checked against the character ROM (`tools/run_tests.py`).
- **Two layers:** screen (bricks, walls, text) and overlay (`$6C00`, mask per cell for ball and paddle).
- **Interrupt:** The editor interrupt (vector `$0090`) is hooked: frame counter, clear keyboard buffer, count sound; then the old routine (keyboard scan, clock).
- **Input:** The editor (BASIC 4) stores the code of the held key in `$97` (`$FF` = none); the buffer must be empty for this (`$9E`). This makes the scan independent of the keyboard layout (graphics/typewriter keyboard).
- **Sound:** only beeps through the shift register of the VIA (line CB2, `$E848`/`$E84A`/`$E84B`).
- **High-score table:** The KERNAL of the PET has no `SETLFS`/`SETNAM`; the file parameters are in the zero page. Also, `OPEN` (`$FFC0`) and `CLOSE` (`$FFC3`) of the jump table are the BASIC statements on the PET; `$F563` and `$F2E0` (BASIC 4) are used. Files exist only if the program was started from disk (device in `$D4`).
- **Memory:** program from `$0401` (about 8.5 KB), level map `$6B00`, overlay `$6C00` - 32 KB RAM required.

## Testing
`python3 tools/run_tests.py` (after `vcc build`): single subroutines through the VICE text monitor (xpet, `-model 4032`), plus runs of the test versions `AUTOPLAY` and `TESTLOSE` (game over, name entry, saving to a disk copy). `vcc test` does not know the target `pet` yet.

## Not tested
Sound (only registers checked, not listened to), real keyboard (only `$97` set), 8032 (80 columns, different character width), PAL/NTSC differences (60 Hz in the test), BASIC 2 (2001, 3000 series), real hardware and a real disk drive.
