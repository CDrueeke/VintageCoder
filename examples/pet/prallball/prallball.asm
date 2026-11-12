// SPDX-License-Identifier: GPL-3.0-or-later
// PRALLBALL fuer den Commodore PET (4000er, 40 Spalten, BASIC 4) - Ball-und-Schlaeger-Spiel.
//
// Der PET hat nur Text: keine Farbe, keine Sprites, keinen Klangchip.  Das Bild besteht aus 40 x 25 Zeichen; gezeichnet wird mit den Viertelblock-
// Zeichen des PET-Zeichensatzes (2 x 2 Teilzellen je Zeichen = 80 x 50 Teilzellen).  Der Ball ist eine Teilzelle, der Schlaeger eine Teilzellenreihe.
// Klang: nur ein Piepton ueber das Schieberegister der VIA (sound.asm).  Eingabe: nur Tastatur (input.asm).
//
// Aufbau des Bildes (Zeichenzellen, Spalte 0..39, Zeile 0..24):
//   Zeile 0        Anzeige: Punkte, Level, Leben, Meldungen
//   Zeile 1        obere Wand; Spalte 0 und 39 = Seitenwaende
//   Zeilen 3..14   12 Steinreihen zu 18 Steinen je 2 Zeichen breit (Spalte 2..37)
//   Zeile 23       Schlaeger (untere Haelfte der Zeichen)
// Zwei Schichten: Der Bildschirm ($8000) enthaelt Steine, Waende und Text; ein "Overlay" ($6C00, je Zelle eine Maske der vier Viertelblock-Bits)
//   enthaelt Ball und Schlaeger.  Ein Overlay-Bit gehoert immer zu einer sonst leeren Zelle; die Zelle zeigt dann das passende Viertelblock-Zeichen.
// Ablauf: Titel -> Spiel (Level 1..22) -> Spielende -> Bestenliste (Namenseingabe) -> Titel.  Die Spiellogik laeuft einmal je Bild (Interrupt zaehlt).
// Speicher: Programm ab $0401, Levelkarte $6B00 (216 Byte), Overlay $6C00 (1000 Byte); gebraucht werden mindestens 32 KB RAM (4032, 8032 mit 40 Spalten).
//
// Bedienung: Z oder , = links, X oder . = rechts, Leertaste = Ball abschiessen / Titel verlassen, P = Pause, RUN/STOP = Abbruch.
// Testfassungen (mit -D): AUTOPLAY=1 (Schlaeger folgt dem Ball), STARTLEVEL=n (n = 0..21), STARTLIVES=n (Leben zu Beginn, Vorgabe 3),
//   TESTLOSE=1 (zusammen mit AUTOPLAY: der Schlaeger bleibt links stehen, der Ball geht verloren).

.target pet
.cpu nmos6502 { illegals = deny }

.const SCR      = $8000                 // Bildschirmspeicher (40 Spalten)
.const OV       = $6c00                 // Overlay: 1000 Byte, Maske je Zelle
.const LV       = $6b00                 // Levelkarte: 12 x 18 Steintypen
.const OVD      = $14                   // Hochbyte-Unterschied Bildschirm - Overlay ($80 - $6C)

.const PADROW   = 23                    // Zeichenzeile des Schlaegers
.const PADY     = 47                    // Teilzeile des Schlaegers (untere Haelfte der Zeile 23)
.const LOSTY    = 49                    // ab dieser Teilzeile ist der Ball verloren

.const ST_TITLE = 0
.const ST_PLAY  = 1

// ---- Nullseite ($02..$7F gehoert dem Spiel; $80 und darueber benutzt der Editor-Interrupt)
.const frame    = $04                   // Bildzaehler (Interrupt)
.const spdlvl   = $05                   // Ballgeschwindigkeit 0..7
.const lives    = $07
.const level    = $08                   // 0..21
.const bricks   = $09                   // noch zu zerstoerende Steine
.const sc0      = $0a                   // Punkte BCD: sc0 = Einer/Zehner, sc1 = Hunderter/Tausender, sc2 = Zehntausender/Hunderttausender
.const sc1      = $0b
.const sc2      = $0c
.const px       = $0e                   // Schlaeger: linke Teilspalte
.const pw       = $0f                   // Schlaegerbreite in Teilzellen
.const bxl      = $10                   // Ball: Lage in 8.8 Festkomma (Teilzellen)
.const bxh      = $11
.const byl      = $12
.const byh      = $13
.const vxl      = $14                   // Ballgeschwindigkeit, vorzeichenbehaftet 8.8
.const vxh      = $15
.const vyl      = $16
.const vyh      = $17
.const ballon   = $18                   // 1 = Ball fliegt, 0 = klebt am Schlaeger
.const lbx      = $19                   // zuletzt gezeichnete Ballzelle (Teilzellen)
.const lby      = $1a
.const lbon     = $1b                   // 1 = Ball ist gezeichnet
.const hits     = $1c                   // Steintreffer (fuer die Geschwindigkeit)
.const t0       = $1d                   // Hilfsspeicher
.const t1       = $1e
.const t2       = $1f
.const t3       = $20
.const t4       = $21
.const t5       = $22
.const t6       = $23
.const t7       = $24
.const lostf    = $25                   // 1 = Ball in diesem Bild verloren
.const cp       = $26                   // Zeiger in das Overlay (2 Byte)
.const sp       = $28                   // Zeiger in den Bildschirm (2 Byte)
.const lp       = $2a                   // Zeiger auf die Leveldaten
.const tp       = $2c                   // Zeiger auf Text
.const inp      = $2e                   // Eingabemaske dieses Bildes
.const dirty    = $2f                   // 1 = Anzeige neu schreiben
.const msgt     = $30                   // Restzeit einer Meldung in der Anzeige
.const npos     = $31                   // Namenseingabe: Cursorstelle
.const tick     = $32                   // Hilfszaehler (Titel, Warten)
.const lvdone   = $33                   // 1 = Level geschafft
// $34..$37 input.asm (IN_ZP), $38..$3e hiscore.asm (HS_ZP), $3f sound.asm (SND_ZP), $02/$03 rand.asm (RAND_ZP)

