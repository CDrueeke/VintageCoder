; music.inc - C64: player routine for tracker songs (SID, 3 voices) from the export.
; GENERATED from music.inc.in by tools/gen_music.py - make changes only in the template music.inc.in (it is the C64 version and
; is copied unchanged; the MEGA65 version is created from it by text replacements).
;
;   .include "lib/music.inc"
;   music_init tune_order, tune_patterns, tune_ins, tune_speed, tune_rows, tune_patsize   ; labels/constants from the export
;   ...                                   ; once per frame (50/60 Hz, e.g. in the raster line IRQ):
;   jsr music_play
;   music_stop                            ; gates off, volume 0
;   music_runtime                         ; ONCE, after the main program (not executed)
;
; Frequency table PAL; with -D NTSC=1 (or .const NTSC = 1 before the include) the NTSC table.
; Zero page used: $F7-$FA. State: mus_row, mus_ord, mus_tick (memory); mus_sid = image of the SID registers ($D400-$D418).
; Instruments: at most 32 (SID record 8 bytes). NOTE_B7 is too high on PAL for the 16-bit frequency counter (upper limit 65535, see NOTE_TABLE_HIGHEST).
;
; Data format of the song tables (export of the tracker, encoded in crates/vc-media/src/song.rs):
;   order      pattern numbers, then $FF and the repeat position (index into order where it continues after the end)
;   patterns   per pattern: rows * voices * 2 bytes, row by row, per voice note and instrument.
;              Note: 0 = nothing (tone keeps playing), 1..96 = C-0..B-7 (index into the frequency table), $FF = gate off (tone dies away).
;              Instrument: $FF = leave unchanged, otherwise number of the instrument (applies from then on to this voice).
;   ins        per instrument 8 bytes: +0 control byte (waveform: $10 triangle, $20 sawtooth, $40 pulse, $80 noise; gate bit 0 is off here),
;              +1 attack/decay, +2 sustain/release, +3 pulse width low, +4 pulse width high (bits 0-3), +5..+7 unused
;   speed      frames per row (tempo); rows = rows per pattern; patsize = bytes per pattern (rows * voices * 2)
; Flow: music_play is called once per frame and counts mus_tick up to speed; at mus_tick = 0 mus_step processes the next row
;   of the pattern (for each voice: remember the instrument, on a note write frequency, pulse width, envelope and control byte into the SID
;   registers, briefly switching the gate off and on again so that the envelope restarts). After the last row the
;   next pattern of the order follows; if $FF is there, the routine jumps to the repeat position.
; Self-modifying code: music_init writes the table addresses (order, patterns, ins, patsize) into the operands of individual instructions of the
;   player routine (labels mus_o1.., mus_pb_.., mus_ps_.., mus_ib_..: `.const x = *` gives the next instruction a name, x+1 is
;   its operand). The player routine must therefore be in RAM (not in ROM/cartridge), and music_init must run before the first music_play.
; Zero page: $F7-$FA must not be used otherwise between the music_play calls: mus_ptr ($F7/$F8) keeps the position of the
;   current pattern row from one call to the next. (The tile library uses $F7-$FE - do not use together.)
; Interrupt safety: music_play is not reentrant; it may be called from one place only (e.g. only in the IRQ).
;   Registers A, X, Y and flags are changed; the IRQ frame must save them.

.const mus_ptr = $f7            ; 16 bit: pointer to the current pattern row (is kept between the calls)
.const mus_ip  = $f9            ; 16 bit: pointer to the instrument record (valid only during mus_step)

; music_init(order, patterns, ins, speed, rows, patsize) - connect the player routine to the song tables and start it.
;   Input: labels of the tables (order, patterns, ins) and constants speed, rows, patsize, as the export supplies them.
;   Effect: patches the table addresses into the player routine (see "Self-modifying code"), sets tick, row, order position and
;   the instruments of the voices to 0, clears all SID registers including the image mus_sid, sets the volume to 15 and switches the music
;   on (mus_on = 1). Changes A, X and the flags. The first music_play call then plays the first row immediately.
.macro music_init(order, patterns, ins, speed, rows, patsize)
        lda #<order
        sta mus_o1+1
        lda #>order
        sta mus_o1+2
        lda #<(order+1)
        sta mus_o2+1
        lda #>(order+1)
        sta mus_o2+2
        lda #<order
        sta mus_o3+1
        lda #>order
        sta mus_o3+2
        lda #<patterns
        sta mus_pb_l+1
        lda #>patterns
        sta mus_pb_h+1
        lda #<patsize
        sta mus_ps_l+1
        lda #>patsize
        sta mus_ps_h+1
        lda #<ins
        sta mus_ib_l+1
        lda #>ins
        sta mus_ib_h+1
        lda #speed
        sta mus_speed
        lda #rows
        sta mus_rows
        lda #0
        sta mus_tick
        sta mus_row
        sta mus_ord
        sta mus_ins
        sta mus_ins+1
        sta mus_ins+2
        ; clear all SID registers including the image (loop over all register addresses, downwards until X < 0)
        ldx #24
@clr:   sta mus_sid,x
        sta $d400,x
        dex
        bpl @clr
        lda #15
        sta mus_sid+24
        sta $d418
        lda #1
        sta mus_on
.endmacro

; music_stop - stop the music: mus_on = 0, control registers of all voices to 0 (gate off, no waveform), volume 0.
;   Changes A and the flags. The state of the player routine remains (mus_row etc.); a restart is only possible with music_init.
.macro music_stop
        lda #0
        sta mus_on
        sta $d404
        sta $d40b
        sta $d412
        sta $d418
.endmacro

