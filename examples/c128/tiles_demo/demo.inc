; =====================================================================================================================
; demo.inc - gemeinsamer Teil des Beispiels tiles_demo (C128, VIC-IIe, 40-Spalten-Textbildschirm)
; =====================================================================================================================
; Erzeugt von tools/gen_tiles_demo.py (nicht von Hand ändern; dieser Kopf steht dort in der Vorlage demo_inc()).
;
; WAS DAS BEISPIEL ZEIGT
;   Die Kachel-Bibliothek #include <c128/tiles.asm> (systemneutraler Teil: data/sys/common/tiles.asm): Ein Kachelsatz (hier 6 Kacheln zu
;   3x2 oder 2x2 Zeichenzellen, Daten in data_3x2.inc / data_2x2.inc) und eine Screen-Bank (3 Karten, 12x10 bzw. 15x10 Kacheln) werden mit
;   tiles_use verbunden.  Das Programm main zeichnet dann Screen 0 (screen_draw), eine Einzelkachel (tile_draw), eine eingefärbte
;   Kachel (tile_draw_color), ein Rechteck aus 2x2 gleichen Kacheln (tile_fill) und zur Laufzeit eine Kachel (tile_put).
;   Zu sehen ist danach ein Muster aus Zeichen und Farben auf dem 40x25-Bildschirm; das Programm bleibt in "jmp *" stehen.
;
; VIER VARIANTEN (tiles_demo_<größe>_<art>.asm legen die Schalter fest und binden diese Datei ein)
;   DEMO_SET   3 = Kacheln zu 3x2 Zellen, 2 = Kacheln zu 2x2 Zellen (wählt data_*.inc und expect_*.inc)
;   TILE_SMC   0 = Zeigerversion von screen_draw (ROM-tauglich), 1 = selbstverändernde Version (schneller, nur im RAM)
;   TILE_MAPRAM  (nur 3x2_smc, $3000) die Karte wird beim Wählen des Screens in einen RAM-Puffer kopiert
;
; PRÜFROUTINEN (vcc test, siehe tiles_demo_*.vctest)
;   Die Routinen t_* unten werden von vcc test über eine Falle bei $0B00 (laut .vctest; Kassettenpuffer des C128) einzeln aufgerufen.
;   Sie bauen ein Bild auf und vergleichen eine CRC-16 über Bildschirm und Farb-RAM mit dem unabhängig in Python berechneten
;   Sollwert (expect_*.inc); A = 0 heißt "richtig", 1 = Bildschirm falsch, 2 = Farbe falsch.  Die Prüfung läuft im Emulator (VICE x128).
;
; SPEICHERPLAN (alles RAM-Bank 0, MMU-Konfiguration unverändert $00 wie nach dem Start: Systemroms und E/A sichtbar)
;   $1C01        BASIC-Zeile "10 SYS <start>" (C128-BASIC-Start), dann "start: jmp main"
;   danach       Daten (Kachelsatz hg_*, Screen-Bank levels_*, Sollwerte), main, Prüfroutinen, Laufzeit der Bibliothek (tiles_runtime);
;                das Programm muss vor $3000 enden (.assert am Ende), weil dort der Kartenpuffer TILE_MAPRAM der 3x2_smc-Variante liegt
;   $0400-$07E7  Textbildschirm (TILE_SCREEN = SCREEN_RAM, 40 x 25 Zeichen; KERNAL-Standard des C128)
;   $D800-$DBE7  Farb-RAM (TILE_COLORRAM = COLOR_RAM, 4 Bit je Zelle; Bit 3 = Mehrfarbe wie am C64)
;   Zeropage:    $22-$29 Prüfroutinen (ZP_*), $F7-$FE Kachelgruppe der Bibliothek (TILE_ZP, 8 Byte)
;
; GRENZEN
;   Nur der VIC-IIe (40 Spalten); der VDC (80 Spalten) wird von c128/tiles.asm nicht unterstützt.  Das Programm sperrt die Interrupts
;   (sei) und ruft das KERNAL nie auf; es setzt voraus, dass der E/A-Bereich und RAM-Bank 0 sichtbar sind (Zustand nach dem Start).
; =====================================================================================================================
.target c128
.cpu 8502 { illegals = deny }
#include <c128/tiles.asm>

; ZP für die Prüfroutinen (außerhalb der Kachelgruppe $F7-$FE)
.const ZP_CRC = $22             ; CRC-16 (2 Byte)
.const ZP_PTR = $24             ; Zeiger auf die zu prüfenden Daten (2 Byte)
.const ZP_X   = $26             ; Merker für das X-Register (crc_area: Byte-Maske; t_map_get/t_prop: Spalte)
.const ZP_Y   = $27             ; Merker für das Y-Register (t_map_get/t_prop: Zeile)
.const ZP_CNT = $28             ; Zähler der noch zu verarbeitenden Bytes (2 Byte)
.const AREA   = TILE_COLS * TILE_ROWS   ; Zahl der Zellen = Bytes einer Prüfsumme (40 * 25 = 1000)