// ---------------------------------------------------------------------------------------------------------------------
// Programmstart
// ---------------------------------------------------------------------------------------------------------------------
.org $0401                              // BASIC-Programmanfang des PET
.basicstub start                        // 10 SYS <start>

start:  sei
        lda #$0c
        sta $e84c                       // Zeichensatz Grossbuchstaben + Grafik (VIA-Steuerleitung CA2 niedrig; auf dem 8032 noetig)
        lda #1
        sta $a7                         // Editor: Cursorblinken aus (sonst schreibt der Interrupt Umkehrzeichen in den Bildschirm)
        lda $90                         // bisheriger Interrupt-Vektor (zeigt auf die Editor-Routine $E455) merken
        sta oldirq
        lda $91
        sta oldirq + 1
        lda #<irq                       // eigene Routine einhaengen; sie ruft danach die alte auf
        sta $90
        lda #>irq
        sta $91
        jsr hs_init                     // Startgeraet fuer die Bestenliste merken
        jsr input_init
        jsr sfx_off
        lda $e845
        jsr rand_seed
        cli
        jsr hs_load                     // Bestenliste von der Diskette (bei Fehlern gilt die Standardliste)
        jmp title

// irq: Interrupt je Bild (50/60 Hz, vom Bildruecklauf des Monitors).  Die Register A, X, Y hat der Einsprung ($E442) schon gesichert.
//   Aufgaben: Bildzaehler, Tastaturpuffer leeren (siehe input.asm), Klang weiterzaehlen; dann in die alte Routine (Editor: Tastaturabfrage, Uhr).
irq:    inc frame
        lda in_kbclr
        beq .i1
        lda #0
        sta KEYBUFN                     // Tastaturpuffer leeren, damit der Editor die gehaltene Taste in $97 eintraegt
.i1:    jsr sfx_tick
        jmp (oldirq)
oldirq: .word 0

wait_frame:
        lda frame
.wf:    cmp frame
        beq .wf
        rts

// ---------------------------------------------------------------------------------------------------------------------
// Zellen und Overlay
// ---------------------------------------------------------------------------------------------------------------------
// setcell: Spalte X (0..39), Zeile Y (0..24) -> cp (Overlay-Zelle), sp (Bildschirmzelle).  Y bleibt, A und X werden veraendert.
setcell:
        txa
        clc
        adc ovrow_lo,y
        sta cp
        sta sp
        lda ovrow_hi,y
        adc #0
        sta cp + 1
        clc
        adc #OVD
        sta sp + 1
        rts

// ov_or / ov_clr: Bits A im Overlay der Zelle (cp/sp) setzen bzw. loeschen und das Zeichen der Zelle neu bestimmen.  Veraendert A, X, Y.
ov_or:  ldy #0
        ora (cp),y
        jmp ov_put
ov_clr: eor #$ff
        ldy #0
        and (cp),y
ov_put: sta (cp),y
        tax
        lda qtab,x                      // Maske 0 = Leerzeichen
        sta (sp),y
        rts

// cell_static: Zelle Spalte X, Zeile Y: C = 1, wenn dort etwas Festes steht (Stein, Wand); Overlay-Zellen und Leerzeichen gelten als frei.
cell_static:
        jsr setcell
        ldy #0
        lda (cp),y
        bne .free                       // Ball oder Schlaeger in der Zelle
        lda (sp),y
        cmp #$20
        beq .free
        sec
        rts
.free:  clc
        rts

clear_screen:
        lda #$20
        ldx #0
