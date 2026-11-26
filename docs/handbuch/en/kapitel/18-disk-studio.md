---
id: disk-studio
titel: Disk editor, segments, ROMs
reihenfolge: 18
stufe: Advanced
zeit: 30 minutes
siehe_auch: [bauen-starten, assembler, debugger]
referenz: [disks, cartridges, roms, memmap, asm-directives]
i18n_status: maschinell
i18n_quelle: 1f04b836eded
---

# Disk editor, segments, ROMs

**Goal:** You open disk images, fetch files from them into your project and have your program written to a disk. You organize the memory of your program with **segments** and you know where VintageCoder looks for the **ROM files** of your systems.

This chapter is **advanced**: It assumes that you already build assembler programs ([Assembler for beginners](kapitel:assembler)) and know what memory addresses look like.

## What you need

- a disk image to look at (`.d64`, `.d71`, `.d81` or `.atr`) or a new, empty disk that you create in the disk editor,
- an open project if you want to import files from the image into the project (for looking, checking and creating, the disk editor alone is enough),
- for the example, the project `examples/c64/prallball`.

## What is the editor for?

The disk editor shows and edits **disk images**. You need it when your program should load from disk, when you want to fetch files from an old archive or when you want to check whether your image is in order.

Typical uses from real retro projects:

- **Deliver a game on disk:** main program, levels, music and title picture as files on a D64.
- **Look at an old archive:** What is on the disk, and what of it can I import into my project (BASIC as readable text)?
- **Look for errors:** Why does the program not load? The allocation map and **Check** show broken chains.
- **Look at a boot disk** for Atari without a file system.

In the finished program, this is an **image** (`.d64`, `.d71`, `.d81`, `.atr`) that an emulator or a disk replacement (SD card) inserts like a real disk.

## Basic terms without previous knowledge

- **Disk image:** a file that contains the content of a disk sector by sector.
- **Track and sector:** rings and slices of a disk; a sector usually has 256 bytes.
- **BAM / VTOC:** the allocation table (**B**lock **A**vailability **M**ap at Commodore, VTOC at Atari): which sectors are free.
- **Directory:** the list of files on the disk.
- **Chain:** a file is a sequence of sectors, each pointing to the next.
- **PRG / SEQ:** file types at Commodore (program / text sequence).
- **Interleave:** the distance at which consecutive sectors of a file lie on the disk.
- **Segment, bank:** sections of the program memory (part 2).

## Part 1: The disk editor

In the past, programs came on **disks**. Today there are **disk images** for this, files that contain the content of a disk sector by sector. Emulators and many devices read them. The disk editor knows four formats:

| Format | Computer/drive | Layout |
|---|---|---|
| D64 | C64, drive 1541 | 35 tracks, 683 sectors |
| D71 | C64/C128, drive 1571 | 70 tracks (two sides), 1366 sectors |
| D81 | C64/C128/MEGA65, drive 1581 | 80 tracks of 40 sectors, 3200 sectors |
| ATR | Atari with DOS 2 | 720 sectors of 128 bytes |

> **Info:** A disk is divided into **tracks** (rings) and **sectors** (slices, usually 256 bytes). A file is a **chain** of sectors: each sector tells where the next one is. An overview table, at Commodore the **BAM** (Block Availability Map), at Atari the **VTOC**, remembers which sectors are in use.

### Open an image

The disk editor lives in the main section **Data**, in the second row under **Disk**. As long as no disk is open, it says "No disk image open" with the buttons **Open disk image …** and **New disk …**. You open an image in these ways:

- **Click the file** (`.d64`, `.d71`, `.d81`, `.atr`) in the file tree: the disk editor opens by itself,
- the button **Open disk image …** in the disk editor (you choose any image file),
- drag the image file from Finder or Explorer onto the window,
- **Tree > Open disk image ...** (Mac menu; otherwise through the command palette **Cmd/Ctrl+K**).

Only **one** disk is open at a time. If you open a second one, the first is closed; if it has unsaved changes, the disk editor asks first: **Save**, **Discard** or **Cancel**.

![The area 'Daten' with the page 'Disk' and the disk image beispiel.d64. On the left are the cards 'DISKETTE', 'BELEGUNG' and 'AKTIONEN'. In the middle are the directory with the three files HUGO, GUMBO and MUSIK and the allocation map. On the right is the sector view 'SEKTOR 18/0' as a hex dump.](bild:disk-studio "The disk editor with directory, allocation map and sector view.")

