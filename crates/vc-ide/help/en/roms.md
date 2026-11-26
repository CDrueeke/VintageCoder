# ROM files

VintageCoder does **not include any original ROMs** (KERNAL, BASIC, character sets, operating systems). If you need them, put your **own images** in the ROM folder. VintageCoder recognizes them by their **content** (size and checksum). The file name does not matter. It loads nothing from the network and links to no ROM collection.


## What are ROMs for?
- **Emulators:** VICE gets `-kernal`, `-basic`, `-chargen` and so on, atari800 gets `-xlxe_rom`, `-osb_rom`, `-basic_rom`, Xemu (MEGA65) gets `-rom`. Without a ROM, VICE and atari800 use their own built-in replacement ROMs. **Xemu needs a MEGA65 ROM file of exactly 131072 bytes.** Altirra manages its firmware itself.
- **Character sets:** The character set editor reads the font of the system from your ROM. If it is missing, VintageCoder's **own standard set** (VC Standard, CC0, can be passed on) appears with a note. In the dialog **Create new project**, you choose under **Character set of the project** whether it is added to the project as a copy (`vcc new --charset`).

## Where to put them
The easiest place is the page **Settings > ROMs**; the step **ROMs** in the setup assistant shows the same page (**Setup assistant: ROMs only ...** in the menu **Help** and in the command palette, see [Setup assistant](help:wizard)). Alternatively, you put the files in the ROM folder by hand (see the search order below) or on the command line: `vcc roms list|check|path|where|import`. `vcc roms where` shows where VintageCoder searches.

![Settings, ROMs page. The list of pages is on the left. At the top right are the buttons ROM-Ordner oeffnen and Neu pruefen, below them ROM-Datei oder Zip waehlen, ROMs anderer Programme suchen, and Fundliste kopieren. Tabs for each system (C64 selected) and for each role the status Fehlt with the expected size. The searched places are at the bottom.](bild:einstellungen-roms "The ROMs page shows for each system and role whether a file was found; here everything is still missing.")

The page has these parts:

- **Open ROM folder** shows the folder, **Check again** reads it again.
- **Use ROMs of other programs (VICE, Xemu, atari800)** (default: off): VintageCoder then also reads the ROM folders of these programs. It only reads; nothing is copied or changed.
- **Importing ROMs:** **Choose ROM file or zip ...** and **Search ROMs of other programs** show a list of what was recognized, with system, role and the action ("will be copied" or "already present"). Each line has a check mark; only **Import n** copies the ticked files into the ROM folder. **Copy list of finds** puts the list on the clipboard. Files can also be dropped on the window. Nothing is copied without confirmation, and ROMs are never downloaded.
- **Systems:** one tab for each system with a traffic light (green: everything needed was recognized, yellow: replacement or unknown revision, red: a required role is missing). Below it, each role (for example KERNAL, BASIC, character set) shows the status (*Recognized*, *Unknown revision*, *Replacement (free)*, *Missing*, *Not usable*), the file name and, if found, the place, revision, size, CRC-32 and SHA-1. If a file is missing, the expected size in bytes is shown. If there are several files of the same role, you pin one (**Automatic** or a file name); the choice goes into `roms.json`.
- **Places searched:** all places of the search order with the number of files.

The page belongs to the draft of the settings: **Use ROMs of other programs** applies only after **Save**.

Search order (the first place found for each role is used):
1. Setting for a single file (for example atari800 "OS ROM", Xemu "Folder with MEGA65.ROM")
2. `roms/<system>/` in the **project** (it is in `.gitignore`; only for this project)
3. `roms/<system>/` **next to the program** (portable mode, see below)
4. **User folder:** Windows `%APPDATA%\vintagecoder\roms`, macOS `~/.config/vintagecoder/roms`, Linux `$XDG_CONFIG_HOME/vintagecoder/roms` (otherwise `~/.config/vintagecoder/roms`)
5. old location `~/.config/vintagecoder/mega65/MEGA65.ROM`
6. only on request (**"Use ROMs of other programs"**, default off): ROM folders of VICE, Xemu and atari800, read only
7. supplied free replacement ROMs (currently none)

`VC_ROMS` replaces 3 to 5 with a single folder. `vcc roms import` copies recognized files to the ROM folder.

## Folders and roles
`roms/<system>/` with `c64`, `c128`, `vic20`, `plus4`, `atari8`, `mega65`. File names are free. These are common:

| System | Roles (size in bytes) |
|---|---|
| C64 | `kernal` (8192), `basic` (8192), `chargen` (4096); optional drives `dos1541` and others |
| C128 | `kernal`, `basiclo`, `basichi` (16384 each), `chargen` (8192); optional `kernal64`, `basic64` |
| VIC-20 | `kernal`, `basic` (8192 each), `chargen` (4096) |
| Plus/4, C16 | `kernal`, `basic` (16384 each); optional `functionlo`, `functionhi` |
| Atari 8-bit | `os-xl` (16384), `os-b`, `os-a` (10240 each), `basic` (8192) - one operating system is enough |
| MEGA65 | `rom` (131072) |

Several files of the same role are allowed. The default revision wins, and a file can be **pinned** (`roms.json`; on the page **ROMs** you choose it for each role). A file with a matching name and size but an unknown checksum counts as an **Unknown revision** and is used with a warning. Subfolders (one level) are allowed.

## Portable mode
If a folder `roms/` (or a file `portable`) is next to the program, it applies first, and new ROMs are stored there if it is writable. "Next to the program" means on Windows and Linux the folder of the executable file (AppImage: next to the `.AppImage`), on macOS the folder that contains `VintageCoder.app`. **Never inside the app bundle:** Writing there would invalidate the signature. If the app still runs from its download folder ("app translocation"), the mode is not available. Move it to "Applications" once. `vcc roms where` shows everything.

## Character sets as a reference
A character set file can contain a **reference** instead of the characters: `"base": "rom:c64/chargen#upper"` (format 2) plus only your own changes under `overrides`. This way, the original characters are neither in the project nor in the repository. References: `rom:<system>/<role>[@<revision>]#<set>`, for example `rom:c64/chargen#lower`, `rom:atari8/os#standard`, `rom:plus4/kernal#upper` (Plus/4: `upper` and `lower`; `standard` still counts as `upper`). With `@<revision>` (for example `@906143-02`), exactly this image is needed. Old files (format 1) remain readable.

## Is something missing?
- *"ROM ... not found"* at start: put the file in the ROM folder (`vcc roms where` names it). The message names the target folder, the expected size and all places that were searched.
- The character set looks "wrong": probably the own standard set instead of your ROM (card **Origin** in the character set editor). Put the ROM in place.
- Unknown revision: works, but symbols and offsets can differ.

## The emulator quits immediately
If the emulator quits with an error shortly after the start (for example a VICE installed through a package manager without ROM files), VintageCoder writes the last lines of its output (such as "Couldn't load kernal ROM") to the **Output** (tab in the tray below the editor) and gives the hint: missing ROM files, see this page. An emulator that you close yourself does not trigger a message.
