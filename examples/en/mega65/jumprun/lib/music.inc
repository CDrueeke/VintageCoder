; music.inc - C64 (adapted for jumprun: voice 3 pauses during an effect): player routine for tracker songs (SID, 3 voices) from the export.
; GENERATED from music.inc.in by tools/gen_music.py (frequency tables) - make changes in the .in file.
;
;   .include "lib/music.inc"
;   music_init tune_order, tune_patterns, tune_ins, tune_speed, tune_rows, tune_patsize   ; labels/constants from the export
;   ...                                   ; every screen frame (50/60 Hz, e.g. in the IRQ):
;   jsr music_play
;   music_stop                            ; gates off, volume 0
;   music_runtime                         ; ONCE, behind the main program (not executed)
;
; Frequency table PAL; with -D NTSC=1 (or .const NTSC = 1 before the include) the NTSC table.
; MEGA65 version for "Hugo & Gumbo": writes to the first SID ($D400-$D418) as on the C64; the HAL's sound effects
; use the second SID ($D420), so the sfx_on check in voice 3 is dropped (see @keep).
; Zero page used: $F7-$FA. State: mus_row, mus_ord, mus_tick (memory); mus_sid = image of the SID registers ($D400-$D418).
; Instruments: at most 32 (SID record 8 bytes). Notes above about A#6 are too high for the 16-bit frequency counter (upper limit 65535).

.const mus_ptr = $f7            ; 16 bit: pointer to the current pattern row
.const mus_ip  = $f9            ; 16 bit: pointer to the instrument record

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

.macro music_stop
        lda #0
        sta mus_on
        sta $d404
        sta $d40b
        sta $d412
        sta $d418
.endmacro

.macro music_runtime
music_play:                         ; call once per frame (here from the raster interrupt at the end of the frame)
        lda mus_on
        beq @rts
        lda mus_tick                ; play a new pattern row only when the counter is 0 (every mus_speed frames)
        bne @t
        jsr mus_step
@t:     inc mus_tick
        lda mus_tick
        cmp mus_speed
        bcc @rts
        lda #0
        sta mus_tick
@rts:   rts

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
@row:   ldx #0                      ; voice 0..2 (3 voices per row with 2 bytes each: note, instrument)
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
@keep:                              ; (MEGA65: sound effects run on the second SID, voice 3 does not need to be paused;
                                    ;  the C64 version checks here whether voice 3 currently belongs to an effect, and then skips it)
@go:    lda mus_n
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
        lda mus_flo,y
        sta mus_sid,x
        sta $d400,x                 ; frequency lo into the FIRST SID ($D400); mus_sid is a copy of the registers
        lda mus_fhi,y
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
        sta mus_sid+4,x             ; gate off (restart the envelope) ...
        sta $d404,x
        ora #1                      ; ... and on again
        sta mus_sid+4,x
        sta $d404,x
        jmp @next
@off:   lda mus_voff,x              ; note $FF = tone off: only clear the gate, the envelope fades out
        tax
        lda mus_sid+4,x
        and #$fe
        sta mus_sid+4,x
        sta $d404,x
@next:  inc mus_v                    ; next voice; after the third the pattern row is done
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
mus_voff:  .byte 0,7,14               ; register offset of voices 1-3 in the SID (7 bytes each)
mus_sid:   .fill 25, 0
.ifdef NTSC
mus_flo:
        .byte $00,$0c,$1c,$2d,$3f,$52,$66,$7b,$92,$aa,$c3,$de
        .byte $fa,$18,$38,$5a,$7e,$a4,$cc,$f7,$24,$54,$86,$bc
        .byte $f5,$31,$71,$b4,$fc,$48,$98,$ed,$48,$a7,$0c,$78
        .byte $e9,$62,$e2,$69,$f8,$90,$30,$db,$8f,$4e,$19,$f0
        .byte $d3,$c4,$c3,$d1,$f0,$1f,$61,$b6,$1e,$9d,$32,$df
        .byte $a6,$88,$86,$a3,$e0,$3f,$c2,$6b,$3d,$3a,$64,$be
        .byte $4c,$0f,$0c,$46,$bf,$7d,$84,$d6,$7a,$73,$c8,$7d
        .byte $97,$1e,$18,$8b,$7f,$fb,$07,$ac,$f4,$e7,$8f,$f9
        .byte $2f
mus_fhi:
        .byte $00,$01,$01,$01,$01,$01,$01,$01,$01,$01,$01,$01
        .byte $01,$02,$02,$02,$02,$02,$02,$02,$03,$03,$03,$03
        .byte $03,$04,$04,$04,$04,$05,$05,$05,$06,$06,$07,$07
        .byte $07,$08,$08,$09,$09,$0a,$0b,$0b,$0c,$0d,$0e,$0e
        .byte $0f,$10,$11,$12,$13,$15,$16,$17,$19,$1a,$1c,$1d
        .byte $1f,$21,$23,$25,$27,$2a,$2c,$2f,$32,$35,$38,$3b
        .byte $3f,$43,$47,$4b,$4f,$54,$59,$5e,$64,$6a,$70,$77
        .byte $7e,$86,$8e,$96,$9f,$a8,$b3,$bd,$c8,$d4,$e1,$ee
        .byte $fd
.else
mus_flo:
        .byte $00,$16,$27,$39,$4b,$5f,$74,$8a,$a1,$ba,$d4,$f0
        .byte $0e,$2d,$4e,$71,$96,$be,$e7,$14,$42,$74,$a9,$e0
        .byte $1b,$5a,$9c,$e2,$2d,$7b,$cf,$27,$85,$e8,$51,$c1
        .byte $37,$b4,$38,$c4,$59,$f7,$9d,$4e,$0a,$d0,$a2,$81
        .byte $6d,$67,$70,$89,$b2,$ed,$3b,$9c,$13,$a0,$45,$02
        .byte $da,$ce,$e0,$11,$64,$da,$76,$39,$26,$40,$89,$04
        .byte $b4,$9c,$c0,$23,$c8,$b4,$eb,$72,$4c,$80,$12,$08
        .byte $68,$39,$80,$45,$90,$68,$d6,$e3,$99,$00,$24,$10
        .byte $ff
mus_fhi:
        .byte $00,$01,$01,$01,$01,$01,$01,$01,$01,$01,$01,$01
        .byte $02,$02,$02,$02,$02,$02,$02,$03,$03,$03,$03,$03
        .byte $04,$04,$04,$04,$05,$05,$05,$06,$06,$06,$07,$07
        .byte $08,$08,$09,$09,$0a,$0a,$0b,$0c,$0d,$0d,$0e,$0f
        .byte $10,$11,$12,$13,$14,$15,$17,$18,$1a,$1b,$1d,$1f
        .byte $20,$22,$24,$27,$29,$2b,$2e,$31,$34,$37,$3a,$3e
        .byte $41,$45,$49,$4e,$52,$57,$5c,$62,$68,$6e,$75,$7c
        .byte $83,$8b,$93,$9c,$a5,$af,$b9,$c4,$d0,$dd,$ea,$f8
        .byte $ff
.endif
.endmacro
