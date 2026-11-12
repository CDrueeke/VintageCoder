; =====================================================================================================================
; hal_c64.asm - Hardware-Schicht (HAL) des C64 fuer "Hugo & Gumbo"
; =====================================================================================================================
;
; Die Spiellogik (common/jumprun/core.asm) kennt keine Hardware. Diese Datei uebersetzt ihre Wuensche in Zugriffe auf
; die C64-Chips.
;
; SCHNITTSTELLE (vollstaendig in core.asm beschrieben; hier, was diese HAL dazu liefert und braucht)
;   fuer den Kern:           hal_sfx (A = Effekt 0..7), hal_put_tile (= tile_put der Kachel-Bibliothek: X = Spalte, Y = Zeile,
;                            A = Kachel), hal_score_draw, sfx_on (Byte: ungleich 0, solange ein Effekt klingt)
;   fuer das Hauptprogramm:  hal_init, hal_wait_frame, hal_blank, hal_unblank, hal_title, hal_load_level (A = Bild),
;                            hal_update_sprites; ausserdem die Konstanten SCREEN und COLOR und die Tabelle skyc (von show_title benutzt)
;   Die HAL liest vom Kern: lvl, sc0..sc2 und die Ausgabewerte h_px_l/h/h_py/h_fr, e_px_l/h/e_py/e_fr; aus lib/levels.inc: lvl_mc1,
;   lvl_mc2, lvl_sky0..5, txt_hud, txt_lv_lo/hi, CH_DIGIT0, CH_STAR; aus dem Export: title_chars, title_attrs; aus der Kachel-
;   Bibliothek: tile_put, screen_select_a, screen_draw_all.  Sie veraendert nur eigene Nullseitenzellen und darf in allen Routinen
;   A, X, Y und die Flags beliebig aendern.  Die Musik (lib/music.inc) liest sfx_on, um Stimme 3 waehrend eines Effekts auszulassen.
;   Unterschiede zu den anderen Systemen (Sprites, Klang, Start): siehe core.asm, Abschnitt "Unterschiede".
;
;   VIC-II (Bild)
;     * Text-MEHRFARBMODUS, 40 x 25 Zeichen. Jedes Zeichen ist 4 x 8 breite Pixel gross (ein Pixel = 2 Bildpunkte breit,
;       deshalb 160 x 200 sichtbare Bildpunkte). Je Pixel stehen zwei Bit zur Wahl:
;           00 = Hintergrundfarbe 0   ($D021)     01 = Mehrfarbe 1 ($D022)
;           10 = Mehrfarbe 2          ($D023)     11 = Zeichenfarbe aus dem Farb-RAM ($D800, nur Farben 0-7)
;       Das Farb-RAM-Byte schaltet ein Zeichen zwischen Mehrfarbe (Bit 3 gesetzt) und Hires (Bit 3 geloescht) um.
;     * Der EIGENE ZEICHENSATZ liegt bei $2000 (2 KB); der Bildschirmspeicher bei $0400 (1000 Byte).
;     * SPRITES: Held und Gegner bestehen je aus ZWEI uebereinanderliegenden Sprites:
;           - einem MEHRFARB-Sprite fuer die Flaechen (Hemd, Haut, Hose): Sprite 1 (Held) und Sprite 3 (Gegner),
;           - einem HIRES-Sprite fuer Umriss, Auge und Mund (feine Details, eine Farbe): Sprite 0 (Held), Sprite 2 (Gegner).
;       Bei gleicher Position liegt die LOWERE Spritenummer VORNE, deshalb sind die Hires-Overlays die Sprites 0 und 2.
;     * HIMMEL: sechs Hintergrundfarben uebereinander (Rasterbalken). Ein Raster-Interrupt wechselt $D021 an festen Zeilen.
;
;   SID (Klang)
;     * Stimme 1 + 2 + 3 spielen die Musik (lib/music.inc, Tracker-Export). Stimme 3 wird fuer KLANGEFFEKTE geborgt: solange
;       ein Effekt klingt (sfx_on <> 0), setzt die Musik dort aus.
;
; SPEICHERPLAN (VIC-Bank 0, Adressen $0000-$3FFF sieht der VIC; $1000-$1FFF ist dort das Zeichen-ROM und daher unbrauchbar)
;     $0400-$07E7  Bildschirmspeicher         $07F8-$07FF  Sprite-Zeiger (Blocknummer = Adresse / 64)
;     $2000-$27FF  Zeichensatz                $2800-$31FF  Sprites (Held 14+14 Bilder, Gegner 6+6 Bilder)
;     $D800-$DBE7  Farb-RAM                   $C000-$C0EF  Bildkarte (LMAP, von der Spiellogik benutzt)
;
; ZEIGER IM NULLSEITENBEREICH: $50-$56 (sp, cp: Bildschirm- und Farb-RAM-Zeiger; pt0-pt2: Hilfsspeicher dieser Datei).
;   Frei von Ueberschneidung mit der Spiellogik ($10-$43), der Kachel-Bibliothek ($58-$5F) und der Musik ($F7-$FA).
;   Der Rasterinterrupt (irq, music_play, sfx_update) benutzt nur Zellen der Musik und eigene Speicherstellen, keine der Spiellogik.
; =====================================================================================================================

