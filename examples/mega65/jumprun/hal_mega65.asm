; =====================================================================================================================
; hal_mega65.asm - Hardware-Schicht (HAL) des MEGA65 fuer "Hugo & Gumbo"  (Stufe 1: C64-Code im VIC-II-Modus)
; =====================================================================================================================
;
; Die Spiellogik (common/jumprun/core.asm) kennt keine Hardware. Diese Datei uebersetzt ihre Wuensche in Zugriffe auf
; die Chips des MEGA65.
;
; SCHNITTSTELLE (vollstaendig in core.asm beschrieben; hier, was diese HAL dazu liefert und braucht)
;   fuer den Kern:           hal_sfx (A = Effekt 0..7), hal_put_tile (X = Spalte, Y = Zeile, A = Kachel), hal_score_draw,
;                            sfx_on (Byte: ungleich 0, solange ein Effekt klingt; die Musik liest es hier nicht)
;   fuer das Hauptprogramm:  hal_init, hal_wait_frame, hal_blank, hal_unblank, hal_title, hal_load_level (A = Bild),
;                            hal_update_sprites (liest h_px_l/h/h_py/h_fr und e_px_l/h/e_py/e_fr); ausserdem SCREEN, COLOR und die
;                            Tabelle skyc (von show_title direkt benutzt)
;   interne Hilfen:          hal_draw_level (alle 240 Kacheln aus LMAP zeichnen, benutzt get_tile des Kerns), hal_hud, put_bcd,
;                            copy1000, sfx_update, irq
;   Die HAL liest vom Kern: lvl, sc0..sc2 und die Ausgabewerte; aus lib/levels.inc: lvl_mc1/2, lvl_sky0..5, txt_hud, txt_lv_lo/hi,
;   CH_DIGIT0, CH_STAR; aus jumprun.asm: maps_lo/hi; aus dem Export: title_chars, title_attrs, lv1_tiles, lv1_colors.  Sie veraendert nur
;   eigene Nullseitenzellen und darf in allen Routinen A, X, Y und die Flags beliebig aendern.
;   Unterschiede zu den anderen Systemen (Sprites, Klang, Start): siehe core.asm, Abschnitt "Unterschiede".
;
; Sie ist die Schwester von hal_c64.asm: Der MEGA65 enthaelt einen C64-kompatiblen Bildchip
; (VIC-IV, der im VIC-II-Modus wie der C64-VIC arbeitet) und mehrere SID-Klangchips, deshalb bleibt der Grossteil
; (Rasterbalken, Kacheln, Sprites, Effekttabelle) Zeile fuer Zeile gleich.  Der MEGA65 startet aber nicht in einem
; C64-Zustand, sondern in der Umgebung von BASIC 65.  Die ANPASSUNGEN gegenueber dem C64 sind darum:
;
;   * CPU 45GS02 statt 6510: Der Befehlssatz ist eine Obermenge des 6502; unser Code benutzt nur 6502-Befehle.  Neu ist
;     nur der Start (map/eom, taz), mit dem hal_init den Adressraum in einen schlichten 64-KB-RAM-Zustand versetzt.
;   * $01 = $35: Weder BASIC- noch KERNAL-ROM sind sichtbar, das RAM liegt bis $FFFF frei, Ein-/Ausgabe bei $D000.
;     Folge: Es gibt KEINEN KERNAL-Interrupt-Einsprung.  Der Prozessor liest den Vektor direkt aus dem RAM ($FFFE/$FFFF),
;     und die Interrupt-Routine muss ihre Register selbst sichern und mit rti enden (siehe irq).
;   * VIC-IV-Freischaltung ($D02F = $47, $53) und Ruecksetzen der VIC-III/IV-Register ($D030, $D031, $D054, $D058, $D05E),
;     weil BASIC 65 den Bildchip in einen anderen Modus (z. B. 80 Spalten) geschaltet haben kann.
;   * VIC-BANK 1 ($4000-$7FFF) statt Bank 0: Die Kunstdaten (Zeichensatz, Sprites) liegen wegen des laengeren Programms und
;     der Bank-0-Zeichen-ROM-Falle des VIC an anderen Adressen.  Bildschirm $4400, Zeichensatz $6000, Sprites $6800.
;   * KLANGEFFEKTE auf dem ZWEITEN SID ($D420-$D438).  Die Musik hat den ersten SID ($D400) mit allen drei Stimmen fuer
;     sich allein; der Effekt-SID braucht deshalb keine "geborgte" Stimme 3 und die Musik muss nie aussetzen.
;
;   VIC-II-Modus (Bild)
;     * Text-MEHRFARBMODUS, 40 x 25 Zeichen. Jedes Zeichen ist 4 x 8 breite Pixel gross (ein Pixel = 2 Bildpunkte breit,
;       deshalb 160 x 200 sichtbare Bildpunkte). Je Pixel stehen zwei Bit zur Wahl:
;           00 = Hintergrundfarbe 0   ($D021)     01 = Mehrfarbe 1 ($D022)
;           10 = Mehrfarbe 2          ($D023)     11 = Zeichenfarbe aus dem Farb-RAM ($D800, nur Farben 0-7)
;       Das Farb-RAM-Byte schaltet ein Zeichen zwischen Mehrfarbe (Bit 3 gesetzt) und Hires (Bit 3 geloescht) um.
;     * Der EIGENE ZEICHENSATZ liegt bei $6000 (2 KB); der Bildschirmspeicher bei $4400 (1000 Byte).
;     * SPRITES: Held und Gegner bestehen je aus ZWEI uebereinanderliegenden Sprites:
;           - einem MEHRFARB-Sprite fuer die Flaechen (Hemd, Haut, Hose): Sprite 1 (Held) und Sprite 3 (Gegner),
;           - einem HIRES-Sprite fuer Umriss, Auge und Mund (feine Details, eine Farbe): Sprite 0 (Held), Sprite 2 (Gegner).
;       Bei gleicher Position liegt die LOWERE Spritenummer VORNE, deshalb sind die Hires-Overlays die Sprites 0 und 2.
;     * HIMMEL: sechs Hintergrundfarben uebereinander (Rasterbalken). Ein Raster-Interrupt wechselt $D021 an festen Zeilen.
;
;   SID (Klang)
;     * SID 1 ($D400): Stimme 1 + 2 + 3 spielen die Musik (lib/music.inc, Tracker-Export).
;     * SID 2 ($D420): Klangeffekte (Stimme 3 dieses Chips: $D42E-$D434; Lautstaerke $D438 = $D420 + $18).  Im Gegensatz
;       zum C64 setzt die Musik waehrend eines Effekts nicht aus; sfx_on bleibt nur als Zustandsmerker des Effekts.
;
; SPEICHERPLAN (MEGA65, VIC-Bank 1: der VIC sieht $4000-$7FFF; $D018 und die Sprite-Zeiger zaehlen von $4000 an)
;     $4400-$47E7  Bildschirmspeicher         $47F8-$47FF  Sprite-Zeiger (Blocknummer = Adresse in der Bank / 64)
;     $6000-$67FF  Zeichensatz                $6800-$71FF  Sprites (Held 14+14 Bilder, Gegner 6+6 Bilder)
;     $D800-$DBE7  Farb-RAM                   $9000-$90EF  Bildkarte (LMAP, von der Spiellogik benutzt)
;   (Das Programm selbst liegt bei $2001-$2F2E, also unterhalb der VIC-Bank; die Bank-Daten werden von jumprun.asm
;   mit .org an die festen Adressen gelegt.)
;
; ZEIGER IM NULLSEITENBEREICH: $50-$56 (sp, cp: Bildschirm- und Farb-RAM-Zeiger; pt0-pt2: Hilfsspeicher dieser Datei).
; =====================================================================================================================

