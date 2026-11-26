; =====================================================================================================================
; data.inc - Tabellen, zur Uebersetzungszeit erzeugt
; =====================================================================================================================
; Herkunft: Aufbau wie examples/vic20/troxx/data.inc (Stand Commit 4697d074) und examples/c64/troxx/data.inc (Stand Commit d173cb4a), angepasst an
;   den Atari.  Gleich geblieben sind TPL, ROW_TPL, DIR_BIT, DIR_DX, DIR_DY, BITMASK, SEGBASE, COUNT, PROB und die Tempotabelle (Bilder je Schritt,
;   je Norm).
; Inhalt: Zeilenanfaenge des Bildschirms (ROW_LO/ROW_HI) und des Spielfeldzustands (GST_LO/GST_HI), Schablonen der Gitterzeilen (TPL, ROW_TPL),
;         Richtungen, Fuellzeichen, Gegnerdaten, Farben, Tempo je Norm.  Konstanten und Namen stehen in main.asm.
; =====================================================================================================================

; Bildschirmzeilen 0..23: Anfang im Bildschirmspeicher, erst alle Low-Bytes, dann alle High-Bytes
ROW_LO: .lohitable SCR_ROWS, SCR + SCR_COLS * i
.const ROW_HI = ROW_LO + SCR_ROWS

; Zeilenanfaenge des Spielfeldzustands GRIDST (33 Byte je Gitterzeile)
GST_LO: .lohitable GRID_H, GRIDST + GRID_W * i
.const GST_HI = GST_LO + GRID_H

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
KIND_COL: .byte COL_RUNNER, COL_WANDER, COL_GUARD

; Fuellzeichen eines Rechtecks: je Innenzeile drei Zeichen (Rahmen mit Abschraegung; $FF = Mitte, dort steht eines der vier Muster FILLP)
FILLB:  .byte CH_FTL, CH_FT, CH_FTR
        .byte CH_FL, $ff, CH_FR
        .byte CH_FBL, CH_FB, CH_FBR
FILLP:  .byte CH_FILL0, CH_FILL1, CH_FILL2, CH_FILL3
; Zeichen abgefahrener Linien: waagerecht, senkrecht (Knoten behalten ihr Zeichen)
LINECH_D: .byte CH_HLINE_D, CH_VLINE_D
ITEM_CH:  .byte CH_STAR, CH_ICE
; gelbe Knoten: Grundzeichen -> Zeichen des gelben Knotens (Ecken benutzen das T-Stueck mit zwei Armen, die ein kleines Stueck nach aussen ragen)
NODE_FROM: .byte CH_CROSS, CH_TDOWN, CH_TUP, CH_TRIGHT, CH_TLEFT, CH_TL, CH_TR, CH_BL, CH_BR
NODE_TO:   .byte CH_CROSS_D, CH_TDOWN_D, CH_TUP_D, CH_TRIGHT_D, CH_TLEFT_D, CH_TDOWN_D, CH_TDOWN_D, CH_TUP_D, CH_TUP_D

; Tempo: Bilder je Schritt fuer Tempo 1..9; PAL (50 Bilder/s) dann NTSC (60 Bilder/s, Werte mal 1,2, gerundet)
TEMPO:
.var tempo_pal = [12, 10, 9, 8, 7, 6, 5, 4, 3]
.for v in tempo_pal
    .byte v
.endfor
.for v in tempo_pal
    .byte round(v * 1.2)
.endfor