.const SCREEN = $0400
.const COLOR  = $d800
; Zeiger fuer (indirekt),y-Zugriffe und Hilfsspeicher der HAL (Nullseite; die Spiellogik belegt $10-$43, die Musik $F7-$FA)
.const sp  = $50                 ; Zeiger (2 Byte)
.const cp  = $52
.const pt0 = $54
.const pt1 = $55
.const pt2 = $56

; Sprite-Zeiger: Der VIC liest Sprite-Daten ab (Zeiger * 64) innerhalb der Bank. Beispiel: $2800 / 64 = $A0.
; Die vier Spritesaetze liegen hintereinander: Held-Basis 14 Bilder ($A0..$AD), Held-Overlay 14 ($AE..$BB), Gegner-Basis 6 ($BC..$C1),
; Gegner-Overlay 6 ($C2..$C7).
.const HERO_BASE_PTR  = $a0      ; $2800 / 64
.const HERO_OVER_PTR  = $ae      ; $2b80 / 64
.const ENEMY_BASE_PTR = $bc      ; $2f00 / 64
.const ENEMY_OVER_PTR = $c2      ; $3080 / 64

; ---------------------------------------------------------------- Start
; ---------------------------------------------------------------- Start
; hal_init: richtet den VIC, den Interrupt und die Sprites einmal beim Programmstart ein.  Gibt den Interrupt noch NICHT frei
;           (das macht das Hauptprogramm mit cli, nachdem auch die Musik initialisiert ist).
hal_init:
        sei                             ; Interrupts aus, solange die Chips eingerichtet werden
        lda #$7f                        ; Bit 7 = 0, alle anderen 1: "loeschen" -> alle CIA-Interrupt-Quellen aus (CIA 1 bei $DC0D, CIA 2 bei $DD0D)
        sta $dc0d                ; CIA-Interrupts aus
        sta $dd0d
        lda $dc0d
        lda $dd0d
        lda $dd00
        ora #$03                 ; VIC-Bank 0 ($0000-$3fff)
        sta $dd00
        lda #$18                 ; Bildschirm $0400, Zeichensatz $2000
        sta $d018
        lda #$18                 ; Mehrfarb-Text, 40 Spalten
        sta $d016
        lda #0
        sta $d020
        sta $d015                ; Sprites aus
        sta $d01b                ; Sprites vor dem Hintergrund
        sta $d017
        sta $d01d
        lda #%00001010           ; Sprites 1 und 3 mehrfarbig (die Basis-Sprites; die Hires-Overlays 0 und 2 bleiben einfarbig)
        sta $d01c
        lda #10                  ; Sprite-Mehrfarbe 1 (hellrot: Haut), gilt fuer alle Mehrfarb-Sprites
        sta $d025
        lda #14                  ; Sprite-Mehrfarbe 2 (hellblau: Hose, Augen)
        sta $d026
        lda #0
        sta $d027                ; Held-Overlay schwarz
        lda #2
        sta $d028                ; Held-Basis rot (Farbe fuer das Bitpaar 11, z. B. das Hemd)
        lda #0
        sta $d029                ; Gegner-Overlay schwarz
        lda #5
        sta $d02a                ; Gegner-Basis gruen (Bitpaar 11)
        lda #<irq                       ; KERNAL-IRQ-Vektor $0314/$0315 auf unsere Routine biegen
        sta $0314
        lda #>irq
        sta $0315
        lda #85                         ; erster Raster-Interrupt bei Zeile 85 (Ende des ersten Himmelsbalkens)
        sta $d012
        lda $d011
        and #$7f                        ; Bit 7 von $D011 = Bit 8 der Rasterzeile: 0, denn alle unsere Zeilen sind kleiner als 256
        sta $d011
        lda #1                          ; $D01A: Raster-Interrupt freigeben
        sta $d01a                ; Raster-Interrupt an
        lda #0
        sta band
        sta vblank
        sta sfx_on
        rts

