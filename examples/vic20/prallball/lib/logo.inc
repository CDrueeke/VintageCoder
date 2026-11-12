; logo.inc - ERZEUGT von tools/gen.py (nicht von Hand aendern)
; logo_bits: Titellogo "PRALL" / "BALL" in 3 x 5 Blockschrift, 10 Zeilen zu je 3 Byte = 24 Bit (hoechstes Bit von Byte 0 = Bildschirmspalte 0; draw_logo benutzt
; die Spalten 0..21); jedes gesetzte Bit wird ein Wandzeichen.  logo_y: Bildschirmzeile je Logozeile (PRALL: 2..6, BALL: 8..12).
logo_bits:
        .byte $77, $74, $40
        .byte $55, $54, $40
        .byte $76, $74, $40
        .byte $45, $54, $40
        .byte $45, $57, $70
        .byte $19, $d1, $00
        .byte $15, $51, $00
        .byte $19, $d1, $00
        .byte $15, $51, $00
        .byte $19, $5d, $c0
logo_y:      .byte 2, 3, 4, 5, 6, 8, 9, 10, 11, 12
