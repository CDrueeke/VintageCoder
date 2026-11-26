// BASIC to assembler (basic2asm), Atari 8-bit: OPEN #n,aux1,aux2,"name" (needs the string runtime rt_astr.asm and rt_gfx.asm for rt_chx and rt_cio_err)

// OPEN: stack (top): aux2, aux1, channel; name in the string accumulator (rt_sal, rt_sap)
rt_open:  lda rt_sap
          sta rt_s1
          lda rt_sap+1
          sta rt_s1+1
          lda rt_sal
          cmp #60
          bcc .l
          lda #60
.l:       sta rt_cv_n
          ldy #0
.c:       cpy rt_cv_n
          beq .e
          lda (rt_s1),y
          sta rt_fnbuf,y
          iny
          bne .c
.e:       lda #$9B
          sta rt_fnbuf,y
          jsr rt_pop0
          lda rt_t0
          sta rt_cv_a2
          jsr rt_pop0
          lda rt_t0
          sta rt_cv_a1
          jsr rt_pop0
          lda rt_t0
          jsr rt_chx
          lda #CIO_OPEN
          sta ICCOM,x
          lda #<rt_fnbuf
          sta ICBAL,x
          lda #>rt_fnbuf
          sta ICBAL+1,x
          lda rt_cv_a1
          sta ICAX1,x
          lda rt_cv_a2
          sta ICAX2,x
          lda #0
          sta ICBLL,x
          sta ICBLL+1,x
          jsr CIOV
          tya
          jmp rt_cio_err

rt_cv_a2o: .byte 0
