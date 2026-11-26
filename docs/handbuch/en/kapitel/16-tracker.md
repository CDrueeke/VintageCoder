---
id: tracker
titel: The Tracker
reihenfolge: 16
stufe: Beginner
zeit: 40 minutes
siehe_auch: [play-tracker, grafik-ueberblick, systeme]
referenz: [tracker, envelope, export]
i18n_status: maschinell
i18n_quelle: 1113bb6dcaac
---

# The Tracker

**Goal:** You compose a short piece of music in the tracker, listen to it, export it and play it in a program.

## What you need

- a project for a system with a sound chip, here mainly **C64** (chip **SID**, 3 voices) or **Atari** (chip **POKEY**, 4 voices),
- speakers or headphones. Listening works inside VintageCoder itself, you do not need an emulator for it.

## What is the editor for?

In the tracker you compose the **music and sounds** of your program: title tune, game music, sound effects (shot, explosion, jump).

Typical uses from real retro projects:

- **A title tune** with three voices: bass, accompaniment, melody.
- **Game music in a loop** made of a few patterns that repeat.
- **Sound effects** as short patterns with noise or fast jumps in pitch.

In the finished program, the song ends up as a **data table plus a small player routine**. You call the routine once for every frame (50 or 60 times per second), and it controls the sound chip. The editor creates both.

## Basic terms without prior knowledge

- **Sound chip:** the part that makes sounds (C64: **SID**, three voices; Atari: **POKEY**, four voices; MEGA65: **PLAY** command of BASIC 65).
- **Voice / channel:** one sound track that the chip can play at the same time as others.
- **Note, octave:** pitch; each octave doubles the frequency.
- **Pattern, order:** the building blocks of a song and the list in which they are played.
- **Instrument:** waveform and envelope of a sound.
- **Waveform:** triangle (soft), sawtooth (bright), pulse (strong, with pulse width), noise (for drums).
- **Envelope (ADSR):** how the volume of a tone changes over time: rising (Attack), falling (Decay), holding (Sustain), fading out (Release).
- **Beat / tempo:** here the number of frames per row; the field in the toolbar is called **Tempo**.
- **Frame rate:** 50 Hz (PAL) or 60 Hz (NTSC); the player routine runs once per frame.

## What is a tracker?

A **tracker** is a music program in which you put notes not on staves but in **tables**. Each **column** is a **voice** (a channel of the sound chip), each **row** is a step in time. It comes from computer music of the 1980s and fits the chips perfectly: each channel plays only one tone at a time.

The most important terms:

- **Cell:** an entry made of a note and an instrument, for example `C-4 01` (note C in octave 4, instrument 01). `---` means "nothing" (shown as dots in the grid), `===` means "note off".
- **Pattern:** a table with a fixed number of rows, for example 32. A pattern is a building block, such as "four bars of bass".
- **Order:** the list that says in which sequence the patterns are played. Because you can reuse patterns, songs stay small.
- **Instrument:** the tone color. An instrument sets the waveform and the **envelope**, that is, how a tone becomes loud and fades out again (see below).
- **Tempo** (also called beat): the number of **frames** per row. A frame is one pass of the screen, 1/50 second on PAL. Tempo 6 means: each row lasts six frames.

## My first song (using the example `music_demo`)

1. **Open the example:** On the start page, open `examples/c64/music_demo` from the example projects (to make changes, use **Copy**, then you have your own version). *Expected:* The file tree contains `music/tune.vcsong`.
2. **Open the song:** At the top, choose the main section **Music**. If nothing is open yet, the editor shows the **Songs** of the project as buttons; click `music/tune.vcsong`. A double click on the file in the file tree works too. *Expected:* On the left the order and the instruments, in the middle the grid with three voices, on the right the chosen instrument.
3. **Listen:** Click **Play song (Space)** (arrow in the toolbar) or press the **Space** key while the cursor is in the grid. *Expected:* The song plays through the built-in synthesizer, the current row is marked, the levels move in the voice headers. **Stop playback** (or the Space key again) stops it.
4. **Enter a note:** Click a cell, press the key at the bottom left (German keyboard: **Y**). *Expected:* The cell shows a note, the cursor moves on, the tone sounds briefly.
5. **Change the sound:** In the list **Instruments**, choose another instrument. On the right you change the waveform or move the sliders. A click on a key of the **Keyboard** previews the tone with the instrument. *Expected:* The sound changes, and you can hear it.
6. **Save:** **Cmd/Ctrl+S**. *Expected:* The **●** ("unsaved") in the footer disappears.
7. **Build and run:** **F7**, **F5**. *Expected:* The program plays the song in the emulator.

