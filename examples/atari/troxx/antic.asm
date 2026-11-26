; =====================================================================================================================
; antic.asm - Anzeige: Zeichensatz, Display-Listen, Farbtabellen, Bildtakt (ANTIC, GTIA)
; =====================================================================================================================
; ZWECK     Richtet den Bildschirm ein und stellt ihn je Zustand um: Spielbild (24 Zeilen, Seitenleiste) und Titelbild.  Der Display-List-
;           Interrupt (dli.asm) setzt die Farben je Textzeile aus den Tabellen RC_BK, RC_P1, RC_P2, RC_H; die Tabellen dieses Moduls sind die
;           Vorlagen (GAME_*, TITLE_*), die use_game / use_title nach RC_* kopieren.
; ANZEIGE   ANTIC-Modus 4 (Mehrfarbtext 40 x 24, jedes Zeichen 4 Pixelpaare breit, 8 Zeilen hoch, vier Farben) fuer Gitter, Seitenleiste und Logo;
;           ANTIC-Modus 2 (Hires-Text, ROM-Schrift) fuer Kopfzeile, Meldungszeile und die Texte des Titelbilds.  Beide Modi teilen sich einen
;           Zeichensatz (CHBASE): Codes 0..63 Kopie der ROM-Schrift (Modus 2), Codes 64..127 eigene Grafik und Seitenschrift (gfx.inc, Modus 4).
;           Zeichensatz-RAM an CHARSET, Bildschirmspeicher SCR (40 Byte je Zeile, in einem 4-KB-Block), je Zeile Modus und Farben nach Tabellen.
; FARBEN    Im Modus 4 gelten COLPF0..3 und COLBK fuer den ganzen Bildschirm; der DLI schaltet sie je Zeile um (Farbverlaeufe).  Pixelwert 01 =
;           COLPF0 (unberuehrte Linie), 10 = COLPF1 (abgefahrene Linie, Lichtkante), 11 = COLPF2 (Zeilenfarbe) bzw. mit gesetztem Bit 7 im
;           Bildschirmcode COLPF3.  Im Modus 2 ist COLPF2 der Hintergrund und die Schrift hat dessen Farbton mit der Helligkeit von COLPF1.
; BILDRAND  Die ersten 24 Rasterzeilen (Leerzeilen) und die 16 Zeilen unter der letzten Textzeile zeigen animierte Farbbalken: der DLI setzt dort
;           COLBK je Rasterzeile aus der Tabelle BAR (dli.asm).
; =====================================================================================================================

