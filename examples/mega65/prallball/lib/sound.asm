// SPDX-License-Identifier: GPL-3.0-or-later
// Klang MEGA65 (PRALLBALL): vier SID-Chips.  Einbinden: .include "lib/sound.asm"
//   SID 1 ($D400)  Musik wie am C64 (Melodie, Bass, Schlagzeug)
//   SID 2 ($D420)  Chor: dieselbe Musik, etwas verstimmt (+0,6 %): klingt breiter und lebendiger (vermutlich liegen SID 1 und SID 2 auf dem MEGA65 auf verschiedenen Stereoseiten; in data/chips nicht belegt)
//   SID 3 ($D440)  Oktavverdopplung: Melodie und Arpeggio eine Oktave HOEHER (Dreieck), Bass eine Oktave TIEFER (Rechteck); kein Schlagzeug;
//                  nach mus_init3 spielt SID 3 stattdessen eine eigene zweite Liedspur (Gegenstimme, Akkordtoene)
//   SID 4 ($D460)  Klangeffekte, dreistimmig geschichtet: Hauptton, Oktave darueber, Rauschen bei Aufprall; die Musik wird nie unterbrochen
//   sfx_init      alle vier SIDs stumm schalten, Lautstaerken setzen, DMA-Audio-Samples erzeugen (dma_init)
//   sfx_play      A = Effekt (SFX_*); startet ihn auf SID 4; bei sfx_mute <> 0 ohne Wirkung
//   sfx_update    einmal je Bild aufrufen (Rasterinterrupt): Tonhoehe gleiten lassen, Effekt ausklingen lassen
//   mus_init / mus_tick / mus_stop   Titelmusik (siehe unten)
//   dma_play      (von sfx_play gerufen) digitales Sample auf Audio-DMA-Kanal 0; sfx_pan = Stereo-Lage 0 (links) .. 255 (rechts), Mitte 128, setzt der Aufrufer vor sfx_play
// Zeropage: 4 Byte fuer die Titelmusik (mus_ptr/mus_hdr, siehe MUS_ZP, Vorgabe $E8-$EB), 8 Byte SYN_ZP (Vorgabe $90-$97) und kurz $E0-$E3 bei dma_init;
//   RAM: sfx_t (Restdauer in Bildern), sfx_id
.const SFX_PADDLE = 0
.const SFX_WALL   = 1
.const SFX_BRICK  = 2
.const SFX_HARD   = 3
.const SFX_LOSE   = 4
.const SFX_EXTRA  = 5
.const SFX_START  = 6
.const SFX_LASER  = 7

sfx_init:
        ldx #$18                        // alle vier SIDs: Register $00-$18 loeschen
        lda #0
-       sta $d400,x
        sta $d420,x
        sta $d440,x
        sta $d460,x
        dex
        bpl -
        lda #$0f
        sta $d418                       // SID 1 (Musik): Lautstaerke 15
        lda #$0c
        sta $d438                       // SID 2 (Chor): 12
        lda #$08
        sta $d458                       // SID 3 (Oktaven): 8
        lda #$0f
        sta $d478                       // SID 4 (Effekte): 15
        lda #0
        sta sfx_t
        jmp dma_init                    // DMA-Audio: Samples erzeugen (siehe unten)

// sfx_play: A = Effekt.  Stimme 1 von SID 4: Hauptton (Frequenz sfx_freq, Wellenform sfx_wave); Stimme 2: eine Oktave darueber (Dreieck);
// Stimme 3: Rauschen bei Stein, hartem Stein und Verlust.  Das Steuerbyte (Gate) wird zuletzt geschrieben.
sfx_play:
        ldx sfx_mute                    // Demo-Modus: keine Effekte (die Musik laeuft unabhaengig weiter)
        bne .mute
        tax
        lda #0
        sta $d464                       // erst Gate aus: die Huellkurven starten von vorn
        sta $d46b
        sta $d472
        lda sfx_freq,x
        sta sfx_h1
        sta $d461                       // Stimme 1: Frequenz hoch
        asl a                           // Stimme 2: doppelte Frequenz (eine Oktave hoeher), begrenzt auf $FF
        bcc +
        lda #$ff
