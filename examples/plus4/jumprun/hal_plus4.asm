; =====================================================================================================================
; hal_plus4.asm - Hardware-Schicht (HAL) des Commodore Plus/4 fuer "Hugo & Gumbo"
; =====================================================================================================================
;
; Die Spiellogik (common/jumprun/core.asm) kennt keine Hardware. Diese Datei uebersetzt ihre Wuensche in Zugriffe auf den TED,
; den einzigen Chip des Plus/4 fuer Bild UND Klang.
;
; SCHNITTSTELLE (vollstaendig in core.asm beschrieben; hier, was diese HAL dazu liefert und braucht)
;   fuer den Kern:           hal_sfx (A = Effekt 0..7), hal_put_tile (X = Spalte, Y = Zeile, A = Kachel), hal_score_draw,
;                            sfx_on (Byte: ungleich 0, solange ein Effekt klingt)
;   fuer das Hauptprogramm:  hal_init, hal_wait_frame, hal_blank, hal_unblank, hal_title, hal_load_level (A = Bild),
;                            hal_update_sprites (liest h_px_l/h/h_py/h_fr und e_px_l/h/e_py/e_fr); ausserdem SCREEN, BGS, CH_BLANK,
;                            TED_MC1/TED_MC2 und die Tabelle skyc (von show_title direkt benutzt)
;   interne Hilfen:          hal_draw_level (alle 240 Kacheln aus LMAP zeichnen, benutzt get_tile des Kerns), hal_hud, put0/put23,
;                            put_bcd, set_ptrs, put_cell, restore, compose, char_addr, forget_figures, sfx_update, music_play, irq
;   Die HAL liest vom Kern: lvl, sc0..sc2 und die Ausgabewerte; aus lib/levels.inc: lvl_mc1/2, lvl_sky0..5, char_col, txt_hud,
;   txt_lv_lo/hi, CH_DIGIT0, CH_STAR, hero_lo/hi, enemy_lo/hi, sfx_tab, mus1_lo/hi, mus2_lo/hi; aus jumprun.asm: maps_lo/hi; aus dem
;   Export: title_chars, lv1_tiles.  Sie veraendert nur eigene Nullseitenzellen und darf in allen Routinen A, X, Y und die Flags
;   beliebig aendern.  Die Musik gehoert hier zur HAL (music_play), nicht zu einer eigenen Bibliothek.
;   Unterschiede zu den anderen Systemen (Sprites, Klang, Start): siehe core.asm, Abschnitt "Unterschiede".
;
;   BILD: Mehrfarb-Text 40 x 25 (Zeichen 4 x 8 breite Pixel; ein Pixel ist 2 Bildpunkte breit, sichtbar sind also 160 x 200 Pixel).
;       Jedes Pixel hat zwei Bit und waehlt damit eine von vier Farben:
;           00 = Hintergrund ($FF15)   01 = Farbe 1 ($FF16)   10 = Farbe 2 ($FF17)   11 = Zeichenfarbe (aus dem Farb-RAM)
;       Mehrfarbe gilt je Zelle nur, wenn im Farb-RAM-Byte Bit 3 gesetzt ist (sonst wird das Zeichen "hires" mit zwei Farben
;       gezeichnet).  Ein Farb-RAM-Byte ist beim TED selbst schon eine Farbe MIT Helligkeit:
;           Farbwert = Helligkeit * 16 + 8 + Farbton      (Helligkeit 0..7, Farbton 0..7 fuer die Zeichenfarbe, +8 = Mehrfarbe)
;       Die Register $FF15-$FF17 enthalten dagegen die volle Farbe (Helligkeit * 16 + Farbton 0..15) ohne das Mehrfarb-Bit.
;       Ein eigener Zeichensatz (256 Zeichen = 2 KB) liegt im RAM bei $4000 (kein ROM-Zeichensatz), der Bildschirm bei $0C00,
;       das Farb-RAM bei $0800 (je 1000 Byte; beim TED liegt das Farb-RAM fest $0400 vor dem Bildschirmspeicher).
;       Der Himmel besteht aus sechs Balken: ein Rasterinterrupt setzt die Hintergrundfarbe $FF15 jeweils um.
;
;   FIGUREN: Der TED hat KEINE Sprites.  Held und Gegner sind ZEICHEN-SPRITES:
;       Aus vorverschobenen Bildern (um 0..3 Mehrfarbpunkte nach rechts, vom Generator tools/gen_jumprun_plus4.py vorberechnet) werden
;       in jedem Bild, in dem sich eine Figur bewegt hat, Zeichen des Zeichensatzes ("Pool", Codes $E0-$F4) neu definiert:
;       Zeichenpunkt fuer Zeichenpunkt wird das Bild mit dem Hintergrund gemischt (Hintergrundzeichen der Zelle: Bildpunkte, die das
;       Bild belegt, ersetzen die Hintergrundpunkte; wo das Bild durchsichtig ist, bleibt der Hintergrund stehen), und die Zellen
;       werden auf dem Bildschirm auf die Pool-Zeichen gesetzt.  Der Hintergrund liegt dazu in einer Kopie ($9400, 960 Byte), denn der
;       Bildschirm selbst zeigt ja gerade die Pool-Zeichen und "vergisst" so, was darunter war.
;       Der Held belegt 3 x 4 Zellen (12 Pool-Zeichen $E0-$EB), der Gegner 3 x 3 (9 Pool-Zeichen $EC-$F4): zusammen 21 Zeichen.
;       Weil die Zeichen immer an ganzen Zellen haengen, wird die feine Lage so erreicht: waagerecht ueber das Vorverschieben des
;       Bildes (0..3 Punkte), senkrecht ueber den Zeilenversatz beim Mischen (0..7 Pixelzeilen).
;
;   KLANG: Der TED hat zwei Stimmen (Rechteck; Stimme 2 auch Rauschen) und eine gemeinsame Lautstaerke.  Stimme 1 = Melodie, Stimme 2 = Bass;
;       Klangeffekte leihen sich Stimme 2 (der Bass setzt waehrend eines Effekts aus).
;       Tonhoehe: ein 10-Bit-Wert N je Stimme ergibt die Frequenz  f = 110840,5 / (1024 - N)  Hz  (PAL-TED); je groesser N, desto hoeher
;       der Ton.  Der Generator rechnet die Notenfrequenzen mit genau dieser Formel in N um.
;
; SPEICHERPLAN: $1001 Programm  $4000 Zeichensatz  $4800 Tabellen und Figurenbilder  $6800 Bilddaten  $9000 Bildkarte  $9400 Hintergrundkopie
;               $0800 Farb-RAM  $0C00 Bildschirm  $FF00-$FF3F TED-Register (immer sichtbar, auch bei ausgeschaltetem ROM)
;               Der Plus/4 hat 64 KB RAM; nach dem Abschalten des ROMs ($FF3F) gehoert dieses ganze RAM dem Programm, auch $FFFA-$FFFF.
; NULLSEITE:    $80-$B3 Spiellogik, $C0-$E3 diese Datei (der Plus/4 braucht nach dem Abschalten des Betriebssystems keine Nullseite mehr).
;               Der Interrupt benutzt KEINE Nullseitenzelle: er unterbricht das Hauptprogramm an beliebiger Stelle mitten in dessen
;               Rechnung mit den Zeigern, und er darf nichts davon veraendern.
; =====================================================================================================================