### What you see in the editor

The disk editor has three cards side by side. In narrow windows, the right card moves down.

**Left: Disk.**

- the format (D64, D71, D81 or ATR; the current one is highlighted),
- the file name of the image and its size; a dot in front of it means "unsaved"; below it is the layout (tracks and sectors),
- the **Usage** as a bar with the number of used and free blocks,
- the **Actions** (see below),
- if needed, the **Import** field (target folder and options) and below it the **Check report** and the result of the import.

**Middle: Directory and allocation map.**

- **Directory:** all files with name (at Commodore in **PETSCII**, at Atari in **ATASCII**, the character sets of these computers), type, blocks, size and start address. A lock shows a protected file, **Autostart** the file that starts by itself when loading. A click (or the arrow keys and Enter) chooses a file.
- **Changing the order:** The order in the directory decides in which sequence a loader or a directory listing shows the files. Drag a row to another place; a line shows where it lands. With the keyboard, choose the row and press **Alt+Arrow up** or **Alt+Arrow down**; the buttons **Move up** and **Move down** above the list do the same. Every move is one step for **Undo** (Cmd/Ctrl+Z, also the button next to the list) and **Redo**. Only the directory entries move, the files themselves stay unchanged. On Atari this works for Atari DOS 2 and 2.5 (128-byte sectors) and only in the main directory; for other images the page tells you why it does not work. **Adding files** can be undone as well. The change is in the file only after **Save**.
- **Allocation map:** one row per track, one square per sector; large disks (from 41 tracks) are shown in two blocks side by side. The legend names four colors: **Selected file**, **Used**, **Directory** and **Header/BAM**. Sectors with an error (for example reached by several chains) are red. With **Show chain**, the map colors all sectors of the chosen file. A click chooses the sector; with the arrow keys you move over the map.

**Right: the sector.**

- The header names track and sector. Below it, it says which file the sector belongs to ("Block 2 of 5").
- Below that you see the bytes of the sector in hex and as characters. Pointer bytes (the link to the next sector) are highlighted in color.
- **Follow ▶** jumps to the next sector of the chain; at the end it says "End of chain". **Back** goes the way back.
- At the bottom are **Next sector** and **Bytes in this sector**.

### The buttons

All actions are on the left in the **Disk** card:

| Action | Effect |
|---|---|
| **Insert in emulator** | starts the default emulator of the system with this disk (drive 8). The image must be saved first; otherwise the button is locked ("Save first, then insert"). |
| **Add files …** | chooses files and puts them on the disk. For a disk without a file system (boot disk), the button is locked. |
| **Check** | checks BAM or VTOC, directory and file chains and writes the result into the **Check report**. |
| **Import into project** | fetches the marked files into the project (see below); the number of marks is shown behind it. |
| **Save** | writes the image (only active when there are changes). |
| **Save as …** | saves a copy under a new name. |
| **Open disk image …** | opens another image. |
| **New disk …** | creates a new, empty disk. |
| **Close disk image** | closes the image; if there are changes, the question appears. |

**Cmd/Ctrl+S** does not save in the disk editor; use the button **Save**.

New empty disk: **New disk …**, in the dialog enter the **Format** (D64, D71, D81 or ATR), the **Disk name** (up to 16 characters) and the **ID (2 characters)**, then **Create**. ATR has neither name nor ID. The new disk stays "unsaved" until you store it with **Save**.

*[Bild folgt in E13: disk-neu]*

**Unsaved changes are lost when you close**; the editor shows "Image not saved" and asks first.

### Check a disk

Click **Check**. In the field **Check report** it then says either "No problems found." or one line per finding. If something is conspicuous, the **Disk** card also shows the number of problems.

*[Bild folgt in E13: disk-validieren]*

### Import files into the project

You have an old disk (for example from an archive) and want to fetch programs from it into your project. For this, a project must be open, otherwise the button is locked ("A project must be open to import.").

1. In the column **Mark**, put a **check mark** at each file you want to import. The buttons **All**, **None**, **Programs** and **BASIC** above the directory choose several at once.
2. As soon as something is marked, the field **Import** appears on the left. Enter the **Target folder in the project**. It must be inside the project, otherwise "The target folder must be inside the project." appears.
3. Options:
   - **BASIC as source text:** a BASIC program (on disk in a short form, the **tokens**) is translated back into readable, numbered text.
   - **Keep original file:** also the unchanged file.
   - **Also as assembler:** machine programs as source text. This is purely mechanical, data appears as instructions and must be checked by hand.
