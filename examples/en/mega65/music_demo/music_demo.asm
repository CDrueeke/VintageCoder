; =====================================================================================================================
; music_demo.asm - MEGA65: tracker song on four SID chips (12 voices)
; =====================================================================================================================
; What the program shows
;   The MEGA65 has four SID chips (SID 1-4 at $D400, $D420, $D440, $D460, 3 voices each = 12 voices).  The program plays the song
;   music/tune4.vcsong composed with the tracker (profile mega65.sid4.song: 12 tracks per row, 4 instruments, 2 patterns of 32 rows,
;   order 0, 1, 0, 1 and then from the start again) in an endless loop.  You hear the music; nothing changes on the screen.
;   You learn: export the song, include the player routine, call it once per frame.
;
; Usage / flow
;   * The song is converted by itself at `.include "music/tune4.vcsong"` (by hand: `vcc media export music/tune4.vcsong`)
;     (labels tune4_order, tune4_patterns, tune4_ins and the constants tune4_speed, tune4_rows, tune4_patsize).  `vcc build` in the
;     project folder needs no export for this.
;   * Build by hand: vcc asm music_demo.asm --target mega65 --cpu 45gs02 -o music_demo.prg   (load address $2000, no BASIC starter:
;     load it in the emulator/monitor and start it with `g 2000` or `SYS 8192`).  For NTSC machines give -D NTSC=1 (different frequency table).
;   * Main loop: once per frame (raster line $FF, i.e. at 50 Hz PAL / 60 Hz NTSC) it calls `music_play`.  One call is one
;     tick of the player; the song has 6 ticks (frames) per row (tune4_speed) and 32 rows per pattern.
;
; Hardware / library
;   lib/music.inc (generated from examples/c64/music_demo/lib/music.inc.in by tools/gen_music.py): macros music_init, music_stop,
;   music_runtime and the subroutine music_play.  It writes the SID registers $D400-$D47F and keeps a copy of them (mus_sid).
;   $D012 = raster line (bits 0-7), on which the loop waits for the frame change.
;
; Memory
;   Program and player routine from $2000 (approx. 2.4 KB with the song data); zero page $F7-$FA (pointer of the player routine); nothing else.
;   The program runs without KERNAL interrupts (SEI) and never returns.
; =====================================================================================================================
.target mega65
.cpu 45gs02 { illegals = deny }

.org $2000
.include "lib/music.inc"            ; only defines constants and macros, generates no bytes here yet

start:  sei                         ; no interrupts: the program clocks the music itself via the raster line
        music_init tune4_order, tune4_patterns, tune4_ins, tune4_speed, tune4_rows, tune4_patsize   ; set pointers, clear SIDs, volume 15
frame:  lda $d012                   ; wait for raster line $FF, then play one tick
        cmp #$ff
        bne frame
        jsr music_play              ; one player tick (row change every tune4_speed ticks)
wait:   lda $d012                   ; leave the line so that it plays only once per frame
        cmp #$ff
        beq wait
        bra frame

music_runtime                       ; place the player routine once after the main program (it is not "run through")
.include "music/tune4.vcsong"   ; song data from the export (order, patterns, instruments)