.const SCREEN  = $0c00
.const COLORRAM= $0800
.const CHARSET = $4000
.const BGS     = $9400            ; Kopie des Hintergrunds (Bildschirmcodes), 40 x 24 = 960 Byte, Aufbau wie der Bildschirm
.const POOL_H  = $e0              ; Pool-Zeichen des Helden ($E0-$EB), des Gegners ($EC-$F4)
.const POOL_E  = $ec

; Zellfarben der Figuren.  Mehrfarbe = Bit 3 (+8) ist schon eingerechnet: Wert = Helligkeit * 16 + 8 + Farbton.
.const cell_skin = $6f           ; Zellfarben der Figuren (Mehrfarbe = Bit 3): gelb (Haut)  = Helligkeit 6, Farbton 7
.const cell_red  = $4a           ; rot (Hemd)  = Helligkeit 4, Farbton 2
.const cell_blue = $5e           ; blau (Hose) = Helligkeit 5, Farbton 6
.const cell_green= $4d           ; gruen (Gegner) = Helligkeit 4, Farbton 5
.const TOP     = 4                ; Rasterzeile der ersten Textzeile (siehe Himmelsbalken); die Balken sind je 32 Rasterzeilen = 4 Textzeilen hoch

; ---- TED-Register
.const TED_BG0   = $ff15          ; Hintergrundfarbe 0 (Paar 00)
.const TED_MC1   = $ff16          ; Farbe 1 (Paar 01)
.const TED_MC2   = $ff17          ; Farbe 2 (Paar 10)
.const TED_BORDER= $ff19          ; Rahmenfarbe
.const TED_CTL1  = $ff06          ; Bit 4 Bild an, Bit 3 25 Zeilen, Bits 0-2 vertikaler Versatz (3)
.const TED_CTL2  = $ff07          ; Bit 7 Umkehrung AUS (damit alle 256 Zeichen waehlbar sind), Bit 4 Mehrfarbe, Bit 3 40 Spalten
.const TED_CHAR  = $ff12          ; Bit 2: Zeichensatz ROM(1)/RAM(0); Bits 0-1: Tonhoehe Stimme 1 (hoch)
.const TED_CHBASE= $ff13          ; Bits 2-7: Adresse des Zeichensatzes (Bits 10-15); Bit 1: hier als "schneller Takt ausserhalb
                                  ; des Bildes" gesetzt (ted.chip.json nennt Bit 1 "ROM-Status", Bit 0 "Taktstatus": im Handbuch pruefen)
.const TED_SCRBASE=$ff14          ; Bits 3-7: Adresse (Bits 11-15) des Farb-RAMs; die Zeichen liegen $0400 dahinter ($08: Farben $0800, Zeichen $0C00)
.const TED_IRQ   = $ff09          ; Interrupt-Anforderungen (Schreiben von 1 loescht)
.const TED_IRQEN = $ff0a          ; Bit 1: Rasterinterrupt erlauben, Bit 0: Rasterzeile Bit 8
.const TED_RLO   = $ff0b          ; Rasterzeile, die den Interrupt ausloest (niedrige 8 Bit)
.const TED_RASTER= $ff1d          ; aktuelle Rasterzeile (niedrige 8 Bit)
.const TED_V1LO  = $ff0e          ; Stimme 1 Tonhoehe niedrig (hoch: $FF12 Bits 0-1)
.const TED_V2LO  = $ff0f          ; Stimme 2 Tonhoehe niedrig
.const TED_V2HI  = $ff10          ; Stimme 2 Tonhoehe hoch (Bits 0-1)
.const TED_VOL   = $ff11          ; Bits 0-3 Lautstaerke, Bit 4 Stimme 1 an, Bit 5 Stimme 2 an, Bit 6 Stimme 2 Rauschen
.const TED_ROMOFF= $ff3f          ; Schreiben: ROM abschalten (RAM sichtbar)  [Schreiben nach $FF3E schaltet das ROM wieder ein]

; ---- Nullseite dieser Datei
.const sp   = $c0                 ; Zeiger auf Bildschirmzelle (2 Byte)
.const bp   = $c2                 ; Zeiger auf die Hintergrundkopie
.const cp   = $c4                 ; Zeiger auf das Farb-RAM
.const dp   = $c6                 ; allgemeiner Zeiger (2 Byte)
.const pt0  = $c8                 ; Hilfsspeicher (kurzlebig, je Routine unterschiedlich benutzt)
.const pt1  = $c9
.const pt2  = $ca
.const pt3  = $cb
.const src  = $cc                 ; Zeiger auf das Figurenbild (2 Byte)
.const gp   = $ce                 ; Zeiger auf die Zeichendaten des Hintergrundzeichens (2 Byte)
.const op   = $d0                 ; Zeiger auf die Zeichendaten des Pool-Zeichens (2 Byte)
.const cp_col = $d2               ; Parameter der Zusammensetzung (compose/restore): linke Zelle, obere Zelle, Versatz 0..7 in der Zelle,
.const cp_row = $d3
.const cp_v   = $d4
.const cp_pool= $d5               ; ... erstes Pool-Zeichen, Zellzeilen, Bildzeilen
.const cp_nr  = $d6
.const cp_img = $d7
.const cp_c0  = $d8               ; Farben je Zellzeile (4 Byte)
.const cr     = $dc               ; laufende Zellzeile
.const cc     = $dd               ; laufende Zellspalte
.const ly     = $de               ; laufende Pixelzeile 0..7
.const kb     = $df               ; Bildzeile der Pixelzeile 0
.const anym   = $e0               ; ODER aller Masken der Zelle: 0 = die Zelle enthaelt keinen Bildpunkt der Figur
.const bgc    = $e1               ; Hintergrundzeichen der Zelle
.const fig    = $e2               ; Bildbyte der aktuellen Pixelzeile (4 Pixel)
.const msk    = $e3               ; Maske dazu

