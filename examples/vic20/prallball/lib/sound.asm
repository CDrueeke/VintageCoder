// SPDX-License-Identifier: GPL-3.0-or-later
// Klang VC-20 (PRALLBALL): Effekte und Titelmusik auf dem VIC-I. Einbinden: .include "lib/sound.asm"
// Vertrag wie auf allen Systemen:
//   sfx_init      Lautstaerke 15 (Hilfsfarbe weiss), alle Stimmen still
//   sfx_play      A = Effekt (SFX_*); startet ihn auf Stimme "Alt" oder "Rauschen"; bei sfx_mute <> 0 ohne Wirkung
//   sfx_update    einmal je Bild aufrufen: laesst den Effekt gleiten und ausklingen
//   mus_init      X/Y = Adresse (Low/High) des Liedkopfes; startet das Lied am Anfang
//   mus_tick      einmal je Bild; verwaltet Zeilenschritte und schreibt die VIC-I-Register
//   mus_stop      alle Stimmen still
//
// DER KLANGCHIP DES VC-20 (VIC-I, Register $900A..$900E):
//   Drei Rechteckstimmen (Bass $900A, Alt $900B, Sopran $900C) und ein Rauschgenerator ($900D); Bit 7 = Stimme an, Bits 0..6 = Tonhoehe N (0..126, groesser =
//   hoeher).  Frequenz f = Takt * m / (256 * (127 - N)) mit m = 1 (Bass), 2 (Alt), 4 (Sopran), PAL-Takt 1 108 405 Hz.  $900E: Bits 0..3 Lautstaerke (gemeinsam),
//   Bits 4..7 Hilfsfarbe fuer Mehrfarbzeichen (hier immer 1 = weiss; deshalb wird $900E immer ganz geschrieben).  Es gibt keine Huellkurven.
//
// AUFTEILUNG: Melodie auf Sopran, Bass auf Bass, Arpeggio auf Alt, Schlagzeug als Rauschstoss.  Klangeffekte nehmen Alt (Toene) bzw. Rauschen; solange ein
//   Effekt Alt belegt, ruht das Arpeggio.  Im Spiel laeuft keine Musik.
// LIEDFORMAT: siehe examples/c64/prallball/lib/sound.asm (identisch; Erzeuger: tools/compose.py der Spieldatei, hier auf wenige Takte gekuerzt).
// Zeropage (4 Byte, -D MUS_ZP=... verschiebbar): mus_ptr (Zeiger), mus_hdr (Liedkopf).
// ---------------------------------------------------------------------------------------------------------------------
.const SFX_PADDLE = 0
.const SFX_WALL   = 1
.const SFX_BRICK  = 2
.const SFX_HARD   = 3
.const SFX_LOSE   = 4
.const SFX_EXTRA  = 5
.const SFX_START  = 6
.const SFX_LASER  = 7

.const VIC_BASS    = $900a
.const VIC_ALTO    = $900b
.const VIC_SOPR    = $900c
.const VIC_NOISE   = $900d
.const VIC_VOLUME  = $900e

.ifndef MUS_ZP
.const MUS_ZP = $04
.endif
.const mus_ptr = MUS_ZP
.const mus_hdr = MUS_ZP + 2

sfx_init:
        lda #$1f                        // Hilfsfarbe 1 (weiss), Lautstaerke 15
        sta VIC_VOLUME
        lda #0
        sta VIC_BASS
        sta VIC_ALTO
        sta VIC_SOPR
        sta VIC_NOISE
        sta sfx_t
        rts

// Effekte: Tonhoehenwert (Bit 7 = an), Gleiten je Bild (signed), Dauer in Bildern; Rauschen oder Alt.
sfx_play:
        ldx sfx_mute                    // Demo-Modus: keine Effekte
        bne .mute
        tax
        lda sfx_val,x
        sta sfx_cur
        lda sfx_len,x
        sta sfx_t
        stx sfx_id
        lda sfx_noise,x
        sta sfx_isn
        beq +
        lda sfx_cur
        sta VIC_NOISE
        rts
+       lda sfx_cur
        sta VIC_ALTO
.mute:  rts

sfx_update:
        lda sfx_t
        beq .done
        dec sfx_t
        bne .glide
        lda #0                          // Effekt zu Ende: Stimme aus
        ldx sfx_isn
        beq +
        sta VIC_NOISE
        rts
+       sta VIC_ALTO
        rts
.glide: ldx sfx_id
        lda sfx_slide,x
        clc
        adc sfx_cur
        ora #$80                        // Bit 7 (an) bleibt gesetzt
        sta sfx_cur
        ldx sfx_isn
        beq +
        sta VIC_NOISE
        rts
+       sta VIC_ALTO
.done:  rts

//                 paddle wall brick hard lose  extra start laser
sfx_val:   .byte   $e4,   $dc, $9c,  $a4, $f0,  $d0,  $d8,  $f4     // Tonhoehe (Rauschen: Klangfarbe)
sfx_len:   .byte   3,     2,   4,    7,   24,   12,   8,    4
sfx_slide: .byte   0,     0,   $fe,  $fd, $fc,  3,    2,    $fa     // je Bild zur Tonhoehe addiert (signed)
sfx_noise: .byte   0,     0,   1,    1,   0,    0,    0,    0
sfx_mute:  .byte 0                      // <> 0: sfx_play tut nichts (Demo-Modus)
sfx_t:     .byte 0
sfx_id:    .byte 0
sfx_cur:   .byte 0
sfx_isn:   .byte 0