antic: {

; init: einmalige Einrichtung: Norm erkennen, Zeichensatz aufbauen, Player/Missiles vorbereiten, Interrupts setzen.  Veraendert A, X, Y.
init:
        lda #0
        sta SDMCTL
        sta NMIEN                       ; keine Interrupts, solange wir umbauen
        sta DMACTL
        lda GTIA_PAL                    ; Norm: Bit 1..3 = 0 -> PAL, sonst NTSC
        and #$0e
        bne .ntsc
        lda #F_PAL
        sta FLAGS
.ntsc:  jsr setbar
        ldx #0
.rom:   lda $e000,x                     ; Schrift (Codes 0..63) aus dem Zeichen-ROM des Rechners
        sta CHARSET,x
        lda $e100,x
        sta CHARSET + $100,x
        lda GFX_CHARS,x                 ; eigene Zeichen 64..127
        sta CHARSET + $200,x
        lda GFX_CHARS + $100,x
        sta CHARSET + $300,x
        inx
        bne .rom
        lda #>CHARSET
        sta CHBAS
        lda #>(PM_BASE)
        sta PMBASE
        lda #3
        sta GRACTL                      ; Player und Missiles freigeben
        lda #PM_PRIOR                   ; Prioritaet (Schatten): Spieler vor dem Spielfeld, Missiles als fuenfter Spieler (Farbe COLPF3)
        sta GPRIOR
        lda #PM_WIDTH_P
        ldx #3
.sz:    sta SIZEP0,x                    ; Breite der Spieler 0..3
        dex
        bpl .sz
        lda #PM_WIDTH_M
        sta SIZEM
        jsr colors_pm
        lda FLAGS
        and #F_PAL
        beq .pn
        lda #COL_LINE_PF0
        bne .pf0
.pn:    lda #COL_LINE_PF0_N
.pf0:   sta COLOR0                      ; COLPF0: unberuehrte Linie (Schatten, das Betriebssystem kopiert ihn jedes Bild)
        lda #<dli_top
        sta VDSLST
        lda #>dli_top
        sta VDSLST + 1
        lda #<vbi
        sta VVBLKD
        lda #>vbi
        sta VVBLKD + 1
        jsr use_game
.if FX_DLI
        lda #$c0
.else
        lda #$40                        ; ohne DLI: nur der Bildruecklauf-Interrupt (eine Farbe je Bild)
.endif
        sta NMIEN                       ; DLI und Bildruecklauf-Interrupt an
        rts

; setbar: Farbbalken-Tabelle BAR nach BARRAM kopieren (NTSC: Blauwerte 7x -> 8x, weil Farbton 7 im NTSC-Farbkreis purpur ist).  Veraendert A, X.
setbar: ldx #127
.b:     lda BAR,x
        jsr ntsc_adj
        sta BARRAM,x
        dex
        bpl .b
        rts

; ntsc_adj: A = Farbwert; im NTSC-Betrieb wird der Farbton 7 zu 8 (dunkelblau statt purpur), sonst unveraendert.  Veraendert A (und Y).
ntsc_adj:
        tay
        lda FLAGS
        and #F_PAL
        bne .keep
        tya
        and #$f0
        cmp #$70
        bne .keep
        tya
        clc
        adc #$10
        rts
.keep:  tya
        rts

; ntsc_fix: die Farbtabellen RC_BK und RC_P2 an die Norm anpassen (siehe ntsc_adj).  Veraendert A, X, Y.
ntsc_fix:
        ldx #23
.f:     lda RC_BK,x
        jsr ntsc_adj
        sta RC_BK,x
        lda RC_P2,x
        jsr ntsc_adj
        sta RC_P2,x
        dex
        bpl .f
        rts

; colors_pm: Farben der Spieler (Schatten PCOLR0..3) und Grundfarben.  Veraendert A.
colors_pm:
        lda #COL_PLAYER
        sta PCOLR0
        lda #COL_RUNNER
        sta PCOLR0 + 1
        lda #COL_WANDER
        sta PCOLR0 + 2
        lda #COL_GUARD
        sta PCOLR0 + 3
        lda #COL_RUNNER
        sta PF3COL
        rts

; wait_frame: auf das naechste Bild warten (RTCLOK zaehlt im Bildruecklauf).  Veraendert A.
wait_frame:
        lda RTCLOK
.w:     cmp RTCLOK
        beq .w
        rts

; set_dl: A/X = Adresse der Display-Liste (Schatten; wirkt im naechsten Bild), Y = Modus fuer den DLI (siehe dli.asm).  Veraendert A.
set_dl: sta SDLSTL
        stx SDLSTL + 1
        rts

; use_game: Spielbild: Display-Liste, Farbtabellen, Bildschirm einschalten.  Veraendert A, X.
use_game:
        lda #SM_GAME
        sta SCRMODE
        lda #$ff
        sta SCRROW                      ; keine Feinscroll-Zeile
        ldx #<DLG
        lda #>DLG
        jsr set_screen
        ldx #23
.c1:    lda GAME_BK,x
        sta RC_BK,x
        lda GAME_P1,x
        sta RC_P1,x
        lda GAME_P2,x
        sta RC_P2,x
        lda #0
        sta RC_H,x
        dex
        bpl .c1
        jmp ntsc_fix

; use_title: Titelbild: Display-Liste und Farbtabellen.  Veraendert A, X.
use_title:
        lda #SM_TITLE
        sta SCRMODE
        lda #7
        sta SCRROW                      ; Zeile 7 ist die Lauftextzeile (Feinscrollen)
        ldx #<DLT
        lda #>DLT
        jsr set_screen
        ldx #23
.c2:    lda TITLE_BK,x
        sta RC_BK,x
        lda TITLE_P1,x
        sta RC_P1,x
        lda TITLE_P2,x
        sta RC_P2,x
        lda #0
        sta RC_H,x
        dex
        bpl .c2
        jmp ntsc_fix

; set_screen: Liste X (niedrig) / A (hoch) einsetzen und die Anzeige einschalten (Schatten von DMACTL: Liste, Player, Missiles, Einzelzeilen, normales Feld).
set_screen:
        stx SDLSTL
        sta SDLSTL + 1
        lda #$3e
        sta SDMCTL
        rts

; clear: Bildschirmspeicher (960 Byte) mit dem Leerzeichen des Modus 2 / 4 (Code 0) loeschen.  Veraendert A, X.
clear:  lda #0
        ldx #0
.c:     sta SCR,x
        sta SCR + $100,x
        sta SCR + $200,x
        cpx #$c0
        bcs .n
        sta SCR + $300,x
.n:     inx
        bne .c
        rts

; hide_pm: alle Spieler und Missiles verbergen und ihren Speicher loeschen.  Veraendert A, X.
hide_pm:
        lda #0
        ldx #7
.h:     sta HPOSP0,x
        dex
        bpl .h
        tax
.m:     sta PM_BASE + $300,x
        sta PM_BASE + $400,x
        sta PM_BASE + $500,x
        sta PM_BASE + $600,x
        sta PM_BASE + $700,x
        inx
        bne .m
        rts
}

