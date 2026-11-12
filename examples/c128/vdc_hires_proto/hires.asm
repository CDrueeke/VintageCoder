// SPDX-License-Identifier: GPL-3.0-or-later
// C128: VDC-Hires-Prototyp (Machbarkeitsstudie fuer den PRALLBALL-Ausbau).
//
// Zweck: Zeigen und MESSEN, ob die VDC im Bitmapmodus 640 x 200 fuer ein Breakout-Spiel schnell genug ist.  Kein Spiel, kein Klang, keine Eingabe.
//   - VDC-Bitmap 640 x 200: 80 Byte je Pixelzeile ($0000..$3E7F), Farbattribute je 8 x 8 Pixel ab $4000 (2000 Byte).
//   - Hintergrundbild im VDC-RAM ab $5000 (nur mit 64-KB-VDC: Register 28 Bit 4); die Kugeln werden mit der Blockkopie der VDC (Register 24 Bit 7,
//     30, 32, 33) aus dem Hintergrund wiederhergestellt und danach neu gezeichnet.  Gezeichnet wird in 8 Feinlagen (Pixel-Versatz 0..7 im Byte).
//   - NB Kugeln (16 x 8 Pixel breite Zeichenflaeche, Kugel 8 x 8) springen im freien Feld unter der Steinwand.
//   - Der VIC-II bleibt dunkel und liefert nur die Rasterzeile als Zeitmass; der Prozessor laeuft mit 2 MHz.
// Ergebnis nach FRAMES Bildern im Hauptspeicher (mit dem Monitor lesbar):
//   res_done ($C200) = $FF fertig, res_vdc64 ($C201) = 1 wenn 64-KB-VDC erkannt, res_max ($C202/$C203) = groesste Zahl benutzter Rasterzeilen je Bild
//   (312 Zeilen je Bild bei PAL), res_sum ($C204..$C206) = Summe ueber alle Bilder, res_frames ($C207/$C208) = gezaehlte Bilder.
// Ohne 64-KB-VDC zeigt der Prototyp nur die Steine ohne Kugeln (der Hintergrund braeuchte den Platz) und meldet res_vdc64 = 0.

.const VA = $d600                       // VDC: Registerauswahl / Status
.const VD = $d601                       // VDC: Daten
.const NB = 8                           // Anzahl Kugeln
.const FRAMES = 200                     // Bilder, die gemessen werden
.const BG = $5000                       // Hintergrundbild im VDC-RAM (nur mit 64 KB)
.const ATTR = $4000                     // Attribute im VDC-RAM

.const res_done   = $c200
.const res_vdc64  = $c201
.const res_max    = $c202
.const res_sum    = $c204
.const res_frames = $c207
.const res_us     = $c210                // Mikrosekunden des letzten Bildes (32 Bit)
.const res_usmax  = $c214                // groesster Wert (32 Bit)

// Zeropage (Bank 0, ohne Betriebssystem frei)
.const zp   = $fb                       // Zeiger/Hilfen
.const t0   = $02
.const t1   = $03
.const t2   = $04
.const t3   = $05
.const bi   = $06                       // Kugelnummer
.const sh   = $07                       // Feinlage
.const byt  = $08                       // Byte-Spalte
.const lin  = $09                       // Pixelzeile
.const fcl  = $0a                       // Bildzaehler niedrig
.const fch  = $0b
.const r0l  = $0c                       // Rasterzeile am Anfang des Bildes
.const r0h  = $0d

.org $1c01
.basicstub start

start:  sei
        lda #$3e                        // alles RAM der Bank 0 plus E/A
        sta $ff00
        lda #$7f
        sta $dc0d                       // CIA-Interrupts aus
        sta $dd0d
        lda $dc0d
        lda $dd0d
        lda #<rti_only
        sta $fffa
        sta $fffe
        lda #>rti_only
        sta $fffb
        sta $ffff
        lda #$0b
        sta $d011                       // VIC-II: Bild aus (nur Rasterzaehler laeuft)
        lda #0
        sta $d015
        lda $d030
        ora #1
        sta $d030                       // 2 MHz
        jsr vdc_detect
        jsr vdc_setup
        jsr draw_wall
        lda res_vdc64
        beq nobg                        // ohne 64 KB kein Hintergrundbild und keine Kugeln
        jsr copy_bg
        jsr balls_init
        jsr run
nobg:   lda #$ff
        sta res_done
        jmp *
rti_only: rti

// ---- VDC-Zugriffe
wr:     stx VA                          // X = Register, A = Wert
.w:     bit VA
        bpl .w
        sta VD
        rts
