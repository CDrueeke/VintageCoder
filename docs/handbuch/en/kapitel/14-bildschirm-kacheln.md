---
id: bildschirm-kacheln
titel: Screen and tile editor
reihenfolge: 14
stufe: Beginner
zeit: 35 minutes
siehe_auch: [zeichensatz, sprites, paletten]
referenz: [screen, tiles, export]
i18n_status: maschinell
i18n_quelle: a3453ed41975
---

# Screen and tile editor

**Goal:** You design a text screen (for example a title picture), build a playfield from it with **tiles**, and show both in your program.

## What you need

- a character set from [chapter 12](kapitel:zeichensatz) (or the font of your system),
- a C64, VIC-20, Plus/4, Atari or MEGA65 project. The examples run on the C64.

## What are the editors for?

Two editors work together here, and a third one follows later:

- The **screen editor** designs **one** text screen: title pictures, menus, displays, pages of a text adventure.
- The **tile editor** builds **building blocks** (tiles) from several characters: wall, grass, crate, ladder.
- The **map editor** (the screen bank) puts tiles together into **playfields**: levels, rooms, world maps.

Typical uses from real retro projects:

- **A title picture** made of text and colors that appears at the start.
- **A platform game:** few tiles, many levels; each level is a small map.
- **A role-playing game with a world map:** rooms as screens of a bank; the program chooses the room by number.
- **A menu system:** several screens for the main menu, settings, high scores.

In the finished program these are data tables (character codes, colors, map rows) plus a character set that your program copies into memory and reports to the graphics chip.

## Basic terms without prior knowledge

- **Cell:** a place on the text screen (the size of one character).
- **Character:** the picture pattern of a cell, from the [character set](kapitel:zeichensatz).
- **Color RAM:** the memory area that holds the color of each cell (C64).
- **Tile:** a building block made of cells, for example 2x2.
- **Tile set:** all tiles of a project, all of the same size.
- **Map / screen:** a playfield made of tiles; **bank:** a file with several screens.
- **Property:** a freely named feature of a tile ("solid", "ladder") that your program can query.
- **Target character set:** the character set that the export builds from all characters used by the tiles.
- **Video standard:** PAL (Europe, 50 Hz) or NTSC (USA, 60 Hz); it affects colors and timing.

## More terms

- A **text screen** is a grid of **cells**. On the C64 it is 40x25, on the VIC-20 22x23 and on the Atari (ANTIC 2) 40x24; the size is in the profile of the target system. Each cell holds a character from the character set. On the C64, each cell also gets a color from the **color RAM**.
- A **tile** is a building block made of several cells, for example 2x2 characters that together make a wall. A **tile set** (`.vctiles`) holds many tiles of the same size.
- A **screen bank** (`.vcscreen`) is a file with one or more **screens** that are built from tiles of a tile set. It is the **map**: it says which tile is where.

Why tiles? A whole screen is 1000 bytes. A map made of 2x2 tiles (20x12 = 240 bytes) is much smaller. You draw a few building blocks once and use them as often as you like. This is the classic way to build playfields.

## My first screen (using `gfx_demo` as the example)

1. **Open the example:** On the start page, open `examples/c64/gfx_demo` from the example projects (to make changes, use **Copy**). *Expected:* In the file tree are `graphics/title.vcscreen`, `level_tiles.vctiles`, `level.vcscreen` and `font.vcchar`.
2. **Open the screen:** At the top, choose the main section **Graphics** and in the second row **Screen**. If nothing is open yet, the editor shows the **Screen files** as buttons; click `graphics/title.vcscreen`. A double click in the file tree works too. *Expected:* On the left the character table (and on the C64 the character color), in the middle the title picture as a 40x25 grid, on the right the card **Screen**.
3. **Change a character:** Tool **Set character** (key **D**), choose a character in the character table, click in the area. *Expected:* The character is in the cell, **●** appears in the footer.
4. **Type text:** Tool **Type text** (**T**), click a cell and write. *Expected:* Your letters appear cell by cell.
5. **Save, build, run:** **Cmd/Ctrl+S**, **F7**, **F5** (with the program from chapter 12). *Expected:* The changed title picture in the emulator.

## Layout of the user interface

Both editors have three areas side by side. In narrow windows the right area moves below the other two.

