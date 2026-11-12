; dirtable.inc - ERZEUGT von tools/gen.py (nicht von Hand aendern)
; dir_tab: 8 Geschwindigkeitsstufen (0..7) x 8 Abprallzonen (0..7) x (vx, vy), je 16 Bit vorzeichenbehaftet (niedrig, hoch) in 1/256 Teilzelle je Bild.
; Index = (Stufe * 8 + Zone) * 4.  Betrag = 0,70 + 0,08 * Stufe Teilzellen je Bild; vx = Betrag * sin(Winkel), vy = -Betrag * cos(Winkel) (negativ = nach oben).
dir_tab:
        .byte $65, $ff, $a6, $ff, $86, $ff, $7d, $ff, $b1, $ff, $5f, $ff, $e4, $ff, $4f, $ff, $1c, $00, $4f, $ff, $4f, $00, $5f, $ff, $7a, $00, $7d, $ff, $9b, $00, $a6, $ff
        .byte $53, $ff, $9c, $ff, $78, $ff, $6e, $ff, $a8, $ff, $4d, $ff, $e1, $ff, $3b, $ff, $1f, $00, $3b, $ff, $58, $00, $4d, $ff, $88, $00, $6e, $ff, $ad, $00, $9c, $ff
        .byte $41, $ff, $92, $ff, $6a, $ff, $5f, $ff, $9f, $ff, $3a, $ff, $de, $ff, $27, $ff, $22, $00, $27, $ff, $61, $00, $3a, $ff, $96, $00, $5f, $ff, $bf, $00, $92, $ff
        .byte $30, $ff, $88, $ff, $5c, $ff, $50, $ff, $97, $ff, $28, $ff, $da, $ff, $12, $ff, $26, $00, $12, $ff, $69, $00, $28, $ff, $a4, $00, $50, $ff, $d0, $00, $88, $ff
        .byte $1e, $ff, $7d, $ff, $4e, $ff, $41, $ff, $8e, $ff, $15, $ff, $d7, $ff, $fe, $fe, $29, $00, $fe, $fe, $72, $00, $15, $ff, $b2, $00, $41, $ff, $e2, $00, $7d, $ff
        .byte $0c, $ff, $73, $ff, $40, $ff, $32, $ff, $85, $ff, $03, $ff, $d4, $ff, $ea, $fe, $2c, $00, $ea, $fe, $7b, $00, $03, $ff, $c0, $00, $32, $ff, $f4, $00, $73, $ff
        .byte $fa, $fe, $69, $ff, $32, $ff, $23, $ff, $7c, $ff, $f0, $fe, $d1, $ff, $d6, $fe, $2f, $00, $d6, $fe, $84, $00, $f0, $fe, $ce, $00, $23, $ff, $06, $01, $69, $ff
        .byte $e9, $fe, $5f, $ff, $24, $ff, $14, $ff, $73, $ff, $de, $fe, $ce, $ff, $c1, $fe, $32, $00, $c1, $fe, $8d, $00, $de, $fe, $dc, $00, $14, $ff, $17, $01, $5f, $ff
