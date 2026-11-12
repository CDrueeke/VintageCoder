// SPDX-License-Identifier: GPL-3.0-or-later
// Klang Atari 8 Bit (PRALLBALL): Effekte auf POKEY-Kanal 4, Titelmusik auf allen vier Kanälen.
// Einbinden: .include "lib/sound.asm"   (setzt POKEY-Register aus <atari8bit/pokey.asm> voraus; hier mit eigenen Namen, damit die Datei allein steht)
//
//   sfx_init      POKEY einrichten (AUDCTL = 0, SKCTL = 3), alle Kanäle stumm
//   sfx_play      A = Effekt (SFX_*); startet ihn auf Kanal 4 (bei sfx_mute <> 0 ohne Wirkung)
//   sfx_update    einmal je Bild: lässt den Effekt ausklingen (Lautstärke fällt, Tonhöhe gleitet)
//   mus_init      X/Y = Adresse (Low/High) des Liedkopfes; startet das Lied am Anfang; schaltet Kanal 3+4 zu einem 16-Bit-Kanal (Bass)
//   mus_tick      einmal je Bild: Zeilenschritt, Klangfarben, Hüllkurven
//   mus_stop      alles still, AUDCTL = 0 (Kanäle wieder einzeln)
//
// Liedformat: wie in examples/c64/prallball/lib/sound.asm (Kopf: Bilder je Zeile, Takte, Wiederholen, Abstand; Reihenfolge je Takt 3 Muster; Muster je 16 Zeilen;
// Zeilenbyte 0 halten, 1..95 Note (C0 = 1), 96 Pause, 128 + n Schlagzeug n).  Die Zuordnung der Stimmen zu den POKEY-Kanälen:
//   Stimme 1 (Melodie)  -> Kanal 1, 8 Bit, Takt 64 kHz (Töne ab etwa 125 Hz), Verzerrung $A0 (reiner Ton)
//   Stimme 2 (Bass)     -> Kanal 3+4 zusammengeschaltet (16 Bit, Takt 1,79 MHz, AUDCTL $28): die Teiler stehen in AUDF3 (niedrig) und AUDF4 (hoch);
//                          die Lautstärke bestimmt AUDC4
//   Stimme 3 (Arpeggio/Schlag) -> Kanal 2, 8 Bit: Töne bei Arpeggio, Rauschen ($80) bei Snare/Hi-Hat, Ton mit fallender Tonhöhe bei Bassdrum/Toms
// Hüllkurven: der POKEY hat keine; wir senken die Lautstärke (0..15) je Bild in Schritten (mus_dr: alle n Bilder um 1) bis zu einem Sockel (mus_fl).
// Zeropage (2 Byte, mit -D MUS_ZP=... verschiebbar): mus_ptr (Zeiger), mus_hdr (Liedkopf, 2 Byte)  = 4 Byte
.const SND_AUDF1 = $d200
.const SND_AUDC1 = $d201
.const SND_AUDF2 = $d202
.const SND_AUDC2 = $d203
.const SND_AUDF3 = $d204
.const SND_AUDC3 = $d205
.const SND_AUDF4 = $d206
.const SND_AUDC4 = $d207
.const SND_AUDCTL = $d208
.const SND_SKCTL = $d20f

.const SFX_PADDLE = 0
.const SFX_WALL   = 1
.const SFX_BRICK  = 2
.const SFX_HARD   = 3
.const SFX_LOSE   = 4
.const SFX_EXTRA  = 5
.const SFX_START  = 6
.const SFX_LASER  = 7

.ifndef MUS_ZP
.const MUS_ZP = $d4
.endif
.const mus_ptr = MUS_ZP
.const mus_hdr = MUS_ZP + 2

sfx_init:
        lda #0
        sta SND_AUDCTL                  // alle Kanäle einzeln, Takt 64 kHz
        ldx #7
.si1:   sta SND_AUDF1,x                 // $D200..$D207: Frequenzen und Lautstärken löschen
        dex
        bpl .si1
        lda #3
        sta SND_SKCTL                   // POKEY aus dem Initialisierungszustand holen (sonst steht der Zufallsgenerator und die Tastatur)
        lda #0
        sta sfx_t
        rts