rd:     stx VA                          // X = Register -> A
.r:     bit VA
        bpl .r
        lda VD
        rts
// vdc_at: A = Adresse hoch, Y = Adresse niedrig -> Register 18/19
vdc_at: ldx #18
        jsr wr
        inx
        tya
        jmp wr
put:    ldx #31                         // A = Byte an die aktuelle VDC-Adresse (zaehlt hoch)
        jmp wr

// 64-KB-VDC erkennen: Register 28 Bit 4 setzen, an $0000 und $8000 verschiedene Werte schreiben; bei nur 16 KB spiegelt $8000 auf $0000.
vdc_detect:
        ldx #28
        jsr rd
        ora #$10
        ldx #28
        jsr wr
        lda #0
        ldy #0
        jsr vdc_at
        lda #$11
        jsr put
        lda #$80
        ldy #0
        jsr vdc_at
        lda #$22
        jsr put
        lda #0
        ldy #0
        jsr vdc_at
        ldx #31
        jsr rd
        ldy #0
        cmp #$11
        bne +                           // $0000 ueberschrieben: Spiegelung, nur 16 KB
        iny
+       sty res_vdc64
        rts

// vdc_setup: Bitmapmodus 640 x 200 mit Attributen, Bild leeren, Attribute weiss auf schwarz
vdc_setup:
        lda #0
        ldx #12
        jsr wr                          // Bildanfang $0000
        ldx #13
        jsr wr
        lda #>ATTR
        ldx #20
        jsr wr                          // Attributanfang $4000
        lda #<ATTR
        ldx #21
        jsr wr
        lda #$00
        ldx #26
        jsr wr                          // Hintergrund schwarz (Tiefnibble)
        lda #$c7
        ldx #25
        jsr wr                          // Bit 7 Bitmap, Bit 6 Attribute, Verschiebung 7
        lda #$20
        ldx #10
        jsr wr                          // Cursor aus
        // Bild ($0000..$3E7F = 16000 Byte) mit 0 fuellen: 64 x 250 Byte per Blockfuellen
        lda #0
        tay
        jsr vdc_at
        lda #64
        sta t0
        lda #0
        jsr fill_run
        // Attribute ($4000..$47CF = 2000 Byte) weiss (15): 8 x 250
        lda #>ATTR
        ldy #<ATTR
        jsr vdc_at
        lda #8
        sta t0
        lda #$0f
        jmp fill_run
// fill_run: t0 mal 250 Byte mit dem Wert A fuellen (ab der aktuellen VDC-Adresse); Register 24 Bit 7 = 0 (Fuellen)
fill_run:
        sta t1
        lda #$20
        ldx #24
        jsr wr
.f1:    lda t1
        jsr put                         // 1 Byte schreiben, danach 249 Byte kopieren = 250 Byte
        lda #249
        ldx #30
        jsr wr
        dec t0
        bne .f1
        rts

// draw_wall: Rand und Steine.  Rand oben (Zeilen 0..1) und links/rechts (Byte 0 und 79) ganz, 6 Reihen mit 26 Steinen zu je 3 Byte (24 Pixel),
// jede Reihe in einer Zellenzeile (Zeilen (r+2)*8 .. +5), Farbe je Stein.
draw_wall:
        // Rand oben: 2 Zeilen x 80 Byte
        lda #0
        tay
        jsr vdc_at
        lda #$ff
        jsr put
        lda #79
        ldx #30
        jsr wr
        lda #0
        ldy #80
        jsr vdc_at
        lda #$ff
        jsr put
        lda #79
        ldx #30
        jsr wr
        // Seitenwand: Byte 0 und Byte 79 jeder Zeile 2..199 = $ff (je Zeile zwei Adressen, einzeln)
        lda #2
        sta lin
.sw:    lda lin
        jsr line_addr
        jsr vdc_at
        lda #$ff
        jsr put
        lda lin
        jsr line_addr
        sta t2
        tya
        clc
        adc #79
        tay
        lda t2
        adc #0
        jsr vdc_at
        lda #$ff
        jsr put
        inc lin
        lda lin
        cmp #200
        bne .sw
        // Steine
        lda #0
        sta bi                          // Reihe 0..5
.rw:    lda bi
        clc
        adc #2
        asl
        asl
        asl
        sta lin                         // erste Pixelzeile der Reihe
        lda #6
        sta t3                          // 6 gefuellte Zeilen
