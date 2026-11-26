---
id: systeme
titel: Target systems compared
reihenfolge: 19
stufe: Beginner
zeit: 20 minutes
siehe_auch: [willkommen, erstes-projekt, debugger, tests, grafik-ueberblick]
referenz: [overview, c128-plus4, vic20, mega65, mega65-hardware, roms, settings]
i18n_status: maschinell
i18n_quelle: dd3a81cdd519
---

# Target systems compared

**Goal:** You learn what sets the individual computers apart and can choose the system you want to program for. For each system you find an example project to try out.

A **target system** is the computer your program is built for. You choose it in the dialog **Create new project** (see [Your first project](kapitel:erstes-projekt)), and it is stored in the project file. The header shows it in the **Run target** button together with the run target. With `.target` in the source you set it for one file (for example `.target c64`).

## What you need

- VintageCoder with emulators set up (see [Welcome to VintageCoder](kapitel:willkommen)). For some systems you also need ROM files (see below and the help page "ROM files").
- A bit of curiosity: all systems here have a 6502 processor or a close relative, so many things are similar.

## The systems at a glance

| System | Identifier | Processor | Graphics | Sound | Emulator in the IDE | Debugger |
|---|---|---|---|---|---|---|
| Commodore 64 | `c64` | 6510 | VIC-II | SID | VICE (x64sc, x64) | yes, several breakpoints |
| Commodore 128 | `c128` | 8502 (C128 mode) | VIC-IIe, VDC | SID | VICE (x128) | yes |
| Plus/4 | `plus4` | 7501/8501 | TED | TED | VICE (xplus4) | yes |
| C16 / C116 | `c16` | 7501 | TED | TED | VICE (xplus4, model c16) | yes |
| VIC-20 | `vic20` | 6502 | VIC-I | VIC-I | VICE (xvic) | yes |
| Atari 800 XL / XE | `atari8` | 6502C (Sally) | ANTIC, GTIA | POKEY | atari800, Altirra | atari800: one breakpoint |
| MEGA65 | `mega65` | 45GS02 | VIC-IV | four SIDs | Xemu | one breakpoint |

For all of them: build with **F7**, run the start program with **F5**, debug it with **F6**. Which emulator starts is set in the settings under **Targets and hardware** (section **Emulators**). A click on the **Run target** button in the header chooses another device instead of the default emulator, for example an Ultimate 64.

### Clock, lines, cycles

If you write raster programs, you need these values (per standard, from the target data):

| System | Standard | Clock | Lines per frame | Cycles per line |
|---|---|---|---|---|
| C64 / C128 | PAL | 985 248 Hz | 312 | 63 |
| C64 / C128 | NTSC | 1 022 727 Hz | 263 | 65 |
| Plus/4 | PAL | 886 724 Hz | 312 | 57 |
| VIC-20 | PAL | 1 108 405 Hz | 312 | 71 |
| Atari 8 bit | PAL | 1 773 447 Hz | 312 | 114 |
| Atari 8 bit | NTSC | 1 789 773 Hz | 262 | 114 |
| MEGA65 | PAL | 40 500 000 Hz | 625 | 810 |

## Commodore 64 (`c64`)

The classic. Programs start with a small BASIC stub (`.basicstub`) at `$0801`.

- **Memory:** 64 KB RAM. RAM from `$0000` to `$9FFF`, BASIC ROM at `$A000`, free area `$C000` to `$CFFF`, I/O at `$D000` to `$DFFF`, KERNAL ROM at `$E000`. The screen is at `$0400` by default. Memory switching is done in the processor port `$0001`; the Memory map shows presets for this (for example "All RAM").
- **Graphics:** VIC-II with 8 sprites, text mode and bitmap. In the IDE there are the sprite, character set, screen and tile editors for this.
- **Sound:** SID, in the tracker with three voices.
- **Emulator:** VICE, either **x64sc** (cycle exact) or **x64** (fast).
- **Debugger:** full range (several breakpoints, data breakpoints, emulator picture, snapshot).
- **Tests:** cycle measurement and the default trap address exist only here so far; mailbox `$033C`.
- **Real hardware:** The **Ultimate 64 / II+** can be connected over the network (run, read memory, mailbox tests, cycle measurement on the Ultimate 64).
- **Example:** `examples/c64/border` (border color), `examples/c64/sprite_demo` (sprites), `examples/c64/rasterbars` (raster bars), `examples/c64/prallball`.

