; =====================================================================================================================
; demo.inc - gemeinsamer Teil des Beispiels tiles_demo (C16)
; =====================================================================================================================
; Erzeugt von tools/gen_tiles_demo.py im Wurzelverzeichnis des Repositorys (nicht von Hand ändern: bei einem neuen Lauf des Skripts
; gehen Änderungen verloren; Kommentare gehören dann in die Vorlage des Skripts).
;
; ZWECK
;   Zeigt die Kachel-Bibliothek (#include <plus4/tiles.asm>, systemneutraler Teil data/sys/common/tiles.asm): ein Bildschirm aus Kacheln
;   (3x2 oder 2x2 Zeichen große Kacheln) wird in den Textbildschirm des TED gezeichnet, dazu Einzelkacheln, Farbvarianten und ein
;   Rechteck.  Die Datei enthält außerdem Prüfroutinen, die vcc test im Emulator aufruft und die Bildschirm- und Farbspeicher
;   gegen eine unabhängig berechnete CRC-16 vergleichen (tiles_demo_*.vctest).
;   Die vier Programme tiles_demo_*.asm unterscheiden sich nur in Kachelgröße (DEMO_SET = 3 für 3x2, 2 für 2x2), TILE_SMC (0 = Zeigerversion,
;   1 = selbstveränderter Code, schneller, nur im RAM) und TILE_MAPRAM (nur 3x2_smc: die Karte wird beim Wählen in einen RAM-Puffer kopiert);
;   sie legen diese Schalter fest und binden dann diese Datei ein.
;
; ABLAUF DES PROGRAMMS (Marke main)
;   Interrupts aus (das Beispiel läuft ohne KERNAL), Rahmen und Hintergrund schwarz, Bildschirm löschen (clear_all), Screen 0 zeichnen
;   (screen_draw), dann Kachel 3 an Kachelposition (5,2), Kachel 1 in Farbe $77 an (0,0), ein 2x2-Rechteck aus Kachel 4 und zur Laufzeit
;   (tile_put) Kachel 1 an (4,6); danach Endlosschleife.  Es gibt keine Eingabe.
;   Die Prüfroutinen t_* ruft nur der Test auf (Einsprung über die Marke; die Falle liegt laut .vctest bei $0340).
;
; BENUTZTE HARDWARE
;   Im Wesentlichen Speicher: Bildschirm $0C00 (Zeichencodes) und Farb-RAM $0800 (TED-Farbwert je Zelle: Farbton, Helligkeit, Blinken);
;   dazu die TED-Register $FF19 (Rahmenfarbe) und $FF15 (Hintergrundfarbe 0).
;
; SPEICHERPLAN
;   $1001 BASIC-Starter, danach main, die Prüfroutinen, die Kacheldaten und -karten (data_*.inc) und die Laufzeit der Bibliothek
;   (tiles_runtime).  Das Programm bleibt unter $4000 (.assert am Ende); bei TILE_MAPRAM liegt der Kartenpuffer bei $3000.
;   Nullseite: Kachelgruppe $F7-$FE (TILE_ZP der Bibliothek), Prüfroutinen $22-$28; die .vctest-Datei verbietet $02-$21.
; =====================================================================================================================
.target c16
.cpu 7501 { illegals = deny }
#include <plus4/tiles.asm>

; ZP für die Prüfroutinen (außerhalb der Kachelgruppe $F7-$FE)
.const ZP_CRC = $22             ; CRC-16 (2 Byte, Low/High): Ergebnis von crc_area
.const ZP_PTR = $24             ; Zeiger auf die zu prüfenden Daten (2 Byte)
.const ZP_X   = $26             ; Retter für X (Maske in crc_area; Spalte in t_map_get/t_prop)
.const ZP_Y   = $27             ; Retter für Y (Zeile in t_map_get/t_prop)
.const ZP_CNT = $28             ; Zähler der noch zu prüfenden Bytes (2 Byte)
.const AREA   = TILE_COLS * TILE_ROWS   ; Größe des geprüften Bereichs: 40 * 25 = 1000 Byte (Bildschirm bzw. Farb-RAM)

.org $1001                      ; Start des BASIC-Speichers beim C16 und Plus/4
.basicstub start                ; "10 SYS <start>": RUN springt nach start
start:  jmp main                ; (eigene Marke, damit der Einsprung unabhängig vom Starter bleibt)

; ---------------------------------------------------------------- Daten und Bibliothek
; data_*.inc: Kachelsatz hg (planare Zellentabellen hg_c0.., Farbe hg_col, Eigenschaften hg_props) und Screen-Bank levels (Karten, 3 Screens);
; expect_*.inc: erwartete CRC-16 je Prüfszenario und die Größe der Karte (MAP_TILES_W) sowie die Lage des Rechtecks (DEMO_FX/FY).
.if DEMO_SET == 3
.include "data_3x2.inc"
.include "expect_3x2.inc"
.else
.include "data_2x2.inc"
.include "expect_2x2.inc"
.endif

        tiles_use hg_c0, hg_col, hg_props, hg_tw, hg_th, hg_count, levels_screens_lo, levels_screens_hi, levels_tiles_w, levels_tiles_h
                                    ; verbindet Kachelsatz und Screen-Bank mit der Bibliothek (Makro, erzeugt keinen Code, nur Zeiger/Konstanten)
.const FX = DEMO_FX                 ; Position (Kachelspalte/-zeile) des 2x2-Rechtecks in main: aus expect_*.inc
.const FY = DEMO_FY

; ---------------------------------------------------------------- Programm: zeigt Screen 0, dann ein paar Einzelkacheln
main:
        sei                         ; keine Interrupts: das Beispiel läuft ohne KERNAL
        lda #0
        sta $ff19                  ; Rahmen schwarz
        sta $ff15                  ; Hintergrund schwarz
        jsr clear_all
        screen_draw lv1_id          ; Screen 0 wählen und zeichnen
        tile_draw 3, 5, 2           ; Kachel 3 an Kachelposition (5, 2), nur Konstanten
        tile_draw_color 1, 0, 0, 119 ; Kachel 1 oben links mit Farbe $77 (Gelb, Helligkeit 7) statt der Kachelfarbe
        tile_fill 4, FX, FY, 2, 2   ; 2 x 2 Kacheln der Kachel 4
        lda #1
        ldx #4
        ldy #6
        jsr tile_put                ; Laufzeit: A = Kachel, X = Spalte, Y = Zeile
        jmp *

; ---------------------------------------------------------------- Prüfroutinen (Aufruf durch vcc test, siehe tiles_*.vctest)
; clear_all: Bildschirm löschen (Zeichen = Leerzeichen $20, Farbe = $71 = Weiß, Helligkeit 7).  Verändert A, X.
;   Vier Stores je Durchlauf decken die 1000 Zellen ab: die Seiten +$000, +$100, +$200 und der überlappende Rest +$2E8..+$3E7.
clear_all:
        ldx #0
        lda #$20
.l:
        sta TILE_SCREEN + $0,x
        sta TILE_SCREEN + $100,x
        sta TILE_SCREEN + $200,x
        sta TILE_SCREEN + $2e8,x
        lda #$71
        sta TILE_COLORRAM + $0,x
        sta TILE_COLORRAM + $100,x
        sta TILE_COLORRAM + $200,x
        sta TILE_COLORRAM + $2e8,x
        lda #$20
        inx
        bne .l
        rts

; crc_area: CRC-16 (XMODEM: Polynom $1021, Startwert 0, bitweise, höchstes Bit zuerst) über AREA Byte ab (ZP_PTR).
;   Eingabe: ZP_PTR = Anfang, X = Maske, die auf jedes Byte per UND angewendet wird ($FF = alle Bits; kleinere Masken blenden z. B. Farbbits aus).
;   Ausgabe: ZP_CRC (Low, High).  Verändert A, X, Y, ZP_PTR+1, ZP_CNT, ZP_X.  Langsam, aber nur im Test benutzt.
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

; check(exp_scr, exp_col): Makro, das den Bildschirm und das Farb-RAM mit crc_area prüft und mit rts endet (steht daher am Ende einer t_-Routine).
;   Ausgabe in A: 0 = beide CRC stimmen mit den Erwartungen überein; 1 = Bildschirm falsch; 2 = Farb-RAM falsch.
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
        ldx #$ff
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

; ---- Prüfszenarien (je mit vcc test aufgerufen; Screen-Nummer 0 = lv1_id, 1 = lv2_id, 2 = lv3_id; Rückgabe A siehe check)
t_screen1:                          ; ganzer Screen mit der Nummer 1 (lv2_id) zeichnen und prüfen
        jsr clear_all
        screen_draw lv2_id
        check EXP_S1_SCR, EXP_S1_COL

t_screen1_nocheck:                  ; wie t_screen1, aber ohne CRC: der Test liest einzelne Zellen (Stichproben)
        jsr clear_all
        screen_draw lv2_id
        rts

t_draw_only:                        ; nur das Zeichnen (für die Zyklenmessung)
        screen_draw lv2_id
        rts

t_rows:                             ; Kachelzeilen 3..5 von Screen-Nummer 2 (lv3_id) mit screen_draw_rows (A = erste Zeile, X = Anzahl)
        jsr clear_all
        screen_select lv3_id
        lda #3
        ldx #3
        jsr screen_draw_rows
        check EXP_ROWS_SCR, EXP_ROWS_COL

t_macros:                           ; die Makros tile_draw, tile_draw_color und tile_fill
        jsr clear_all
        tile_draw 3, 5, 2
        tile_draw_color 1, 0, 0, 14
        tile_fill 4, FX, FY, 2, 2
        check EXP_MACRO_SCR, EXP_MACRO_COL

t_puts:                             ; tile_put zur Laufzeit: Kachel 5 an (2,3), Kachel 0 in der letzten Ecke, Kachel 3 oben links
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

t_map_get:                          ; X = Spalte, Y = Zeile (in Kacheln) von Screen-Nummer 1 (lv2_id) -> A = Kachelnummer (screen_map_get)
        stx ZP_X
        sty ZP_Y
        screen_select lv2_id
        ldx ZP_X
        ldy ZP_Y
        jmp screen_map_get

t_prop:                             ; X = Spalte, Y = Zeile von Screen-Nummer 1 (lv2_id) -> A = Eigenschaftenbyte der Kachel (tile_prop)
        stx ZP_X
        sty ZP_Y
        screen_select lv2_id
        ldx ZP_X
        ldy ZP_Y
        jmp tile_prop

; ---------------------------------------------------------------- Laufzeit der Bibliothek (einmal, hinter dem Programm)
; Enthält tile_put, screen_draw, screen_select_a usw. als Unterprogramme (Zeiger- oder SMC-Version je nach TILE_SMC).
        tiles_runtime
        .assert * <= $4000, "Programm überschreitet die Grenze des freien Speichers (Bildschirm, Display-Liste oder Karte)"