+       sta $d468
        lda #0
        sta $d460
        sta $d467
        sta $d465                       // Attack 0, Decay 0
        sta $d46c
        lda #$a8
        sta $d466                       // Sustain 10, Release 8
        lda #$88
        sta $d46d
        cpx #2                          // Rauschen (Stimme 3) nur bei Stein (2), hartem Stein (3) und Verlust (4)
        bcc .nonz
        cpx #5
        bcs .nonz
        lda #$30
        sta $d470
        lda #0
        sta $d46f
        lda #$94
        sta $d474
        lda #$81
        sta $d472                       // Stimme 3: Rauschen, Gate an
.nonz:  lda #$11
        sta $d46b                       // Stimme 2: Dreieck, Gate an
        lda sfx_wave,x
        sta $d464                       // Stimme 1: Wellenform + Gate
        lda sfx_len,x
        sta sfx_t
        stx sfx_id
        jsr dma_play                    // zusaetzlich das digitale Sample (DMA-Audio), falls der Effekt eines hat
.mute:  rts

// sfx_update: Tonhoehe je Bild um sfx_slide[sfx_id] aendern (Stimme 2 immer eine Oktave darueber); am Ende alle Gates aus (Ausklingen).
sfx_update:
        lda sfx_t
        beq .done
        dec sfx_t
        bne .glide
        lda #0
        sta $d464                       // Gate aus: Release
        sta $d46b
        sta $d472
        rts
.glide: ldx sfx_id
        lda sfx_slide,x
        clc
        adc sfx_h1
        sta sfx_h1
        sta $d461
        asl a
        bcc +
        lda #$ff
+       sta $d468
.done:  rts

//                 paddle wall brick hard lose  extra start
sfx_freq:  .byte   $18,   $10, $28,  $20, $20,  $30,  $14, $50
sfx_wave:  .byte   $41,   $41, $81,  $81, $21,  $11,  $11, $21    // Puls, Puls, Rauschen, Rauschen, Sägezahn, Dreieck, Dreieck
sfx_len:   .byte   4,     3,   5,    8,   30,   14,   10, 4
sfx_slide: .byte   0,     0,   $fe,  $ff, $ff,  2,    1, $fc
sfx_mute:  .byte 0                  // <> 0: sfx_play tut nichts (Demo-Modus)
sfx_t:     .byte 0
sfx_id:    .byte 0
sfx_h1:    .byte 0                  // aktuelle Frequenz (hoch) der Stimme 1

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
//          0 = halten, 1..95 = neue Note (Halbtonindex ab C0 = 1: Index n -> n - 1 in der Frequenztabelle), 96 = Pause (Gate aus),
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
        sta mus_h3 + 1                  // keine zweite Liedspur (siehe mus_init3)
        sta mus_set
        ldx #0                          // Klangfarben einrichten: Pulsbreite $0800, Huellkurven
.mi:    lda mus_ad,x
        ldy mus_sidoff,x
        sta $d405,y                     // Attack/Decay
        sta $d425,y                     // SID 2 (Chor): gleiche Huellkurven
        lda mus_sr,x
        sta $d406,y                     // Sustain/Release
        sta $d426,y
        lda mus_ad3,x
        sta $d445,y                     // SID 3 (Oktaven): eigene, weichere Huellkurven
        lda mus_sr3,x
        sta $d446,y
        lda #$00
        sta $d402,y
        sta $d422,y
        sta $d442,y
        lda #$08
        sta $d403,y                     // Pulsbreite $0800 = 50 %
        sta $d423,y
        sta $d443,y
        inx
        cpx #3
        bne .mi
        // Filter: Tiefpass auf dem Bass (Stimme 2), Resonanz 15.  $D417: Bits 4..7 = Resonanz, Bit 0..2 = Stimmen 1..3 durch den Filter.
        //   $D418: Bit 4 = Tiefpass, Bits 0..3 = Lautstaerke.  Die Grenzfrequenz ($D416) wird in mus_tick gefahren (Wah-Wah).
        lda #$f2
        sta $d417
        sta $d437                       // SID 2: gleicher Filter
        lda #$1f
        sta $d418
        lda #$1c
        sta $d438                       // SID 2: Tiefpass, Lautstaerke 12
        lda #0
        sta mus_fc
        rts