; ---------------------------------------------------------------------------------------------------------------------
; Display-Listen
; ---------------------------------------------------------------------------------------------------------------------
; Befehle: $70 acht Leerzeilen, $F0 dasselbe mit DLI; $C2 / $82 Modus 2 mit DLI (mit / ohne Adresse LMS $40), $84 Modus 4 mit DLI, $D2 Modus 2 mit
;   Adresse, Feinscrollen ($10) und DLI; $41 Sprung und auf das Bildende warten.  Alle 24 Textzeilen tragen das DLI-Bit.
DLG:    .byte $f0, $70, $70             ; 24 Leerzeilen: der erste DLI faehrt die Farbbalken des oberen Rands
        .byte $c2, <SCR, >SCR           ; Zeile 0: Kopfzeile (Modus 2)
.for r in range(22)
        .byte $84                       ; Zeilen 1..22: Gitter und Seitenleiste (Modus 4)
.endfor
        .byte $82                       ; Zeile 23: Meldungszeile (Modus 2)
        .byte $41, <DLG, >DLG

DLT:    .byte $f0, $70, $70
        .byte $c2, <SCR, >SCR           ; Zeile 0 (Modus 2)
.for r in range(5)
        .byte $84                       ; Zeilen 1..5: Logo (Modus 4)
.endfor
        .byte $82                       ; Zeile 6
        .byte $d2, <SCRL, >SCRL         ; Zeile 7: Lauftext mit Feinscrollen (Puffer SCRL, 48 Zeichen)
        .byte $c2, <(SCR + 8 * 40), >(SCR + 8 * 40)   ; Zeile 8: wieder aus dem Bildschirmspeicher
.for r in range(15)
        .byte $82                       ; Zeilen 9..23
.endfor
        .byte $41, <DLT, >DLT

; ---------------------------------------------------------------------------------------------------------------------
; Farbvorlagen je Zeile (24): COLBK, COLPF1, COLPF2.  Spiel: Hintergrund dunkelblau mit Verlauf; COLPF1 gelb (abgefahrene Linien), in den Textzeilen
;   0 und 23 die Helligkeit der Schrift; COLPF2: Zeilenfarbe der Fuellungen (je Rechteckzeile ein Farbton, innen von dunkel nach hell) und der
;   Seitenschrift, in den Textzeilen der Hintergrund.
; ---------------------------------------------------------------------------------------------------------------------
GAME_BK:
.for r in range(24)
        .byte $70 + ((r * 5 / 24) * 2)
.endfor
GAME_P1:
.for r in range(24)
    .if r == 0 || r == 23
        .byte $0e
    .else
        .byte COL_DONE_LINE
    .endif
.endfor
.var BOXHUE = [$c0, $20, $50, $40, $90]    ; Farbton je Rechteckzeile: Gruen, Orange, Purpur, Rot, Hellblau
GAME_P2:
.for r in range(24)
    .if r == 0
        .byte $70
    .elseif r == 23
        .byte $74 + 0
    .else
        .var gy = r - 1
        .var hue = BOXHUE[min(gy / 4, 4)]
        .if gy % 4 == 0 || gy == 21
            .byte hue + 14
        .else
            .byte hue + 8 + 2 * (gy % 4)
        .endif
    .endif
.endfor

TITLE_BK:
.for r in range(24)
        .byte $70 + ((r * 5 / 24) * 2)
.endfor
TITLE_P1:
.for r in range(24)
    .if r >= 1 && r <= 5
        .byte COL_DONE_LINE
    .else
        .byte $0e
    .endif
.endfor
TITLE_P2:
        .byte $70                                       ; 0
        .byte $1e, $2e, $3e, $4e, $5e                   ; 1..5 Logo (Farbzyklus ueberschreibt)
        .byte $72                                       ; 6
        .byte $a2                                       ; 7 Lauftext (Hintergrund dunkles Tuerkis)
        .byte $72                                       ; 8
        .byte $72, $72, $72, $72, $72, $72              ; 9..14 Einstellungen
        .byte $12                                       ; 15 "FIRE = START" (dunkles Gold)
        .byte $72                                       ; 16
        .byte $c2                                       ; 17 "HIGH SCORES" (dunkles Gruen)
        .byte $72, $72, $72, $72, $72                   ; 18..22 Liste
        .byte $72                                       ; 23 Status
