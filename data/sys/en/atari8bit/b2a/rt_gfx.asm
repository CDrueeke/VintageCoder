// BASIC to assembler (basic2asm), Atari 8-bit: graphics, sound and joystick as in Atari BASIC, via CIO (channel 6 = S:) and the hardware registers.
//   GRAPHICS n     close channel 6 and open S: with mode n (n + 16 without text window, n + 32 without clearing)
//   POSITION x,y   set cursor (COLCRS, ROWCRS)       PLOT x,y   set cursor, plot a point with the color of COLOR
//   DRAWTO x,y     line from the last point to the new one    COLOR n    color value for PLOT and DRAWTO     LOCATE x,y,v  color value of a point
//   SETCOLOR r,f,h color register r (0-4) = f * 16 + h (shadow register COLOR0)
//   SOUND v,t,v,l  voice 0-3: frequency, distortion * 16 + volume; AUDCTL = 0, SKCTL = 3
//   STICK(n), STRIG(n) shadow registers STICK0 and STRIG0
// Arguments: all but the last on the word stack (rt_push), the last in A/Y.
// Requires: rt_core.asm, rt_os.asm (CIOV). OS errors (Y >= $80 after CIO) become ILLEGAL QUANTITY.

.const RT_IOCB6 = $60
.const RT_AUDF1 = $D200   // POKEY (own names: the runtime does not include pokey.asm)
.const RT_AUDC1 = $D201
.const RT_AUDCTL = $D208
.const RT_SKCTL = $D20F
.const CIO_OPEN   = $03
.const CIO_DRAW   = $11
.const ATACHR     = $02FB

// OS error (A = Y >= $80): "?DEVICE ERROR <number> IN <line>" with the Atari error number (e.g. 170 file not found, 136 end of file)
rt_cio_err: bpl .ok
          sta rt_cv_n
          lda #13
          jsr CHROUT
          lda #'?'
          jsr CHROUT
          lda #<rt_s_dev
          ldy #>rt_s_dev
          jsr rt_print_str
          lda rt_cv_n
          ldy #0
          jsr rt_print_dec
          lda rt_line
          ora rt_line+1
          beq .nl
          lda #<rt_s_in
          ldy #>rt_s_in
          jsr rt_print_str
          lda rt_line
          ldy rt_line+1
          jsr rt_print_dec
.nl:      lda #13
          jsr CHROUT
          jmp rt_end
.ok:      rts
rt_s_dev: .petscii "device error "
          .byte 0

// GRAPHICS n (A/Y)
rt_gr:    sta rt_gv
          ldx #RT_IOCB6
          lda #CIO_CLOSE
          sta ICCOM,x
          jsr CIOV
          ldx #RT_IOCB6
          lda #CIO_OPEN
          sta ICCOM,x
          lda #<rt_gdev
          sta ICBAL,x
          lda #>rt_gdev
          sta ICBAL+1,x
          lda rt_gv
          and #$30
          ora #$0C
          sta ICAX1,x
          lda rt_gv
          and #$0F
          sta ICAX2,x
          jsr CIOV
          tya
          jmp rt_cio_err

// POSITION x,y: x on the stack, y in A/Y
rt_pos:   sta ROWCRS
          jsr rt_pop0
          lda rt_t0
          sta COLCRS
          lda rt_t0+1
          sta COLCRS+1
          rts

// COLOR n
rt_color: sta rt_gcol
          rts

// write a byte with the color to channel 6
rt_plot:  jsr rt_pos
          ldx #RT_IOCB6
          lda #CIO_PUTCHR
          sta ICCOM,x
          lda #0
          sta ICBLL,x
          sta ICBLL+1,x
          lda rt_gcol
          jsr CIOV
          tya
          jmp rt_cio_err

rt_drawto: jsr rt_pos
          lda rt_gcol
          sta ATACHR
          ldx #RT_IOCB6
          lda #CIO_DRAW
          sta ICCOM,x
          lda #$0C
          sta ICAX1,x
          lda #0
          sta ICAX2,x
          jsr CIOV
          tya
          jmp rt_cio_err

// Turbo BASIC XL: FCOLOR n (fill color, FILDAT $02FD) and FILLTO x,y (fill line from the last point to x,y, XIO 18)
rt_fcolor: sta rt_gfc
          sta $02FD
          rts
rt_fillto: jsr rt_pos
          lda rt_gcol
          sta ATACHR
          lda rt_gfc
          sta $02FD
          ldx #RT_IOCB6
          lda #$12
          sta ICCOM,x
          lda #$0C
          sta ICAX1,x
          lda #0
          sta ICAX2,x
          jsr CIOV
          tya
          jmp rt_cio_err
rt_gfc:   .byte 0

