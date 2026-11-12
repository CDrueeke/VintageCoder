; =====================================================================================================================
; hal_atari.asm - Hardware-Schicht (HAL) des Atari 8 Bit fuer "Hugo & Gumbo"
; =====================================================================================================================
;
; Die Spiellogik (common/jumprun/core.asm) kennt keine Hardware. Diese Datei uebersetzt ihre Wuensche in Zugriffe auf
; die Atari-Chips.  Die Logik rechnet in einem logischen Raster von 320 x 200 Punkten; die HAL macht daraus Farbtakte,
; Zeichenzellen und Spielerzeilen (1 Farbtakt = 2 logische Punkte, 1 Zeile = 1 logischer Punkt in der Hoehe).
;
; SCHNITTSTELLE (vollstaendig in core.asm beschrieben; hier, was diese HAL dazu liefert und braucht)
;   fuer den Kern:           hal_sfx (A = Effekt 0..7), hal_put_tile (X = Spalte, Y = Zeile, A = Kachel), hal_score_draw,
;                            sfx_on (Byte: ungleich 0, solange ein Effekt klingt)
;   fuer das Hauptprogramm:  hal_init, hal_wait_frame, hal_blank, hal_unblank, hal_title, hal_load_level (A = Bild),
;                            hal_update_sprites (liest h_px_l/h/h_py/h_fr und e_px_l/h/e_py/e_fr); ausserdem SCREEN, CH_BLANK, die
;                            Schattenregister COLOR0.. und die Tabelle skyc (von show_title direkt benutzt)
;   interne Hilfen:          hal_draw_level (alle 240 Kacheln aus LMAP zeichnen, benutzt get_tile des Kerns), hal_hud, put_bcd,
;                            clear_pm, sfx_update, dli, vbi
;   Die HAL liest vom Kern: lvl, sc0..sc2 und die Ausgabewerte; aus lib/levels.inc: lvl_pf0..2, lvl_sky0..5, tile_hi, txt_hud,
;   txt_lv_lo/hi, CH_DIGIT0, CH_STAR, die Spielerbilder hero_p0..p3 und enemy_m; aus jumprun.asm: maps_lo/hi; aus dem Export: title_chars,
;   lv1_tiles.  Sie veraendert nur eigene Nullseitenzellen und darf in allen Routinen A, X, Y und die Flags beliebig aendern.
;   Unterschiede zu den anderen Systemen (Sprites, Klang, Start): siehe core.asm, Abschnitt "Unterschiede".
;
;   ANTIC (Bild)
;     * Text-MEHRFARBMODUS 4 (ANTIC-Modus 4, in BASIC GRAPHICS 12), 40 x 24 Zeichen.  Ein Zeichen ist 4 x 8 breite Pixel gross
;       (ein Pixel = ein Farbtakt = 2 Hires-Bildpunkte, deshalb 160 Farbtakte Breite).  Jedes Pixel hat zwei Bit:
;           00 = Hintergrund COLBK       01 = COLPF0       10 = COLPF1       11 = COLPF2  (Zeichenbit 7 gesetzt: COLPF3 statt COLPF2)
;       Zeichencodes laufen deshalb nur von 0 bis 127 (7 Bit); Bit 7 ist keine "inverse" Anzeige, sondern die Farbwahl.
;       Die Farbe gilt fuer den GANZEN Bildschirm (anders als beim C64 gibt es kein Farb-RAM je Zeichen). Je Bild werden
;       COLPF0..2 neu gesetzt; Gras und Busch tragen Bit 7 und erscheinen deshalb in COLPF3 (gruen, in allen Bildern gleich).
;     * Der Zeichensatz (128 Zeichen = 1 KB) liegt bei $4400 (er muss auf einer 1-KB-Grenze beginnen), der Bildschirmspeicher
;       (24 * 40 = 960 Byte) bei $4000, die Display-Liste bei $4800.  Der Bildschirmspeicher darf keine 4-KB-Grenze ueberschreiten
;       (ANTIC zaehlt nur 12 Adressbit weiter); $4000-$43BF liegt komplett in einem 4-KB-Block.
;     * DISPLAY-LISTE: Programm fuer ANTIC: 24 Leerzeilen, 24 Textzeilen im Modus 4 (die erste mit Adresse LMS), am Ende ein Sprung
;       zurueck an den Anfang.  Bei fuenf Zeilen (3, 7, 11, 15, 19) ist das DLI-Bit gesetzt: dort loest ANTIC einen
;       DISPLAY-LIST-INTERRUPT aus, in dem wir die Hintergrundfarbe wechseln.  So entsteht der Himmel aus sechs Balken
;       (Zeilen 0-3, 4-7, 8-11, 12-15, 16-19, 20-23; jeder Balken ist vier Textzeilen = 32 Rasterzeilen hoch).
;
;   GTIA + ANTIC (Figuren): PLAYER-MISSILE-GRAFIK, Einzelzeilenaufloesung
;     * Spieler (Player) sind 8 Farbtakte breite Streifen, die ueber die ganze Bildhoehe reichen; die Bits eines Bytes aus dem
;       Spielerspeicher sind die Pixel EINER Rasterzeile (Bit 7 links).  Missiles sind vier 2 Bit breite Streifen, die sich ein
;       gemeinsames Byte je Zeile teilen.  ANTIC holt die Bytes per DMA, GTIA zeichnet sie, HPOSPn/HPOSMn setzen die x-Lage.
;     * Der HELD besteht aus VIER Spielern uebereinander (je 8 Farbtakte breit, ein Farbtakt = ein Mehrfarbpunkt des Textes):
;           P0 = Umriss und Auge (schwarz)   P1 = Haut   P2 = Hemd   P3 = Hose.
;       Spieler 0 liegt vor den anderen (Prioritaet), deshalb zeichnet er den Umriss.  Die vier Streifen haben dieselbe x-Lage und
;       ihre Bits ueberdecken sich nie: jedes Pixel gehoert genau einem Spieler, so ergibt sich eine vierfarbige Figur.  Ein
;       feinerer "Hires-Overlay" ist auf dem Atari nicht moeglich: der kleinste Pixel eines Spielers ist ein Farbtakt.
;     * Der GEGNER besteht aus den vier MISSILES (je 2 Takte breit), nebeneinander als "5. Spieler" in der Farbe COLPF3 (PRIOR Bit 4):
;       so haben sie die gruene Farbe des Grases und bilden zusammen 8 Takte Breite, die Breite eines Spielers.
;     * Spielerspeicher: ein Byte = eine Bildschirmzeile.  Seite $50 + 0..7: Missiles bei $5300, P0 $5400, P1 $5500, P2 $5600, P3 $5700.
;       (Bei Einzelzeilenaufloesung ist ein Spielerblock 256 Byte gross und der Speicher 2 KB lang; PMBASE zeigt auf $5000, davon
;       sind die ersten $300 Byte ungenutzt.)
;     * Die erste sichtbare Bildzeile hat in diesem Speicher den Index 32 (8 Zeilen Rand der Normalbreite + 24 Leerzeilen der
;       Display-Liste).  Alles davor und dahinter liegt ausserhalb des Textbildes und bleibt leer.
;
;   POKEY (Klang)
;     * Musik im RMT-Format (Original-Player von Raster, lib/rmtplayr.asm, schreibt in jedem Bild alle vier POKEY-Kanaele).  Das Lied
;       (music/tune.vcsong) benutzt nur die Spuren 1-3; Kanal 4 (AUDF4/AUDC4) bleibt fuer die Klangeffekte frei: nach dem Abspielen der
;       Musik ueberschreibt der Effekt-Teil dieser Datei den Kanal 4 (siehe vbi).  Zur Lage der Daten siehe "Klangeffekte" unten.
;
;   BETRIEBSSYSTEM (OS): Wir nutzen es nur wenig.  Das OS pflegt "Schattenregister" im RAM (SDMCTL, COLOR0.., PCOLR0.., GPRIOR,
;   CHBAS, SDLSTL), die es in jedem Bildruecklauf in die Chip-Register kopiert; die meisten Chip-Register kann man nicht lesen und
;   ein Schreiben direkt ins Chip-Register wuerde im naechsten Bild vom OS ueberschrieben.  Nur Register OHNE Schatten
;   (HPOSPn, GRACTL, PMBASE, NMIEN, WSYNC, POKEY) werden direkt beschrieben.  Wirkung eines Schattenwerts: erst im naechsten Bild.
;
; SPEICHERPLAN:  $2000-$3FFF Programm   $4000 Bildschirm   $4400 Zeichensatz   $4800 Display-Liste   $5000-$57FF Spielerspeicher
;                $5800 Bildkarte (LMAP)   $5CE0-$5FFF RMT-Tabellen   $6000 RMT-Player   $7000 Tabellen, Bilddaten, Musikmodul
; NULLSEITE:     $80-$B3 Spiellogik, $B4-$BB diese Datei, $CB-$DD RMT-Player
;                Die Interrupts (dli, vbi, sfx_update) benutzen keine Nullseitenzelle ausser dem Player (RMT) selbst.
; =====================================================================================================================

