# PLAY tracker (MEGA65)

A grid of **steps x voices** for the BASIC 65 command `PLAY`. One step is a sixteenth note. A note sounds until a new note or `===` (rest) follows. The editor is the [Tracker](help:tracker) in the area **Music**; for `.vcplay` files it has six voices, no order of patterns (a pattern is played once) and envelopes 0 to 9 instead of instruments.

- The dialog **New file** does not yet create new PLAY songs in this version. Open an existing piece or the example (see below).
- Input as in the tracker: keyboard rows Z S X D C V G B H N J M, Q 2 W 3 E R 5 T 6 Y 7 U I (by key position; on QWERTZ keyboards Y and Z are swapped). Del or Backspace clears a cell, `1` sets a rest, `[` and `]` choose the envelope, `,` and `.` change the octave. **Tempo**, **Rows** (steps) and **Octave** are in the toolbar, the **Volume** is on the right below the keyboard.

*[Bild folgt in E13: play-tracker]*

- Envelopes 0-9 correspond to `ENVELOPE n,a,d,s,r,waveform,pw`.
- **Export ...** (toolbar) opens the export dialog. **Save to project** writes `build/export/NAME.bas`: `VOL`, `TEMPO`, `ENVELOPE` and, for each voice, one or more `PLAY "V1T0O4QC..."` strings (T envelope, O octave, W/H/Q/I/S duration, `.` dotted, R rest).
- Lengths that you cannot write as a single note (for example 5 steps) are written as a repeated attack.

Note: The preview in the app is an approximation. The generated BASIC text has not been tested on real hardware.

## Example
`examples/mega65/play_demo/music/tune_play.vcplay` is the same piece of music as `tune4` (tracker with four SIDs), converted to six PLAY voices: melody, bass, three arpeggio voices and snare, 64 steps, the same four envelopes. Open it in the project (group "PLAY tracker"), listen to it with the space bar and export it as a BASIC program with `vcc media export`.