// LOCATE x,y,v: color value of the point -> A
rt_locate: jsr rt_pos
          ldx #RT_IOCB6
          lda #CIO_GETCHR
          sta ICCOM,x
          lda #0
          sta ICBLL,x
          sta ICBLL+1,x
          jsr CIOV
          sta rt_gv
          tya
          jsr rt_cio_err
          lda rt_gv
          rts

// SETCOLOR r,f,h: r and f on the stack, h in A/Y
rt_setcolor: sta rt_t1
          jsr rt_pop0              // f
          lda rt_t0
          asl
          asl
          asl
          asl
          ora rt_t1
          sta rt_gv
          jsr rt_pop0              // r
          lda rt_t0+1
          bne .bad
          ldx rt_t0
          cpx #5
          bcs .bad
          lda rt_gv
          sta COLOR0,x
          rts
.bad:     jmp rt_e_illegal

// SOUND v,t,d,l: v, t and d on the stack, l in A/Y
rt_sound: sta rt_t1                // volume
          jsr rt_pop0              // distortion
          lda rt_t0
          asl
          asl
          asl
          asl
          ora rt_t1
          sta rt_gv
          jsr rt_pop0              // frequency
          lda rt_t0
          sta rt_gf
          jsr rt_pop0              // voice
          lda rt_t0+1
          bne .bad
          lda rt_t0
          cmp #4
          bcs .bad
          asl
          tax
          lda rt_gf
          sta RT_AUDF1,x
          lda rt_gv
          sta RT_AUDC1,x
          lda #0
          sta RT_AUDCTL
          lda #3
          sta RT_SKCTL
          rts
.bad:     jmp rt_e_illegal

// STICK(n) / STRIG(n): n in A/Y (0-3)
rt_stick: cpy #0
          bne .bad
          cmp #4
          bcs .bad
          tax
          lda STICK0,x
          and #$0F
          ldy #0
          rts
.bad:     jmp rt_e_illegal
rt_strig: cpy #0
          bne .bad
          cmp #4
          bcs .bad
          tax
          lda STRIG0,x
          and #$01
          ldy #0
          rts
.bad:     jmp rt_e_illegal

rt_gdev:  .byte 'S', ':', $9B
rt_gv:    .byte 0
rt_gf:    .byte 0
rt_gcol:  .byte 0

// ---- Channels (CIO): OPEN #n,aux1,aux2,"name"; CLOSE #n; PUT #n,b; GET #n,v; Turbo BASIC XL: BPUT/BGET #n,address,length
// Channel number 0-7 -> X = channel number * 16; OS errors (Y >= $80) become ILLEGAL QUANTITY
.const CIO_CLOSE2 = $0C
.const CIO_GETCH  = $07
.const CIO_PUTCH  = $0B

// A (0-7) -> X = A * 16
rt_chx:   and #$07
          asl
          asl
          asl
          asl
          tax
          rts

// CLOSE #n: A = channel (errors are ignored as in Atari BASIC)
rt_close: jsr rt_chx
          lda #CIO_CLOSE2
          sta ICCOM,x
          jmp CIOV

// PUT #n,b: channel on the stack, byte in A
rt_put:   sta rt_cv_n
          jsr rt_pop0
          lda rt_t0
          jsr rt_chx
          lda #CIO_PUTCH
          sta ICCOM,x
          lda #0
          sta ICBLL,x
          sta ICBLL+1,x
          lda rt_cv_n
          jsr CIOV
          tya
          jmp rt_cio_err

// GET #n,v: A = channel -> A = byte
rt_getc:  jsr rt_chx
          lda #CIO_GETCH
          sta ICCOM,x
          lda #0
          sta ICBLL,x
          sta ICBLL+1,x
          jsr CIOV
          sta rt_cv_n
          tya
          jsr rt_cio_err
          lda rt_cv_n
          rts

// BPUT/BGET #n,address,length: channel and address on the stack (address on top), length in A/Y
rt_bput:  ldx #CIO_PUTCH
          bne rt_blk
rt_bget:  ldx #CIO_GETCH
rt_blk:   stx rt_cv_c
          sta rt_cv_l
          sty rt_cv_l+1
          jsr rt_pop0
          lda rt_t0
          sta rt_cv_a1
          lda rt_t0+1
          sta rt_cv_a2
          jsr rt_pop0
          lda rt_t0
          jsr rt_chx
          lda rt_cv_c
          sta ICCOM,x
          lda rt_cv_a1
          sta ICBAL,x
          lda rt_cv_a2
          sta ICBAL+1,x
          lda rt_cv_l
          sta ICBLL,x
          lda rt_cv_l+1
          sta ICBLL+1,x
          jsr CIOV
          tya
          jmp rt_cio_err

rt_cv_n:  .byte 0
rt_cv_c:  .byte 0
rt_cv_a1: .byte 0
rt_cv_a2: .byte 0
rt_cv_l:  .word 0
rt_fnbuf: .fill 64, 0
