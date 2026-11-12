// SPDX-License-Identifier: GPL-3.0-or-later
// Klang C64 (PRALLBALL): Effekte auf SID-Stimme 3. Einbinden: .include "lib/sound.asm"
// Die Titelmusik (mus_*) folgt in Phase P1 (Tracker-Export); hier sind nur Stubs, die den Vertrag festlegen.
//   sfx_init      SID stumm schalten, Lautstärke 15
//   sfx_play      A = Effekt (SFX_*); startet ihn auf Stimme 3; bei sfx_mute <> 0 ohne Wirkung
//   sfx_update    einmal je Bild aufrufen (Rasterinterrupt): lässt den Effekt ausklingen
//   mus_init / mus_tick / mus_stop   Stubs (kein Klang)
// Zeropage: keine; RAM: sfx_t (Restdauer in Bildern), sfx_id
.const SFX_PADDLE = 0
.const SFX_WALL   = 1
.const SFX_BRICK  = 2
.const SFX_HARD   = 3
.const SFX_LOSE   = 4
.const SFX_EXTRA  = 5
.const SFX_START  = 6
.const SFX_LASER  = 7

sfx_init:
        ldx #$18
        lda #0
-       sta $d400,x
        dex
        bpl -
        lda #$0f
        sta $d418
        lda #0
        sta sfx_t
        rts

sfx_play:
        ldx sfx_mute            // Demo-Modus: keine Effekte (die Musik laeuft unabhaengig weiter)
        bne .mute
        tax
        lda sfx_freq,x
        sta $d40f               // Stimme 3 Frequenz hoch
        lda #$00
        sta $d40e
        lda #$00
        sta $d413               // Attack 0, Decay 0
        lda #$a8
        sta $d414               // Sustain 10, Release 8
        lda sfx_wave,x
        sta $d412               // Wellenform + Gate
        lda sfx_len,x
        sta sfx_t
        stx sfx_id
.mute:  rts

sfx_update:
        lda sfx_t
        beq .done
        dec sfx_t
        bne .glide
        lda $d412
        and #$fe                // Gate aus: Release
        sta $d412
        rts
.glide: ldx sfx_id
        lda sfx_slide,x         // Tonhöhe je Bild verändern (Effekt-spezifisch, signed)
        clc
        adc $d40f
        sta $d40f
.done:  rts

//                 paddle wall brick hard lose  extra start
sfx_freq:  .byte   $18,   $10, $28,  $20, $20,  $30,  $14, $50
sfx_wave:  .byte   $41,   $41, $81,  $81, $21,  $11,  $11, $21    // Puls, Puls, Rauschen, Rauschen, Sägezahn, Dreieck, Dreieck
sfx_len:   .byte   4,     3,   5,    8,   30,   14,   10, 4
sfx_slide: .byte   0,     0,   $fe,  $ff, $ff,  2,    1, $fc
sfx_mute:  .byte 0                  // <> 0: sfx_play tut nichts (Demo-Modus)
sfx_t:     .byte 0
sfx_id:    .byte 0

// ---------------------------------------------------------------------------------------------------------------------
// Musik (Titelmusik, 3 Stimmen): ein einfacher Muster-Spieler.  Einmal je Bild (50 Hz) mus_tick aufrufen.
//   mus_init   X/Y = Adresse (Low/High) der Liedkopfes; startet das Lied am Anfang
//   mus_tick   einmal je Bild; verwaltet Zeilenschritte (Dauer siehe Kopf) und schreibt die SID-Register
//   mus_stop   alle Stimmen still (Gate aus), Filter aus
//
// LIEDFORMAT (vom Werkzeug tools/compose.py der Spieldatei erzeugt):
//   Kopf:  +0 Bilder je Zeile (Zeile = 1/16-Note)   +1 Takte (Anzahl)   +2 Wiederholen (1 = von vorn)   +3/+4 Abstand der Mustertabelle
//          ab +5: Reihenfolge: je Takt 3 Byte = Mustername der Stimmen 1, 2, 3
//   Muster: 16 Byte (eine Zeile je Byte) ab Kopf + Abstand + Name * 16.  Zeilenbyte:
//          0 = halten, 1..95 = neue Note (Halbtonindex ab C0 = 1 = NOTE_C0, zugleich Index in der Frequenztabelle), 96 = Pause (Gate aus),
//          128 + n = Schlagzeug n (0 Bassdrum, 1 Snare, 2 Hi-Hat zu, 3 Hi-Hat offen, 4 Tom hoch, 5 Tom tief; nur Stimme 3)
// Klangfarben je Stimme sind hier fest: 1 = Puls mit Pulsbreitenmodulation (Melodie), 2 = Saege mit Tiefpass (Bass), 3 = Dreieck
// (Arpeggio) bzw. Schlagzeug (Bassdrum/Toms = Dreieck mit fallender Tonhoehe, Snare/Hi-Hat = Rauschen mit eigener Huellkurve).
// Warum zweistufig (Zeilenanfang, dann ein Bild spaeter): Fuer einen neuen Ton muss das Gate erst fallen und dann steigen, damit
//   die Huellkurve neu startet.  Bild 0 der Zeile setzt Frequenz und Gate AUS, Bild 1 setzt das Gate AN.
// Zeropage (4 Byte, mit -D MUS_ZP=... verschiebbar): mus_ptr (Zeiger), mus_hdr (Liedkopf)
// ---------------------------------------------------------------------------------------------------------------------
.ifndef MUS_ZP
.const MUS_ZP = $e8
.endif
.const mus_ptr = MUS_ZP
.const mus_hdr = MUS_ZP + 2

