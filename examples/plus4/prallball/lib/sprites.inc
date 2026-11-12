; ERZEUGT von tools/gen.py: Zeichen-Sprites des TED (vorverschobene Streifen).  Siehe prallball.asm (Abschnitt Zeichen-Sprites).
; Je Bild: Daten- und Maskenstreifen.  Streifen (Feinlage s, Zeichenspalte c) beginnen bei Bildanfang + (s * NC + c) * (H + 16); jeder Streifen
; hat 8 Leerzeilen davor und danach, damit jede senkrechte Verschiebung 0..7 ohne Pruefung gelesen werden kann.
; Maske: 0 bei Pixeln des Bildes (der Hintergrund wird geloescht), 3 bei durchsichtigen Pixeln (der Hintergrund bleibt).
; Zusammensetzen im Zeichen: Ergebnis = (Hintergrund UND Maske) ODER Daten.  Pixelwerte (2 Bit): 01 = $FF16, 10 = $FF17, 11 = Zellfarbe.
; Bilder 0 = Ball, 1..3 = Schlaeger 16/32/48 px, 4 = Laserschuss, 5..7 = Muenze Kante/halb/3-4, 8..18 = Muenze von vorn mit Buchstaben B S R L F K + P M G D.
imd_0:   ; ball (3 x 6 Pixel, 2 Zeichenspalten, 4 Feinlagen)
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $30, $dc, $f8, $f8, $e8, $20, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $0c, $37, $3e, $3e, $3a, $08, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $03, $0d, $0f, $0f, $0e, $02, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $c0, $80, $80, $80, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $03, $03, $03
        .byte $03, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $c0, $70, $e0, $e0, $a0, $80, $00, $00, $00, $00, $00, $00, $00, $00
imm_0:
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $cf, $03, $03, $03, $03, $cf, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $f3, $c0, $c0, $c0, $c0, $f3, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $fc, $f0, $f0, $f0, $f0, $fc, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $3f, $3f, $3f, $3f, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $fc, $fc, $fc
        .byte $fc, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $3f, $0f, $0f, $0f, $0f, $3f, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
imd_1:   ; pad16 (8 x 6 Pixel, 3 Zeichenspalten, 4 Feinlagen)
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $15, $55, $ff, $ff, $aa, $2a, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $54, $55
        .byte $ff, $ff, $aa, $a8, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $05, $15, $3f, $3f, $2a, $0a
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $55, $55, $ff, $ff, $aa, $aa, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $40, $c0, $c0, $80, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $01, $05, $0f, $0f
        .byte $0a, $02, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $55, $55, $ff, $ff, $aa, $aa, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $40, $50, $f0, $f0, $a0, $80, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $01
        .byte $03, $03, $02, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $55, $55, $ff, $ff, $aa, $aa, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $50, $54, $fc, $fc, $a8, $a0
        .byte $00, $00, $00, $00, $00, $00, $00, $00
imm_1:
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $c0, $00, $00, $00, $00, $c0, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $03, $00
        .byte $00, $00, $00, $03, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $f0, $c0, $c0, $c0, $c0, $f0
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $00, $00, $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $3f, $3f, $3f, $3f, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $fc, $f0, $f0, $f0
        .byte $f0, $fc, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $00, $00, $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $3f, $0f, $0f, $0f, $0f, $3f, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $fc
        .byte $fc, $fc, $fc, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $00, $00, $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $0f, $03, $03, $03, $03, $0f
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
imd_2:   ; pad32 (16 x 6 Pixel, 5 Zeichenspalten, 4 Feinlagen)
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $15, $55, $ff, $ff, $aa, $2a, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $55, $55
        .byte $ff, $ff, $aa, $aa, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $55, $55, $ff, $ff, $aa, $aa, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $54, $55, $ff, $ff, $aa, $a8
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $05, $15, $3f, $3f, $2a, $0a, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $55, $55, $ff, $ff
        .byte $aa, $aa, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $55, $55, $ff, $ff, $aa, $aa, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $55, $55, $ff, $ff, $aa, $aa, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $40
        .byte $c0, $c0, $80, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $01, $05, $0f, $0f, $0a, $02, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $55, $55, $ff, $ff, $aa, $aa
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $55, $55, $ff, $ff, $aa, $aa, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $55, $55, $ff, $ff, $aa, $aa, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $40, $50, $f0, $f0
        .byte $a0, $80, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $01, $03, $03, $02, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $55, $55, $ff, $ff, $aa, $aa, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $55, $55
        .byte $ff, $ff, $aa, $aa, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $55, $55, $ff, $ff, $aa, $aa, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $50, $54, $fc, $fc, $a8, $a0
        .byte $00, $00, $00, $00, $00, $00, $00, $00