#include <mega65/gs4502.asm>   // Makro mega65_enable_vic4 (schaltet den VIC-IV frei, siehe hal_init)
.const SCREEN = $4400                  ; Bildschirm in VIC-Bank 1 ($4000-$7FFF): Bank-Anfang $4000 + $0400 aus $D018
.const COLOR  = $d800                  ; Farb-RAM-Fenster; bei $01 = $35 sichtbar (der Farbspeicher liegt fest, nicht in der VIC-Bank)
; Zeiger fuer (indirekt),y-Zugriffe und Hilfsspeicher der HAL (Nullseite; die Spiellogik belegt $10-$43, die Musik $F7-$FA)
.const sp  = $50                 ; Zeiger (2 Byte) auf Quelle bzw. Bildschirmzelle
.const cp  = $52                 ; Zeiger (2 Byte) auf Ziel bzw. Farb-RAM-Zelle
.const pt0 = $54                 ; Hilfsbyte 0 (Kachelnummer, Zwischenwerte)
.const pt1 = $55                 ; Hilfsbyte 1 (Zeile; Bit 8 der Held-x-Koordinate)
.const pt2 = $56                 ; Hilfsbyte 2 (Spalte; Bit 8 der Gegner-x-Koordinate)

; Sprite-Zeiger: Der VIC liest Sprite-Daten ab (Zeiger * 64) innerhalb der Bank, also hier ab $4000 + Zeiger * 64.
; Beispiel: Zeiger $A0 -> $A0 * 64 = $2800 in der Bank = $6800 im Speicher des MEGA65.
; Die vier Spritesaetze liegen hintereinander: Held-Basis 14 Bilder ($A0..$AD), Held-Overlay 14 ($AE..$BB), Gegner-Basis 6 ($BC..$C1),
; Gegner-Overlay 6 ($C2..$C7).
; (Die Zeigerwerte sind dieselben wie am C64, weil Bank 1 gegenueber Bank 0 nur um $4000 verschoben ist, was der VIC
; nicht mitzaehlt: im VIC-Adressraum liegen die Sprites weiter bei $2800 ff.)
.const HERO_BASE_PTR  = $a0      ; $2800 in der Bank = $6800 im Speicher
.const HERO_OVER_PTR  = $ae      ; $2b80 in der Bank = $6b80
.const ENEMY_BASE_PTR = $bc      ; $2f00 in der Bank = $6f00
.const ENEMY_OVER_PTR = $c2      ; $3080 in der Bank = $7080

