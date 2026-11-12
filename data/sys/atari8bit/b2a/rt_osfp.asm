// BASIC→Assembler (basic2asm), Atari 8 Bit, Stufe 2: die Einsprünge F_* der gemeinsamen Fließkomma-Laufzeit (common/b2a/rt_fp.asm) als Schicht über dem
// Fließkommapaket des Betriebssystems. FAC = FR0, zweiter Operand = FR1 (6-Byte-BCD). Namen und Bedeutung wie beim C64-BASIC-ROM:
//   F_CONUPK  FR1 = (A niedrig/Y hoch)          F_MOVMF  (X/Y) = FAC
//   F_FADDT, F_FSUBT, F_FMULTT, F_FDIVT, F_FPWRT  FAC = ARG op FAC (ARG liegt nach F_CONUPK in FR1)
//   F_FCOMP   vergleicht FAC mit (A/Y): 0 gleich, 1 FAC größer, $FF FAC kleiner (FAC bleibt erhalten)
//   F_INT, F_ABS, F_SGN, F_NEGOP, F_SQR, F_LOG, F_EXP, F_RND
// Fehler des Pakets (Carry gesetzt) werden zu OVERFLOW, DIVISION BY ZERO oder ILLEGAL QUANTITY.
// Der zweite Operand ist bei den "T"-Einsprüngen das linke Argument: bei Minus und Geteilt werden FR0 und FR1 vorher vertauscht.

F_CONUPK: tax
          jmp FLD1R
F_MOVMF:  jmp FST0R

rt_fswap: ldx #5
.l:       lda FR0,x
          ldy FR1,x
          sta FR1,x
          sty FR0,x
          dex
          bpl .l
          rts

F_FADDT:  jsr FADD
          bcs rt_e_overflow
          rts
F_FMULTT: jsr FMUL
          bcs rt_e_overflow
          rts
F_FSUBT:  jsr rt_fswap
          jsr FSUB
          bcs rt_e_overflow
          rts
F_FDIVT:  jsr rt_fswap
          lda FR1
          beq rt_e_div0
          jsr FDIV
          bcs rt_e_overflow
          rts

F_FCOMP:  tax
          jsr FLD1R
          ldx #<rt_fcs
          ldy #>rt_fcs
          jsr FST0R
          jsr FSUB
          lda FR0+1
          beq .eq
          lda FR0
          bmi .lt
          lda #1
          bne .st
.lt:      lda #$ff
          bne .st
.eq:      lda #0
.st:      sta rt_fcr
          ldx #<rt_fcs
          ldy #>rt_fcs
          jsr FLD0R
          lda rt_fcr
          rts

// INT: abrunden (Richtung -unendlich). Betrag < 1: 0 bzw. -1; sonst die Ziffernpaare hinter dem Komma auf 0; war bei einer negativen Zahl etwas dabei, kommt -1 hinzu
F_INT:    lda FR0
          and #$7f
          sec
          sbc #$40
          bmi .small
          cmp #4
          bcs .ret                 // 5 Paare vor dem Komma: schon ganzzahlig
          tax
          inx
          inx                      // erstes abzuschneidendes Byte
          lda #0
          sta rt_tmp
.z:       lda FR0,x
          ora rt_tmp
          sta rt_tmp
          lda #0
          sta FR0,x
          inx
          cpx #6
          bcc .z
          lda FR0
          bpl .ret
          lda rt_tmp
          beq .ret
          lda #<rt_fm1
          ldy #>rt_fm1
          jsr F_CONUPK
          jmp F_FADDT
.ret:     rts
.small:   lda FR0
          beq .ret
          bmi .neg
          jmp ZFR0
.neg:     lda #<rt_fm1
          ldy #>rt_fm1
          jmp rt_movfm

F_ABS:    lda FR0
          and #$7f
          sta FR0
          rts
F_NEGOP:  lda FR0
          beq .r
          eor #$80
          sta FR0
.r:       rts
F_SGN:    lda FR0
          beq .z
          bmi .n
          lda #<rt_f1
          ldy #>rt_f1
          jmp rt_movfm
.n:       lda #<rt_fm1
          ldy #>rt_fm1
          jmp rt_movfm
.z:       rts

F_LOG:    lda FR0                  // 0 und negative Zahlen: das Paket meldet es nicht immer
          beq .e
          bmi .e
          jsr LOG
          bcs .e
          rts
