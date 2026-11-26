# Glossary for beginners

Short explanations of the terms used in the manual. In alphabetical order of the German terms, in simple words.

## A

**Tray** (Ablage) - the area below the editor with the tabs Problems, Output, Tests, Search and Outline (Cmd/Ctrl+J). -> [The user interface at a glance](kapitel:oberflaeche)

**Address** (Adresse) - Each of the 65536 memory cells of an 8-bit computer has a number, the address. You usually write it in hexadecimal, for example `$d020`.

**Accumulator (A)** (Akkumulator) - the most important working register of the 6502 processor. Almost all calculations and copy operations go through A.

**Assembler** - a program that translates readable short codes like `lda #2` into machine code. The language itself is also called this. The assembler of VintageCoder is called **VCASM** (`vcasm`).

**Asset** - Everything that is not program text: sprite, character set, screen, tile set, palette, song. It lies in the project as a small text file. -> [Graphics and sound: overview](kapitel:grafik-ueberblick)

**ATASCII** - the character code of the Atari 8-bit computers, the counterpart to PETSCII on the Commodore.

**ATR** - file format for disk images of the Atari 8-bit computers. -> [Disk editor, segments, ROMs](kapitel:disk-studio)

**AUTORUN** (autostart file) - name of a file that the Atari (`AUTORUN.SYS`) or Turbo BASIC XL (`AUTORUN.BAS`) loads and runs by itself after starting a disk. VintageCoder writes your Turbo BASIC program to the disk as `AUTORUN.BAS`. -> [Run Turbo BASIC XL](kapitel:basic)

## B

**BAM / VTOC** - the management table of a disk that notes which sectors are used (Commodore: BAM, Atari: VTOC).

**BASIC stub** (BASIC-Stub) - a tiny BASIC line like `10 SYS 2061` at the start of a machine program. It calls the machine program so that you can start it with `RUN`. In the assembler, the directive `.basicstub` creates it.

**Build** (Bauen) - create the finished program for the target system from the source code (key F7). The technical terms "compile" or "assemble" mean the same.

**Command palette** (Befehlspalette) - a search window (Cmd/Ctrl+K) that finds commands, files, recently opened projects and the places of the navigation. -> [The user interface at a glance](kapitel:oberflaeche)

**Screen code** (Bildschirmcode) - the numbers that the screen memory of the C64 and related computers expects. They differ from PETSCII. In the assembler, `.screencode` turns text into screen codes.

**Byte** - a memory place for a number from 0 to 255 (eight bits).

## D

**Debugger** - a tool with which you stop a running program, follow it step by step and look at its registers and memory.

**Dialect** (BASIC) (Dialekt) - the form of BASIC that a computer has: BASIC V2 (C64, VIC-20), BASIC 7.0 (C128), BASIC 3.5 (Plus/4), BASIC 65 (MEGA65), Atari BASIC or Turbo BASIC XL.

**Directive** (Direktive) - an instruction to the assembler (or the BASIC translator), not to the processor. In the assembler it starts with a period (`.org`), in BASIC with `#` (`#include`).

**Disk image** (Diskettenabbild) (D64, D71, D81, ATR) - a file that holds the content of a disk sector by sector. -> [Disk editor, segments, ROMs](kapitel:disk-studio)

**DOS 2** - the file system of the Atari disks (Atari DOS 2, also the Turbo BASIC XL disk). VintageCoder reads and writes it in the ATR image.

## E

**Editor** - a tool in VintageCoder for one kind of file: Code editor, Sprite editor, Character set editor, Tile editor, Screen editor, Palette editor, Disk editor and Tracker. Each editor belongs to a main area. -> [Graphics and sound: overview](kapitel:grafik-ueberblick)

**Property** (Eigenschaft) (tile) - a freely named feature of a tile, for example "solid"; your program can query it. -> [Screen and tiles](kapitel:bildschirm-kacheln)

**Emulator** - a program that imitates an old computer on today's computer. VintageCoder starts your program in an emulator like VICE (Commodore), atari800 or Altirra (Atari) and Xemu (MEGA65).

## F

**Color RAM** (Farb-RAM) - the memory area that holds the color of each cell of the text screen on the C64 (from `$D800`). -> [Screen and tiles](kapitel:bildschirm-kacheln)

