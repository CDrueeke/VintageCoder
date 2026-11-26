; ---------------------------------------------------------------------------------------------------------------------
; music.inc - VIC-20: player routine for tracker songs (VIC-I: bass, alto, soprano and noise = 4 voices) from the export (profile vic20.vic.song).
; ---------------------------------------------------------------------------------------------------------------------
;
; USAGE
;   .include "lib/music.inc"
;   music_init tune_order, tune_patterns, tune_ins, tune_speed, tune_rows, tune_patsize   ; labels/constants from the export
;   ...                                   ; once per frame (50/60 Hz, e.g. in the IRQ):
;   jsr music_play
;   music_stop                            ; all voices off (macro)
;   music_runtime                         ; ONCE, behind the main program (not executed): player routine and variables
;   .include "lib/vic_freq.inc"           ; frequency tables (with .const NTSC = 1 before the include: the NTSC table)
;
; DATA FORMAT (tracker export, here the text from music/tune.vcsong)
;   order: one pattern number per byte; $FF = end of the order, the byte behind it is the position where it starts again (loop).
;   patterns: patterns of patsize bytes each = rows rows of 8 bytes each; a row holds the 4 voices of 2 bytes each (note, instrument). Voice 0 =
;   bass, 1 = alto, 2 = soprano, 3 = noise. Note: 0 = nothing (the tone keeps playing), $FF = tone off, 1..96 = note (C-0 = 1; larger values are not
;   defined, the frequency tables end at 96). Instrument: $FF = keep, otherwise number 0..7.
;   ins: instrument records of 32 bytes each, byte 0 = distortion (ignored), byte 1 = loop point of the envelope (0..15), bytes 2..17
;   = 16 volume values of the envelope, the rest is unused. speed = frames per row, rows = rows per pattern, patsize = bytes per pattern (rows *
;   8).
;
; VIC-I AND LIMITS
;   The VIC-I has only one shared volume ($900E, bits 0-3), no envelope per voice. The volume envelope of the instrument (16 values, one
;   per frame) therefore only switches the voice on (value > 0) or off (0); the volume stays at 15. The distortion of the instrument is
;   ignored. Voice 0 = bass ($900A), 1 = alto ($900B), 2 = soprano ($900C), 3 = noise ($900D). Instruments: at most 8 (record 32 bytes,
;   number * 32 must fit in a byte). Notes outside the pitch range of a voice are moved by octaves (see tools/gen_vic_freq.py); for noise
;   the note only determines the timbre.
;   The player routine changes its own code: music_init writes the addresses of order, patterns and instruments into instruction operands of
;   music_runtime (the instructions marked with ".const mus_xx = *"). It therefore runs only in RAM, not in ROM, and music_init must be called before the first
;   music_play.
;
; MEMORY
;   Zero page used: $FB-$FE (mus_ptr, mus_ip). Variables mus_* are in the program behind the code. All routines are not reentrant:
;   music_play may be called from one place only (IRQ or main loop). music_play changes A, X, Y and the flags.
; ---------------------------------------------------------------------------------------------------------------------

.const mus_ptr = $fb            ; 16 bit: pointer to the current pattern row
.const mus_ip  = $fd            ; 16 bit: pointer to instrument record or frequency table of the voice (each used only briefly)

; music_init(order, patterns, ins, speed, rows, patsize): prepare and start the player (macro, once before the first music_play; call again = song from the start).
;   order/patterns/ins are labels, speed/rows/patsize constants of the export. Writes the addresses into the operands of music_runtime (see above),
;   clears the state of all voices (off, instrument 0, envelope from the start), sets the voice registers $900A-$900D to 0 and the volume to 15. Changes A, X.

.macro music_init(order, patterns, ins, speed, rows, patsize)
        lda #<order                 ; write the order into the three instructions that read it:
        sta mus_o1+1                ;   mus_o1: read the pattern name at the position
        lda #>order
        sta mus_o1+2
        lda #<(order+1)             ;   mus_o2: at the end marker $FF read the byte behind it (loop position), hence order+1
        sta mus_o2+1
        lda #>(order+1)
        sta mus_o2+2
        lda #<order                 ;   mus_o3: read the pattern name at the loop position
        sta mus_o3+1
        lda #>order
        sta mus_o3+2
        lda #<patterns              ; write the start of the patterns, the size of a pattern (for "number * patsize") and the instrument base
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
        lda #speed                  ; frames per row and rows per pattern
        sta mus_speed
        lda #rows
        sta mus_rows
        lda #0                      ; start from the very beginning: tick, row and position in the order 0
        sta mus_tick
        sta mus_row
        sta mus_ord
        ldx #3                      ; voice 3..0:
@clr:   sta mus_act,x               ;   not sounding, instrument 0, envelope position 0, no register value, VIC-I voice off
        sta mus_ins,x
        sta mus_pos,x
        sta mus_reg,x
        sta $900a,x
        dex
        bpl @clr
        lda $900e                   ; keep the auxiliary color (bits 4-7), volume 15
        and #$f0
        ora #$0f
        sta $900e
        lda #1                      ; release the player (music_play only does something if mus_on <> 0)
        sta mus_on
.endmacro

; music_stop: stop the player and switch off all four voices (macro). Start again with music_init. Changes A.
.macro music_stop
        lda #0
        sta mus_on
        sta $900a
        sta $900b
        sta $900c
        sta $900d
.endmacro

; music_runtime: player routine and variables (macro, call ONCE behind the main program).
.macro music_runtime
; music_play: one player step, call once per frame (does nothing while mus_on = 0). In the first frame of each row (mus_tick = 0) the next
;   pattern row is read (mus_step); in every frame the envelope advances one step (mus_env). After speed frames the next row begins.
;   Changes A, X, Y, flags, mus_* and $FB-$FE; writes $900A-$900D.
music_play:                         ; call once per frame
        lda mus_on
        beq @rts
        lda mus_tick
        bne @t
        jsr mus_step
@t:     jsr mus_env
        inc mus_tick
        lda mus_tick
        cmp mus_speed
        bcc @rts
        lda #0
        sta mus_tick
@rts:   rts

; mus_step: read a pattern row and trigger the notes. If mus_row = 0, the pattern is determined first: name from the order (mus_ord), mus_ptr =
;   patterns + name * patsize (by repeated adding). Then for voices 0..3: read note and instrument; note 0 = do nothing, $FF = voice off, otherwise
;   register value from the frequency table (index note - 1), voice active, envelope from the start. At the end mus_ptr + 8, mus_row + 1; after rows rows mus_row = 0 and
;   mus_ord + 1. Changes A, X, Y, mus_ptr, mus_ip and the voice variables. mus_v = current voice, mus_y = Y holder (row position), mus_n = note.
mus_step:                           ; read a pattern row and trigger the notes
        lda mus_row
        bne @row
        ldx mus_ord                 ; new pattern: read the order ($ff = end, then the loop position)
.const mus_o1 = *                   ; (the operand of this instruction is written by music_init: address of the order)
        lda $ffff,x
        cmp #$ff
        bne @pat
.const mus_o2 = *                   ; (operand = order + 1: the byte behind the $FF)
        lda $ffff,x
        sta mus_ord                 ; loop position
        tax
.const mus_o3 = *                   ; (operand = order)
        lda $ffff,x
@pat:   sta mus_t                   ; pattern number
.const mus_pb_l = *                 ; (operand of lda: low byte of the pattern address; likewise mus_pb_h, mus_ps_l/h, mus_ib_l/h)
        lda #0
        sta mus_ptr
.const mus_pb_h = *
        lda #0
        sta mus_ptr+1
        ldx mus_t
        beq @row                    ; pattern 0: mus_ptr already points to the start
@add:   clc                         ; mus_ptr += patsize, pattern number times
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
@row:   ldx #0                      ; voice 0..3
        stx mus_v
        ldy #0
@voice: lda (mus_ptr),y            ; note of the voice
        sta mus_n
        iny
        lda (mus_ptr),y             ; instrument (Y stays as row position in mus_y)
        iny
        sty mus_y
        cmp #$ff
        beq @keep                   ; $FF: keep instrument
        sta mus_ins,x
