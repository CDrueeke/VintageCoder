# Sprite/PMG editor

Drawing area for C64 sprites (hires 24x21, multicolor 12x21) and Atari players/missiles (8 pixels wide, 4 frames). You find the editor in the main section **Graphics**, in the second row under **Sprites**. A double-click on a `.vcsprite` or `.vcpmg` file in the file tree also opens it there.

## Layout
| Area | What it shows |
|---|---|
| Left | the **Sprite files** of the project, **Open sprite file …**, **New sprite file …** and the tabs **Frames** and **Animations** |
| Middle | the toolbar, the **Drawing area** with the chosen frame, below it the timeline of the animation and the footer |
| Right | the cards **Colors**, **Palette**, on the Atari **Player/missile**, **Sprite**, **Preview in the game** and **Shift** |

![The Grafik tab with the Sprites page and the file hero.vcsprite: on the left the sprite files and two frames, in the middle the 12x21 pixel multicolor sprite of a figure in yellow, brown-red and blue, on the right the cards for colors, sprite (multicolor on), preview in the game and move. Below the drawing area a note says there is no animation.](bild:sprite-c64 "The sprite editor with a multicolor sprite.")

- **Tools:** **Pen** (D), **Eraser** (E), **Eyedropper** (I), **Fill** (F), **Line** (L), **Rectangle** (R) and **Selection** (S: drag, Cmd/Ctrl+C/X/V, Del). Also **Flip horizontally** (X), **Flip vertically** (Y), Undo, Redo, **Onion Skin** (O), **Grid** (G) and the zoom (Zoom out, Fit, Zoom in).
- The **right mouse button** erases.
- **Colors:** The card shows for each value (for example transparent, sprite color, multicolor 1 and 2; on the Atari the color of each player) the color and, where there is one, the register (for example `$D025` for multicolor 1 or `$D027+n` for the sprite color; on the Atari `COLPM0` to `COLPM3`). A click chooses the value that the pen sets, and so do the keys 1 to 4. Below it, the color grid assigns a palette color to the value. On the Atari, you switch the **Palette view** between **Project**, **PAL** and **NTSC**.
- **Frames:** tab **Frames** on the left with **New**, **Duplicate** and **Delete**; **Onion Skin** shows the previous frame as a shadow.
- **Shift:** four arrow buttons move the frame by one dot; **Clear frame** deletes all dots.
- **Export:** **Export ...** (toolbar, Cmd/Ctrl+Shift+E), see [Export](help:export): 64 bytes per sprite frame. In the program, set the pointer to `address / 64` (see `examples/c64/sprite_demo/sprite_demo.asm`).
- You can also reach all tools, zoom, grid and Onion Skin through the command palette (Cmd/Ctrl+K). The single keys are listed under [Keyboard shortcuts](help:shortcuts).

## New sprite file
**New sprite file …** (on the left below the file list, also on the empty page of the editor) opens a dialog: Under **Type and system** you choose the profile, enter the **Name** (letters, digits, `_` and `-`), check the **Folder in the project** (default `graphics`) and click **Create**. The profiles exist only for the target systems of the project.

## Hardware flags
For C64 and C128, the card **Sprite** shows three switches: **Double width** (X2, `$D01D`), **Double height** (Y2, `$D017`) and **Behind the background** (BG, `$D01B`). Multicolor (MC, `$D01C`) follows the mode of the chosen frame and has no switch of its own; you set the mode with the switch **Multicolor** in the same card. The three switches apply to the whole file and are stored as the optional field `hw` in the `.vcsprite` (`expandX`, `expandY`, `behind`). If all are off, the field is missing, and old files stay unchanged. The export then appends the constants `<name>_expand_x`, `<name>_expand_y` and `<name>_behind` (0 or 1). The card **Preview in the game** shows the chosen frame in front of a fixed background, with the switches that are set, and says how many bytes a frame and the file use.

For Atari players/missiles, the card **Player/missile** sets the **Width** of each player (SIZEP0 to SIZEP3: Normal, Double, Quadruple) and the **Priority** (PRIOR). The export writes both as constants. A player shows its memory line by line from top to bottom; the first 32 or so lines lie above the visible picture.

## Mixing hires and multicolor (C64)
A sprite asset belongs to a profile (hires 24x21 or multicolor 12x21), but each frame may have its own **mode**. In the card **Sprite**, you switch **Multicolor** on or off for the chosen frame. The editor adjusts the grid width, pixel values and colors. When you switch, the pixels are converted (hires to multicolor: every two pixels become one; multicolor to hires: every pixel becomes double width, and all colors except "transparent" become the sprite color). Cmd/Ctrl+Z undoes the switch. Multicolor colors 1 and 2 apply to all frames of the asset (like the registers `$D025`/`$D026`).

The export writes each frame in its mode and creates the constants `<name>_count` and `<name>_multicolor_mask` (bit i = frame i, for `$D01C`). For mixed frames it also creates the table `<name>_modes` (for each frame 0 = hires, 1 = multicolor, see below). "Onion Skin" and "Load PNG" apply only to frames in the mode of the profile.

## Animations
In the tab **Animations** (left), **New** creates a sequence with the chosen frame. You choose between several animations with their buttons, and you name them in the field **Animation name**. The steps are shown as a filmstrip: each one shows a thumbnail, its number and the **Duration in ticks** (1/50 s), and can be moved or removed. **Append selected frame** adds the chosen frame as a step. The **Playback** is *Loop*, *Back and forth* (the end frames are not shown twice) or *Once*. The **Timeline** below the drawing area plays the animation in real time. Frames in an animation may mix hires and multicolor. If a frame is inserted or deleted, the animations are updated.

You can also select several frames in the frame list with Shift-click or Ctrl-click (Cmd/Ctrl+A selects all) and choose **Animation from selection**.

If there are animations or mixed frames, the export writes a second file **`<name>.anim.asm`** next to `<name>.asm` (data that belongs after the sprite data: when you include the media file, `.include "graphics/<name>.vcsprite"`, they follow the sprite data by themselves). It contains, for each animation (`<name>_anim_<animation>_...`):
- `_ptr`: the sprite pointers, that is, the block number in the VIC bank (`(address & $3FFF) / 64 + frame`), with "Back and forth" already written out;
- `_delay`: the duration of each step in ticks;
- `_len`: the number of steps, `_mode`: 0 = loop, 1 = once.

In the raster interrupt, you count the duration down, and when it ends, you write the next pointer to the sprite pointer (`$07F8` + sprite number). The sprite data (`<name>`) must be placed on a 64-byte boundary, for example with `.org $2000`.

## Importing sprites from other sources
**Load PNG …** (toolbar) converts an image to pixel values of the chosen frame; the editor saves the file first. Importing from SPD files, raw data and sprites of other systems is not yet available in this version.
