; music_demo.asm - plays a song composed with the tracker through the SID (C64), clocked by a raster line IRQ
;
; Purpose: shows how a tracker song (music/tune.vcsong) is exported, included and played. The player routine is in
;   lib/music.inc (macros music_init, music_play, music_stop, music_runtime); this program only makes sure that `music_play`
;   is called exactly once per frame.
;
; Preparation and usage: `vcc build examples/c64/music_demo` exports the song itself (by hand: `vcc media export
;   music/tune.vcsong`). Load the PRG, RUN: the song plays until you press RESET. The program has no
;   controls; the keyboard is no longer scanned (the KERNAL part of the IRQ with the keyboard scan is bypassed).
;
; The song (music/tune.vcsong): 3 voices, tempo 6 frames per row, 32 rows per pattern, 2 patterns in the order 0, 1, 0, 1 and
;   then back to the start (loop 0). Voice 1 plays arpeggios with the instrument "Puls" (pulse wave), voice 2 with "Bass"
;   (sawtooth) one bass note every eight rows, voice 3 with "Snare" (noise) one beat every eight rows. Pattern 0 sounds like C major over C,
;   pattern 1 like D major over G. At 50 frames per second (PAL) a row is 0.12 s long, a pattern 3.84 s, the song about 15 s.
;
; Timer: The player routine counts in frames. So the song does not run from the KERNAL IRQ (CIA timer, 60 Hz even on the PAL C64:
;   the song would be 20 % too fast), but from a raster line IRQ of the VIC-II at line 0: exactly one call per frame (PAL 50 Hz,
;   NTSC 60 Hz). The IRQ uses the KERNAL entry $EA81 (only restore registers and RTI), so without keyboard and clock.
;
; Hardware: VIC-II $D011 (bit 7 = raster line bit 8), $D012 (compare line), $D019 (interrupt status, acknowledge), $D01A
;   (interrupt enable); CIA 1 and 2 $DC0D/$DD0D (interrupt control register); SID $D400-$D418 (via music.inc); IRQ vector $0314/$0315.
; Memory map: $0801 BASIC line `10 SYS 2061`, program from $080D, then player routine and song data; the whole block ends at $0C64
;   (1126 bytes with load address). Zero page $F7-$FA (player routine, see music.inc). PAL/NTSC: the default is the PAL frequency table,
;   with -D NTSC=1 the NTSC table.
; Limits: no return to BASIC, end only with RESET; the zero page $F7-$FA holds the state of the player routine and must not be used
;   otherwise between the calls of music_play.
.target c64
.cpu 6510 { illegals = deny }

.org $0801
.basicstub start

.include "lib/music.inc"

start:  sei                         ; during setup no interruption
        music_init tune_order, tune_patterns, tune_ins, tune_speed, tune_rows, tune_patsize   ; fill the player routine with the song tables
        ; Raster line IRQ instead of KERNAL IRQ: exactly one tick per frame (PAL 50 Hz, NTSC 60 Hz).
        ; The KERNAL IRQ runs at 60 Hz even on the PAL C64 and played the song 20 % too fast.
        lda #$7f
        sta $dc0d                   ; CIA interrupts off (bit 7 = 0: the set bits 0-6 are cleared)
        sta $dd0d
        lda $dc0d                   ; acknowledge pending messages
        lda $dd0d
        lda #$01
        sta $d01a                   ; raster IRQ on
        lda #$00
        sta $d012                   ; raster line 0 (lower eight bits of the compare line)
        lda $d011
        and #$7f
        sta $d011                   ; bit 8 of the compare line = 0 (bit 7 of $D011 is bit 8 of the compare value when writing)
        lda #<irq
        sta $0314                   ; redirect the KERNAL IRQ vector to our own routine
        lda #>irq
        sta $0315
        cli                         ; interrupts enabled: from now on the music plays
loop:   jmp loop                    ; main program has nothing to do

; irq - raster line interrupt, once per frame at line 0 (via the KERNAL entry $FF48/$0314; registers are already saved there).
;   Acknowledges the interrupt, plays one tick of the music and leaves the interrupt via the KERNAL end at $EA81.
irq:    lda #$01
        sta $d019                   ; acknowledge raster IRQ
        jsr music_play
        jmp $ea81                   ; restore registers, RTI

music_runtime                       ; player routine (music_play, mus_step, state, frequency table) - after the main program
.include "music/tune.vcsong"    ; song data: tune_order, tune_patterns, tune_ins and the constants tune_speed, tune_rows, tune_patsize