.ln:    lda lin
        jsr line_addr
        sta t2
        tya
        clc
        adc #1                          // Byte 1 (links der Rand Byte 0)
        tay
        lda t2
        adc #0
        jsr vdc_at
        lda #26
        sta t1                          // 26 Steine
.br:    lda #$ff
        jsr put
        lda #$ff
        jsr put
        lda #$fe                        // letzte Spalte laesst 1 Pixel Fuge
        jsr put
        dec t1
        bne .br
        inc lin
        dec t3
        bne .ln
        // Attribute dieser Reihe: Zellenzeile (bi+2): $4000 + (bi+2)*80 + 1 .. 78; Farbe je Stein = ((Stein + Reihe) mod 14) + 1
        lda bi
        clc
        adc #2
        jsr cell_addr                   // A = hoch, Y = niedrig
        sta t2
        tya
        clc
        adc #1
        tay
        lda t2
        adc #0
        jsr vdc_at
        lda #0
        sta t1                          // Steinnummer
.ac:    lda t1
        clc
        adc bi
.m14:   cmp #14
        bcc +
        sbc #14
        jmp .m14
+       clc
        adc #1                          // Farbe 1..14
        jsr put
        jsr put
        jsr put                         // drei Zellen je Stein
        inc t1
        lda t1
        cmp #26
        bne .ac
        inc bi
        lda bi
        cmp #6
        beq .rwe
        jmp .rw
.rwe:   rts

// line_addr: Pixelzeile A -> VDC-Adresse (A * 80) in A (hoch) / Y (niedrig).  Zeilen 0..199.
line_addr:
        sta zp + 2
        lda #0
        sta zp + 3                          // zp + 3:zp + 2 = Zeile
        // * 80 = * 64 + * 16
        lda zp + 2
        sta zp
        lda zp + 3
        sta zp + 1                      // zp = Zeile
        asl zp
        rol zp + 1
        asl zp
        rol zp + 1
        asl zp
        rol zp + 1
        asl zp
        rol zp + 1                      // * 16
        lda zp
        sta zp + 2
        lda zp + 1
        sta zp + 3                          // zp + 3:zp + 2 = * 16
        asl zp
        rol zp + 1
        asl zp
        rol zp + 1                      // * 64
        lda zp
        clc
        adc zp + 2
        tay
        lda zp + 1
        adc zp + 3
        rts
// cell_addr: Zellenzeile A (0..24) -> ATTR + A * 80 in A / Y
cell_addr:
        jsr line_addr                   // A * 80 (Zeilenzahl 0..24 reicht)
        clc
        adc #>ATTR
        rts

// copy_bg: Anzeige $0000..$3E7F nach BG kopieren (Blockkopie, 64 x 250 Byte)
copy_bg:
        lda #$a0
        ldx #24
        jsr wr                          // Blockkopie
        lda #>BG
        ldy #<BG
        jsr vdc_at                      // Ziel
        lda #0
        ldx #32
        jsr wr                          // Quelle $0000
        ldx #33
        jsr wr
        lda #64
        sta t0
.cb:    lda #250
        ldx #30
        jsr wr                          // 250 Byte kopieren (Quelle und Ziel zaehlen weiter)
        dec t0
        bne .cb
        lda #$20
        ldx #24
        jmp wr

// ---- Kugeln: Zustand (NB Stueck) und Bewegung
balls_init:
        ldx #NB - 1
.bi:    txa
        asl
        asl
        asl
        asl
        sta bxl,x                       // X-Anfang 16 px Abstand
        txa
        lsr
        lsr
        lsr
        lsr
        sta bxh,x
        lda #90
        sta byy,x
        txa
        and #3
        clc
        adc #1
        sta bdx,x                       // dx 1..4
        txa
        and #1
        beq +
        lda #$ff                        // dx negativ fuer jede zweite Kugel
        sta bdh,x
        lda bdx,x
        eor #$ff
        clc
        adc #1
        sta bdx,x
        jmp .pp1
+       lda #0
        sta bdh,x

.pp1:  txa
        and #1
        clc
        adc #1
        sta bdy,x                       // dy 1..2
        lda #1
        sta bsh,x                       // Vorzeichen dy: 1 = abwaerts
        dex
        bmi .bie
        jmp .bi
.bie:
        // Verschobene Kugelbilder: 8 Feinlagen x 8 Zeilen x 2 Byte (hoch, tief), aus dem 8-Pixel-Muster
        ldy #0
        sty sh
.sf:    ldx #0
.sr:    lda ballpat,x
        sta t0
        lda #0
        sta t1                          // 16-Bit-Wert t0:t1, nach rechts um sh
        ldy sh
        beq .sd
