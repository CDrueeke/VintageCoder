// SPDX-License-Identifier: GPL-3.0-or-later
// Klang Plus/4 (PRALLBALL): Effekte und Titelmusik auf den zwei Stimmen des TED. Einbinden: .include "lib/sound.asm"
// Vertrag wie auf allen Systemen:
//   sfx_init      TED-Klang einrichten (Lautstaerke 8, beide Stimmen stumm)
//   sfx_play      A = Effekt (SFX_*); startet ihn auf Stimme 2; bei sfx_mute <> 0 ohne Wirkung
//   sfx_update    einmal je Bild aufrufen: lässt den Effekt gleiten und ausklingen
//   mus_init      X/Y = Adresse (Low/High) des Liedkopfes; startet das Lied am Anfang
//   mus_tick      einmal je Bild; verwaltet Zeilenschritte und schreibt die TED-Register
//   mus_stop      alle Stimmen still
//
// DER KLANGCHIP DES PLUS/4 (TED, Register $FF0E..$FF12):
//   Der TED hat nur ZWEI Stimmen und KEINE Huellkurven.  Stimme 1 ist ein Rechteck; Stimme 2 ist wahlweise Rechteck oder Rauschen.  Die
//   Lautstaerke ($FF11 Bits 0..3, Werte 0..8; groesser als 8 klingt wie 8) gilt fuer beide Stimmen gemeinsam.
//   $FF11: Bit 4 = Stimme 1 an, Bit 5 = Stimme 2 an, Bit 6 = Stimme 2 Rauschen statt Rechteck, Bit 7 = Direktausgabe (nicht benutzt).
//   Tonhoehe: 10-Bit-Wert N je Stimme; Frequenz f = 110840,5 / (1024 - N) Hz (PAL).  Je groesser N, desto hoeher der Ton.
//     Stimme 1: niedrige 8 Bit in $FF0E, hohe 2 Bit in $FF12 Bits 0..1 (die uebrigen Bits von $FF12 waehlen den Zeichensatz: erhalten!).
//     Stimme 2: niedrige 8 Bit in $FF0F, hohe 2 Bit in $FF10 Bits 0..1.
//   Weil der TED keine Huellkurven kennt, wird jeder neue Ton durch ein kurzes Abschalten der Stimme (ein Bild) vom vorigen getrennt.
//
// AUFTEILUNG: Die Musik spielt Melodie auf Stimme 1 und Bass auf Stimme 2.  Schlagzeug ist ein kurzer Rauschstoss auf Stimme 2 (der Bass
//   setzt dabei kurz aus).  Die dritte Notenspur des Liedes (Arpeggio) entfaellt.  Klangeffekte nehmen Stimme 2; solange ein Effekt laeuft,
//   bleibt die Stimme 2 fuer Bass und Schlagzeug gesperrt.  Im Spiel laeuft keine Musik, auf dem Titel sind Effekte selten.
// Zeropage (4 Byte, -D MUS_ZP=... verschiebbar): mus_ptr (Zeiger), mus_hdr (Liedkopf).
// LIEDFORMAT: siehe examples/c64/prallball/lib/sound.asm (identisch; Erzeuger: tools/compose.py der Spieldatei).
// ---------------------------------------------------------------------------------------------------------------------
.const SFX_PADDLE = 0
.const SFX_WALL   = 1
.const SFX_BRICK  = 2
.const SFX_HARD   = 3
.const SFX_LOSE   = 4
.const SFX_EXTRA  = 5
.const SFX_START  = 6
.const SFX_LASER  = 7

.const TED_V1LO  = $ff0e
.const TED_V2LO  = $ff0f
.const TED_V2HI  = $ff10
.const TED_VOL   = $ff11
.const TED_V1HI  = $ff12                // Bits 0..1 = Tonhoehe Stimme 1 hoch; Bit 2 und die hoeheren Bits gehoeren dem Zeichensatz