; ---------------------------------------------------------------- Start
; hal_init: TED einrichten. Danach laeuft das Programm ohne Betriebssystem: ROM aus, eigener Interrupt-Vektor.  Aendert A.
;   Die Interrupts bleiben gesperrt (sei); das Hauptprogramm gibt sie mit cli frei.
;   Wichtig ist die Reihenfolge: Zuerst mit sei jeden Interrupt sperren (das KERNAL-ROM-Programm des Plus/4 wuerde sonst mitten in
;   der Umschaltung dazwischenfunken), dann das ROM abschalten und erst danach die Vektoren im RAM setzen, denn bei sichtbarem
;   ROM waeren Schreibzugriffe auf $FFFA-$FFFF wirkungslos (sie landeten im RAM unter dem ROM, gelesen wuerde aber das ROM).
hal_init:
        sei
        sta TED_ROMOFF           ; RAM statt ROM: das ganze Betriebssystem ist ab jetzt weg
                                 ; (Das Schreiben genuegt, der geschriebene Wert A ist gleichgueltig.)
        lda #<irq
        sta $fffe
        lda #>irq
        sta $ffff                ; Interrupt-Vektor (im RAM): bei IRQ springt die CPU ueber $FFFE/$FFFF
        lda #<rti_only
        sta $fffa
        sta $fffc
        lda #>rti_only
        sta $fffb
        sta $fffd                ; NMI-Vektor ($FFFA) und Reset-Vektor ($FFFC) zeigen auf ein blosses rti: ein NMI tut nichts
        lda #0
        sta sfx_on               ; kein Klangeffekt aktiv
        sta sfx_n
        sta mus_tick             ; Musik beginnt bei Zeile 0 mit einem frischen Takt
        sta mus_row
        sta frame
        lda #$98                 ; = %10011000
        sta TED_CTL2             ; Bit 7: Umkehrung aus, Bit 4: Mehrfarbe an, Bit 3: 40 Spalten, Bits 0-2: waagerechter Versatz 0
        lda #$0b                 ; = %00001011
        sta TED_CTL1             ; Bild noch aus (Bit 4 = 0), 25 Zeilen (Bit 3), vertikaler Versatz 3 (Bits 0-2, Standardwert)
        lda #$08
        sta TED_SCRBASE          ; Bildschirm bei $0C00 (und damit das Farb-RAM bei $0800)
        lda TED_CHAR
        and #$fb                 ; Bit 2 loeschen: Zeichensatz aus dem RAM (die uebrigen Bits, v.a. die Tonhoehe von Stimme 1, bleiben)
        sta TED_CHAR             ; Zeichensatz aus dem RAM
        lda #$42                 ; = %01000010: Bits 2-7 = $10 -> $10 * 1024 = $4000;  Bit 1 = 1: schneller Takt ausserhalb des Bildes
        sta TED_CHBASE           ; Zeichensatz bei $4000, schneller Takt ausserhalb des Bildes
        lda #0
        sta TED_BORDER           ; Rahmen schwarz
        lda #$38                 ; = %00111000: Lautstaerke 8 (Bits 0-3), Stimme 1 (Bit 4) und Stimme 2 (Bit 5) an, kein Rauschen (Bit 6)
        sta TED_VOL              ; Lautstaerke 8, beide Stimmen an
        ; Himmelsinterrupt: erster Interrupt in Rasterzeile 0
        lda #0
        sta irq_idx              ; Nummer des naechsten Interrupts je Bild (0..5)
        sta TED_RLO              ; Rasterzeile 0 (niedrige 8 Bit) ...
        lda #$02
        sta TED_IRQEN            ; ... Rasterinterrupt erlauben (Bit 1); Bit 0 = 0: Rasterzeile hat nur 8 Bit, alle Zeilen < 256
        lda TED_IRQ
        sta TED_IRQ              ; moegliche alte Anforderungen loeschen (gelesene Bits zurueckschreiben)
        rts                      ; Die Interrupts bleiben gesperrt: das Hauptprogramm gibt sie mit cli frei.

rti_only: rti                    ; Ziel fuer NMI- und Reset-Vektor (siehe hal_init): tut nichts

; ---------------------------------------------------------------- Interrupt
; Sechs Interrupts je Bild.  Nummer 0 (Rasterzeile 0, oberhalb des Bildes): Himmelsfarbe 0, Musik, Effekte, Bildzaehler.
; Nummer 1..5: eine Zeile vor dem Beginn von Balken k wird der Interrupt ausgeloest; dann wartet die Routine auf den Beginn
; der Zeile (sonst wuerde die Farbe mitten in der Zeile wechseln) und setzt die Hintergrundfarbe.
;
; Warum so? Der TED hat nur EINE Hintergrundfarbe fuer Pixelpaar 00.  Ein Himmel mit Farbverlauf entsteht, indem $FF15 waehrend der
; Bilderzeugung mehrfach umgestellt wird: Was oberhalb der Umstellzeile gezeichnet wurde, behaelt die alte Farbe.
; Der Interrupt laeuft 50 mal je Sekunde sechsmal; die Zeilen stehen in irq_line (Balkenanfang) und irq_trig (eine Zeile davor).
; Warum eine Zeile davor?  Das Ausloesen des Interrupts und der Einsprung kosten einige Taktzyklen mit schwankender Dauer
; (je nachdem, welchen Befehl das Hauptprogramm gerade ausfuehrte).  Faende der Wechsel einfach "irgendwann nach der Zeile" statt,
; zoge sich der Farbwechsel um einen wechselnden Betrag nach rechts in die Zeile hinein.  Deshalb: frueh kommen, dann auf den
; exakten Zeilenanfang warten (Schleife .band).
;   Der Interrupt benutzt keine Nullseite und teilt keine Zellen mit der Hauptschleife (ausser frame, sfx_on, den Effekt- und Musikzellen),
;   darf sie also an beliebiger Stelle unterbrechen.  Nummer 0 liegt oberhalb des Bildes und hat Zeit fuer Musik und Effekt.
irq:    pha                      ; Register retten: bei abgeschaltetem ROM springt die CPU direkt hierher, kein KERNAL rettet sie
        txa
        pha
        tya
        pha
        lda TED_IRQ
        sta TED_IRQ              ; Interrupt quittieren (sonst loeste er sofort wieder aus)
        ldx irq_idx
        bne .band
        ; --- Nummer 0: ober- und ausserhalb des sichtbaren Bildes, hier ist Zeit fuer die Bildarbeit
        lda skyc
        sta TED_BG0              ; Himmelsfarbe des obersten Balkens
        inc frame                ; Bildzaehler: hal_wait_frame wartet auf seine Aenderung
        jsr music_play           ; Musik einen Schritt
        jsr sfx_update           ; laufenden Klangeffekt einen Schritt
        ldx #1
        jmp .next
