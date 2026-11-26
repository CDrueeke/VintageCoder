; =====================================================================================================================
; charset.asm - eigene Grafikzeichen ab Zeichencode 128 (19 Zeichen zu 8 Byte)
; =====================================================================================================================
; ZWECK     vic.init kopiert die Zeichen 0..127 aus dem Zeichen-ROM (Text) nach $3000 und diese Zeichen nach $3000 + 128 * 8.
;           Codes: 128 waagerechte Linie, 129 senkrechte, 130 Kreuzung, 131/132 Ecke oben links/rechts, 133/134 Ecke unten links/rechts,
;           135/136 T nach unten/oben, 137/138 T nach rechts/links, 139..142 Fuellmuster (Mehrfarben: Schraffur, Punkte, Mauer, Schachbrett),
;           143 Herz, 144 Stern, 145 Eisblock, 146 Auswahlzeiger, 147 abgeschraegter Block (Logo).
;           Linien sind zwei Punkte dick (Zeilen/Spalten 3 und 4) und einfarbig; die Fuellmuster nutzen Bitpaare (00 Hintergrund, 01 schwarz,
;           10 dunkelgrau, 11 Farbe der Zelle) und werden mit Farbspeicher-Bit 3 = 1 als Mehrfarben-Zeichen angezeigt.
; =====================================================================================================================

GFX:
        .byte $00,$00,$00,$ff,$ff,$00,$00,$00   ; 128 waagerechte Linie
        .byte $18,$18,$18,$18,$18,$18,$18,$18   ; 129 senkrechte Linie
        .byte $18,$18,$18,$ff,$ff,$18,$18,$18   ; 130 Kreuzung
        .byte $00,$00,$00,$1f,$1f,$18,$18,$18   ; 131 Ecke oben links
        .byte $00,$00,$00,$f8,$f8,$18,$18,$18   ; 132 Ecke oben rechts
        .byte $18,$18,$18,$1f,$1f,$00,$00,$00   ; 133 Ecke unten links
        .byte $18,$18,$18,$f8,$f8,$00,$00,$00   ; 134 Ecke unten rechts
        .byte $00,$00,$00,$ff,$ff,$18,$18,$18   ; 135 T nach unten
        .byte $18,$18,$18,$ff,$ff,$00,$00,$00   ; 136 T nach oben
        .byte $18,$18,$18,$1f,$1f,$18,$18,$18   ; 137 T nach rechts
        .byte $18,$18,$18,$f8,$f8,$18,$18,$18   ; 138 T nach links
        .byte $d5,$75,$5d,$57,$d5,$75,$5d,$57   ; 139 Fuellmuster: Schraffur (Mehrfarben)
        .byte $dd,$55,$55,$55,$dd,$55,$55,$55   ; 140 Fuellmuster: Punkte
        .byte $aa,$fe,$fe,$fe,$aa,$ef,$ef,$ef   ; 141 Fuellmuster: Mauer
        .byte $dd,$dd,$77,$77,$dd,$dd,$77,$77   ; 142 Fuellmuster: Schachbrett
        .byte $66,$ff,$ff,$ff,$7e,$3c,$18,$00   ; 143 Herz
        .byte $18,$18,$ff,$7e,$3c,$3c,$66,$c3   ; 144 Stern
        .byte $ff,$81,$b9,$a1,$81,$81,$81,$ff   ; 145 Eisblock
        .byte $30,$38,$3c,$3e,$3c,$38,$30,$00   ; 146 Auswahlzeiger
        .byte $aa,$fd,$fd,$fd,$fd,$fd,$fd,$55   ; 147 abgeschraegter Block (Mehrfarben: Glanz oben/links, Schatten unten/rechts)
.assert * - GFX == NGFX * 8, "NGFX stimmt nicht mit den Zeichendaten ueberein"