4. Click **Import into project**. What was written is then shown in the list **Import**; the file tree reads itself again.

*[Bild folgt in E13: disk-editor-uebernahme]*

### Have a disk written when building

A project can have its image written when building. The project file then has a section `disk`; in the "New project" dialog you choose a format for this under **Disk**. The example project `examples/c64/prallball` has, for example:

```json
"disk": {
  "format": "d64"
}
```

In the **Project** area, the overview shows in the line for the disk whether one is set up (for example "Disk D64, 1 programs").

1. Open `examples/c64/prallball` (start page > **Open example**), build with **F7**. Next to the program, `build/disk.d64` is created; the log names the disk.
2. Open `build/disk.d64`: click the file in the file tree. In the directory there is `PRALLBALL`.
3. Click **Check**: "No problems found." should appear.
4. With **Insert in emulator** you start the emulator with the disk.

The same works on the command line: `vcc disk list build/disk.d64` shows the directory, `vcc disk extract build/disk.d64 PRALLBALL -o datei.prg` fetches a file back.

## Part 2: Segments and banks

Your program consists of blocks of bytes at fixed addresses. Each block is called a **segment**: every `.org` starts a new one. With `.segment` you give a block a **name** and an **allowed range**. If a byte leaves this range, the build stops with the message **E301**. This way you catch errors early such as "code runs into my data".

```asm
.target c64
.cpu 6510 { illegals = deny }

.segment "CODE", $0801, $0fff       ; program: must stay between $0801 and $0FFF
.org $0801
.basicstub start
start:  lda #5
        sta $d020
        jmp *

.segment "DATEN", $c000, $c0ff      ; table in its own range
.org $c000
tabelle: .byte 1,2,3,4
```

After a build, the machine panel on the right lists the segments as a table under **Last build**: number, start address (**from**), end address (**to**) and **length**. Next to it are source, program, size and the start address (SYS). You show and hide the panel with **Cmd/Ctrl+Alt+B**. You see the address space as a grid in the **Memory map** (main section **Data**, second row **Memory**): 256 pages of 256 bytes each, with the program from the last build drawn in, with a **Filter** (All, Program, RAM, ROM, I/O) and, where the system knows it, **Bank switching**. The map exists only after a build (**F7**). The details are in the reference [Memory Map](hilfe:memmap).

*[Bild folgt in E13: segmente-baenke]*

> **Caution:** In the C64 PRG format, everything becomes **one** block from the lowest address; the assembler fills gaps with zero bytes. The example above therefore results in a file of about 47 KB, although there is only a few bytes of content. In the Atari XEX, each segment stays a block of its own.

### Cartridges and banks

A **cartridge** is a plug-in module with read-only memory (ROM): the computer sees its content at once, and the program starts at power-on. Cartridges are often larger than the address space that the computer provides for them (on the C64 8 KB at `$8000`). The rest is in further **banks**; at run time the program switches a bank on, and its content appears at the same address.

- `.cartridge "typ", "NAME"` turns the program into a cartridge image (`.crt` on the C64, `.car` on the Atari).
- `.bank n` chooses the bank (0 to 1023). Each bank has its own program counter. Banks exist only in cartridge formats.
- The example `examples/c64/cart_demo` has two 8 KB banks and switches between them in RAM. With **F7**, `build/cart_demo.crt` is created.

All directives, cartridge types and examples are in the reference [Segments, banks, cartridges](hilfe:cartridges).

## Part 3: ROM files

A **ROM** (*Read-Only Memory*) is a read-only memory that contains an operating system, BASIC or a character set. Emulators need the **images** of these ROMs to reproduce the real computer.

> **Caution:** VintageCoder does **not ship original ROMs** and does not download any from the network. You put your **own** images in place.

VintageCoder recognizes ROM files by their **content** (size and checksum); the file name does not matter.

### Set up ROMs

The easiest way is the assistant: **Help > Setup assistant: ROMs only ...** guides you through the setup. You can also put the files yourself into one of the folders named in the next section. The page **Settings > ROMs** does the same work (see below).