mus_init:
        stx mus_hdr
        sty mus_hdr + 1
        lda #0
        sta mus_bar
        sta mus_row
        sta mus_tk
        sta mus_pend
        sta mus_pend + 1
        sta mus_pend + 2
        ldx #0                          // Klangfarben einrichten: Pulsbreite $0800, Huellkurven
.mi:    lda mus_ad,x
        ldy mus_sidoff,x
        sta $d405,y                     // Attack/Decay
        lda mus_sr,x
        sta $d406,y                     // Sustain/Release
        lda #$00
        sta $d402,y
        lda #$08
        sta $d403,y                     // Pulsbreite $0800 = 50 %
        inx
        cpx #3
        bne .mi
        // Filter: Tiefpass auf dem Bass (Stimme 2), Resonanz 15.  $D417: Bits 4..7 = Resonanz, Bit 0..2 = Stimmen 1..3 durch den Filter.
        //   $D418: Bit 4 = Tiefpass, Bits 0..3 = Lautstaerke.  Die Grenzfrequenz ($D416) wird in mus_tick gefahren (Wah-Wah).
        lda #$f2
        sta $d417
        lda #$1f
        sta $d418
        lda #0
        sta mus_fc
        rts

mus_stop:
        lda #0
        sta $d404
        sta $d40b
        sta $d412                       // Steuerregister aller drei Stimmen: Gate aus
        sta $d417                       // Filter aus
        lda #$0f
        sta $d418                       // nur Lautstaerke 15, kein Tiefpass
        rts

mus_tick:
        lda mus_slc                     // laufende Tonhoehenglide der Stimme 3 (Bassdrum/Toms)
        beq .nsl
        dec mus_slc
        sec
        lda mus_fh
        sbc mus_sl
        sta mus_fh
        sta $d40f
.nsl:   lda mus_fc                      // Pulsbreitenmodulation der Melodiestimme: 5..12 im Takt der Filterfahrt
        bpl .pw1
        eor #$ff
.pw1:   lsr
        lsr
        lsr
        lsr
        clc
        adc #5
        sta $d403
        inc mus_fc                      // Filterfahrt: Dreieckswelle ueber 256 Bilder (rund 5 s): Grenzfrequenz $20 .. $60
        lda mus_fc
        bpl .f1
        eor #$ff                        // zweite Haelfte: abwaerts
.f1:    lsr                             // 0..63
        clc
        adc #$20
        sta $d416                       // Grenzfrequenz (obere 8 Bit)
        lda mus_tk
        bne .m1                         // nicht Zeilenanfang
        jsr mus_row_start               // Bild 0 der Zeile: Toene setzen, Gates aus
        jmp .mnext
.m1:    cmp #1
        bne .mnext
        ldx #2                          // Bild 1: ausstehende Gates setzen
.mg:    lda mus_pend,x
        beq +
        ldy mus_sidoff,x
        sta $d404,y
        lda #0
        sta mus_pend,x
+       dex
        bpl .mg
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
        sta mus_bar                     // Lied zu Ende: von vorn (Kopf +2 = 1) -- sonst still: hier immer von vorn
.mt9:   rts

// mus_row_start: fuer alle drei Stimmen das Zeilenbyte holen und auswerten.
mus_row_start:
        ldx #0                          // X = Stimme 0..2