.const SCREEN  = $4000
.const CHARSET = $4400
.const DLIST   = $4800
.const PM_M    = $5300
.const PM_P0   = $5400            ; P1 = $5500, P2 = $5600, P3 = $5700

; ---- Betriebssystem-Schattenregister (werden vom OS im Bildruecklauf in die Chips kopiert)
.const SDMCTL  = $022f            ; DMA: Bit 5 Display-Liste, Bit 4 Einzelzeile, Bit 3 Spieler, Bit 2 Missiles, Bits 0-1 Breite (2 = normal)
.const SDLSTL  = $0230            ; Adresse der Display-Liste (niedrig, hoch)
.const GPRIOR  = $026f            ; Prioritaet: Bit 4 = Missiles als 5. Spieler (COLPF3), Bit 0 = Spieler vor Spielfeld
.const PCOLR0  = $02c0            ; Spielerfarben 0..3
.const COLOR0  = $02c4            ; COLPF0..COLPF3
.const COLOR4  = $02c8            ; COLBK
.const CHBAS   = $02f4            ; Seite des Zeichensatzes
.const VDSLST  = $0200            ; Vektor des Display-List-Interrupts
.const RTCLOK  = $14              ; Bildzaehler (niedrig), zaehlt 50 (PAL) bzw. 60 (NTSC) Mal je Sekunde
.const SETVBV  = $e45c            ; OS-Routine: Vektor setzen (A = 7: verzoegerter Bildruecklauf-Interrupt, X = hoch, Y = niedrig)
.const XITVBV  = $e462            ; Ende eines verzoegerten Bildruecklauf-Interrupts