.c1:    sta SCR,x
        sta SCR + $100,x
        sta SCR + $200,x
        sta SCR + $300,x
        inx
        bne .c1
        lda #0
        tax
.c2:    sta OV,x
        sta OV + $100,x
        sta OV + $200,x
        sta OV + $300,x
        inx
        bne .c2
        sta lbon                        // kein Ball gezeichnet
        rts

// puts: Text ab (tp), mit 0 abgeschlossen, an Spalte X, Zeile Y (Bildschirmcodes).  Veraendert A, Y.
puts:   jsr setcell
        ldy #0
.p1:    lda (tp),y
        beq .p9
        sta (sp),y
        iny
        bne .p1
.p9:    rts

// ---------------------------------------------------------------------------------------------------------------------
// Anzeige (Zeile 0)
// ---------------------------------------------------------------------------------------------------------------------
draw_hud_static:
        lda #<hud_text
        sta tp
        lda #>hud_text
        sta tp + 1
        ldx #0
        ldy #0
        jsr puts
        lda #1
        sta dirty
        rts

// update_hud: Zahlen der Anzeige schreiben: Punkte (6 Ziffern ab Spalte 7), Level (2 Ziffern ab Spalte 21), Leben (Spalte 31).
update_hud:
        lda #0
        sta dirty
        sta t7
        ldx #7
        ldy #0
        jsr setcell
        lda sc2
        jsr hud_bcd
        lda sc1
        jsr hud_bcd
        lda sc0
        jsr hud_bcd
        ldx level                       // Level + 1 als zwei Ziffern
        inx
.l1:    cpx #10
        bcc .l2
        txa
        sec
        sbc #10
        tax
        lda t7
        clc
        adc #1
        sta t7
        jmp .l1
.l2:    stx t6
        ldx #21
        ldy #0
        jsr setcell
        ldy #0
        lda t7
        ora #$30
        sta (sp),y
        iny
        lda t6
        ora #$30
        sta (sp),y
        ldx #31
        ldy #0
        jsr setcell
        lda lives
        ora #$30
        ldy #0
        sta (sp),y
        rts
// hud_bcd: zwei BCD-Ziffern A ab (sp) schreiben, sp wandert um 2 weiter
hud_bcd:
        pha
        lsr
        lsr
        lsr
        lsr
        ora #$30
        ldy #0
        sta (sp),y
        pla
        and #$0f
        ora #$30
        iny
        sta (sp),y
        lda sp
        clc
        adc #2
        sta sp
        bcc .h1
        inc sp + 1
.h1:    rts
// level-Ziffern: t7 wird vor update_hud nicht geloescht: in update_hud ab "ldx level" mit t7 = 0 starten
hud_text:  .screen "punkte 000000  level 01  leben 3"
           .byte 0

// ---------------------------------------------------------------------------------------------------------------------
// Punkte (BCD)
// ---------------------------------------------------------------------------------------------------------------------
// add_score: A = BCD-Wert (zwei Ziffern) zu den Punkten addieren.  Verändert A.
add_score:
        sed
        clc
        adc sc0
        sta sc0
        lda sc1
        adc #0
        sta sc1
        lda sc2
        adc #0
        sta sc2
        cld
        lda #1
        sta dirty
        rts

// ---------------------------------------------------------------------------------------------------------------------
// Steine
// ---------------------------------------------------------------------------------------------------------------------
// draw_brick: Stein vom Typ A an Steinspalte t2 (0..17), Steinzeile t3 (0..11) zeichnen (Typ 0 loescht).  Veraendert A, X, Y, t4.
draw_brick:
        sta t4
        lda t2
        asl
        clc
        adc #2
        tax                             // Zeichenspalte
        lda t3
        clc
        adc #3
        tay                             // Zeichenzeile
        jsr setcell
        ldx t4
        ldy #0
        lda brl,x
        sta (sp),y
        iny
        lda brr,x
        sta (sp),y
        rts

// load_level: Level `level` in die Levelkarte kopieren, Steine zaehlen und zeichnen.
load_level:
        ldx level
        lda level_tab_lo,x
        sta lp
        lda level_tab_hi,x
        sta lp + 1
        ldy #0
        sty bricks
.ll1:   lda (lp),y
        sta LV,y
        beq .ll2                        // leer
        cmp #11
        beq .ll2                        // unzerstoerbar: zaehlt nicht
        inc bricks
.ll2:   iny
        cpy #216
        bne .ll1
        // zeichnen
        lda #0
        sta t3                          // Zeile
.ll3:   lda #0
        sta t2                          // Spalte
.ll4:   ldx t3
        lda rowlv,x
        clc
        adc t2
        tax
        lda LV,x
        beq .ll5
        jsr draw_brick
.ll5:   inc t2
        lda t2
        cmp #18
        bne .ll4
        inc t3
        lda t3
        cmp #12
        bne .ll3
        rts