imm_2:
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $c0, $00, $00, $00, $00, $c0, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $00, $00
        .byte $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $00, $00, $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $03, $00, $00, $00, $00, $03
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $f0, $c0, $c0, $c0, $c0, $f0, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $00, $00, $00, $00
        .byte $00, $00, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $00, $00, $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $00, $00, $00, $00, $00, $00, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $3f
        .byte $3f, $3f, $3f, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $fc, $f0, $f0, $f0, $f0, $fc, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $00, $00, $00, $00, $00, $00
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $00, $00, $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $00, $00, $00, $00, $00, $00, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $3f, $0f, $0f, $0f
        .byte $0f, $3f, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $fc, $fc, $fc, $fc, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $00, $00, $00, $00, $00, $00, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $00, $00
        .byte $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $00, $00, $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $0f, $03, $03, $03, $03, $0f
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
imd_3:   ; pad48 (24 x 6 Pixel, 7 Zeichenspalten, 4 Feinlagen)
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $15, $55, $ff, $ff, $aa, $2a, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $55, $55
        .byte $ff, $ff, $aa, $aa, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $55, $55, $ff, $ff, $aa, $aa, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $55, $55, $ff, $ff, $aa, $aa
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $55, $55, $ff, $ff, $aa, $aa, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $54, $55, $ff, $ff, $aa, $a8, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $05, $15, $3f, $3f, $2a, $0a, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $55, $55, $ff, $ff, $aa, $aa, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $55, $55
        .byte $ff, $ff, $aa, $aa, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $55, $55, $ff, $ff, $aa, $aa, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $55, $55, $ff, $ff, $aa, $aa
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $55, $55, $ff, $ff, $aa, $aa, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $40, $c0, $c0, $80, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $01, $05, $0f, $0f
        .byte $0a, $02, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $55, $55, $ff, $ff, $aa, $aa, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $55, $55, $ff, $ff, $aa, $aa, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $55, $55
        .byte $ff, $ff, $aa, $aa, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $55, $55, $ff, $ff, $aa, $aa, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $55, $55, $ff, $ff, $aa, $aa
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $40, $50, $f0, $f0, $a0, $80, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $01, $03, $03, $02, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $55, $55, $ff, $ff
        .byte $aa, $aa, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $55, $55, $ff, $ff, $aa, $aa, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $55, $55, $ff, $ff, $aa, $aa, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $55, $55
        .byte $ff, $ff, $aa, $aa, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $55, $55, $ff, $ff, $aa, $aa, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $50, $54, $fc, $fc, $a8, $a0
        .byte $00, $00, $00, $00, $00, $00, $00, $00
imm_3:
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $c0, $00, $00, $00, $00, $c0, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $00, $00
        .byte $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $00, $00, $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $00, $00, $00, $00, $00, $00
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $00, $00, $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $03, $00, $00, $00, $00, $03, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $f0, $c0, $c0, $c0, $c0, $f0, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $00, $00, $00, $00, $00, $00, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $00, $00
        .byte $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $00, $00, $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $00, $00, $00, $00, $00, $00
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $00, $00, $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $3f, $3f, $3f, $3f, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $fc, $f0, $f0, $f0
        .byte $f0, $fc, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $00, $00, $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $00, $00, $00, $00, $00, $00, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $00, $00
        .byte $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $00, $00, $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $00, $00, $00, $00, $00, $00
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $3f, $0f, $0f, $0f, $0f, $3f, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $fc, $fc, $fc, $fc, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $00, $00, $00, $00
        .byte $00, $00, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $00, $00, $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $00, $00, $00, $00, $00, $00, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $00, $00
        .byte $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $00, $00, $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $0f, $03, $03, $03, $03, $0f
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
imd_4:   ; shot (1 x 8 Pixel, 1 Zeichenspalten, 4 Feinlagen)
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $c0, $c0, $c0, $c0, $c0, $c0, $c0, $c0
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $30, $30, $30, $30, $30, $30, $30, $30, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $0c, $0c, $0c, $0c, $0c, $0c, $0c, $0c
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $03, $03, $03, $03, $03, $03, $03, $03, $00, $00, $00, $00, $00, $00, $00, $00
imm_4:
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $3f, $3f, $3f, $3f, $3f, $3f, $3f, $3f
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $cf, $cf, $cf, $cf, $cf, $cf, $cf, $cf, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $f3, $f3, $f3, $f3, $f3, $f3, $f3, $f3
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $fc, $fc, $fc, $fc, $fc, $fc, $fc, $fc, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
imd_5:   ; coin_edge (8 x 12 Pixel, 2 Zeichenspalten, 1 Feinlagen)
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $15, $3f, $2a, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $54, $fc, $a8, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00
imm_5:
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $c0, $c0, $c0, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $03, $03, $03, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
imd_6:   ; coin_half (8 x 12 Pixel, 2 Zeichenspalten, 1 Feinlagen)
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $01, $07, $1f, $1f, $1f, $07
        .byte $02, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $40, $e0, $f8, $f8, $f8, $e0, $80, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00
