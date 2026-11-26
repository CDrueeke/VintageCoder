# Tracker

Songs for **SID** (C64, 3 voices) or **POKEY** (Atari, 4 voices). One cell: `C-4 01` = note + instrument (hex), `---` nothing, `===` note off.

## Input
The keys apply as long as the grid has the focus.

| Key | Effect |
|---|---|
| Z S X D C V G B H N J M | notes of the lower row (current octave); on QWERTZ: Y S X D C V G B H N J M |
| Q 2 W 3 E R 5 T 6 Y 7 U I | notes of the upper row (octave + 1); on QWERTZ: Q 2 W 3 E R 5 T 6 Z 7 U I |
| Del / Backspace | clear the cell |
| `1` | note off |
| `[` `]` | choose instrument |
| `,` `.` | octave down / up |
| Alt+Up Down | change the instrument of the cell (or of the selection) |
| Arrow keys, Page Up/Down, Home/End | move (with Shift: select) |
| Space | play / stop the song (with Shift: the pattern only) |
| Cmd/Ctrl+C, X, V, A | copy, cut, paste, select all |
| Cmd/Ctrl+1, 2, 3 | focus on grid, instruments, order |

## Layout
The editor has three areas side by side (in narrow windows they move below each other).

![The Musik tab with the file tune.vcsong: on the left the order list (pattern 1, 2, 1, 2) and the instruments Puls, Bass and Snare, in the middle the toolbar with follow, tempo, rows, octave and step and the grid with three voices, on the right the cards for instrument 00 Puls, keyboard and volume. The cursor is on row 02, and the status line says unsaved.](bild:tracker-ueberblick "The tracker with order list, grid with three voices and instruments.")

- **Left** is the song: name with **New pattern** (plus), the **Order** of the patterns (**Append**, **Set**, **Remove**, arrows to move, **Repeat from entry**; ↻ marks the repeat), **Duplicate** and **Delete** for the chosen pattern, and the list of **Instruments** (**New instrument**, **Duplicate**, **Delete**: only the last, unused one). If the project has several songs, the **Songs** follow.
- **In the middle** is the toolbar with Undo, Redo, **Play song**, **Pattern ▶**, **Stop playback**, **Follow playback** and the fields **Tempo** (frames per row), **Rows**, **Octave** and **Step**, plus **Semitone ↑**, **Semitone ↓**, **Apply instrument**, **Convert ...**, **Import RMT module ...** (POKEY only), **Export ...** and **Save**. Below are the choice of **Voices**, the patterns as buttons and the grid. Each voice has **M** (mute), **S** (solo) and a level bar in its header.
- **Right** is the instrument with its controls, see [Envelope](help:envelope), below it the **Keyboard** and the **Volume**.

The dialog **New file** does not yet create new songs in this version.

**Convert ...** brings a song from another system into the project: you choose a file, also from another project folder. It is converted for the chip of this tracker (SID <-> POKEY) and created as a new file. Possible sources are a song, a **GoatTracker file** (`*.sng`, GoatTracker 2) or a **SID file** (`*.sid`).
  - *GoatTracker:* The song structure (order lists with repeat and transposition), notes, note off, instrument numbers, ADSR, first waveform and pulse width, and the tempo (command `F`) are kept. The three voices are put on a common time axis and cut into patterns of equal length. Wave, pulse and filter tables, vibrato and pattern commands (portamento, filter, ...) are not kept. Notes about this appear in the output. Only the first subtune is loaded. A GoatTracker song in the POKEY tracker is converted from SID to POKEY.
  - *SID file (`.sid`, PSID/RSID):* A SID file contains a player program, not patterns. So the program runs in a built-in 6502 core, and the state of the SID registers is recorded for each frame. From this, notes (from gate and pitch) and instruments (from waveform, ADSR and pulse width) are created. The row speed is determined from the note starts. In the dialog you choose the **subtune** and the **length in seconds** (default 30; the recording ends there, even if the piece continues). The result is an approximation of the sound: vibrato, portamento, filter, ring modulation and synchronization are dropped, arpeggios appear as single notes, and the repeat point is not detected. Only the first SID chip is recorded. RSID files that expect a real C64 can fail.
  - On the command line: `vcc media import-song <source> --name <name> [--profile <id>] [--song <n>] [--seconds 30]` (`--song` and `--seconds` apply to `.sid`).

**Import RMT module ...** (POKEY only) loads a Raster Music Tracker module.

## Output
- SID: `vcc media export` -> tables; player routine `examples/c64/music_demo/lib/music.inc`.
- POKEY: export as an **RMT module**; play it with the original RMT player (`examples/atari/music_demo/lib/rmtplayr.asm`, macros `rmt.inc`). C-3 is the lowest note.

## Tempo
The **Tempo** (also "speed") is the number of **frames per row**. This is important in your own program: call the player routine once per **frame** (raster interrupt). The KERNAL interrupt of the C64 runs at 60 Hz even on a PAL machine and plays too fast. So the example `examples/c64/music_demo/music_demo.asm` uses a raster IRQ.

## Displays: level and keyboard
During playback, the tracker shows what the built-in synthesizer produces (not the real chip):
- In the header of each voice, a bar shows the **level**.
- **M** (mute) and **S** (solo) also affect the sound. The buttons of the **Voices** above the grid hide a voice in the grid (this changes only the view: the voice keeps playing if it is not muted, and the file does not change); **Show all** brings all of them back. One voice always stays visible. With many voices (for example 12 on the MEGA65), the grid has a horizontal scroll bar. The channel headers and row numbers stay in place, and the cursor stays visible.
- The current row is marked in the grid. **Follow playback** makes the grid scroll along.
- The **Keyboard** shows the sounding notes. A click previews the note with the chosen instrument, but does not enter anything.
You hear changes in the grid at once, even while the song is playing.

The keys apply by their **position** on the keyboard: The letters are those of the US QWERTY keyboard. On a German keyboard (QWERTZ), the first key of the lower row is **Y** (instead of Z), and in the upper row there is **Z** instead of Y. The hints in the tracker show the letters of your layout (Settings, Keyboard layout).