.band:  lda irq_line,x           ; Zeile, in der der Balken beginnt
-       cmp TED_RASTER
        bcs -                    ; Schleife laeuft, solange irq_line >= Rasterzeile; sie endet also erst, wenn die Rasterzeile
                                 ; groesser als irq_line ist (die Farbe wechselt dann am Anfang der Zeile irq_line + 1)
        lda skyc,x               ; Farbe des Balkens x ...
        sta TED_BG0              ; ... ab jetzt gilt sie fuer die folgenden Zeilen
        inx
        cpx #6
        bcc .next
        ldx #0                   ; nach Balken 5 kommt wieder der Interrupt 0 im naechsten Bild
.next:  stx irq_idx
        lda irq_trig,x           ; Rasterzeile des naechsten Interrupts (eine vor dem Balkenbeginn)
        sta TED_RLO
        pla                      ; Register zurueckholen (umgekehrte Reihenfolge)
        tay
        pla
        tax
        pla
        rti
irq_idx: .byte 0                 ; Nummer des naechsten Interrupts (0 = Bildanfang, 1..5 = Balken)
frame:   .byte 0                 ; Bildzaehler, vom Interrupt 0 hochgezaehlt
irq_line: .byte 0, TOP+32, TOP+64, TOP+96, TOP+128, TOP+160     ; Rasterzeile, in der Balken k beginnt
irq_trig: .byte 0, TOP+31, TOP+63, TOP+95, TOP+127, TOP+159     ; Rasterzeile, in der der Interrupt dafuer ausgeloest wird (eine davor)
skyc:    .byte 0,0,0,0,0,0       ; Himmelsfarben der sechs Balken (TED-Farbwert fuer $FF15), von hal_load_level / .colors gesetzt

; hal_wait_frame: auf das naechste Bild warten: der Interrupt zaehlt frame hoch.  Aendert nur A.
;   Das Hauptprogramm ist dadurch an den Bildtakt gekoppelt (50 Bilder je Sekunde): je Bild genau ein Schritt der Spiellogik.
hal_wait_frame:
        lda frame
-       cmp frame
        beq -                    ; frame ist ein Byte, das der Interrupt im Hintergrund aendert
        rts

; hal_blank / hal_unblank: das Bild aus- und einschalten.  Beim Aus gibt der TED nur noch die Rahmenfarbe aus; so sieht man den
; Neuaufbau eines Bildes (Zellen schreiben, Zeichen umdefinieren) nicht.  Zuerst auf das Bildende warten, damit nicht mitten im
; Bild abgeschaltet wird.  Beide aendern nur A.  $0B bzw. $1B: Bit 3 = 25 Zeilen, Bits 0-2 = vertikaler Versatz 3, Bit 4 = Bild an.
hal_blank:
        jsr hal_wait_frame
        lda #$0b
        sta TED_CTL1             ; Bild aus
        rts
hal_unblank:
        lda #$1b
        sta TED_CTL1             ; Bild an (Bit 4)
        rts

; ---------------------------------------------------------------- Zellen
; Zeilenanfaenge: Bildschirm + 40 * Zeile.  Tabellen, weil der 6502 nicht multiplizieren kann.
row40_lo:
    .repeat 25, i
        .byte <(SCREEN + i * 40)
    .endrepeat
row40_hi:
    .repeat 25, i
        .byte >(SCREEN + i * 40)
    .endrepeat

; set_ptrs: X = Spalte, Y = Zeile -> sp (Bildschirm), bp (Hintergrundkopie), cp (Farb-RAM) zeigen auf die Zelle.  Aendert A, Flags.
;   Alle drei Bereiche haben denselben Aufbau (Zeile * 40 + Spalte); sie unterscheiden sich nur im oberen Adressbyte:
;   BGS - SCREEN = $9400 - $0C00 (Hintergrundkopie), COLORRAM - SCREEN = -$0400 (Farb-RAM).
set_ptrs:
        txa
        clc
        adc row40_lo,y
        sta sp                   ; niedriges Byte ist fuer alle drei Zeiger gleich
        sta bp
        sta cp
        lda row40_hi,y
        adc #0                   ; Uebertrag aus der Addition der Spalte
        sta sp+1
        clc
        adc #(>BGS) - (>SCREEN)  ; Hintergrundkopie: obere Adresse verschoben
        sta bp+1
        lda sp+1
        sec
        sbc #(>SCREEN) - (>COLORRAM)   ; Farb-RAM: obere Adresse verschoben (4 Seiten weiter unten)
        sta cp+1
        rts

; put_cell: Y = Versatz gegenueber (sp), A = Zeichencode: Bildschirm, Hintergrundkopie und Farbe der Zelle schreiben.  Aendert A, X.
;   Die Farbe kommt aus der Tabelle char_col (je Zeichencode ein Farb-RAM-Wert mit gesetztem Mehrfarb-Bit).
put_cell:
        sta (sp),y
        sta (bp),y
        tax
        lda char_col,x
        sta (cp),y
        rts

; hal_put_tile: X = Spalte (0..19), Y = Zeile (0..11), A = Kachelnummer: die vier Zeichen der Kachel (aus der exportierten Kachelliste) in die
;   2 x 2 Zellen schreiben.  Aendert alle Register, sp, bp, cp, pt0.  Die Kachelliste lv1_tiles gilt fuer alle vier Bilder.
;   Das Spiel denkt in Kacheln zu 2 x 2 Zeichen (20 x 12 Kacheln = 40 x 24 Zellen); die Kachelliste lv1_tiles enthaelt je Kachel
;   vier Zeichencodes (oben links, oben rechts, unten links, unten rechts).  Versatz 40 = naechste Zeile.
hal_put_tile:
        asl a
        asl a
        sta pt0                  ; Kachelnummer * 4 = Index in die Zeichenliste
        txa
        asl a
        tax                      ; Zellspalte = 2 * Spalte
        tya
        asl a
        tay                      ; Zellzeile = 2 * Zeile
        jsr set_ptrs
        ldx pt0
        lda lv1_tiles,x
        ldy #0
        jsr put_cell
        ldx pt0
        lda lv1_tiles+1,x
        ldy #1
        jsr put_cell
        ldx pt0
        lda lv1_tiles+2,x
        ldy #40
        jsr put_cell
        ldx pt0
        lda lv1_tiles+3,x
        ldy #41
        jmp put_cell

; hal_draw_level: alle 20 x 12 Kacheln aus der Bildkarte LMAP auf den Bildschirm zeichnen (get_tile liest sie aus der Spiellogik).
;   pt3 = Zeile, pt2 = Spalte.  Aendert alle Register, sp, bp, cp, pt0..pt3.
hal_draw_level:
        lda #0
        sta pt3                  ; Zeile