.org $1c01                      ; BASIC-Programmstart des C128
.basicstub start                ; erzeugt "10 SYS <start>", damit RUN das Programm startet
start:  jmp main                ; (vcc test springt die t_*-Routinen direkt an; RUN führt main aus)

; ---------------------------------------------------------------- Daten und Bibliothek
; Kachelsatz (hg_*) und Screen-Bank (levels_*) im Exportformat des Kachel-Editors, dazu die erwarteten Prüfsummen
.if DEMO_SET == 3
.include "data_3x2.inc"
.include "expect_3x2.inc"
.else
.include "data_2x2.inc"
.include "expect_2x2.inc"
.endif

; Bibliothek an Satz und Bank binden: Zeichen-Tabellen, Farbe, Eigenschaften, Breite/Höhe/Anzahl, Screen-Adresstabellen, Kartengröße
        tiles_use hg_c0, hg_col, hg_props, hg_tw, hg_th, hg_count, levels_screens_lo, levels_screens_hi, levels_tiles_w, levels_tiles_h
.const FX = DEMO_FX             ; Position des tile_fill-Rechtecks (Kachelposition), aus expect_*.inc
.const FY = DEMO_FY

; ---------------------------------------------------------------- Programm: zeigt Screen 0, dann ein paar Einzelkacheln
; main: Hauptprogramm (nach RUN).  Baut das Demobild auf und bleibt in einer Endlosschleife stehen.
main:
        sei                         ; Interrupts sperren: die KERNAL-Interruptroutine (Cursorblinken, Tastaturabfrage) würde Zeichen und
                                    ; Farben auf dem Bildschirm verändern und die Prüfsummen stören; die Bibliothek ruft das KERNAL nie auf
        lda #0
        sta VIC_BORDERCOLOR         ; Rahmen schwarz
        sta VIC_BGCOLOR0            ; Hintergrund schwarz
        jsr clear_all
        screen_draw lv1_id          ; Screen 0 wählen und zeichnen
        tile_draw 3, 5, 2           ; Kachel 3 an Kachelposition (5, 2), nur Konstanten
        tile_draw_color 1, 0, 0, 7  ; Kachel 1 links oben (0, 0) mit Farbe 7 (gelb) statt der Kachelfarbe
        tile_fill 4, FX, FY, 2, 2   ; 2 x 2 Kacheln der Kachel 4
        lda #1
        ldx #4
        ldy #6
        jsr tile_put                ; Laufzeit: A = Kachel, X = Spalte, Y = Zeile
        jmp *

; ---------------------------------------------------------------- Prüfroutinen (Aufruf durch vcc test, siehe tiles_*.vctest)
; clear_all: Bildschirm löschen (alle 1000 Zeichen = Leerzeichen $20, Farb-RAM = 1 = weiß).  Verändert A, X.
;   Vier Zugriffe je Schleifendurchlauf, X läuft 0..255: drei volle Seiten (Versatz $000, $100, $200) und die letzten 256 Zellen ab $2E8
;   ($2E8-$3E7 = Zellen 744..999; die Seiten überlappen sich bei $2E8-$2FF, was nichts schadet).  A wechselt je Durchlauf zwischen $20
;   (Zeichen) und $01 (Farbe) und wird vor dem INX wieder auf $20 gesetzt.
clear_all:
        ldx #0
        lda #$20
.l:
        sta TILE_SCREEN + $0,x
        sta TILE_SCREEN + $100,x
        sta TILE_SCREEN + $200,x
        sta TILE_SCREEN + $2e8,x
        lda #$01
        sta TILE_COLORRAM + $0,x
        sta TILE_COLORRAM + $100,x
        sta TILE_COLORRAM + $200,x
        sta TILE_COLORRAM + $2e8,x
        lda #$20
        inx
        bne .l
        rts

; crc_area: CRC-16/XMODEM (Polynom $1021, Startwert 0, höchstwertiges Bit zuerst, bitweise gerechnet) über AREA Byte ab (ZP_PTR).
;   Eingabe: ZP_PTR = Anfang der Daten, X = Maske, die auf jedes Byte per UND angewendet wird ($FF für den Bildschirm, $0F für das Farb-RAM,
;   dessen obere 4 Bit beim C128 nicht definiert sind).  Ausgabe: ZP_CRC (niedriges Byte zuerst).  Verändert A, X, Y, ZP_PTR (läuft
;   weiter), ZP_X, ZP_CNT.  Langsam (einige hunderttausend Takte), aber nur für den Test.
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
        and ZP_X                    ; Maske anwenden
        eor ZP_CRC+1                ; Byte in das obere CRC-Byte einmischen
        sta ZP_CRC+1
        ldx #8                      ; 8 Bits schieben
.bit:   asl ZP_CRC
        rol ZP_CRC+1
        bcc .nx                     ; kein Bit herausgefallen: nichts tun
        lda ZP_CRC+1
        eor #$10                    ; Polynom $1021: oberes Byte $10 ...
        sta ZP_CRC+1
        lda ZP_CRC
        eor #$21                    ; ... unteres Byte $21
        sta ZP_CRC
.nx:    dex
        bne .bit
        iny
        bne .pg
        inc ZP_PTR+1                ; Y lief über: nächste Seite