.sx:    lsr t0
        ror t1
        dey
        bne .sx
.sd:    lda sh
        asl
        asl
        asl
        asl                             // sh * 16
        sta t2
        txa
        asl                             // Zeile * 2
        clc
        adc t2
        tay
        lda t0
        sta ballimg,y
        lda t1
        sta ballimg + 1,y
        inx
        cpx #8
        bne .sr
        inc sh
        lda sh
        cmp #8
        bne .sf
        // erstes Bild zeichnen: Kugeln an ihre Anfangslage
        rts

// run: FRAMES Bilder: auf Zeile 251 warten, alle Kugeln bewegen (alte Lage aus dem Hintergrund herstellen, neue zeichnen), Rasterzeilen messen.
run:    lda #0
        sta fcl
        sta fch
        sta res_max
        sta res_max + 1
        sta res_sum
        sta res_sum + 1
        sta res_sum + 2
        ldx #7
        lda #0
.cl:    sta res_us,x
        dex
        bpl .cl
.fr:    lda $d012
        cmp #251
        bne .fr
.fr2:   lda $d012                       // aus Zeile 251 heraus warten (sonst waere ein Bild zweimal gezaehlt)
        cmp #251
        beq .fr2
        // Messbeginn: Zeile 252 +- 1; Referenz sauber ueber $d012/$d011-Bit 7
        lda $d012
        sta r0l
        lda $d011
        and #$80
        sta r0h
        lda #$ff
        sta $dc04                       // CIA1 Zeitgeber A: Startwert $FFFF, laeuft frei (Takt phi2)
        sta $dc05
        sta $dc06                       // Zeitgeber B: zaehlt Unterlaeufe von A
        sta $dc07
        lda #$41
        sta $dc0f                       // B: starten, zaehlt A-Unterlaeufe
        lda #$01
        sta $dc0e                       // A: starten
        lda #0
        sta bi
.bl:    ldx bi
        jsr ball_erase
        ldx bi
        jsr ball_move
        ldx bi
        jsr ball_draw
        inc bi
        lda bi
        cmp #NB
        bne .bl
        lda #0
        sta $dc0e                       // Zeitgeber anhalten
        sta $dc0f
        // Mikrosekunden = $FFFFFFFF - (B:A) ; als 32 Bit in res_us (little endian)
        sec
        lda #$ff
        sbc $dc04
        sta res_us
        lda #$ff
        sbc $dc05
        sta res_us + 1
        lda #$ff
        sbc $dc06
        sta res_us + 2
        lda #$ff
        sbc $dc07
        sta res_us + 3
        lda res_us + 3
        cmp res_usmax + 3
        bcc .um
        bne .us
        lda res_us + 2
        cmp res_usmax + 2
        bcc .um
        bne .us
        lda res_us + 1
        cmp res_usmax + 1
        bcc .um
        bne .us
        lda res_us
        cmp res_usmax
        bcc .um
.us:    ldx #3
.uc:    lda res_us,x
        sta res_usmax,x
        dex
        bpl .uc
.um:
        // gebrauchte Zeilen = (Ende - Anfang) mod 312
        lda $d012
        sta t0
        lda $d011
        and #$80
        beq +
        lda #1
        bne .pp2
+       lda #0

.pp2:  sta t1                          // t1:t0 = Ende
        lda r0h
        beq +
        lda #1
        bne .pp3
+       lda #0

.pp3:  sta t3                          // t3:r0l = Anfang
        sec
        lda t0
        sbc r0l
        sta t0
        lda t1
        sbc t3
        sta t1                          // t1:t0 = Ende - Anfang
        bpl +
        clc
        lda t0
        adc #<312
        sta t0
        lda t1
        adc #>312
        sta t1                          // Umlauf ueber das Bildende
+       // Summe und Maximum
        clc
        lda res_sum
        adc t0
        sta res_sum
        lda res_sum + 1
        adc t1
        sta res_sum + 1
        bcc +
        inc res_sum + 2
+       lda t1
        cmp res_max + 1
        bcc .nm
        bne .sm
        lda t0
        cmp res_max
        bcc .nm
.sm:    lda t0
        sta res_max
        lda t1
        sta res_max + 1
.nm:    inc fcl
        bne +
        inc fch
+       lda fcl
        sta res_frames
        lda fch
        sta res_frames + 1
        lda fch
        bne .done
        lda fcl
        cmp #FRAMES
        beq .done
        jmp .fr
.done:  rts