.row:   lda #0
        sta pt2                  ; Spalte
.col:   ldx pt2
        ldy pt3
        jsr get_tile
        ldx pt2
        ldy pt3
        jsr hal_put_tile
        inc pt2
        lda pt2
        cmp #20
        bcc .col
        inc pt3
        lda pt3
        cmp #12
        bcc .row
        rts

; hal_title: Titelbild, 960 Zeichencodes (24 Zeilen * 40), Zelle fuer Zelle mit Zellfarbe (aus char_col) auf Bildschirm, Hintergrundkopie
;   und Farb-RAM.  Aendert alle Register, sp, bp, cp, dp, pt1, pt2.
;   Zeile 24 (die 25.) bleibt unberuehrt; die Statuszeilen des Spiels liegen in Zeile 0 und 23.
hal_title:
        lda #<title_chars
        sta dp
        lda #>title_chars
        sta dp+1
        lda #0
        sta pt2                  ; Zeile
.trow:  ldx #0
        ldy pt2
        jsr set_ptrs
        ldy #0
-       sty pt1                  ; Spaltenzaehler Y merken (put_cell/char_col benutzen die Register frei)
        lda (dp),y
        jsr put_cell
        ldy pt1
        iny
        cpy #40
        bne -
        lda dp                   ; naechste Zeile der Quelle
        clc
        adc #40
        sta dp
        bcc +
        inc dp+1
+       inc pt2
        lda pt2
        cmp #24
        bcc .trow
        jmp forget_figures

; Die Figuren-Zeichen sind vom Bildschirm verschwunden (Bild neu gezeichnet): Zustand vergessen.
;   h_nr/e_nr = 0 heisst "keine Zellen zu restaurieren"; hk/ek = $FF im ersten Byte passt zu keiner echten x-Lage, also wird beim
;   naechsten hal_update_sprites sicher neu zusammengesetzt.
forget_figures:
        lda #0
        sta h_nr
        sta e_nr
        lda #$ff
        sta hk
        sta ek
        rts

; ---------------------------------------------------------------- Bild laden: A = Bildnummer
; hal_load_level: Farben (Farbe 1, Farbe 2, Himmelsbalken), Karte nach LMAP, Bild und Statuszeile zeichnen.  Aendert alle Register.
hal_load_level:
        tax
        lda lvl_mc1,x
        sta TED_MC1              ; Farbe 1 / Farbe 2 des Bildes (Paare 01 und 10 im Zeichensatz)
        lda lvl_mc2,x
        sta TED_MC2
        lda lvl_sky0,x           ; sechs Himmelsbalkenfarben (der Interrupt setzt sie nacheinander in $FF15)
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
        lda maps_lo,x            ; Kachelkarte des Bildes (240 Byte = 20 x 12) ...
        sta dp
        lda maps_hi,x
        sta dp+1
        ldy #0
-       lda (dp),y
        sta LMAP,y               ; ... nach LMAP kopieren: dort darf die Spiellogik sie veraendern (Muenzen werden eingesammelt)
        iny
        cpy #240
        bne -
        jsr hal_draw_level
        jsr forget_figures
        jmp hal_hud

; put0: A = Zeichen, X = Spalte in Zeile 0;  put23: dasselbe in Zeile 23 (Versatz 920). Bildschirm, Hintergrundkopie und Farbe.
;   (Zeile 23 beginnt bei 23 * 40 = 920.)  Y wird als Index fuer char_col benutzt.
put0:   sta SCREEN,x
        sta BGS,x
        tay
        lda char_col,y
        sta COLORRAM,x
        rts
put23:  sta SCREEN+920,x
        sta BGS+920,x
        tay
        lda char_col,y
        sta COLORRAM+920,x
        rts

; hal_hud: Statuszeile (Zeile 0) und Laufschrift (Zeile 23, die letzte; sie ist die untere Zeichenzeile der Kachelzeile 11 und
;   ueberschreibt dort die Kacheln).  Schreibt in Bildschirm, Hintergrundkopie und Farb-RAM.  Aendert alle Register, pt0, pt1, dp.
hal_hud:
        ldx #0
-       lda txt_hud,x            ; "HUGO 000000   BILD 1/4   LEBEN": 30 Zeichen
        sta pt0
        inx
        lda pt0
        jsr put0                 ; Spalte X = Textindex + 1
        cpx #30
        bne -
        lda lvl
        clc
        adc #CH_DIGIT0+1         ; Bildnummer = lvl + 1 als Ziffernzeichen
        ldx #20
        jsr put0                 ; Bildnummer
        lda #CH_STAR
        ldx #31
        jsr put0                 ; drei Sterne = drei Leben, Spalten 31..33
        inx
        jsr put0
        inx
        jsr put0
        ldx lvl
        lda txt_lv_lo,x          ; Name des Bildes (25 Zeichen)
        sta dp
        lda txt_lv_hi,x
        sta dp+1
        ldy #0
-       lda (dp),y
        sta pt0
        tya
        clc
        adc #7                   ; Spalte 7 ... 31
        tax
        lda pt0
        sty pt1
        jsr put23
        ldy pt1
        iny
        cpy #25
        bne -
        jmp hal_score_draw

; hal_score_draw: Punktestand (sechs BCD-Ziffern in sc2:sc1:sc0) in Zeile 0 ab Spalte 6.  Aendert A, X, Y.
hal_score_draw:
        lda sc2
        ldx #6
        jsr put_bcd
        lda sc1
        ldx #8
        jsr put_bcd
        lda sc0
        ldx #10
put_bcd:                         ; A = BCD, X = Spalte der ersten Ziffer (6, 8, 10); aendert A, X, Y
        pha
        lsr a
        lsr a
        lsr a
        lsr a                    ; obere Ziffer
        clc
        adc #CH_DIGIT0
        jsr put0
        inx
        pla
        and #$0f                 ; untere Ziffer
        clc
        adc #CH_DIGIT0
        jmp put0

; ---------------------------------------------------------------- Zeichen-Sprites
; Zustand je Figur: hk/ek = letzter Stand (x niedrig, x hoch, y, Bild) -> nur bei einer Aenderung neu zusammensetzen;
; h_ocol/h_orow/h_nr = Zellen, die die Figur zuletzt belegt hat (h_nr = 0: keine).
;   Das Zusammensetzen kostet viel Rechenzeit (21 Pool-Zeichen mal 8 Pixelzeilen); wenn beide Figuren stehen, ist es nicht noetig.
hk:      .byte $ff, 0, 0, 0
ek:      .byte $ff, 0, 0, 0
h_ocol:  .byte 0
h_orow:  .byte 0
h_nr:    .byte 0                 ; Zahl der belegten Zellzeilen (0 = nichts zu restaurieren)
e_ocol:  .byte 0
e_orow:  .byte 0
e_nr:    .byte 0
hero_cols: .byte cell_skin, cell_red, cell_blue, cell_blue      ; Zellfarbe je Zellzeile des Helden (Kopf, Hemd, Hose, Hose)