; ---- Chip-Register
.const HPOSP0  = $d000            ; horizontale Lage Spieler 0..3 ($d000-$d003) und Missiles 0..3 ($d004-$d007), in Farbtakten
.const GRACTL  = $d01d            ; Bit 0 Missiles, Bit 1 Spieler freigeben
.const PMBASE  = $d407            ; Seite des Spielerspeichers
.const WSYNC   = $d40a            ; Schreiben: auf das Zeilenende warten
.const COLBK   = $d01a
.const NMIEN   = $d40e            ; Bit 7 DLI, Bit 6 Bildruecklauf-Interrupt
.const AUDF4   = $d206
.const AUDC4   = $d207

; Zeiger fuer (indirekt),y-Zugriffe und Hilfsspeicher der HAL (Nullseite; die Spiellogik belegt $80-$B3, der RMT-Player $CB-$DD)
.const sp  = $b4                  ; Zeiger (2 Byte)
.const dp  = $b6                  ; zweiter Zeiger (2 Byte), meist das Ziel
.const pt0 = $b8                  ; Hilfsspeicher (je Routine anders benutzt, siehe dort)
.const pt1 = $b9
.const pt2 = $ba
.const pt3 = $bb

; ---------------------------------------------------------------- Start
; hal_init: Chips einrichten.  Der Bildschirm bleibt dunkel (SDMCTL = 0), bis hal_unblank ihn einschaltet.  Aendert A, X, Y.
;   Hinweis: Display-Liste (DLIST), Bildschirm, Zeichensatz und Skyc-Farben muessen vor hal_unblank bereitstehen; die Display-Liste
;   kopiert das Hauptprogramm (jumprun.asm, start) vor dem Aufruf von hal_init.
;   Alle Farbwerte sind Atari-Farben: hoeherwertiges Halbbyte = Farbton (0 grau, 2 orange, 3 rot-orange, 8 blau, 12 gruen ...),
;   niederwertiges Halbbyte = Helligkeit (0..14, gerade Werte).
hal_init:
        lda #0
        sta SDMCTL               ; keine DMA: Bildschirm aus
        sta sfx_on               ; kein Klangeffekt aktiv
        lda #<DLIST
        sta SDLSTL               ; Zeiger auf die Display-Liste (das OS kopiert ihn im naechsten Bild nach DLISTL/H)
        lda #>DLIST
        sta SDLSTL+1
        lda #>CHARSET
        sta CHBAS                ; nur die Seite: $44 -> Zeichensatz bei $4400
        lda #>(PM_M - $300)           ; Spielerspeicher: Seite $50 (Missiles liegen $300 dahinter)
        sta PMBASE
        lda #3
        sta GRACTL               ; Spieler und Missiles freigeben
        lda #$11
        sta GPRIOR               ; Spieler vor dem Spielfeld, Missiles als 5. Spieler
        lda #$00
        sta PCOLR0               ; P0 Umriss: schwarz
        lda #$3c
        sta PCOLR0+1             ; P1 Haut
        lda #$36
        sta PCOLR0+2             ; P2 Hemd: rot
        lda #$86
        sta PCOLR0+3             ; P3 Hose: blau
        lda #$c6
        sta COLOR0+3             ; COLPF3: gruen (Gras, Busch und der Gegner)
        ; Display-List-Interrupt und Bildtakt einhaengen
        lda #<dli
        sta VDSLST               ; Vektor des DLI: das OS springt aus dem NMI hierher
        lda #>dli
        sta VDSLST+1
        lda #7
        ldx #>vbi
        ldy #<vbi
        jsr SETVBV               ; verzoegerten Bildruecklauf-Vektor (VVBLKD) auf vbi setzen
        lda #$c0
        sta NMIEN                ; DLI (Bit 7) und Bildruecklauf-Interrupt (Bit 6) freigeben
        jsr clear_pm
        rts

; Display-Liste: 3 x 8 Leerzeilen (24 Zeilen Oberrand), dann 24 Textzeilen Modus 4; bei den Zeilen 3, 7, 11, 15, 19 das DLI-Bit ($80).
;   Befehlsbytes von ANTIC:  $70 = 8 leere Rasterzeilen (Hintergrundfarbe)   $44 = Modus 4 + Bit 6 (LMS: es folgt die 16-Bit-Adresse
;   des Bildschirmspeichers)   $04 = Modus 4, Speicher laeuft einfach weiter (je Zeile 40 Byte)   $84 = wie $04, aber mit DLI-Bit 7
;   $41 = JVB: springe an die folgende Adresse und warte auf das Bildende (Display-Liste von vorn).
;   Die Liste liegt im Programmbereich und wird beim Start nach DLIST ($4800) kopiert (siehe jumprun.asm), vermutlich damit die
;   XEX-Datei kein eigenes Segment bei $4800 braucht.  (Eine Display-Liste darf nicht ueber eine 1-KB-Grenze laufen; die 32 Byte
;   bei $4800 tun das nicht.)
;   Die Schleife zaehlt i = 0..22 fuer die Textzeilen 1..23; Zeile 0 ist das $44 davor.  i = 2 ist also Textzeile 3, usw.
dlist_data:
        .byte $70,$70,$70
        .byte $44,<SCREEN,>SCREEN
        .repeat 23, i
        .if i == 2 || i == 6 || i == 10 || i == 14 || i == 18
        .byte $84
        .else
        .byte $04
        .endif
        .endrepeat
        .byte $41,<DLIST,>DLIST
