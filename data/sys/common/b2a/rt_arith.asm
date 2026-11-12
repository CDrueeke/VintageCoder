// BASIC→Assembler, Laufzeit C64: Ganzzahl-Arithmetik (16 Bit, Zweierkomplement, Überlauf läuft um), Vergleiche, PEEK/POKE/SYS
// Verlangt: rt_core.asm

#include <common/math.asm>

rt_add:   sta rt_t1
          sty rt_t1+1
          jsr rt_pop0
          clc
          lda rt_t0
          adc rt_t1
          pha
          lda rt_t0+1
          adc rt_t1+1
          tay
          pla
          rts

rt_sub:   sta rt_t1
          sty rt_t1+1
          jsr rt_pop0
          sec
          lda rt_t0
          sbc rt_t1
          pha
          lda rt_t0+1
          sbc rt_t1+1
          tay
          pla
          rts

rt_mul:   sta rt_t1
          sty rt_t1+1
          jsr rt_pop0
          lda rt_t0
          sta MATH_A
          lda rt_t0+1
          sta MATH_A+1
          lda rt_t1
          sta MATH_B
          lda rt_t1+1
          sta MATH_B+1
          jsr mul16
          lda MATH_R+1
          tay
          lda MATH_R
          rts

// ganzzahlige Division mit Vorzeichen (nur mit Option --int-div erzeugt)
rt_div:   sta rt_t1
          sty rt_t1+1
          jsr rt_pop0
          lda rt_t1
          ora rt_t1+1
          beq rt_e_div0
          lda rt_t0+1
          eor rt_t1+1
          sta rt_tmp
          lda rt_t0+1
          bpl .a
          neg16(rt_t0)
.a:       lda rt_t1+1
          bpl .b
          neg16(rt_t1)
.b:       lda rt_t0
          sta MATH_A
          lda rt_t0+1
          sta MATH_A+1
          lda rt_t1
          sta MATH_B
          lda rt_t1+1
          sta MATH_B+1
          jsr div16
          lda MATH_A+1
          tay
          lda MATH_A
          bit rt_tmp
          bpl .r
          jmp rt_neg
.r:       rts

.ifdef B2A_ATARI
// Atari-BASIC: AND, OR und NOT sind logisch (Ergebnis 1 oder 0), Vergleiche liefern 1
rt_and:   sta rt_t1
          sty rt_t1+1
          jsr rt_pop0
          lda rt_t0
          ora rt_t0+1
          beq rt_false
          lda rt_t1
          ora rt_t1+1
          beq rt_false
          jmp rt_true
rt_or:    sta rt_t1
          sty rt_t1+1
          jsr rt_pop0
          lda rt_t0
          ora rt_t0+1
          ora rt_t1
          ora rt_t1+1
          beq rt_false
          jmp rt_true
.else
rt_and:   sta rt_t1
          sty rt_t1+1
          jsr rt_pop0
          lda rt_t0
          and rt_t1
          pha
          lda rt_t0+1
          and rt_t1+1
          tay
          pla
          rts

rt_or:    sta rt_t1
          sty rt_t1+1
          jsr rt_pop0
          lda rt_t0
          ora rt_t1
          pha
          lda rt_t0+1
          ora rt_t1+1
          tay
          pla
          rts

.endif

rt_neg:   pha
          tya
          eor #$ff
          tay
          pla
          eor #$ff
          clc
          adc #1
          bne .d
          iny
.d:       rts

.ifdef B2A_ATARI
rt_not:   sta rt_t0
          tya
          ora rt_t0
          beq rt_true
          jmp rt_false
.else
rt_not:   eor #$ff
          pha
          tya
          eor #$ff
          tay
          pla
          rts

.endif

rt_abs:   cpy #$80
          bcc .p
          jmp rt_neg
.p:       rts

rt_sgn:   sta rt_t0
          tya
          bmi .m
          ora rt_t0
          beq .z
          lda #1
          ldy #0
          rts
.z:       lda #0
          ldy #0
          rts
.m:       lda #$ff
          tay
          rts

// ---- Vergleiche: Ergebnis wahr = $FFFF (-1), falsch = 0
// rt_cmp: A = 0 gleich, 1 links groesser, $FF links kleiner (vorzeichenbehaftet)
rt_cmp:   sta rt_t1
          sty rt_t1+1
          jsr rt_pop0
          lda rt_t0
          cmp rt_t1
          bne .ne
          lda rt_t0+1
          cmp rt_t1+1
          bne .ne
          lda #0
          rts
.ne:      lda rt_t0
          cmp rt_t1
          lda rt_t0+1
          sbc rt_t1+1
          bvc .v
          eor #$80
.v:       bmi .lt
          lda #1
          rts
.lt:      lda #$ff
          rts

.ifdef B2A_ATARI
rt_true:  lda #1
          ldy #0
          rts
.else
rt_true:  lda #$ff
          tay
          rts
.endif
rt_false: lda #0
          tay
          rts

rt_eq:    jsr rt_cmp
          beq rt_true
          bne rt_false
rt_ne:    jsr rt_cmp
          bne rt_true
          beq rt_false
rt_lt:    jsr rt_cmp
          cmp #$ff
          beq rt_true
          bne rt_false
rt_gt:    jsr rt_cmp
          cmp #1
          beq rt_true
          bne rt_false
rt_le:    jsr rt_cmp
          cmp #1
          bne rt_true
          beq rt_false
rt_ge:    jsr rt_cmp
          cmp #$ff
          bne rt_true
          beq rt_false

// ---- Speicherzugriff
rt_byte:  cpy #0              // Wert muss 0..255 sein
          bne rt_e_illegal
          rts

// TI: untere 16 Bit der Systemuhr nach A/Y
rt_ti:    jsr RDTIM
          sta rt_tmp
          txa
          tay
          lda rt_tmp
          rts

rt_peek:  sta rt_t0
          sty rt_t0+1
          ldy #0
          lda (rt_t0),y
          rts

// POKE: Adresse auf dem Stapel, Wert in A/Y
rt_poke:  cpy #0
          bne rt_e_illegal
          sta rt_tmp
          jsr rt_pop0
          ldy #0
          lda rt_tmp
          sta (rt_t0),y
          rts

rt_sys:   sta rt_t0
          sty rt_t0+1
          jmp (rt_t0)

// WAIT: Adresse und Maske auf dem Stapel, Xor-Wert in A
rt_wait:  sta rt_tmp
          jsr rt_pop0
          lda rt_t0
          sta rt_v
          jsr rt_pop0
.l:       ldy #0
          lda (rt_t0),y
          eor rt_tmp
          and rt_v
          beq .l
          rts

math_routines
