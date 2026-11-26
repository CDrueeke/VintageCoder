// General: bit masks, processor flags and macros for bytes, words and pointers
// Include with: #include <common/types.asm>

// ---- Bit masks
.const BIT0 = $01
.const BIT1 = $02
.const BIT2 = $04
.const BIT3 = $08
.const BIT4 = $10
.const BIT5 = $20
.const BIT6 = $40
.const BIT7 = $80

// ---- Processor status bits (for comparisons with the pushed status value)
.const P_CARRY     = $01
.const P_ZERO      = $02
.const P_IRQ_OFF   = $04
.const P_DECIMAL   = $08
.const P_BREAK     = $10
.const P_OVERFLOW  = $40
.const P_NEGATIVE  = $80

.const FALSE = 0
.const TRUE  = 1

// ---- Bits in a memory location (changes A)
.macro set_bits(addr, mask)
 lda addr
 ora #mask
 sta addr
.endmacro

.macro clear_bits(addr, mask)
 lda addr
 and #(mask) ^ $ff
 sta addr
.endmacro

.macro toggle_bits(addr, mask)
 lda addr
 eor #mask
 sta addr
.endmacro

// ---- Words and pointers (low byte first, as on the 6502)
// Write a 16-bit constant to addr, addr+1 (changes A)
.macro ld16(addr, value)
 lda #<(value)
 sta addr
 lda #>(value)
 sta addr+1
.endmacro

// Set a pointer to an address (same as ld16, reads better for zero-page pointers)
.macro ptr_set(zp, target)
 lda #<(target)
 sta zp
 lda #>(target)
 sta zp+1
.endmacro

// Copy a 16-bit value from src to dst (changes A)
.macro mov16(dst, src)
 lda src
 sta dst
 lda src+1
 sta dst+1
.endmacro

// Load A/X with a 16-bit constant (A = low, X = high)
.macro ldax(value)
 lda #<(value)
 ldx #>(value)
.endmacro

// ---- Bytes (dst first, as with mov16; src may also be a value with #, e.g. mov8 $d020, #0)
// Copy a byte from src to dst (changes A)
.macro mov8(dst, src)
 lda src
 sta dst
.endmacro

// like mov8 via X or Y (A is preserved)
.macro movx8(dst, src)
 ldx src
 stx dst
.endmacro

.macro movy8(dst, src)
 ldy src
 sty dst
.endmacro

// dst = dst + src or dst - src (changes A and the carry flag)
.macro add8(dst, src)
 clc
 lda dst
 adc src
 sta dst
.endmacro

.macro sub8(dst, src)
 sec
 lda dst
 sbc src
 sta dst
.endmacro