; hal_update_sprites: wird je Bild aufgerufen; setzt Held und Gegner auf den vom Spiel verlangten Stand (h_px_l/h_px_h/h_py/h_fr
; und e_*, von core_prepare_sprites bereitgestellt).  Schritte:
;   1. nichts geaendert?  -> sofort zurueck
;   2. die alten Zellen beider Figuren aus der Hintergrundkopie wiederherstellen (restore)
;   3. Gegner, danach Held neu mischen (compose); der Held kommt zuletzt, damit er VOR dem Gegner liegt: beide benutzen verschiedene
;      Pool-Zeichen, aber wo sich die Zellen ueberlappen, ueberschreibt die spaeter geschriebene Zelle auf dem Bildschirm die frueher
;      geschriebene.  Den Hintergrund liest compose immer aus der Kopie BGS, nie vom Bildschirm, denn dort stehen ja Pool-Zeichen.
;   Aendert alle Register, die Hilfszellen und die Parameter cp_*.  Laeuft in der Hauptschleife, nicht im Interrupt.  Figuren, deren
;   Oberkante ueber dem Bildrand liegen wuerde (Fuss-y kleiner als die Bildhoehe der Figur: 13 bzw. 21), werden nicht gezeichnet.
hal_update_sprites:
        lda h_px_l               ; hat sich etwas geaendert?
        cmp hk
        bne .chg
        lda h_px_h
        cmp hk+1
        bne .chg
        lda h_py
        cmp hk+2
        bne .chg
        lda h_fr
        cmp hk+3
        bne .chg
        lda e_px_l
        cmp ek
        bne .chg
        lda e_px_h
        cmp ek+1
        bne .chg
        lda e_py
        cmp ek+2
        bne .chg
        lda e_fr
        cmp ek+3
        bne .chg
        rts
.chg:   lda h_px_l               ; neuen Stand merken
        sta hk
        lda h_px_h
        sta hk+1
        lda h_py
        sta hk+2
        lda h_fr
        sta hk+3
        lda e_px_l
        sta ek
        lda e_px_h
        sta ek+1
        lda e_py
        sta ek+2
        lda e_fr
        sta ek+3
        ; 1. alte Zellen auf den Hintergrund zuruecksetzen
        lda e_nr
        beq +
        sta cp_nr
        lda e_ocol
        sta cp_col
        lda e_orow
        sta cp_row
        jsr restore
+       lda h_nr
        beq +
        sta cp_nr
        lda h_ocol
        sta cp_col
        lda h_orow
        sta cp_row
        jsr restore
        ; 2. Gegner neu zusammensetzen (Fuss-y - 13 = obere Zeile des Bildes)
        ;    Die y-Koordinate der Spiellogik ist die Fusszeile der Figur; das 13 Zeilen hohe Gegnerbild beginnt 13 Zeilen weiter oben.
+       lda #0
        sta e_nr
        lda e_py
        sec
        sbc #13
        bcc .hero                ; ueber dem Bildrand: nicht zeichnen
        pha
        lsr a
        lsr a
        lsr a
        sta cp_row               ; obere Zellzeile = y / 8
        pla
        and #7
        sta cp_v                 ; Versatz innerhalb der Zelle = y mod 8 (Pixelzeilen nach unten)
        lda e_px_h
        lsr a
        lda e_px_l
        ror a                    ; x / 2 = Mehrfarbpunkte (16-Bit-Wert um ein Bit nach rechts: x ist in Bildpunkten, ein Pixel = 2)
        clc
        adc #2                   ; Spalte 2 des 12 Punkte breiten Quellbildes = linker Rand des Bildes
        jsr cell_shift           ; cp_col = Zelle, A = Verschiebung 0..3
        sta pt0
        lda e_fr
        asl a
        asl a
        ora pt0                  ; Bildnummer * 4 + Verschiebung = Index in die Tabelle der vorverschobenen Bilder
        tax
        lda enemy_lo,x
        sta src
        lda enemy_hi,x
        sta src+1
        lda #POOL_E
        sta cp_pool
        lda #3
        sta cp_nr                ; 3 Zellzeilen
        sta e_nr
        lda #13
        sta cp_img               ; 13 Bildzeilen
        lda #cell_green
        sta cp_c0                ; alle Zellzeilen des Gegners gruen
        sta cp_c0+1
        sta cp_c0+2
        sta cp_c0+3
        lda cp_col
        sta e_ocol               ; Belegung merken, damit sie beim naechsten Mal restauriert werden kann
        lda cp_row
        sta e_orow
        jsr compose
        ; 3. Held (liegt vor dem Gegner)
.hero:  lda #0
        sta h_nr
        lda h_py
        sec
        sbc #21                  ; das Heldenbild ist 21 Zeilen hoch
        bcc .done
        pha
        lsr a
        lsr a
        lsr a
        sta cp_row
        pla
        and #7
        sta cp_v
        lda h_px_h
        lsr a
        lda h_px_l
        ror a
        clc
        adc #2
        jsr cell_shift
        sta pt0
        lda h_fr
        asl a
        asl a
        ora pt0
        tax
        lda hero_lo,x
        sta src
        lda hero_hi,x
        sta src+1
        lda #POOL_H
        sta cp_pool
        lda #4
        sta cp_nr                ; 4 Zellzeilen (21 Zeilen + bis zu 7 Zeilen Versatz = bis zu 28 Zeilen)
        sta h_nr
        lda #21
        sta cp_img
        ldx #3
-       lda hero_cols,x          ; Zellfarben je Zellzeile: Haut, Hemd, Hose, Hose
        sta cp_c0,x
        dex
        bpl -
        lda cp_col
        sta h_ocol
        lda cp_row
        sta h_orow
        jsr compose
.done:  rts

; cell_shift: A = Mehrfarbpunkt (x) -> cp_col = A / 4, A = Rest 0..3
;   Eine Zelle ist 4 Mehrfarbpunkte breit.  Die ganzen Zellen bestimmen, wo die 3 Pool-Spalten stehen; der Rest (0..3) waehlt das
;   um so viele Punkte nach rechts vorverschobene Bild.
cell_shift:
        pha
        lsr a
        lsr a
        sta cp_col
        pla
        and #3
        rts