dlist_end:

; ---------------------------------------------------------------- Interrupts
; DLI: wird am Ende einer Bandzeile ausgeloest und setzt die Hintergrundfarbe fuer den naechsten Balken.  WSYNC wartet bis zum Zeilenende,
; damit die Farbe genau mit der naechsten Zeile wechselt (sonst entsteht ein Farbkeil mitten in der Zeile).
;   ANTIC meldet den DLI zu Beginn der LETZTEN Rasterzeile (von 8) der markierten Textzeile.  Die Farbe aus der Tabelle wird schon
;   vor WSYNC in A geholt, damit nach dem Aufwachen nur noch ein einziger Schreibzugriff (sta COLBK) bleibt.
;   Register: A und X werden gerettet (pha / txa pha ...) und am Ende zurueckgeholt; Y bleibt unberuehrt.  rti beendet den Interrupt.
;   dli_i = Nummer des naechsten Balkens (1..5): der Balken 0 ist die Farbe, die das OS aus COLOR4 in COLBK kopiert.
;   Dauer: nur wenige Takte plus das Warten auf WSYNC; benutzt keine Nullseite.
dli:    pha
        txa
        pha
        ldx dli_i
        lda skyc,x
        sta WSYNC
        sta COLBK
        inx
        stx dli_i
        pla
        tax
        pla
        rti
dli_i:  .byte 1                             ; Nummer des Balkens, den der naechste DLI setzt (vbi stellt 1 ein)
skyc:   .byte $9c,$9a,$98,$96,$94,$92       ; Hintergrundfarbe je Balken (Anfangswerte; hal_load_level / show_title ueberschreiben sie je Bild;
                                            ; Balken 0 setzt hal_unblank ueber COLOR4, die Balken 1-5 der DLI)

; VBI (verzoegerter Bildruecklauf-Interrupt, 50/60 Mal je Sekunde): Balkenzaehler zuruecksetzen, Musik, Effekte.
;   "Verzoegert" heisst: das OS hat seine eigene Arbeit (Schattenregister kopieren, RTCLOK erhoehen) schon erledigt und die Register
;   gerettet; wir enden mit XITVBV, das sie zurueckholt.  Der Balkenzaehler wird auf 1 gestellt, weil Balken 0 ueber COLOR4 laeuft.
;   RASTERMUSICTRACKER+3 ist der Einsprung "Play" des RMT-Players (+0 = Init, +9 = Stopp); er schreibt die POKEY-Kanaele 1-4.
vbi:    lda #1
        sta dli_i                ; naechster DLI beginnt wieder bei Balken 1
        jsr RASTERMUSICTRACKER+3 ; RMT-Player: ein Schritt der Musik (schreibt alle vier POKEY-Kanaele)
        jsr sfx_update           ; Effekt danach: ueberschreibt Kanal 4
        jmp XITVBV               ; OS-Ende des verzoegerten VBI: stellt die Register wieder her (kein rti hier)

; hal_wait_frame: auf das naechste Bild warten; RTCLOK zaehlt in jedem Bild hoch.  Aendert nur A.
;   Die Schleife liest den Zaehler einmal und wartet, bis er sich geaendert hat (das OS erhoeht ihn im Bildruecklauf).  So laeuft die
;   Spiellogik genau einmal je Bild: 50 Mal je Sekunde (PAL).
hal_wait_frame:
        lda RTCLOK
-       cmp RTCLOK
        beq -
        rts

; hal_blank / hal_unblank: Bildschirm aus/an (hal_blank wartet 2 Bilder und loescht danach alle Figuren; hal_unblank aendert nur A).
; Waehrend des Aufbaus eines neuen Bildes bleibt der Bildschirm einfarbig, man sieht kein halbfertiges Bild.
; Mit SDMCTL = 0 holt ANTIC keine Daten mehr (kein Bild, keine Spieler); hal_blank wartet zuerst auf ein Bildende, damit der Schalter
; nicht mitten im Bild liegt.  Der neue Wert wirkt erst im naechsten Bildruecklauf.
hal_blank:
        jsr hal_wait_frame
        lda #0
        sta SDMCTL               ; Bildschirm und DMA aus
        sta COLOR4               ; Hintergrund schwarz: ohne DMA zeigt ANTIC nur COLBK (sonst ein einfarbiges Zwischenbild in Himmelsfarbe)
        jsr hal_wait_frame       ; beide Werte wirken im naechsten Bildruecklauf; erst danach ist das alte Bild weg und der Umbau unsichtbar
        jmp clear_pm
