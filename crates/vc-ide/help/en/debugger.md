# Debugger

**F6** builds and runs the start program in the emulator, stopped at the entry point, and opens the debug session in the main section **Debug** (**Alt+Shift+F6** debugs the active file). Without a session, the section shows the card "No debug session" with the button **Start debugging**. Which emulator or machine starts is decided by the run target from the [Settings](help:settings). All debug commands are in the menu **Project > Debug** and in the command palette (**Cmd/Ctrl+K**). During a session, the status bar shows "Debug: halted at $PC"; a click takes you back to the Debug section.

## What the adapter can do
Not every adapter provides everything. The debug line shows the state ("running", "Stopped at $0810 · file:line"); if an adapter cannot do a step at the moment, the reason is shown next to it.

| Adapter | Registers | Step | Breakpoints | Data breakpoints |
|---|---|---|---|---|
| VICE (C64, C128, Plus/4, VIC-20) | yes | yes | several | yes (hardware checkpoints) |
| Fujisan (Atari 8-bit, TCP) | yes (also set, via a preamble) | yes (also "Over") | several | no |
| atari800 (text monitor) | yes | yes | only one at a time | no |
| MEGA65 (UART monitor, Xemu or hardware) | yes | yes | one | as a write watch by single-stepping (slow) |
| Ultimate 64 / II+ (REST) | no | no | no | no |

**Atari:** For `.xex` programs, the debugger takes **Fujisan** if it is found (setting `emulators.fujisan.path`, otherwise automatic search, including the version loaded with `vcc emulator install fujisan`), otherwise atari800. With `emulators.fujisan.debug_for_atari` = `atari800` or `fujisan` you can force a choice. Fujisan runs as a program of its own, is controlled through its TCP server (port 6502) and can be paused with a button; the adapter sets registers itself (preamble in RAM), data breakpoints and cycle counting do not exist, and Fujisan writes memory byte by byte (slow, about 50 bytes per second). Disk images and cartridges are still debugged by atari800 (**F8** in the emulator window to pause). More: [Command line](help:cli) (`vcc emulator`).

atari800 cannot be paused with a button (**F8** in the emulator window). With the Ultimate, reading and writing memory, pause and continue are possible.

## Operation
In the toolbar in the middle of the Debug section there are icon buttons; the mouse over one shows its name.

| Name (button, menu Project > Debug) | Key | Effect |
|---|---|---|
| Debug: Continue | Alt+F5 | runs to the next breakpoint (without a session, it starts one) |
| Debug: Pause | Alt+F6 | pauses (VICE, Ultimate; atari800: **F8** in the emulator window) |
| Debug: Step over | F10 | one instruction, `JSR` is skipped |
| Debug: Step into | F11 | one instruction, `JSR` jumps into the subroutine |
| Debug: Step out | Shift+F11 | runs to the return (`RTS`/`RTI`) of the current subroutine; calls inside it are skipped, otherwise single steps. After 10 s without a return, it stops with a message (then set a breakpoint at the return) |
| Stop debugging | Shift+F5 | ends the session |
| Debug: Run to here | none | runs to the line at the cursor in the editor (one-time breakpoint) |

Next to the buttons is the state ("Starting …", "Running", "Stopped at $0810 file:line", "Waiting for the emulator") and on the right the name of the emulator. If an emulator cannot do a step at the moment, the reason is shown below. A session that was created from the Tests panel with **Debug this test** carries the mark "Test session".

## The debug area
The Debug section has three columns. In narrow windows, they move below one another.

- **Left:** **Breakpoints** (see [Breakpoints](help:breakpoints)), **Watch**, **Snapshot** and **Chips**.
- **Middle:** toolbar, **Disassembly** and **Memory**.
- **Right:** **Registers**, **Stack** and **Screen**.

Details:

- **Registers** shows PC, A, X, Y, SP as hex values (changed ones are highlighted) and the flags N V B D I Z C as boxes (display only). Below are further values of the emulator, for VICE and atari800 for example the raster position. The **Stack** lists the bytes from the stack pointer on with their address.
- **Disassembly** from the PC with address, bytes, instruction and cycles. The round button in front of a line sets or deletes a breakpoint there; in the line of the PC, the link `file:line` leads to the editor.
- **Memory** (16 bytes per line, hex and as characters, changed bytes highlighted): enter an address or symbol, Enter or **Show**.
- **Watch:** expressions (registers, symbols, `[address]`) are evaluated again at every stop and shown in hexadecimal and decimal; ✕ removes one. Expressions are also possible before the session starts.
- **Chips:** a checkbox for each chip of the target system; a checked chip shows its registers with address, name and current value (mouse over: description). Without a chip description, it says "There is no chip description for this target system."

While the program is running, the columns say "The program is running. Stopping shows registers, disassembly and memory." Right-click menus and scrolling with a button in memory do not exist. In a session, the **Machine** panel also shows registers, flags and a small picture.

## Snapshot (screen, character set, sprites)

In the **Snapshot** section (left), the function reads the current screen, the character set and the active sprites from the stopped machine (C64, Atari 8-bit) and creates them under `graphics/snapshot/`.

- **Detect what is in memory** evaluates the hardware: C64 - VIC bank (`$DD00`), `$D018`, color RAM, sprite pointers; Atari - display list (`$0230`), `CHBAS` (`$02F4`) and color shadow registers. The result appears below: mode and screen address, character set, number of sprites.
- With the switches **Include sprites** and **Build tiles from the screen**, you choose what else is created. If the standard font is in ROM, **Fetch charset from ROM** is added.
- Then you enter a name and click **Save as files**. A snapshot works only while the program is stopped. This creates `<name>_screen.vcscreen`, `<name>_charset.vcchar`, `<name>_sprite0.vcsprite` ... and `<name>_tiles.vctiles`.
- C64 memory is read as RAM, also under ROM and I/O (VICE). Note: the standard fonts belong to the manufacturers - do not check saved ROM character sets into public repositories.
- Not supported: bitmap modes (no text screen) and ANTIC modes other than 2.

A preview and changing the detected addresses do not exist.

## Emulator picture (VICE)
The **Screen** section (right in the Debug section) shows the picture of the VICE emulator in VintageCoder: C64, C128 (VIC-II and VDC), VIC-20 and Plus/4. Atari and MEGA65 provide no picture ("This emulator does not provide a picture."). The **Machine** panel shows the picture smaller.
- **Refresh picture** gets the picture immediately through the monitor of the emulator. With the switch **Refresh at every stop**, VintageCoder gets it at every stop (breakpoint, single step, pause) itself; it is exactly the state at which the machine stands.
- **Live (slow motion):** VintageCoder stops the machine every few milliseconds, gets the picture and continues it. The machine runs at about 40 to 80 percent of its speed and without sound; this is not meant for playing.
- **Picture source:** On the C128, a selection list switches between VIC-II (40 columns) and VDC (80 columns).
- **Send text to the emulator:** The field writes text into the keyboard buffer of the emulator (Enter or **Send**), for example `RUN`. This is only text: keyboard and joystick are not passed on to the emulator.
- The picture has the full size of the VICE frame buffer (C64 504 x 312, C128 VIC-II 384 x 272, VDC 856 x 288, Plus/4 384 x 288, VIC-20 568 x 284); areas that are not drawn are black.

Saving the picture as PNG, copying it to the clipboard and setting it as a project picture do not exist.

## Hex view (Data > Hex)
The hex view shows binary files of the project read only. On the left, you choose a file under **Binary files** (in the file tree, **View as hex (read only)** also works). In the middle are address, bytes and text; you select with a click, Shift, dragging and the arrow keys. On the right, **Interpreted as** shows the byte (decimal, hex, binary), the word (low byte first) and the text, and for program files also the load address and the memory address of the selection. **Search** takes bytes (`$A9 00`) or text. **16 bytes** and **8 bytes** set the row width. At most the first 4 MiB are read, and 4 KB per page are shown; the buttons of the page selection turn the pages. **Open in the default program** hands the file to the operating system.