; ---------------------------------------------------------------- Start
; hal_init: richtet den Adressraum, den VIC, den Interrupt und die Sprites einmal beim Programmstart ein.  Gibt den
;           Interrupt noch NICHT frei (das macht das Hauptprogramm mit cli, nachdem auch die Musik initialisiert ist).  Aendert A, X, Y, Z.
;           Der NMI-Vektor ($FFFA/$FFFB) zeigt auf rti_only (ein NMI, z. B. RESTORE, tut nichts), wie beim Plus/4.
hal_init:
        sei                             ; Interrupts aus, solange die Chips eingerichtet werden
        ; ---- MEGA65: aus dem BASIC-65-Zustand in ein reines "C64-artiges" 64-KB-RAM mit Ein-/Ausgabe bei $D000 wechseln
        ; map nimmt seine Parameter aus den vier Registern A, X, Y, Z (Verschiebung und Freigabebits fuer die vier 8-KB-Fenster
        ; der unteren und oberen 32 KB).  Mit A = X = Y = Z = 0 ist kein Fenster umgeblendet: jede CPU-Adresse zeigt auf
        ; denselben Platz in Bank 0.  Dazu muessen alle vier Register 0 sein (taz setzt Z aus A).
        lda #$00
        tax
        tay
        taz
        map                             ; Speicherabbildung des Betriebssystems aufheben: alle 64 KB sind RAM (Bank 0)
        eom                             ; beendet die Interruptsperre des map (sie verhindert Interrupts mit halber Abbildung)
        lda #$35
        sta $01                         ; CPU-Port: RAM ueberall, Ein-/Ausgabe bei $D000 (keine ROMs): LORAM = 1, HIRAM = 0, CHAREN = 1
        ; ---- VIC-IV freischalten: Die erweiterten Register (und die Freiheit, sie zu setzen) gibt es erst nach dem Schluessel
        ; $47, $53 in dieser Reihenfolge nach $D02F.  Danach setzen wir die Bildchip-Einstellungen zurueck, die BASIC 65 veraendert
        ; haben kann, damit der Chip sich wie ein reiner C64-VIC-II verhaelt.
        mega65_enable_vic4              ; VIC-IV freischalten: erst $47, dann $53 nach $D02F (Makro aus <mega65/gs4502.asm>)
        lda #0
        sta $d030                       ; keine C65-ROM-Fenster (Bits fuer ROM bei $8000/$A000/$C000/$E000 und Farb-RAM-2K-Fenster aus)
        sta $d031                       ; VIC-III: 40 Spalten (H640 aus), keine Zeilenverdopplung, C65-Schnelltakt (FAST) aus
        sta $d054                       ; VIC-IV: keine 16-Bit-Zeichen, keine Vollfarb-Zeichen, normale Sprites
        lda #40
        sta $d058                       ; Zeilenabstand des Textbildschirms: 40 Byte (Low-Byte; BASIC 65 stellt 80 ein)
        lda #0
        sta $d059                       ; ... High-Byte 0: 40 = $0028
        lda #40
        sta $d05e                       ; 40 Zeichen je Zeile (kein 80-Spalten-Text)
        lda #15
        sta $d438                       ; zweiter SID ($D420): Lautstaerke 15 (er gehoert den Klangeffekten; $D438 = $D420 + $18);
                                        ; den ersten SID stellt music_init ein
        lda #$7f                        ; Bit 7 = 0, andere 1: "loeschen" -> alle CIA-Interrupt-Quellen aus (CIA 1 bei $DC0D, CIA 2 bei $DD0D)
        sta $dc0d                ; CIA-Interrupts aus (Tastaturabfrage, Zeitgeber): nur der Raster-Interrupt soll uebrigbleiben
        sta $dd0d
        lda $dc0d                       ; Lesen quittiert noch anstehende CIA-Interrupts
        lda $dd0d
        lda $dd02
        ora #$03
        sta $dd02                ; Bits 0-1 von CIA 2 Port A als Ausgang (VIC-Bank-Wahl)
        lda $dd00
        and #$fc                        ; Bank-Bits loeschen ...
        ora #$02                 ; VIC-Bank 1 ($4000-$7fff): die Bits sind invertiert, %10 waehlt Bank 1 (%11 waere Bank 0)
        sta $dd00
        lda #$18                 ; Bildschirm: Hochnibble 1 * $400 = $0400 in der Bank = $4400; Zeichensatz: Bits 1-3 = 4 * $800 = $2000 -> $6000
        sta $d018
        lda #$18                 ; Mehrfarb-Text (Bit 4), 40 Spalten (Bit 3)
        sta $d016
        lda #0
        sta $d020                       ; Rahmenfarbe schwarz
        sta $d015                ; Sprites aus
        sta $d01b                ; Sprites vor dem Hintergrund
        sta $d017                       ; keine Sprite-Hoehenverdopplung
        sta $d01d                       ; keine Sprite-Breitenverdopplung
        lda #%00001010           ; Sprites 1 und 3 mehrfarbig
        sta $d01c
        lda #10                  ; Mehrfarb 1 (Haut)
        sta $d025
        lda #14                  ; Mehrfarb 2 (Hose, Augen)
        sta $d026
        lda #0
        sta $d027                ; Held-Overlay schwarz
        lda #2
        sta $d028                ; Held-Basis rot
        lda #0
        sta $d029                ; Gegner-Overlay schwarz
        lda #5
        sta $d02a                ; Gegner-Basis gruen
        lda #<irq                       ; Interrupt-Vektor im RAM ($FFFE/$FFFF; das Betriebssystem ist abgeschaltet)
        sta $fffe                       ; (ohne KERNAL-ROM gibt es den Umweg ueber $0314/$0315 nicht, der Prozessor springt direkt hierher)
        lda #>irq
        sta $ffff
        lda #<rti_only                  ; NMI-Vektor ($FFFA/$FFFB) im RAM auf ein blosses rti: ein NMI (RESTORE) tut nichts
        sta $fffa
        lda #>rti_only
        sta $fffb
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