| Area | Screen editor | Tile editor |
|---|---|---|
| Left | **Characters**: the character table with the name of the character set, on the C64 the **Character color** below it; with several screens the **Screen files** | the **Tile set**: all tiles as thumbnails with name and number of uses, **New tile** (plus); with several sets the **Tile sets** |
| Middle | toolbar, hint line, the screen, footer | toolbar, the large tile with the character table next to it, footer |
| Right | card **Screen**: size, system colors (border, background, characters), charset, output | card **Tile**: name, color, background color, properties, size of the tiles, target charset, tile actions |

**Keys in the screen editor:** **D** Set character, **R** Frame, **F** Fill, **T** Type text, **I** Eyedropper, **E** Erase. In the window, **Cmd/Ctrl+Z** (undo), **Cmd/Ctrl+Shift+Z** (redo) and **Cmd/Ctrl+S** (save) also work. You also find all actions in the command palette (**Cmd/Ctrl+K**). You can switch the single keys off in the settings ("Single keys in the editors").

## Step by step: Design a text screen

1. Open a screen (see above). The **New file** dialog does not yet create a new screen in this version; edit an existing one or make a copy of the example.
2. The screen is in the **middle**. On the **left** you choose the character from the **character table** (arrow keys move from character to character) and on the C64 the **Character color**. On the **right**, under **System colors**, you set **Border**, **Background** and **Characters** (on the Atari you also choose the video standard).
3. The tools: **Set character** (a click sets the chosen character, the right mouse button erases, **Enter** sets it at the cursor cell), **Frame** (drag, or set the corner and the opposite corner with **Enter**; **Esc** cancels), **Fill**, **Type text** (click a cell, then write; arrow keys move the cursor, **Esc** ends), **Eyedropper** and **Erase**.
4. If the screen should use **your** character set, choose it on the right under **Charset**. With **No charset** the system character set appears (or, without a ROM, the own set of VintageCoder).
5. **Cmd/Ctrl+S** saves.

![The screen editor in the Graphics area with the file title.vcscreen. On the left are the character picker and the list of screen files. In the middle is the 40 by 25 text screen with a cyan frame made of characters and some white and yellow characters. On the right are the cards for size, system colors, character set and output.](bild:bildschirm-editor "The screen editor with character picker, screen area and settings.")

The example project `examples/c64/gfx_demo` has such a screen (`graphics/title.vcscreen`, with the character set `graphics/font.vcchar`). How you show it is already described in [chapter 12](kapitel:zeichensatz).

## Step by step: Draw tiles

1. Choose **Graphics** > **Tiles**. If nothing is open yet, the editor shows the **Tile sets** of the project as buttons; in the example `gfx_demo` it is `graphics/level_tiles.vctiles`. The **New file** dialog does not yet create a new tile set in this version.
2. The set needs at least one **character set**. On the right under **Target charset**, choose one from the list and click **Add**. Without a character set there is nothing to build tiles with. If the set has several character sets, choose the tab above the character table.
3. On the left is the **Tile set**: thumbnail, name and how often the tile is used in screen banks ("0x" if never). A **=** tells you that a tile is the same as an earlier one. **New tile** (plus above the list) creates one. On the right, in the card **Tile**, are **Duplicate**, **Merge** (for identical tiles) and **Delete**. If the tile is used in screen banks, the editor asks before deleting.
4. In the **middle** is the large tile. The **left mouse button** (also dragging) puts the chosen character in the cell, the **right mouse button** (or **Alt**+click) picks up the character of a cell as the brush. With the arrow keys you move from cell to cell, **Enter** sets the character, **I** picks it up. You choose the character in the **character table** next to the tile.
5. In the toolbar are **Flip** (horizontal, vertical), **Rotate tile** (**90°**, only for square tiles), **Roll cells** (four arrows), **Clear**, **Copy** and **Paste**. You set the **size** of the whole set on the right under **Size of the tiles** (**Width** and **Height**, 1 to 16 characters per side, then **Apply**; when you make it smaller, cells are lost, and the editor asks first).
6. You set the **color** for the whole set on the right under **Color**: **Per tile** (one color value for the whole tile, the most compact), **Per cell** (each cell its own color; then you choose above the tile whether a click sets **Character and color**, **Character only** or **Color only**) or **None**. The **Background color** comes in addition.

![The tile editor in the Graphics area with the file level_tiles.vctiles. On the left is the tile set with the tiles leer, stein, gras and kiste. In the middle are the large 2 by 2 tile and the character picker with a grid. On the right are the cards Kachel, Farbe, Eigenschaften, Groesse der Kacheln, Zielzeichensatz and Kachel.](bild:kachel-editor "The tile editor with tile set, tile area and character picker.")

