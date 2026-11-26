# VintageCoder: Overview

VintageCoder is a development environment for 6502 systems: **C64**, **C128**, **C16** and **Plus/4**, **VIC-20**, **Atari 8-bit** and **MEGA65**, with its own assembler, BASIC, emulator connection and editors for graphics and sound.

## Quick start
1. On the **start page** (area **Code**, as long as no project is open), open a project from the list **Recently opened projects**, choose **Open project** or **Open example** (list from the folder `examples/`), or open the dialog **Create new project** with **New project** (see [Start page, setup assistant and new project](help:wizard)). The project tree then shows what the project contains; the [project overview](help:overview) summarizes its state.
2. Open an `.asm` or `.bas` file in the project tree.
3. **F7** builds, **F5** builds and runs the start program on the run target (emulator, see [Settings](help:settings)). **F6** runs the program with a debug session in the area **Debug** (see [Debugger](help:debugger)).
4. **Cmd/Ctrl+K** opens the command palette: it finds all commands and files of the project.

## Where do I find what?
- Write source code: [Editor](help:editor), [Assembler](help:assembler), [BASIC](help:basic)
- Build and run: [Build and run](help:build)
- Find errors: [Debugger](help:debugger), [Breakpoints](help:breakpoints), [Memory Map](help:memmap), [Tests](help:tests)
- Graphics and sound: [Sprites, character sets and songs](help:graphics), [Export](help:export), [Tracker](help:tracker), [Envelope](help:envelope)
- Cartridges and disks: [Segments, banks, cartridges](help:cartridges), [Disk editor](help:disks)
- New system: [MEGA65](help:mega65)
- Report errors: [Send feedback](help:feedback)
- Look for a new version: [Check for updates](help:updates)
- More topics: [Path fields](help:paths), [Markdown](help:markdown), [Number tools](help:numbers), [BASIC to assembler](help:basic2asm), [Back up configuration](help:config)
- Operation: [Settings](help:settings), [Clock in the status bar](help:clock), [Project overview](help:overview), [Keyboard shortcuts](help:shortcuts), [Customize the user interface](help:layout), [File tree](help:filetree)

**Build project:** F7 builds all programs according to the project file, F5 runs the start program; with **Alt+Shift+F7** and **Alt+Shift+F5** the active file is used. You see and change sources, start program and target systems in the main area **Project** (Cmd/Ctrl+Alt+0).
