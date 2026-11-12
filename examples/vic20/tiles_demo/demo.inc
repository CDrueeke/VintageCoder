; =====================================================================================================================
; demo.inc – tiles_demo für den VC-20 (Grundgerät): Kachel-Bibliothek #include <vic20/tiles.asm>, gemeinsamer Teil der vier Programme
; =====================================================================================================================
;
; ZWECK
;   Zeigt die Kachel-Bibliothek (data/sys/vic20/tiles.asm mit dem systemneutralen Teil data/sys/common/tiles.asm): Ein Kachelsatz „hg“ (6 Kacheln,
;   3x2 oder 2x2 Zeichen groß) und eine Screen-Bank „levels“ (3 Karten aus Kachelnummern) werden auf den Textbildschirm (22 x 23 Zeichen)
;   gezeichnet. Außerdem enthält die Datei die Prüfroutinen, die vcc test (tiles_demo_*.vctest) einzeln aufruft; ihr Ergebnis wird dort gegen
;   Referenzwerte geprüft. Diese Datei, data_*.inc und expect_*.inc sind ERZEUGT von tools/gen_tiles_demo.py (nicht von Hand ändern); die Daten sind
;   frei erfunden und zeigen kein Bild.
;
; DIE VIER PROGRAMME (tiles_demo_<Größe>_<Variante>.asm, je wenige Zeilen: Schalter festlegen, dann diese Datei einbinden)
;   DEMO_SET = 3  Kacheln 3 x 2 Zeichen (data_3x2.inc), Karten 7 x 11 Kacheln;   DEMO_SET = 2  Kacheln 2 x 2 Zeichen (data_2x2.inc), Karten 11 x 11 Kacheln
;   TILE_SMC = 0  Zeigerversion (screen_draw über Zeropage-Zeiger, ROM-tauglich);   TILE_SMC = 1  selbstmodifizierender Code (schneller, nur im RAM)
;   TILE_MAPRAM = $0360 (nur 3x2_smc): screen_select kopiert die gewählte Karte in diesen RAM-Puffer (Kassettenpuffer, 77 Byte), damit ein Spiel sie ändern dürfte
;
; ABLAUF DES PROGRAMMS (main)
;   Interrupts aus; Hintergrund und Rahmen schwarz ($900F = $08, VIC_COLOR); Bildschirm löschen (Leerzeichen, weiß); Screen 0 zeichnen; Kachel 3 an
;   Kachelposition (5, 2); Kachel 1 an (0, 0) in gelb (Farb-RAM-Wert 7); 2 x 2 Kacheln der Kachel 4 ab (FX, FY); Kachel 1 per Laufzeitaufruf
;   tile_put an (4, 6); dann Endlosschleife. Kachelpositionen zählen in Kacheln, nicht in Zeichen.
;
; HARDWARE UND SPEICHER (VC-20 ohne Erweiterung, Projekt tiles_demo.vcprj: memory unexpanded)
;   Bildschirm $1E00 (22 x 23 Zeichen), Farb-RAM $9600 (nur Bit 0..3; Bit 3 = Mehrfarbzeichen); die Adressen kommen aus <vic20/hardware.asm>
;   (SCREEN_RAM, COLOR_RAM). Die Bibliothek schreibt nur in diese beiden Bereiche, Bildschirm- und Zeichensatzzeiger des VIC-I bleiben auf den
;   Werten des KERNALs; das Programm setzt nur $900F. Programm ab $1001 (.basicstub, „10 SYS“), dahinter Daten und die Laufzeit der Bibliothek
;   (tiles_runtime); die .assert-Zeile am Ende hält alles unter $1E00. Die Zeichencodes der Kacheln sind Testmuster; hg_col ist die C64-Tabelle,
;   darum haben auf dem VIC-I die Kacheln 3, 4 und 5 (Werte 15, 13, 14) das Mehrfarbbit.
;   Zeropage: Bibliothek $F7-$FE (TILE_ZP); die Prüfroutinen unten brauchen $22-$29. $0340 (Kassettenpuffer) ist die Rücksprungfalle von vcc test.
;
; PRÜFVERFAHREN
;   Die Prüfroutinen zeichnen ein Szenario und bilden die CRC-16 (XMODEM) über den Bildschirm (506 Byte) und das Farb-RAM (506 Byte, Maske $0F) und
;   vergleichen sie mit den Referenzwerten EXP_* aus expect_*.inc, die der Generator aus seinem eigenen Abbild berechnet. Dazu kommen Stichproben
;   und Einzelaufrufe (tile_put, screen_map_get, tile_prop) in den .vctest-Dateien.
; =====================================================================================================================
.target vic20
.cpu 6502 { illegals = deny }
#include <vic20/tiles.asm>