**Frame** - A single image of a sprite; several frames one after the other make an animation.

## H

**Main area** (Hauptbereich) - one of the six areas that stand next to each other in the header bar: Project, Code, Graphics, Music, Data and Debug. For Project, Graphics and Data there is a second row below with subareas. -> [The user interface at a glance](kapitel:oberflaeche)

**Hexadecimal** - a way of writing numbers to base 16 with the digits 0 to 9 and a to f. In source code a `$` stands in front: `$ff` is 255. You write binary with `%`: `%1010` is 10.

**Hires / Multicolor** - Two drawing kinds for character sets and sprites: *Hires* has fine dots with one color, *Multicolor* has double-wide dots, but four color values per dot. -> [Character set editor](kapitel:zeichensatz), [Sprite and PMG editor](kapitel:sprites)

## I

**IDE** - *Integrated Development Environment*: editor, translator, debugger and other tools in one program. VintageCoder is an IDE.

**Include** - putting another file into the source code. In the assembler with `.include "file.asm"` or `#include <path>` for the system library.

**Instrument / envelope (ADSR)** - the tone color of a sound; the envelope describes how the volume changes: Attack (rise), Decay (fall), Sustain (holding level), Release (fading out).

**Interleave** - the distance at which consecutive sectors of a file lie on the disk; it decides the loading time. -> [Disk editor](kapitel:disk-studio)

## K

**Tile / tile set / map** (Kachel / Kachelsatz / Karte) - A tile is a building block made of several characters (for example 2x2). A tile set collects tiles, a map (screen bank) says which tile stands where. -> [Screen and tile editor](kapitel:bildschirm-kacheln)

**KERNAL** - the operating system of the Commodore computers, fixed in ROM. It offers routines like `CHROUT` (print a character), which you call with `jsr`.

**Chain** (Kette) (disk) - a file consists of sectors; each sector tells where the next one lies. -> [Disk editor](kapitel:disk-studio)

**Comment** (Kommentar) - text in the source code that the translator skips. In the assembler it starts with `;` (or `//`), in BASIC at the start of a line with `;`.

**Setup assistant** (Konfigurationsassistent) - the guided dialog for setting up VintageCoder (systems, emulators, ROMs, hardware, design, project folder). It opens by itself on the first start, shows a list of changes before writing, and can be opened again from the **Help** menu. -> [Installation and first-time setup](kapitel:installation)

## L

**Label** - a name for an address in the assembler, with a colon at the end (`start:`). Local labels start with a period (`.loop:`).

**Listing** - the `.lst` file that shows the address and the generated bytes for each line of the source code.

## M

**Macro** (Makro) - a named short form for several lines. The assembler inserts the text of the macro at each call. A macro is not a subroutine.

**Label** (Marke) (BASIC) - a name for a line that you use instead of a line number: `@loop:` and `goto @loop`.

**Machine code** (Maschinencode) - the numbers that the processor runs. An assembler creates them from your source code.

**Cartridge / bank** (Modul / Bank) - a cartridge with ROM that starts when you switch on; banks are switchable parts of the cartridge that appear at the same address.

**Pattern / order** (Muster / Reihenfolge) - A pattern is a table with a fixed number of rows; the order decides in which sequence the patterns are played.

## O

**Onion Skin** - (English, is not translated) shows the previous frame as a pale shadow under the current one, so that you draw movements cleanly.

**Opcode** - the number that names a processor instruction. The listing shows it before the operand: in `a9 02`, `a9` is the opcode of `lda #`.

## P

**Palette** - a table that assigns colors to color numbers. The MEGA65 has 256 freely changeable entries. -> [Palettes](kapitel:paletten)

**Machine panel** (Panel Maschine) - the panel on the right in the main area Code: target, state and result of the last build (Cmd/Ctrl+Alt+B). -> [The user interface at a glance](kapitel:oberflaeche)

**PETSCII** - the character code of the Commodore computers (letters, graphic characters, control characters like "clear screen").

**Pixel value** (Pixelwert) - the "color" that a pixel carries as a number (for example 0 = transparent); which real color is behind it is decided by a register or the palette. -> [Sprite editor](kapitel:sprites)

**PLAY** - BASIC 65 command of the MEGA65 that plays a string of notes. -> [PLAY tracker](kapitel:play-tracker)

