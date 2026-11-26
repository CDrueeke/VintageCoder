# Project overview

You can see the state of your project in several places: in the area **Project**, in the project tree, in the machine panel **Machine**, in the memory map (**Data > Memory**) and in the tray below the editor.

## Page "Overview" in the area Project
The main area **Project** (Cmd/Ctrl+Alt+0) has the pages **Overview**, **General**, **Targets**, **Sources**, **Build and run**, **Disk** and **Share** in the second row. The page **Overview** shows four cards, each with a dot for the state: green "OK", yellow "Note" (for example no programs), red "Missing" (for example the name) or gray "Not switched on".

- **General:** the project name and the start program (without a choice, the first program).
- **Targets:** the target systems of the project.
- **Sources:** the number of programs and source groups; missing files are counted.
- **Disk:** off, or the format and the number of programs (without a selection: all programs).

**Change ›** jumps to the matching page. If there are errors or warnings in the project file, they appear below in the box **Problems in the project file**.

## Project tree
The sidebar **Project** (area Code) shows all files with a type tag (see [File tree](help:filetree)). A button with a magnifier filters the files by name.

## Memory
The right panel **Machine** (Cmd/Ctrl+Alt+B) shows under **Last build**, after building, where your program sits in memory: source, program, size, start (SYS) and a table of the segments (for the exact layout see [Memory map](help:memmap)). Before that, it says "Not built yet. Build with F7." During a debug session, the panel also shows registers, flags and, where the emulator supplies it, the picture of the machine. You find the **graphical memory map** in the main area **Data** under **Memory**; it shows the address space as a grid of 256 pages and the program from the last build in it (see [Memory map](help:memmap)).

## Last build
The tray below the editor (Cmd/Ctrl+J) shows in the tab **Output** what happened during building and running, and in the tab **Problems** the errors and warnings with file and line. The status bar names the result ("built: ... bytes" or "Build: ... errors"). See [Build and run](help:build).

## Target and connection
- The button **Run target** in the header bar (for example "C64 · VICE x64sc") shows where F5 runs. A click opens the choice.
- On the start page, the box **Toolchain and hardware** shows which emulators were found ("ready" or "not found").
- The general choice of emulator and run target is in the settings, see [Settings](help:settings). The page **Targets** in the area Project shows the target systems of the project with their variant (for example PAL or NTSC).

## Editors
You create new files with the button **New file** above the project tree (type Assembler, BASIC, Text or Note). You edit sprites, character sets, tiles, screens and palettes in the area **Graphics**, songs in the area **Music** (tracker) and disks in the area **Data** (New disk in the disk editor). Debugging has the area **Debug**. See [Sprites, character sets, songs](help:graphics) and [Disk editor](help:disks).

**Run** (F5) and **Build** (F7) are at the top in the header bar, see [Build and run](help:build).
