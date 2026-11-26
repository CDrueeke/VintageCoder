; =====================================================================================================================
; data.inc - Tabellen, zur Uebersetzungszeit erzeugt
; =====================================================================================================================
; Herkunft: Aufbau wie examples/c64/troxx/data.inc (Stand Commit 4697d074 der Referenz); angepasst an das MEGA65-Gitter (14 x 7 Rechtecke, Innenraum
;   4 x 5, 71 x 43 Zellen) und an den 80 x 50-Bildschirm.  Gleich geblieben: BITMASK, DIR_DX, DIR_DY.
; Inhalt: Zeilenanfaenge (Bildschirm, Schattengitter), Teilertabellen, Kachelform je Zelle, Richtungen, Fuellfarben, Spritelage, Gegnerdaten, Tempo.
; =====================================================================================================================

; Zeilenanfaenge des Bildschirms: Zeile * 160 Byte (80 Zellen zu je 2 Byte), erst alle Low-Bytes, dann alle High-Bytes
ROW160_LO: .lohitable 50, i * 160
.const ROW160_HI = ROW160_LO + 50

; Zeilenanfaenge des Schattengitters CELL (Gitterzeile gy * 80 Byte, ein Byte je Zelle)
ROW_LO: .lohitable GRID_H, CELL + 80 * i
.const ROW_HI = ROW_LO + GRID_H

; Teiler: gx mod 5 und gx div 5 (Spalten), gy mod 6 und gy div 6 (Zeilen)
MOD5: .for gx in range(GRID_W)
    .byte gx % (BOX_W + 1)
.endfor
DIV5: .for gx in range(GRID_W)
    .byte gx / (BOX_W + 1)
.endfor
MOD6: .for gy in range(GRID_H)
    .byte gy % (BOX_H + 1)
.endfor
; 14 * (gy div 6): Nummer des ersten Rechtecks der Zeile (SEGBASE wie in der Referenz, mit BOX_COLS = 14)
SEGBASE: .for gy in range(GRID_H)
    .byte (gy / (BOX_H + 1)) * BOX_COLS
.endfor

; 6 * (gy div 6): Zeile der oberen waagerechten Linie des Rechtecks (fuer senkrechte Segmente)
SEGROW: .for gy in range(GRID_H)
    .byte (gy / (BOX_H + 1)) * (BOX_H + 1)
.endfor

; Fortschrittsbalken: volle Zellen (von 22) bei n gefuellten Rechtecken (0..98)
BAR_N: .for n in range(NBOX + 1)
    .byte (n * 22) / NBOX
.endfor

; Kachelform je Zelle: Zeilenart rk (0 oben, 1 waagerechte Linie, 2 unten, 3 Innenzeile) und Spaltenart ck (0 links, 1 senkrechte Linie, 2 rechts, 3 innen)
;   -> Form (Nummer in assets.inc: 0 waagerecht, 1 senkrecht, 2 Kreuz, 3 Ecke oben links, 4 oben rechts, 5 unten links, 6 unten rechts, 7 T nach unten,
;   8 T nach oben, 9 T nach rechts, 10 T nach links; 255 = nichts)
SHAPE: .byte 3, 7, 4, 0
       .byte 9, 2, 10, 0
       .byte 5, 8, 6, 0
       .byte 1, 1, 1, 255
ROWKIND: .for gy in range(GRID_H)
    .if gy == 0
        .byte 0
    .elseif gy == GRID_H - 1
        .byte 2
    .elseif gy % (BOX_H + 1) == 0
        .byte 1
    .else
        .byte 3
    .endif
.endfor
COLKIND: .for gx in range(GRID_W)
    .if gx == 0
        .byte 0
    .elseif gx == GRID_W - 1
        .byte 2
    .elseif gx % (BOX_W + 1) == 0
        .byte 1
    .else
        .byte 3
    .endif
.endfor

; Richtungen (Dir): Maske der Eingabe, Schritt in gx und gy
DIR_BIT: .byte IN_UP, IN_DOWN, IN_LEFT, IN_RIGHT
DIR_DX:  .byte 0, 0, -1, 1
DIR_DY:  .byte -1, 1, 0, 0

; Bitmasken 1, 2, 4, ... 128: Bit n eines Bytes; die ersten vier dienen zugleich als Masken der Richtungen (Dir)
BITMASK: .byte 1, 2, 4, 8, 16, 32, 64, 128

; Gegner: Startknoten (oben rechts, oben links, oben Mitte, Mitte links, Mitte rechts; nie unten, dort starten die Figuren); Gegnerzahl und Laeufer-Wahrscheinlichkeit (von 256) je Stufe 1..9
START_X: .byte GRID_W - 1, 0, 35, 0, GRID_W - 1
START_Y: .byte 0, 0, 0, 24, 24
COUNT:   .byte 2, 2, 3, 3, 4, 4, 5, 5, 5
PROB:    .byte 128, 144, 160, 176, 192, 200, 210, 220, 230
; Typ je Gegnernummer: mit Waechter Laeufer, Streuner, Waechter, Laeufer, Streuner; sonst Laeufer, Streuner im Wechsel
.if EXTRA_GUARD
KIND_OF:  .byte KIND_RUNNER, KIND_WANDER, KIND_GUARD, KIND_RUNNER, KIND_WANDER
.else
KIND_OF:  .byte KIND_RUNNER, KIND_WANDER, KIND_RUNNER, KIND_WANDER, KIND_RUNNER
.endif
ENMASK:   .byte $03, $07, $0f, $1f, $3f ; Sprite-Freigabe: zwei Spieler und n Gegner (Sprites 0, 1 Spieler; 2..6 Gegner)

; Fuellfarbe der Rechtecke nach Rechteckzeile r (0..6): Nummer der Kachelfarbe in assets.inc
BOX_COLOR: .byte 0, 1, 2, 3, 4, 5, 6

; Lage der Sprites: Gitterzelle (gx, gy) -> Bildpunkt links oben (640 x 400): x = 8 (gx + GRID_COL0), y = 8 gy; Sprite-Koordinaten X' = x + 79 - 4 (16 x 16
;   mittig auf der 8 x 8-Zelle: 4 Punkte nach links und oben), Y' = y + 107 - 4.  Hier nur die Punkte; die Addition macht sprites.asm.
XLO:
.for gx in range(GRID_W)
    .byte (8 * (gx + GRID_COL0) + 79 - 4) & 255
.endfor
XHI:
.for gx in range(GRID_W)
    .byte (8 * (gx + GRID_COL0) + 79 - 4) >> 8
.endfor
YLO:
.for gy in range(GRID_H)
    .byte (8 * (gy + GRID_ROW0) + 107 - 4) & 255
.endfor
YHI:
.for gy in range(GRID_H)
    .byte (8 * (gy + GRID_ROW0) + 107 - 4) >> 8
.endfor

; Tempo: Bilder je Schritt fuer Tempo 1..9; PAL (50 Bilder/s) dann NTSC (60 Bilder/s, Werte mal 1,2, gerundet)
TEMPO:
.var tempo_pal = [12, 10, 9, 8, 7, 6, 5, 4, 3]
.for v in tempo_pal
    .byte v
.endfor
.for v in tempo_pal
    .byte round(v * 1.2)
.endfor