### Properties and usage

On the right under **Properties** you assign freely named features, for example "solid" or "ladder" (at most eight per set): type the name in **New property**, then **Add**. For each tile you set the switch. A game can later query whether you may stand on a tile. The export writes them as a table `<name>_props` and as constants `<name>_p_<property>`.

All used characters of all tiles make **one** shared **target character set** on export. Identical pictures count only once. The bar under **Target charset** shows how full it is (for example "187 of 256 characters used") and reports an overflow. On the C64, Atari and MEGA65 there is the switch **Set character bit 7 (color selection of the character)**.

## The map: screen bank and map editor

If you open a screen bank (for example `graphics/level.vcscreen` from `gfx_demo`), the map editor appears automatically.

- **Screens below:** Below the map is the bar with all maps (screens) of the bank, each with a thumbnail and a name. A click chooses one; **New screen** appends an empty map, **Copy screen**, **Delete screen** (Undo brings it back) and **Move forward/Move back** set the order; you can also change the order by dragging a screen in the bar to another place (one undo step). The number ("Screen 2 of 4") is the selection number that your program uses to choose the map.
- **Tile mode:** On the left is the **tile grid** of the tile set. A click chooses the brush. Tools (keys in brackets): Pen (**D**), Line (**L**), Rectangle (**R**, filled or as a frame), Fill (**F**), Eyedropper (**I**, also the right mouse button), Select (**A**), Stamp (**S**), Erase (**E**, sets tile 0). While you drag, the cells you touch light up at once; a whole stroke is **one** undo step.
- **Selection:** Drag a rectangle with the Select tool. Then you can **keep it as a stamp** (the Stamp tool places it again anywhere), **clear** it (Del), move it with **Alt+arrow keys** or **create it as a tile ...** (a dialog asks for the name and whether the new tile should replace the selection on the map; if the cells already exist as a tile, that tile is used).
- **Cell mode:** With the **Tab** key (or the switch **Tiles | Cells**) you change to the **cell layer**: On the left the character table appears; you put single characters **over** the tiles (at most 255 such exceptions per map). The tool **Erase** lets the tile underneath show through again; the text tool (**T**) puts typed characters from the cursor on.
- **Layers:** On the left you switch **Tile layer** and **Cell layer** on and off separately, and with **Show properties** you make a chosen property (for example "solid") appear in color.
- On the right you set **Name**, **Tile set**, **Size** and **Origin** of the tile grid and choose the colors of the map. What does not fit, for example the 25th row on the C64 with 2x2 tiles, stays a leftover cell (reserved for the cell layer only). You can set the view to **PAL** or **NTSC**.
- **View:** In the toolbar you zoom (**-**, **+**, "Fit to width") and show the **cell grid** and **tile borders**. With the arrow keys you move the cursor over the map; the status line names column, row and tile.

![The 'Grafik' area with the tab 'Karten' in the project gfx_demo. On the left the four tiles of the tile set and the switches for layers, in the middle the map 'level' with a floor of tiles and a strip of grass, on the right the fields for name, size, colors and output. The levels are listed below the map.](bild:kachel-karte "The map editor with tile set, map and the map settings.")

### A text screen becomes a map

If you have painted a text screen that contains repeating blocks, **Split into tiles ...** (toolbar of the screen editor) converts it. In the dialog **Split into tiles** you choose the **Tile size** (2 x 2, 3 x 3 or 4 x 4); the preview names the result beforehand. **Split** creates `<name>_tiles.vctiles` and `<name>_bank.vcscreen` next to the old screen; the old one stays. After that, the tile editor opens the new tile set.

## Export

As everywhere: toolbar > **Export ...** (or **Cmd/Ctrl+Shift+E**) or automatically when building. Result:

- **Screen:** `<name>_chars` (character codes) and `<name>_attrs` (colors, C64) as well as the constants `_width`, `_height`, `_border` and `_bg`.
- **Tile set:** tables `<name>_c0 ...` (one per cell of the tile; index = tile number), `<name>_col`, `<name>_props` and the target character set `<name>_chars`; plus the constants `_tw`, `_th`, `_count`, `_glyphs`.
- **Screen bank:** one map `<name>_map<i>` per screen and the pointer tables `<name>_screens_lo/hi`.

## Example: Playfield from tiles

In the project `examples/c64/gfx_demo`, these belong together: `graphics/level_tiles.vctiles` (2x2 tiles, among them empty, stone, grass and crate), `graphics/level.vcscreen` (the map, 20x12 tiles) and `graphics/font.vcchar`. This program draws the map:

