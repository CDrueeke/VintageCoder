// BASIC->assembler, C64 runtime: number output, INPUT, GET
// Requires: rt_core.asm

// PRINT of a number like the interpreter: CBM: sign position (space or -), digits, space; Atari: only - and digits. A/Y = value
rt_print_int: sta rt_t0
          sty rt_t0+1
          tya
          bpl .pos
          lda #'-'
          jsr CHROUT
          neg16(rt_t0)
          jmp .go
.pos:
.ifndef B2A_ATARI
          lda #' '
          jsr CHROUT
.endif
.go:      lda rt_t0
          ldy rt_t0+1
.ifdef B2A_ATARI
          jmp rt_print_dec
.else
          jsr rt_print_dec
          lda #' '
          jmp CHROUT
.endif

// INPUT of a number: prints "? ", reads a line. Result A/Y; carry set = empty input (variable is kept)
rt_input_int:
.ask:     lda #'?'
          jsr CHROUT
          lda #' '
          jsr CHROUT
          ldx #0
.rd:      jsr CHRIN
          cmp #13
          beq .eol
          cpx #80
          bcs .rd
          sta rt_buf,x
          inx
          bne .rd
.eol:     lda #13
          jsr CHROUT
          lda #0
          sta rt_buf,x
          lda #0
          sta rt_t0
          sta rt_t0+1
          sta rt_tmp               // Bit 7: negative, bit 0: at least one digit
          sta rt_v                 // Index into the buffer
.sp1:     ldx rt_v
          lda rt_buf,x
          cmp #' '
          bne .sg
          inc rt_v
          bne .sp1
.sg:      cmp #0
          bne .nz
          sec                      // empty input
          rts
.nz:      cmp #'-'
          bne .pl
          lda #$80
          sta rt_tmp
          inc rt_v
          jmp .dg
.pl:      cmp #'+'
          bne .dg
          inc rt_v
.dg:      ldx rt_v
          lda rt_buf,x
          sec
          sbc #'0'
          cmp #10
          bcs .end
          pha
          asl rt_t0                // t0 = t0 * 10
          rol rt_t0+1
          lda rt_t0
          sta rt_t1
          lda rt_t0+1
          sta rt_t1+1
          asl rt_t0
          rol rt_t0+1
          asl rt_t0
          rol rt_t0+1
          clc
          lda rt_t0
          adc rt_t1
          sta rt_t0
          lda rt_t0+1
          adc rt_t1+1
          sta rt_t0+1
          pla
          clc
          adc rt_t0
          sta rt_t0
          bcc .nc
          inc rt_t0+1
.nc:      lda rt_tmp
          ora #1
          sta rt_tmp
          inc rt_v
          jmp .dg
.end:     lda rt_tmp
          and #1
          beq .redo
.tr:      ldx rt_v                 // only spaces allowed now
          lda rt_buf,x
          beq .ok
          cmp #' '
          bne .redo
          inc rt_v
          jmp .tr
.ok:      lda rt_tmp
          bpl .done
          neg16(rt_t0)
.done:    lda rt_t0
          ldy rt_t0+1
          clc
          rts
.redo:    lda #<rt_s_redo
          ldy #>rt_s_redo
          jsr rt_print_str
          jmp .ask

// GET of a number: key 0-9 -> value, no key -> 0, otherwise error
rt_get_int: jsr GETIN
          cmp #0
          beq .none
          sec
          sbc #'0'
          cmp #10
          bcs rt_e_syntax
          ldy #0
          rts
.none:    ldy #0
          rts

rt_s_redo: .byte 63
          .petscii "redo from start"
          .byte 13, 0
rt_buf:   .fill 82, 0
