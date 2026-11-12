// BASIC→Assembler (basic2asm), Atari 8 Bit, Turbo BASIC XL: DPOKE, DPEEK, PAUSE, MOVE und die Hilfen für DIV (Fließkomma) und DPEEK.
// Argumente: alle bis auf das letzte auf dem Wortestapel (rt_push), das letzte in A/Y.

// DPOKE adr,wert: Wort (niedrig, hoch) nach adr
rt_dpoke: sta rt_v
          sty rt_v+1
          jsr rt_pop0
          ldy #0
          lda rt_v
          sta (rt_t0),y
          iny
          lda rt_v+1
          sta (rt_t0),y
          rts

// DPEEK(adr): A/Y = Adresse -> A/Y = Wort
rt_dpeek: sta rt_t0
          sty rt_t0+1
          ldy #0
          lda (rt_t0),y
          pha
          iny
          lda (rt_t0),y
          tay
          pla
          rts

// PAUSE n: n Bildwechsel (Schritte der Uhr RTCLOK+2) warten
rt_pause: sta rt_v
          sty rt_v+1
.l:       lda rt_v
          ora rt_v+1
          beq .done
          lda RTCLOK+2
.w:       cmp RTCLOK+2
          beq .w
          lda rt_v
          bne .a
          dec rt_v+1
.a:       dec rt_v
          jmp .l
.done:    rts

// MOVE von,nach,anzahl: aufsteigend kopieren
rt_move:  sta rt_v
          sty rt_v+1
          jsr rt_pop0
          lda rt_t0
          sta rt_s2
          lda rt_t0+1
          sta rt_s2+1
          jsr rt_pop0
          lda rt_t0
          sta rt_s1
          lda rt_t0+1
          sta rt_s1+1
.l:       lda rt_v
          ora rt_v+1
          beq .done
          ldy #0
          lda (rt_s1),y
          sta (rt_s2),y
          inc rt_s1
          bne .a
          inc rt_s1+1
.a:       inc rt_s2
          bne .b
          inc rt_s2+1
.b:       lda rt_v
          bne .c
          dec rt_v+1
.c:        dec rt_v
          jmp .l
.done:    rts

// -MOVE von,nach,anzahl: absteigend kopieren (für überlappende Bereiche, wenn das Ziel hinter der Quelle liegt)
rt_movd:  sta rt_v
          sty rt_v+1
          jsr rt_pop0
          clc
          lda rt_t0
          adc rt_v
          sta rt_s2
          lda rt_t0+1
          adc rt_v+1
          sta rt_s2+1
          jsr rt_pop0
          clc
          lda rt_t0
          adc rt_v
          sta rt_s1
          lda rt_t0+1
          adc rt_v+1
          sta rt_s1+1
.l:       lda rt_v
          ora rt_v+1
          beq .done
          lda rt_s1
          bne .a
          dec rt_s1+1
.a:       dec rt_s1
          lda rt_s2
          bne .b
          dec rt_s2+1
.b:       dec rt_s2
          ldy #0
          lda (rt_s1),y
          sta (rt_s2),y
          lda rt_v
          bne .c
          dec rt_v+1
.c:       dec rt_v
          jmp .l
.done:    rts

// Fließkomma-Hilfen (nur mit RT_FLOAT)
.ifdef RT_FLOAT
// Zahl 0..65535 in A/Y -> FAC (rt_i2f rechnet vorzeichenbehaftet)
rt_u2f:   sty rt_ut
          jsr rt_i2f
          lda rt_ut
          bpl .d
          lda #<rt_f65536
          ldy #>rt_f65536
          jsr F_CONUPK
          jmp F_FADDT
.d:       rts

// FAC -> Richtung 0 abgeschnitten (F_INT rundet nach unten): negative Zahlen über den Betrag
rt_ftrunc: lda FR0
          bpl F_INT
          and #$7F
          sta FR0
          jsr F_INT
          jmp F_NEGOP

rt_f65536: .byte $42, $06, $55, $36, $00, $00
.endif
rt_ut:    .byte 0