.ifndef MUS_ZP
.const MUS_ZP = $04
.endif
.const mus_ptr = MUS_ZP
.const mus_hdr = MUS_ZP + 2

sfx_init:
        lda #0
        sta sfx_t
        sta ted_vol                     // beide Stimmen aus
        sta mus_dr
        sta mus_b_on
        jmp snd_flush

// snd_flush: ted_vol (Bits 4..6) mit Lautstaerke 8 ins Register schreiben.
snd_flush:
        lda ted_vol
        and #$70
        ora #$08
        sta TED_VOL
        rts

// v1_set / v2_set: Tonhoehe aus (A = N niedrig, Y = N hoch) in die Register der Stimme schreiben.  Veraendert A.
v1_set: sta TED_V1LO
        lda TED_V1HI
        and #$fc
        sta mus_tmp
        tya
        ora mus_tmp
        sta TED_V1HI
        rts
v2_set: sta TED_V2LO
        sty TED_V2HI
        rts

// ---------------------------------------------------------------------------------------------------------------------
// Klangeffekte auf Stimme 2.  Je Effekt: Anfangston N (10 Bit), Gleiten je Bild (16 Bit, vorzeichenbehaftet, in N-Einheiten), Dauer in
// Bildern, Rauschen oder Rechteck.
// ---------------------------------------------------------------------------------------------------------------------
sfx_play:
        ldx sfx_mute                    // Demo-Modus: keine Effekte (die Musik laeuft unabhaengig weiter)
        bne .mute
        tax
        lda sfx_nlo,x
        sta sfx_cl
        lda sfx_nhi,x
        sta sfx_ch
        lda sfx_len,x
        sta sfx_t
        stx sfx_id
        lda ted_vol
        and #$9f                        // Rauschbit und Stimme 2 zunaechst loeschen
        sta ted_vol
        lda sfx_noise,x
        beq +
        lda ted_vol
        ora #$40                        // Rauschen
        sta ted_vol
+       lda ted_vol
        ora #$20                        // Stimme 2 an
        sta ted_vol
        lda sfx_cl
        ldy sfx_ch
        jsr v2_set
        jsr snd_flush
.mute:  rts

sfx_update:
        lda sfx_t
        beq .done
        dec sfx_t
        bne .glide
        lda ted_vol                     // Effekt zu Ende: Stimme 2 aus, Rauschen aus; der Bass uebernimmt beim naechsten Ton wieder
        and #$9f
        sta ted_vol
        jsr snd_flush
        rts
.glide: ldx sfx_id
        ldy #0
        lda sfx_slide,x                 // Gleiten je Bild: signed 8 Bit, auf 16 Bit erweitert
        bpl +
        dey                             // negativ: High-Byte der Erweiterung = $FF
+       clc
        adc sfx_cl
        sta sfx_cl
        tya
        adc sfx_ch
        and #$03
        sta sfx_ch
        lda sfx_cl
        ldy sfx_ch
        jsr v2_set
.done:  rts

//                 paddle wall brick hard lose  extra start laser
sfx_nlo:   .byte   $cd,   $32, $20,  $bc, $1a,  $e0,  $a0,  $b0     // N = 717, 562, 800 (Rauschen), 700 (Rauschen), 794, 1000, 928, 944
sfx_nhi:   .byte   $02,   $02, $03,  $02, $03,  $03,  $03,  $03
sfx_len:   .byte   4,     3,   5,    8,   30,   14,   10,   4
sfx_slide: .byte   0,     0,   $fa,  $fd, $fc,  $05,  $03,  $ec    // je Bild zu N addiert (signed): Rauschen sinkt, "lose" faellt, "extra" steigt
sfx_noise: .byte   0,     0,   1,    1,   0,    0,    0,    0
sfx_mute:  .byte 0                      // <> 0: sfx_play tut nichts (Demo-Modus)
sfx_t:     .byte 0
sfx_id:    .byte 0
sfx_cl:    .byte 0                      // laufende Tonhoehe des Effekts (N, niedrig/hoch)
sfx_ch:    .byte 0
ted_vol:   .byte 0                      // Schattenwert von $FF11: Bit 4 Stimme 1 an, Bit 5 Stimme 2 an, Bit 6 Rauschen
mus_tmp:   .byte 0