; ---------------------------------------------------------------- Interrupt: Himmelsbalken und Bildtakt
; Der Raster-Interrupt feuert sechsmal je Bild, jeweils an einer Rasterzeile aus irq_line (85, 119, 153, 187, 221, 251).
; Bei jedem Aufruf wird die Hintergrundfarbe auf die naechste Balkenfarbe gesetzt: Zeile 51 ist die oberste Bildschirmzeile,
; die Balken sind je 34 Zeilen hoch.  irq_ci verteilt die Farben (Balken 1..5, danach wieder Balken 0 fuers naechste Bild).
; Beim LETZTEN Aufruf (Zeile 251, unterhalb des sichtbaren Bildes) passiert zusaetzlich das, was je Bild genau einmal laufen soll:
;   * music_play   die Musik einen Schritt weiter (SID-Register schreiben),
;   * sfx_update   den laufenden Klangeffekt einen Schritt weiter,
;   * vblank = 1   Signal an das Hauptprogramm: "naechstes Bild darf berechnet werden" (siehe hal_wait_frame).
; Quittieren: Schreiben von 1 nach $D019 loescht das Raster-Interrupt-Flag.  Beenden: $EA81 holt die vom KERNAL-Einsprung
; gesicherten Register zurueck und kehrt mit rti zurueck.  Der Vektor $0314/$0315 wird vom KERNAL-IRQ angesprungen (ROMs sind an).
; band = Nummer des Balkens, dessen Farbe jetzt gesetzt wird (0..5); irq_ci gibt den Eintrag in skyc: 1,2,3,4,5 und zuletzt 0.
irq:    lda #1
        sta $d019
        ldx band
        lda irq_ci,x
        tay
        lda skyc,y
        sta $d021
        cpx #5
        bne +
        jsr music_play                  ; Musik: einen Schritt (SID-Register schreiben)
        jsr sfx_update
        lda #1
        sta vblank
+       inx
        cpx #6
        bcc +
        ldx #0
+       stx band
        lda irq_line,x
        sta $d012
        jmp $ea81                       ; KERNAL-Ende: Register zurueckholen und rti

irq_line: .byte 85,119,153,187,221,251     ; Rasterzeile des Interrupts Nr. 0..5 (Abstand 34, der letzte liegt unter dem Bild)
irq_ci:   .byte 1,2,3,4,5,0               ; Index in skyc, der beim Interrupt Nr. 0..5 gesetzt wird
band:     .byte 0                         ; Nummer des naechsten Interrupts
vblank:   .byte 0                         ; 1 = Bildende gemeldet (vom Interrupt gesetzt, von hal_wait_frame geloescht)
skyc:     .byte 3,3,3,3,3,14              ; Hintergrundfarben der sechs Balken (hal_load_level / show_title ueberschreiben sie je Bild)

; hal_wait_frame: wartet, bis der Interrupt das naechste Bildende meldet (so laeuft die Spiellogik genau 50 Mal pro Sekunde).
;   Loescht vblank zuerst, wartet also immer auf das NAECHSTE Bildende (bis zu einem ganzen Bild).  Aendert nur A.
hal_wait_frame:
        lda #0
        sta vblank
