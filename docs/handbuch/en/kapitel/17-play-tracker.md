---
id: play-tracker
titel: PLAY tracker (MEGA65)
reihenfolge: 17
stufe: Beginner
zeit: 20 minutes
siehe_auch: [tracker, basic]
referenz: [play, envelope, mega65]
i18n_status: maschinell
i18n_quelle: b98d99f7beda
---

# PLAY tracker (MEGA65)

**Goal:** You write a piece of music for the MEGA65 in a table and let VintageCoder create a **BASIC program** with `PLAY` commands from it. You can play this program without your own player routine.

## What you need

- a **MEGA65 project** (emulator Xemu, see [Welcome to VintageCoder](kapitel:willkommen)),
- the chapter [The tracker](kapitel:tracker) (terms such as cell and envelope) is helpful, but not required.

## What is the editor for?

The PLAY tracker is for **quick music in the BASIC of the MEGA65**: You write a piece in a table, and VintageCoder turns it into a finished BASIC program. This suits title music in BASIC games, sound tests and learning projects where you do not want to write machine code.

Typical uses:

- **Title music** for a BASIC game on the MEGA65.
- **Effects** that you trigger with `PLAY` from your own BASIC program.
- **A sound experiment** that you want to listen to on the real device without writing a player routine.

In the finished program, this is a **text**: a BASIC program with `ENVELOPE` and `PLAY` lines.

## Basic terms, no prior knowledge needed

- **BASIC 65:** the BASIC dialect of the MEGA65. It has built-in music commands.
- **PLAY:** BASIC command that plays a string of notes.
- **Step:** one line in the grid; here a sixteenth note.
- **Voice:** one sound track (there are six).
- **Envelope (ENVELOPE 0 to 9):** the volume curve and waveform of a sound.
- **Tempo:** speed of the piece (1 to 255).

## How is the PLAY tracker different?

The MEGA65 has the command **`PLAY`** in its BASIC dialect (BASIC 65). You give it a string such as `"V1T0O4QCEG"` ("voice 1, envelope 0, octave 4, quarter notes C, E, G"), and the computer plays it without you programming interrupts. The **PLAY tracker** creates these strings from a grid.

The differences from the normal tracker:

- There are **no patterns and no order**. A PLAY song is a single grid of **steps x voices**.
- A **step** is a **sixteenth note**. A note sounds until a new note or `===` (rest) follows.
- The song has **6 voices** and 32 steps by default.
- Instead of instruments there are **envelopes 0 to 9**. They correspond to the BASIC command `ENVELOPE n,a,d,s,r,waveform,pw`.
- The result is **text** (a BASIC program), not machine code.

## My first PLAY song (using `play_demo`)

1. **Open the example:** On the start page, open `examples/mega65/play_demo` from the example projects (to make changes, use **Copy**). *Expected:* In the tree there is `music/tune_play.vcplay` (group "PLAY tracker").
2. **Open the song:** At the top, choose the main section **Music** and click `music/tune_play.vcplay`; a double click in the file tree works too. *Expected:* The editor looks like the normal tracker, with **six voices** and without an order of patterns.
3. **Listen:** **Play song** or the **Space** key (the cursor is in the grid). *Expected:* The piece plays, and the row moves along.
4. **Set a note:** Click a cell and press the key at the bottom left (German keyboard: **Y**). *Expected:* A note, and the cursor moves one row on.
5. **Save:** **Cmd/Ctrl+S**; then **Export …**, see below.

The **New file** dialog does not yet create a new PLAY song in this version. Start with the example project or an existing song.

## Layout of the user interface

As in the [Tracker](kapitel:tracker): on the left the song with the envelopes, in the middle the grid with the toolbar, on the right the settings of the chosen envelope with the keyboard and the volume. Instead of the order, the hint "PLAY files have one pattern and no order." is on the left; a pattern is played once. In the toolbar you set **Tempo** (1 to 255) and **Rows** (the number of steps), and the **Volume** (0 to 15) is on the right below the keyboard. You type notes as in the tracker, by key position: the bottom row `Y S X D C V G B H N J M` for one octave, the top row `Q 2 W 3 E R 5 T 6 Z 7 U I` (QWERTZ) for the next. **Del or Backspace** clear a cell, **1** sets a rest (`===`), **[** and **]** choose the envelope, **,** and **.** the octave, and the **Space** key plays.

*[Bild folgt in E13: play-tracker]*

### Editing envelopes

On the left, the list **Instruments** shows the envelopes 0 to 9. **New instrument** (at most ten for PLAY) creates another one, **Duplicate** copies the chosen one. For each one you set on the right the waveform (Triangle, Saw, Pulse or Noise), the pulse width and the four values **Attack**, **Decay**, **Sustain** and **Release**, just as for the SID in the tracker chapter. A click on a key of the **Keyboard** previews the tone with the chosen envelope.