// ---------------------------------------------------------------------------------------------------------------------
// Musik.  Einmal je Bild mus_tick.  Zeilenbeginn (Bild 0): neue Toene setzen, die betroffenen Stimmen abschalten; Bild 1: wieder einschalten
// (das trennt aufeinanderfolgende gleiche Toene, die ohne Huellkurve sonst ineinanderflossen).
// ---------------------------------------------------------------------------------------------------------------------
mus_init:
        stx mus_hdr
        sty mus_hdr + 1
        lda #0
        sta mus_bar
        sta mus_row
        sta mus_tk
        sta mus_pend
        sta mus_dr
        sta mus_b_on
        sta ted_vol
        jmp snd_flush

mus_stop:
        lda #0
        sta ted_vol
        sta mus_pend
        sta mus_dr
        sta mus_b_on
        jmp snd_flush

mus_tick:
        lda mus_dr                      // laufender Schlagzeugstoss auf Stimme 2
        beq .nodr
        dec mus_dr
        bne .nodr
        lda ted_vol                     // Stoss zu Ende: Rauschen aus, Bass wieder an (falls er klingt)
        and #$bf
        sta ted_vol
        jsr snd_flush
        lda sfx_t
        bne .nodr                       // ein Effekt belegt Stimme 2
        lda mus_b_on
        beq .bq
        lda ted_vol
        ora #$20
        sta ted_vol
        lda mus_b_lo
        ldy mus_b_hi
        jsr v2_set
        jsr snd_flush
        jmp .nodr
.bq:    lda ted_vol
        and #$df
        sta ted_vol
        jsr snd_flush
.nodr:  lda mus_tk
        bne .m1                         // nicht Zeilenanfang
        jsr mus_row_start               // Bild 0 der Zeile: Toene setzen, Stimmen aus
        jmp .mflush
.m1:    cmp #1
        bne .mnext
        lda ted_vol                     // Bild 1: ausstehende Stimmen einschalten
        ora mus_pend
        sta ted_vol
        lda #0
        sta mus_pend
.mflush: jsr snd_flush
.mnext: inc mus_tk                      // Zeilenzaehler (in Bildern); am Zeilenende die naechste Zeile vorbereiten
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

// mus_row_start: fuer die drei Notenspuren das Zeilenbyte holen und auswerten.
mus_row_start:
        ldx #0                          // X = Spur 0..2
.rv:    // Mustername aus der Reihenfolge: Kopf + 5 + Takt * 3 + Spur
        lda mus_bar
        asl
        clc
        adc mus_bar                     // Takt * 3
        clc
        adc #5
        stx mus_x
        adc mus_x
        tay
        lda (mus_hdr),y                 // A = Mustername
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
        adc (mus_hdr),y                 // + Abstand der Mustertabelle (Low)
        sta mus_ptr
        iny
        lda mus_ptr + 1
        adc (mus_hdr),y                 // (High)
        sta mus_ptr + 1
        clc
        lda mus_ptr
        adc mus_hdr
        sta mus_ptr
        lda mus_ptr + 1
        adc mus_hdr + 1
        sta mus_ptr + 1                 // + Kopfadresse
        ldy #0
        lda (mus_ptr),y                 // A = Zeilenbyte
        beq .rnext                      // 0: halten
        stx mus_x
        cmp #96
        beq .rest
        bcs .drum
        // Note 1..95
        cpx #2
        beq .rnext                      // Spur 3 (Arpeggio) entfaellt auf dem TED
        tay
        lda note_ted_lo,y
        sta mus_nl
        lda note_ted_hi,y
        sta mus_nh
        cpx #0
        bne .bass
        lda ted_vol                     // Melodie: Stimme 1 kurz aus, Ton setzen, Bild 1 wieder an
        and #$ef
        sta ted_vol
        lda mus_pend
        ora #$10
        sta mus_pend
        lda mus_nl
        ldy mus_nh
        jsr v1_set
        jmp .rnext1