// hit_cell: der Ball ist gegen die Zelle Spalte X, Zeile Y gestossen: Stein beschaedigen/zerstoeren, Ton.  Veraendert A, X, Y, t2..t6.
hit_cell:
        cpy #3
        bcc .wall
        cpy #15
        bcs .wall
        cpx #2
        bcc .wall
        cpx #38
        bcs .wall
        tya
        sec
        sbc #3
        sta t3                          // Steinzeile
        tay
        lda rowlv,y
        sta t5
        txa
        sec
        sbc #2
        lsr
        sta t2                          // Steinspalte
        clc
        adc t5
        sta t1                          // Index in der Levelkarte
        tax
        lda LV,x
        beq .wall                       // kein Stein mehr (z. B. gerade zerstoert)
        cmp #11
        beq .hard
        cmp #9
        beq .silver
        cmp #10
        beq .gold
        // zerstoerbar: 1..8, 12 (explosiv), 13 (Ueberraschung)
        pha
        jsr kill_brick
        pla
        cmp #12
        beq .boom
        cmp #13
        beq .bonus
        jsr sfx_brick
        rts
.hard:  jsr sfx_hard
        rts
.silver: lda #1                         // Silber: wird zum einfachen Stein (der naechste Treffer zerstoert ihn)
        jmp .newtype
.gold:  lda #9                          // Gold: wird zu Silber
.newtype: ldx t1
        sta LV,x
        jsr draw_brick
        jsr sfx_hard
        rts
.wall:  jsr sfx_wall
        rts
.boom:  jsr explode
        jsr sfx_level
        rts
.bonus: jsr bonus
        rts

// kill_brick: Stein (Index t1, Spalte t2, Zeile t3) entfernen, Punkte, Zaehler, Geschwindigkeit.  Veraendert A, X, Y, t4.
kill_brick:
        ldx t1
        lda LV,x
        tax
        lda tscore,x
        jsr add_score
        ldx t1
        lda #0
        sta LV,x
        jsr draw_brick                  // Typ 0 = loeschen
        dec bricks
        bne .k1
        lda #1
        sta lvdone                      // letzter Stein: Level geschafft
.k1:    inc hits
        lda hits
        and #7
        bne .k2
        lda spdlvl                      // alle 8 Treffer eine Stufe schneller (bis 7)
        cmp #7
        bcs .k2
        inc spdlvl
.k2:    rts

// explode: explosiver Stein (Spalte t2, Zeile t3 sind die Mitte): die 8 Nachbarn zerstoeren, ausser unzerstoerbaren.
explode:
        lda t2
        sta t6
        lda t3
        sta t7                          // Mitte merken
        ldx #7
.e1:    stx t5
        lda t6
        clc
        adc dcol,x
        sta t2                          // Nachbarspalte (0..17, sonst ausserhalb)
        cmp #18
        bcs .e9                         // negativ (als 255 gelesen) oder zu gross
        lda t7
        clc
        adc drow,x
        sta t3
        cmp #12
        bcs .e9
        tax
        lda rowlv,x
        clc
        adc t2
        sta t1
        tax
        lda LV,x
        beq .e9
        cmp #11
        beq .e9
        jsr kill_brick
.e9:    ldx t5
        dex
        bpl .e1
        rts
dcol:   .byte $ff, 0, 1, $ff, 1, $ff, 0, 1
drow:   .byte $ff, $ff, $ff, 0, 0, 1, 1, 1

// bonus: Ueberraschungsstein: Zufall 0..2: breiter Schlaeger / ein Leben mehr / langsamer.
bonus:
        jsr rand_byte
        and #3
        cmp #3
        bne .b0
        lda #0
.b0:    beq .wide
        cmp #1
        beq .life
        // langsamer: zwei Stufen
        lda spdlvl
        sec
        sbc #2
        bcs .s1
        lda #0
.s1:    sta spdlvl
        lda #<msg_slow
        ldx #>msg_slow
        jmp bonus_msg
.life:  lda lives
        cmp #9
        bcs .l9
        inc lives
        lda #1
        sta dirty
.l9:    lda #<msg_life
        ldx #>msg_life
        jmp bonus_msg
.wide:  jsr erase_paddle
        lda #18
        sta pw
        lda px
        cmp #60                         // rechts nicht aus dem Spielfeld ragen lassen (px + 18 <= 78)
        bcc .w1
        lda #60
        sta px
.w1:    jsr draw_paddle
        lda #<msg_wide
        ldx #>msg_wide
bonus_msg:
        sta tp
        stx tp + 1
        lda #120
        sta msgt
        ldx #33
        ldy #0
        jsr puts
        lda #$50                        // Ueberraschung: 50 Punkte
        jmp add_score
msg_wide:  .screen "breit  "
           .byte 0
msg_life:  .screen "leben+ "
           .byte 0
msg_slow:  .screen "langsam"
           .byte 0
