# Export

The editors for sprites, character sets, palettes, tiles, screens and songs have the button **Export ...** (menu File > Export ..., **Cmd/Ctrl+Shift+E**). It opens the export dialog for the file that is open in the editor. A strip with the output file and the chip "current" or "outdated" does not exist in the user interface yet.

![The 'Exportieren' (export) dialog over the sprite editor for graphics/hero.vcsprite. At the top are the formats Assembler, Konstanten, Binaer, PRG and BASIC DATA (Assembler selected), below that the name field with 'hero' and the generated assembler text. At the bottom are the ready files, a yellow note under 'Ziel im Quelltext' and the buttons 'Abbrechen', 'Im Emulator testen', 'In Projekt speichern', 'Als Include einfuegen' and 'In Quelltext kopieren'.](bild:grafik-exportieren "The export dialog with format, name, generated text and target choice.")

The exported files and the formats are free to use (see [License and acknowledgements](help:license)).

## The dialog
- **Format** (segment switch):

| Format | File in `build/export/` | Content |
|---|---|---|
| Assembler | `<name>.asm` | data as `.byte` lines with labels, plus constants (colors, sizes, for sprites the count and modes) |
| Constants | `<name>.inc` | `<name>_len` for each part and, if an address is given, `<name>_addr` |
| Binary | `<name>.bin` | the raw data, for several parts one file per part |
| PRG | `<name>.prg` | the raw data with a load address |
| BASIC DATA | `<name>.data.bas` | `DATA` lines with the label `@<name>`, to be included with `#include` (see [BASIC](help:basic-directives)) |

- **Name:** the label that is used as the prefix of the names in the export.
- **Address:** optional for *Constants*, the load address for *PRG* (`$2000`, `0x2000` or decimal). An invalid entry turns red, and for PRG it blocks the actions.
- **File list:** the files that this format creates, with their sizes.
- **Include a read routine:** only for BASIC DATA; the routine reads the DATA lines into memory with POKE, starting at the address (call it with GOSUB).
- **Target in the source:** the open source file into which the text is inserted (assembler for the format Assembler, BASIC for BASIC DATA).
- **Preview:** calculates the current state of the editor file, including unsaved changes, in a temporary folder. Nothing is created in the project. Tabs: *Text* and, except for BASIC DATA, *Hex* (the first bytes of the first file). Below it, it says "Ready: n files, n bytes" or, in red, the error.
- There is no choice of an assembler dialect (there is only `vcasm`) and no compression.

## Actions
- **Test in emulator:** saves, exports as assembler, builds the project and runs it like F5.
- **Save in project:** saves the editor file and writes the files of the chosen format to `build/export/`. The dialog stays open.
- **Insert as include** (for Assembler and BASIC DATA): saves, exports and inserts an `.include` line into the source file chosen under **Target in the source**. If no matching source file is open: first open the file into which you want to insert.
- **Copy into source** (for Assembler and BASIC DATA): writes the text itself at the cursor line of this source file, without an include.

## Automatic export
To include a file in a program you do not need the export: `.include "graphics/hero.vcsprite"` converts the file itself when you build. The dialog is the manual way with a free choice of format, name and address. On the command line: `vcc media export file --format asm|inc|bin|prg|data`.