rti_only: rti                          ; Ziel des NMI-Vektors (siehe hal_init)

; ---------------------------------------------------------------- Interrupt: Himmelsbalken und Bildtakt
; Der Raster-Interrupt feuert sechsmal je Bild, jeweils an einer Rasterzeile aus irq_line (85, 119, 153, 187, 221, 251).
; Bei jedem Aufruf wird die Hintergrundfarbe auf die naechste Balkenfarbe gesetzt: Zeile 51 ist die oberste Bildschirmzeile,
; die Balken sind je 34 Zeilen hoch.  irq_ci verteilt die Farben (Balken 1..5, danach wieder Balken 0 fuers naechste Bild).
; Beim LETZTEN Aufruf (Zeile 251, unterhalb des sichtbaren Bildes) passiert zusaetzlich das, was je Bild genau einmal laufen soll:
;   * music_play   die Musik einen Schritt weiter (SID-Register schreiben),
;   * sfx_update   den laufenden Klangeffekt einen Schritt weiter,
;   * vblank = 1   Signal an das Hauptprogramm: "naechstes Bild darf berechnet werden" (siehe hal_wait_frame).
; Quittieren: Schreiben von 1 nach $D019 loescht das Raster-Interrupt-Flag.
; band = Nummer des Balkens, dessen Farbe jetzt gesetzt wird (0..5); irq_ci gibt den Eintrag in skyc: 1,2,3,4,5 und zuletzt 0.
; Unterschied zum C64: Hier gibt es keinen KERNAL-Einsprung, der die Register sichert, und kein "jmp $EA81".  Die Routine
; legt A, X und Y selbst auf den Stapel (der Prozessor hat nur Statusregister und Ruecksprungadresse gesichert), holt sie am
; Ende zurueck und kehrt mit rti zurueck.  (Das Z-Register der 45GS02 benutzen wir nicht, es muss nicht gesichert werden.)
irq:    pha                             ; Register sichern (ohne KERNAL muss die Routine das selbst tun)
        txa
        pha
        tya
        pha
        lda #1
        sta $d019                       ; Raster-Interrupt quittieren
        ldx band                        ; X = Nummer des Balkens, der jetzt beginnt
        lda irq_ci,x                    ; Farbindex dieses Balkens ...
        tay
        lda skyc,y                      ; ... in der Farbtabelle des aktuellen Bildes nachschlagen ...
        sta $d021                       ; ... und als Hintergrundfarbe setzen
        cpx #5                          ; letzter Aufruf des Bildes (unterhalb des sichtbaren Bereichs)?
        bne +
        jsr music_play                  ; Musik: einen Schritt (SID-Register schreiben)
        jsr sfx_update
        lda #1
        sta vblank
+       inx                             ; naechster Balken; nach dem sechsten wieder von vorn
        cpx #6
        bcc +
        ldx #0
+       stx band
        lda irq_line,x                  ; Rasterzeile des naechsten Interrupts
        sta $d012
        pla                             ; Register zurueckholen
        tay
        pla
        tax
        pla
        rti

