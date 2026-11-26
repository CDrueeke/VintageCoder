// BASIC->assembler, C64 runtime, level 2: floating point via the routines of the BASIC ROM (same rounding and errors as the interpreter).
// Value in the FAC ($61). Binary operators: left operand on the floating-point stack (5 bytes per entry), right one in the FAC; result in the FAC.
// Requires: rt_core.asm, rt_zp.asm with RT_FLOAT
// Atari 8 bit (B2A_ATARI): same interface, but 6-byte BCD numbers of the OS package (FR0 = FAC); the F_* entry points are provided by atari8bit/b2a/rt_osfp.asm.
.ifndef RT_FS
.const RT_FS  = 5        // Bytes per number
.const RT_SGN = 1        // Byte with the sign bit (MBF: byte 1, BCD: byte 0)
.endif
.const RT_FES_LEN = 16*RT_FS   // Size of the floating-point stack (16 entries)

.ifdef B2A_ATARI
// ---- Numbers: 6-byte BCD (byte 0: sign + exponent to base 100, bytes 1-5: digit pairs); FR0 is the FAC, FR1 the second operand
rt_movfm: tax                    // FAC = (A/Y)
          jmp FLD0R

// A/Y (low/high, signed) -> FAC
rt_i2f:   sta rt_cv
          sty rt_cv+1
          lda #0
          sta rt_cs
          tya
          bpl .p
          sec
          lda #0
          sbc rt_cv
          sta rt_cv
          lda #0
          sbc rt_cv+1
          sta rt_cv+1
          lda #$80
          sta rt_cs
.p:       lda rt_cv
          sta FR0
          lda rt_cv+1
          sta FR0+1
          jsr IFP
          lda FR0
          ora rt_cs
          sta FR0
          rts

// FAC -> A/Y, signed; decimals are rounded down as with INT; outside -32768..32767: ILLEGAL QUANTITY
rt_f2i:   jsr F_INT
          lda FR0
          and #$80
          sta rt_cs
          lda FR0
          and #$7f
          sta FR0
          jsr FPI
          bcs .big
          lda rt_cs
          bne .neg
          lda FR0+1
          bmi .big                 // 32768 and above
          lda FR0
          ldy FR0+1
          rts
.neg:     lda FR0+1
          cmp #$80
          bcc .ok
          bne .big
          lda FR0
          bne .big
.ok:      sec
          lda #0
          sbc FR0
          pha
          lda #0
          sbc FR0+1
          tay
          pla
          rts
.big:     jmp rt_e_illegal

// FAC -> A/Y as a number 0..65535 (addresses, array indices), rounded to the nearest integer (FPI, like Atari BASIC for POKE, array indices, CHR$ etc.);
// negative or too large: ILLEGAL QUANTITY
rt_f2addr: lda FR0
          bmi .neg
.go:      jsr FPI
          bcs .big
          lda FR0
          ldy FR0+1
          rts
.neg:     and #$7f                 // -0 is allowed, any other negative number is not
          ora FR0+1
          bne .big
          beq .go
.big:     jmp rt_e_illegal

rt_cv:    .word 0              // Scratch area of the conversions (the callers' pointers rt_t0 etc. stay untouched)
rt_cs:    .byte 0
rt_fb:    .byte 0, 0, 0, 0, 0, 0
.else
// FAC = (A/Y). The carry flag is cleared: on the Plus/4 the ROM routine reads directly if C = 1, via the RAM helper if C = 0 (memory under the ROM);
// the variables and constants of the program are in RAM. On the C64 and C128 the flag has no meaning.
rt_movfm: clc
          jmp F_MOVFM

// ---- Integer <-> FAC (in assembler, without ROM entry points: those differ per system and are partly bank-dependent)
// A/Y (low/high, signed) -> FAC: build a 5-byte number and load it
rt_i2f:   sta rt_cv
          sty rt_cv+1
          tya
          ora rt_cv
          beq .z
          lda #0
          sta rt_cs               // Sign bit
          lda rt_cv+1
          bpl .p
          sec
          lda #0
          sbc rt_cv
          sta rt_cv
          lda #0
          sbc rt_cv+1
          sta rt_cv+1
          lda #$80
          sta rt_cs
.p:       ldx #$90                 // Exponent if bit 15 is set
.n:       lda rt_cv+1
          bmi .d
          asl rt_cv
          rol rt_cv+1
          dex
          jmp .n
.d:       stx rt_fb
          and #$7f
          ora rt_cs
          sta rt_fb+1
          lda rt_cv
          sta rt_fb+2
          lda #0
          sta rt_fb+3
          sta rt_fb+4
          lda #<rt_fb
          ldy #>rt_fb
          jmp rt_movfm
