; music.inc – C64 (angepasst fuer jumprun: Stimme 3 setzt waehrend eines Effekts aus): Spielroutine für Tracker-Songs (SID, 3 Stimmen) aus dem Export.
; ERZEUGT aus music.inc.in durch tools/gen_music.py (Frequenztabellen) – Änderungen in der .in-Datei vornehmen.
;
;   .include "lib/music.inc"
;   music_init tune_order, tune_patterns, tune_ins, tune_speed, tune_rows, tune_patsize   ; Labels/Konstanten aus dem Export
;   ...                                   ; jede Bildschirmzeile (50/60 Hz, z. B. im IRQ):
;   jsr music_play
;   music_stop                            ; Gates aus, Lautstärke 0
;   music_runtime                         ; EINMAL, hinter dem Hauptprogramm (nicht ausgeführt)
;
; Frequenztabelle PAL; mit -D NTSC=1 (oder .const NTSC = 1 vor dem Include) die NTSC-Tabelle.
; MEGA65-Fassung fuer "Hugo & Gumbo": schreibt wie am C64 in den ersten SID ($D400-$D418); die Klangeffekte der HAL
; benutzen den zweiten SID ($D420), daher entfaellt die Abfrage von sfx_on in Stimme 3 (siehe @keep).
; Belegte Zeropage: $F7-$FA. Zustand: mus_row, mus_ord, mus_tick (Speicher); mus_sid = Abbild der SID-Register ($D400-$D418).
; Instrumente: höchstens 32 (SID-Datensatz 8 Bytes). Noten über etwa A#6 sind für den 16-Bit-Frequenzzähler zu hoch (Obergrenze 65535).

.const mus_ptr = $f7            ; 16 Bit: Zeiger auf die aktuelle Patternzeile
.const mus_ip  = $f9            ; 16 Bit: Zeiger auf den Instrumentendatensatz

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
music_play:                         ; einmal pro Bild aufrufen (hier vom Raster-Interrupt am Bildende)
        lda mus_on
        beq @rts
        lda mus_tick                ; nur bei Zaehlerstand 0 (alle mus_speed Bilder) eine neue Patternzeile spielen
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
        ldx mus_ord                 ; neues Pattern: Reihenfolge lesen ($ff = Ende, danach die Wiederholungsposition)
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
@row:   ldx #0                      ; Stimme 0..2 (je Zeile 3 Stimmen mit je 2 Byte: Note, Instrument)
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
@keep:                              ; (MEGA65: Klangeffekte laufen auf dem zweiten SID, Stimme 3 muss nicht ausgesetzt werden;
                                    ;  die C64-Fassung prueft hier, ob Stimme 3 gerade einem Effekt gehoert, und ueberspringt sie dann)
@go:    lda mus_n
        beq @next
        cmp #$ff
        beq @off
        lda mus_voff,x              ; Registerversatz der Stimme
        sta mus_r
        lda mus_ins,x               ; Instrumentendatensatz = Basis + Nummer * 8
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
        sta $d400,x                 ; Frequenz lo in den ERSTEN SID ($D400); mus_sid ist eine Abschrift der Register
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
        sta mus_sid+4,x             ; Gate aus (Neustart der Hüllkurve) …
        sta $d404,x
        ora #1                      ; … und wieder an
        sta mus_sid+4,x
        sta $d404,x
        jmp @next
@off:   lda mus_voff,x              ; Note $FF = Ton aus: nur das Gate loeschen, die Huellkurve klingt aus
        tax
        lda mus_sid+4,x
        and #$fe
        sta mus_sid+4,x
        sta $d404,x
@next:  inc mus_v                    ; naechste Stimme; nach der dritten ist die Patternzeile fertig
        ldx mus_v
        ldy mus_y
        cpx #3
        beq @rowend
        jmp @voice
@rowend:
        clc                         ; Zeiger auf die nächste Zeile (3 Stimmen * 2 Bytes)
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
mus_voff:  .byte 0,7,14               ; Registerversatz der Stimmen 1-3 im SID (je 7 Byte)
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