@keep:  lda mus_n
        beq @next                   ; note 0: do nothing
        cmp #$ff
        beq @off                    ; note $FF: voice off
        lda mus_fpl,x               ; frequency table of the voice
        sta mus_ip
        lda mus_fph,x
        sta mus_ip+1
        ldy mus_n
        dey
        lda (mus_ip),y
        sta mus_reg,x
        beq @off                    ; 0 = tone cannot be represented
        lda #1                      ; voice sounds; envelope starts (mus_env writes the register in the same frame)
        sta mus_act,x
        lda #0
        sta mus_pos,x
        beq @next                   ; (always, A = 0)
@off:   lda #0
        sta mus_act,x
        sta $900a,x                 ; VIC-I voice x off (X = 0..3 is also the register offset)
@next:  inc mus_v
        ldx mus_v
        ldy mus_y
        cpx #4
        beq @rowend
        jmp @voice                  ; next voice
@rowend:
        clc                         ; pointer to the next row (4 voices * 2 bytes)
        lda mus_ptr
        adc #8
        sta mus_ptr
        bcc @c
        inc mus_ptr+1
@c:     inc mus_row
        lda mus_row
        cmp mus_rows
        bcc @done
        lda #0                      ; end of pattern: next pattern of the order
        sta mus_row
        inc mus_ord
@done:  rts

; mus_env: advance the envelope one step, for each active voice (X = 3..0): instrument record = ins + number * 32 (operands set by music_init), value =
;   record[2 + mus_pos]. Value > 0 (and register value present): voice on, register $900A+x = mus_reg; value 0: register 0. Then mus_pos + 1; at 16 mus_pos
;   jumps to the loop point (record[1]). Changes A, X, Y, mus_ip and the voice registers.
mus_env:                            ; envelope: voice on (volume value > 0) or off, advance one step
        ldx #3
@v:     lda mus_act,x
        beq @nx
        lda mus_ins,x               ; instrument record = base + number * 32
        asl a
        asl a
        asl a
        asl a
        asl a                       ; * 32 (for instrument >= 8 the carry bit would be lost: at most 8 instruments)
        clc
.const mus_ib_l = *
        adc #0                      ; (operand: low byte of ins, from music_init)
        sta mus_ip
.const mus_ib_h = *
        lda #0
        adc #0
        sta mus_ip+1
        lda mus_pos,x               ; position in the envelope (values start at byte 2 of the record)
        clc
        adc #2
        tay
        lda (mus_ip),y              ; volume value
        beq @quiet
        lda mus_reg,x
        bne @wr
@quiet: lda #0
@wr:    sta $900a,x                 ; register value ($80 | N) switches the voice on, 0 off
        inc mus_pos,x
        lda mus_pos,x
        cmp #16
        bcc @nx
        ldy #1                      ; loop point
        lda (mus_ip),y
        sta mus_pos,x
@nx:    dex
        bpl @v
        rts

; Player variables (all set by music_init; the initial values here only apply until then)
mus_on:    .byte 0                  ; 1 = player running
mus_speed: .byte 6                  ; frames per row (from music_init)
mus_rows:  .byte 32                 ; rows per pattern (from music_init)
mus_tick:  .byte 0                  ; frame within the row (0 = start of row)
mus_row:   .byte 0                  ; current row in the pattern
mus_ord:   .byte 0                  ; position in the order
mus_t:     .byte 0                  ; number of the current pattern
mus_n:     .byte 0                  ; scratch in mus_step: note of the current voice
mus_v:     .byte 0                  ; scratch: current voice 0..3
mus_y:     .byte 0                  ; scratch: Y holder (position in the row)
mus_act:   .byte 0,0,0,0            ; per voice: 1 = sounding (envelope running)
mus_ins:   .byte 0,0,0,0            ; per voice: current instrument number
mus_pos:   .byte 0,0,0,0            ; per voice: step in the envelope (0..15)
mus_reg:   .byte 0,0,0,0            ; per voice: VIC-I register value of the current note ($80 | N)
mus_fpl:   .byte <mus_f_bass,<mus_f_alto,<mus_f_sopr,<mus_f_nois   ; frequency table per voice, low byte
mus_fph:   .byte >mus_f_bass,>mus_f_alto,>mus_f_sopr,>mus_f_nois   ; and high byte
.endmacro