// ball_erase: Zellenflaeche der Kugel X aus dem Hintergrund wiederherstellen (8 Zeilen x 2 Byte per Blockkopie)
ball_erase:
        lda bxl,x
        sta t0
        lda bxh,x
        sta t1
        lsr t1
        ror t0
        lsr t1
        ror t0
        lsr t1
        ror t0                          // Byte-Spalte = X / 8
        lda t0
        sta byt
        lda byy,x
        sta lin
        lda #$a0
        ldx #24
        jsr wr                          // Blockkopie
        lda #8
        sta t3
.ee:    lda lin
        jsr line_addr                   // A = hoch, Y = niedrig (Zeilenanfang)
        sta t2
        tya
        clc
        adc byt
        tay
        lda t2
        adc #0
        sta t2                          // Ziel = Zeilenanfang + Byte
        pha
        tya
        pha
        lda t2
        jsr vdc_at                      // Ziel in R18/19
        pla
        tay
        pla                             // A = Ziel hoch, Y = Ziel niedrig; Quelle = Ziel + BG
        clc
        adc #>BG
        ldx #32
        jsr wr
        tya
        ldx #33
        jsr wr
        lda #2
        ldx #30
        jsr wr                          // 2 Byte kopieren
        inc lin
        dec t3
        bne .ee
        rts

// ball_move: Lage der Kugel X um (dx, dy) verschieben; an Waenden abprallen (X 8..615, Y 72..184)
ball_move:
        clc
        lda bxl,x
        adc bdx,x
        sta bxl,x
        lda bxh,x
        adc bdh,x
        sta bxh,x
        // Grenzen: x < 8 oder x > 615 (= $0267) -> Richtung umkehren
        lda bxh,x
        bmi .bn                         // negativ (unter 0)
        cmp #2
        bcc .bx                         // < 512
        bne .bn                         // >= 768: zu weit
        lda bxl,x
        cmp #$67
        bcc .bx
.bn:    sec                             // dx umkehren, X zurueck auf gueltigen Bereich
        lda #0
        sbc bdx,x
        sta bdx,x
        lda bdh,x
        eor #$ff
        sta bdh,x
        lda bdx,x
        bne +
        inc bdh,x                       // (-0 -> Hochbyte neu)
+       clc
        lda bxl,x
        adc bdx,x
        sta bxl,x
        lda bxh,x
        adc bdh,x
        sta bxh,x
        clc
        lda bxl,x
        adc bdx,x
        sta bxl,x
        lda bxh,x
        adc bdh,x
        sta bxh,x
.bx:    lda bsh,x
        beq .up
        clc
        lda byy,x
        adc bdy,x
        sta byy,x
        cmp #184
        bcc .yd
        lda #0
        sta bsh,x
        rts
.up:    sec
        lda byy,x
        sbc bdy,x
        sta byy,x
        cmp #72
        bcs .yd
        lda #1
        sta bsh,x
.yd:    rts

// ball_draw: Kugel X zeichnen: Feinlage = X mod 8, 8 Zeilen x 2 Byte aus ballimg
ball_draw:
        lda bxl,x
        and #7
        asl
        asl
        asl
        asl
        sta sh                          // sh * 16 = Offset im Bild
        lda bxl,x
        sta t0
        lda bxh,x
        sta t1
        lsr t1
        ror t0
        lsr t1
        ror t0
        lsr t1
        ror t0
        lda t0
        sta byt
        lda byy,x
        sta lin
        // Blockkopie aus, damit R31 normal schreibt
        lda #$20
        ldx #24
        jsr wr
        lda #0
        sta t3                          // Zeilenzaehler
.dd:    lda lin
        jsr line_addr
        sta t2
        tya
        clc
        adc byt
        tay
        lda t2
        adc #0
        jsr vdc_at
        lda sh
        clc
        adc t3
        adc t3                          // sh*16 + Zeile * 2
        tax
        lda ballimg,x
        jsr put
        lda ballimg + 1,x
        jsr put
        inc lin
        inc t3
        lda t3
        cmp #8
        bne .dd
        rts

ballpat: .byte $3c, $7e, $ff, $ff, $ff, $ff, $7e, $3c
bxl:    .fill NB, 0
bxh:    .fill NB, 0
byy:    .fill NB, 0
bdx:    .fill NB, 0
bdh:    .fill NB, 0
bdy:    .fill NB, 0
bsh:    .fill NB, 0
ballimg: .fill 128, 0                   // 8 Feinlagen x 8 Zeilen x 2 Byte