The **New file** dialog does not yet create a new song in this version. Start with an example project or an existing song.

## Layout of the user interface

The music editor has three areas side by side. In narrow windows the areas move below each other.

| Area | Content |
|---|---|
| Left: **Song** | name of the song with **New pattern** (plus), the **Order** of the patterns (Append, Set, Remove, arrows, **Repeat from entry**), the list of **Instruments** and, if the project has several songs, the **Songs** |
| Middle | toolbar, selection of the visible **Voices**, the patterns as buttons, the **grid** (each column a voice, each row a step in time) and the footer |
| Right: **Instrument** | name, waveform and sliders of the chosen instrument, the **Keyboard** and the **Volume** |

The toolbar contains, from left to right: **Undo**, **Redo**, **Play song (Space)**, **Pattern ▶**, **Stop playback**, the switch **Follow playback**, the fields **Tempo**, **Rows**, **Octave** and **Step** (how many rows the cursor moves on after a note), **Semitone ↑**, **Semitone ↓**, **Apply instrument**, **Convert**, for POKEY **Import RMT module**, **Export …** and **Save**.

![The Musik tab with the file tune.vcsong: on the left the order list (pattern 1, 2, 1, 2) and the instruments Puls, Bass and Snare, in the middle the toolbar with follow, tempo, rows, octave and step and the grid with three voices, on the right the cards for instrument 00 Puls, keyboard and volume. The cursor is on row 02, and the status line says unsaved.](bild:tracker-ueberblick "The tracker with order list, grid with three voices and instruments.")

The footer shows the name of the file, the current pattern, the row and the voice of the cursor and a key hint; **●** reports unsaved changes, and while playback runs it shows the entry and the row.

The commands **Tracker: Notes**, **Tracker: Instruments** and **Tracker: Order** (**Cmd/Ctrl+1**, **Cmd/Ctrl+2**, **Cmd/Ctrl+3**) move the focus to the grid, the instrument list or the order. The order of the patterns has no place in PLAY files: There, the hint "PLAY files have one pattern and no order." is on the left.

### Entering notes

Click a cell or move with the **arrow keys** (**Page up/down** jumps eight rows, **Home/End** to the first/last row). Hold the mouse button and drag to select. Then you play with the computer keyboard like a piano:

```text
upper row:  Q  2  W  3  E  R  5  T  6  Z  7  U  I      (one octave higher, German keyboard)
lower row:  Y  S  X  D  C  V  G  B  H  N  J  M         (current octave)
```

The keys count by **position**, not by the label: On an American or French keyboard, the same tones are in the same places. You set the keyboard layout in the settings ("Automatic" detects it).

After each note the cursor moves on by the **Step**. More keys, as long as the grid has the focus:

| Key | Effect |
|---|---|
| Del or Backspace | clear the cell |
| **1** | set "note off" (`===`) |
| **[** and **]** | choose the previous/next **instrument** |
| **,** and **.** | octave down/up (period = up) |
| Alt+Up / Alt+Down | change the instrument of the cell (or of the selection) |
| Shift+arrows | select several cells |
| Space | play/stop the song |
| Shift+Space | play only the chosen pattern |
| Cmd/Ctrl+C, X, V, A | copy, cut, paste a block, select all |
| Cmd/Ctrl+S | save |
| Cmd/Ctrl+Z, Cmd/Ctrl+Shift+Z | undo, redo |

You can switch the single keys (notes, **1**, **[**, **]**, **,**, **.**) off in the settings ("Single keys in the editors"); then the arrows, Space, Del and all shortcuts with Cmd/Ctrl stay.

**Semitone ↑** and **Semitone ↓** move the selected notes by one semitone. **Apply instrument** sets the chosen instrument on all notes of the selection.

In the **Keyboard** on the right, the keys of the tones that are sounding right now light up. A click on a key previews the tone with the chosen instrument but does not enter anything. The label is the computer key when the key hints are switched on.

![The 'Musik' tab with tune.vcsong. On the left the order list and the instruments, in the middle the toolbar, voices 1 to 3 and the pattern grid with notes like C-4 00 and C-2 01; the cursor is in voice 3, row 01. On the right the instrument 'Puls' with waveform, sliders and the keyboard with computer keys.](bild:tracker-muster "The pattern grid with three voices and the keyboard on the right.")