;   $3E = %00111110: Bit 5 Display-Liste, Bit 4 Einzelzeilen-Spieler, Bit 3 Spieler, Bit 2 Missiles, Bits 1-0 = 10 normale Breite
;   (40 Zeichen = 160 Farbtakte).  Nur mit Bit 5 (Display-Liste) sieht man ueberhaupt etwas.
;   Die Himmelsfarbe des obersten Balkens (skyc, von hal_load_level / show_title gesetzt) kommt hier in COLOR4: Farbe und DMA
;   werden im selben Bildruecklauf uebernommen, das neue Bild erscheint also in einem Zug.
hal_unblank:
        lda skyc
        sta COLOR4               ; COLBK fuer den obersten Balken (die uebrigen holt der DLI)
        lda #$3e
        sta SDMCTL               ; Display-Liste, Spieler, Missiles, Einzelzeile, normale Breite
        rts

; clear_pm: alle Spieler- und Missile-Speicher loeschen (Figuren unsichtbar) und die gemerkten Zeilen zuruecksetzen.  Aendert A, X.
;   Die Schleife laeuft mit X von 0 bis 255 (Ueberlauf auf 0 beendet sie) und loescht je Durchgang ein Byte in jeder der fuenf Seiten.
;   old_top / old_etop = Startzeile, an der die Figuren zuletzt gezeichnet wurden (siehe hal_update_sprites).
clear_pm:
        lda #0
        tax
-       sta PM_M,x
        sta PM_P0,x
        sta PM_P0+$100,x
        sta PM_P0+$200,x
        sta PM_P0+$300,x
        inx
        bne -
        sta old_top
        sta old_etop
        rts

; ---------------------------------------------------------------- Kacheln
; Eine Kachel = 2 x 2 Zeichen.  Kachel (c, k) liegt in den Zellen (2c, 2k) ... (2c+1, 2k+1); eine Textzeile hat 40 Byte.
;   Eine Kachelzeile belegt also zwei Textzeilen = 80 Byte.  Die Tabellen liefern die Anfangsadresse (niedriges/hohes Byte) der
;   oberen Zeichenzeile jeder der 12 Kachelzeilen: $4000, $4050, $40A0, ...
row80_lo:
    .repeat 12, i
        .byte <(SCREEN + i * 80)
    .endrepeat
row80_hi:
    .repeat 12, i
        .byte >(SCREEN + i * 80)
    .endrepeat

; hal_put_tile: X = Spalte (0..19), Y = Zeile (0..11), A = Kachelnummer.  Gras und Busch bekommen Zeichenbit 7 (tile_hi): Farbe COLPF3 = gruen.
;   Aendert A, X, Y, sp, pt0, pt1.  Die Kachelliste lv1_tiles (aus dem Export von lv1) gilt fuer alle vier Bilder.
;   Schritte: (1) sp = row80[Zeile] + 2 * Spalte = Adresse der Zelle oben links, (2) Bit 7 der Kachel aus tile_hi holen,
;   (3) die vier Zeichennummern der Kachel (lv1_tiles, 4 Byte je Kachel: oben links, oben rechts, unten links, unten rechts)
;   mit diesem Bit verodern und an die Offsets 0, 1, 40, 41 schreiben (Offset 40 = eine Zeile tiefer).
;   Anders als beim C64 gibt es hier kein Farb-RAM: die Farbe steckt allein in Bit 7 des Zeichencodes.
hal_put_tile:
        sta pt0
        lda row80_lo,y
        sta sp
        lda row80_hi,y
        sta sp+1
        txa
        asl a
        clc
        adc sp
        sta sp
        bcc +
        inc sp+1
+       ldx pt0
        lda tile_hi,x
        sta pt1                  ; Zeichenbit 7 oder 0
        lda pt0
        asl a
        asl a
        tax                      ; X = Kachelnummer * 4 = Index in lv1_tiles
        ldy #0
        lda lv1_tiles,x
        ora pt1
        sta (sp),y
        iny
        lda lv1_tiles+1,x
        ora pt1
        sta (sp),y
        ldy #40
        lda lv1_tiles+2,x
        ora pt1
        sta (sp),y
        iny
        lda lv1_tiles+3,x
        ora pt1
        sta (sp),y
        rts

; hal_draw_level: zeichnet das ganze Bild aus der Karte im RAM (LMAP): 12 Zeilen x 20 Spalten, je Kachel get_tile (aus core.asm,
;   liefert die Kachelnummer fuer X = Spalte, Y = Zeile in A) und hal_put_tile.  Dauert einige Bilder lang (240 Kacheln); deshalb
;   laeuft sie bei abgeschaltetem Bildschirm.  pt3 = Zeile, pt2 = Spalte.  Aendert A, X, Y, pt0..pt3, sp.
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