irq_line: .byte 85,119,153,187,221,251  ; Rasterzeilen der sechs Interrupts je Bild
irq_ci:   .byte 1,2,3,4,5,0             ; Farbindex, der beim jeweiligen Aufruf gesetzt wird
band:     .byte 0                       ; laufende Nummer des naechsten Interrupts (0..5)
vblank:   .byte 0                       ; 1 = ein Bild ist zu Ende (vom Interrupt gesetzt, von hal_wait_frame geloescht)
skyc:     .byte 3,3,3,3,3,14            ; die sechs Himmelsfarben (hal_load_level ueberschreibt sie je Bild)

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
        sta $d015                       ; Sprite-Freigabe: alle Sprites aus (Bits 0-3 waeren Sprites 0-3)
        rts
hal_unblank:
        lda #$1b                        ; $D011 = $1B: DEN gesetzt = Bildschirm an, 25 Zeilen (Bit 3), Y-Scroll 3 (Standard)
        sta $d011
        rts

; ---------------------------------------------------------------- Kacheln
; Eine Kachel besteht aus 2 x 2 Zeichen (16 x 16 Bildpunkte).  Kachel (Spalte c, Zeile k) liegt auf dem Bildschirm
; in den Zeichenzellen (2c, 2k), (2c+1, 2k), (2c, 2k+1), (2c+1, 2k+1).  Die Adresse der Zeichenzeile 2k ist
; $4400 + 80 * k (jede Zeichenzeile hat 40 Byte, eine Kachelzeile also 80).  Die Tabellen row80_lo/hi liefern diese Adressen.
row80_lo:
    .repeat 12, i
        .byte <(SCREEN + i * 80)
    .endrepeat
row80_hi:
    .repeat 12, i
        .byte >(SCREEN + i * 80)
    .endrepeat

; hal_put_tile: legt eine Kachel auf den Bildschirm.  Eingang X = Spalte (0..19), Y = Zeile (0..11), A = Kachelnummer.
;   Schritte: (1) Zeiger sp auf die obere linke Zeichenzelle = row80[Zeile] + 2 * Spalte,
;             (2) Zeiger cp auf die gleiche Zelle im Farb-RAM = sp + $9400 (Farb-RAM $D800 minus Bildschirm $4400),
;             (3) die vier Zeichennummern der Kachel aus lv1_tiles holen (4 Byte je Kachel: oben links, oben rechts,
;                 unten links, unten rechts) und an Offsets 0, 1, 40, 41 schreiben,
;             (4) die Farbe der Kachel (lv1_colors, ein Byte je Kachel) in alle vier Farb-RAM-Zellen schreiben.
;   Die Kachelbeschreibung stammt aus dem Tilemap-Export von lv1; sie ist fuer alle vier Bilder gleich.  Aendert A, X, Y, sp, cp, pt0.
hal_put_tile:
        sta pt0                         ; Kachelnummer merken
        lda row80_lo,y                  ; sp = Adresse der Zeichenzeile 2 * Zeile
        sta sp
        lda row80_hi,y
        sta sp+1
        txa
        asl a                           ; Spalte * 2 = Zeichenspalte der Kachel (jede Kachel ist 2 Zeichen breit)
        clc
        adc sp
        sta sp
        bcc +
        inc sp+1
+       lda sp                          ; cp = sp, aber im Farb-RAM
        sta cp
        lda sp+1
        clc
        adc #(>COLOR) - (>SCREEN) ; Farb-RAM = Bildschirm + $9400 ($D800 - $4400); nur das High-Byte aendert sich
        sta cp+1
        lda pt0
        asl a
        asl a
        tax
        ldy #0
        lda lv1_tiles,x                 ; Zeichen oben links der Kachel (X = Kachelnummer * 4)
        sta (sp),y
        iny
        lda lv1_tiles+1,x               ; oben rechts
        sta (sp),y
        ldy #40                         ; eine Zeichenzeile tiefer (40 Zellen)
        lda lv1_tiles+2,x               ; unten links
        sta (sp),y
        iny
        lda lv1_tiles+3,x               ; unten rechts
        sta (sp),y
        ldx pt0
        lda lv1_colors,x                ; Farbe der Kachel (Farb-RAM-Wert: Bit 3 = Mehrfarbe, Bits 0-2 = Zeichenfarbe)
        ldy #0
        sta (cp),y
        iny
        sta (cp),y
        ldy #40
        sta (cp),y
        iny
        sta (cp),y
        rts

; hal_draw_level: zeichnet das ganze Bild aus der Karte im RAM (LMAP): 12 Zeilen x 20 Spalten, je Kachel get_tile + hal_put_tile.
;   Dauert einige Bilder lang (240 Kacheln); deshalb laeuft sie bei abgeschaltetem Bildschirm.  pt1 = Zeile, pt2 = Spalte.
;   Aendert A, X, Y, sp, cp, pt0..pt2.
hal_draw_level:
        lda #0
        sta pt1                  ; Zeile
