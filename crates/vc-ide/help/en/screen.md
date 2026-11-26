# Screen editor

Text screen; the size comes from the profile: 40x25 with color RAM (C64), 22x23 (VIC-20), 40x24 (Plus/4, Atari ANTIC 2).

- Tools: place character, type text, fill, frame, eyedropper, clear.
- On the right, choose the **character code** from the character set and the **color** (C64: color per cell).
- The linked character set is the character source; without it, the system character set.
- Export: `<name>_chars` (and `<name>_attrs` with color values on the C64) plus the constants `_width`, `_height`, `_border`, `_bg`.
- Loading: `gfx_load_screen` (assembler) / `gosub @gfx_screen` (BASIC).

## Layout
The editor has three areas side by side (in narrow windows, the right one moves down).

![The screen editor in the Graphics area with the file title.vcscreen. On the left are the character picker and the list of screen files. In the middle is the 40 by 25 text screen with a cyan frame made of characters and some white and yellow characters. On the right are the cards for size, system colors, character set and output.](bild:bildschirm-editor "The screen editor with character picker, screen area and settings.")

- **On the left** the **Character table** with the name of the character set (the arrow keys choose a character) and, where there is color RAM (C64), the **Character color**; if the project has several screen files, the **Screen files** are below it.
- **In the middle** the toolbar (Set character **D**, Frame **R**, Fill **F**, Type text **T**, Eyedropper **I**, Erase **E**, Undo, Redo, **Split into tiles …**, **Export ...**, **Save**), a hint line for the chosen tool and the screen. A click or drag applies the tool, the right mouse button erases; the arrow keys move the cursor, **Enter** applies the tool at the cursor cell.
- **On the right** the section **Screen**: size, **System colors** (border, background, characters), **Charset** (selection or **No charset**, then the system character set applies), on the Atari the video standard (Project, PAL, NTSC) and **Output** with **Export ...**. This opens the [Export dialog](help:export).

The grid size comes from the profile of the target system. The dialog **New file** does not create a new screen in this version yet.

## Screen banks with tiles
A `.vcscreen` file with `screens` is a **screen bank**: one or more screens built from tiles of a [tile set](help:tiles). You edit it in the **Map editor** (Graphics > Maps, see [Screen and tile editor](kapitel:bildschirm-kacheln)). A bank can be exported like any screen; the bank delivers per screen `<name>_map<i>`, the patch lists and the shared target character set.

### Splitting into tiles
In the screen editor (text screen), the button **Split into tiles …** in the toolbar converts a character screen (dialog "Split into tiles"): choose the **Tile size** (2 x 2, 3 x 3 or 4 x 4), and the preview names the result. **Split** creates `<name>_tiles.vctiles` and `<name>_bank.vcscreen` next to the old screen and opens the new tile set; the old screen stays.
