# Disk Editor

The disk editor opens disk images of the formats **D64**, **D71**, **D81** and **ATR** (Atari DOS 2). You find it in the main area **Data**, second row **Disk**. It also runs without a project; a project must be open only to take files into it.

To open:
- Click the image in the project tree,
- the button **Open disk image ...** in the disk editor,
- drag the image from Finder or Explorer onto the window,
- **Tree ▸ Open disk image ...** (Mac menu; otherwise the command palette Cmd/Ctrl+K).

You create an empty image with **New disk ...** (format D64, D71, D81 or ATR, plus name and ID; for ATR without them). Only one image is open at a time.

## Using it
On the left the card **Disk** (format, usage, actions), in the middle **Directory** and **Allocation map**, on the right the **Sector**.
- **Save** writes the image, **Save as ...** saves a copy under a new name. **Add files ...** puts files on the disk (not on a disk without a file system). Cmd/Ctrl+S does not save in the disk editor. Unsaved changes ("Image not saved") are lost when you close; **Close disk image** asks first.
- **Directory:** the files with names (PETSCII on Commodore, ATASCII on Atari), type, blocks, size and start. A lock shows a protected file, **Autostart** the start file.
- **Check** checks the BAM or VTOC, the directory and the file chains; the result appears in the **Check report**.
- **Allocation map:** one row per track, one box per sector. The colors mean Selected file, Used, Directory and Header/BAM; sectors with errors are red. **Show chain** colors all sectors of the chosen file. A click or the arrow keys choose the sector.
- **Sector** (card on the right): shows the bytes of the chosen sector in hex and as characters; pointer bytes are highlighted. **Follow ▶** jumps to the next sector ("End of chain" at the end), **Back** goes back along the path.
- **Insert in emulator** starts the default emulator of the system with this disk (drive 8 on Commodore, D1: on Atari). The image must be saved beforehand; otherwise the button is disabled.

## Adding files to the project
With the check mark for each file (column **Mark**), you mark it; **All**, **None**, **Programs** and **BASIC** choose several at once. Then the box **Import** appears on the left. **Import into project** writes the files to the **Target folder in the project**; it must be inside the project. Options:
- **BASIC as source text:** the tokenized program is translated back into readable, numbered text.
- **Keep original file:** also the unchanged file.
- **Also as assembler:** machine programs as source text. This is purely mechanical, data appears as instructions and must be checked by hand.

## Formats
| Format | System | Layout |
|---|---|---|
| D64 | C64, 1541 | 35 tracks, 683 sectors |
| D71 | C64/C128, 1571 | 70 tracks (2 sides), 1366 sectors |
| D81 | C64/C128/MEGA65, 1581 | 80 tracks of 40 sectors, 3200 sectors |
| ATR | Atari, DOS 2 | 720 sectors of 128 bytes |

A project can have its image written when building: section `disk` of the project file (format, name, output file, programs), selectable as "Disk" in the dialog "New project".

## Command line
`vcc disk <format> -o output --add file[:NAME]` creates an image (`d64`, `d71`, `d81`, `atr [--boot prog.xex]`). `vcc disk list x.d64` shows the directory and checks the BAM, `vcc disk extract x.d64 NAME -o file` gets a file back. The images are cross-checked with `c1541`.