msg_none:  .screen "       "
           .byte 0

// Anzeige der Brickfarben ersetzt durch Muster: Typ 0..13.  brl/brr = linkes/rechtes Zeichen des Steins.
//   1,3,5,7: Vollblock; 2,4,6,8: Raster; 9 Silber: Block mit Kopflinie frei; 10 Gold: umgekehrtes Raster; 11 unzerstoerbar: schmaler Block; 12 explosiv: XX; 13 ?: ??
brl:    .byte $20, $a0, $66, $a0, $66, $a0, $66, $a0, $66, $e3, $e6, $e1, $18, $3f
brr:    .byte $20, $ec, $66, $ec, $66, $ec, $66, $ec, $66, $e3, $e6, $61, $18, $3f
// tscore: Punkte je zerstoertem Stein (BCD, zwei Ziffern): a..h = 10..80, Silber/Gold zaehlen erst beim endgueltigen Zerstoeren (Typ 1 -> 10)
tscore: .byte 0, $10, $20, $30, $40, $50, $60, $70, $80, $20, $30, 0, $50, $50

// ---------------------------------------------------------------------------------------------------------------------
// Schlaeger
// ---------------------------------------------------------------------------------------------------------------------
// draw_paddle / erase_paddle: Teilzellen px .. px + pw - 1 der Teilzeile 47 (untere Haelfte der Zeile 23).
draw_paddle:
        ldx #0
        jmp paddle_do
erase_paddle:
        ldx #1
paddle_do:
        stx t6                          // 0 = zeichnen, 1 = loeschen
        lda px
        sta t4                          // aktuelle Teilspalte
        lda pw
        sta t5                          // Anzahl
.pd1:   lda t4
        lsr
        tax                             // Zeichenspalte
        ldy #PADROW
        jsr setcell
        lda t4
        and #1
        beq .pd2
        lda #8                          // rechte Teilspalte: unten rechts
        bne .pd3
.pd2:   lda #4                          // linke Teilspalte: unten links
.pd3:   ldx t6
        bne .pd4
        jsr ov_or
        jmp .pd5
.pd4:   jsr ov_clr
.pd5:   inc t4
        dec t5
        bne .pd1
        rts

// move_paddle: A = neue linke Teilspalte (wird auf 2 .. 78 - pw begrenzt); zeichnet neu, wenn sich etwas aendert; bewegt den Ball mit, solange er klebt.
move_paddle:
        cmp #2
        bcs .m1
        lda #2
.m1:    sta t0
        lda #78
        sec
        sbc pw
        cmp t0
        bcs .m2
        sta t0                          // zu weit rechts
.m2:    lda t0
        cmp px
        beq .m9
        jsr erase_paddle
        lda t0
        sta px
        jsr draw_paddle
        lda ballon
        bne .m9
        jmp place_ball
.m9:    rts

// ---------------------------------------------------------------------------------------------------------------------
// Ball
// ---------------------------------------------------------------------------------------------------------------------
// ball_cell: Zelle des Balls (Spalte X, Zeile Y) und Bitmaske A fuer die Teilzelle (X, Y der Teilzellen aus bxh/byh).
ball_cell:
        lda bxh
        and #1
        sta t7
        lda byh
        and #1
        asl
        ora t7
        tax
        lda b2m,x
        pha
        lda bxh
        lsr
        tax
        lda byh
        lsr
        tay
        jsr setcell
        pla
        rts
b2m:    .byte 1, 2, 4, 8

draw_ball:
        jsr ball_cell
        jsr ov_or
        lda bxh
        sta lbx
        lda byh
        sta lby
        lda #1
        sta lbon
        rts
erase_ball:
        lda lbon
        beq .e9
        lda lbx
        and #1
        sta t7
        lda lby
        and #1
        asl
        ora t7
        tax
        lda b2m,x
        pha
        lda lbx
        lsr
        tax
        lda lby
        lsr
        tay
        jsr setcell
        pla
        jsr ov_clr
        lda #0
        sta lbon
.e9:    rts

// place_ball: Ball auf die Mitte des Schlaegers setzen (klebt), neu zeichnen.
place_ball:
        jsr erase_ball
        lda pw
        lsr
        clc
        adc px
        sta bxh
        lda #0
        sta bxl
        sta byl
        lda #PADY - 1
        sta byh
        jmp draw_ball

// set_vel: Geschwindigkeit nach Abprallzone A (0..7) und Stufe spdlvl aus dir_tab.  Veraendert A, X, t4.
set_vel:
        sta t4
        lda spdlvl
        asl
        asl
        asl
        ora t4
        asl
        asl
        tax
        lda dir_tab,x
        sta vxl
        lda dir_tab + 1,x
        sta vxh
        lda dir_tab + 2,x
        sta vyl
        lda dir_tab + 3,x
        sta vyh
        rts