*[Bild folgt in E13: system-vergleich-c64]*

## Commodore 128 (`c128`)

In this IDE, the C128 starts in **C128 mode** with CPU 8502 (the same instruction set as the 6510). If you want C64 mode, use the target system `c64`. The Z80 is not available.

- **BASIC:** BASIC 7.0 (dialect `basic7`); programs start at `$1C01`. `.basicstub` creates `SYS 7181` at `.org $1c01`.
- **Chips:** VIC-IIe (with `$D02F` keyboard and `$D030` 2 MHz mode), SID, two CIAs, the memory controller MMU (`$D500`, `$FF00`) and the **VDC** for the 80-column screen (`$D600`, `$D601`).
- **Graphics and sound:** The C64 profiles (sprites, character sets, screens, tiles, tracker) are also available in the C128 project. VDC graphics and an MMU helper in the assembler do not exist yet.
- **Emulator:** VICE **x128**; the debugger talks to its binary monitor. In the area **Debug** (only in a debug session), you choose at the picture of the emulator, under **Picture source**, between **VIC-II** (40 columns) and **VDC** (80 columns).
- **Tests:** mailbox at `$0B00`.
- **Disks:** D64, D71 and D81.
- **Example:** `examples/c128/border`, `examples/c128/prallball`, `examples/c128/prallball_vdc`.

## C16 and Plus/4 (`c16`, `plus4`)

- **BASIC:** BASIC 3.5 (dialect `basic35`); programs start at `$1001`. `.basicstub` creates `SYS 4109` at `.org $1001`.
- **Chip:** The **TED** does picture and sound in one. It has **no sprites**, and there is no SID.
- **Graphics and sound:** There are no editor profiles for sprites, character sets or songs of this system yet. So you write your program directly against the TED registers.
- **Memory:** color RAM at `$0800`, screen at `$0C00`. The **C16/C116** (target `c16`) has the same hardware, but only 16 KB RAM (user memory up to `$3FFF`). When creating a new project, choose "Commodore 16 / 116", then F5 starts the emulator as a C16 (`-model c16`). A C16 program also runs on the Plus/4.
- **Emulator:** VICE **xplus4**.
- **Example:** `examples/plus4/border`, `examples/plus4/prallball`, for the C16 `examples/c16/border`.

## VIC-20 (`vic20`)

In Germany, the VIC-20 was called "VC-20".

- **Chips:** VIC-I (picture and sound, border and background in `$900F`), two VIAs.
- **BASIC:** BASIC V2.
- **Memory expansion:** This is the special feature of this system. The expansion is stored in the project file; you choose it in the dialog **Create new project** (field **Memory expansion**) or by hand in the project file. BASIC start, screen and color RAM depend on it:

| Expansion | BASIC from | Screen | Color RAM |
|---|---|---|---|
| Basic machine (5 KB) | `$1001` | `$1E00` | `$9600` |
| with 3 KB | `$0401` | `$1E00` | `$9600` |
| 8, 16, 24 or 35 KB | `$1201` | `$1000` | `$9400` |

  So write your program with `.org BASIC_START` instead of a fixed address, so that it adapts to the expansion. The system file `#include <vic20/hardware.asm>` provides `SCREEN_RAM`, `COLOR_RAM` and `BASIC_START`.