imm_6:
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $fc, $f0, $c0, $c0, $c0, $f0
        .byte $fc, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $3f, $0f, $03, $03, $03, $0f, $3f, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
imd_7:   ; coin_tq (8 x 12 Pixel, 2 Zeichenspalten, 1 Feinlagen)
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $01, $07, $07, $1f, $1f, $1f, $1f
        .byte $07, $07, $02, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $40, $e0, $e0, $f8, $f8, $f8, $f8, $e0, $e0, $80, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00
imm_7:
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $fc, $f0, $f0, $c0, $c0, $c0, $c0
        .byte $f0, $f0, $fc, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $3f, $0f, $0f, $03, $03, $03, $03, $0f, $0f, $3f, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
imd_8:   ; coin_B (8 x 12 Pixel, 2 Zeichenspalten, 1 Feinlagen)
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $01, $07, $0a, $0a, $1a, $1a, $1a, $1a
        .byte $0a, $0f, $07, $02, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $40, $e0, $b0, $a0, $a8, $b8, $a8, $a8, $b0, $f0, $e0, $80
        .byte $00, $00, $00, $00, $00, $00, $00, $00
imm_8:
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $fc, $f0, $f0, $f0, $c0, $c0, $c0, $c0
        .byte $f0, $f0, $f0, $fc, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $3f, $0f, $0f, $0f, $03, $03, $03, $03, $0f, $0f, $0f, $3f
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
imd_9:   ; coin_S (8 x 12 Pixel, 2 Zeichenspalten, 1 Feinlagen)
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $01, $07, $0e, $0a, $1a, $1e, $1f, $1a
        .byte $0e, $0f, $07, $02, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $40, $e0, $b0, $a0, $f8, $b8, $a8, $a8, $b0, $f0, $e0, $80
        .byte $00, $00, $00, $00, $00, $00, $00, $00
imm_9:
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $fc, $f0, $f0, $f0, $c0, $c0, $c0, $c0
        .byte $f0, $f0, $f0, $fc, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $3f, $0f, $0f, $0f, $03, $03, $03, $03, $0f, $0f, $0f, $3f
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
imd_10:   ; coin_R (8 x 12 Pixel, 2 Zeichenspalten, 1 Feinlagen)
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $01, $07, $0a, $0a, $1a, $1a, $1a, $1a
        .byte $0a, $0f, $07, $02, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $40, $e0, $b0, $a0, $a8, $b8, $b8, $b8, $a0, $f0, $e0, $80
        .byte $00, $00, $00, $00, $00, $00, $00, $00
imm_10:
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $fc, $f0, $f0, $f0, $c0, $c0, $c0, $c0
        .byte $f0, $f0, $f0, $fc, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $3f, $0f, $0f, $0f, $03, $03, $03, $03, $0f, $0f, $0f, $3f
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
imd_11:   ; coin_L (8 x 12 Pixel, 2 Zeichenspalten, 1 Feinlagen)
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $01, $07, $0a, $0a, $1a, $1a, $1a, $1a
        .byte $0a, $0f, $07, $02, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $40, $e0, $f0, $f0, $f8, $f8, $f8, $f8, $a0, $f0, $e0, $80
        .byte $00, $00, $00, $00, $00, $00, $00, $00