bounce_x:
        sec
        lda #0
        sbc vxl
        sta vxl
        lda #0
        sbc vxh
        sta vxh
        rts
bounce_y:
        sec
        lda #0
        sbc vyl
        sta vyl
        lda #0
        sbc vyh
        sta vyh
        rts

// paddle_test: der Ball waere auf Teilzeile t1 >= PADY.  C = 1: Schlaeger getroffen (Ball neu gesetzt, Richtung nach Zone), C = 0: vorbei.
//   rel = bx - px + 1 muss in 0 .. pw + 1 liegen; Zone = rel * 8 / (pw + 2), hoechstens 7.
paddle_test:
        lda bxh
        sec
        sbc px
        clc
        adc #1
        bmi .miss
        sta t4
        lda pw
        clc
        adc #1
        cmp t4
        bcc .miss
        lda t4
        asl
        asl
        asl
        sta t5
        lda pw
        clc
        adc #2
        sta t6
        ldx #0
.z1:    lda t5
        cmp t6
        bcc .z2
        sbc t6
        sta t5
        inx
        cpx #7
        bne .z1
.z2:    lda #PADY - 1
        sta byh
        lda #0
        sta byl
        txa
        jsr set_vel
        jsr sfx_paddle
        sec
        rts
.miss:  clc
        rts

// move_ball: einen Schritt; zuerst X, dann Y.  Trifft die neue Zelle auf etwas Festes, kehrt die Richtung um und der Stein wird getroffen.
//   Setzt lostf = 1, wenn der Ball verloren ist.
move_ball:
        clc
        lda bxl
        adc vxl
        sta t0
        lda bxh
        adc vxh
        sta t1
        lsr
        tax
        lda byh
        lsr
        tay
        jsr cell_static
        bcc .xok
        jsr bounce_x
        lda t1
        lsr
        tax
        lda byh
        lsr
        tay
        jsr hit_cell
        jmp .ydo
.xok:   lda t0
        sta bxl
        lda t1
        sta bxh
.ydo:   clc
        lda byl
        adc vyl
        sta t0
        lda byh
        adc vyh
        sta t1
        lda vyh
        bmi .ynp                        // aufwaerts
        lda t1
        cmp #PADY
        bcc .ynp
        jsr paddle_test                 // auf Schlaegerhoehe: Schlaeger getroffen?
        bcs .done
        lda t1
        cmp #LOSTY
        bcc .ynp
        lda #1
        sta lostf
        rts
.ynp:   lda bxh
        lsr
        tax
        lda t1
        lsr
        tay
        jsr cell_static
        bcc .yok
        jsr bounce_y
        lda bxh
        lsr
        tax
        lda t1
        lsr
        tay
        jsr hit_cell
        rts
.yok:   lda t0
        sta byl
        lda t1
        sta byh
.done:  rts

// ---------------------------------------------------------------------------------------------------------------------
// Spielablauf
// ---------------------------------------------------------------------------------------------------------------------
new_game:
.ifdef STARTLIVES
        lda #STARTLIVES
.else
        lda #3
.endif
        sta lives
        lda #0
        sta sc0
        sta sc1
        sta sc2
.ifdef STARTLEVEL
        lda #STARTLEVEL
.endif
        sta level
        jmp start_level

// start_level: Bild fuer das Level `level` aufbauen und mit klebendem Ball beginnen.
start_level:
        jsr clear_screen
        jsr draw_frame
        jsr draw_hud_static
        lda #0
        sta hits
        sta ballon
        sta lvdone
        sta lostf
        sta msgt
        lda level                       // Startgeschwindigkeit steigt mit dem Level: 0,0,0,1,1,1,2,...,7
        ldx #0
.sl1:   cmp #3
        bcc .sl2
        sbc #3
        inx
        bne .sl1
.sl2:   cpx #7
        bcc .sl3
        ldx #7
.sl3:   stx spdlvl
        jsr load_level
        jsr update_hud
        jsr reset_paddle
        jmp game_loop

reset_paddle:
        lda #12
        sta pw
        lda #34
        sta px
        jsr draw_paddle
        jmp place_ball

// draw_frame: obere Wand (Zeile 1) und Seitenwaende (Spalte 0 und 39, Zeile 1..24) aus Vollbloecken.
draw_frame:
        ldx #0
.f1:    ldy #1
        jsr setcell
        lda #$a0
        ldy #0
        sta (sp),y
        inx
        cpx #40
        bne .f1
        lda #2
        sta t0                          // Zeile 2..24: links und rechts
.f2:    ldx #0
        ldy t0
        jsr setcell
        lda #$a0
        ldy #0
        sta (sp),y
        ldx #39
        ldy t0
        jsr setcell
        lda #$a0
        ldy #0
        sta (sp),y
        inc t0
        lda t0
        cmp #25
        bne .f2
        rts

