// General: 16-bit arithmetic as macros and as subroutines (multiplication, division)
// Include with: #include <common/math.asm>
// Runs on any 6502 (no 65C02/45GS02 instructions). All macros change A (and the carry flag).

#include <common/types.asm>

// Zero page for the subroutines: 6 bytes (A, B, result); must lie entirely in the zero page.
// Can be overridden before the include with .const MATH_ZP = $nn (default $F7-$FC: free on the C64 as long as RS232 is not used).
.ifndef MATH_ZP
.const MATH_ZP = $F7
.endif
.assert MATH_ZP + 5 < $100, "MATH_ZP: 6 Byte müssen in der Zeropage liegen"
.const MATH_A = MATH_ZP        // factor 1 / dividend (2 bytes)
.const MATH_B = MATH_ZP + 2    // factor 2 / divisor (2 bytes)
.const MATH_R = MATH_ZP + 4    // product / remainder (2 bytes)

// ---- Macros
.macro inc16(addr)
 inc addr
 bne @done
 inc addr+1
@done:
.endmacro

.macro dec16(addr)
 lda addr
 bne @low
 dec addr+1
@low: dec addr
.endmacro

// dst = dst + src
.macro add16(dst, src)
 clc
 lda dst
 adc src
 sta dst
 lda dst+1
 adc src+1
 sta dst+1
.endmacro

// dst = dst + constant
.macro add16i(dst, value)
 clc
 lda dst
 adc #<(value)
 sta dst
 lda dst+1
 adc #>(value)
 sta dst+1
.endmacro

// dst = dst - src
.macro sub16(dst, src)
 sec
 lda dst
 sbc src
 sta dst
 lda dst+1
 sbc src+1
 sta dst+1
.endmacro

// dst = dst - constant
.macro sub16i(dst, value)
 sec
 lda dst
 sbc #<(value)
 sta dst
 lda dst+1
 sbc #>(value)
 sta dst+1
.endmacro

// Unsigned compare: afterwards carry = 1 if a >= b (the zero flag cannot be used)
.macro cmp16(a, b)
 lda a
 cmp b
 lda a+1
 sbc b+1
.endmacro

// Two's complement
.macro neg16(addr)
 sec
 lda #0
 sbc addr
 sta addr
 lda #0
 sbc addr+1
 sta addr+1
.endmacro

// Shift by one bit to the left or right
.macro asl16(addr)
 asl addr
 rol addr+1
.endmacro

.macro lsr16(addr)
 lsr addr+1
 ror addr
.endmacro

// ---- Subroutines: place once with math_routines at a free spot in the program
// mul16: MATH_A * MATH_B -> MATH_R (low 16 bits); MATH_A and MATH_B are destroyed
// div16: MATH_A / MATH_B -> quotient in MATH_A, remainder in MATH_R; MATH_B is kept; for divisor 0 the quotient is $FFFF
.macro math_routines
mul16:  lda #0
        sta MATH_R
        sta MATH_R+1
        ldx #16
@mloop: lsr MATH_B+1
        ror MATH_B
        bcc @mskip
        clc
        lda MATH_R
        adc MATH_A
        sta MATH_R
        lda MATH_R+1
        adc MATH_A+1
        sta MATH_R+1
@mskip: asl MATH_A
        rol MATH_A+1
        dex
        bne @mloop
        rts

div16:  lda #0
        sta MATH_R
        sta MATH_R+1
        ldx #16
@dloop: asl MATH_A
        rol MATH_A+1
        rol MATH_R
        rol MATH_R+1
        lda MATH_R
        sec
        sbc MATH_B
        tay
        lda MATH_R+1
        sbc MATH_B+1
        bcc @dskip
        sta MATH_R+1
        sty MATH_R
        inc MATH_A
@dskip: dex
        bne @dloop
        rts
.endmacro