imm_11:
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $fc, $f0, $f0, $f0, $c0, $c0, $c0, $c0
        .byte $f0, $f0, $f0, $fc, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $3f, $0f, $0f, $0f, $03, $03, $03, $03, $0f, $0f, $0f, $3f
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
imd_12:   ; coin_F (8 x 12 Pixel, 2 Zeichenspalten, 1 Feinlagen)
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $01, $07, $0a, $0a, $1a, $1a, $1a, $1a
        .byte $0a, $0f, $07, $02, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $40, $e0, $a0, $f0, $f8, $b8, $f8, $f8, $f0, $f0, $e0, $80
        .byte $00, $00, $00, $00, $00, $00, $00, $00
imm_12:
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $fc, $f0, $f0, $f0, $c0, $c0, $c0, $c0
        .byte $f0, $f0, $f0, $fc, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $3f, $0f, $0f, $0f, $03, $03, $03, $03, $0f, $0f, $0f, $3f
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
imd_13:   ; coin_K (8 x 12 Pixel, 2 Zeichenspalten, 1 Feinlagen)
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $01, $07, $0a, $0a, $1a, $1a, $1a, $1a
        .byte $0a, $0f, $07, $02, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $40, $e0, $a0, $b0, $b8, $f8, $b8, $b8, $a0, $f0, $e0, $80
        .byte $00, $00, $00, $00, $00, $00, $00, $00
imm_13:
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $fc, $f0, $f0, $f0, $c0, $c0, $c0, $c0
        .byte $f0, $f0, $f0, $fc, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $3f, $0f, $0f, $0f, $03, $03, $03, $03, $0f, $0f, $0f, $3f
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
imd_14:   ; coin_plus (8 x 12 Pixel, 2 Zeichenspalten, 1 Feinlagen)
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $01, $07, $0f, $0e, $1e, $1a, $1a, $1e
        .byte $0e, $0f, $07, $02, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $40, $e0, $f0, $b0, $b8, $a8, $a8, $b8, $b0, $f0, $e0, $80
        .byte $00, $00, $00, $00, $00, $00, $00, $00
imm_14:
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $fc, $f0, $f0, $f0, $c0, $c0, $c0, $c0
        .byte $f0, $f0, $f0, $fc, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $3f, $0f, $0f, $0f, $03, $03, $03, $03, $0f, $0f, $0f, $3f
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
imd_15:   ; coin_P (8 x 12 Pixel, 2 Zeichenspalten, 1 Feinlagen)
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $01, $07, $0a, $0a, $1a, $1a, $1a, $1a
        .byte $0a, $0f, $07, $02, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $40, $e0, $b0, $a0, $a8, $b8, $f8, $f8, $f0, $f0, $e0, $80
        .byte $00, $00, $00, $00, $00, $00, $00, $00
imm_15:
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $fc, $f0, $f0, $f0, $c0, $c0, $c0, $c0
        .byte $f0, $f0, $f0, $fc, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $3f, $0f, $0f, $0f, $03, $03, $03, $03, $0f, $0f, $0f, $3f
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
imd_16:   ; coin_M (8 x 12 Pixel, 2 Zeichenspalten, 1 Feinlagen)
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $01, $07, $0a, $0a, $1a, $1a, $1a, $1a
        .byte $0a, $0f, $07, $02, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $40, $e0, $e0, $a0, $a8, $a8, $e8, $e8, $e0, $f0, $e0, $80
        .byte $00, $00, $00, $00, $00, $00, $00, $00
imm_16:
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $fc, $f0, $f0, $f0, $c0, $c0, $c0, $c0
        .byte $f0, $f0, $f0, $fc, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $3f, $0f, $0f, $0f, $03, $03, $03, $03, $0f, $0f, $0f, $3f
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
imd_17:   ; coin_G (8 x 12 Pixel, 2 Zeichenspalten, 1 Feinlagen)
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $01, $07, $0e, $0a, $1a, $1a, $1a, $1a
        .byte $0e, $0f, $07, $02, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $40, $e0, $b0, $a0, $f8, $a8, $a8, $a8, $b0, $f0, $e0, $80
        .byte $00, $00, $00, $00, $00, $00, $00, $00