-       lda vblank
        beq -
        rts


; hal_blank / hal_unblank: Bildschirm aus/an (Bit 4 von $D011 = DEN: Display Enable).  hal_blank schaltet ausserdem alle Sprites ab
; (hal_unblank nicht; die Sprites schaltet hal_update_sprites wieder ein).  Waehrend des Aufbaus eines neuen Bildes bleibt der
; Bildschirm so schwarz, man sieht kein halbfertiges Bild.  hal_blank wartet zuerst auf das Bildende, damit der Schalter im
; unsichtbaren Teil des Bildes umgelegt wird.  Beide aendern nur A.
hal_blank:
        jsr hal_wait_frame       ; Bildschirm erst im Austastbereich abschalten
        lda #$0b                        ; $D011 = $0B: Bit 4 (DEN) geloescht = Bildschirm aus; Bit 3 = 25 Zeilen; Y-Scroll 3
        sta $d011
        lda #0
        sta $d015                       ; Sprite-Freigabe: alle Sprites aus (Bits 0-3 waeren die Sprites 0-3)
        rts
hal_unblank:
        lda #$1b                        ; $D011 = $1B: DEN gesetzt = Bildschirm an, 25 Zeilen (Bit 3), Y-Scroll 3 (Standard)
        sta $d011
        rts

; ---------------------------------------------------------------- Kacheln
; Eine Kachel besteht aus 2 x 2 Zeichen (16 x 16 Bildpunkte).  Kachel (Spalte c, Zeile k) liegt auf dem Bildschirm in den Zeichenzellen
; (2c, 2k), (2c+1, 2k), (2c, 2k+1), (2c+1, 2k+1).  Das Zeichnen uebernimmt die Kachel-Bibliothek (#include <c64/tiles.asm>, in jumprun.asm
; eingebunden und mit tiles_use an den Kachelsatz hg und die Screen-Bank levels gebunden):
;   hal_put_tile   = tile_put: eine Kachel legen.  Eingang X = Spalte (0..19), Y = Zeile (0..11), A = Kachelnummer.  Zeichen und Farbe der Kachel
;                    (Farb-RAM-Wert: Bit 3 = Mehrfarbe, Bits 0-2 = Zeichenfarbe) kommen aus den Tabellen des Kachelsatzes (hg_c0..hg_c3, hg_col).
;   screen_select_a: A = Bildnummer; kopiert die Karte des Bildes nach LMAP (TILE_MAPRAM): die Spiellogik loescht Muenzen aus dieser Kopie.
;   screen_draw_all: zeichnet das ganze Bild aus LMAP (12 Zeilen x 20 Spalten).  Dauert einige Bilder lang; deshalb bei abgeschaltetem Bildschirm.
.const hal_put_tile = tile_put

; hal_title: kopiert das im Bildschirm-Editor gezeichnete Titelbild (title_chars, title_attrs: je 1000 Byte) in den Bildschirm
;   und das Farb-RAM.  copy1000 kopiert 1000 Byte von (sp) nach (cp): drei volle Seiten zu 256 Byte plus 232 Byte.
hal_title:
        lda #<title_chars
        sta sp
        lda #>title_chars
        sta sp+1
        lda #<SCREEN
        sta cp
        lda #>SCREEN
        sta cp+1
        jsr copy1000
        lda #<title_attrs
        sta sp
        lda #>title_attrs
        sta sp+1
        lda #<COLOR
        sta cp
        lda #>COLOR
        sta cp+1
copy1000:                        ; 1000 Byte von (sp) nach (cp)
        ldx #3                          ; drei volle Seiten (3 * 256 = 768 Byte) ...
.pg:    ldy #0
-       lda (sp),y
        sta (cp),y
        iny
        bne -
        inc sp+1
        inc cp+1
        dex
        bne .pg
        ldy #0
-       lda (sp),y
        sta (cp),y
        iny
        cpy #232                        ; ... plus die letzten 232 Byte: zusammen 1000
        bne -
        rts