mus_stop:
        lda #0
        sta $d404
        sta $d40b
        sta $d412                       // Steuerregister aller drei Stimmen: Gate aus
        sta $d424                       // SID 2 und SID 3: ebenso
        sta $d42b
        sta $d432
        sta $d444
        sta $d44b
        sta $d452
        sta $d417                       // Filter aus
        sta $d437
        lda #$0f
        sta $d418                       // nur Lautstaerke 15, kein Tiefpass
        lda #$0c
        sta $d438                       // SID 2: Lautstaerke 12
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
        sta $d42f                       // SID 2: gleiche Gleitbewegung der Bassdrum
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
        sta $d423                       // SID 2: gleiche Pulsbreitenmodulation
        inc mus_fc                      // Filterfahrt: Dreieckswelle ueber 256 Bilder (rund 5 s): Grenzfrequenz $20 .. $60
        lda mus_fc
        bpl .f1
        eor #$ff                        // zweite Haelfte: abwaerts
.f1:    lsr                             // 0..63
        clc
        adc #$20
        sta $d416                       // Grenzfrequenz (obere 8 Bit)
        sta $d436
        lda mus_tk
        bne .m1                         // nicht Zeilenanfang
        jsr mus_rows                    // Bild 0 der Zeile: Toene setzen, Gates aus
        jmp .mnext
.m1:    cmp #1
        bne .mnext
        ldx #2                          // Bild 1: ausstehende Gates setzen
.mg:    ldy mus_sidoff,x
        lda mus_pend,x
        beq +
        sta $d404,y
        sta $d424,y                     // SID 2: gleiches Steuerbyte
        lda #0
        sta mus_pend,x
+       lda mus_pend3,x
        beq +
        sta $d444,y                     // SID 3: eigenes Steuerbyte (0 = nichts einschalten, z. B. bei Schlagzeug)
        lda #0
        sta mus_pend3,x
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

// mus_rows: Zeilenanfang fuer beide Liedspuren.  Spur 1 (Stimmen 1..3 auf SID 1 und, verstimmt, SID 2) liest den Liedkopf mus_hdr;
// Spur 2 (Stimmen 4..6 auf SID 3) liest den Kopf aus mus_init3 -- gleiches Format, gleiche Taktzahl.  Ohne mus_init3 gibt es keine Spur 2.
mus_rows:
        jsr mus_row_start
        lda mus_h3 + 1
        beq .nr
        lda mus_hdr
        pha
        lda mus_hdr + 1
        pha
        lda mus_h3
        sta mus_hdr
        lda mus_h3 + 1
        sta mus_hdr + 1
        lda #1
        sta mus_set
        jsr mus_row_start
        lda #0
        sta mus_set
        pla
        sta mus_hdr + 1
        pla
        sta mus_hdr
.nr:    rts

// mus_init3: X/Y = Liedkopf der zweiten Spur (Format wie oben, Muster nur Noten).  Danach spielen die Stimmen 4..6 auf SID 3 echte
// eigene Linien (Gegenstimme, Akkordtoene) statt der Oktavverdopplung.  Nach mus_init aufrufen.
mus_init3:
        stx mus_h3
        sty mus_h3 + 1
        ldx #0
.i3:    ldy mus_sidoff,x
        lda mus_ad4,x
        sta $d445,y
        lda mus_sr4,x
        sta $d446,y
        lda #0
        sta $d442,y
        lda #$08
        sta $d443,y                     // Pulsbreite 50 %
        inx
        cpx #3
        bne .i3
        rts

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
        ldy mus_set
        bne .set1                       // zweite Liedspur: SID 3
        stx mus_x                       // Stimme merken (X wird gleich fuer die Tabellen gebraucht)
        ldy mus_sidoff,x                // Y = Registerversatz der Stimme (0, 7, 14)
        cmp #96
        beq .rest
        bcs .drum
        sta mus_n                       // Note 1..95 merken (fuer die Spiegelung auf SID 2 und SID 3)
        tax
        lda mus_flo - 1,x
        sta $d400,y
        lda mus_fhi - 1,x
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
        jsr mus_mirror_note             // SID 2 (verstimmt) und SID 3 (Oktave) mit derselben Note
        jmp .rnext1