imm_17:
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $fc, $f0, $f0, $f0, $c0, $c0, $c0, $c0
        .byte $f0, $f0, $f0, $fc, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $3f, $0f, $0f, $0f, $03, $03, $03, $03, $0f, $0f, $0f, $3f
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
imd_18:   ; coin_D (8 x 12 Pixel, 2 Zeichenspalten, 1 Feinlagen)
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $01, $07, $0a, $0a, $1a, $1a, $1a, $1a
        .byte $0a, $0f, $07, $02, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $40, $e0, $b0, $b0, $a8, $a8, $a8, $b8, $b0, $f0, $e0, $80
        .byte $00, $00, $00, $00, $00, $00, $00, $00
imm_18:
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $fc, $f0, $f0, $f0, $c0, $c0, $c0, $c0
        .byte $f0, $f0, $f0, $fc, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $3f, $0f, $0f, $0f, $03, $03, $03, $03, $0f, $0f, $0f, $3f
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
; Bildnummern
.const IMG_BALL    = 0
.const IMG_PAD16    = 1
.const IMG_PAD32    = 2
.const IMG_PAD48    = 3
.const IMG_SHOT    = 4
.const IMG_COIN_EDGE    = 5
.const IMG_COIN_HALF    = 6
.const IMG_COIN_TQ    = 7
.const IMG_COIN_B    = 8
.const IMG_COIN_S    = 9
.const IMG_COIN_R    = 10
.const IMG_COIN_L    = 11
.const IMG_COIN_F    = 12
.const IMG_COIN_K    = 13
.const IMG_COIN_PLUS    = 14
.const IMG_COIN_P    = 15
.const IMG_COIN_M    = 16
.const IMG_COIN_G    = 17
.const IMG_COIN_D    = 18
; Anfang der Datenstreifen, Low-Byte (Index = Bildnummer)
im_dlo:   .byte <imd_0, <imd_1, <imd_2, <imd_3, <imd_4, <imd_5, <imd_6, <imd_7, <imd_8, <imd_9, <imd_10, <imd_11, <imd_12, <imd_13, <imd_14, <imd_15, <imd_16, <imd_17, <imd_18
; Anfang der Datenstreifen, High-Byte (Index = Bildnummer)
im_dhi:   .byte >imd_0, >imd_1, >imd_2, >imd_3, >imd_4, >imd_5, >imd_6, >imd_7, >imd_8, >imd_9, >imd_10, >imd_11, >imd_12, >imd_13, >imd_14, >imd_15, >imd_16, >imd_17, >imd_18
; Anfang der Maskenstreifen, Low-Byte (Index = Bildnummer)
im_mlo:   .byte <imm_0, <imm_1, <imm_2, <imm_3, <imm_4, <imm_5, <imm_6, <imm_7, <imm_8, <imm_9, <imm_10, <imm_11, <imm_12, <imm_13, <imm_14, <imm_15, <imm_16, <imm_17, <imm_18
; Anfang der Maskenstreifen, High-Byte (Index = Bildnummer)
im_mhi:   .byte >imm_0, >imm_1, >imm_2, >imm_3, >imm_4, >imm_5, >imm_6, >imm_7, >imm_8, >imm_9, >imm_10, >imm_11, >imm_12, >imm_13, >imm_14, >imm_15, >imm_16, >imm_17, >imm_18
; Hoehe in Pixelzeilen (Index = Bildnummer)
im_h:   .byte 6, 6, 6, 6, 8, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12
; Zeichenspalten je Streifensatz (Breite plus Platz fuer die Verschiebung) (Index = Bildnummer)
im_nc:   .byte 2, 3, 5, 7, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2
; AND-Maske fuer die Feinlage: 3 = vier Feinlagen, 0 = nur Feinlage 0 (Muenzen liegen auf der Zeichengrenze) (Index = Bildnummer)
im_smask:   .byte 3, 3, 3, 3, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
; Laenge eines Streifens in Byte (Hoehe + 16) (Index = Bildnummer)
im_stride:   .byte 22, 22, 22, 22, 24, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28
; Abstand zweier Feinlagen im Bild (Zeichenspalten * Streifenlaenge) (Index = Bildnummer)
im_sset:   .byte 44, 66, 110, 154, 24, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56
; Index = Bild * 4 + Feinlage: Zahl der Zeichenspalten, die Pixel enthalten
im_used:   .byte 1, 1, 2, 2, 2, 3, 3, 3, 4, 5, 5, 5, 6, 7, 7, 7, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2
