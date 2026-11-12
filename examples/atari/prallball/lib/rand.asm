// SPDX-License-Identifier: GPL-3.0-or-later
// Zufall Atari 8 Bit (PRALLBALL). Einbinden: .include "lib/rand.asm"
//
// Der POKEY liefert ein echtes Hardware-Zufallsregister: RANDOM ($D20A) ist ein 17-Bit-Schieberegister, das mit dem POKEY-Takt weiterläuft;
// jedes Lesen gibt die oberen 8 Bit. Voraussetzung: der POKEY läuft (SKCTL Bit 0..1 = 1, nach dem Einschalten des OS der Fall).
// Dennoch halten wir den Vertrag der anderen Systeme ein (rand_seed / rand_byte), damit Spielcode ohne Änderung der Aufrufe auskommt.
//   rand_seed   A = Zusatz (wird ignoriert, der POKEY braucht keinen Startwert); mischt die Rasterzeile (VCOUNT) ein
//   rand_byte   A = Zufallszahl 0..255 (verändert nur A)
// Zeropage: keine.   Zyklen: rand_byte = 4 + 6 (jsr/rts) 
.const RAND_RANDOM = $d20a
.const RAND_VCOUNT = $d40b

rand_seed:
        lda RAND_VCOUNT                 // nur ein Lesezugriff als „Anstoß“; Zufall kommt aus dem POKEY
        rts

rand_byte:
        lda RAND_RANDOM
        rts