```asm
.target c64
.cpu 6510 { illegals = deny }

.org $0801
.basicstub start

.include "lib/gfx.inc"

start:  sei
        gfx_load_charset level_tiles_chars, $2800   ; target character set of the tile set
        gfx_charset_at $2800
        gfx_screen_at gfx_screen
        tiles_use level_tiles_c0, level_tiles_col, level_tiles_props, level_tiles_tw, level_tiles_th, level_tiles_count, level_screens_lo, level_screens_hi, level_level_tiles_w, level_level_tiles_h
        gfx_clear 32, level_level_bg
        gfx_colors level_level_border, level_level_bg
        gfx_draw_map level_level_id
done:   jmp done

gfx_runtime
tiles_runtime
.include "graphics/level_tiles.vctiles"
.include "graphics/level.vcscreen"
```

- First the **target character set** of the tile set (`level_tiles_chars`) goes into memory, because the tiles are made of **its** characters.
- `tiles_use` connects the tile set with the bank. The long names are labels from the two export files. It is the line that people copy most often.
- `gfx_draw_map` draws the chosen screen (here `level_level_id`, number 0) into the screen memory and the color RAM.
- `tiles_runtime` and `gfx_runtime` place the subroutines. Both are **after** the main program.

1. Open `examples/c64/gfx_demo`, replace `gfx_demo.asm` with the program above (or work in a copy).
2. Build with **F7**, run with **F5**. You see a playfield with a stone floor, a row of grass, a stone column and a floating crate.
3. Open `graphics/level_tiles.vctiles`, change a tile in the tile editor (for example the crate), save, build and run again. The map shows the changed tile everywhere it stands.

The original `gfx_demo.asm` shows the map instead of the title picture with the switch `-D DEMO_MAP=1`. A bigger example with properties and four maps is `examples/c64/jumprun`.

## Hardware details and limits

| System | Screen | Characters / color | Special feature |
|---|---|---|---|
| C64 | 40x25 cells, 1000 bytes + 1000 colors | 256 characters, 16 colors | Color RAM `$D800`; screen memory and character set must be in the same 16 KB bank of the VIC |
| VIC-20 | 22x23 | 256 characters | little memory; the memory expansion decides the addresses |
| Plus/4 | 40x25 | 256 characters, 121 colors (TED) | TED palette not complete in the editor yet (note "TED palette missing") |
| Atari (ANTIC 2) | 40x24 | 128 characters | inverse display through bit 7 |
| MEGA65 | rows and columns depending on the profile | like C64, plus full-color characters | 256-color palette |

Limits of tiles: at most **16** characters per tile side, at most **255** cell exceptions per screen, a tile set of up to 256 character pictures in the target character set. If 256 characters are not enough, you merge characters or use fewer different building blocks.

## From the editor into the program

- **Assembler:** `.include "graphics/<name>.vctiles"` or `.vcscreen` after the main program, plus `lib/gfx.inc` (`gfx_load_screen`, `gfx_draw_map` ...), see the programs in this chapter.
- **BASIC:** export in the format **BASIC DATA**, include it with `#include` and copy it into the screen memory with `READ`/`POKE`.
- **Raw data:** format **Binary** for `.incbin` or for loading from disk.
- **Atari and MEGA65:** the same files, other target addresses; for the Atari, `examples/atari/gfx_demo` shows the way.

## Common pitfalls

- **The map shows nonsense:** The target character set was not loaded or is not on a 2 KB boundary. It must be registered with `gfx_charset_at`.
- **The map is not drawn:** `tiles_use` is missing or names wrong labels, or `tiles_runtime` is missing after the program.
- **"Overflow" in the target character set:** You use more than 256 different character pictures. Delete tiles or merge characters (**Merge** for identical tiles).
- **You cannot draw anything in the tile editor:** The set has no character set. First add one on the right under **Target charset**.

## Neighbor editors and glossary

- You paint characters in the [Character set editor](kapitel:zeichensatz), you choose colors from the [palette](kapitel:paletten), and moving figures are [sprites](kapitel:sprites).
- Terms: glossary (file `GLOSSAR.md`): tile, map, bank, cell, color RAM.

## What next?

If you want to define your own colors (MEGA65), the next chapter is about [Palettes](kapitel:paletten). For reference: [Screen editor](hilfe:screen), [Tile editor](hilfe:tiles), [Export](hilfe:export).