.pg:    lda ZP_CNT                  ; 16-Bit-Zähler herunterzählen (erst das obere Byte, wenn das untere 0 ist)
        bne .d1
        dec ZP_CNT+1
.d1:    dec ZP_CNT
        lda ZP_CNT
        ora ZP_CNT+1
        bne .byte
        rts

; check(exp_scr, exp_col): Makro, beendet eine Prüfroutine mit RTS.  Rechnet die CRC über den Bildschirm ($0400, 1000 Byte, Maske $FF)
;   und über das Farb-RAM ($D800, 1000 Byte, Maske $0F) und vergleicht mit den Sollwerten.
;   Rückgabe A: 0 = beides richtig, 1 = Bildschirm falsch, 2 = Farb-RAM falsch.  Verändert A, X, Y und die ZP_*-Zellen.
.macro check(exp_scr, exp_col)
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
        lda #<TILE_COLORRAM
        sta ZP_PTR
        lda #>TILE_COLORRAM
        sta ZP_PTR+1
        ldx #$0f
        jsr crc_area
        lda ZP_CRC
        cmp #<exp_col
        bne @bad2
        lda ZP_CRC+1
        cmp #>exp_col
        bne @bad2
        lda #0
        rts
@bad1:  lda #1
        rts
@bad2:  lda #2
        rts
.endmacro

; t_screen1: Szenario "ganzer Screen 1" (lv2_id): löschen, zeichnen, prüfen.  Rückgabe A wie check.
t_screen1:                          ; ganzer Screen 1
        jsr clear_all
        screen_draw lv2_id
        check EXP_S1_SCR, EXP_S1_COL

; t_screen1_nocheck: wie t_screen1, aber ohne Prüfsumme; vcc test liest danach einzelne Zellen (Stichproben) direkt aus dem Speicher.
t_screen1_nocheck:
        jsr clear_all
        screen_draw lv2_id
        rts

; t_draw_only: nur das Zeichnen, ohne vorheriges Löschen und ohne Prüfung (für die Messung der Zyklen von screen_draw)
t_draw_only:                        ; nur das Zeichnen (für die Zyklenmessung)
        screen_draw lv2_id
        rts

; t_rows: Szenario "Kachelzeilen 3..5 von Screen 3 (lv3_id)": screen_draw_rows mit A = erste Kachelzeile (3), X = Anzahl (3).
t_rows:                             ; Kachelzeilen 3..5 von Screen 3
        jsr clear_all
        screen_select lv3_id
        lda #3
        ldx #3
        jsr screen_draw_rows
        check EXP_ROWS_SCR, EXP_ROWS_COL

; t_macros: Szenario "Makros": tile_draw, tile_draw_color (Farbe 14) und tile_fill auf leerem Bildschirm (wie main, aber andere Farbe).
t_macros:
        jsr clear_all
        tile_draw 3, 5, 2
        tile_draw_color 1, 0, 0, 14
        tile_fill 4, FX, FY, 2, 2
        check EXP_MACRO_SCR, EXP_MACRO_COL

; t_puts: Szenario "tile_put": drei Kacheln zur Laufzeit setzen, darunter die rechte untere Ecke (letzte Spalte, letzte Zeile der
;   Karte) und die linke obere Ecke.  Prüft damit auch die Randfälle der Adressrechnung.
t_puts:
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
        check EXP_PUT_SCR, EXP_PUT_COL

; t_map_get: Eingabe X = Spalte, Y = Zeile (in Kacheln) von Screen 2 (lv2_id); Ausgabe A = Kachelnummer an dieser Stelle der Karte.
;   X und Y werden in ZP_X/ZP_Y gerettet, weil das Makro screen_select die Register verändert; danach springt die Routine mit JMP in
;   screen_map_get (Endaufruf: dessen RTS kehrt direkt zu vcc test zurück).
t_map_get:                          ; X = Spalte, Y = Zeile von Screen 2 (lv2_id) -> A = Kachelnummer
        stx ZP_X
        sty ZP_Y
        screen_select lv2_id
        ldx ZP_X
        ldy ZP_Y
        jmp screen_map_get

; t_prop: Eingabe X = Spalte, Y = Zeile von Screen 2; Ausgabe A = Eigenschaftenbyte (hg_props) der Kachel an dieser Stelle.
t_prop:
        stx ZP_X
        sty ZP_Y
        screen_select lv2_id
        ldx ZP_X
        ldy ZP_Y
        jmp tile_prop

; ---------------------------------------------------------------- Laufzeit der Bibliothek (einmal, hinter dem Programm)
; tiles_runtime erzeugt die Unterprogramme screen_draw_all, screen_draw_rows, screen_select_a, tile_put, screen_map_get, tile_prop
; (einmal je Programm, deshalb hinter dem eigenen Code).  Das .assert prüft die Obergrenze des Programms: darüber beginnt der
; Kartenpuffer ($3000, nur 3x2_smc) bzw. - allgemein - der freie Bereich unterhalb der Systemroms ($4000).
        tiles_runtime
        .assert * <= $3000, "Programm überschreitet die Grenze des freien Speichers (Bildschirm, Display-Liste oder Karte)"