// sfx_play: Teiler, Verzerrung + Lautstärke und Dauer aus den Tabellen; AUDC4 = Verzerrung (Bits 5..7) + Lautstärke (Bits 0..3).
sfx_play:
        ldx sfx_mute                    // Demo-Modus: keine Effekte
        bne .mute
        tax
        lda sfx_div,x
        sta SND_AUDF4
        lda sfx_dist,x
        ora #12                         // Anfangslautstärke 12
        sta SND_AUDC4
        lda sfx_len,x
        sta sfx_t
        stx sfx_id
.mute:  rts

// sfx_update: je Bild die Restdauer senken; die Lautstärke folgt der Restdauer (ausklingend), die Tonhöhe gleitet um sfx_slide.
sfx_update:
        lda sfx_t
        beq .su9
        dec sfx_t
        bne .su1
        lda #0
        sta SND_AUDC4                   // Ende: Kanal 4 stumm
        rts
.su1:   ldx sfx_id
        lda sfx_t
        cmp #12
        bcc +
        lda #12
+       sta sfx_vol                     // Lautstärke = min(12, Restdauer)
        lda sfx_dist,x
        ora sfx_vol
        sta SND_AUDC4
        lda SND_AUDF4_COPY              // Teiler gleiten: (Kopie, weil AUDF4 beim POKEY nicht lesbar ist)
        clc
        adc sfx_slide,x
        sta SND_AUDF4_COPY
        sta SND_AUDF4
.su9:   rts

//                 paddle wall brick hard lose extra start laser
sfx_div:   .byte   $50,   $70,  $20,  $30, $60,  $30,  $40, $20
sfx_dist:  .byte   $a0,   $a0,  $80,  $80, $a0,  $a0,  $a0, $a0     // $A0 reiner Ton, $80 Rauschen
sfx_len:   .byte   4,     3,    5,    8,   30,   14,   10,  4
sfx_slide: .byte   0,     0,    $02,  $01, $02,  $fe,  $ff, $04     // Teiler je Bild (vorzeichenbehaftet)
sfx_t:     .byte 0
sfx_id:    .byte 0
sfx_vol:   .byte 0
sfx_mute:  .byte 0
SND_AUDF4_COPY: .byte 0

// ---------------------------------------------------------------------------------------------------------------------
// Musik
// ---------------------------------------------------------------------------------------------------------------------
mus_init:
        stx mus_hdr
        sty mus_hdr + 1
        lda #0
        sta mus_bar
        sta mus_row
        sta mus_tk
        ldx #3
.mi0:   sta mus_pend,x
        sta mus_vol,x
        sta mus_cnt,x
        dex
        bpl .mi0
        lda #$28                        // Kanal 3 mit 1,79 MHz getaktet (Bit 5) und mit Kanal 4 zu 16 Bit verbunden (Bit 3)
        sta SND_AUDCTL
        lda #0
        sta SND_AUDC3                   // Kanal 3 selbst bleibt stumm; der 16-Bit-Ton kommt von Kanal 4
        rts

mus_stop:
        lda #0
        sta SND_AUDC1
        sta SND_AUDC2
        sta SND_AUDC3
        sta SND_AUDC4
        sta SND_AUDCTL                  // Kanäle wieder einzeln (Effekte nutzen Kanal 4 mit 8 Bit)
        rts

mus_tick:
        // Hüllkurven und Tonhöhenglide (je Bild)
        ldx #2
.mt1:   lda mus_vol,x
        cmp mus_fl,x
        bcc .mt2                        // schon unter dem Sockel
        beq .mt2
        dec mus_cnt,x
        bpl .mt2
        lda mus_dr,x
        sta mus_cnt,x
        dec mus_vol,x                   // Lautstärke um 1 senken
.mt2:   dex
        bpl .mt1
        lda mus_slc                     // Bassdrum/Tom: Tonhöhe fällt (Teiler steigt)
        beq .mt3
        dec mus_slc
        lda mus_fh
        clc
        adc mus_sl
        sta mus_fh
        sta SND_AUDF2
.mt3:   jsr mus_write
        lda mus_tk
        bne .m1                         // nicht Zeilenanfang
        jsr mus_row_start
        jmp .mnext
.m1:    cmp #1
        bne .mnext
        jsr mus_write                   // Bild 1: Töne hören (Wert wurde in Bild 0 gesetzt)
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
        lda #0
        sta mus_row
        inc mus_bar
        lda mus_bar
        ldy #1
        cmp (mus_hdr),y                 // Anzahl Takte
        bcc .mt9
        lda #0
        sta mus_bar                     // Lied zu Ende: von vorn
.mt9:   rts