; ---------------------------------------------------------------- Bild laden
; hal_load_level: A = Bildnummer.  Setzt die Mehrfarb-Farben 1 und 2 des Bildes ($D022/$D023: so sehen dieselben Kachel-
;   zeichen auf Wiese, in der Hoehle, in der Burg und am Himmel verschieden aus), die sechs Himmelsbalken (skyc),
;   kopiert die Kachelkarte nach LMAP (screen_select_a), zeichnet sie (screen_draw_all) und die Statuszeile.
hal_load_level:
        sta pt0
        tax
        lda lvl_mc1,x
        sta $d022
        lda lvl_mc2,x
        sta $d023
        lda lvl_sky0,x
        sta skyc
        lda lvl_sky1,x
        sta skyc+1
        lda lvl_sky2,x
        sta skyc+2
        lda lvl_sky3,x
        sta skyc+3
        lda lvl_sky4,x
        sta skyc+4
        lda lvl_sky5,x
        sta skyc+5
        ; Karte nach LMAP kopieren und das Bild zeichnen (Bildnummer = Screen-Nummer der Bank levels)
        lda pt0
        jsr screen_select_a
        jsr screen_draw_all
        jmp hal_hud

; hal_hud: Statuszeile (Zeile 0: "HUGO 000000   BILD n/4   LEBEN ***") und Laufschrift (Zeile 24: Name des Bildes).
;   Die Schrift sind Zeichen des eigenen Zeichensatzes (Nummern ab CH_FONT), die Farbe < 8 schaltet das Zeichen auf Hires.
hal_hud:
        ldx #0
-       lda txt_hud,x
        sta SCREEN+1,x
        lda #1
        sta COLOR+1,x
        inx
        cpx #30
        bne -
        ldx lvl
        txa
        clc
        adc #CH_DIGIT0+1                ; Ziffer des Bildes (1..4) als Zeichen
        sta SCREEN+20            ; Bildnummer
        lda #CH_STAR
        sta SCREEN+31
        sta SCREEN+32
        sta SCREEN+33
        lda #7
        sta COLOR+31
        sta COLOR+32
        sta COLOR+33
        ldx lvl
        lda txt_lv_lo,x
        sta sp
        lda txt_lv_hi,x
        sta sp+1
        ldy #0
-       lda (sp),y
        sta SCREEN+960+7,y
        lda #1                   ; Zeichenfarbe weiss: Farbwert unter 8 = Hires-Text (ab 8 wuerde die Hires-Schrift als Mehrfarbe gelesen)
        sta COLOR+960+7,y
        iny
        cpy #25
        bne -
        jmp hal_score_draw


