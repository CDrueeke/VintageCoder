---
id: willkommen
titel: Welcome to VintageCoder
reihenfolge: 1
stufe: Beginner
zeit: 10 minutes
siehe_auch: [installation, erstes-projekt, oberflaeche]
referenz: [start, settings, roms, linux]
i18n_status: maschinell
i18n_quelle: 38fdcb623a14
---

# Welcome to VintageCoder

**Goal:** You know what VintageCoder is and for which computers you can write programs with it. After that you know how to continue: first install and set up, then build the first program.

## What is VintageCoder?

VintageCoder is a **development environment** (*IDE*) for computers from the 1980s whose heart is a processor of the **6502 family**: the Commodore 64, the Atari 800 XL and their relatives. In a single program you get:

- an **editor** with color highlighting, completion and help for source code,
- your own **assembler** (`vcasm`) for machine programs and a **BASIC translator** that turns BASIC without line numbers into a real program of the target system,
- the connection to **emulators** (programs that imitate an old computer on your machine),
- **editors** for drawing sprites, character sets, tiles and screens, for palettes, for music (tracker) and for disk images (Disk editor), plus a hex view and the memory map,
- a **debugger** that stops your program in the emulator and follows it step by step.

> **Info:** You find the editors in the main sections **Graphics**, **Music** and **Data**, the debugger in the section **Debug**.

> **Info:** A *source code* is the text file that you write. *Building* means creating from it the program that the old computer understands. *Running* means letting this program run in an emulator (or on real hardware). More terms are in the glossary (file `GLOSSAR.md`).

## For which computers can you program?

The user interface knows six target systems. You choose them in the dialog **Create new project**:

| Target system | Processor and chips | Emulator (type) |
|---|---|---|
| Commodore 64 (C64) | 6510, VIC-II (graphics), SID (sound) | VICE x64sc (cycle-exact) or x64 (fast) |
| Commodore 128 (C128) | 8502, VIC-IIe, VDC (80 columns) | VICE x128 |
| C16 / Plus/4 | 7501/8501, TED | VICE xplus4 |
| VIC-20 | 6502, VIC | VICE xvic |
| Atari 800 XL / XE | 6502C, ANTIC, GTIA, POKEY | Altirra (Windows only) or atari800 |
| MEGA65 | 45GS02, VIC-IV, 40 MHz | Xemu (xmega65) |

How you build a program and run it in the emulator is described in [Build and run](kapitel:bauen-starten). You enter real hardware (Ultimate 64 / II+, MEGA65) in the settings under **Targets and hardware**; depending on the package, the MEGA65 hardware is hidden there.

## PET

The Commodore PET/CBM is **not included** in this version: no entry in the dialog "Create new project", no run with F5, no examples in the packages. The manual therefore describes the systems in the table.

## Install and set up

How you install VintageCoder, set it up with the **setup assistant**, check the emulator detection and test it with F5 is described in the chapter [Installation and first-run setup](kapitel:installation). Two things in advance:

- VintageCoder runs your program in an **emulator** that you install yourself (to get started, VICE for the C64 is enough).
- VintageCoder does not include **ROM files** (operating system, BASIC, character set of the old computers). For your first program with VICE you do not need any.

The **start page** appears in the section **Code** as long as no project is open. It has the cards **New project**, **Open project** and **Open example**, the link **Choose project file (.vcprj) …**, the list **Recently opened projects**, the **Example projects** and, at the bottom, the box **Toolchain and hardware** with the **detected emulators** (ready or not found) and the **quick reference** of the most important keyboard shortcuts. At the first start, the card **Welcome to VintageCoder** with the button **Start setup** is in the place of the box; the **setup assistant** also opens by itself at the very first start.

![The start page with no project open: the title VintageCoder, the cards Neues Projekt, Projekt oeffnen, and Beispiel oeffnen, the link Projektdatei (.vcprj) waehlen, the section Zuletzt geoeffnete Projekte (still empty), and the sample projects with filters per system and cards with Oeffnen and Kopieren.](bild:willkommen-seite "The start page appears as long as no project is open.")

## The most important terms

| Term | Meaning |
|---|---|
| Target system | the old computer for which you program (C64, Atari, ...) |
| Project | a folder with a project file (`.vcprj`), your source codes and the built programs |
| Source code | the text file that you write (`.asm` for assembler, `.bas` for BASIC) |
| Build | create a program for the target system from source code (key F7) |
| Run | build and execute in the emulator or on hardware (key F5) |
| Emulator | a program that imitates an old computer |
| ROM | operating system, BASIC or character set of the old computer as a file |

The glossary explains more terms (file `GLOSSAR.md`).

## Keyboard shortcuts in this manual

Most shortcuts are the same on all systems, only one key has a different name: **Cmd** on the Mac corresponds to **Ctrl** on Windows. In the manual, for example, **Cmd/Ctrl+Shift+O** stands for "Open project". The function keys F5 and F7 are the same everywhere.

## Common pitfalls

- **You are looking for the setup of the emulator, ROMs or hardware.** It is in the chapter [Installation and first-run setup](kapitel:installation). You set up emulator, ROMs and hardware in the **Settings** (gear icon in the header bar or **Cmd/Ctrl+Comma**) or with the **setup assistant** (menu **Help**). Whether VintageCoder found your emulator, you see on the start page in the box **Toolchain and hardware**.

## What next?

In the next chapter you create your [first project](kapitel:erstes-projekt), build it and run it in the emulator. If you want to look around the user interface first, read [The interface at a glance](kapitel:oberflaeche).