### Where does VintageCoder look?

The first place found for each role applies:

1. setting for a single file (for example atari800 "OS ROM"),
2. `roms/<system>/` in the **project** (it is in `.gitignore`),
3. `roms/<system>/` **next to the program** (portable mode),
4. the **user folder**: Windows `%APPDATA%\vintagecoder\roms`, macOS `~/.config/vintagecoder/roms`, Linux `$XDG_CONFIG_HOME/vintagecoder/roms` (otherwise `~/.config/vintagecoder/roms`),
5. the old location `~/.config/vintagecoder/mega65/MEGA65.ROM`,
6. only on request (switch **Use ROMs of other programs (VICE, Xemu, atari800)**, default off): ROM folders of VICE, Xemu and atari800, read only.

The subfolders are called `c64`, `c128`, `vic20`, `plus4`, `atari8` and `mega65`. On the page **Settings > ROMs** you see for each system and role whether a matching file was found; **Choose ROM file or zip …** copies files into the ROM folder after your confirmation, **Open ROM folder** shows it, and at the bottom are the **Places searched**. Usual files are:

| System | Files (size in bytes) |
|---|---|
| C64 | `kernal` (8192), `basic` (8192), `chargen` (4096) |
| Atari 8 bit | one operating system is enough: `os-xl` (16384), `os-b` or `os-a` (10240 each); plus `basic` (8192) |
| MEGA65 | `rom` (131072) |

> **Info:** **Xemu** (the MEGA65 emulator) needs a ROM file of **exactly 131072 bytes**. VICE and atari800 use their own replacement ROMs if none is there.

If you have several images of the same role, the default revision wins. A file with a matching name and size but an unknown checksum counts as **Unknown revision** and is used with a warning.

From the command line there are `vcc roms list`, `check`, `path`, `where` and `import`.

## My first disk image (using `prallball`)

1. **Open the example:** start page > **Open example** > `examples/c64/prallball`. *Expected:* The project is open; a D64 disk is entered in the project file.
2. **Build:** **F7**. *Expected:* The log reports that the disk was written.
3. **Open the image:** click the built `.d64` file (under `build/`) in the file tree. *Expected:* The area Data > Disk shows the directory and the allocation map.
4. **Look at a file:** choose a file in the directory. *Expected:* The allocation map colors its sectors (with **Show chain**), the card on the right shows the bytes of the sector.
5. **Check.** *Expected:* The check report says "No problems found."
6. **Insert in emulator.** *Expected:* The emulator starts with the disk.

## Hardware features and limits

- **D64** has only 683 sectors (about 170 KB); more does not fit. **D81** holds 800 KB.
- File names on Commodore disks are at most **16 characters** long and are in PETSCII; on Atari **8+3** in ATASCII.
- A **boot disk** (Atari) has no files, only boot sectors.
- On a real drive, the **interleave** decides the load time.

## Neighboring editors and glossary

- The program on the disk is created with the [Assembler](kapitel:assembler) and the editors for graphics and sound; the data comes from their exports.
- The machine panel shows after a build where your program lies in memory, see part 2.
- Terms: Glossary (file `GLOSSAR.md`): disk image, sector, BAM, chain.

## Common stumbling blocks

- **"ROM ... not found" at start:** Put the file in place (**Settings > ROMs** or **Help > Setup assistant: ROMs only ...**). The message names the target folder, expected size and all places searched.
- **A character set looks "wrong":** Usually VintageCoder shows its own default set because the ROM is missing. Put the ROM in place and open the view again.
- **E301 when building:** A segment exceeds its range or overlaps another one. The log names segment and addresses.
- **"Insert in emulator" is gray:** The image is not saved (or has no location yet). First **Save**, then insert.
- **The disk in the emulator is empty:** The saved image contained no files. Check the directory in the disk editor.
- **A file cannot be imported:** The target folder must be inside the project (no absolute paths, no `..`).
- **"Add files ..." is gray:** The disk has no file system (boot disk).

## What next?

[Target systems compared](kapitel:systeme) deals with the differences between systems. For looking things up: [Disk editor](hilfe:disks), [Segments, banks, cartridges](hilfe:cartridges), [ROM files](hilfe:roms), [Memory Map](hilfe:memmap). For finding errors in the program itself, there is the debugger, see [The debugger](kapitel:debugger).
