// BASIC to assembler (basic2asm), Atari 8-bit, Turbo BASIC XL: string words HEX$, DEC, INSTR, INKEY$, TIME$ (measured on a real Turbo BASIC XL).
// Requires: rt_astr.asm (strings, rt_tbuf), rt_turbo.asm (rt_u2f, rt_i2f)

// ---- HEX$(n): A/Y = 0..65535 -> two digits if n < 256, otherwise four (uppercase letters, leading zeros up to that)
rt_hexs:  sta rt_v
          sty rt_v+1
          ldx #0
          lda rt_v+1
          beq .lo
          jsr rt_hx
.lo:      lda rt_v
          jsr rt_hx
          stx rt_sal
          jmp rt_tbufacc
rt_hx:    pha
          lsr
          lsr
          lsr
          lsr
          jsr rt_hn
          pla
          and #$0F
rt_hn:    cmp #10
          bcc .d
          adc #6
.d:       adc #'0'
          sta rt_tbuf,x
          inx
          rts

// The accumulator points to rt_tbuf (length is in rt_sal, high byte is cleared)
rt_tbufacc: lda #0
          sta rt_sal+1
          lda #<rt_tbuf
          sta rt_sap
          lda #>rt_tbuf
          sta rt_sap+1
          rts

// ---- DEC(a$): hex number at the start (0-9, A-F), ends at the first other character; with more than four digits the value overflows 16 bits (as in the original)
rt_dec:   lda #0
          sta rt_v
          sta rt_v+1
          lda rt_sap
          sta rt_s1
          lda rt_sap+1
          sta rt_s1+1
          lda rt_sal
          sta rt_cn
          lda rt_sal+1
          sta rt_cn+1
.l:       lda rt_cn
          ora rt_cn+1
          beq .ok
          ldy #0
          lda (rt_s1),y
          sec
          sbc #'0'
          cmp #10
          bcc .dig
          sbc #7
          cmp #10
          bcc .ok
          cmp #16
          bcs .ok
.dig:     pha
          asl rt_v
          rol rt_v+1
          asl rt_v
          rol rt_v+1
          asl rt_v
          rol rt_v+1
          asl rt_v
          rol rt_v+1
          pla
          ora rt_v
          sta rt_v
          inc rt_s1
          bne .a
          inc rt_s1+1
.a:       lda rt_cn
          bne .b
          dec rt_cn+1
.b:       dec rt_cn
          jmp .l
.ok:      lda rt_v
          ldy rt_v+1
          rts

// ---- INSTR(a$,b$[,n]): stack: a$ (length, pointer), b$ (length, pointer); A/Y = number of characters to skip (n, measured on the original).
// Result: position from 1 or 0 (A/Y); empty b$ or negative n: 0, or n treated as 0
rt_instr: sta rt_ik
          sty rt_ik+1
          jsr rt_spop2
          lda rt_pp
          sta rt_ibp
          lda rt_pp+1
          sta rt_ibp+1
          lda rt_pl
          sta rt_ibl
          lda rt_pl+1
          sta rt_ibl+1
          jsr rt_spop2
          lda rt_ik+1
          bpl .k
          lda #0
          sta rt_ik
          sta rt_ik+1
.k:       lda rt_ibl
          ora rt_ibl+1
          beq .none
          sec
          lda rt_pl
          sbc rt_ibl
          sta rt_il
          lda rt_pl+1
          sbc rt_ibl+1
          sta rt_il+1
          bcc .none
.try:     lda rt_il
          cmp rt_ik
          lda rt_il+1
          sbc rt_ik+1
          bcc .none
          clc
          lda rt_pp
          adc rt_ik
          sta rt_s1
          lda rt_pp+1
          adc rt_ik+1
          sta rt_s1+1
          lda rt_ibp
          sta rt_s2
          lda rt_ibp+1
          sta rt_s2+1
          lda rt_ibl
          sta rt_cn
          lda rt_ibl+1
          sta rt_cn+1
.cmp:     ldy #0
          lda (rt_s1),y
          cmp (rt_s2),y
          bne .next
          inc rt_s1
          bne .a
          inc rt_s1+1
.a:       inc rt_s2
          bne .b
          inc rt_s2+1
.b:       lda rt_cn
          bne .c
          dec rt_cn+1
.c:       dec rt_cn
          lda rt_cn
          ora rt_cn+1
          bne .cmp
          clc
          lda rt_ik
          adc #1
          pha
          lda rt_ik+1
          adc #0
          tay
          pla
          rts
.next:    inc rt_ik
          bne .try
          inc rt_ik+1
          jmp .try
.none:    lda #0
          tay
          rts

// ---- INKEY$: key pressed as a string (without waiting), otherwise empty
rt_inkeys: jsr GETIN
          cmp #0
          beq .none
          sta rt_tbuf
          lda #1
          sta rt_sal
          jmp rt_tbufacc
.none:    sta rt_sal
          jmp rt_tbufacc

// ---- TIME$: hhmmss from the clock RTCLOK (seconds = ticks / 50 on PAL, / 60 on NTSC), 24 hours
rt_times: lda RTCLOK+2
          sta rt_d0
          lda RTCLOK+1
          sta rt_d1
          lda RTCLOK
          sta rt_d2
          lda $D014
          cmp #1
          beq .pal
          lda #60
          bne .go
.pal:     lda #50
.go:      jsr rt_div24
          lda #60
          jsr rt_div24
          sta rt_ts
          lda #60
          jsr rt_div24
          sta rt_tm
          lda rt_d0
.h:       cmp #24
          bcc .hd
          sbc #24
          bcs .h
.hd:      ldx #0
          jsr rt_dig2
          lda rt_tm
          jsr rt_dig2
          lda rt_ts
          jsr rt_dig2
          lda #6
          sta rt_sal
          jmp rt_tbufacc
// A (0..99) as two digits to rt_tbuf,x
rt_dig2:  ldy #'0'
.t:       cmp #10
          bcc .o
          sbc #10
          iny
          bne .t
.o:       adc #'0'
          pha
          tya
          sta rt_tbuf,x
          pla
          sta rt_tbuf+1,x
          inx
          inx
          rts
// divide rt_d2:rt_d1:rt_d0 by A: quotient there, remainder in A
rt_div24: sta rt_dv
          lda #0
          ldx #24
.l:       asl rt_d0
          rol rt_d1
          rol rt_d2
          rol
          cmp rt_dv
          bcc .n
          sbc rt_dv
          inc rt_d0
.n:       dex
          bne .l
          rts

rt_ik:    .word 0
rt_il:    .word 0
rt_ibp:   .word 0
rt_ibl:   .word 0
rt_d0:    .byte 0
rt_d1:    .byte 0
rt_d2:    .byte 0
rt_dv:    .byte 0
rt_ts:    .byte 0
rt_tm:    .byte 0
