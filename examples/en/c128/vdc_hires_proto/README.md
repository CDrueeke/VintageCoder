# C128: VDC hires prototype (640 x 200)

Feasibility study for the PRALLBALL extension. Not a game: no sound, no input.

## What it shows
- VDC bitmap 640 x 200 with color attributes per 8 x 8 pixels (register 25 = `$C7`, image start `$0000`, attributes `$4000`).
- 64 KB VDC detected (register 28 bit 4; mirroring test `$0000`/`$8000`); background image in VDC RAM from `$5000`.
- Brick wall (6 rows x 26 bricks, each brick with its own color), border, 8 balls (8 x 8 pixels) in 8 fine positions. Per frame: restore the old position from the background by **block copy**, draw the new position.
- Measurement of the frame time with the CIA timer (microseconds) and the raster line; result in main memory from `$C200` (layout in the source).

## Measured values (VICE 3.10 x128, `-80col -VDC64KB`, PAL, 2 MHz)
- 200 frames, 8 balls: **about 28 ms per frame** (a PAL frame is 20 ms), which is about **3.5 ms per ball** (erasing with 8 block copies + drawing 8 lines). With 8 balls it is not enough for 50 frames/s, with about 4 to 5 balls it is.
- The VDC access is the bottleneck: every register and data byte waits for the ready bit. This is **emulator time**; not checked on real hardware.

## Known defects
- Next to each ball there is a small vertical bar (remainder of the drawing routine, cause not clarified).
- Balls are drawn without combining (overwrite, no OR): they only fly in the free field below the bricks. For the game you need the combining (reading in the VDC is slow) or separate bitmap planes.
- Without a 64 KB VDC (16 KB) the prototype shows only the bricks.

## Reading the results
`bank ram`, then `m c200 c21f` in the monitor: `$C200` = `$FF` finished, `$C201` = 1 with 64 KB, `$C202/3` = largest number of raster lines (wraps around over 312 lines!), `$C204..6` sum, `$C207/8` frames, `$C210..3` last frame time in us.
