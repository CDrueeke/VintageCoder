// SPDX-License-Identifier: GPL-3.0-or-later
// Zufall Plus/4: 16-Bit-Galois-LFSR (PRALLBALL). Einbinden: .include "lib/rand.asm"
//   rand_seed   A = Startwert (0 wird zu $A5), mischt die Rasterzeile des TED ($FF1D) ein
//   rand_byte   A = nächste Zufallszahl 0..255 (verändert nur A)
// Zeropage (2 Byte, -D RAND_ZP=... verschiebbar): rand_lo, rand_hi
//   Warum $02/$03: Die Nullseite ab $90 gehört dem KERNAL (Dateizugriff der Bestenliste verändert sie); $02-$0F bleibt unberührt.
.ifndef RAND_ZP
.const RAND_ZP = $02
.endif
.const rand_lo = RAND_ZP
.const rand_hi = RAND_ZP + 1

rand_seed:
        eor $ff1d               // TED: aktuelle Rasterzeile (Bits 0..7) als Zufallsquelle
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