; restore: Zellen (cp_col, cp_row, 3 Spalten, cp_nr Zeilen) auf den Hintergrund zuruecksetzen (Zeichen und Farbe)
;   Zellen ausserhalb des Bildschirms (Zeile >= 24, Spalte >= 40) werden uebersprungen: Figuren duerfen am Rand halb verschwinden.
restore:
        lda #0
        sta cr
.rr:    lda cp_row
        clc
        adc cr
        cmp #24
        bcs .rnext               ; unterhalb der letzten Zeile
        sta pt0
        lda #0
        sta cc
.rc:    lda cp_col
        clc
        adc cc
        cmp #40
        bcs .rcn                 ; rechts neben dem Bildschirm
        tax
        ldy pt0
        jsr set_ptrs
        ldy #0
        lda (bp),y               ; Hintergrundzeichen aus der Kopie ...
        sta (sp),y               ; ... zurueck auf den Bildschirm
        tax
        lda char_col,x           ; und seine Farbe zurueck ins Farb-RAM
        sta (cp),y
.rcn:   inc cc
        lda cc
        cmp #3
        bcc .rc
.rnext: inc cr
        lda cr
        cmp cp_nr
        bcc .rr
        rts

; compose: das Bild (Zeiger src, 3 Byte je Zeile, cp_img Zeilen) an die Zellen (cp_col, cp_row), um cp_v Pixelzeilen nach unten versetzt,
; in cp_nr Zellzeilen mit dem Hintergrund mischen und in die Pool-Zeichen ab cp_pool schreiben.
;   Mischen je Pixelzeile:  Maske = alle Paare des Bildes, die nicht 00 sind;  neu = (Hintergrund UND NICHT Maske) ODER Bild.
;   Die Zellfarbe wird auf die Figurenfarbe der Zellzeile gesetzt, wenn die Zelle ueberhaupt Bildpunkte enthaelt.
;
; Ein Beispiel fuer eine Pixelzeile (vier Paare je Byte, links das hoechste Paar):
;     Hintergrund   01 10 11 00        Bild  00 11 00 01
;     Maske         00 11 00 11        (Paar belegt -> 11, durchsichtig -> 00)
;     Hintergrund AND NOT Maske = 01 00 11 00;  OR Bild = 01 11 11 01
;   Beachte: Wo das Bild 11 hat, entsteht Zeichenfarbe, also die Zellfarbe -- und die ist fuer die ganze Zelle gleich.  Daher haengt
;   die Farbe an der ZELLE, nicht am Pixel; der Generator legt die Figurenfarben so auf Zellzeilen (Kopf/Hemd/Hose), dass das passt.
;   Der Umriss der Figur (Farbe 1 = 01) bleibt dagegen pro Pixel frei waehlbar und ist bei allen Zellen gleich.
;
; Die Pool-Zeichen: Zelle (Spalte cc, Zeile cr) der Figur -> Zeichen cp_pool + 3 * cr + cc.  Jede Figurenzelle hat also IHR eigenes
; Pool-Zeichen, dessen 8 Datenbytes (8 Pixelzeilen) hier neu geschrieben werden; auf dem Bildschirm steht in der Zelle dieser Code.
compose:
        lda #0
        sta cr
.cr:    lda cr
        asl a
        asl a
        asl a
        sec
        sbc cp_v
        sta kb                   ; Bildzeile der Pixelzeile 0 dieser Zellzeile (negativ = 8-Bit-Zahl ueber 127)
        lda #0
        sta cc
.cc:    lda cp_row
        clc
        adc cr
        cmp #24
        bcc +
        jmp .cnext               ; Zelle unterhalb des Bildschirms: auslassen
+       sta pt0                  ; Zellzeile auf dem Bildschirm
        lda cp_col
        clc
        adc cc
        cmp #40
        bcc +
        jmp .cnext               ; Zelle rechts ausserhalb: auslassen
+       tax
        ldy pt0
        jsr set_ptrs
        lda cr                   ; Pool-Zeichen = cp_pool + 3 * Zellzeile + Spalte
        asl a
        clc
        adc cr
        clc
        adc cc
        clc
        adc cp_pool
        sta pt1
        ldy #0
        lda (bp),y
        sta bgc                  ; Hintergrundzeichen der Zelle (aus der Kopie, nicht vom Bildschirm!)
        lda pt1
        sta (sp),y               ; Zelle zeigt jetzt das Pool-Zeichen
        lda bgc                  ; gp = Zeichendaten des Hintergrundzeichens
        jsr char_addr
        lda dp
        sta gp
        lda dp+1
        sta gp+1
        lda pt1                  ; op = Zeichendaten des Pool-Zeichens
        jsr char_addr
        lda dp
        sta op
        lda dp+1
        sta op+1
        lda #0
        sta anym                 ; bisher kein Bildpunkt der Figur in dieser Zelle
        ldy #0                   ; Y = Pixelzeile 0..7 der Zelle
.py:    tya
        clc
        adc kb                   ; Bildzeile k dieser Pixelzeile
        cmp cp_img
        bcs .tr                  ; ausserhalb des Bildes: durchsichtig (auch "negative" k, die als Zahl ueber 127 erscheinen)
        sta pt2
        asl a
        clc
        adc pt2                  ; 3 * k
        clc
        adc cc                   ; + Spalte
        sty ly
        tay
        lda (src),y              ; das Bildbyte (vier Paare) dieser Zeile und Spalte
        ldy ly
        jmp .have
.tr:    lda #0
.have:  sta fig
        lsr a
        ora fig
        and #$55                 ; je Paar steht im unteren Bit "Paar ist nicht 00" (oberes ODER unteres Bit des Paares)
        sta msk
        asl a
        ora msk
        sta msk                  ; Maske: 11 fuer jedes Paar, das das Bild belegt
        ora anym
        sta anym                 ; merken, ob die Zelle ueberhaupt etwas von der Figur enthaelt
        lda msk
        eor #$ff                 ; NICHT Maske
        and (gp),y               ; Hintergrund ohne die vom Bild belegten Paare
        ora fig                  ; Bild hineinsetzen
        sta (op),y               ; in das Pool-Zeichen schreiben
        iny
        cpy #8
        bne .py
        lda anym                 ; Zellfarbe
        beq .bgcol               ; kein Bildpunkt: Zelle behaelt die Farbe des Hintergrundzeichens
        ldx cr
        lda cp_c0,x              ; Figurenfarbe dieser Zellzeile
        jmp .setc
.bgcol: ldx bgc
        lda char_col,x
.setc:  ldy #0
        sta (cp),y
.cnext: inc cc
        lda cc
        cmp #3
        bcs +
        jmp .cc                  ; naechste Zelle (Spalte) in derselben Zellzeile
+       inc cr
        lda cr
        cmp cp_nr
        bcs +
        jmp .cr                  ; naechste Zellzeile