> **Info:** An **envelope** describes the volume curve of a tone: how fast it swells (attack), how fast it falls to the held volume (decay, sustain) and how long it fades out after the end (release).

## Exporting

In the toolbar there is **Export …**. The dialog offers two formats: **BASIC program (PLAY)** and **BASIC DATA**. With **Save in project** you write the file `build/export/<name>.bas`. The result for the example piece looks like this:

```text
10 REM TUNE_PLAY - created by VintageCoder
20 VOL 8
30 TEMPO 8
40 ENVELOPE 0,0,9,8,4,2,2048
50 ENVELOPE 1,0,6,10,2,1,2048
60 ENVELOPE 2,0,5,0,3,3,2048
70 ENVELOPE 3,1,8,9,5,0,2048
80 PLAY "V1T0O4QCEGEO5CO4GEGDF#AF#O5DO4AF#A"
90 PLAY "V2T1O2QCCCCCCCCGGGGGGGG"
100 PLAY "V3T3O3HEEEEF#F#F#F#"
110 PLAY "V4T3O3HGGGGAAAA"
120 PLAY "V5T3O4HCCCCDDDD"
130 PLAY "V6QRT2O4HCCCCCCCQC"
```

This is how you read the `PLAY` strings:

| Character | Meaning |
|---|---|
| `V1` ... `V6` | voice |
| `T0` ... `T9` | envelope |
| `O4` | octave 4 |
| `W`, `H`, `Q`, `I`, `S` | note length: whole, half, quarter, eighth, sixteenth note |
| `.` | dotted (one and a half times as long) |
| `R` | rest |
| `C`, `E`, `F#` ... | notes |

Lengths that cannot be written as a single note (for example five steps) are written by the export as a **repeated attack**.

## Example to try yourself

The project `examples/mega65/play_demo` contains `music/tune_play.vcplay`: the same piece as the tracker example with four SIDs, converted to six `PLAY` voices (melody, bass, three arpeggio voices and snare, 64 steps, four envelopes).

1. Open the project and the file `music/tune_play.vcplay` (it is in the group "PLAY tracker" of the file tree).
2. Press the **Space** key to listen to it.
3. In the toolbar, click **Export …** and in the dialog click **Save in project**. This creates `build/export/tune_play.bas`.
4. So that the program also runs in the emulator, you add it to the project as a BASIC source. In the project file `play_demo.vcprj`, replace `"sources": []` with:

```json
"sources": [
  { "targets": ["mega65"], "kind": "basic", "files": ["build/export/tune_play.bas"] }
],
"main": "build/export/tune_play.bas"
```

5. Build with **F7** and run with **F5**. Xemu starts and plays the piece.

> **Caution:** The BASIC file must exist **before** you build (step 3). Otherwise the build reports "file missing". After every change to the song, you export again. On the command line, it works with `vcc media export music/tune_play.vcplay --format bas`.

## Hardware details and limits

- **Six voices** (MEGA65), no patterns and no order.
- At most **ten envelopes** (0 to 9).
- The result is BASIC **text**. The timing accuracy and the sound are decided by the BASIC interpreter of the MEGA65.
- The preview is an approximation. The generated BASIC text has not been tested on real hardware.

## From the editor to the program

1. **Export …** (toolbar or **Cmd/Ctrl+Shift+E**) creates `build/export/<name>.bas`.
2. Add the file to the project as a BASIC source (see the example above) or paste its lines into your own BASIC program.
3. Build with **F7**, run in the emulator **Xemu** with **F5**.

## Neighboring editors and glossary

- More sounds and machine code are offered by the [Tracker](kapitel:tracker).
- BASIC basics: [BASIC in the IDE](kapitel:basic).
- Terms: Glossary (file `GLOSSAR.md`): PLAY, envelope, BASIC 65.

## Common pitfalls

- **The sound in the editor is different from the MEGA65:** It is an approximation. The generated BASIC text has not been tested on real hardware.
- **A voice stays silent or plays wrongly:** Check whether the cell really contains a note and whether the envelope (`T`) exists.
- **The piece is too fast or too slow:** Change **Tempo** in the toolbar. It is written to the program as `TEMPO`.
- **The build reports "file missing":** The export is still missing (see the caution above).

## What next?

If music and program should come together in machine code, use the normal [Tracker](kapitel:tracker). Including BASIC programs is explained in [BASIC in the IDE](kapitel:basic). For reference: [PLAY tracker](hilfe:play), [Envelope](hilfe:envelope), [MEGA65](hilfe:mega65).