- **Graphics and sound:** editors for character sets (hires, multicolor), the screen (22x23 characters) and tiles (11x11). The tracker has four voices. The chip has only **one shared volume**; envelopes therefore only switch the voices on or off.
- **Cartridges:** `.cartridge "a0", "NAME"` creates a raw 8 KB image (only without banks).
- **Emulator:** VICE **xvic**, with the matching `-memory` argument.
- **Example:** `examples/vic20/prallball`, `examples/vic20/tiles_demo`, `examples/vic20/music_demo`.

## Atari 800 XL / XE (`atari8`)

The Atari is **not a Commodore**: chips, memory and program format are different, only the processor is related.

- **Chips:** **ANTIC** (display lists) and **GTIA** (colors, players/missiles) for the picture, **POKEY** for sound, **PIA** for memory switching.
- **Graphics:** The ANTIC works with a **display list**; the snapshot in the debugger reads it and the color shadow registers (only ANTIC mode 2 and no bitmap). In the IDE there is an editor for players/missiles (PMG), for character sets, screens and tiles. In the Atari editors you choose the video standard PAL or NTSC on the right.
- **Sound:** POKEY, in the tracker.
- **Program format:** output as **XEX** (default), ATR (disk image) or BIN. Unlike on the C64, there is no BASIC stub: you set the start point with `.run start`, for example `.org $2000` ... `.run start`.
- **BASIC:** Atari BASIC or Turbo BASIC XL. The files are created in the SAVE format, which you load with `LOAD` or `RUN`. Turbo BASIC XL starts from a disk that you provide yourself (not included); VintageCoder writes your program as `AUTORUN.BAS` into a copy of it. See [Start Turbo BASIC XL](kapitel:basic).
- **Models:** 800 XL (64 KB), 130 XE (128 KB with bank switching through PORTB `$D301`) and 800 (48 KB, different operating system, no BASIC ROM).
- **Memory:** page 6 (`$0600` to `$06FF`) is free for users; in the operating system `$00` to `$7F` take the zero page (BASIC and OS), `$80` to `$FF` are free without BASIC. On the Atari 800 XL/XE there are bank presets in the Memory map.
- **Operating system:** For character sets from the ROM you need the operating system ROM (see help page "ROM files").
- **Emulator and debugger:** **atari800** or **Altirra**. For debugging, use atari800; it knows **only one** breakpoint at a time, no data breakpoints and cannot be paused with a button (**F8 in the emulator window**). The section **Screen** in the area **Debug** shows no picture for the Atari ("This emulator does not provide a picture.").
- **Tests:** mailbox at `$0600`.
- **Example:** `examples/atari/colorbars` (raster bars with `WSYNC`), `examples/atari/pmg_ship`, `examples/atari/prallball`, `examples/atari/music_demo`.

## MEGA65 (`mega65`)

A modern rebuild in the C65/C64 style with 40 MHz.

- **CPU:** 45GS02, an extension of the 65CE02 with 32-bit pointers and the Q register. New instructions are `ldz`, `phw`, `lbra`, `ldq` and more.
- **BASIC:** BASIC 65; programs start at `$2001` (`.basicstub` at `.org $2001`).
- **Graphics:** **VIC-IV** with text up to 640 pixels wide, full color characters with 256 colors and wide sprites. **Important:** The extended registers are **locked** after power-on. You unlock them at the start of the program with the macro `mega65_enable_vic4` (writes `$47` and `$53` to `$D02F`).
- **Sound:** **Four SIDs** (`$D400`, `$D420`, `$D440`, `$D460`), in the tracker 12 voices. In addition there is the **PLAY tracker** for BASIC programs with `PLAY` strings (see [PLAY tracker](kapitel:play-tracker)).
- **Palettes:** 256 colors with 8 bits per channel (see [Palettes](kapitel:paletten)).
- **Format:** PRG, or as a disk image **D81**.
- **Emulator:** **Xemu** (`xmega65`). For it you need a MEGA65 ROM file in the ROM folder (Settings > **ROMs**); the name does not matter. Starting takes about 15 seconds.
- **Debugger:** registers, memory, pause/continue and single step are possible, but only **one** breakpoint per session. Xemu stops at the breakpoint only *after* the instruction; the program counter is then on the following instruction. Data breakpoints exist only as slow watching by single step. The section **Screen** in the area **Debug** shows no picture for the MEGA65.
- **Real hardware:** The connection by USB or Ethernet is built in, but not yet tested on a real device.
- **Example:** `examples/mega65/hello`, `examples/mega65/pal_demo`, `examples/mega65/play_demo`, `examples/mega65/music_demo`.