.rest:  lda #0
        sta $d404,y                     // Gate aus (alles still)
        sta $d424,y
        sta $d444,y
        ldx mus_x
        lda #0
        sta mus_pend,x
        sta mus_pend3,x
        jmp .rnext1
.drum:  and #$7f                        // Schlag n: eigene Wellenform, Tonhoehe, Huellkurve und ggf. fallende Tonhoehe
        tax
        lda #0
        sta $d400,y                     // Frequenz niedrig = 0
        sta $d420,y
        sta $d444,y                     // SID 3 spielt kein Schlagzeug: still
        lda mus_dhi,x
        sta $d401,y
        sta $d421,y
        sta mus_fh                      // Startwert fuer die Glide (Bassdrum faellt von hoch nach tief)
        lda mus_dad,x
        sta $d405,y
        sta $d425,y
        lda mus_dsr,x
        sta $d406,y
        sta $d426,y
        lda mus_dsl,x
        sta mus_sl
        beq .nogl
        lda #5                          // 5 Bilder lang gleiten
        sta mus_slc
.nogl:  lda mus_dwv,x
        sta $d404,y                     // Wellenform ohne Gate, mit Gate im naechsten Bild
        sta $d424,y
        ora #1
        ldx mus_x
        sta mus_pend,x
        lda #0
        sta mus_pend3,x
.set1:  stx mus_x                       // zweite Liedspur (Stimmen 4..6 auf SID 3): Note oder Pause, kein Schlagzeug, keine Spiegelung
        cmp #96
        beq .r1rest
        bcs .rnext1
        sta mus_n
        ldy mus_sidoff,x
        ldx mus_n
        lda mus_flo - 1,x
        sta $d440,y
        lda mus_fhi - 1,x
        sta $d441,y
        ldx mus_x
        lda mus_ad4,x
        sta $d445,y
        lda mus_sr4,x
        sta $d446,y
        lda mus_wave4,x
        sta mus_pend3,x                 // Gate im naechsten Bild
        and #$fe
        sta $d444,y
        jmp .rnext1
.r1rest: ldy mus_sidoff,x
        lda #0
        sta $d444,y                     // Gate aus
        sta mus_pend3,x
.rnext1: ldx mus_x
.rnext: inx
        cpx #3
        bne .rv
        rts

// mus_mirror_note: schreibt die gerade gesetzte Note auch auf SID 2 (Chor: verstimmte Tabelle mus_flo2/mus_fhi2, gleiche Huellkurve und
// Wellenform) und SID 3 (Oktave: Melodie und Arpeggio +12 Halbtoene, Bass -12, eigene Wellenform und Huellkurve).  Eingang: Y = Registerversatz
// der Stimme, mus_x = Stimme 0..2, mus_n = Note 1..95.  Das Gate kommt wie bei SID 1 erst im naechsten Bild (mus_pend, mus_pend3).
mus_mirror_note:
        ldx mus_n
        lda mus_flo2 - 1,x
        sta $d420,y
        lda mus_fhi2 - 1,x
        sta $d421,y
        ldx mus_x
        lda mus_wave,x
        and #$fe
        sta $d424,y                     // SID 2: Wellenform ohne Gate
        lda mus_h3 + 1
        beq .mm0
        rts                             // mit zweiter Liedspur gehoert SID 3 den eigenen Stimmen
.mm0:   lda mus_n                       // SID 3: Note um eine Oktave verschieben und auf 1..95 begrenzen
        clc
        adc mus_oct,x
        cmp #96
        bcc .mm1
        cmp #$f0                        // $F0..$FF: Ergebnis unter 0 (Bass bei tiefen Noten)
        bcs .mmlo
        lda #95
        bne .mm2
.mmlo:  lda #1
        bne .mm2
.mm1:   cmp #1
        bcs .mm2
        lda #1
.mm2:   tax
        lda mus_flo - 1,x
        sta $d440,y
        lda mus_fhi - 1,x
        sta $d441,y
        ldx mus_x
        lda mus_wave3,x
        sta mus_pend3,x                 // Steuerbyte mit Gate fuer das naechste Bild
        and #$fe
        sta $d444,y
        rts

