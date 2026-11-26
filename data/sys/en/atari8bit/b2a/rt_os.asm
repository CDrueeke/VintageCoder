// BASIC to assembler (basic2asm), Atari 8-bit: the KERNAL calls used by the common runtime (data/sys/common/b2a),
// as a thin layer above the operating system. Names and interface as in the C64 KERNAL; X and Y are preserved.
//   CHROUT  A = character (13 = end of line, becomes ATASCII $9B), output via CIO channel 0 (editor)
//   CHRIN   reads a character from the editor (line input, echo); end of line comes back as 13, carry cleared
//   GETIN   key without waiting (A = 0 if none is pressed), ATASCII, via the keyboard driver K:
//   PLOT    carry set: X = row, Y = column of the cursor; carry cleared: set cursor to X = row, Y = column (columns count from the left margin)
//   RDTIM   A = low, X = middle, Y = high (clock RTCLOK, 1/60 s)    SETTIM   the same in the opposite direction
.const CIO_GETCHR = $07
.const CIO_PUTCHR = $0B

CHROUT:   cmp #13
          bne .go
          lda #$9B
.go:      sta rt_os_a
          stx rt_os_x
          sty rt_os_y
          ldx rt_chan
          lda #CIO_PUTCHR
          sta ICCOM,x
          lda #0
          sta ICBLL,x
          sta ICBLH,x
          lda rt_os_a
          jsr CIOV
          lda rt_os_a
          ldx rt_os_x
          ldy rt_os_y
          rts

CHRIN:    stx rt_os_x
          sty rt_os_y
          ldx #0
          lda #CIO_GETCHR
          sta ICCOM
          lda #0
          sta ICBLL
          sta ICBLH
          jsr CIOV
          cmp #$9B
          bne .k
          lda #13
.k:       ldx rt_os_x
          ldy rt_os_y
          clc
          rts

GETIN:    lda CH
          cmp #$FF
          bne .key
          lda #0
          rts
.key:     stx rt_os_x
          sty rt_os_y
          jsr .callk
          ldx rt_os_x
          ldy rt_os_y
          rts
// K: driver: table at $E420 (open, close, read, write, ...), each entry is the address minus 1
.callk:   lda $E425
          pha
          lda $E424
          pha
          rts

// Columns count from the left margin (LMARGN), like the tabs of Atari BASIC
PLOT:     bcc .set
          lda COLCRS
          sec
          sbc LMARGN
          bcs .ok
          lda #0
.ok:      tay
          ldx ROWCRS
          rts
.set:     stx ROWCRS
          tya
          clc
          adc LMARGN
          sta COLCRS
          rts

RDTIM:    lda RTCLOK+2
          ldx RTCLOK+1
          ldy RTCLOK
          rts

SETTIM:   sta RTCLOK+2
          stx RTCLOK+1
          sty RTCLOK
          rts

rt_os_a:  .byte 0
rt_os_x:  .byte 0
rt_os_y:  .byte 0
rt_chan:  .byte 0               // IOCB offset for CHROUT: 0 = E: (editor), $60 = channel 6 (S:, PRINT #6)
