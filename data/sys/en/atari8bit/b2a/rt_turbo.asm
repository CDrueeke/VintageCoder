// BASIC to assembler (basic2asm), Atari 8-bit, Turbo BASIC XL: DPOKE, DPEEK, PAUSE, MOVE and the helpers for DIV (floating point) and DPEEK.
// Arguments: all but the last on the word stack (rt_push), the last in A/Y.

// DPOKE adr,value: word (low, high) to adr
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

// DPEEK(adr): A/Y = address -> A/Y = word
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

// PAUSE n: wait n frames (steps of the clock RTCLOK+2)
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

// MOVE from,to,count: copy ascending
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

// -MOVE from,to,count: copy descending (for overlapping areas when the destination lies after the source)
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

// Floating-point helpers (only with RT_FLOAT)
.ifdef RT_FLOAT
// Number 0..65535 in A/Y -> FAC (rt_i2f works signed)
rt_u2f:   sty rt_ut
          jsr rt_i2f
          lda rt_ut
          bpl .d
          lda #<rt_f65536
          ldy #>rt_f65536
          jsr F_CONUPK
          jmp F_FADDT
.d:       rts

// FAC -> truncated toward 0 (F_INT rounds down): negative numbers via the absolute value
rt_ftrunc: lda FR0
          bpl F_INT
          and #$7F
          sta FR0
          jsr F_INT
          jmp F_NEGOP

rt_f65536: .byte $42, $06, $55, $36, $00, $00
.endif
rt_ut:    .byte 0