mus_sidoff: .byte 0, 7, 14                // Registerversatz der Stimmen
mus_oct:    .byte $0c, $f4, $0c           // SID 3: Halbtonverschiebung je Stimme (+12, -12, +12)
mus_wave3:  .byte $11, $41, $11           // SID 3: Dreieck (Melodie +12), Rechteck (Bass -12), Dreieck (Arpeggio +12), mit Gate
mus_ad3:    .byte $06, $05, $05
mus_sr3:    .byte $84, $94, $50
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
mus_pend3:  .byte 0, 0, 0                  // Steuerbytes fuer SID 3 (0 = nichts einschalten)
mus_n:      .byte 0
mus_set:    .byte 0                        // 0 = Spur 1 wird gelesen, 1 = Spur 2
mus_h3:     .byte 0, 0                     // Liedkopf der zweiten Spur (High = 0: keine)
// Spur 2 (SID 3): Gegenstimme = Dreieck (weich), Akkordton 1 = Puls (breit, langer Nachklang), Akkordton 2 = Saege (kurz, perlend)
mus_wave4:  .byte $11, $41, $21
mus_ad4:    .byte $28, $4a, $09
mus_sr4:    .byte $a8, $a9, $54

// Frequenztabelle (PAL, 985248 Hz): Index n = Halbton ab C0 (16,35 Hz), 96 Eintraege (ab etwa B6 auf 65535 begrenzt: hoeher kann der SID nicht).  Fn = f * 2^24 / 985248.
mus_flo:
        .byte $16, $27, $39, $4b, $5f, $74, $8a, $a1, $ba, $d4, $f0, $0e
        .byte $2d, $4e, $71, $96, $be, $e7, $14, $42, $74, $a9, $e0, $1b
        .byte $5a, $9c, $e2, $2d, $7b, $cf, $27, $85, $e8, $51, $c1, $37
        .byte $b4, $38, $c4, $59, $f7, $9d, $4e, $0a, $d0, $a2, $81, $6d
        .byte $67, $70, $89, $b2, $ed, $3b, $9c, $13, $a0, $45, $02, $da
        .byte $ce, $e0, $11, $64, $da, $76, $39, $26, $40, $89, $04, $b4
        .byte $9c, $c0, $23, $c8, $b4, $eb, $72, $4c, $80, $12, $08, $68
        .byte $39, $80, $45, $90, $68, $d6, $e3, $99, $00, $24, $10, $ff
mus_fhi:
        .byte $01, $01, $01, $01, $01, $01, $01, $01, $01, $01, $01, $02
        .byte $02, $02, $02, $02, $02, $02, $03, $03, $03, $03, $03, $04
        .byte $04, $04, $04, $05, $05, $05, $06, $06, $06, $07, $07, $08
        .byte $08, $09, $09, $0a, $0a, $0b, $0c, $0d, $0d, $0e, $0f, $10
        .byte $11, $12, $13, $14, $15, $17, $18, $1a, $1b, $1d, $1f, $20
        .byte $22, $24, $27, $29, $2b, $2e, $31, $34, $37, $3a, $3e, $41
        .byte $45, $49, $4e, $52, $57, $5c, $62, $68, $6e, $75, $7c, $83
        .byte $8b, $93, $9c, $a5, $af, $b9, $c4, $d0, $dd, $ea, $f8, $ff
// Verstimmte Tabelle fuer SID 2 (Chor): Frequenz * 1,006, begrenzt auf 65535.
mus_flo2:
        .byte $18, $29, $3b, $4d, $61, $76, $8c, $a4, $bd, $d7, $f3, $11
        .byte $30, $52, $75, $9a, $c2, $eb, $19, $47, $79, $af, $e6, $21
        .byte $61, $a3, $ea, $35, $83, $d8, $30, $8f, $f3, $5c, $cd, $44
        .byte $c1, $46, $d3, $69, $08, $af, $61, $1e, $e5, $b8, $99, $86
        .byte $82, $8c, $a7, $d2, $0f, $5f, $c2, $3b, $ca, $72, $32, $0c
        .byte $03, $19, $4d, $a4, $1d, $bd, $85, $76, $95, $e3, $63, $19
        .byte $07, $31, $9b, $47, $3b, $7a, $09, $ec, $2a, $c6, $c7, $32
        .byte $0f, $63, $35, $8e, $75, $f3, $11, $d9, $53, $8c, $8d, $ff