// ---------------------------------------------------------------------------------------------------------------------
// Musik.  Einmal je Bild mus_tick.  Eine Zeile = 1/16-Note; Dauer in Bildern im Liedkopf.
// ---------------------------------------------------------------------------------------------------------------------
mus_init:
        stx mus_hdr
        sty mus_hdr + 1
        lda #0
        sta mus_bar
        sta mus_row
        sta mus_tk
        sta mus_dr
        rts

mus_stop:
        lda #0
        sta VIC_BASS
        sta VIC_SOPR
        sta VIC_NOISE
        lda sfx_t
        bne +
        sta VIC_ALTO
+       rts

mus_tick:
        lda mus_dr                      // laufender Rauschstoss
        beq .nodr
        dec mus_dr
        bne .nodr
        lda #0
        sta VIC_NOISE
.nodr:  lda mus_tk
        bne .mnext
        jsr mus_row_start
.mnext: inc mus_tk
        ldy #0
        lda mus_tk
        cmp (mus_hdr),y                 // Bilder je Zeile
        bcc .mt9
        lda #0
        sta mus_tk
        inc mus_row
        lda mus_row
        cmp #16
        bcc .mt9
        lda #0                          // naechster Takt
        sta mus_row
        inc mus_bar
        lda mus_bar
        ldy #1
        cmp (mus_hdr),y                 // Anzahl Takte
        bcc .mt9
        lda #0
        sta mus_bar                     // Lied zu Ende: von vorn
.mt9:   rts

// mus_row_start: fuer die drei Notenspuren das Zeilenbyte holen und auswerten (Spur 0 Sopran, 1 Bass, 2 Alt/Schlagzeug).
mus_row_start:
        ldx #0                          // X = Spur 0..2
.rv:    lda mus_bar                     // Mustername aus der Reihenfolge: Kopf + 5 + Takt * 3 + Spur
        asl
        clc
        adc mus_bar
        clc
        adc #5
        stx mus_x
        adc mus_x
        tay
        lda (mus_hdr),y
        sta mus_x
        lda #0
        sta mus_ptr + 1
        lda mus_x
        asl
        rol mus_ptr + 1
        asl
        rol mus_ptr + 1
        asl
        rol mus_ptr + 1
        asl
        rol mus_ptr + 1                 // (mus_ptr+1 : A) = Name * 16
        clc
        adc mus_row
        sta mus_ptr
        ldy #3
        clc
        lda mus_ptr
        adc (mus_hdr),y                 // + Abstand der Mustertabelle
        sta mus_ptr
        iny
        lda mus_ptr + 1
        adc (mus_hdr),y
        sta mus_ptr + 1
        clc
        lda mus_ptr
        adc mus_hdr
        sta mus_ptr
        lda mus_ptr + 1
        adc mus_hdr + 1
        sta mus_ptr + 1
        ldy #0
        lda (mus_ptr),y                 // A = Zeilenbyte
        beq .rnext                      // 0: halten
        stx mus_x
        cmp #96
        beq .rest
        bcs .drum
        tay                             // Note 1..95
        cpx #0
        bne .n1
        lda note_vic_sopr,y
        sta VIC_SOPR
        jmp .rnext1
.n1:    cpx #1
        bne .n2
        lda note_vic_bass,y
        sta VIC_BASS
        jmp .rnext1
.n2:    lda sfx_t                       // Alt ist von einem Effekt belegt?
        bne .rnext1
        lda note_vic_alto,y
        sta VIC_ALTO
        jmp .rnext1
.rest:  lda #0
        cpx #0
        bne .r1
        sta VIC_SOPR
        jmp .rnext1
.r1:    cpx #1
        bne .r2
        sta VIC_BASS
        jmp .rnext1
.r2:    ldy sfx_t
        bne .rnext1
        sta VIC_ALTO
        jmp .rnext1
.drum:  cpx #2
        bne .rnext1                     // Schlagzeug nur auf Spur 3
        and #$7f
        tax                             // X = Schlag 0..5
        lda mus_dnoise,x
        sta VIC_NOISE
        lda mus_dlen,x
        sta mus_dr
.rnext1: ldx mus_x
.rnext: inx
        cpx #3
        bne .rv
        rts

// Schlagzeug (Index 0..5): Bassdrum, Snare, Hi-Hat zu, Hi-Hat offen, Tom hoch, Tom tief.  Rauschen: Tonhoehe = Klangfarbe, Dauer in Bildern.
mus_dnoise: .byte $a0, $d0, $f0, $ea, $b8, $a8
mus_dlen:   .byte 4, 3, 1, 2, 3, 3

mus_bar:    .byte 0
mus_row:    .byte 0
mus_tk:     .byte 0
mus_x:      .byte 0
mus_dr:     .byte 0

// Frequenztabelle: aus <vic20/notes.inc> (ERZEUGT durch tools/gen_notes.py, mit -D NTSC=1 die NTSC-Tabelle).  Indiziert mit dem Notenwert (C0 = 1, Eintrag 0 = keine Note).
// Tonhöhenwerte ($80 | N): Sopran, Bass, Alt; tiefe Töne oktaviert nach oben.
#include <vic20/notes.inc>
