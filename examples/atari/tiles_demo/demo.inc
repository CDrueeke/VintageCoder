; =====================================================================================================================
; demo.inc - Gemeinsamer Teil des Beispiels tiles_demo (Atari 8 Bit, ANTIC-Modus 2): Kachel-Bibliothek und Selbsttest
; =====================================================================================================================
;
; ZWECK
;   Zeigt die Kachel-Bibliothek (#include <atari8bit/tiles.asm>, die ihrerseits common/tiles.asm einbindet) und enthält die
;   Prüfroutinen für vcc test (Postfach-Modus, atari8bit/test.inc).  Die vier Programme tiles_demo_*.asm unterscheiden sich nur in
;   Kachelgröße (DEMO_SET = 3 für 3x2, 2 für 2x2), TILE_SMC (0 = Zeigerversion, ROM-tauglich; 1 = selbstmodifizierender Code,
;   schneller) und bei 3x2_smc zusätzlich in TILE_MAPRAM (= $5000: die Karte wird beim Wählen eines Screens in einen RAM-Puffer
;   kopiert).  Sie legen diese Konstanten fest und binden dann diese Datei ein.
;   Diese Datei sowie data_*.inc, expect_*.inc und tiles_demo_*.asm stammen aus tools/gen_tiles_demo.py (nicht von Hand ändern;
;   Änderungen gehören in den Generator).  Die Kachel- und Kartendaten sind frei erfunden, kein Bildinhalt.
;
; ABLAUF (main)
;   1. test_begin: Postfach (Seite 6) löschen, Status "läuft".
;   2. selftest: alle Bildschirm-CRC-Prüfungen (Screen 1, Zeilenbereich, Makros, tile_put) sowie je drei Prüfungen von
;      screen_map_get und tile_prop an festen Kartenstellen (SELF_*-Werte aus expect_*.inc).
;   3. Demo: Display-Liste einrichten, Bildschirm löschen, Screen 0 zeichnen, Einzelkachel, Kachelfüllung, Kachel per Unterprogramm.
;   4. test_end: Ergebnis ins Postfach, Endlosschleife.  Der Läufer liest nur Speicher; das Demo-Bild bleibt stehen und wird
;      vom Test (siehe .vctest) im Bildschirmspeicher geprüft.
;
; SPEICHERPLAN
;   $2000 Programm (start: jmp main), danach Kachel-/Kartendaten (data_*.inc), die Bibliothek (tiles_runtime) und die
;   Prüfroutinen; das Programm darf $3800 nicht überschreiten (.assert).  $3800 Display-Liste (32 Bytes, darf keine
;   1-KB-Grenze kreuzen).  $4000-$43BF Bildschirm (TILE_SCREEN, 40 x 24 = 960 Bytes, Standard der Bibliothek).
;   $5000 Kartenpuffer (nur 3x2_smc, TILE_MAPRAM).
;   Zeropage: Kachelgruppe $F7-$FE (Bibliothek), $80-$87 für die Prüfroutinen (ZP_CRC bis ZP_CNT).
; KONVENTIONEN: Screens sind 0-basiert nummeriert (lv1_id = 0, lv2_id = 1, lv3_id = 2).  Positionen in Kacheln (nicht Zeichen), Bildschirmcode 0 = Leerzeichen.  Der Atari hat keinen Farbspeicher
;   (TILE_NOCOLOR): tile_draw_color zeichnet nur das Zeichen, die Farbwerte der Daten (hg_col) sind ohne Wirkung.
; BAUEN UND PRÜFEN: vcc build . ; vcc test . --backend atari800 (siehe tiles_demo_*.vctest: Überlappung, Zeropage, Zyklen von tile_put)
; =====================================================================================================================
.target atari8
.cpu sally { illegals = deny }
#include <atari8bit/tiles.asm>
#include <atari8bit/test.inc>

; Zeropage für die Prüfroutinen (außerhalb der Kachelgruppe $F7-$FE; das .vctest verbietet dem Programm Zugriffe auf $00-$7F)
.const ZP_CRC = $80             ; CRC-16 (2 Byte): Ergebnis von crc_area
.const ZP_PTR = $82             ; Zeiger auf die zu prüfenden Daten (2 Byte)
.const ZP_X   = $84             ; Zwischenspeicher X (Maske in crc_area, Spalte in t_map_get/t_prop)
.const ZP_Y   = $85             ; Zwischenspeicher Y (Zeile)
.const ZP_CNT = $86             ; Zähler (2 Byte)
.const AREA   = TILE_COLS * TILE_ROWS   ; Bytes des Bildschirmspeichers (40 x 24 = 960): so viele fließen in die CRC

.org $2000
start:  jmp main                    ; die Daten und Prüfroutinen folgen hinter dem Einsprung

; ---------------------------------------------------------------- Daten und Bibliothek
; Kachelsatz hg, Screen-Bank levels (Karten) und die erwarteten CRCs/Selbsttest-Werte des gewählten Satzes (2x2 oder 3x2).
.if DEMO_SET == 3
.include "data_3x2.inc"
.include "expect_3x2.inc"
.else
.include "data_2x2.inc"
.include "expect_2x2.inc"
.endif

        ; Satz und Bank verbinden (Namen aus dem Export; Makroargumente lassen sich nicht zusammensetzen)
        tiles_use hg_c0, hg_col, hg_props, hg_tw, hg_th, hg_count, levels_screens_lo, levels_screens_hi, levels_tiles_w, levels_tiles_h
.const FX = DEMO_FX             ; Kachelposition (Spalte) der 2 x 2 Füllkacheln, aus expect_*.inc
.const FY = DEMO_FY             ; dito (Zeile)

; ---------------------------------------------------------------- Programm: zeigt Screen 0, dann ein paar Einzelkacheln
main:
        test_begin                  ; Postfach-Modus: das Programm prüft sich selbst (siehe unten), zeigt dann die Demo
        jsr selftest
        jsr setup_display
        jsr clear_all
        screen_draw lv1_id          ; Screen 0 wählen und zeichnen
        tile_draw 3, 5, 2           ; Kachel 3 an Kachelposition (5, 2), nur Konstanten
        tile_draw_color 1, 0, 0, 0  ; Kachel 1 oben links; die Farbe 0 hat auf dem Atari keine Wirkung (kein Farbspeicher)
        tile_fill 4, FX, FY, 2, 2   ; 2 x 2 Kacheln der Kachel 4
        lda #1
        ldx #4
        ldy #6
        jsr tile_put                ; Laufzeit: A = Kachel, X = Spalte, Y = Zeile
        test_end                    ; Ergebnis ins Postfach, danach Endlosschleife

; Display-Liste: 3 x 8 leere Zeilen, 24 Textzeilen im ANTIC-Modus 2 (40 Spalten) ab TILE_SCREEN (LMS), Zeichensatz im ROM ($E000).
; Modus 4 (Mehrfarbtext) unterscheidet sich nur im Befehlsbyte ($44/$04 statt $42/$02) und braucht einen eigenen Zeichensatz.
SDLSTL = $0230                  ; OS-Schatten der Display-Listen-Adresse (2 Byte; das OS übernimmt ihn im nächsten Bild)
CHBAS  = $02f4                  ; OS-Schatten der Zeichensatzseite ($E0 = ROM-Zeichensatz bei $E000)
setup_display:
        lda #<dlist
        sta SDLSTL
        lda #>dlist
        sta SDLSTL+1
        lda #$e0
        sta CHBAS
        rts

selftest:                        ; alle Prüfungen nacheinander; jede t_*-Routine liefert A = 0, wenn die Bildschirm-CRC stimmt
        jsr t_screen1
        assert_a_eq 0
        jsr t_rows
        assert_a_eq 0
        jsr t_macros
        assert_a_eq 0
        jsr t_puts
        assert_a_eq 0
        ldx #SELF_X0
        ldy #SELF_Y0
        jsr t_map_get
        assert_a_eq SELF_T0
        ldx #SELF_X0
        ldy #SELF_Y0
        jsr t_prop
        assert_a_eq SELF_P0
        ldx #SELF_X1
        ldy #SELF_Y1
        jsr t_map_get
        assert_a_eq SELF_T1
        ldx #SELF_X1
        ldy #SELF_Y1
        jsr t_prop
        assert_a_eq SELF_P1
        ldx #SELF_X2
        ldy #SELF_Y2
        jsr t_map_get
        assert_a_eq SELF_T2
        ldx #SELF_X2
        ldy #SELF_Y2
        jsr t_prop
        assert_a_eq SELF_P2
        rts

; ---------------------------------------------------------------- Prüfroutinen (Aufruf durch selftest und main; Läufer: vcc test)
; clear_all: Bildschirm mit Zeichen 0 (Leerzeichen) füllen: drei Seiten und $2C0 Bytes, alles über denselben Index X.
; Ein: -.  Aus: A = 0, X = 0.
clear_all:
        ldx #0
        lda #$00
.l:
        sta TILE_SCREEN + $0,x
        sta TILE_SCREEN + $100,x
        sta TILE_SCREEN + $200,x
        sta TILE_SCREEN + $2c0,x
        inx
        bne .l
        rts

; crc_area: CRC-16 (XMODEM: Polynom $1021, Startwert 0, MSB zuerst) über AREA Bytes ab (ZP_PTR).
; Ein: ZP_PTR = Anfang, X = Maske, die per AND auf jedes Byte angewendet wird ($FF = alle Bits).
; Aus: ZP_CRC (Low/High).  Verändert A, X, Y, ZP_PTR (läuft mit), ZP_CNT.
crc_area:
        stx ZP_X
        lda #0
        sta ZP_CRC
        sta ZP_CRC+1
        lda #<AREA
        sta ZP_CNT
        lda #>AREA
        sta ZP_CNT+1
        ldy #0
.byte:  lda (ZP_PTR),y
        and ZP_X
        eor ZP_CRC+1
        sta ZP_CRC+1
        ldx #8
.bit:   asl ZP_CRC
        rol ZP_CRC+1
        bcc .nx
        lda ZP_CRC+1
        eor #$10
        sta ZP_CRC+1
        lda ZP_CRC
        eor #$21
        sta ZP_CRC
.nx:    dex
        bne .bit
        iny
        bne .pg
        inc ZP_PTR+1
.pg:    lda ZP_CNT
        bne .d1
        dec ZP_CNT+1
.d1:    dec ZP_CNT
        lda ZP_CNT
        ora ZP_CNT+1
        bne .byte
        rts

; check(exp_scr): Makro mit RTS (beendet das aufrufende Unterprogramm).  A = 0, wenn die CRC des Bildschirms gleich exp_scr
; ist, sonst 1.  Nur der Bildschirm wird geprüft (kein Farbspeicher auf dem Atari).
.macro check(exp_scr)
        lda #<TILE_SCREEN
        sta ZP_PTR
        lda #>TILE_SCREEN
        sta ZP_PTR+1
        ldx #$ff
        jsr crc_area
        lda ZP_CRC
        cmp #<exp_scr
        bne @bad1
        lda ZP_CRC+1
        cmp #>exp_scr
        bne @bad1
        lda #0
        rts
@bad1:  lda #1
        rts
.endmacro

t_screen1:                          ; ganzer Screen 1 (lv2_id) zeichnen und prüfen
        jsr clear_all
        screen_draw lv2_id
        check EXP_S1_SCR

t_screen1_nocheck:                  ; wie t_screen1, ohne Prüfung
        jsr clear_all
        screen_draw lv2_id
        rts

t_draw_only:                        ; nur das Zeichnen (für die Zyklenmessung), ohne vorheriges Löschen
        screen_draw lv2_id
        rts

t_rows:                             ; Kachelzeilen 3..5 von Screen 2 (lv3_id): screen_draw_rows mit A = erste Zeile, X = Anzahl
        jsr clear_all
        screen_select lv3_id
        lda #3
        ldx #3
        jsr screen_draw_rows
        check EXP_ROWS_SCR

t_macros:                           ; die Makros tile_draw, tile_draw_color und tile_fill
        jsr clear_all
        tile_draw 3, 5, 2
        tile_draw_color 1, 0, 0, 14
        tile_fill 4, FX, FY, 2, 2
        check EXP_MACRO_SCR

t_puts:                             ; tile_put: eine Kachel bei (2, 3), eine in der letzten Spalte/Zeile, eine bei (0, 0)
        jsr clear_all
        lda #5
        ldx #2
        ldy #3
        jsr tile_put
        lda #0
        ldx #levels_tiles_w - 1
        ldy #levels_tiles_h - 1
        jsr tile_put
        lda #3
        ldx #0
        ldy #0
        jsr tile_put
        check EXP_PUT_SCR

t_map_get:                          ; X = Spalte, Y = Zeile (in Kacheln) von Screen 1 (lv2_id) -> A = Kachelnummer aus der Karte
        stx ZP_X
        sty ZP_Y
        screen_select lv2_id
        ldx ZP_X
        ldy ZP_Y
        jmp screen_map_get

t_prop:                             ; X = Spalte, Y = Zeile von Screen 1 -> A = Eigenschaftenbyte der Kachel dort (hg_props)
        stx ZP_X
        sty ZP_Y
        screen_select lv2_id
        ldx ZP_X
        ldy ZP_Y
        jmp tile_prop

; ---------------------------------------------------------------- Laufzeit der Bibliothek (einmal, hinter dem Programm)
        tiles_runtime
        .assert * <= $3800, "Programm überschreitet die Grenze des freien Speichers (Bildschirm, Display-Liste oder Karte)"

        .org $3800                  ; Display-Liste (darf keine 1-KB-Grenze überschreiten)
dlist:  .byte $70,$70,$70           ; 3 x 8 leere Rasterzeilen oben
        .byte $42,<TILE_SCREEN,>TILE_SCREEN   ; erste Textzeile: Modus 2 mit LMS (Bildschirmadresse)
        .repeat TILE_ROWS - 1
        .byte $02
        .endrepeat
        .byte $41,<dlist,>dlist     ; Sprung an den Listenanfang, Warten auf das Bildende (JVB)
        .run start
