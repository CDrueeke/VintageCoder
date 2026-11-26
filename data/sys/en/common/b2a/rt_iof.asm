// BASIC->assembler, C64 runtime, level 2: number output, INPUT, GET with floating point
// Requires: rt_core.asm, rt_fp.asm

.ifdef B2A_ATARI
// PRINT of a number like Atari BASIC: text of the OS package (FASC), without sign position and without trailing space; a trailing period is dropped. Value in FR0
// As in the BASIC module: if the first digit pair is >= 10, the tenth digit is dropped (at most 9 significant digits)
// Turbo BASIC XL (RT_P10) always prints all ten digits
rt_print_f:
.ifndef RT_P10
          lda FR0+1
          cmp #$10
          bcc .go
          lda FR0+5
          and #$F0
          sta FR0+5
.endif
.go:      jsr FASC
          ldy #0
.l:       lda (INBUFF),y
          bmi .last
          jsr CHROUT
          iny
          bne .l
.last:    and #$7f
          cmp #'.'
          beq .e
          jmp CHROUT
.e:       rts

// INPUT of a number: prints "? ", reads a line. Result in the FAC (AFP); carry set = empty input (variable is kept)
rt_input_f:
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
          ldx #0                   // empty input (only spaces)?
.bl:      lda rt_buf,x
          beq .empty
          inx
          cmp #' '
          beq .bl
          lda #<rt_buf
          sta INBUFF
          lda #>rt_buf
          sta INBUFF+1
          lda #0
          sta CIX
          jsr AFP
          bcs .redo
          ldy CIX                  // afterwards only spaces
.tr:      lda rt_buf,y
          beq .ok
          cmp #' '
          bne .redo
          iny
          bne .tr
.ok:      clc
          rts
.empty:   sec
          rts
.redo:    lda #<rt_s_redo
          ldy #>rt_s_redo
          jsr rt_print_str
          jmp .ask
.else
// PRINT of a number: text from the ROM (FOUT: sign position, digits), then a space. Value in the FAC
rt_print_f: jsr F_FOUT
          jsr rt_print_str
          lda #' '
          jmp CHROUT

// INPUT of a number: prints "? ", reads a line. Result in the FAC; carry set = empty input (variable is kept)
rt_input_f:
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
          ldx #0                   // check: only characters of a number, at least one digit
          stx rt_tmp
.chk:     lda rt_buf,x
          beq .done
          inx
          cmp #' '
          beq .chk
          cmp #'+'
          beq .chk
          cmp #'-'
          beq .chk
          cmp #'.'
          beq .chk
          cmp #'E'
          beq .chk
          cmp #'0'
          bcc .redo
          cmp #'9'+1
          bcs .redo
          inc rt_tmp
          jmp .chk
.done:    lda rt_buf
          bne .some
          sec                      // empty input
          rts
.some:    lda rt_tmp
          beq .redo
          lda F_TXTPTR             // save the BASIC text pointer, set it to the buffer
          sta rt_txt
          lda F_TXTPTR+1
          sta rt_txt+1
          lda #<rt_buf
          sta F_TXTPTR
          lda #>rt_buf
          sta F_TXTPTR+1
          jsr F_CHRGOT
          jsr F_FIN
          lda rt_txt
          sta F_TXTPTR
          lda rt_txt+1
          sta F_TXTPTR+1
          clc
          rts
.redo:    lda #<rt_s_redo
          ldy #>rt_s_redo
          jsr rt_print_str
          jmp .ask

.endif

// GET of a number: key 0-9 -> value, no key -> 0, otherwise error (A/Y)
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