; hal_title: Titelbild, 960 Zeichencodes (24 Zeilen * 40), in den Bildschirmspeicher kopieren.  Aendert A, X, Y, sp, dp.
;   Quelle ist title_chars (im Bildschirm-Editor gezeichnet, Export build/export/title.asm).  Kopiert werden 3 volle Seiten
;   (3 * 256 = 768 Byte) und dann noch 192 Byte: zusammen 960.  sp = Quelle, dp = Ziel; beide Zeiger werden seitenweise erhoeht.
hal_title:
        lda #<title_chars
        sta sp
        lda #>title_chars
        sta sp+1
        lda #<SCREEN
        sta dp
        lda #>SCREEN
        sta dp+1
        ldx #3
.pg:    ldy #0
-       lda (sp),y
        sta (dp),y
        iny
        bne -
        inc sp+1
        inc dp+1
        dex
        bne .pg
        ldy #0
-       lda (sp),y
        sta (dp),y
        iny
        cpy #192
        bne -
        rts

; ---------------------------------------------------------------- Bild laden: A = Bildnummer
; hal_load_level: Farben (COLPF0..2 und die Himmelsbalken), Karte nach LMAP, Bild und Statuszeile zeichnen.  Aendert alle Register.
;   Dieselben Kachelzeichen sehen auf der Wiese, in der Hoehle, in der Burg und am Himmel verschieden aus, weil sich je Bild
;   COLPF0, COLPF1 und COLPF2 aendern (Schattenregister COLOR0..2).  COLPF3 (gruen) bleibt immer gleich.
;   Der erste Balken (sky0) geht zusaetzlich nach COLOR4 = COLBK, weil das OS ihn dort fuer den obersten Balken einsetzt;
;   die Balken 1..5 holt der DLI aus skyc.  Die Karte (240 Byte = 20 x 12 Kacheln) wird aus maps_lo/maps_hi nach LMAP kopiert;
;   die Spiellogik loescht daraus spaeter eingesammelte Muenzen.
hal_load_level:
        sta pt0
        tax
        lda lvl_pf0,x
        sta COLOR0
        lda lvl_pf1,x
        sta COLOR0+1
        lda lvl_pf2,x
        sta COLOR0+2
        lda lvl_sky0,x
        sta skyc                 ; COLOR4 (COLBK) setzt erst hal_unblank: waehrend der Pause bleibt der Hintergrund schwarz
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
        lda maps_lo,x
        sta sp
        lda maps_hi,x
        sta sp+1
        ldy #0
-       lda (sp),y
        sta LMAP,y
        iny
        cpy #240
        bne -
        jsr hal_draw_level
        jmp hal_hud

; hal_hud: Statuszeile (Zeile 0) und Laufschrift (Zeile 23, die letzte)
;   Zeile 0 (30 Zeichen ab Spalte 1): "HUGO 000000   BILD 1/4   LEBEN"; die Ziffern werden danach ueberschrieben:
;   Punkte in den Spalten 6-11 (hal_score_draw), die Bildnummer in Spalte 20 (Textindex 19 + Spalte 1), drei Sterne als Leben
;   in den Spalten 31-33.  Die Ziffernzeichen sind CH_DIGIT0 + Ziffer.  Zeile 23 (Index 23 * 40 = 920) zeigt den Namen des
;   Bildes (25 Zeichen ab Spalte 7), Zeiger aus txt_lv_lo/hi.  Zeile 23 ist die untere Zeichenzeile der Kachelzeile 11 und ueberschreibt dort
;   die Kacheln.  Anders als beim C64 gibt es keine Farb-RAM-Schreibzugriffe: die Farben gelten fuer das ganze Bild.
hal_hud:
        ldx #0
-       lda txt_hud,x
        sta SCREEN+1,x
        inx
        cpx #30
        bne -
        lda lvl
        clc
        adc #CH_DIGIT0+1
        sta SCREEN+20            ; Bildnummer
        lda #CH_STAR
        sta SCREEN+31
        sta SCREEN+32
        sta SCREEN+33
        ldx lvl
        lda txt_lv_lo,x
        sta sp
        lda txt_lv_hi,x
        sta sp+1
        ldy #0
-       lda (sp),y
        sta SCREEN+920+7,y       ; Zeile 23 = Index 23 * 40 = 920
        iny
        cpy #25
        bne -
        jmp hal_score_draw

; hal_score_draw: Punktestand als sechs Ziffern (drei BCD-Bytes sc2, sc1, sc0, hoechstwertiges zuerst) in die Statuszeile (Spalten 6-11).
;   Aendert A, X.  Das Ende (put_bcd) wird vom dritten Aufruf direkt mit durchlaufen, deshalb steht dort der rts fuer alle drei Aufrufe.
hal_score_draw:
        lda sc2
        ldx #0
        jsr put_bcd
        lda sc1
        ldx #2
        jsr put_bcd
        lda sc0
        ldx #4
put_bcd:                         ; A = BCD, X = Spaltenversatz ab Spalte 6 (0, 2, 4); aendert A
        pha
        lsr a
        lsr a
        lsr a
        lsr a                    ; obere Ziffer (Zehner)
        clc
        adc #CH_DIGIT0
        sta SCREEN+6,x
        pla
        and #$0f                 ; untere Ziffer (Einer)
        clc
        adc #CH_DIGIT0
        sta SCREEN+7,x
        rts

