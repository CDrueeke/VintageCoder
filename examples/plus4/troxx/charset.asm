; =====================================================================================================================
; charset.asm - eigene Grafikzeichen ab Zeichencode 128 (69 Zeichen zu 8 Byte)  [erzeugt von tools/gen_charset.py, nicht von Hand aendern]
; =====================================================================================================================
; ZWECK     ted.init kopiert die Zeichen 0..127 aus dem Zeichen-ROM (Text) nach $4000 und diese Zeichen nach $4000 + 128 * 8.
;           Linien sind vier Punkte dick (Zeilen/Spalten 2 bis 5) und einfarbig.  Fuellungen und Logo-Block nutzen Bitpaare (00 Hintergrund 0 = Schatten,
;           01 Mehrfarben-Hintergrund 1 = Grund, 10 Mehrfarben-Hintergrund 2 = Glanz, 11 Farbe der Zelle = Muster) und werden mit Farbspeicher-Bit 3 = 1
;           als Mehrfarben-Zeichen angezeigt.  Jedes Fuellmuster hat neun Zeichen (Lage im 3 x 3 Block): Glanz oben und links, Schatten unten und rechts.
; =====================================================================================================================

GFX:
        .byte $00,$00,$ff,$ff,$ff,$ff,$00,$00   ; 128 waagerechte Linie
        .byte $3c,$3c,$3c,$3c,$3c,$3c,$3c,$3c   ; 129 senkrechte Linie
        .byte $3c,$3c,$ff,$ff,$ff,$ff,$3c,$3c   ; 130 Kreuzung
        .byte $00,$00,$3f,$3f,$3f,$3f,$3c,$3c   ; 131 Ecke oben links
        .byte $00,$00,$fc,$fc,$fc,$fc,$3c,$3c   ; 132 Ecke oben rechts
        .byte $3c,$3c,$3f,$3f,$3f,$3f,$00,$00   ; 133 Ecke unten links
        .byte $3c,$3c,$fc,$fc,$fc,$fc,$00,$00   ; 134 Ecke unten rechts
        .byte $00,$00,$ff,$ff,$ff,$ff,$3c,$3c   ; 135 T nach unten
        .byte $3c,$3c,$ff,$ff,$ff,$ff,$00,$00   ; 136 T nach oben
        .byte $3c,$3c,$3f,$3f,$3f,$3f,$3c,$3c   ; 137 T nach rechts
        .byte $3c,$3c,$fc,$fc,$fc,$fc,$3c,$3c   ; 138 T nach links
        .byte $66,$ff,$ff,$ff,$7e,$3c,$18,$00   ; 139 Herz
        .byte $18,$18,$ff,$7e,$3c,$3c,$66,$c3   ; 140 Stern
        .byte $ff,$81,$b9,$a1,$81,$81,$81,$ff   ; 141 Eisblock
        .byte $30,$38,$3c,$3e,$3c,$38,$30,$00   ; 142 Auswahlzeiger
        .byte $aa,$bf,$bf,$bf,$bf,$bf,$bf,$55   ; 143 abgeschraegter Block (Logo, Mehrfarben)
        .byte $81,$42,$24,$18,$18,$24,$42,$81   ; 144 Treffer
        .byte $3c,$7e,$db,$ff,$ff,$ff,$7e,$5a   ; 145 Figur Bild 1
        .byte $3c,$7e,$db,$ff,$ff,$ff,$7e,$a5   ; 146 Figur Bild 2
        .byte $3c,$7e,$ff,$ff,$ff,$ff,$7e,$5a   ; 147 Figur Bild 3 (blinzelt)
        .byte $3c,$7e,$db,$ff,$ff,$ff,$7e,$a5   ; 148 Figur Bild 4
        .byte $24,$18,$3c,$66,$ff,$bd,$a5,$18   ; 149 Laeufer Bild 1
        .byte $24,$99,$bd,$e7,$ff,$7e,$24,$42   ; 150 Laeufer Bild 2
        .byte $42,$18,$3c,$66,$ff,$bd,$a5,$18   ; 151 Laeufer Bild 3
        .byte $42,$99,$bd,$e7,$ff,$7e,$24,$42   ; 152 Laeufer Bild 4
        .byte $3c,$7e,$db,$ff,$ff,$ff,$ff,$db   ; 153 Streuner Bild 1
        .byte $3c,$7e,$db,$ff,$ff,$ff,$ff,$bd   ; 154 Streuner Bild 2
        .byte $3c,$7e,$ed,$ff,$ff,$ff,$ff,$db   ; 155 Streuner Bild 3
        .byte $3c,$7e,$ed,$ff,$ff,$ff,$ff,$bd   ; 156 Streuner Bild 4
        .byte $18,$3c,$7e,$db,$ff,$24,$5a,$a5   ; 157 Waechter Bild 1
        .byte $18,$3c,$7e,$db,$ff,$5a,$a5,$42   ; 158 Waechter Bild 2
        .byte $18,$3c,$7e,$ed,$ff,$24,$5a,$a5   ; 159 Waechter Bild 3
        .byte $18,$3c,$7e,$ed,$ff,$5a,$a5,$42   ; 160 Waechter Bild 4
        .byte $aa,$97,$9d,$b5,$95,$97,$9d,$b5   ; 161 Fuellung Schraffur, Lage 0,0 (Zeile, Spalte)
        .byte $aa,$57,$5d,$75,$d5,$57,$5d,$75   ; 162 Fuellung Schraffur, Lage 0,1 (Zeile, Spalte)
        .byte $aa,$54,$5c,$74,$d4,$54,$5c,$74   ; 163 Fuellung Schraffur, Lage 0,2 (Zeile, Spalte)
        .byte $95,$97,$9d,$b5,$95,$97,$9d,$b5   ; 164 Fuellung Schraffur, Lage 1,0 (Zeile, Spalte)
        .byte $d5,$57,$5d,$75,$d5,$57,$5d,$75   ; 165 Fuellung Schraffur, Lage 1,1 (Zeile, Spalte)
        .byte $d4,$54,$5c,$74,$d4,$54,$5c,$74   ; 166 Fuellung Schraffur, Lage 1,2 (Zeile, Spalte)
        .byte $95,$97,$9d,$b5,$95,$97,$9d,$80   ; 167 Fuellung Schraffur, Lage 2,0 (Zeile, Spalte)
        .byte $d5,$57,$5d,$75,$d5,$57,$5d,$00   ; 168 Fuellung Schraffur, Lage 2,1 (Zeile, Spalte)
        .byte $d4,$54,$5c,$74,$d4,$54,$5c,$00   ; 169 Fuellung Schraffur, Lage 2,2 (Zeile, Spalte)
        .byte $aa,$b5,$95,$97,$95,$b5,$95,$97   ; 170 Fuellung Punkte, Lage 0,0 (Zeile, Spalte)
        .byte $aa,$75,$55,$57,$55,$75,$55,$57   ; 171 Fuellung Punkte, Lage 0,1 (Zeile, Spalte)
        .byte $aa,$74,$54,$54,$54,$74,$54,$54   ; 172 Fuellung Punkte, Lage 0,2 (Zeile, Spalte)
        .byte $95,$b5,$95,$97,$95,$b5,$95,$97   ; 173 Fuellung Punkte, Lage 1,0 (Zeile, Spalte)
        .byte $55,$75,$55,$57,$55,$75,$55,$57   ; 174 Fuellung Punkte, Lage 1,1 (Zeile, Spalte)
        .byte $54,$74,$54,$54,$54,$74,$54,$54   ; 175 Fuellung Punkte, Lage 1,2 (Zeile, Spalte)
        .byte $95,$b5,$95,$97,$95,$b5,$95,$80   ; 176 Fuellung Punkte, Lage 2,0 (Zeile, Spalte)
        .byte $55,$75,$55,$57,$55,$75,$55,$00   ; 177 Fuellung Punkte, Lage 2,1 (Zeile, Spalte)
        .byte $54,$74,$54,$54,$54,$74,$54,$00   ; 178 Fuellung Punkte, Lage 2,2 (Zeile, Spalte)
        .byte $aa,$bd,$bd,$bd,$aa,$9f,$9f,$9f   ; 179 Fuellung Mauer, Lage 0,0 (Zeile, Spalte)
        .byte $aa,$fd,$fd,$fd,$aa,$df,$df,$df   ; 180 Fuellung Mauer, Lage 0,1 (Zeile, Spalte)
        .byte $aa,$fc,$fc,$fc,$a8,$dc,$dc,$dc   ; 181 Fuellung Mauer, Lage 0,2 (Zeile, Spalte)
        .byte $aa,$bd,$bd,$bd,$aa,$9f,$9f,$9f   ; 182 Fuellung Mauer, Lage 1,0 (Zeile, Spalte)
        .byte $aa,$fd,$fd,$fd,$aa,$df,$df,$df   ; 183 Fuellung Mauer, Lage 1,1 (Zeile, Spalte)
        .byte $a8,$fc,$fc,$fc,$a8,$dc,$dc,$dc   ; 184 Fuellung Mauer, Lage 1,2 (Zeile, Spalte)
        .byte $aa,$bd,$bd,$bd,$aa,$9f,$9f,$80   ; 185 Fuellung Mauer, Lage 2,0 (Zeile, Spalte)
        .byte $aa,$fd,$fd,$fd,$aa,$df,$df,$00   ; 186 Fuellung Mauer, Lage 2,1 (Zeile, Spalte)
        .byte $a8,$fc,$fc,$fc,$a8,$dc,$dc,$00   ; 187 Fuellung Mauer, Lage 2,2 (Zeile, Spalte)
        .byte $aa,$b5,$9f,$9f,$b5,$b5,$9f,$9f   ; 188 Fuellung Schachbrett, Lage 0,0 (Zeile, Spalte)
        .byte $aa,$f5,$5f,$5f,$f5,$f5,$5f,$5f   ; 189 Fuellung Schachbrett, Lage 0,1 (Zeile, Spalte)
        .byte $aa,$f4,$5c,$5c,$f4,$f4,$5c,$5c   ; 190 Fuellung Schachbrett, Lage 0,2 (Zeile, Spalte)
        .byte $b5,$b5,$9f,$9f,$b5,$b5,$9f,$9f   ; 191 Fuellung Schachbrett, Lage 1,0 (Zeile, Spalte)
        .byte $f5,$f5,$5f,$5f,$f5,$f5,$5f,$5f   ; 192 Fuellung Schachbrett, Lage 1,1 (Zeile, Spalte)
        .byte $f4,$f4,$5c,$5c,$f4,$f4,$5c,$5c   ; 193 Fuellung Schachbrett, Lage 1,2 (Zeile, Spalte)
        .byte $b5,$b5,$9f,$9f,$b5,$b5,$9f,$80   ; 194 Fuellung Schachbrett, Lage 2,0 (Zeile, Spalte)
        .byte $f5,$f5,$5f,$5f,$f5,$f5,$5f,$00   ; 195 Fuellung Schachbrett, Lage 2,1 (Zeile, Spalte)
        .byte $f4,$f4,$5c,$5c,$f4,$f4,$5c,$00   ; 196 Fuellung Schachbrett, Lage 2,2 (Zeile, Spalte)
.assert * - GFX == NGFX * 8, "NGFX stimmt nicht mit den Zeichendaten ueberein"