.rv:    // Mustername aus der Reihenfolge: Kopf + 5 + Takt * 3 + Stimme
        lda mus_bar
        asl
        clc
        adc mus_bar                     // Takt * 3 (hoechstens 3 * 99 = 297: Lieder haben hoechstens 85 Takte)
        clc
        adc #5
        stx mus_x
        adc mus_x
        tay
        lda (mus_hdr),y                 // A = Mustername
        // Adresse = Kopf + Abstand + Name * 16 + Zeile
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
        adc mus_row                     // + Zeile (Uebertrag moeglich bei Namen >= 16 nicht: 16 * Name hat Low-Nibble 0)
        sta mus_ptr
        ldy #3
        clc
        lda mus_ptr
        adc (mus_hdr),y                 // + Abstand (Low)
        sta mus_ptr
        iny
        lda mus_ptr + 1
        adc (mus_hdr),y                 // + Abstand (High)
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
        stx mus_x                       // Stimme merken (X wird gleich fuer die Tabellen gebraucht)
        ldy mus_sidoff,x                // Y = Registerversatz der Stimme (0, 7, 14)
        cmp #96
        beq .rest
        bcs .drum
        tax                             // Note 1..95
        lda note_sid_lo,x
        sta $d400,y
        lda note_sid_hi,x
        sta $d401,y
        ldx mus_x
        lda mus_ad,x                    // Huellkurve der Stimme (nach einem Schlagzeugton muss sie neu gesetzt werden)
        sta $d405,y
        lda mus_sr,x
        sta $d406,y
        lda #0
        sta mus_slc                     // eine laufende Tonhoehenglide (Bassdrum) beenden
        lda mus_wave,x                  // Wellenform ohne Gate jetzt, mit Gate im naechsten Bild
        sta mus_pend,x
        and #$fe
        sta $d404,y
        jmp .rnext1
.rest:  lda #0
        sta $d404,y                     // Gate aus (alles still)
        ldx mus_x
        lda #0
        sta mus_pend,x
        jmp .rnext1
.drum:  and #$7f                        // Schlag n: eigene Wellenform, Tonhoehe, Huellkurve und ggf. fallende Tonhoehe
        tax
        lda #0
        sta $d400,y                     // Frequenz niedrig = 0
        lda mus_dhi,x
        sta $d401,y
        sta mus_fh                      // Startwert fuer die Glide (Bassdrum faellt von hoch nach tief)
        lda mus_dad,x
        sta $d405,y
        lda mus_dsr,x
        sta $d406,y
        lda mus_dsl,x
        sta mus_sl
        beq .nogl
        lda #5                          // 5 Bilder lang gleiten
        sta mus_slc
.nogl:  lda mus_dwv,x
        sta $d404,y                     // Wellenform ohne Gate, mit Gate im naechsten Bild
        ora #1
        ldx mus_x
        sta mus_pend,x
.rnext1: ldx mus_x
.rnext: inx
        cpx #3
        bne .rv
        rts

mus_sidoff: .byte 0, 7, 14                // Registerversatz der Stimmen
mus_wave:   .byte $41, $21, $11            // Puls (Melodie), Saege (Bass), Dreieck (Arpeggio), jeweils mit Gate
// Huellkurven (AD = Attack << 4 | Decay, SR = Sustain << 4 | Release): Melodie weich mit Nachklang, Bass kurz und knackig, Arpeggio perlend
mus_ad:     .byte $08, $06, $05
mus_sr:     .byte $a6, $64, $00
// Schlagzeug (Index 0..5): Bassdrum, Snare, Hi-Hat zu, Hi-Hat offen, Tom hoch, Tom tief
mus_dwv:    .byte $10, $80, $80, $80, $10, $10     // Wellenform ohne Gate: Dreieck bzw. Rauschen
mus_dhi:    .byte $14, $28, $70, $60, $18, $10     // Frequenz (hohes Byte)
mus_dad:    .byte $05, $04, $01, $03, $06, $06     // Attack 0, Decay: kurz (Hat) bis mittel (Tom)
mus_dsr:    .byte $00, $00, $00, $00, $00, $00     // Sustain 0: der Ton klingt von selbst aus, kein Gate-Aus noetig
mus_dsl:    .byte 3, 0, 0, 0, 1, 1                 // Glide: so viel faellt das hohe Frequenzbyte je Bild (Bassdrum schnell, Toms langsam)
mus_fh:     .byte 0
mus_sl:     .byte 0
mus_slc:    .byte 0
mus_fc:     .byte 0
mus_bar:    .byte 0
mus_row:    .byte 0
mus_tk:     .byte 0
mus_x:      .byte 0
mus_pend:   .byte 0, 0, 0

// Frequenztabelle: aus <c64/notes.inc> (ERZEUGT durch tools/gen_notes.py, mit -D NTSC=1 die NTSC-Tabelle).  Indiziert mit dem Notenwert (C0 = 1, Eintrag 0 = keine Note).
// Oberhalb von NOTE_TABLE_HIGHEST (PAL: A#7) ist der 16-Bit-Zähler des SID voll.
#include <c64/notes.inc>