mus_fhi2:
        .byte $01, $01, $01, $01, $01, $01, $01, $01, $01, $01, $01, $02
        .byte $02, $02, $02, $02, $02, $02, $03, $03, $03, $03, $03, $04
        .byte $04, $04, $04, $05, $05, $05, $06, $06, $06, $07, $07, $08
        .byte $08, $09, $09, $0a, $0b, $0b, $0c, $0d, $0d, $0e, $0f, $10
        .byte $11, $12, $13, $14, $16, $17, $18, $1a, $1b, $1d, $1f, $21
        .byte $23, $25, $27, $29, $2c, $2e, $31, $34, $37, $3a, $3e, $42
        .byte $46, $4a, $4e, $53, $58, $5d, $63, $68, $6f, $75, $7c, $84
        .byte $8c, $94, $9d, $a6, $b0, $ba, $c6, $d1, $de, $eb, $f9, $ff

// ---------------------------------------------------------------------------------------------------------------------
// DMA-Audio (MEGA65): digitale Klangeffekte aus Samples im Chip-RAM, zusaetzlich zu den SID-Effekten auf SID 4.
//   Die Samples werden beim Start SELBST ERZEUGT (kein Datenblock im Programm): 8 Bit mit Vorzeichen, 6 kHz, im Chip-RAM Bank 1: $10800-$117FF (Riss, Klang, Zap), $1C800 (Pling), $1F000 (Knall); $10000-$107FF und $11800-$11FFF gehoeren dem DOS des KERNALs
//   (die Zeiger sind 32 Bit, der VIC-IV-Audio-DMA liest die Bank 1 direkt).  Fuenf Klaenge:
//     0 Riss (Stein)       Rauschen mit abnehmender Lautstaerke, 4 Seiten zu 256 Byte
//     1 Klang (harter Stein) zwei Rechtecktoene (539 und 867 Hz, unharmonisch = metallisch), 6 Seiten
//     2 Knall (Verlust)    Rauschen durch einen Tiefpass, langer Abklang, 12 Seiten
//     3 Pling (Extra)      Dreieck mit steigender Tonhoehe (vier Toene), 6 Seiten
//     4 Zap (Laser)        fallender Rechteckton, 4 Seiten
//   dma_init   AUDEN einschalten ($D711 = $80), Samples erzeugen (mit $01 = $35, E/A sichtbar)
//   dma_play   X = Effekt (SFX_*): spielt das zugehoerige Sample auf DMA-Kanal 0 (links und rechts), Effekte ohne Sample: nichts
//   Der DMA-Audio braucht freie Prozessorzyklen: der Prozessor laeuft deshalb im 40-MHz-Modus (VFAST, $D054 Bit 6; siehe mega_init).
// Zeropage: SYN_ZP (8 Byte, Vorgabe $90) und der 32-Bit-Zeiger $E0-$E3 nur waehrend dma_init.
// ---------------------------------------------------------------------------------------------------------------------
.ifndef SYN_ZP
.const SYN_ZP = $90
.endif
.const syn_lfsr = SYN_ZP
.const syn_p1   = SYN_ZP + 1
.const syn_p2   = SYN_ZP + 2
.const syn_sh   = SYN_ZP + 3                // Abklang: Anzahl der Rechtsverschiebungen je Seite
.const syn_y    = SYN_ZP + 4                // Zustand des Tiefpasses
.const syn_pg   = SYN_ZP + 5                // Seitenzaehler
.const syn_st   = SYN_ZP + 6                // Schrittweite (Tonhoehe)
.const syn_t    = SYN_ZP + 7
.const SYN_RATE = 2486                      // Abspielfrequenz: 6000 Hz * 2^24 / 40,5 MHz
.const syn_ptr  = $e0                       // 32-Bit-Zeiger fuer die Ausgabe ($E0-$E3, Z = Index innerhalb der Seite)

dma_init:
        lda #$80
        sta $d711                           // AUDEN: Audio-DMA an
        lda #$a5
        sta syn_lfsr
        // ---- 0 Riss: Rauschen, Seite p: Lautstaerke / 2^p
        jsr syn_start0
        lda #4
        sta syn_pg
        lda #0
        sta syn_sh