### Patterns and order

1. The **plus** next to the name of the song creates an empty **New pattern**. The patterns appear above the grid as buttons ("Pattern 1", "Pattern 2" ...); a click switches the grid.
2. The **Order** on the left is the list in which the patterns are played. **Append** adds the chosen pattern at the end, **Set** replaces the chosen entry with the chosen pattern, **Remove** deletes the entry, the arrows **Move entry up** and **Move entry down** reorder.
3. **Repeat from entry** names the place where the song starts again after the end. In the list, **↻** marks this place.
4. **Duplicate** copies the chosen pattern, **Delete** removes it.
5. **Rows** (number of rows of the pattern, 1 to 128) and **Tempo** (1 to 255) are in the toolbar.

### Instruments and envelope

On the right you see the instrument chosen in the list **Instruments** (number and name, hex number as in the cell). **New instrument** creates one (at most 10 in the PLAY tracker, otherwise 32), **Duplicate** copies the chosen one. Only the last, unused instrument can be deleted. The name field above the sliders renames it.

**SID (C64):**

- **Waveform:** Triangle (soft), Saw (bright and sharp), Pulse (strong, you set the width with **Pulse width**) and Noise (for drums and explosions).
- **Envelope** made of four sliders: **Attack**, **Decay**, **Sustain** and **Release**, together ADSR. It describes how the volume of a tone changes: *Attack* is the time in which it rises from silence to full volume; *Decay* is the time in which it falls to the holding volume *Sustain*; *Release* is the time in which it fades out after "note off". Each value runs from 0 to 15; for Sustain it is the **level**, not a time value.

![The 'Musik' tab with tune.vcsong. On the right the column 'Instrument 00 - Puls' with the waveforms (square selected), the sliders Anschlag, Abfall, Halten, Ausklang and Pulsbreite, the keyboard and the volume. On the left the order list and instruments, in the middle the pattern with three voices.](bild:tracker-instrument "The right column of the tracker with waveform, envelope, pulse width and keyboard.")

**POKEY (Atari):** Here there is no ADSR envelope, but a **Distortion** (Pure tone, Bass, Buzz or Gritty) and the **Envelope per frame**: 16 vertical sliders from 0 to 15, one value per frame. The field **Envelope repeats from frame** says from which frame the curve repeats.

*[Bild folgt in E13: tracker-pokey]*

The **Volume** (0 to 15) below the keyboard applies to the whole song.

### Listening

- **Play song (Space)** plays from the chosen entry of the order, **Pattern ▶** only the current pattern. **Stop playback** stops. The Space key does the same (with Shift for the pattern).
- Listening uses a **built-in synthesizer** that imitates the chips. It sounds similar, but not identical.
- If the song changes in the grid while it plays, you hear the change at once. **Follow playback** lets the grid and the order scroll along.
- In each voice header there are **M** (mute), **S** (solo) and a level bar. Mute and solo also act on the sound.
- **Many voices** (twelve on the MEGA65): The grid then has a horizontal scroll bar. Above the grid are the buttons of the **Voices**; with them you hide voices that you do not need right now. They only disappear from the grid, **but keep playing** (if they are not muted), and the file stays unchanged. **Show all** brings them back. One voice always stays visible.

![The 'Musik' tab with tune.vcsong. Above the pattern grid are the buttons for voices 1, 2 and 3 and 'Alle einblenden'. Voice 1 has the note C-3 in row 01 and the cursor is in row 02; on the right is the 'Klaviatur' card with no key highlighted.](bild:tracker-hoerprobe "The tracker with the voice bar above the grid and the keyboard on the right.")

## Hardware details and limits

| | C64 (SID) | Atari (POKEY) | MEGA65 (PLAY) |
|---|---|---|---|
| Voices | 3 | 4 | as set by the PLAY command |
| Sound | triangle, sawtooth, pulse, noise, ADSR | distortion and volume per frame | envelopes 0 to 9 |
| Instruments | up to 32 | up to 32 | at most 10 |
| Patterns | up to 128 rows | like SID | one pattern, played once |

Listening uses a **built-in synthesizer** and sounds similar, but not identical, to the real chip. Check important sounds in the emulator or on the hardware.

## Neighbor editors and glossary