.z:       lda #<rt_f0
          ldy #>rt_f0
          jmp rt_movfm

// FAC -> A/Y, signed; decimals are rounded down as with INT; outside -32768..32767: ILLEGAL QUANTITY
rt_f2i:   jsr F_INT
          ldx #<rt_fb
          ldy #>rt_fb
          jsr F_MOVMF
          lda rt_fb
          beq .zero
          cmp #$91
          bcs .big
          cmp #$90
          bne .sh
          lda rt_fb+1              // 2^15 is allowed only as -32768
          cmp #$80
          bne .big
          lda rt_fb+2
          bne .big
          lda #0
          ldy #$80
          rts
.sh:      lda rt_fb+1
          ora #$80
          sta rt_cv+1
          lda rt_fb+2
          sta rt_cv
          lda #$90
          sec
          sbc rt_fb
          tax
.l:       cpx #0
          beq .s
          lsr rt_cv+1
          ror rt_cv
          dex
          jmp .l
.s:       lda rt_fb+1
          bpl .pos
          sec
          lda #0
          sbc rt_cv
          sta rt_cv
          lda #0
          sbc rt_cv+1
          sta rt_cv+1
.pos:     lda rt_cv
          ldy rt_cv+1
          rts
.zero:    lda #0
          tay
          rts
.big:     jmp rt_e_illegal

// FAC -> A/Y as a number 0..65535 (addresses, array indices), decimals truncated; negative or too large: ILLEGAL QUANTITY
rt_f2addr: ldx #<rt_fb
          ldy #>rt_fb
          jsr F_MOVMF
          lda rt_fb
          beq .zero
          lda rt_fb+1
          bmi .big
          lda rt_fb
          cmp #$91
          bcs .big
          cmp #$81
          bcc .zero
          lda rt_fb+1
          ora #$80
          sta rt_cv+1
          lda rt_fb+2
          sta rt_cv
          lda #$90
          sec
          sbc rt_fb
          tax
.l:       cpx #0
          beq .d
          lsr rt_cv+1
          ror rt_cv
          dex
          jmp .l
.d:       lda rt_cv
          ldy rt_cv+1
          rts
.zero:    lda #0
          tay
          rts
.big:     jmp rt_e_illegal

rt_fb:    .byte 0, 0, 0, 0, 0
rt_cv:    .word 0              // Scratch area of the conversions (the callers' pointers rt_t0 etc. stay untouched)
rt_cs:    .byte 0

.endif

// ---- Floating-point stack (16 entries of 5 bytes each)
rt_fpush: lda rt_esp
          cmp #RT_FES_LEN
          bcs rt_e_formula
          clc
          adc #<rt_fes
          tax
          lda #>rt_fes
          adc #0
          tay
          jsr F_MOVMF
          lda rt_esp
          clc
          adc #RT_FS
          sta rt_esp
          rts

// remove top entry; A/Y = its address (stays valid until the next push)
rt_fpop:  lda rt_esp
          sec
          sbc #RT_FS
          sta rt_esp
          clc
          adc #<rt_fes
          pha
          lda #>rt_fes
          adc #0
          tay
          pla
          rts

rt_fadd:  jsr rt_fpop
          jsr F_CONUPK
          jmp F_FADDT
rt_fsub:  jsr rt_fpop
          jsr F_CONUPK
          jmp F_FSUBT
rt_fmul:  jsr rt_fpop
          jsr F_CONUPK
          jmp F_FMULTT
rt_fdiv:  jsr rt_fpop
          jsr F_CONUPK
          jmp F_FDIVT
rt_fpow:  jsr rt_fpop
          jsr F_CONUPK
          jmp F_FPWRT

// ---- Comparisons: result -1 (true) or 0 (false) as a floating-point number
// rt_fcmp: A = 0 equal, 1 left greater, $FF left smaller (FAC holds the right operand; F_FCOMP returns it the other way round)
rt_fcmp:  jsr rt_fpop
          jsr F_FCOMP
          eor #$ff
          clc
          adc #1
          rts

.ifdef B2A_ATARI
rt_ftrue: lda #<rt_f1               // Atari BASIC: true = 1
          ldy #>rt_f1
          jmp rt_movfm
.else
rt_ftrue: lda #<rt_fm1
          ldy #>rt_fm1
          jmp rt_movfm
.endif
rt_ffalse: lda #<rt_f0
          ldy #>rt_f0
          jmp rt_movfm

rt_feq:   jsr rt_fcmp
          beq rt_ftrue
          bne rt_ffalse