.bass:  lda mus_nl
        sta mus_b_lo
        lda mus_nh
        sta mus_b_hi
        lda #1
        sta mus_b_on
        lda sfx_t
        ora mus_dr
        bne .rnext1                     // Stimme 2 belegt (Effekt oder Schlagzeug): der Bass merkt sich nur den Ton
        lda ted_vol
        and #$df
        sta ted_vol
        lda mus_pend
        ora #$20
        sta mus_pend
        lda mus_nl
        ldy mus_nh
        jsr v2_set
        jmp .rnext1
.rest:  cpx #2
        beq .rnext1
        cpx #0
        bne .brest
        lda ted_vol
        and #$ef
        sta ted_vol
        lda mus_pend
        and #$ef
        sta mus_pend
        jmp .rnext1
.brest: lda #0
        sta mus_b_on
        lda sfx_t
        ora mus_dr
        bne .rnext1
        lda ted_vol
        and #$df
        sta ted_vol
        lda mus_pend
        and #$df
        sta mus_pend
        jmp .rnext1
.drum:  cpx #2
        bne .rnext1                     // Schlagzeug nur auf Spur 3
        and #$7f
        tax                             // X = Schlag 0..5
        lda sfx_t
        bne .rnext1                     // Effekt belegt Stimme 2: Schlag entfaellt
        lda mus_dlen,x
        sta mus_dr
        lda ted_vol
        ora #$60                        // Stimme 2 an, Rauschen
        sta ted_vol
        lda mus_pend
        and #$df
        sta mus_pend
        lda mus_dlo,x
        ldy mus_dhi,x
        jsr v2_set
.rnext1: ldx mus_x
.rnext: inx
        cpx #3
        bne .rv
        rts

// Schlagzeug (Index 0..5): Bassdrum, Snare, Hi-Hat zu, Hi-Hat offen, Tom hoch, Tom tief.  Rauschen: Tonhoehe N bestimmt die Rauschfarbe.
mus_dlen:   .byte 4, 3, 1, 2, 3, 3                                  // Dauer des Rauschstosses in Bildern
mus_dlo:    .byte $40, $a0, $f0, $e0, $20, $00                      // N niedrig
mus_dhi:    .byte $01, $02, $03, $03, $02, $01                      // N hoch  (Bassdrum 320, Snare 672, Hat 1008, offen 992, Tom 544/256)

mus_bar:    .byte 0
mus_row:    .byte 0
mus_tk:     .byte 0
mus_x:      .byte 0
mus_pend:   .byte 0                     // Stimmen, die im naechsten Bild eingeschaltet werden ($10 Stimme 1, $20 Stimme 2)
mus_dr:     .byte 0                     // Rest-Bilder des Schlagzeugstosses
mus_b_on:   .byte 0                     // 1 = der Bass hat einen laufenden Ton
mus_b_lo:   .byte 0
mus_b_hi:   .byte 0
mus_nl:     .byte 0
mus_nh:     .byte 0

// Frequenztabelle: aus <plus4/notes.inc> (ERZEUGT durch tools/gen_notes.py, mit -D NTSC=1 die NTSC-Tabelle).  Indiziert mit dem Notenwert (C0 = 1, Eintrag 0 = keine Note).
// Töne unter 110 Hz (NOTE_TABLE_LOWEST) stehen um ganze Oktaven nach oben gefaltet (der TED erreicht bei N = 0 nur 108 Hz).
#include <plus4/notes.inc>