; hal_score_draw: schreibt den Punktestand (sc2 sc1 sc0 als BCD) als sechs Ziffern in die Statuszeile (Spalten 6..11).
;   put_bcd zerlegt ein BCD-Byte in zwei Ziffern: obere Haelfte (vier Mal rechts schieben) und untere Haelfte (and #$0f);
;   Zeichennummer = CH_DIGIT0 + Ziffer.
hal_score_draw:
        lda sc2                         ; hoechstwertiges BCD-Byte: Spalten 6-7
        ldx #0
        jsr put_bcd
        lda sc1                         ; Spalten 8-9
        ldx #2
        jsr put_bcd
        lda sc0                         ; niedrigstes BCD-Byte: Spalten 10-11 (das rts von put_bcd beendet auch hal_score_draw)
        ldx #4
put_bcd:                         ; A = BCD, X = Spaltenversatz ab Spalte 6 (0, 2, 4); aendert A
        pha
        lsr a
        lsr a
        lsr a
        lsr a
        clc
        adc #CH_DIGIT0
        sta SCREEN+6,x
        pla
        and #$0f
        clc
        adc #CH_DIGIT0
        sta SCREEN+7,x
        rts

; ---------------------------------------------------------------- Sprites
; hal_update_sprites: setzt die Sprite-Register aus den Werten der Spiellogik (h_px/h_py/h_fr, e_px/e_py/e_fr).
;   Umrechnung logisch -> VIC-Koordinaten:
;     x_vic = x + 24      (der sichtbare Bildschirm beginnt bei x = 24)
;     y_vic = y_fuss - 20 + 50  = h_py + 30   beim Helden  (Sprite ist 21 Zeilen hoch, die Fuesse liegen 20 Zeilen unter dem
;                                              Sprite-Anfang; die erste Bildschirmzeile ist Rasterzeile 50)
;     y_vic = e_py + 36   beim Gegner  (Fuss 14 Zeilen unter dem Sprite-Anfang)
;   x kann ueber 255 liegen: das neunte Bit steht in $D010 (je ein Bit je Sprite).  Beide Sprites einer Figur bekommen
;   dieselbe Position; die Sprite-Zeiger waehlen das Bild (Bildnummer + Basis).  Schaltet am Ende alle vier Sprites ein.
;   Aendert A, X und die Hilfszellen pt0-pt2.  Wird einmal je Bild aus der Hauptschleife aufgerufen (nicht aus dem Interrupt).
hal_update_sprites:
        ; Held: x + 24, y + 30 (Fuss-y 20 Zeilen unter dem Sprite-Anfang, Rahmen bei Zeile 50)
        clc
        lda h_px_l
        adc #24
        sta pt0
        lda h_px_h
        adc #0
        sta pt1                  ; Bit 8 der x-Koordinate des Helden (0 oder 1)
        lda pt0
        sta $d000                       ; x (niedrige 8 Bit) von Sprite 0 (Held-Overlay) ...
        sta $d002                       ; ... und Sprite 1 (Held-Basis)
        lda h_py
        clc
        adc #30
        sta $d001
        sta $d003
        ; Gegner: Fuss 14 Zeilen unter dem Sprite-Anfang
        clc
        lda e_px_l
        adc #24
        sta pt0
        lda e_px_h
        adc #0
        sta pt2                         ; Bit 8 der x-Koordinate des Gegners
        lda pt0
        sta $d004                       ; Sprite 2 (Gegner-Overlay) und Sprite 3 (Gegner-Basis)
        sta $d006
        lda e_py
        clc
        adc #36
        sta $d005
        sta $d007
        lda #0
        ldx pt1
        beq +
        ora #%00000011
+       ldx pt2
        beq +
        ora #%00001100
+       sta $d010                       ; Bit 8 der x-Koordinaten (Bits 0-1 Held, 2-3 Gegner)
        lda h_fr                        ; Sprite-Zeiger = Bildnummer + Basiszeiger des jeweiligen Satzes (Overlay: Sprite 0 / 2, Basis: 1 / 3)
        clc
        adc #HERO_OVER_PTR
        sta $07f8
        lda h_fr
        clc
        adc #HERO_BASE_PTR
        sta $07f9
        lda e_fr
        clc
        adc #ENEMY_OVER_PTR
        sta $07fa
        lda e_fr
        clc
        adc #ENEMY_BASE_PTR
        sta $07fb
        lda #%00001111
        sta $d015
        rts

; ---------------------------------------------------------------- Klangeffekte (SID-Stimme 3: $D40E-$D414)
; Ein Effekt ist eine kurze Klangfolge, beschrieben durch 9 Byte:
;     0,1  Startfrequenz (lo, hi)  16 Bit: Frequenz in Hz = Wert * 985248 / 2^24 = Wert * 0,0587 (PAL).  Beispiel $0C00 = 3072 = 180 Hz
;     2,3  Frequenzaenderung je Bild (lo, hi), vorzeichenbehaftet: positiv = Ton steigt, negativ = faellt
;     4    Steuerbyte: Wellenform ($11 Dreieck, $21 Saege, $41 Rechteck, $81 Rauschen), Bit 0 = Gate (Ton an)
;     5    Dauer in Bildern                      6 AD = Anstieg/Abfall   7 SR = Haltepegel/Ausklang   8 Pulsbreite (hi)
; hal_sfx kopiert die Werte in die SID-Register und schaltet den Ton ein; sfx_update veraendert die Frequenz je Bild und
; schaltet den Ton nach Ablauf der Dauer ab (Gate aus).  sfx_on zeigt der Musik an, dass Stimme 3 gerade belegt ist.
; Effekt  0 Absprung (steigender Ton)   1 Landung (kurzes Rauschen)   2 Schritt (Klick)   3 Klettern (Blip)
;         4 Muenze (schneller Aufwaertsklang)   5 Gumbo klettert (Blubb)   6 Bildwechsel-Jingle   7 Fanfare
sfx_tab:
        .byte $00,$0c,$60,$00,$41,22,$08,$a4,$08     ; 0 Absprung: steigender Ton
        .byte $00,$14,$80,$ff,$81,7,$00,$a5,$08      ; 1 Landung: Rauschen
        .byte $00,$28,$00,$00,$81,2,$00,$90,$08      ; 2 Schritt: Klick
        .byte $00,$18,$00,$02,$11,4,$00,$a4,$08      ; 3 Klettern: Blip
        .byte $00,$40,$00,$03,$41,10,$00,$f0,$08     ; 4 Muenze: schneller Aufwaertsgleiter
        .byte $00,$0a,$b0,$ff,$11,12,$00,$a8,$08     ; 5 Gumbo: Blubb
        .byte $00,$20,$00,$01,$41,40,$08,$f4,$08     ; 6 Bildwechsel-Jingle
        .byte $00,$18,$00,$02,$41,60,$08,$f6,$08     ; 7 Fanfare
sfx_on:   .byte 0                         ; ungleich 0, solange ein Effekt klingt (core.asm und die Musik lesen es)
sfx_cnt:  .byte 0                         ; verbleibende Bilder des Effekts
sfx_fl:   .byte 0                         ; aktuelle Frequenz (lo)
sfx_fh:   .byte 0                         ; aktuelle Frequenz (hi)
sfx_dl:   .byte 0                         ; Frequenzaenderung je Bild (lo, 16 Bit mit Vorzeichen)
sfx_dh:   .byte 0                         ; ... (hi)
sfx_ctrl: .byte 0                         ; Steuerbyte (Wellenform + Gate) des Effekts

; hal_sfx: A = Effektnummer 0..7.  Zeile in sfx_tab = Nummer * 9 (Nummer*8 + Nummer).  Das Gate wird erst aus- und dann wieder eingeschaltet,
; damit die Huellkurve jedes Mal von vorn beginnt.  Aendert A, X und pt0.  sfx_on wird zuerst 0 (sfx_update tut dann nichts, solange
; die Werte gesetzt werden) und zuletzt wieder 1.
hal_sfx:
        sta pt0
        asl a
        asl a
        asl a
        clc
        adc pt0
        tax
        lda #0
        sta $d412                ; Gate aus (Neustart)
        sta sfx_on               ; Effekt waehrend des Setzens aus (der Interrupt ruft sfx_update)
        lda sfx_tab,x
        sta sfx_fl
        sta $d40e
        lda sfx_tab+1,x
        sta sfx_fh
        sta $d40f
        lda sfx_tab+2,x
        sta sfx_dl
        lda sfx_tab+3,x
        sta sfx_dh
        lda sfx_tab+4,x
        sta sfx_ctrl
        lda sfx_tab+5,x
        sta sfx_cnt
        lda sfx_tab+6,x
        sta $d413
        lda sfx_tab+7,x
        sta $d414
        lda #0
        sta $d410
        lda sfx_tab+8,x
        sta $d411
        lda sfx_ctrl
        sta $d412                ; Gate an
        lda #1
        sta sfx_on
        rts

; sfx_update: einmal je Bild vom Interrupt aufgerufen.  Addiert die Frequenzaenderung, zaehlt die Dauer herunter und beendet den Effekt.
;   Aendert A (der Interrupt-Einsprung sichert die Register).
sfx_update:
        lda sfx_on
        beq +
        clc
        lda sfx_fl
        adc sfx_dl                      ; Frequenz += Aenderung je Bild (16 Bit)
        sta sfx_fl
        sta $d40e
        lda sfx_fh
        adc sfx_dh
        sta sfx_fh
        sta $d40f
        dec sfx_cnt
        bne +
        lda sfx_ctrl
        and #$fe                        ; Gate (Bit 0) loeschen: der Ton klingt mit dem Ausklang (Release) aus
        sta $d412
        lda #0
        sta sfx_on
+       rts