**PMG (Player/Missile Graphics)** - The sprites of the Atari: four players (8 dots wide) and four missiles. -> [Sprite and PMG editor](kapitel:sprites)

**Profile** (Profil) - decides which system and which kind of file a character set, sprite or screen is (for example `c64.char.hires`). -> [Character set editor](kapitel:zeichensatz)

**Project** (Projekt) - a folder with a project file (`.vcprj`), your source files and the built programs.

**Project area** (Projekt-Bereich) - the main area **Project** (Cmd/Ctrl+Alt+0) with the subareas Overview, General, Targets, Sources, Build and run, Disk and Share. Here you set what applies to the whole project. -> [Projects and files](kapitel:projekte)

## Q

**Source code** (Quelltext) - the text file that you write (`.asm` for assembler, `.bas` for BASIC).

## R

**Raster interrupt** (Raster-Interrupt) - an interruption that the graphics chip triggers at a certain screen line; ideal for calling a play routine exactly once per frame.

**Register** - a tiny memory cell in the processor (A, X, Y) or in a chip (for example the register of the border color at `$d020`).

**Leftover cell** (Restzelle) - a cell of the screen that does not fit into the tile grid. -> [Screen and tiles](kapitel:bildschirm-kacheln)

**ROM** - fixed built-in memory of a computer with operating system, BASIC and character set. VintageCoder does not ship original ROMs; you provide your own images.

## S

**Segment** - A connected block of generated bytes at a fixed address; `.segment` gives it a name and an allowed range (error E301 if violated).

**Sidebar** (Seitenleiste) - the column on the left in the main area Code with the file tree of the project (Cmd/Ctrl+B). -> [The user interface at a glance](kapitel:oberflaeche)

**Sector** (Sektor) - a piece of a disk (usually 256 bytes); track and sector give its position. -> [Disk editor](kapitel:disk-studio)

**SID / POKEY** - sound chips of the C64 (3 voices) and the Atari (4 voices).

**Sprite pointer** (Sprite-Zeiger) - A byte (on the C64 `$07F8`+n) that tells the graphics chip in which 64-byte block the data of a sprite lies (address divided by 64).

**Start program** (Startprogramm) - the program of a project that F5 starts. You set it in the project area (subareas General and Sources, where it carries a star ★).

**Control character** (Steuerzeichen) - a character that causes something instead of showing something, for example "clear screen". In source code you write them in curly braces: `{clr}`.

**Voice (channel)** (Stimme / Kanal) - a sound track that a sound chip can play at the same time as others. -> [Tracker](kapitel:tracker)

## T

**Speed** (Takt) (tracker) - the number of frames per row; it decides the tempo. -> [Tracker](kapitel:tracker)

**Target** - see *target system*. In the project file and in the source code (`.target c64`) stands the identifier of the target system.

**Tracker** - a music program in which notes stand in tables: columns are voices, rows are steps in time. -> [The Tracker](kapitel:tracker)

## V

**VC standard** (VC-Standard) - the own character sets of VintageCoder (free under CC0, may be passed on) that take the place of the original font without a ROM. -> [Character set editor](kapitel:zeichensatz)

**Video standard (PAL/NTSC)** (Videonorm) - a television standard with 50 or 60 frames per second; it decides the color look (Atari) and the tempo of music.

## Z

**Character set** (Zeichensatz) - the table that assigns a dot pattern (usually 8x8 dots) to each screen code. -> [Character set editor](kapitel:zeichensatz)

**Cell** (Zelle) - a place on the text screen (the size of one character); in the tracker an entry made of a note and an instrument. -> [Screen and tiles](kapitel:bildschirm-kacheln)

**Zen mode** (Zen-Modus) - hides the sidebar, tray, panel and status bar so that only the editor remains (Cmd/Ctrl+Alt+Z). -> [The user interface at a glance](kapitel:oberflaeche)

**Zero page** (Zeropage) - the first 256 bytes of memory (addresses `$00` to `$ff`). The processor reaches them with shorter and faster instructions.

**Target system** (Zielsystem) - the old computer you program for: C64, C128, C16/Plus/4, VIC-20, Atari 800 XL/XE or MEGA65.

**Target character set** (Zielzeichensatz) - The character set that is created at export from all used characters of all tiles.