.e:       jmp rt_e_illegal
F_EXP:    jsr EXP
          bcs rt_e_overflow
          rts

// SQR: 10^(log10(x) / 2), danach ein Newton-Schritt (y = (y + x/y) / 2), damit Quadratzahlen genau herauskommen
F_SQR:    lda FR0
          beq .r
          bmi .e
          ldx #<rt_fpa
          ldy #>rt_fpa
          jsr FST0R                // x
          jsr LOG10
          bcs .e
          lda #<rt_fhalf
          ldy #>rt_fhalf
          jsr F_CONUPK
          jsr FMUL
          jsr EXP10
          bcs rt_e_overflow
          ldx #<rt_fpb
          ldy #>rt_fpb
          jsr FST0R                // y
          jsr FMOVE                // FR1 = y
          lda #<rt_fpa
          ldy #>rt_fpa
          jsr rt_movfm             // FR0 = x
          jsr FDIV                 // x / y
          bcs rt_e_overflow
          lda #<rt_fpb
          ldy #>rt_fpb
          jsr F_CONUPK
          jsr FADD
          lda #<rt_fhalf
          ldy #>rt_fhalf
          jsr F_CONUPK
          jmp FMUL
.r:       rts
.e:       jmp rt_e_illegal

// RND: Zufallszahl 0 <= r < 1 aus POKEY (der Wert des Arguments spielt keine Rolle): fünf Ziffernpaare 00-99, danach normalisiert
F_RND:    lda #$3F
          sta FR0
          ldx #1
.p:       lda $D20A
.m:       cmp #100
          bcc .d
          sbc #100
          bcs .m
.d:       ldy #0
.t:       cmp #10
          bcc .u
          sbc #10
          iny
          bne .t
.u:       sta rt_tmp
          tya
          asl
          asl
          asl
          asl
          ora rt_tmp
          sta FR0,x
          inx
          cpx #6
          bcc .p
          ldy #5
.n:       lda FR0+1
          bne .ok
          lda FR0
          and #$7f
          cmp #$3B
          bcc .zero
          lda FR0
          sec
          sbc #1
          sta FR0
          ldx #1
.s:       lda FR0+1,x
          sta FR0,x
          inx
          cpx #5
          bcc .s
          lda #0
          sta FR0+5
          dey
          bne .n
.zero:    jmp ZFR0
.ok:      rts

// Potenz: FAC = ARG ^ FAC (ARG in FR1). Wie Atari-BASIC: positive ganze Exponenten bis 255 durch wiederholtes Multiplizieren (genau), alles andere über
// 10^(b * log10(a)) (2^-2 ergibt dort 0.250000002); negative Basis nur mit ganzem Exponenten
F_FPWRT:  ldx #<rt_fpb
          ldy #>rt_fpb
          jsr FST0R                // b
          jsr rt_fswap
          ldx #<rt_fpa
          ldy #>rt_fpa
          jsr FST0R                // a
          lda rt_fpb
          beq .one                 // x^0 = 1
          lda FR0
          bne .nz
          lda rt_fpb               // 0^b: b < 0 ist eine Division durch 0, sonst 0
          bpl .zero
          jmp rt_e_div0
.zero:    jmp ZFR0
.one:     lda #<rt_f1
          ldy #>rt_f1
          jmp rt_movfm
.nz:      lda #<rt_fpb             // Exponent ganzzahlig? FAC = INT(b), gleich b?
          ldy #>rt_fpb
          jsr rt_movfm
          lda FR0
          and #$7f
          sta FR0                  // |b|
          jsr F_INT
          ldx #<rt_fpn
          ldy #>rt_fpn
          jsr FST0R                // n = |b| abgerundet
          lda rt_fpb
          and #$7f
          sta rt_fpc               // |b| als Vergleichswert zurück
          ldx #1
.cp:      lda rt_fpb,x
          sta rt_fpc,x
          inx
          cpx #6
          bcc .cp
          lda #<rt_fpc
          ldy #>rt_fpc
          jsr F_FCOMP              // FAC = n gegen |b|
          bne .real
          lda #<rt_fpn
          ldy #>rt_fpn
          jsr rt_movfm
          jsr rt_f2addr            // n als Zahl
          cpy #0
          bne .real
          sta rt_fpk
          lda rt_fpb
          bmi .negint              // negativer ganzer Exponent: über Logarithmus
          lda #<rt_f1              // Ergebnis = 1
          ldy #>rt_f1
          jsr rt_movfm
          ldx #<rt_fpr
          ldy #>rt_fpr
          jsr FST0R