; ---------------------------------------------------------------- Figuren
; Tabellen fuer den Zugriff auf die Animationsbilder.  Der Held hat 14 Bilder (7 nach rechts, 7 gespiegelt nach links) zu je
; 21 Zeilen (Bytes) je Spieler; der Gegner 6 Bilder (3 + 3 gespiegelt) zu je 13 Bytes.  frame21_lo/hi = Bildnummer * 21 als 16-Bit-Wert
; (Versatz in hero_p0..p3), frame13_lo = Bildnummer * 13 (das hoechste Bild 5 * 13 = 65 passt in ein Byte; das hohe Byte der Adresse
; ergibt sich beim Addieren aus dem Uebertrag).
frame21_lo:
    .repeat 14, i
        .byte <(i * 21)
    .endrepeat
frame21_hi:
    .repeat 14, i
        .byte >(i * 21)
    .endrepeat
frame13_lo:
    .repeat 6, i
        .byte <(i * 13)
    .endrepeat
herotab_lo: .byte <hero_p0,<hero_p1,<hero_p2,<hero_p3       ; Anfang der Bildtabelle je Spieler
herotab_hi: .byte >hero_p0,>hero_p1,>hero_p2,>hero_p3
old_top:  .byte 0                ; Startzeile des Helden im Spielerspeicher beim letzten Zeichnen (zum Loeschen)
old_etop: .byte 0                ; dasselbe fuer den Gegner (Startzeile in der Missile-Seite PM_M)

; hal_update_sprites: Held und Gegner aus h_px/h_py/h_fr und e_px/e_py/e_fr in die Spielerspeicher schreiben.
;   Horizontal: Farbtakt = x / 2 + 48 (linker Rand des Bildes) + 2 (Spalte 2 des 12 Punkte breiten Bildes = Beginn des 8 Takte breiten Spielers)
;   Vertikal:   Index im Spielerspeicher = 32 + Fuss-y - Hoehe; Held 21 Zeilen (Fuss in Zeile 20) -> h_py + 12; Gegner 13 Zeilen -> e_py + 19.
;   Ablauf: (1) x-Lage setzen, (2) die alten Zeilen aller vier Spieler loeschen (das Bild wandert ja in y; die Bytes an der
;   alten Stelle wuerden sonst als Schatten stehenbleiben), (3) die 21 Bytes des aktuellen Bildes jedes Spielers an die neue Stelle
;   kopieren.  Der Gegner wird ebenso behandelt, nur mit 13 Bytes, einer gemeinsamen Missile-Seite und vier HPOS-Registern.
;   Aendert A, X, Y, sp, dp, pt0.  Die Spieler werden durch Schreiben in den Speicher bewegt (ANTIC holt sie per DMA): es gibt kein
;   Sprite-Register fuer y.  Die Hoehe wird nicht auf Wertebereich geprueft (h_py + 12 muss unter 256 bleiben, hier stets der Fall).
hal_update_sprites:
        ; ---- Held: Lage
        lda h_px_h
        lsr a                    ; Bit 0 des hohen Bytes in den Uebertrag ...
        lda h_px_l
        ror a                    ; ... und von links wieder einschieben: 16-Bit-Wert / 2, Ergebnis passt in ein Byte (0..160)
        clc
        adc #50                  ; 48 (linker Rand) + 2 (Spalte 2 des Bildes)
        sta HPOSP0
        sta HPOSP0+1
        sta HPOSP0+2
        sta HPOSP0+3             ; alle vier Spieler gleich: sie liegen genau uebereinander
        ; ---- Held: alte Zeilen loeschen, neue schreiben
        ; Schleife .clr: X = Spieler 0..3.  dp zeigt auf Seite (PM_P0 + X) in der Zeile old_top (dp low), geloescht werden 21 Byte
        ; (Y = 20 bis 0; bpl endet, wenn Y unter 0 faellt).
        ldx #0
.clr:   lda #>PM_P0
        stx pt0                  ; Spieler k
        clc
        adc pt0
        sta dp+1
        lda old_top
        sta dp
        ldy #20
        lda #0
-       sta (dp),y
        dey
        bpl -
        inx
        cpx #4
        bne .clr
        lda h_py
        clc
        adc #12
        sta old_top              ; neue Startzeile = Fuss-y + 12 (siehe oben)
        ; Schleife .put: sp = Anfang der Tabelle von Spieler k + Bildnummer * 21; dp = Ziel in Seite k; 21 Byte kopieren.
        ldx #0
.put:   stx pt0
        lda h_fr
        tay
        lda herotab_lo,x
        clc
        adc frame21_lo,y
        sta sp
        lda herotab_hi,x
        adc frame21_hi,y
        sta sp+1
        lda #>PM_P0
        clc
        adc pt0
        sta dp+1
        lda old_top
        sta dp
        ldy #20