+       rts

; char_addr: A = Zeichencode -> dp = CHARSET + 8 * Code
;   Jedes Zeichen belegt 8 Byte (eine je Pixelzeile); 8 * Code ist ein 16-Bit-Ergebnis, daher dreimal um ein Bit nach links.
char_addr:
        sta dp
        lda #0
        sta dp+1
        asl dp
        rol dp+1
        asl dp
        rol dp+1
        asl dp
        rol dp+1
        lda dp+1
        clc
        adc #>CHARSET
        sta dp+1
        rts

; ---------------------------------------------------------------- Klangeffekte (TED-Stimme 2)
; Eintrag je 6 Byte (sfx_tab in lib/levels.inc): Tonhoehe niedrig/hoch, Aenderung je Bild niedrig/hoch, Rauschen, Dauer in Bildern.
;   Ein Effekt ist ein Ton, dessen Wert N (10 Bit, siehe oben f = 110840,5 / (1024 - N)) sich in jedem Bild um einen festen Betrag
;   aendert (Glissando: Sprung, Fall, Piepton); Rauschen ersetzt den Rechteckton der Stimme 2 durch Rauschen (Bit 6 in $FF11).
sfx_on:  .byte 0                 ; 0 = kein Effekt aktiv (dann spielt der Bass auf Stimme 2)
sfx_lo:  .byte 0                 ; aktuelles N
sfx_hi:  .byte 0
sfx_dl:  .byte 0                 ; Aenderung je Bild (16 Bit mit Vorzeichen, hier getrennt)
sfx_dh:  .byte 0
sfx_nz:  .byte 0                 ; 1 = Rauschen
sfx_n:   .byte 0                 ; verbleibende Bilder

; hal_sfx: A = Effektnummer 0..7: Effekt starten.  Aendert A, X, pt0, pt1.  Effekte: 0 Absprung, 1 Landung, 2 Schritt, 3 Klettern, 4 Muenze,
;   5 Gumbo klettert, 6 Bildwechsel-Jingle, 7 Fanfare.  Der Effekt beginnt mit dem naechsten Interrupt Nr. 0 (sfx_update).
hal_sfx:
        sta pt0
        asl a                    ; Nummer * 6
        sta pt1
        asl a
        clc
        adc pt1
        tax
        lda #0
        sta sfx_on               ; waehrend des Setzens aus (der Interrupt schaut hinein)
        lda sfx_tab,x
        sta sfx_lo
        lda sfx_tab+1,x
        sta sfx_hi
        lda sfx_tab+2,x
        sta sfx_dl
        lda sfx_tab+3,x
        sta sfx_dh
        lda sfx_tab+4,x
        sta sfx_nz
        lda sfx_tab+5,x
        sta sfx_n
        lda #1
        sta sfx_on
        rts

; sfx_update: vom Interrupt Nr. 0 je Bild aufgerufen (Nullseite wird nicht benutzt).  Aendert A, X (der Interrupt sichert die Register).
sfx_update:
        lda sfx_on
        beq .rts
        lda sfx_lo
        sta TED_V2LO             ; aktuelle Tonhoehe auf Stimme 2
        lda sfx_hi
        sta TED_V2HI
        lda #$38                 ; Lautstaerke 8, beide Stimmen an
        ldx sfx_nz
        beq +
        lda #$78                 ; Bit 6: Stimme 2 als Rauschen
+       sta TED_VOL
        clc
        lda sfx_lo               ; N := N + Aenderung (16 Bit; negative Aenderung = Zweierkomplement, der Uebertrag faellt weg)
        adc sfx_dl
        sta sfx_lo
        lda sfx_hi
        adc sfx_dh
        and #3                   ; nur 2 Bit hoch: N ist ein 10-Bit-Wert
        sta sfx_hi
        dec sfx_n
        bne .rts
        lda #0
        sta sfx_on               ; Effekt zu Ende: Bass wieder einschalten
        lda mus_n2l              ; zuletzt gespielte Basstonhoehe wiederherstellen ...
        sta TED_V2LO
        lda mus_n2h
        sta TED_V2HI
        lda #$38                 ; ... und das Rauschen abschalten (Bit 6 = 0)
        sta TED_VOL
.rts:   rts

; ---------------------------------------------------------------- Musik (zwei Stimmen, Tabellen aus lib/levels.inc)
; Eine Zeile dauert 6 Bilder; 128 Zeilen, dann von vorn. Eintrag mit hohem Byte $FF = keine neue Note (die alte klingt weiter).
;   Die Tabellen mus1_lo/mus1_hi (Melodie) und mus2_lo/mus2_hi (Bass) haben je 128 Eintraege (N niedrig bzw. hoch).  Der Generator
;   schreibt fuer "keine neue Note" $FFFF, also hoch = $FF; ein echtes N hat hoch 0..3, deshalb genuegt der Vorzeichentest (bmi).
mus_tick: .byte 0                ; Bilder seit dem letzten Zeilenschritt (0..5)
mus_row:  .byte 0                ; aktuelle Zeile 0..127
mus_n2l:  .byte 0                ; zuletzt gespielte Basstonhoehe (wird nach einem Effekt wieder eingestellt)
mus_n2h:  .byte 0
; music_play: vom Interrupt Nr. 0 je Bild aufgerufen; schaltet alle 6 Bilder eine Musikzeile weiter.  Aendert A, X.
music_play:
        inc mus_tick
        lda mus_tick
        cmp #6
        bcc .mrts                ; noch nicht Zeit fuer den naechsten Schritt
        lda #0
        sta mus_tick
        ldx mus_row
        lda mus1_hi,x
        bmi .v2                  ; Stimme 1: keine neue Note
        lda mus1_lo,x
        sta TED_V1LO
        lda TED_CHAR             ; die hohen Bits der Stimme 1 liegen in $FF12 Bits 0-1, daneben stehen dort Zeichensatzbits: erhalten
        and #$fc
        ora mus1_hi,x
        sta TED_CHAR
.v2:    lda mus2_hi,x
        bmi .adv                 ; Stimme 2 (Bass): keine neue Note
        sta mus_n2h              ; neue Note merken, damit ein Effekt sie nachher wiederherstellen kann
        lda mus2_lo,x
        sta mus_n2l
        lda sfx_on
        bne .adv                 ; laeuft ein Effekt, gehoert Stimme 2 ihm: Bass nur merken, nicht spielen
        lda mus_n2l
        sta TED_V2LO
        lda mus_n2h
        sta TED_V2HI
.adv:   inx
        cpx #128
        bcc +
        ldx #0                   ; nach der letzten Zeile von vorn
+       stx mus_row
.mrts:  rts