> **Caution:** In this chapter, the MEGA65 has the most stumbling blocks: ROM file needed, VIC-IV must be unlocked, only one breakpoint. It is best to start with `examples/mega65/hello`.

## PET/CBM

The **Commodore PET / CBM** is **not included** in this version: no entry in the dialog **Create new project**, no emulator setting, no start with F5 and no debugger. Only the systems you can choose are described here.

## Try an example for each system

1. Open an example project from the folder `examples/` through the start page or **File > Open project ...** (**Cmd/Ctrl+Shift+O**), for example `c64/border`.
2. Press **F7** (Build). In the tab **Output** of the tray below the editor you see the result, for example `border.asm -> build/border.prg (36 Bytes)`.
3. Press **F5** to run the program in the emulator, or **F6** to debug it.
4. Repeat this with `c128/border`, `plus4/border`, `atari/colorbars`, `mega65/hello` and `vic20/tiles_demo`. These examples build without errors.

## Common stumbling blocks

| Problem | Cause and solution |
|---|---|
| The emulator does not start | Check the path: Settings > **Targets and hardware**, section **Emulators**. If the field stays empty, the IDE searches by itself. On Linux, see the section "Emulators on Linux" below. |
| For the MEGA65, the IDE reports a missing ROM | Put the ROM file into the ROM folder (Settings > **ROMs**, help page "ROM files"). |
| The VIC-IV register does nothing | The VIC-IV is locked: call `mega65_enable_vic4` at the start of the program. |
| The VIC-20 program only runs on the basic machine | A fixed address (`.org $1001`) applies only there. Use `.org BASIC_START`. |
| Atari program does not start | `.run start` is missing. |
| On the Plus/4 there is no sprite editor | The TED has no sprites, and there are no profiles for them yet. |
| Debugger on the Atari does not stop | atari800 cannot be paused with a button: set a breakpoint or press **F8** in the emulator window. |
| I have a program for the C64 and want to start it on the C128 | Change `.target` to `c128` (C128 mode; BASIC and start address differ) or stay with `c64`. |

## Emulators on Linux

VintageCoder **never** ships an emulator. You install it yourself: from the package sources of your distribution, from the maker or, where there is a fixed package, with the button **Install portable …** (see below). VintageCoder then looks for the emulator in the `PATH` and in some usual folders (`/usr/games`, `/usr/local/games`, `/usr/local/bin`, `/snap/bin`, `/opt/vice/bin`, `/opt/xemu`, `~/.local/bin`, `~/bin`). If it is somewhere else, you enter the path by hand.

> **Check the status:** The information about the package sources comes from a query of the default sources in fresh containers (Debian 13, Ubuntu 24.04, Fedora 45, openSUSE Tumbleweed). How to switch the sources on was not tried on a real installation in every case. Everything that is marked **unchecked** below is not confirmed. Distribution states change with every version: if in doubt, look it up with the package manager.

Which emulator you need:

| Target system | Emulator | Program name VintageCoder looks for |
|---|---|---|
| C64 | VICE | `x64sc` (cycle exact) or `x64` (fast) |
| C128 | VICE | `x128` |
| Plus/4, C16 | VICE | `xplus4` |
| VIC-20 | VICE | `xvic` |
| Atari 800 XL/XE | atari800 (Altirra exists only for Windows) | `atari800` |
| MEGA65 | Xemu | `xmega65` (also `xemu-xmega65`, `xmega65.native`, `xemu-mega65`) |