; music_runtime - the actual player routine, state variables and frequency table (include once, do not execute).
.macro music_runtime
; music_play - play one tick (one frame) further.
;   Input: none. Output: SID registers $D400.. and image are updated at the start of a new row.
;   Calls mus_step at mus_tick = 0; counts mus_tick up to mus_speed and then sets it to 0. Without music_init (mus_on = 0)
;   nothing happens. Changes A, X, Y and the flags.
music_play:                         ; call once per frame
        lda mus_on
        beq @rts
        lda mus_tick
        bne @t
        jsr mus_step
@t:     inc mus_tick
        lda mus_tick
        cmp mus_speed
        bcc @rts
        lda #0
        sta mus_tick
@rts:   rts

; mus_step - process one pattern row (called by music_play when mus_tick = 0).
;   At row start 0 the pattern is first determined from the order and mus_ptr = patterns + pattern number * patsize is computed
;   (by repeated addition). Then the voices of the row are processed one after the other (mus_v = voice, mus_y = byte position in the
;   row, mus_r = register offset of the voice in the SID). At the end mus_ptr advances one row; after the last row the next pattern begins with
;   mus_row = 0 and increased mus_ord. Changes A, X, Y and the flags.
mus_step:
        lda mus_row
        bne @row
        ldx mus_ord                 ; new pattern: read the order ($ff = end, then the repeat position)
.const mus_o1 = *
        lda $ffff,x
        cmp #$ff
        bne @pat
.const mus_o2 = *
        lda $ffff,x
        sta mus_ord
        tax
.const mus_o3 = *
        lda $ffff,x
@pat:   sta mus_t
.const mus_pb_l = *
        lda #0
        sta mus_ptr
.const mus_pb_h = *
        lda #0
        sta mus_ptr+1
        ldx mus_t
        beq @row
@add:   clc
        lda mus_ptr
.const mus_ps_l = *
        adc #0
        sta mus_ptr
        lda mus_ptr+1
.const mus_ps_h = *
        adc #0
        sta mus_ptr+1
        dex
        bne @add
@row:   ldx #0                      ; voice 0..2
        stx mus_v
        ldy #0
@voice: lda (mus_ptr),y
        sta mus_n
        iny
        lda (mus_ptr),y
        iny
        sty mus_y
        cmp #$ff
        beq @keep
        sta mus_ins,x
@keep:  lda mus_n
        beq @next
        cmp #$ff
        beq @off
        lda mus_voff,x              ; register offset of the voice
        sta mus_r
        lda mus_ins,x               ; instrument record = base + number * 8
        asl a
        asl a
        asl a
        clc
.const mus_ib_l = *
        adc #0
        sta mus_ip
.const mus_ib_h = *
        lda #0
        adc #0
        sta mus_ip+1
        ldx mus_r
        ldy mus_n
        lda note_sid_lo,y
        sta mus_sid,x
        sta $d400,x
        lda note_sid_hi,y
        sta mus_sid+1,x
        sta $d401,x
        ldy #3
        lda (mus_ip),y
        sta mus_sid+2,x
        sta $d402,x
        iny
        lda (mus_ip),y
        sta mus_sid+3,x
        sta $d403,x
        ldy #1
        lda (mus_ip),y
        sta mus_sid+5,x
        sta $d405,x
        iny
        lda (mus_ip),y
        sta mus_sid+6,x
        sta $d406,x
        ldy #0
        lda (mus_ip),y
        sta mus_sid+4,x             ; gate off (restart of the envelope) ...
        sta $d404,x
        ora #1                      ; ... and on again
        sta mus_sid+4,x
        sta $d404,x
        jmp @next
@off:   lda mus_voff,x
        tax
        lda mus_sid+4,x
        and #$fe
        sta mus_sid+4,x
        sta $d404,x
@next:  inc mus_v
        ldx mus_v
        ldy mus_y
        cpx #3
        beq @rowend
        jmp @voice
@rowend:
        clc                         ; pointer to the next row (3 voices * 2 bytes)
        lda mus_ptr
        adc #6
        sta mus_ptr
        bcc @c
        inc mus_ptr+1
@c:     inc mus_row
        lda mus_row
        cmp mus_rows
        bcc @done
        lda #0
        sta mus_row
        inc mus_ord
@done:  rts

; --- State of the player routine (in RAM, therefore changeable) ----------------------------------------------------------------
; mus_on: 1 = music plays, 0 = stopped. mus_speed/mus_rows: frames per row and rows per pattern (music_init overwrites the
; defaults 6 and 32). mus_tick: frame counter 0..speed-1. mus_row: current row in the pattern. mus_ord: position in the order.
; mus_t, mus_n, mus_v, mus_y, mus_r: working variables of mus_step (pattern number, note, voice, byte position, register offset).
; mus_ins: current instrument per voice. mus_voff: offset of the first SID register per voice (+7 per voice).
; mus_sid: image of the SID registers, because the registers are write-only (for gate bit and volume).
mus_on:    .byte 0
mus_speed: .byte 6
mus_rows:  .byte 32
mus_tick:  .byte 0
mus_row:   .byte 0
mus_ord:   .byte 0
mus_t:     .byte 0
mus_n:     .byte 0
mus_v:     .byte 0
mus_y:     .byte 0
mus_r:     .byte 0
mus_ins:   .byte 0,0,0
mus_voff:  .byte 0,7,14
mus_sid:   .fill 25, 0
; Frequency table (note_sid_lo/hi, indexed by the note value) and note constants: <c64/notes.inc>, PAL or with NTSC the NTSC table
#include <c64/notes.inc>
.endmacro
