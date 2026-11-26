# Tile editor

A **tile set** (`.vctiles`, kind `tileset`) consists of tiles of equal size **W x H** (1 to 16 characters per side). Each cell of a tile names a
character and the character set it comes from; a set may use several character sets. The map (which tile is where) is not in the set,
but in a screen bank (`.vcscreen`) that names the set in `tilesets`.

## Layout
The editor has three areas side by side (in narrow windows the right one moves down).

![The tile editor in the Graphics area with the file level_tiles.vctiles. On the left is the tile set with the tiles leer, stein, gras and kiste. In the middle are the large 2 by 2 tile and the character picker with a grid. On the right are the cards Kachel, Farbe, Eigenschaften, Groesse der Kacheln, Zielzeichensatz and Kachel.](bild:kachel-editor "The tile editor with tile set, tile area and character picker.")

- **Left** is the **tile set**: thumbnail, name and how often the tile is used in the screen banks of the project ("0x" means unused); a **=** marks a tile that is equal to an earlier one. **New tile** (plus) creates one. If the project has several tile sets, the **Tile sets** are listed below.
- **In the middle** are the toolbar (Undo, Redo, flip horizontally and vertically, **Rotate tile** by 90 degrees only for square tiles, roll cells in four directions, **Clear**, **Copy**, **Paste**, **Export ...**, **Save**) and the large tile. The left button (also dragging) puts the chosen character into the cell, the right button or Alt+click picks up the character of the cell as a brush, the arrow keys move, **Enter** sets the character, **I** picks it up. Next to it is the **character table** (with tabs if there are several character sets).
- **Right** is the section **Tile**: name, **Color**, **Background color**, **Properties**, **Size of the tiles** (width and height, 1 to 16, then **Apply**; when making it smaller, filled cells are lost, and the editor asks first), **Target charset** and the tile actions **Duplicate**, **Merge** (for a tile that is equal to an earlier one: the tile is dropped, and the banks then point to the earlier one) and **Delete**. If there are screen banks that use the set, the editor asks before deleting: references to the deleted tile become tile 0, higher numbers move up, and the set is saved.

The dialog **New file** does not yet create a new tile set in this version.

## Color
The color type applies to the whole set (section **Tile**, **Color applies**): *Per tile* (one color byte per tile, the most compact), *Per cell* (each cell has its own color; above the tile you choose whether a click sets character and color, only the character or only the color) or *None*.
On the Atari, the switch "Set character bit 7 (color selection of the character)" sets character bit 7 (second color group, target character set at most 128 characters).

## Character sets and usage
The tabs above the character table show the character sets of the set. Under **Target charset**, **Add** adds one from the project (only with the same profile as the first one), **Remove <name>** takes an unused one out again. On export, **one** target character set is made from all used glyphs; equal images count once. The bar shows how full it becomes (for example "187 of 256 characters used") and reports an overflow.

## Properties
Freely named features such as "solid" or "ladder" (at most eight per set): type the name under **New property**, click **Add**, then set the switch for each tile. The export writes them as the byte table `<name>_props` and the constants `<name>_p_<property>`.

## Export
**Export ...** in the toolbar opens the [Export dialog](help:export) as in the other editors. Result: one table per cell `<name>_c0 ...` (index = tile number), `<name>_col` (per tile) or `<name>_col0 ...` (per cell),
`<name>_props`, the merged character set `<name>_chars` and the constants `_tw`, `_th`, `_count`, `_glyphs`. The set needs at least one character set.

## Examples
- `examples/c64/jumprun`: a tile set `hg` (21 tiles of 2x2, properties "solid" and "ladder") and a screen bank `levels` with the four images; drawing, map copy into RAM and properties come from the library (`#include <c64/tiles.asm>`). All 256 characters of the character set are pinned, so the target character set `hg_chars` is the game character set. Checking the images in the emulator: `python3 tools/check_jumprun_c64.py`.
- `examples/c64/gfx_demo`: tile set `level_tiles` and bank `level`; `gfx_draw_map` in `lib/gfx.inc` is now only a wrapper around `screen_draw`.
- `examples/c64/tiles_demo`: all sizes and variants of the library with tests.

## Old tile maps
`vcc` converts files of the earlier kind `tilemap` (tiles and map in one file) on the command line: `vcc media migrate-tiles` makes one tile set and one screen bank (one screen per map) per folder from all old maps of the project; with `--keep` the old files stay.