.row:   lda #0
        sta pt2                  ; Spalte
.col:   ldx pt2                         ; get_tile (Spiellogik): liefert die Kachelnummer an (X, Y) in A
        ldy pt1
        jsr get_tile
        ldx pt2
        ldy pt1
        jsr hal_put_tile
        inc pt2
        lda pt2
        cmp #20
        bcc .col
        inc pt1
        lda pt1
        cmp #12
        bcc .row
        rts

; hal_title: kopiert das im Bildschirm-Editor gezeichnete Titelbild (title_chars, title_attrs: je 1000 Byte) in den Bildschirm
;   ($4400) und das Farb-RAM ($D800).  copy1000 kopiert 1000 Byte von (sp) nach (cp): drei volle Seiten zu 256 Byte plus 232 Byte.
;   Aendert A, X, Y, sp, cp.
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
        inc sp+1                        ; naechste Seite in Quelle und Ziel
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

; ---------------------------------------------------------------- Bild laden: Farben, Karte, Statuszeile
; hal_load_level: A = Bildnummer.  Setzt die Mehrfarb-Farben 1 und 2 des Bildes ($D022/$D023: so sehen dieselben Kachel-
;   zeichen auf Wiese, in der Hoehle, in der Burg und am Himmel verschieden aus), die sechs Himmelsbalken (skyc),
;   kopiert die Kachelkarte nach LMAP ($9000), zeichnet sie und die Statuszeile.  Aendert alle Register.
hal_load_level:
        sta pt0
        tax                             ; X = Bildnummer als Tabellenindex
        lda lvl_mc1,x
        sta $d022
        lda lvl_mc2,x
        sta $d023
        lda lvl_sky0,x                  ; die sechs Himmelsfarben des Bildes in die Tabelle des Interrupts
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
        ; Karte nach LMAP kopieren
        lda maps_lo,x                   ; Zeiger auf die Kartendaten dieses Bildes (Tabelle im Hauptprogramm)
        sta sp
        lda maps_hi,x
        sta sp+1
        ldy #0
-       lda (sp),y
        sta LMAP,y
        iny
        cpy #240                        ; 240 Byte = 20 Spalten * 12 Zeilen
        bne -
        jsr hal_draw_level
        jmp hal_hud

; hal_hud: Statuszeile (Zeile 0: "HUGO 000000   BILD n/4   LEBEN ***") und Laufschrift (Zeile 24: Name des Bildes).
;   Die Schrift sind Zeichen des eigenen Zeichensatzes (Nummern ab CH_FONT).  Der Farbwert 1 (weiss) hat Bit 3 geloescht:
;   das Zeichen wird als HIRES-Text gezeichnet (volle Schaerfe, 8 x 8 Punkte), nicht als Mehrfarbe.
hal_hud:
        ldx #0
-       lda txt_hud,x
        sta SCREEN+1,x                  ; Zeile 0, ab Spalte 1: 30 Zeichen Text
        lda #1                          ; Farbwert 1 = weiss, Bit 3 aus = Hires
        sta COLOR+1,x
        inx
        cpx #30
        bne -
        ldx lvl
        txa
        clc
        adc #CH_DIGIT0+1                ; Ziffer des Bildes (1..4) als Zeichen
        sta SCREEN+20            ; Bildnummer
        lda #CH_STAR                    ; drei Lebenssterne
        sta SCREEN+31
        sta SCREEN+32
        sta SCREEN+33
        lda #7
        sta COLOR+31                    ; gelb, ebenfalls unter 8 = Hires
        sta COLOR+32
        sta COLOR+33
        ldx lvl                         ; Zeile 24: Name des Bildes (Zeiger aus der Tabelle der Texte)
        lda txt_lv_lo,x
        sta sp
        lda txt_lv_hi,x
        sta sp+1
        ldy #0
-       lda (sp),y
        sta SCREEN+960+7,y              ; 960 = 24 * 40: Beginn der letzten Zeile; Text ab Spalte 7
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
        lda sc2                         ; hoechstwertige zwei Ziffern ab Spalte 6
        ldx #0
        jsr put_bcd
        lda sc1                         ; mittlere zwei Ziffern ab Spalte 8
        ldx #2
        jsr put_bcd
        lda sc0                         ; niedrigste zwei Ziffern ab Spalte 10
        ldx #4
put_bcd:                         ; A = BCD, X = Spaltenversatz ab Spalte 6 (0, 2, 4); aendert A
        pha
        lsr a
        lsr a
        lsr a
        lsr a
        clc
        adc #CH_DIGIT0
        sta SCREEN+6,x                  ; Zehnerziffer
        pla
        and #$0f
        clc
        adc #CH_DIGIT0
        sta SCREEN+7,x                  ; Einerziffer
        rts