.sq:      lsr rt_fpk
          bcc .nm
          lda #<rt_fpa
          ldy #>rt_fpa
          jsr rt_movfm
          lda #<rt_fpr
          ldy #>rt_fpr
          jsr F_CONUPK
          jsr F_FMULTT
          ldx #<rt_fpr
          ldy #>rt_fpr
          jsr FST0R
.nm:      lda rt_fpk
          beq .done
          lda #<rt_fpa
          ldy #>rt_fpa
          jsr rt_movfm
          lda #<rt_fpa
          ldy #>rt_fpa
          jsr F_CONUPK
          jsr F_FMULTT
          ldx #<rt_fpa
          ldy #>rt_fpa
          jsr FST0R
          jmp .sq
.done:    lda #<rt_fpr
          ldy #>rt_fpr
          jmp rt_movfm
.negint:  lda rt_fpa
          bmi .negbase
          jmp .real
.negbase: lda rt_fpk               // negative Basis, negativer ganzer Exponent: Vorzeichen nach der Parität
          and #1
          sta rt_fps
          lda rt_fpa
          and #$7f
          sta rt_fpa
          jmp .rl2
.real:    lda #0
          sta rt_fps
          lda rt_fpa
          bmi .ill                 // negative Basis mit gebrochenem Exponenten
.rl2:     lda #<rt_fpa
          ldy #>rt_fpa
          jsr rt_movfm
          jsr LOG10
          bcs .ill
          lda #<rt_fpb
          ldy #>rt_fpb
          jsr F_CONUPK
          jsr FMUL
          bcs rt_e_overflow
          jsr EXP10
          bcs rt_e_overflow
          lda rt_fps
          beq .rt
          lda FR0
          beq .rt
          eor #$80
          sta FR0
.rt:      rts
.ill:     jmp rt_e_illegal

rt_fhalf: .byte $3F, $50, 0, 0, 0, 0
rt_fcs:   .byte 0, 0, 0, 0, 0, 0
rt_fcr:   .byte 0
rt_fpa:   .byte 0, 0, 0, 0, 0, 0
rt_fpb:   .byte 0, 0, 0, 0, 0, 0
rt_fpc:   .byte 0, 0, 0, 0, 0, 0
rt_fpn:   .byte 0, 0, 0, 0, 0, 0
rt_fpr:   .byte 0, 0, 0, 0, 0, 0
rt_fpk:   .byte 0
rt_fps:   .byte 0

// ---- SIN, COS, ATN: wie das BASIC-Modul (Näherungen mit PLYEVL des OS-Pakets; Schritt für Schritt nachgebaut, damit die Ziffern stimmen)
// SIN/COS: |x| durch Viertelkreis (Pi/2 oder 90 Grad) teilen, Quadrant bestimmen, Rest f (0..1) oder 1-f, sin(Pi/2 * f) = f * P(f^2) mit sechs Koeffizienten.
// ATN: |x| >= 1: (x-c)/(x+c) mit c = 0,9999999999, danach + Pi/4; Näherung y * P(y^2) mit elf Koeffizienten. Der Winkelmodus steht wie im BASIC-Modul in $FB
// (0 = Bogenmaß, 6 = Grad; DEG und RAD setzen ihn über POKE 251). Die Zahlentafeln sind die des BASIC-Moduls (Atari-BASIC Rev. C) und des OS-Pakets.
F_SIN:    lda #$04
          bit FR0
          bpl rt_trig
          lda #$02
          bne rt_trig
F_COS:    lda #$01
rt_trig:  sta rt_tf0
          lda FR0
          and #$7F
          sta FR0
          clc
          lda #<rt_tpi2
          adc $FB
          tax
          lda #>rt_tpi2
          adc #0
          tay
          jsr FLD1R
          jsr FDIV
          bcs .err
          lda FR0
          and #$7F
          sec
          sbc #$40
          bmi .small
          cmp #4
          bpl .err
          tax
          lda FR0+1,x
          sta rt_tf1
          and #$10
          beq .a
          lda #2
.a:       clc
          adc rt_tf1
          and #3
          adc rt_tf0
          sta rt_tf0
          stx rt_tf1
          jsr FMOVE
          ldx rt_tf1
          lda #0