> **Info:** For the **debugger**, VICE needs **version 3.9 or newer**. Whether simply starting with **F5** also works with older versions has not been checked. After the installation, check the version (`x64sc --version`).

### What the default sources offer

| Distribution | VICE | atari800 | Xemu (MEGA65) | Fujisan |
|---|---|---|---|---|
| **Debian 13** | 3.9, only in `contrib` | 5.2.0, only in `contrib` | no package | no package |
| **Ubuntu 24.04** | 3.7.1, in `multiverse` | 5.0.0, in `multiverse` | no package | no package |
| **Fedora 45** | not in the Fedora sources | 7.1.2, repository `fedora` | no package | no package |
| **openSUSE Tumbleweed** | not in oss, non-oss, update | not in oss, non-oss, update | no package | no package |

- **Debian 13:** VICE and atari800 are only there once you switch on the package source section `contrib`.
- **Ubuntu 24.04:** VICE and atari800 are only there once you switch on the section `multiverse`. VICE there is **3.7.1**, so **older than 3.9**: according to this manual, that is too old for the debugger of VintageCoder. For debugging you need a newer VICE (build it yourself or get it from the VICE site).
- **Fedora 45:** atari800 is there without extra sources, VICE is not.
- **openSUSE Tumbleweed:** Neither VICE nor atari800 is in the default sources.
- **Xemu** and **Fujisan** (a second Atari emulator that the debugger prefers for `.xex` programs) are not available as a package in any of the four distributions. The entry `xemu` in the openSUSE sources is an **Xbox emulator**, **not** the one for the MEGA65: do not install it for that.

### Debian 13

**Change to the package sources.** `contrib` is a section of the official Debian package sources with free software that depends on software outside the main archive. Switching it on changes the configuration of your system: afterwards **all** packages from `contrib` can be installed, not only VICE and atari800. Do this only if you want that. Make a backup copy of the file first.

1. Find the file with your package sources. On a new Debian 13 installation it is usually `/etc/apt/sources.list.d/debian.sources`; older or converted systems have `/etc/apt/sources.list` instead.

   ```bash
   ls /etc/apt/sources.list.d/
   grep -n -i -E "^(Components|deb )" /etc/apt/sources.list.d/*.sources /etc/apt/sources.list
   ```

2. Back up the file and open it with an editor (here `debian.sources` as an example):

   ```bash
   sudo cp /etc/apt/sources.list.d/debian.sources /etc/apt/sources.list.d/debian.sources.bak
   sudo nano /etc/apt/sources.list.d/debian.sources
   ```

3. Find the line `Components: main` and add `contrib`, so that it says `Components: main contrib`. In a classic `sources.list`, you append `contrib` after `main`, for example `deb http://deb.debian.org/debian trixie main contrib`.

4. Read the package list again and install:

   ```bash
   sudo apt update
   sudo apt install vice atari800
   ```

   You can also install only one of the two.

5. Check what arrived:

   ```bash
   apt policy vice atari800
   x64sc --version
   command -v atari800
   ```

   For `atari800`, the program is usually under `/usr/games`; VintageCoder looks there too.

To undo it, remove `contrib` from the file again (or restore the `.bak` file) and run `sudo apt update`. Installed packages stay; you can remove them with `sudo apt remove vice atari800`.

### Ubuntu 24.04

**Change to the package sources.** `multiverse` is a section of the official Ubuntu package sources for free software for which Canonical does not promise security updates (maintenance is up to the community). Switching it on changes the package sources for **all** packages from `multiverse`. Do this only if you want that.

1. Check whether `multiverse` is already on: `apt policy | grep -i multiverse`. If there is output, it is active, and you can go to step 3.
2. Switch on `multiverse`:

   ```bash
   sudo add-apt-repository multiverse
   sudo apt update
   ```

   If the command is missing, install it with `sudo apt install software-properties-common`. You can also switch the section on in "Software & Updates".