- For game music and effects you also need the [Sprite](kapitel:sprites) and [Screen editors](kapitel:bildschirm-kacheln) for what people see. The chapter [PLAY tracker](kapitel:play-tracker) describes the MEGA65 PLAY tracker.
- Terms: Glossary (file `GLOSSAR.md`): SID, POKEY, envelope, pattern, tracker.

## Getting songs from elsewhere

- **Convert** (toolbar) brings a song from another system into this project (**SID <-> POKEY**), a **GoatTracker file** (`.sng`) or a **SID file** (`.sid`). You choose the file and, in the dialog **Convert a song from another system**, the **Name of the new file** and the **Target system**. For SID files, the player routine runs in a built-in 6502 core; the editor records the register values and builds notes and instruments from them (fields **Subtune** and **Recording length in seconds**). The result is an approximation; vibrato, filter and similar things are dropped. The output says what was not kept.
- **Import RMT module** (POKEY only) loads a module of the **Raster Music Tracker**, the classic Atari tracker.

*[Bild folgt in E13: tracker-sid-pokey]*

## Using it in the program

As with all editors, the export is created by itself when you build (toolbar > **Export …** or **Cmd/Ctrl+Shift+E** for the dialog).

### C64 (SID)

The project `examples/c64/music_demo` contains `music/tune.vcsong`. The program:

```asm
.target c64
.cpu 6510 { illegals = deny }

.org $0801
.basicstub start

.include "lib/music.inc"

start:  sei
        music_init tune_order, tune_patterns, tune_ins, tune_speed, tune_rows, tune_patsize
        ; Raster line IRQ instead of KERNAL IRQ: exactly one tick per frame (PAL 50 Hz, NTSC 60 Hz).
        lda #$7f
        sta $dc0d                   ; CIA interrupts off
        sta $dd0d
        lda $dc0d                   ; acknowledge pending messages
        lda $dd0d
        lda #$01
        sta $d01a                   ; raster IRQ on
        lda #$00
        sta $d012                   ; raster line 0
        lda $d011
        and #$7f
        sta $d011
        lda #<irq
        sta $0314
        lda #>irq
        sta $0315
        cli
loop:   jmp loop

irq:    lda #$01
        sta $d019                   ; acknowledge raster IRQ
        jsr music_play
        jmp $ea81                   ; restore registers, RTI

music_runtime
.include "music/tune.vcsong"
```

- `music_init` gives the player the tables of the export: order, patterns, instruments, speed, number of rows and pattern size.
- The **play routine** `music_play` must be called **once per frame**. A **raster interrupt** does that: The graphics chip interrupts the main program at raster line 0 and jumps to `irq`.
- `music_runtime` places the play routine **after** the main program.

> **Caution:** Use the **raster** interrupt, not the KERNAL interrupt. The KERNAL interrupt of the C64 also runs at 60 Hz on a PAL machine; the song would then play one fifth too fast.

1. Open `examples/c64/music_demo`, build with **F7**, run with **F5** (VICE).
2. Change a note in the tracker, save, build and run again.

### Atari (POKEY)

The project `examples/atari/music_demo` plays `music/atune.vcsong` with the original player of the Raster Music Tracker:

```asm
.target atari8
.cpu sally { illegals = deny }

.org $2000
.include "lib/rmt.inc"

start:  rmt_start atune_rmt, 0      ; module + start row (returns the instrument speed in A)
        rmt_vbi
done:   jmp done

rmt_vbi_code
.include "lib/rmtplayr.asm"         ; player: tables from $30E0, code from $3400
.org $4000
.include "music/atune.vcsong"
.run start
```

The export for POKEY is an **RMT module** that the player understands directly. The player hooks into the **deferred VBI** of the operating system, the interrupt at the end of each frame. The lowest note in the POKEY tracker is C-3.

## Common stumbling blocks

- **The song runs too fast or too slow:** The play routine must run once per frame, best by a raster interrupt (C64).
- **You hear nothing:** In the tracker, check whether a voice is set to **M** (mute) and whether the volume of your computer is not off. If there is an error message "Audio: ..." at the top, something is wrong with the audio output.
- **A note sounds forever:** Set a "note off" (`===`) with **1**.
- **The sound differs from the emulator:** Listening is a synthesizer, not the real chip.

## What next?

For the MEGA65 there is a separate, simpler way to make music: [PLAY tracker](kapitel:play-tracker). To look things up: [Tracker](hilfe:tracker), [Envelope](hilfe:envelope), [Export](hilfe:export).