// mus_write: Lautstärken in die AUDC-Register (Verzerrung + Lautstärke) schreiben.  Stimme 3 hat je nach Schlag eine andere Verzerrung (mus_d3).
mus_write:
        lda mus_vol
        ora #$a0
        sta SND_AUDC1
        lda mus_vol + 1
        ora #$a0
        sta SND_AUDC4
        lda mus_vol + 2
        ora mus_d3
        sta SND_AUDC2
        rts

// mus_row_start: für alle drei Stimmen das Zeilenbyte holen und auswerten.
mus_row_start:
        ldx #0
.rv:    lda mus_bar
        asl
        clc
        adc mus_bar                     // Takt * 3
        clc
        adc #5
        stx mus_x
        adc mus_x
        tay
        lda (mus_hdr),y                 // Mustername
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
        adc (mus_hdr),y
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
        sta mus_ptr + 1                 // + Kopfadresse
        ldy #0
        lda (mus_ptr),y                 // Zeilenbyte
        bne .rn0
        jmp .rnext                      // 0: halten (Sprung statt Verzweigung: das Ziel liegt zu weit weg)
.rn0:
        stx mus_x
        cmp #96
        beq .rest
        bcs .drum
        tay                             // Note 1..95
        lda mus_x
        beq .v1
        cmp #1
        beq .v2
        // Stimme 3 (Kanal 2): Arpeggio, Ton (reine Verzerrung)
        lda note_pokey8,y
        sta SND_AUDF2
        lda #$a0
        sta mus_d3
        lda #8
        sta mus_vol + 2
        lda #0
        sta mus_slc
        jmp .rnext1
.v1:    lda note_pokey8,y                // Stimme 1 (Kanal 1): Melodie
        sta SND_AUDF1
        lda #12
        sta mus_vol
        jmp .rnext1
.v2:    lda note_pokey16_lo,y             // Stimme 2 (Kanal 3+4): Bass, 16-Bit-Teiler
        sta SND_AUDF3
        lda note_pokey16_hi,y
        sta SND_AUDF4
        lda #14
        sta mus_vol + 1
        jmp .rnext1
.rest:  ldx mus_x
        lda #0
        sta mus_vol,x
        jmp .rnext1
.drum:  and #$7f                        // Schlag n (nur Stimme 3)
        tay
        lda mus_dfreq,y
        sta SND_AUDF2
        sta mus_fh
        lda mus_ddist,y
        sta mus_d3
        lda mus_dvol,y
        sta mus_vol + 2
        lda mus_dsl,y
        sta mus_sl
        beq .nogl
        lda #6
        sta mus_slc
.nogl:  lda #1
        sta mus_cnt + 2
.rnext1: ldx mus_x
.rnext: inx
        cpx #3
        beq .rend
        jmp .rv
.rend:  rts

mus_dr:    .byte 3, 2, 2, 0              // alle n Bilder Lautstärke - 1 (0 = jedes Bild)
mus_fl:    .byte 6, 4, 0, 0              // Sockel
mus_dfreq: .byte $50, $10, $04, $06, $40, $60     // Bassdrum (Ton, Teiler fällt), Snare (Rauschen), Hi-Hat zu, Hi-Hat offen, Tom hoch, Tom tief
mus_ddist: .byte $a0, $80, $80, $80, $a0, $a0
mus_dvol:  .byte 14, 12, 7, 9, 12, 12
mus_dsl:   .byte 24, 0, 0, 0, 12, 10            // Teilerzuwachs je Bild bei den Tönen
mus_d3:    .byte $a0
mus_fh:    .byte 0
mus_sl:    .byte 0
mus_slc:   .byte 0
mus_bar:   .byte 0
mus_row:   .byte 0
mus_tk:    .byte 0
mus_x:     .byte 0
mus_pend:  .byte 0, 0, 0, 0
mus_vol:   .byte 0, 0, 0, 0
mus_cnt:   .byte 0, 0, 0, 0

// Frequenztabelle: aus <atari8bit/notes.inc> (ERZEUGT durch tools/gen_notes.py, mit -D NTSC=1 die NTSC-Tabelle).  Indiziert mit dem Notenwert (C0 = 1, Eintrag 0 = keine Note).
// 8 Bit bei 64 kHz, 16 Bit bei vollem Takt (Kanäle 3+4 gekoppelt); PAL- und NTSC-Takt getrennt (früher stand hier der NTSC-Takt als „PAL“).
#include <atari8bit/notes.inc>