; ---------------------------------------------------------------- Sprites
; hal_update_sprites: setzt die Sprite-Register aus den Werten der Spiellogik (h_px/h_py/h_fr, e_px/e_py/e_fr).
;   Umrechnung logisch -> VIC-Koordinaten (im VIC-II-Modus des MEGA65 unveraendert gegenueber dem C64):
;     x_vic = x + 24      (der sichtbare Bildschirm beginnt bei x = 24)
;     y_vic = y_fuss - 20 + 50  = h_py + 30   beim Helden  (Sprite ist 21 Zeilen hoch, die Fuesse liegen 20 Zeilen unter dem
;                                              Sprite-Anfang; die erste Bildschirmzeile ist Rasterzeile 50)
;     y_vic = e_py + 36   beim Gegner  (Fuss 14 Zeilen unter dem Sprite-Anfang)
;   x kann ueber 255 liegen: das neunte Bit steht in $D010 (je ein Bit je Sprite).  Beide Sprites einer Figur bekommen
;   dieselbe Position; die Sprite-Zeiger (SCREEN+$3F8 ff.) waehlen das Bild (Bildnummer + Basis).
;   Die Sprite-Zeiger liegen am Ende des Bildschirmspeichers, also bei $4400 + $3F8 = $47F8, nicht bei $07F8 wie am C64.
;   Schaltet am Ende alle vier Sprites ein.  Aendert A, X und die Hilfszellen pt0..pt2.  Wird aus der Hauptschleife aufgerufen.
hal_update_sprites:
        ; Held: x + 24, y + 30 (Fuss-y 20 Zeilen unter dem Sprite-Anfang, Rahmen bei Zeile 50)
        clc
        lda h_px_l
        adc #24
        sta pt0
        lda h_px_h
        adc #0
        sta pt1                  ; Bit 8
        lda pt0
        sta $d000                       ; Sprite 0 (Held-Overlay) x
        sta $d002                       ; Sprite 1 (Held-Basis) x
        lda h_py
        clc
        adc #30
        sta $d001                       ; y der beiden Held-Sprites
        sta $d003
        ; Gegner: Fuss 14 Zeilen unter dem Sprite-Anfang
        clc
        lda e_px_l
        adc #24
        sta pt0
        lda e_px_h
        adc #0
        sta pt2
        lda pt0
        sta $d004                       ; Sprite 2 (Gegner-Overlay) x
        sta $d006                       ; Sprite 3 (Gegner-Basis) x
        lda e_py
        clc
        adc #36
        sta $d005                       ; y der beiden Gegner-Sprites
        sta $d007
        lda #0
        ldx pt1
        beq +
        ora #%00000011
+       ldx pt2
        beq +
        ora #%00001100
+       sta $d010                       ; Bit 8 der x-Koordinaten (Bits 0-1 Held, 2-3 Gegner)
        lda h_fr                        ; Bildnummer des Helden + Basis des Overlay-/Basis-Satzes = Sprite-Zeiger
        clc
        adc #HERO_OVER_PTR
        sta SCREEN+$3f8                 ; Zeiger Sprite 0
        lda h_fr
        clc
        adc #HERO_BASE_PTR
        sta SCREEN+$3f9                 ; Zeiger Sprite 1
        lda e_fr
        clc
        adc #ENEMY_OVER_PTR
        sta SCREEN+$3fa                 ; Zeiger Sprite 2
        lda e_fr
        clc
        adc #ENEMY_BASE_PTR
        sta SCREEN+$3fb                 ; Zeiger Sprite 3
        lda #%00001111
        sta $d015                       ; Sprites 0-3 einschalten
        rts