3. Install and check:

   ```bash
   sudo apt install vice atari800
   apt policy vice atari800
   x64sc --version
   ```

   You see VICE **3.7.1** and atari800 **5.0.0**.

> **Caution:** VICE 3.7.1 is older than the version 3.9 that the debugger needs. You can try starting with F5 (**Test** in the emulator card). For the debugger you need a newer VICE. Whether atari800 5.0.0 works with the debugger is **unchecked**.

To undo it: `sudo add-apt-repository --remove multiverse`, then `sudo apt update`. You remove installed packages with `sudo apt remove vice atari800`. **Unchecked:** the exact steps on fresh and on updated systems.

### Fedora 45

atari800 is in the default sources (repository `fedora`, version 7.1.2). For this you change nothing in the package sources:

```bash
sudo dnf install atari800
dnf info atari800
command -v atari800
```

**VICE** is not in the Fedora sources. One possible source is the third-party repository **RPM Fusion**. **Unchecked:** whether VICE is there, in which version and under which name. RPM Fusion is **not** part of Fedora; adding it is a change to the package sources. Follow the current instructions of the project (rpmfusion.org).

### openSUSE Tumbleweed

In the default sources (oss, non-oss, update) there is **neither VICE nor atari800**. Check whether anything has changed for you:

```bash
zypper search vice atari800
```

If the package is missing, the section "If the package is missing or too old" applies. **Unchecked:** whether an additional repository (for example a community repository) has packages. Adding an additional source is a change to the package sources; first check who owns it and how current it is. If the package is reachable, you install it with `sudo zypper install vice atari800`.

### If the package is missing or too old

Then you get the emulator from the maker.

| Emulator | Where | Note |
|---|---|---|
| **VICE** | vice-emu.sourceforge.io, section "Download" | For Linux, the source code is there to build yourself. Whether the site offers ready Linux packages, you check there. VintageCoder does not download VICE by itself on Linux. |
| **atari800** | github.com/atari800/atari800/releases | In the VintageCoder catalog, for Linux (x86_64) there is an **AppImage** file (7.2.1 and 7.1.2). This is a single executable file. Whether the site still offers it in this form, you check there. |
| **Xemu (MEGA65)** | github.com/lgblgblgb/xemu/releases and github.com/lgblgblgb/xemu/wiki | The project only publishes pre-releases. For Linux, the catalog names a `.deb` package (`xemu_<date>_amd64.deb`) or building yourself. On Debian and Ubuntu you install it with `sudo apt install ./xemu_<date>_amd64.deb`; on Fedora and openSUSE the `.deb` does not help. The program is called `xmega65`. You also need a MEGA65 ROM file. |
| **Fujisan** | github.com/pedgarcia/fujisan/releases | Optional, only for the Atari debugger. For Linux, the catalog names archives (`.tar.gz`) for x86_64 and aarch64. |
| **Altirra** | Windows only | on Linux use atari800 or Fujisan |

**Install portable.** On the page **Settings > Targets and hardware**, the emulator card shows a button **Install portable (version) …** where there is a fixed package. After a question ("Download now"), VintageCoder downloads the package from the project site, checks the SHA-256 sum and stores it in a folder of its own below the configuration folder. It changes nothing outside this folder, and it downloads nothing without your click. On Linux, according to the catalog, this offer exists for **atari800** (AppImage, it is only saved and made executable) and for **Fujisan** (archive, is unpacked). For VICE and Xemu, the card shows a note with the download page or the way through the package manager instead.

**Flatpak and Snap.** **Unchecked:** whether VICE, atari800, Xemu or Fujisan exist as Flatpak or Snap and whether VintageCoder copes with them. Flatpak programs are named by their identifier and are not found automatically: you would have to enter the path of a script that runs, for example, `flatpak run <identifier> "$@"`. Snap programs are under `/snap/bin`; VintageCoder looks there.