game_loop:
        jsr wait_frame
        jsr input_poll
        sta inp
        and #IN_ABORT
        beq .g1
        jsr sfx_off
        jmp ($fffc)                     // RUN/STOP: Neustart des Rechners (zurueck ins BASIC)
.g1:    lda inp
        and #IN_PAUSE
        beq .g2
        jsr do_pause
.g2:
.ifdef AUTOPLAY
.ifdef TESTLOSE
        lda #2                          // Testfassung: Schlaeger bleibt ganz links
.else
        lda bxh                         // Testfassung: Schlaeger folgt dem Ball
        sec
        sbc #6
        bcs .ap1
        lda #2
.ap1:
.endif
        jsr move_paddle
        lda ballon
        bne .g4
        lda frame                       // ... und schiesst nach einer kurzen Zeit ab
        and #15
        bne .g4
        jsr launch
        jmp .g4
.else
        lda inp
        and #IN_LEFT
        beq .g3
        lda px
        sec
        sbc #2
        bcs .gl
        lda #2
.gl:    jsr move_paddle
.g3:    lda inp
        and #IN_RIGHT
        beq .g3b
        lda px
        clc
        adc #2
        jsr move_paddle
.g3b:   lda inp
        and #IN_FIRE
        beq .g4
        lda ballon
        bne .g4
        jsr launch
.endif
.g4:    lda ballon
        beq .g5
        jsr erase_ball
        jsr move_ball
        jsr draw_ball
        lda lostf
        bne .lost
        lda lvdone
        bne .done
.g5:    lda msgt                        // Meldung in der Anzeige nach einiger Zeit loeschen
        beq .g6
        dec msgt
        bne .g6
        lda #<msg_none
        sta tp
        lda #>msg_none
        sta tp + 1
        ldx #33
        ldy #0
        jsr puts
.g6:    lda dirty
        beq .g7
        jsr update_hud
.g7:    jmp game_loop
.lost:  jsr sfx_lose
        dec lives
        lda #1
        sta dirty
        jsr update_hud
        lda #0
        sta lostf
        sta ballon
        jsr erase_ball
        jsr wait_long
        lda lives
        bne .alive
        jmp game_over
.alive: jsr erase_paddle                // Schlaeger neu (wieder normale Breite)
        jsr reset_paddle
        jmp game_loop
.done:  jsr sfx_level
        jsr wait_long
        inc level
        lda level
        cmp #level_count
        bcc .d1
        lda #0                          // nach dem letzten Level wieder von vorn (mit steigender Geschwindigkeit bleibt der Wert erhalten)
        sta level
.d1:    jmp start_level

wait_long:
        lda #60
        sta tick
.wl:    jsr wait_frame
        dec tick
        bne .wl
        rts

launch:
        lda #1
        sta ballon
        jsr rand_byte
        and #1
        clc
        adc #3                          // Zone 3 oder 4: fast senkrecht
        jmp set_vel

do_pause:
        jsr input_wait_release
.dp1:   jsr wait_frame
        jsr input_poll
        and #IN_PAUSE
        beq .dp1
        jmp input_wait_release

// ---------------------------------------------------------------------------------------------------------------------
// Titel, Spielende, Namenseingabe
// ---------------------------------------------------------------------------------------------------------------------
title:  jsr clear_screen
        jsr sfx_off
        lda #<t_title
        sta tp
        lda #>t_title
        sta tp + 1
        ldx #15
        ldy #3
        jsr puts
        lda #<t_help1
        sta tp
        lda #>t_help1
        sta tp + 1
        ldx #2
        ldy #6
        jsr puts
        lda #<t_help2
        sta tp
        lda #>t_help2
        sta tp + 1
        ldx #2
        ldy #7
        jsr puts
        lda #<t_help3
        sta tp
        lda #>t_help3
        sta tp + 1
        ldx #2
        ldy #8
        jsr puts
        lda #<t_best
        sta tp
        lda #>t_best
        sta tp + 1
        ldx #12
        ldy #11
        jsr puts
        jsr draw_best
        lda #<t_start
        sta tp
        lda #>t_start
        sta tp + 1
        ldx #6
        ldy #23
        jsr puts
.ifdef AUTOPLAY
        jmp new_game
.endif
        jsr input_wait_release
.tw:    jsr wait_frame
        jsr input_poll
        and #IN_FIRE
        beq .tw
        jsr rand_byte                   // Zufall mit der Wartezeit mischen
        jmp new_game

// draw_best: die ersten 5 Eintraege der Bestenliste ab Zeile 13: "n  NAME      PUNKTE"
draw_best:
        lda #<(hs_buf + HS_FIRST)
        sta lp
        lda #>(hs_buf + HS_FIRST)
        sta lp + 1
        lda #0
        sta t3                          // Eintrag 0..4
.b1:    lda t3
        clc
        adc #13
        tay
        ldx #12
        jsr setcell
        ldy #0
        lda t3
        clc
        adc #$31
        sta (sp),y                      // Platznummer 1..5
        ldy #0
