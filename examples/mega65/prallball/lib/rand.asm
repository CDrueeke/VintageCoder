// SPDX-License-Identifier: GPL-3.0-or-later
// Zufall MEGA65 (Code wie am C64): 16-Bit-Galois-LFSR (PRALLBALL). Einbinden: .include "lib/rand.asm"
//   rand_seed   A = Startwert (0 wird zu $A5), mischt $D012 (Rasterzeile) ein
//   rand_byte   A = nächste Zufallszahl 0..255 (verändert nur A)
// Zeropage (2 Byte, -D RAND_ZP=... verschiebbar): rand_lo, rand_hi
.ifndef RAND_ZP
.const RAND_ZP = $f6
.endif
.const rand_lo = RAND_ZP
.const rand_hi = RAND_ZP + 1

rand_seed:
        eor $d012
        bne +
        lda #$a5
+       sta rand_lo
        lda #$5a
        sta rand_hi
        rts

rand_byte:
        lda rand_hi
        lsr
        sta rand_hi
        lda rand_lo
        ror
        sta rand_lo
        bcc +
        lda rand_hi
        eor #$b4                // Polynom x^16+x^14+x^13+x^11+1
        sta rand_hi
+       lda rand_lo
        eor rand_hi
        rts