.c1:    ldz #0
.c2:    jsr syn_rand
        jsr syn_asr
        jsr syn_put
        bne .c2
        jsr syn_nextpage
        inc syn_sh
        dec syn_pg
        bne .c1
        // ---- 1 Klang: zwei Rechtecke (Phasenschritte 23 und 37), Abklang alle zwei Seiten
        lda #$0c
        sta syn_ptr + 1                     // Seite $0C (nach Riss)
        lda #0
        sta syn_p1
        sta syn_p2
        sta syn_sh
        lda #6
        sta syn_pg
.k1:    ldz #0
.k2:    lda syn_p1
        clc
        adc #23
        sta syn_p1
        lda syn_p2
        clc
        adc #37
        sta syn_p2
        lda syn_p1                          // Rechteck 1: +50 oder -50
        bpl .k3
        lda #50
        bne .k4
.k3:    lda #$ce                            // -50
.k4:    sta syn_t
        lda syn_p2                          // Rechteck 2
        bpl .k5
        lda #50
        bne .k6
.k5:    lda #$ce
.k6:    clc
        adc syn_t                           // -100 .. +100
        jsr syn_asr
        jsr syn_put
        bne .k2
        jsr syn_nextpage
        lda syn_pg
        and #1
        beq .k7
        inc syn_sh
.k7:    dec syn_pg
        bne .k1
        // ---- 2 Knall: Tiefpass (y = y - y/8 + Rauschen/4), 12 Seiten, Abklang ab Seite 4
        lda #$f0
        sta syn_ptr + 1                     // Seite $F0 (oberes Ende der Bank 1)
        lda #0
        sta syn_y
        sta syn_sh
        lda #12
        sta syn_pg
.b1:    ldz #0
.b2:    jsr syn_rand
        ldx #2
.b3:    cmp #$80
        ror                                 // Rauschen / 4 (mit Vorzeichen)
        dex
        bne .b3
        sta syn_t
        lda syn_y
        ldx #3
.b4:    cmp #$80
        ror
        dex
        bne .b4                             // y / 8
        sta syn_st
        lda syn_y
        sec
        sbc syn_st
        clc
        adc syn_t
        sta syn_y
        jsr syn_asr
        jsr syn_put
        bne .b2
        jsr syn_nextpage
        lda syn_pg
        cmp #9
        bcs .b5
        inc syn_sh
.b5:    dec syn_pg
        bne .b1
        // ---- 3 Pling: Dreieck, Schritt 43 / 54 / 64 / 72, je Ton 1,5 Seiten (hier: Seitenpaare 2, 2, 1, 1)
        lda #$c8
        sta syn_ptr + 1                     // Seite $C8
        lda #0
        sta syn_p1
        sta syn_sh
        lda #6
        sta syn_pg
.p1:    lda syn_pg
        cmp #5
        bcs .pa
        cmp #3
        bcs .pb
        cmp #2
        bcs .pc
        lda #72
        bne .pd
.pa:    lda #43
        bne .pd
.pb:    lda #54
        bne .pd
.pc:    lda #64
.pd:    sta syn_st
        ldz #0
.p2:    lda syn_p1
        clc
        adc syn_st
        sta syn_p1
        bpl +
        eor #$ff                            // Dreieck: 255 - Phase ab $80
+       asl                                 // 0 .. 127 -> 0 .. 254
        sec
        sbc #$80                            // -128 .. +126
        jsr syn_asr
        jsr syn_put
        bne .p2
        jsr syn_nextpage
        lda syn_pg
        and #1
        beq .p4
        inc syn_sh
.p4:    dec syn_pg
        bne .p1
        // ---- 4 Zap: fallender Rechteckton, Schritt 100 abnehmend (alle 8 Bytes minus 1), 4 Seiten
        lda #$12
        sta syn_ptr + 1                     // Seite $12 (nach Klang)
        lda #0
        sta syn_p1
        sta syn_sh
        lda #100
        sta syn_st
        lda #4
        sta syn_pg
.z1:    ldz #0
.z2:    lda syn_p1
        clc
        adc syn_st
        sta syn_p1
        bpl .z3
        lda #60
        bne .z4