; ZP für die Prüfroutinen (außerhalb der Kachelgruppe $F7-$FE; vcc test verbietet der Bibliothek $02-$21)
.const ZP_CRC = $22             ; CRC-16 (2 Byte)
.const ZP_PTR = $24             ; Zeiger auf die zu prüfenden Daten (2 Byte)
.const ZP_X   = $26             ; Merker für Spalte bzw. Maske (crc_area, t_map_get, t_prop)
.const ZP_Y   = $27             ; Merker für Zeile
.const ZP_CNT = $28             ; Zähler (2 Byte)
.const AREA   = TILE_COLS * TILE_ROWS      ; Zellen des Bildschirms = Länge von Bildschirm und Farb-RAM (22 * 23 = 506)

.org $1001                      ; BASIC-Start des Grundgeräts
.basicstub start                ; „10 SYS <start>“
start:  jmp main                ; Einsprung des SYS: über Daten und Prüfroutinen hinweg zu main

; ---------------------------------------------------------------- Daten und Bibliothek
; Daten je nach DEMO_SET: Kachelsatz hg_* und Screen-Bank levels_* (Exportformat des Kachel-Editors) und die erwarteten CRC-Werte EXP_*
.if DEMO_SET == 3
.include "data_3x2.inc"
.include "expect_3x2.inc"
.else
.include "data_2x2.inc"
.include "expect_2x2.inc"
.endif

        tiles_use hg_c0, hg_col, hg_props, hg_tw, hg_th, hg_count, levels_screens_lo, levels_screens_hi, levels_tiles_w, levels_tiles_h   ; Satz und Bank verbinden
.const FX = DEMO_FX             ; Ort des 2 x 2-Kachelblocks in main (Kachelposition), aus expect_*.inc
.const FY = DEMO_FY

; ---------------------------------------------------------------- Programm: zeigt Screen 0, dann ein paar Einzelkacheln
main:
        sei                        ; KERNAL-IRQ sperren (das Cursorblinken würde sonst Zeichen im Bild verändern)
        lda #$08
        sta VIC_COLOR              ; Hintergrund und Rahmen schwarz
        jsr clear_all
        screen_draw lv1_id          ; Screen 0 wählen und zeichnen
        tile_draw 3, 5, 2           ; Kachel 3 an Kachelposition (5, 2), nur Konstanten
        tile_draw_color 1, 0, 0, 7  ; Kachel 1 an (0, 0), aber in gelb statt der Kachelfarbe
        tile_fill 4, FX, FY, 2, 2   ; 2 x 2 Kacheln der Kachel 4
        lda #1
        ldx #4
        ldy #6
        jsr tile_put                ; Laufzeit: A = Kachel, X = Spalte, Y = Zeile
        jmp *                       ; fertig: stehen bleiben