.z:       sta FR1+2,x
          inx
          cpx #3
          bcc .z
          jsr FSUB
.small:   lsr rt_tf0
          bcc .b
          jsr FMOVE
          ldx #<rt_tone
          ldy #>rt_tone
          jsr FLD0R
          jsr FSUB
.b:       ldx #<rt_ttmp
          ldy #>rt_ttmp
          jsr FST0R
          jsr FMOVE
          jsr FMUL
          bcs .err
          lda #6
          ldx #<rt_tsin
          ldy #>rt_tsin
          jsr PLYEVL
          ldx #<rt_ttmp
          ldy #>rt_ttmp
          jsr FLD1R
          jsr FMUL
          bcs .err
          lsr rt_tf0
          bcc .r
          lda FR0
          beq .r
          eor #$80
          sta FR0
.r:       rts
.err:     jmp rt_e_illegal

F_ATN:    lda #0
          sta rt_tf0
          sta rt_tf1
          lda FR0
          and #$7F
          cmp #$40
          bmi .lt1
          lda FR0
          and #$80
          sta rt_tf0
          inc rt_tf1
          lda FR0
          and #$7F
          sta FR0
          ldx #<rt_ttx
          ldy #>rt_ttx
          jsr FST0R                // x
          ldx #<rt_tc
          ldy #>rt_tc
          jsr FLD1R
          jsr FADD                 // x + c
          ldx #<rt_ttmp
          ldy #>rt_ttmp
          jsr FST0R
          ldx #<rt_ttx
          ldy #>rt_ttx
          jsr FLD0R
          ldx #<rt_tc
          ldy #>rt_tc
          jsr FLD1R
          jsr FSUB                 // x - c
          ldx #<rt_ttmp
          ldy #>rt_ttmp
          jsr FLD1R
          jsr FDIV                 // (x - c) / (x + c)
.lt1:     ldx #<rt_ttmp
          ldy #>rt_ttmp
          jsr FST0R
          jsr FMOVE
          jsr FMUL
          bcs .err
          lda #11
          ldx #<rt_tatn
          ldy #>rt_tatn
          jsr PLYEVL
          bcs .err
          ldx #<rt_ttmp
          ldy #>rt_ttmp
          jsr FLD1R
          jsr FMUL
          bcs .err
          lda rt_tf1
          beq .nf
          ldx #<rt_tpi4
          ldy #>rt_tpi4
          jsr FLD1R
          jsr FADD
          lda rt_tf0
          ora FR0
          sta FR0
.nf:      lda $FB
          beq .r
          ldx #<rt_tr2d
          ldy #>rt_tr2d
          jsr FLD1R
          jsr FDIV
.r:       rts
.err:     jmp rt_e_illegal

rt_tf0:   .byte 0
rt_tf1:   .byte 0
rt_ttmp:  .byte 0, 0, 0, 0, 0, 0
rt_ttx:   .byte 0, 0, 0, 0, 0, 0
rt_tsin:  .byte $BD, $03, $55, $14, $99, $39
          .byte $3E, $01, $60, $44, $27, $52
          .byte $BE, $46, $81, $75, $43, $55
          .byte $3F, $07, $96, $92, $62, $39
          .byte $BF, $64, $59, $64, $08, $67
rt_tpi2:  .byte $40, $01, $57, $07, $96, $32
rt_t90:   .byte $40, $90, $00, $00, $00, $00
rt_tr2d:  .byte $3F, $01, $74, $53, $29, $25
rt_tone:  .byte $40, $01, $00, $00, $00, $00
rt_tatn:  .byte $3E, $16, $05, $44, $49, $00
          .byte $BE, $95, $68, $38, $45, $00
          .byte $3F, $02, $68, $79, $94, $16
          .byte $BF, $04, $92, $78, $90, $80
          .byte $3F, $07, $03, $15, $20, $00
          .byte $BF, $08, $92, $29, $12, $44
          .byte $3F, $11, $08, $40, $09, $11
          .byte $BF, $14, $28, $31, $56, $04
          .byte $3F, $19, $99, $98, $77, $44
          .byte $BF, $33, $33, $33, $31, $13
rt_tc:    .byte $3F, $99, $99, $99, $99, $99
rt_tpi4:  .byte $3F, $78, $53, $98, $16, $34