.z3:    lda #$c4                            // -60
.z4:    jsr syn_asr
        jsr syn_put
        tza
        and #7
        bne .z5
        lda syn_st
        cmp #12
        bcc .z5
        dec syn_st
.z5:    tza
        bne .z2
        jsr syn_nextpage
        inc syn_sh
        dec syn_pg
        bne .z1
        rts

// Hilfsroutinen der Synthese
syn_start0:                                 // Ausgabezeiger auf den Anfang des ersten Samples: Seite $08 der Bank 1 ($10800)
        lda #0
        sta syn_ptr
        lda #8
        sta syn_ptr + 1
        lda #0
        sta syn_ptr + 3
        lda #1
        sta syn_ptr + 2                     // Zeiger = $0001_0000
        rts
syn_put:                                    // A -> [syn_ptr],Z; Z + 1; Z = 0 am Seitenende (Zero-Flag gesetzt)
        sta [syn_ptr],z
        inz
        rts
syn_nextpage:
        inc syn_ptr + 1
        bne .sn
        inc syn_ptr + 2
.sn:    rts
syn_rand:                                   // 8-Bit-Galois-LFSR (Rueckkopplung $B8): A = neuer Wert
        lda syn_lfsr
        lsr
        bcc .sr
        eor #$b8
.sr:    sta syn_lfsr
        rts
syn_asr:                                    // A um syn_sh Stellen mit Vorzeichen nach rechts verschieben
        ldx syn_sh
        beq .sa2
.sa1:   cmp #$80
        ror
        dex
        bne .sa1
.sa2:   rts

// Sample-Tabellen: Anfang und Ende+1 als Seite (mittleres Adressbyte, Bank 1) je Klang: Riss 4, Klang 6, Knall 12, Pling 6, Zap 4 Seiten zu 256 Byte
//                Riss  Klang  Knall  Pling  Zap       (Anfaenge $10800, $10C00, $1F000, $1C800, $11200; Riss, Klang und Zap liegen lueckenlos ab $10800)
dma_pg:    .byte $08, $0c, $f0, $c8, $12
dma_pend:  .byte $0c, $12, $fc, $ce, $16
// Zuordnung Effekt -> Sample (0 = keins): paddle wall brick hard lose extra start laser
dma_map:   .byte 0, 0, 1, 2, 3, 4, 0, 5
sfx_pan:   .byte $80                        // Stereo-Lage des naechsten DMA-Klangs (der Aufrufer setzt sie vor sfx_play); Vorgabe Mitte

// dma_play: X = Effekt 0..7 (bleibt erhalten); Sample n = dma_map[X] - 1.  Kanal 0; Lautstaerken links/rechts aus sfx_pan (Stereo-Lage).
dma_play:
        lda dma_map,x
        beq .nd
        stx syn_t
        tax
        dex                                 // X = Sample 0..4
        lda #0
        sta $d720                           // Kanal anhalten
        sta $d721                           // Anfang: $01xx00 (Low-Byte 0)
        sta $d72a
        lda dma_pg,x
        sta $d722                           // mittleres Byte = Seite
        sta $d72b
        lda #1
        sta $d723                           // Bank 1
        sta $d72c
        lda #<SYN_RATE
        sta $d724
        lda #>SYN_RATE
        sta $d725
        lda #0
        sta $d726
        sta $d727                           // Ende: Low-Byte 0
        lda dma_pend,x
        sta $d728
        lda sfx_pan                         // Stereo-Lage 0 (links) .. 255 (rechts); Mitte = 128 = beide Seiten voll
        eor #$ff
        asl                                 // links = (255 - Lage) * 2, hoechstens 255
        bcc +
        lda #$ff
+       sta $d729                           // Lautstaerke links
        lda sfx_pan
        asl                                 // rechts = Lage * 2, hoechstens 255
        bcc +
        lda #$ff
+       sta $d71c                           // Lautstaerke rechts
        lda #$82                            // Kanal an: 8 Bit, mit Vorzeichen, einmal abspielen
        sta $d720
        lda #$80
        sta sfx_pan                         // danach wieder Mitte
        ldx syn_t
.nd:    rts