; ---------------------------------------------------------------- Klangeffekte (zweiter SID, Stimme 3: $D42E-$D434)
; Ein Effekt ist eine kurze Klangfolge, beschrieben durch 9 Byte:
;     0,1  Startfrequenz (lo, hi)  16 Bit: Frequenz in Hz = Wert * 985248 / 2^24 = Wert * 0,0587 (SID-Takt PAL, wie am C64).
;          Beispiel $0C00 = 3072 = 180 Hz
;     2,3  Frequenzaenderung je Bild (lo, hi), vorzeichenbehaftet: positiv = Ton steigt, negativ = faellt
;     4    Steuerbyte: Wellenform ($11 Dreieck, $21 Saege, $41 Rechteck, $81 Rauschen), Bit 0 = Gate (Ton an)
;     5    Dauer in Bildern                      6 AD = Anstieg/Abfall   7 SR = Haltepegel/Ausklang   8 Pulsbreite (hi)
; hal_sfx kopiert die Werte in die SID-Register und schaltet den Ton ein; sfx_update veraendert die Frequenz je Bild und
; schaltet den Ton nach Ablauf der Dauer ab (Gate aus).  sfx_on zeigt an, dass ein Effekt gerade laeuft.
; Registerlage: der zweite SID beginnt bei $D420, Stimme 3 liegt wie beim ersten SID 14 Byte (2 * 7) hinter dem Anfang:
;     $D42E/$D42F Frequenz   $D430/$D431 Pulsbreite   $D432 Steuerbyte (Gate)   $D433 AD   $D434 SR   ($D438 Lautstaerke)
; Am C64 liegen dieselben Register bei $D40E-$D414 im einzigen SID, dort muss die Musik waehrend des Effekts aussetzen.
; Effekt  0 Absprung (steigender Ton)   1 Landung (kurzes Rauschen)   2 Schritt (Klick)   3 Klettern (Blip)
;         4 Muenze (schneller Aufwaertsklang)   5 Gumbo klettert (Blubb)   6 Bildwechsel-Jingle   7 Fanfare
; Die Tabelle ist Byte fuer Byte dieselbe wie am C64; nur die Registeradressen der HAL-Routinen unterscheiden sich.
sfx_tab:
        .byte $00,$0c,$60,$00,$41,22,$08,$a4,$08     ; 0 Absprung: steigender Ton
        .byte $00,$14,$80,$ff,$81,7,$00,$a5,$08      ; 1 Landung: Rauschen
        .byte $00,$28,$00,$00,$81,2,$00,$90,$08      ; 2 Schritt: Klick
        .byte $00,$18,$00,$02,$11,4,$00,$a4,$08      ; 3 Klettern: Blip
        .byte $00,$40,$00,$03,$41,10,$00,$f0,$08     ; 4 Muenze: schneller Aufwaertsgleiter
        .byte $00,$0a,$b0,$ff,$11,12,$00,$a8,$08     ; 5 Gumbo: Blubb
        .byte $00,$20,$00,$01,$41,40,$08,$f4,$08     ; 6 Bildwechsel-Jingle
        .byte $00,$18,$00,$02,$41,60,$08,$f6,$08     ; 7 Fanfare
sfx_on:   .byte 0                       ; 1 = Effekt laeuft
sfx_cnt:  .byte 0                       ; verbleibende Bilder
sfx_fl:   .byte 0                       ; aktuelle Frequenz (lo)
sfx_fh:   .byte 0                       ; aktuelle Frequenz (hi)
sfx_dl:   .byte 0                       ; Aenderung je Bild (lo)
sfx_dh:   .byte 0                       ; Aenderung je Bild (hi)
sfx_ctrl: .byte 0                       ; Steuerbyte des Effekts (Wellenform + Gate)

; hal_sfx: A = Effektnummer 0..7.  Zeile in sfx_tab = Nummer * 9 (Nummer*8 + Nummer).  Das Gate wird erst aus- und dann wieder eingeschaltet,
; damit die Huellkurve jedes Mal von vorn beginnt.  Aendert A, X und pt0.  sfx_on wird zuerst 0, zuletzt wieder 1 (wie am C64).
hal_sfx:
        sta pt0
        asl a
        asl a
        asl a
        clc
        adc pt0
        tax                             ; X = Offset der Tabellenzeile
        lda #0
        sta $d432                ; Gate aus (Neustart)
        sta sfx_on               ; Effekt waehrend des Setzens aus (der Interrupt ruft sfx_update)
        lda sfx_tab,x
        sta sfx_fl
        sta $d42e                       ; Frequenz lo
        lda sfx_tab+1,x
        sta sfx_fh
        sta $d42f                       ; Frequenz hi
        lda sfx_tab+2,x
        sta sfx_dl
        lda sfx_tab+3,x
        sta sfx_dh
        lda sfx_tab+4,x
        sta sfx_ctrl
        lda sfx_tab+5,x
        sta sfx_cnt
        lda sfx_tab+6,x
        sta $d433                       ; AD
        lda sfx_tab+7,x
        sta $d434                       ; SR
        lda #0
        sta $d430                       ; Pulsbreite lo
        lda sfx_tab+8,x
        sta $d431                       ; Pulsbreite hi
        lda sfx_ctrl
        sta $d432                ; Gate an
        lda #1
        sta sfx_on
        rts

; sfx_update: einmal je Bild vom Interrupt aufgerufen.  Addiert die Frequenzaenderung, zaehlt die Dauer herunter und beendet den Effekt.
;   Aendert A (der Interrupt sichert die Register).
sfx_update:
        lda sfx_on
        beq +                           ; kein Effekt aktiv: nichts zu tun
        clc
        lda sfx_fl
        adc sfx_dl                      ; Frequenz += Aenderung je Bild (16 Bit)
        sta sfx_fl
        sta $d42e
        lda sfx_fh
        adc sfx_dh
        sta sfx_fh
        sta $d42f
        dec sfx_cnt
        bne +
        lda sfx_ctrl
        and #$fe                        ; Gate (Bit 0) loeschen: der Ton klingt mit dem Ausklang (Release) aus
        sta $d432
        lda #0
        sta sfx_on
+       rts
