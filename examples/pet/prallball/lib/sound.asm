// SPDX-License-Identifier: GPL-3.0-or-later
// Klang PET (PRALLBALL). Einbinden: .include "lib/sound.asm"
// Der PET hat keinen Klangchip.  Er hat nur einen Ton ueber das Schieberegister der VIA (Leitung CB2): Zeitgeber 2 gibt den Schiebetakt vor
// (Tonhoehe), das Schieberegister ein Bitmuster (Klangfarbe), die Hilfssteuerung ($E84B) schaltet es ein ($10) und aus ($00).  Einstimmig, nur Pieptoene.
//   sfx_beep      A = Zeitgeberwert (kleiner = hoeher), X = Dauer in Bildern; startet den Ton sofort (ersetzt einen laufenden)
//   sfx_tick      einmal je Bild aus dem Interrupt: zaehlt die Dauer herunter und schaltet am Ende aus
//   sfx_off       Ton aus
//   sfx_*         feste Piepser: paddle, brick, wall, lose, level
// Zeropage (1 Byte, mit -D SND_ZP=... verschiebbar): snd_t (verbleibende Bilder)
.ifndef SND_ZP
.const SND_ZP = $3f
.endif
.const snd_t = SND_ZP

.const VIA_T2L = $e848          // Zeitgeber 2 niedrig (Schiebetakt)
.const VIA_SR_ = $e84a          // Schieberegister (Bitmuster)
.const VIA_ACR_ = $e84b         // Hilfssteuerung: Bit 4..2 = 100 -> Schieberegister mit Zeitgeber 2, frei laufend, Ausgabe auf CB2

sfx_beep:
        sta VIA_T2L
        lda #$0f                // Muster 00001111 = Rechteckton
        sta VIA_SR_
        lda #$10
        sta VIA_ACR_
        stx snd_t
        rts
sfx_off:
        lda #0
        sta VIA_ACR_
        sta snd_t
        rts
// sfx_tick: im Interrupt aufrufen (verändert nur A)
sfx_tick:
        lda snd_t
        beq .t9
        dec snd_t
        bne .t9
        lda #0
        sta VIA_ACR_
.t9:    rts

sfx_paddle:
        lda #90
        ldx #3
        jmp sfx_beep
sfx_brick:
        lda #50
        ldx #3
        jmp sfx_beep
sfx_wall:
        lda #130
        ldx #2
        jmp sfx_beep
sfx_hard:                       // unzerstoerbarer Stein
        lda #170
        ldx #3
        jmp sfx_beep
sfx_lose:
        lda #250
        ldx #30
        jmp sfx_beep
sfx_level:
        lda #30
        ldx #25
        jmp sfx_beep