-       lda (sp),y
        sta (dp),y
        dey
        bpl -
        ldx pt0
        inx
        cpx #4
        bne .put
        ; ---- Gegner: Missiles nebeneinander (je 2 Takte)
        ;   Missile 0..3 stehen 2 Takte auseinander (HPOSM0 = $d004 = HPOSP0+4 ...); bei der Addition bleibt der Uebertrag 0,
        ;   weil die Lage unter 256 bleibt, deshalb kommt adc ohne clc aus.
        lda e_px_h
        lsr a
        lda e_px_l
        ror a
        clc
        adc #50
        sta HPOSP0+4
        adc #2
        sta HPOSP0+5
        adc #2
        sta HPOSP0+6
        adc #2
        sta HPOSP0+7
        ; Alle vier Missiles teilen sich eine Seite (PM_M): je Zeile ein Byte, 2 Bit je Missile.  Alte 13 Zeilen loeschen ...
        lda #>PM_M
        sta dp+1
        lda old_etop
        sta dp
        ldy #12
        lda #0
-       sta (dp),y
        dey
        bpl -
        ; ... neue Startzeile = Fuss-y + 19, dann die 13 Byte des Bildes aus enemy_m (Bildnummer * 13) hineinkopieren.
        lda e_py
        clc
        adc #19
        sta old_etop
        sta dp
        ldx e_fr
        lda frame13_lo,x
        clc
        adc #<enemy_m
        sta sp
        lda #>enemy_m
        adc #0
        sta sp+1
        ldy #12
-       lda (sp),y
        sta (dp),y
        dey
        bpl -
        rts

; ---------------------------------------------------------------- Klangeffekte (POKEY-Kanal 4)
; Effekte 0 Absprung, 1 Landung, 2 Schritt, 3 Klettern, 4 Muenze, 5 Gumbo klettert, 6 Bildwechsel-Jingle, 7 Fanfare.
; Eintrag je 4 Byte: AUDF-Startwert (kleinerer Wert = hoeherer Ton), Aenderung je Bild (vorzeichenbehaftet), AUDC (Klangart * 32 + Lautstaerke:
; $A0 = reiner Ton, $80 = Rauschen), Dauer in Bildern.
;   AUDF4 ($D206) ist der Teiler der Tonhoehe, AUDC4 ($D207) enthaelt in den Bits 7-5 die Klangart und in den Bits 3-0 die Lautstaerke
;   (Bit 4 = nur Lautstaerke, hier nie gesetzt).  Die Aenderung wird bei jedem Bild auf AUDF addiert, so gleitet der Ton:
;   253 = -3 (Ton steigt), 250 = -6, 255 = -1, 4 = +4 (Ton faellt), 0 = bleibt.
sfx_tab:
        .byte 100, 253, $a8, 22        ; 0 Absprung: AUDF faellt = Ton steigt
        .byte 40, 4, $8a, 7            ; 1 Landung: Rauschen
        .byte 20, 0, $86, 2            ; 2 Schritt: Klick
        .byte 60, 250, $a6, 4          ; 3 Klettern
        .byte 40, 253, $aa, 10         ; 4 Muenze
        .byte 120, 4, $a8, 12          ; 5 Gumbo klettert
        .byte 70, 255, $aa, 40         ; 6 Bildwechsel-Jingle
        .byte 90, 255, $aa, 60         ; 7 Fanfare
sfx_on:  .byte 0                 ; ungleich 0, solange ein Effekt klingt (core.asm liest das)
sfx_f:   .byte 0                 ; aktueller AUDF-Wert
sfx_d:   .byte 0                 ; Aenderung je Bild (vorzeichenbehaftet, wird zu sfx_f addiert)
sfx_c:   .byte 0                 ; AUDC-Wert (Klangart und Lautstaerke)
sfx_n:   .byte 0                 ; verbleibende Bilder

; hal_sfx: Effekt starten.  A = Effektnummer 0..7; Nummer * 4 ist der Index in sfx_tab.  Ein neuer Effekt ersetzt einen laufenden.
;   Aendert A, X.  Anders als beim C64 gibt es keinen Gate: der Effekt beginnt mit dem naechsten vbi (sfx_update schreibt AUDF4/AUDC4).
;   sfx_on wird zuerst 0 (sfx_update tut dann nichts), die vier Bytes werden gesetzt, sfx_on zuletzt wieder 1.
hal_sfx:
        asl a
        asl a                    ; Nummer * 4
        tax
        lda #0
        sta sfx_on               ; Effekt waehrend des Setzens aus (der VBI ruft sfx_update)
        lda sfx_tab,x
        sta sfx_f
        lda sfx_tab+1,x
        sta sfx_d
        lda sfx_tab+2,x
        sta sfx_c
        lda sfx_tab+3,x
        sta sfx_n
        lda #1
        sta sfx_on
        rts

; sfx_update: wird in jedem Bild vom VBI aufgerufen (nach dem Musikspieler, damit er Kanal 4 ueberschreibt).  Schreibt AUDF4 und AUDC4,
; schaltet den Tonteiler weiter und zaehlt die Dauer herunter; ist sie abgelaufen, wird AUDC4 = 0 (Lautstaerke 0) gesetzt.  Aendert A.
sfx_update:
        lda sfx_on
        beq +
        lda sfx_f
        sta AUDF4
        lda sfx_c
        sta AUDC4
        clc
        lda sfx_f
        adc sfx_d
        sta sfx_f
        dec sfx_n
        bne +
        lda #0
        sta sfx_on
        sta AUDC4                ; Kanal 4 wieder stumm
+       rts
