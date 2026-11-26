; =====================================================================================================================
; data.inc - Tabellen, zur Uebersetzungszeit erzeugt
; =====================================================================================================================
; Herkunft: Aufbau wie examples/c64/troxx/data.inc (Stand Commit d173cb4a) und examples/vic20/troxx/data.inc (Stand Commit 4697d074), angepasst an den Plus/4.
;   Gleich geblieben sind BITMASK, DIR_DX, DIR_DY, COUNT, PROB, die Tempotabelle (Bilder je Schritt, je Norm), START_X und START_Y.
; Inhalt: Zeilenanfaenge des Bildschirms (ROW_LO/ROW_HI) und der Schattenkopie (SH_LO/SH_HI), Schablonen der Gitterzeilen (TPL, ROW_TPL), Richtungen, Farben und
;         Bloecke der Fuellungen, vorgefuellte Bloecke des zweiten Layouts, Farbbaender, Gegnerdaten, Tempo je Norm.  Konstanten und Namen stehen in main.asm.
; =====================================================================================================================

; Bildschirmzeilen 0..24: Anfang im Bildschirmspeicher, erst alle Low-Bytes, dann alle High-Bytes
ROW_LO: .lohitable SCR_ROWS, SCREEN_RAM + SCR_COLS * i
.const ROW_HI = ROW_LO + SCR_ROWS

; Schablonen der Gitterzeilen zu je 33 Zeichen: 0 Oberkante, 1 Mittellinie (waagerechte Linie im Inneren), 2 Unterkante, 3 Innenzeile.
; Knoten: links/Mitte/rechts je Zeilenart: Ecke oder T-Stueck oder Kreuzung
.var NODE = [CH_TL, CH_TDOWN, CH_TR,  CH_TRIGHT, CH_CROSS, CH_TLEFT,  CH_BL, CH_TUP, CH_BR]
.const TPL_INNER = 3 * GRID_W           ; Anfang der Innenschablone in TPL
TPL:
.for t in range(4)
    .for gx in range(GRID_W)
        .if t == 3
            .if gx % (BOX_W + 1) == 0
                .byte CH_VLINE
            .else
                .byte CH_SPACE
            .endif
        .elseif gx % (BOX_W + 1) == 0
            .if gx == 0
                .byte NODE[t * 3]
            .elseif gx == GRID_W - 1
                .byte NODE[t * 3 + 2]
            .else
                .byte NODE[t * 3 + 1]
            .endif
        .else
            .byte CH_HLINE
        .endif
    .endfor
.endfor

; Gitterzeile gy -> Anfang ihrer Schablone in TPL
ROW_TPL:
.for gy in range(GRID_H)
    .if gy == 0
        .byte 0
    .elseif gy == GRID_H - 1
        .byte 2 * GRID_W
    .elseif gy % (BOX_H + 1) == 0
        .byte GRID_W
    .else
        .byte TPL_INNER
    .endif
.endfor

; Richtungen (Dir): Maske der Eingabe, Schritt in gx und gy
DIR_BIT: .byte IN_UP, IN_DOWN, IN_LEFT, IN_RIGHT
DIR_DX:  .byte 0, 0, -1, 1
DIR_DY:  .byte -1, 1, 0, 0

; Bitmasken 1, 2, 4, ... 128: Bit n eines Bytes; die ersten vier dienen zugleich als Masken der Richtungen (Dir)
BITMASK: .byte 1, 2, 4, 8, 16, 32, 64, 128

; 8 * (gy div 4) je Gitterzeile; dazu kommt gx div 4: ergibt b0 = 8 r + c (Nummer des waagerechten Segments H(r, c) und des Rechtecks B(r, c))
SEGBASE:
.for gy in range(GRID_H)
    .byte (gy / (BOX_H + 1)) * BOX_COLS
.endfor

; Gegner: Startknoten (oben rechts, oben links, unten rechts, oben Mitte); Gegnerzahl und Laeufer-Wahrscheinlichkeit (von 256) je Stufe 1..9 (ab 10 wie 9)
START_X: .byte GRID_W - 1, 0, GRID_W - 1, 16
START_Y: .byte 0, 0, GRID_H - 1, 0
COUNT:   .byte 1, 2, 2, 3, 3, 3, 4, 4, 4
PROB:    .byte 128, 144, 160, 176, 192, 200, 210, 220, 230
; Typ je Gegnernummer: mit Waechter Laeufer, Streuner, Waechter, Laeufer; sonst Laeufer, Streuner im Wechsel
.if EXTRA_GUARD
KIND_OF:  .byte KIND_RUNNER, KIND_WANDER, KIND_GUARD, KIND_RUNNER
.else
KIND_OF:  .byte KIND_RUNNER, KIND_WANDER, KIND_RUNNER, KIND_WANDER
.endif
KIND_COL: .byte COL_RED, COL_PURPLE, COL_CYAN    ; Laeufer rot, Streuner violett, Waechter cyan
ITEM_COL: .byte COL_YELLOW, COL_LTBLUE           ; Stern, Eisblock

; Zeilen der Schattenkopie: Anfang je Gitterzeile gy (33 Byte je Zeile), erst alle Low-Bytes, dann alle High-Bytes
SH_LO: .lohitable GRID_H, SHCH + GRID_W * i
.const SH_HI = SH_LO + GRID_H

; Fuellungen: erstes Zeichen des Musters 0..3 (je neun Zeichen) und 3 * k
PAT_BASE: .byte CH_FILL0, CH_FILL0 + 9, CH_FILL0 + 18, CH_FILL0 + 27
K3:       .byte 0, 3, 6

; Farbton der Fuellungen nach Rechteckzeile (Farbtoene 0..7, weil die Mehrfarben-Farbe nur drei Bit hat): gruen, cyan, violett, rot, gelb
BOX_TONE: .byte 5, 3, 4, 2, 7

; Farbbaender je Rechteckzeile: Mehrfarben-Hintergrund 1 (dunkler Ton der Fuellung, Schatten) und 2 (heller Ton, Glanz); die Zeile 0..4 setzt der Rasterinterrupt
; ab der waagerechten Linie ihrer Oberkante; mit FX_RASTER = 0 gilt dauernd der Wert der Zeile 2
BAND_BG1: .byte $15, $1c, $14, $12, $19
BAND_BG2: .byte $5f, $73, $74, $7b, $77

; vorgefuellte Rechtecke des zweiten Layouts (Nummer b = 8 r + c): (1,1) (1,6) (2,3) (2,4) (3,1) (3,6)
PRE_B: .byte 9, 14, 19, 20, 25, 30

; Tempo: Bilder je Schritt fuer Tempo 1..9; PAL (50 Bilder/s) dann NTSC (60 Bilder/s, Werte mal 1,2, gerundet)
TEMPO:
.var tempo_pal = [12, 10, 9, 8, 7, 6, 5, 4, 3]
.for v in tempo_pal
    .byte v
.endfor
.for v in tempo_pal
    .byte round(v * 1.2)
.endfor