### Then set up in VintageCoder

There are two ways, both change the same thing in the end.

**Way 1: Setup assistant.** At the first start it opens by itself; otherwise you start it on the page **Settings > General** in the section **Setup** (**Quick setup (7 steps)** or **Set everything myself (12 steps)**, also just **Emulators …**) or with **Help > Setup assistant: emulators only ...**. In the step **Toolchain**, the list shows "ready" or "not found" for each emulator. The assistant installs nothing: you install missing emulators yourself (see above) or enter them under **Settings > Targets and hardware**; there is also the portable installation there.

**Way 2: By hand in the settings.**

1. Open the settings: **View > Settings** or **Ctrl+Comma**.
2. Switch to the page **Targets and hardware** and choose your system on the left in the list **Target systems**.
3. For the emulator, a line shows whether it was found ("Found: ..." or "Not found. Enter the path by hand."). You can leave the field **Program (empty = search automatically)** empty: then VintageCoder searches by itself. **Search** enters the path that was found. A path of your own also works.
4. If the emulator is missing from the list, choose under **Add emulator …** the kind that exists for the system.
5. The selection field **Default** makes it the emulator for **F5** and **F6** (unless you chose another run target).
6. For the MEGA65, put the ROM file in the ROM folder (**Settings > ROMs**; on Linux `~/.config/vintagecoder/roms` if `XDG_CONFIG_HOME` is not set). Without a MEGA65 ROM, Xemu does not start.
7. Click **Save**.

### Check the setup

1. **Where it was found:** The emulator card says "Found: ...".
2. **Test:** **Test** in the emulator card tries out the emulator; the message below it shows whether it worked.
3. **Hello World:** Open an example project, press **F7** (Build) and **F5** (Run).
4. **In the terminal (optional):** `command -v x64sc`, `command -v atari800`, `command -v xmega65`. Empty output means: not in the `PATH`. On Debian and Ubuntu, `atari800` is usually in `/usr/games`; VintageCoder also looks there, in the terminal the folder is not always in the `PATH`.

### If something does not work

| Problem | Cause and remedy |
|---|---|
| `E: Unable to locate package vice` (Debian, Ubuntu) | `contrib` or `multiverse` is not switched on, or `sudo apt update` is missing after the change. |
| "Unable to find a match" (`dnf`) or "No match" (`zypper`) | The package is not in your sources (see table above). Continue with "If the package is missing or too old". |
| "Not found" although installed | Check the program in the `PATH` (`command -v ...`), otherwise enter the path in the emulator card. |
| Debugger does not start or reports a version that is too old | VICE is older than 3.9 (for example Ubuntu 24.04 with 3.7.1). Get a newer VICE from the maker. |
| Xemu does not start | MEGA65 ROM (exactly 131072 bytes) is missing: **Settings > ROMs**. |
| You installed the package `xemu` from openSUSE | That is an Xbox emulator. Remove it (`sudo zypper remove xemu`) and get Xemu for the MEGA65 from the maker. |

> **Check the status (open points):** program names after the installation (`x64sc`, `x128`, `xplus4`, `xvic` in the `PATH`); whether starting and the debugger work with VICE 3.7.1 and atari800 5.0.0 (Ubuntu), VICE 3.9 and atari800 5.2.0 (Debian) and atari800 7.1.2 (Fedora); whether the maker files named in the catalog are still offered like this; Flatpak and Snap; additional sources for Fedora and openSUSE.

## What next?

- Your first project on another system: [Your first project](kapitel:erstes-projekt).
- The graphics and sound tools: [Graphics and sound at a glance](kapitel:grafik-ueberblick).
- For looking things up: [C128 and Plus/4](hilfe:c128-plus4), [VIC-20](hilfe:vic20), [MEGA65](hilfe:mega65), [MEGA65 on the device](hilfe:mega65-hardware), [ROM files](hilfe:roms).