rt_fne:   jsr rt_fcmp
          bne rt_ftrue
          beq rt_ffalse
rt_flt:   jsr rt_fcmp
          cmp #$ff
          beq rt_ftrue
          bne rt_ffalse
rt_fgt:   jsr rt_fcmp
          cmp #1
          beq rt_ftrue
          bne rt_ffalse
rt_fle:   jsr rt_fcmp
          cmp #1
          bne rt_ftrue
          beq rt_ffalse
rt_fge:   jsr rt_fcmp
          cmp #$ff
          bne rt_ftrue
          beq rt_ffalse

// ---- Logic on 16-bit integers (like the interpreter: operands are truncated)
.ifdef B2A_ATARI
// Atari BASIC: AND, OR and NOT are logical (1 or 0)
rt_fand:  lda F_FACEXP
          sta rt_v
          jsr rt_fpop
          jsr rt_movfm
          lda F_FACEXP
          beq .f
          lda rt_v
          beq .f
          jmp rt_ftrue
.f:       jmp rt_ffalse
rt_forr:  lda F_FACEXP
          sta rt_v
          jsr rt_fpop
          jsr rt_movfm
          lda F_FACEXP
          ora rt_v
          beq .f
          jmp rt_ftrue
.f:       jmp rt_ffalse
rt_fnot:  lda F_FACEXP
          beq .t
          jmp rt_ffalse
.t:       jmp rt_ftrue
.else
rt_fand:  jsr rt_f2i
          sta rt_v
          sty rt_v+1
          jsr rt_fpop
          jsr rt_movfm
          jsr rt_f2i
          and rt_v
          pha
          tya
          and rt_v+1
          tay
          pla
          jmp rt_i2f

rt_forr:  jsr rt_f2i
          sta rt_v
          sty rt_v+1
          jsr rt_fpop
          jsr rt_movfm
          jsr rt_f2i
          ora rt_v
          pha
          tya
          ora rt_v+1
          tay
          pla
          jmp rt_i2f

rt_fnot:  jsr rt_f2i
          eor #$ff
          pha
          tya
          eor #$ff
          tay
          pla
          jmp rt_i2f

.endif

// ---- Memory access (FAC holds the value)
rt_fpeek: jsr rt_f2addr
          jsr rt_peek
          jmp rt_i2f

rt_peek:  sta rt_t0
          sty rt_t0+1
          ldy #0
          lda (rt_t0),y
          rts

// A/Y must be 0..255
rt_byte:  cpy #0
          bne rt_e_illegal
          rts

// POKE addr,value: address on the floating-point stack, value in the FAC
rt_fpoke: jsr rt_f2addr
          jsr rt_byte
          sta rt_tmp
          jsr rt_fpop
          jsr rt_movfm
          jsr rt_f2addr
          sta rt_t0
          sty rt_t0+1
          ldy #0
          lda rt_tmp
          sta (rt_t0),y
          rts

rt_sys:   sta rt_t0
          sty rt_t0+1
          jmp (rt_t0)

// WAIT: address and mask on the floating-point stack, XOR value in the FAC
rt_fwait: jsr rt_f2addr
          jsr rt_byte
          sta rt_tmp
          jsr rt_fpop
          jsr rt_movfm
          jsr rt_f2addr
          jsr rt_byte
          sta rt_v
          jsr rt_fpop
          jsr rt_movfm
          jsr rt_f2addr
          sta rt_t0
          sty rt_t0+1
.l:       ldy #0
          lda (rt_t0),y
          eor rt_tmp
          and rt_v
          beq .l
          rts

// TI: full system clock ($A0-$A2) as a number
rt_fti:   jsr RDTIM              // A = low, X = middle, Y = high
          sta rt_v
          txa
          jsr rt_i2f             // A = middle, Y = high: as a number
          lda #<rt_f256
          ldy #>rt_f256
          jsr F_CONUPK
          jsr F_FMULTT
          jsr rt_fpush
          lda rt_v
          ldy #0
          jsr rt_i2f
          jmp rt_fadd

.ifdef B2A_ATARI
rt_fm1:   .byte $C0, $01, 0, 0, 0, 0
rt_f0:    .byte 0, 0, 0, 0, 0, 0
rt_f1:    .byte $40, $01, 0, 0, 0, 0
rt_f256:  .byte $41, $02, $56, 0, 0, 0
.else
rt_fm1:   .byte $81, $80, 0, 0, 0
rt_f0:    .byte 0, 0, 0, 0, 0
rt_f1:    .byte $81, 0, 0, 0, 0
rt_f256:  .byte $89, 0, 0, 0, 0
.endif
rt_fes:   .fill RT_FES_LEN, 0
