// Allgemein: 16-Bit-Rechnen als Makros und als Unterprogramme (Multiplikation, Division)
// Einbinden mit: #include <common/math.asm>
// Läuft auf jedem 6502 (keine 65C02/45GS02-Befehle). Alle Makros verändern A (und das Carry-Flag).

#include <common/types.asm>

// Nullseite für die Unterprogramme: 6 Byte (A, B, Ergebnis), muss ganz in der Nullseite liegen.
// Vor dem Einbinden mit .const MATH_ZP = $nn überschreibbar (Standard $F7-$FC: auf C64 frei, solange RS232 nicht benutzt wird).
.ifndef MATH_ZP
.const MATH_ZP = $F7
.endif
.assert MATH_ZP + 5 < $100, "MATH_ZP: 6 Byte müssen in der Nullseite liegen"
.const MATH_A = MATH_ZP        // Faktor 1 / Dividend (2 Byte)
.const MATH_B = MATH_ZP + 2    // Faktor 2 / Divisor (2 Byte)
.const MATH_R = MATH_ZP + 4    // Produkt / Rest (2 Byte)

// ---- Makros
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

// dst = dst + Konstante
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

// dst = dst - Konstante
.macro sub16i(dst, value)
 sec
 lda dst
 sbc #<(value)
 sta dst
 lda dst+1
 sbc #>(value)
 sta dst+1
.endmacro

// Vergleich ohne Vorzeichen: danach gilt Carry = 1, wenn a >= b ist (Zero-Flag ist nicht verwendbar)
.macro cmp16(a, b)
 lda a
 cmp b
 lda a+1
 sbc b+1
.endmacro

// Zweierkomplement
.macro neg16(addr)
 sec
 lda #0
 sbc addr
 sta addr
 lda #0
 sbc addr+1
 sta addr+1
.endmacro

// Um ein Bit nach links bzw. rechts schieben
.macro asl16(addr)
 asl addr
 rol addr+1
.endmacro

.macro lsr16(addr)
 lsr addr+1
 ror addr
.endmacro

// ---- Unterprogramme: einmal mit math_routines an eine freie Stelle im Programm setzen
// mul16: MATH_A * MATH_B -> MATH_R (untere 16 Bit); MATH_A und MATH_B werden zerstört
// div16: MATH_A / MATH_B -> Quotient in MATH_A, Rest in MATH_R; MATH_B bleibt; bei Divisor 0 ist der Quotient $FFFF
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
