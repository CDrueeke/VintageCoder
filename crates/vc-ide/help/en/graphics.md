# Sprites, character sets, screens and songs

Sprites, character sets, screens, tiles, palettes and songs are stored as text files `<name>.vc<type>` (for example `font.vcchar`) in the project (pixels as hex lines, easy to compare with diff). You find the editors in the main section **Graphics** (second row: Sprites, Character set, Tiles, Screen, Palette); you find the tracker for songs in the main section **Music**. A double-click on the file in the project tree opens the matching editor. You create new files for sprites and character sets in the editor itself (**New sprite file …**, **New charset …**): In the dialog, you choose the **profile** (for example `c64.sprite.multicolor`) under **Type and system**, the **name** and the **folder in the project**. You open tile sets, screens, palettes and songs from the project; there is no dialog of its own to create them.

| Editor | Content | In the new user interface |
|---|---|---|
| [Sprite/PMG](help:sprite) | C64 sprites, Atari players/missiles | built |
| [Character set](help:charset) | 256 or 128 characters | built |
| Palette | Palettes (MEGA65) | built |
| [Screen](help:screen) | Text screens 40x25/24 and screen banks made of tiles | built |
| [Tiles](help:tiles) | 2x2 tiles | built |
| [Maps](help:screen) | screen banks as maps made of tiles (levels) | built |
| [Tracker](help:tracker) | Songs (SID, POKEY), main section **Music** | built |
| [PLAY tracker](help:play) | Songs for the PLAY command of the MEGA65, also in the main section **Music** | built |

## Layout of the editors
The sprite editor, the character set editor, the palette editor, the tile editor and the screen editor have a similar layout and always show one file. If it is changed, the footer shows "● unsaved".

- **On the left** the list: in the sprite editor the sprite files and the frames, in the character set editor the character overview, in the palette editor the color grid, in the tile editor the tile set, in the screen editor the character table with the character color.
- **In the middle** the **toolbar** with the drawing tools, Undo, Redo, Flip, **Grid**, zoom and (sprites only) **Onion Skin**, below it the canvas. On the right in the toolbar are the file buttons **Export ...** and **Save** (in the sprite editor and character set editor also **Load PNG …**). The tile editor and the screen editor have tools of their own, see [Tiles](help:tiles) and [Screen](help:screen).
- **On the right** the cards with colors, pixel values and properties.
- **Footer:** name, size, cell position and messages.

![The Grafik tab with the Sprites page and the file hero.vcsprite: on the left the sprite files and two frames, in the middle the 12x21 pixel multicolor sprite of a figure in yellow, brown-red and blue, on the right the cards for colors, sprite (multicolor on), preview in the game and move. Below the drawing area a note says there is no animation.](bild:sprite-c64 "The sprite editor with a multicolor sprite.")

**Canvas and space (sprites, character sets):** the canvas fits its zoom to the free space. **-** and **+** in the toolbar, the keys **+** and **-** or **Cmd/Ctrl + mouse wheel** set the zoom; **Fit** (key **0**) returns to automatic fitting. If the window becomes narrow, the editors arrange their cards one below the other.

## Common operation
- Cmd/Ctrl+S saves, Cmd/Ctrl+Z and Cmd/Ctrl+Shift+Z undo and redo. **Export ...** (toolbar, Cmd/Ctrl+Shift+E) opens the export dialog, see [Export](help:export).
- With the right mouse button, you erase on the canvas (sprite editor, character set editor).
- The colors come from `data/palettes` for each target system. On the Atari, the **Palette view** in the **Colors** card selects *Project*, *PAL* or *NTSC*; this changes only the view of the colors, not the file.
- **Load PNG …** imports an image into the selected frame or character: scaling, quantizing to the target palette, a message for color conflicts.
- The single keys of the tools are listed under [Keyboard shortcuts](help:shortcuts).

## Loading in the program
Libraries for loading character set, screen and tiles: `examples/c64/gfx_demo/lib/gfx.inc`, `examples/atari/gfx_demo/lib/gfx.inc`; for BASIC you include the exported `DATA` lines with `#include` (see [BASIC](help:basic-directives)).

## Importing character sets from ROM
The command `vcc media import-rom <file> --profile ID --offset N --name NAME` reads the built-in character set of a system from a ROM image and creates it as a character set under `graphics/` (this copy contains the original characters and is not meant to be checked in). It **recognizes the image by size and checksum**; only unknown files are guessed from the file size, like this:

| File | Size | Character set |
|---|---|---|
| C64 character set image (for example `chargen.bin`) | 4096 | Set 1 from `$0` (uppercase/graphics), set 2 from `$800` (uppercase/lowercase) |
| Atari XL/XE operating system (`ATARIXL.ROM`) | 16384 | from `$2000` (= `$E000` in memory) |
| Atari 400/800 (`ATARIOSB.ROM`) | 10240 | from `$800` |
| Open ROMs character set (`chargen_openroms.rom`) | 4096 | C64 and MEGA65 profile (full color, foreground = color 1) |

Note: The imported character sets stay in your project. The Open ROMs character sets can be used freely.

## Character sets as a PNG grid
A character set can also be exchanged as an image: **16 characters per row in reading order**, set pixels light on a dark background (128x128 pixels for 256 characters of 8x8; images enlarged by a whole factor also work).
- Import: `vcc media import-sheet raster.png --profile c64.char.hires --name font [--invert]`.
- Export: `vcc media export-sheet graphics/font.vcchar font.png`.
The image is converted into a character set during import; in the program, as always, the bytes of the media file (converted when it is included) are used. The file formats of VintageCoder (`.vcsprite`, `.vcchar`, `.vcscreen`, `.vctiles`, `.vcpal`, `.vcsong` and more) can be used freely; anyone may read and write them. Note on copyright: the file format does not change whether characters of an original device are protected.
