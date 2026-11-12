; dirtable.inc - ERZEUGT von tools/gen.py (nicht von Hand aendern)
; dir_tab: Richtungstabelle fuer set_vel, 8 Geschwindigkeitsstufen (0..7) x 8 Abprallzonen (0..7) x (vx, vy) = 128 Byte; Index = (Stufe * 8 + Zone) * 2.
; Zone 0..7 = Abprallwinkel -60, -43, -26, -9, +9, +26, +43, +60 Grad gegen die Senkrechte (von links nach rechts).  Betrag = (0,95 + 0,14 * Stufe) * 32;
; vx = Betrag * sin(Winkel), vy = -Betrag * cos(Winkel) (negativ = nach oben), je ein vorzeichenbehaftetes Byte in 1/32 Bildpunkt je Bild (PAL, 50 Hz).
dir_tab:
        .byte $e6, $f1, $eb, $ea, $f3, $e5, $fb, $e2
        .byte $05, $e2, $0d, $e5, $15, $ea, $1a, $f1
        .byte $e2, $ef, $e8, $e6, $f1, $e1, $fb, $de
        .byte $05, $de, $0f, $e1, $18, $e6, $1e, $ef
        .byte $de, $ec, $e5, $e3, $ef, $dd, $fa, $d9
        .byte $06, $d9, $11, $dd, $1b, $e3, $22, $ec
        .byte $da, $ea, $e2, $e0, $ed, $d9, $f9, $d5
        .byte $07, $d5, $13, $d9, $1e, $e0, $26, $ea
        .byte $d6, $e8, $df, $dd, $eb, $d5, $f8, $d0
        .byte $08, $d0, $15, $d5, $21, $dd, $2a, $e8
        .byte $d2, $e6, $dc, $d9, $e9, $d1, $f8, $cc
        .byte $08, $cc, $17, $d1, $24, $d9, $2e, $e6
        .byte $ce, $e3, $d9, $d6, $e7, $cd, $f7, $c7
        .byte $09, $c7, $19, $cd, $27, $d6, $32, $e3
        .byte $cb, $e1, $d6, $d3, $e5, $c8, $f6, $c3
        .byte $0a, $c3, $1b, $c8, $2a, $d3, $35, $e1
