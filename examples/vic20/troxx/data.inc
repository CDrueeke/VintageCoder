; =====================================================================================================================
; data.inc - Tabellen, zur Uebersetzungszeit erzeugt
; =====================================================================================================================
; Inhalt: Zeilenanfaenge des Bildschirms (ROW_LO/ROW_HI), Schablonen der Gitterzeilen (TPL, ROW_TPL), Richtungen, Fuellfarben, Bildlage je Norm,
;         Tempo je Norm, Tonleiter je Norm.  Konstanten und Namen stehen in main.asm.
; =====================================================================================================================

; Bildschirmzeilen 0..22: Anfang im Bildschirmspeicher, erst alle Low-Bytes, dann alle High-Bytes
; Low-Byte der Zeilenanfaenge des Bildschirms (Zeile 0..22); das High-Byte ist $1E, ab Zeile 12 $1F
ROW_LO:
.for r in range(SCR_ROWS)
    .byte <(SCREEN_RAM + SCR_COLS * r)
.endfor

; Gitterzeile gy -> Art: 0 Oberkante, 6 Mittellinie (waagerechte Linie im Inneren), 12 Unterkante (Abstand in NODETAB), $FF Innenzeile
ROWKIND:
.for gy in range(GRID_H)
    .if gy == 0
        .byte 0
    .elseif gy == GRID_H - 1
        .byte 12
    .elseif gy % (BOX_H + 1) == 0
        .byte 6
    .else
        .byte $ff
    .endif
.endfor

; Knotenzeichen je Zeilenart (3 Reihen zu 6 Spalten gx div 4): Ecken, T-Stuecke, Kreuzungen (Codes 29 30 / 34 / 36 37 / 28 / 31 33 / 35)
NODETAB: .byte 29, 34, 34, 34, 34, 30,  36, 28, 28, 28, 28, 37,  31, 35, 35, 35, 35, 33

; Richtungen (Dir): Maske der Eingabe, Schritt in gx und gy
DIR_BIT: .byte IN_UP, IN_DOWN, IN_LEFT, IN_RIGHT
DIR_DX:  .byte 0, 0, -1, 1
DIR_DY:  .byte -1, 1, 0, 0

; Masken der Richtungen (Dir): Bit n
BITMASK: .byte 1, 2, 4, 8

; 5 * (gy div 5) je Gitterzeile; dazu kommt gx div 4: ergibt b0 = 5 r + c (Nummer des waagerechten Segments H(r, c) und des Rechtecks B(r, c))
SEGBASE:
.for gy in range(GRID_H)
    .byte (gy / (BOX_H + 1)) * BOX_COLS
.endfor

; Fuellfarben der Rechtecke nach Zeile
BOX_COLOR: .byte COL_GREEN, COL_CYAN, COL_PURPLE, COL_RED

; Bildlage ($9000, $9001) nach Norm: Eintrag 0 = NTSC, 1 = PAL (Werte der Einschaltbelegung des KERNALs)
ORIGIN_X: .byte $05, $0c
ORIGIN_Y: .byte $19, $26

; Tempo: Bilder je Schritt fuer Tempo 1..9; PAL (50 Bilder/s) dann NTSC (60 Bilder/s, Werte mal 1,2, gerundet)
TEMPO:
.var tempo_pal = [12, 10, 9, 8, 7, 6, 5, 4, 3]
.for v in tempo_pal
    .byte v
.endfor
.for v in tempo_pal
    .byte round(v * 1.2)
.endfor

; Tonleiter des Sopran-Registers (Notennummer 1..8 = C5 D5 E5 F5 G5 A5 H5 C6), PAL dann NTSC.
; Frequenz = Takt / (64 * (127 - Wert)) laut Programmierhandbuch; Bit 7 = Stimme ein.
.var notes = [523.25, 587.33, 659.26, 698.46, 783.99, 880.0, 987.77, 1046.5]
SCALE:
.for f in notes
    .byte $80 | (127 - round(1108405 / (64 * f)))
.endfor
.for f in notes
    .byte $80 | (127 - round(1022727 / (64 * f)))
.endfor