; ---------------------------------------------------------------- Prüfroutinen (Aufruf durch vcc test, siehe tiles_*.vctest)
; vcc test ruft die Routinen einzeln auf; sie kehren mit RTS zurück (die mit check enden: A = Ergebnis) und verändern A, X, Y und die ZP_*-Zellen.
; clear_all: Bildschirm löschen (Leerzeichen $20) und Farb-RAM auf 1 (weiß). Die beiden Schreibbefehle je Durchlauf decken die Offsets 0..255 und 250..505 ab
;   (überlappen sich, zusammen alle 506 Zellen).
clear_all:
        ldx #0
        lda #$20
.l:
        sta TILE_SCREEN + $0,x
        sta TILE_SCREEN + $fa,x
        lda #$01
        sta TILE_COLORRAM + $0,x
        sta TILE_COLORRAM + $fa,x
        lda #$20
        inx
        bne .l
        rts

; crc_area: CRC-16 (XMODEM: Polynom $1021, Startwert 0, bitweise) über AREA Byte ab (ZP_PTR); X = Maske, die auf jedes Byte angewendet wird ($FF Bildschirm,
;   $0F Farb-RAM, dessen obere Bits unbestimmt sind) -> Ergebnis in ZP_CRC (Low), ZP_CRC + 1 (High). Verändert A, X, Y, ZP_PTR + 1, ZP_CNT, ZP_X.
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

; check(exp_scr, exp_col): A = 0, wenn Bildschirm und Farb-RAM die erwartete CRC haben; 1 = Bildschirm falsch, 2 = Farbe falsch.
;   Endet mit RTS, ist also das Ende der Prüfroutine, in die es eingesetzt wird.
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

t_screen1:                          ; ganzer Screen 1 (lv2_id = zweite Karte, Nummer 1) mit screen_draw, dann CRC-Prüfung
        jsr clear_all
        screen_draw lv2_id
        check EXP_S1_SCR, EXP_S1_COL

t_screen1_nocheck:                  ; wie t_screen1 ohne Prüfung: der Test liest danach einzelne Zellen (Stichproben)
        jsr clear_all
        screen_draw lv2_id
        rts

t_draw_only:                        ; nur das Zeichnen (für die Zyklenmessung)
        screen_draw lv2_id
        rts

t_rows:                             ; Kachelzeilen 3..5 der dritten Karte (lv3_id = Nummer 2) mit screen_draw_rows (A = erste Zeile, X = Anzahl)
        jsr clear_all
        screen_select lv3_id
        lda #3
        ldx #3
        jsr screen_draw_rows
        check EXP_ROWS_SCR, EXP_ROWS_COL

t_macros:                           ; die Makros tile_draw, tile_draw_color (Farbe 14) und tile_fill
        jsr clear_all
        tile_draw 3, 5, 2
        tile_draw_color 1, 0, 0, 14
        tile_fill 4, FX, FY, 2, 2
        check EXP_MACRO_SCR, EXP_MACRO_COL

t_puts:                             ; drei Laufzeitaufrufe tile_put: Kachel 5 an (2, 3), Kachel 0 an der letzten Kachel (rechts unten), Kachel 3 an (0, 0)
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

t_map_get:                          ; X = Spalte, Y = Zeile (in Kacheln) der Karte lv2_id (Nummer 1) -> A = Kachelnummer
        stx ZP_X
        sty ZP_Y
        screen_select lv2_id
        ldx ZP_X
        ldy ZP_Y
        jmp screen_map_get

t_prop:                             ; X = Spalte, Y = Zeile der Karte lv2_id -> A = Eigenschaftenbyte dieser Kachel (props)
        stx ZP_X
        sty ZP_Y
        screen_select lv2_id
        ldx ZP_X
        ldy ZP_Y
        jmp tile_prop

; ---------------------------------------------------------------- Laufzeit der Bibliothek (einmal, hinter dem Programm)
        tiles_runtime               ; Unterprogramme der Bibliothek (tile_put, screen_draw_all, ...), nur einmal
        .assert * <= $1e00, "Programm überschreitet die Grenze des freien Speichers (Bildschirm, Display-Liste oder Karte)"