.b2:    lda (lp),y                      // Name: ASCII -> Bildschirmcode (Buchstaben 65..90 -> 1..26)
        cmp #$40
        bcc .b3
        sec
        sbc #$40
.b3:    sta t4
        tya
        clc
        adc #3                          // Name ab Spalte 15 (sp steht auf Spalte 12)
        sty t5
        tay
        lda t4
        sta (sp),y
        ldy t5
        iny
        cpy #8
        bne .b2
        // Punkte: 3 Byte (niedrig, mittel, hoch) als 6 Ziffern ab Spalte 24
        ldy #10
        lda (lp),y
        sta t4                          // lo
        dey
        lda (lp),y
        sta t5                          // mid
        dey
        lda (lp),y
        sta t6                          // hi
        ldx #24
        lda t3
        clc
        adc #13
        tay
        jsr setcell
        lda t6
        jsr hud_bcd
        lda t5
        jsr hud_bcd
        lda t4
        jsr hud_bcd
        lda lp                          // naechster Eintrag (12 Byte weiter)
        clc
        adc #HS_ENTRY
        sta lp
        bcc .b4
        inc lp + 1
.b4:    inc t3
        lda t3
        cmp #5
        bne .b1
        rts

t_title: .screen "p r a l l b a l l"
         .byte 0
t_help1: .screen "z oder , = links   x oder . = rechts"
         .byte 0
t_help2: .screen "leertaste = abschiessen   p = pause"
         .byte 0
t_help3: .screen "run/stop = ende"
         .byte 0
t_best:  .screen "die besten"
         .byte 0
t_start: .screen "leertaste startet das spiel"
         .byte 0
t_over:  .screen "spiel zu ende"
         .byte 0
t_new:   .screen "neuer rekord! name: "
         .byte 0

// game_over: "SPIEL ZU ENDE"; Bestenliste pruefen, Namenseingabe und Speichern; danach zurueck zum Titel.
game_over:
        lda #<t_over
        sta tp
        lda #>t_over
        sta tp + 1
        ldx #13
        ldy #12
        jsr puts
        jsr wait_long
        lda sc0
        sta hs_points
        lda sc1
        sta hs_points + 1
        lda sc2
        sta hs_points + 2
        lda level
        clc
        adc #1
        sta hs_level
        jsr hs_check
        cmp #$ff
        beq .go9
        pha
        jsr name_entry
        pla
        jsr hs_insert
        jsr hs_save
.go9:   jmp title

// name_entry: bis zu 8 Zeichen ueber die Tastatur; RETURN beendet, DEL loescht.  Ergebnis in hs_name.
name_entry:
        lda #<t_new
        sta tp
        lda #>t_new
        sta tp + 1
        ldx #6
        ldy #15
        jsr puts
        ldx #7
        lda #$20
.ne0:   sta hs_name,x
        dex
        bpl .ne0
        lda #0
        sta npos
        jsr name_draw
.ne1:   jsr wait_frame
        jsr input_char
        beq .ne1
        cmp #$0d
        beq .ne9
        cmp #$14
        beq .del
        ldx npos
        cpx #8
        bcs .ne1
        sta hs_name,x
        inc npos
        jsr name_draw
        jmp .ne1
.del:   lda npos
        beq .ne1
        dec npos
        ldx npos
        lda #$20
        sta hs_name,x
        jsr name_draw
        jmp .ne1
.ne9:   lda #1
        sta in_kbclr                    // Tastaturpuffer wird wieder vom Interrupt geleert
        rts

// name_draw: den Namen in Zeile 15 ab Spalte 26 anzeigen; die Cursorstelle in Umkehrschrift.
name_draw:
        ldx #26
        ldy #15
        jsr setcell
        ldx #0
.nd1:   lda hs_name,x
        cmp #$40
        bcc .nd2
        sec
        sbc #$40                        // Buchstabe: ASCII -> Bildschirmcode
.nd2:   cpx npos
        bne .nd3
        ora #$80                        // Cursorstelle: Umkehrschrift
.nd3:   sta t4
        txa
        tay                             // Y = Stelle im Namen = Spaltenversatz ab Spalte 26
        lda t4
        sta (sp),y
        inx
        cpx #8
        bne .nd1
        rts

// ---------------------------------------------------------------------------------------------------------------------
// Bibliotheken und Daten
// ---------------------------------------------------------------------------------------------------------------------
.include "lib/input.asm"
.include "lib/rand.asm"
.include "lib/sound.asm"
.include "lib/hiscore.asm"
.include "lib/tabs.inc"                 // ovrow_lo/hi, rowlv, qtab (erzeugt von tools/gen.py)
.include "lib/dirtable.inc"             // dir_tab
.include "lib/levels.inc"               // level_count, level_data, level_tab_lo/hi
