; ERZEUGT von tools/gen.py: Zeichen-Sprites des VDC (vorverschobene Streifen, 1 Bit je Pixel).  Siehe prallball.asm (Abschnitt Zeichen-Sprites).
; Je Bild: Daten- und Maskenstreifen.  Streifen (Feinlage s, Zeichenspalte c) beginnen bei Bildanfang + (s * NC + c) * (H + 16); jeder Streifen
; hat 8 Leerzeilen davor und danach, damit jede senkrechte Verschiebung 0..7 ohne Pruefung gelesen werden kann.
; Maske: 0 bei gesetzten Pixeln des Bildes (der Hintergrund wird geloescht), 1 bei den uebrigen (der Hintergrund bleibt).
imd_0:   ; ball (6 x 6 Pixel, 2 Zeichenspalten, 8 Feinlagen)
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $30, $fc, $fc, $fc, $fc, $30, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $18, $7e, $7e, $7e, $7e, $18, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $0c, $3f, $3f, $3f, $3f, $0c, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $06, $1f, $1f, $1f
        .byte $1f, $06, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $80, $80, $80, $80, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $03, $0f, $0f, $0f, $0f, $03, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $c0
        .byte $c0, $c0, $c0, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $01, $07, $07, $07, $07, $01, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $80, $e0, $e0, $e0, $e0, $80
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $03, $03, $03, $03, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $c0, $f0, $f0, $f0, $f0, $c0, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $01, $01, $01
        .byte $01, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $60, $f8, $f8, $f8, $f8, $60, $00, $00, $00, $00, $00, $00, $00, $00
imm_0:
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $cf, $03, $03, $03, $03, $cf, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $e7, $81, $81, $81, $81, $e7, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $f3, $c0, $c0, $c0, $c0, $f3, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $f9, $e0, $e0, $e0
        .byte $e0, $f9, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $7f, $7f, $7f, $7f, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $fc, $f0, $f0, $f0, $f0, $fc, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $3f
        .byte $3f, $3f, $3f, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $fe, $f8, $f8, $f8, $f8, $fe, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $7f, $1f, $1f, $1f, $1f, $7f
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $fc, $fc, $fc, $fc, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $3f, $0f, $0f, $0f, $0f, $3f, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $fe, $fe, $fe
        .byte $fe, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $9f, $07, $07, $07, $07, $9f, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
imd_1:   ; pad16 (16 x 6 Pixel, 3 Zeichenspalten, 8 Feinlagen)
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $3f, $ff, $ff, $ff, $ff, $3f, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $fc, $ff
        .byte $ff, $ff, $ff, $fc, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $1f, $7f, $7f, $7f, $7f, $1f
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $fe, $ff, $ff, $ff, $ff, $fe, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $80, $80, $80, $80, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $0f, $3f, $3f, $3f
        .byte $3f, $0f, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $ff, $ff, $ff, $ff, $ff, $ff, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $c0, $c0, $c0, $c0, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $07, $1f
        .byte $1f, $1f, $1f, $07, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $80, $e0, $e0, $e0, $e0, $80
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $03, $0f, $0f, $0f, $0f, $03, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $c0, $f0, $f0, $f0
        .byte $f0, $c0, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $01, $07, $07, $07, $07, $01, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $e0, $f8
        .byte $f8, $f8, $f8, $e0, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $03, $03, $03, $03, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $f0, $fc, $fc, $fc, $fc, $f0, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $01, $01, $01, $01, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $7f, $ff, $ff, $ff
        .byte $ff, $7f, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $f8, $fe, $fe, $fe, $fe, $f8, $00, $00, $00, $00, $00, $00, $00, $00
imm_1:
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $c0, $00, $00, $00, $00, $c0, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $03, $00
        .byte $00, $00, $00, $03, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $e0, $80, $80, $80, $80, $e0
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $01, $00, $00, $00, $00, $01, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $7f, $7f, $7f, $7f, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $f0, $c0, $c0, $c0
        .byte $c0, $f0, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $00, $00, $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $3f, $3f, $3f, $3f, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $f8, $e0
        .byte $e0, $e0, $e0, $f8, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $00, $00, $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $7f, $1f, $1f, $1f, $1f, $7f
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $fc, $f0, $f0, $f0, $f0, $fc, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $00, $00, $00, $00, $00, $00, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $3f, $0f, $0f, $0f
        .byte $0f, $3f, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $fe, $f8, $f8, $f8, $f8, $fe, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $00, $00, $00, $00, $00, $00, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $1f, $07
        .byte $07, $07, $07, $1f, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $fc, $fc, $fc, $fc, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $00, $00, $00, $00, $00, $00
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $0f, $03, $03, $03, $03, $0f, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $fe, $fe, $fe, $fe, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $80, $00, $00, $00
        .byte $00, $80, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $07, $01, $01, $01, $01, $07, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
imd_2:   ; pad32 (32 x 6 Pixel, 5 Zeichenspalten, 8 Feinlagen)
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $3f, $ff, $ff, $ff, $ff, $3f, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $fc, $ff, $ff, $ff, $ff, $fc
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $1f, $7f, $7f, $7f, $7f, $1f, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $ff, $ff, $ff, $ff, $ff, $ff, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $fe, $ff, $ff, $ff, $ff, $fe, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $80
        .byte $80, $80, $80, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $0f, $3f, $3f, $3f, $3f, $0f, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $c0, $c0, $c0
        .byte $c0, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $07, $1f, $1f, $1f, $1f, $07, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $80, $e0, $e0, $e0, $e0, $80
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $03, $0f, $0f, $0f, $0f, $03, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $ff, $ff, $ff, $ff, $ff, $ff, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $c0, $f0, $f0, $f0, $f0, $c0, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $01, $07
        .byte $07, $07, $07, $01, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $e0, $f8, $f8, $f8, $f8, $e0, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $03, $03, $03
        .byte $03, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $ff, $ff, $ff, $ff, $ff, $ff, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $f0, $fc, $fc, $fc, $fc, $f0, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $01, $01, $01, $01, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $7f, $ff, $ff, $ff, $ff, $7f, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $f8, $fe, $fe, $fe, $fe, $f8, $00, $00, $00, $00, $00, $00, $00, $00
imm_2:
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $c0, $00, $00, $00, $00, $c0, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $00, $00
        .byte $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $00, $00, $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $03, $00, $00, $00, $00, $03
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $e0, $80, $80, $80, $80, $e0, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $00, $00, $00, $00
        .byte $00, $00, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $00, $00, $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $01, $00, $00, $00, $00, $01, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $7f
        .byte $7f, $7f, $7f, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $f0, $c0, $c0, $c0, $c0, $f0, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $00, $00, $00, $00, $00, $00
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $00, $00, $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $00, $00, $00, $00, $00, $00, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $3f, $3f, $3f
        .byte $3f, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $f8, $e0, $e0, $e0, $e0, $f8, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $00, $00, $00, $00, $00, $00, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $00, $00
        .byte $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $00, $00, $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $7f, $1f, $1f, $1f, $1f, $7f
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $fc, $f0, $f0, $f0, $f0, $fc, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $00, $00, $00, $00, $00, $00, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $00, $00, $00, $00
        .byte $00, $00, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $00, $00, $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $3f, $0f, $0f, $0f, $0f, $3f, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $fe, $f8
        .byte $f8, $f8, $f8, $fe, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $00, $00, $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $00, $00, $00, $00, $00, $00
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $00, $00, $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $1f, $07, $07, $07, $07, $1f, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $fc, $fc, $fc
        .byte $fc, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $00, $00, $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $00, $00, $00, $00, $00, $00, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $00, $00
        .byte $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $0f, $03, $03, $03, $03, $0f, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $fe, $fe, $fe, $fe, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $80, $00, $00, $00, $00, $80, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $00, $00, $00, $00, $00, $00, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $00, $00, $00, $00
        .byte $00, $00, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $07, $01, $01, $01, $01, $07, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
imd_3:   ; pad48 (48 x 6 Pixel, 7 Zeichenspalten, 8 Feinlagen)
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $3f, $ff, $ff, $ff, $ff, $3f, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $fc, $ff, $ff, $ff, $ff, $fc, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $1f, $7f, $7f, $7f, $7f, $1f, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $fe, $ff, $ff, $ff, $ff, $fe, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $80, $80, $80, $80, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $0f, $3f, $3f, $3f
        .byte $3f, $0f, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $ff, $ff, $ff, $ff, $ff, $ff, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $c0, $c0, $c0, $c0, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $07, $1f, $1f, $1f, $1f, $07, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $ff, $ff, $ff, $ff, $ff, $ff, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $80, $e0, $e0, $e0, $e0, $80
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $03, $0f, $0f, $0f, $0f, $03, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $ff, $ff, $ff, $ff, $ff, $ff, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $c0, $f0, $f0, $f0, $f0, $c0, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $01, $07, $07, $07, $07, $01
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $ff, $ff, $ff, $ff, $ff, $ff, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $e0, $f8
        .byte $f8, $f8, $f8, $e0, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $03, $03, $03, $03, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $ff, $ff, $ff, $ff, $ff, $ff, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $f0, $fc, $fc, $fc, $fc, $f0, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $01
        .byte $01, $01, $01, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $7f, $ff, $ff, $ff, $ff, $7f, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $f8, $fe, $fe, $fe, $fe, $f8, $00, $00, $00, $00, $00, $00, $00, $00
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
        .byte $ff, $ff, $e0, $80, $80, $80, $80, $e0, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $00, $00, $00, $00, $00, $00, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $00, $00
        .byte $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $00, $00, $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $00, $00, $00, $00, $00, $00
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $01, $00, $00, $00, $00, $01, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $7f, $7f, $7f, $7f, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $f0, $c0, $c0, $c0
        .byte $c0, $f0, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $00, $00, $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $00, $00, $00, $00, $00, $00, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $00, $00
        .byte $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $00, $00, $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $00, $00, $00, $00, $00, $00
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $3f, $3f, $3f, $3f, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $f8, $e0, $e0, $e0, $e0, $f8, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $00, $00, $00, $00
        .byte $00, $00, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $00, $00, $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $00, $00, $00, $00, $00, $00, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $00, $00
        .byte $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $00, $00, $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $7f, $1f, $1f, $1f, $1f, $7f
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $fc, $f0, $f0, $f0, $f0, $fc, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $00, $00, $00, $00, $00, $00, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $00, $00, $00, $00
        .byte $00, $00, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $00, $00, $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $00, $00, $00, $00, $00, $00, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $00, $00
        .byte $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $3f, $0f, $0f, $0f, $0f, $3f, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $fe, $f8, $f8, $f8, $f8, $fe
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $00, $00, $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $00, $00, $00, $00, $00, $00, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $00, $00, $00, $00
        .byte $00, $00, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $00, $00, $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $00, $00, $00, $00, $00, $00, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $1f, $07
        .byte $07, $07, $07, $1f, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $fc, $fc, $fc, $fc, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $00, $00, $00, $00, $00, $00
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $00, $00, $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $00, $00, $00, $00, $00, $00, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $00, $00, $00, $00
        .byte $00, $00, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $00, $00, $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $0f, $03, $03, $03, $03, $0f, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $fe
        .byte $fe, $fe, $fe, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $80, $00, $00, $00, $00, $80, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $00, $00, $00, $00, $00, $00
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $00, $00, $00, $00, $00, $00, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $00, $00, $00, $00, $00, $00, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $00, $00, $00, $00
        .byte $00, $00, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $07, $01, $01, $01, $01, $07, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
imd_4:   ; shot (2 x 8 Pixel, 2 Zeichenspalten, 8 Feinlagen)
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $c0, $c0, $c0, $c0, $c0, $c0, $c0, $c0
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $60, $60, $60, $60, $60, $60, $60, $60
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $30, $30, $30, $30, $30, $30, $30, $30
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $18, $18, $18, $18, $18, $18, $18, $18
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $0c, $0c, $0c, $0c, $0c, $0c, $0c, $0c
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $06, $06, $06, $06, $06, $06, $06, $06
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $03, $03, $03, $03, $03, $03, $03, $03
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $01, $01, $01, $01, $01, $01, $01, $01
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $80, $80, $80, $80, $80, $80, $80, $80, $00, $00, $00, $00, $00, $00, $00, $00
imm_4:
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $3f, $3f, $3f, $3f, $3f, $3f, $3f, $3f
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $9f, $9f, $9f, $9f, $9f, $9f, $9f, $9f
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $cf, $cf, $cf, $cf, $cf, $cf, $cf, $cf
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $e7, $e7, $e7, $e7, $e7, $e7, $e7, $e7
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $f3, $f3, $f3, $f3, $f3, $f3, $f3, $f3
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $f9, $f9, $f9, $f9, $f9, $f9, $f9, $f9
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $fc, $fc, $fc, $fc, $fc, $fc, $fc, $fc
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $fe, $fe, $fe, $fe, $fe, $fe, $fe, $fe
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $7f, $7f, $7f, $7f, $7f, $7f, $7f, $7f, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
imd_5:   ; coin_edge (16 x 12 Pixel, 2 Zeichenspalten, 1 Feinlagen)
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $3f, $3f, $3f, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $fc, $fc, $fc, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00
imm_5:
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $c0, $c0, $c0, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $03, $03, $03, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
imd_6:   ; coin_half (16 x 12 Pixel, 2 Zeichenspalten, 1 Feinlagen)
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $03, $0f, $3f, $3f, $3f, $0f
        .byte $03, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $c0, $f0, $fc, $fc, $fc, $f0, $c0, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00
imm_6:
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $fc, $f0, $c0, $c0, $c0, $f0
        .byte $fc, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $3f, $0f, $03, $03, $03, $0f, $3f, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
imd_7:   ; coin_tq (16 x 12 Pixel, 2 Zeichenspalten, 1 Feinlagen)
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $03, $0f, $0f, $3f, $3f, $3f, $3f
        .byte $0f, $0f, $03, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $00, $c0, $f0, $f0, $fc, $fc, $fc, $fc, $f0, $f0, $c0, $00
        .byte $00, $00, $00, $00, $00, $00, $00, $00
imm_7:
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $fc, $f0, $f0, $c0, $c0, $c0, $c0
        .byte $f0, $f0, $fc, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $3f, $0f, $0f, $03, $03, $03, $03, $0f, $0f, $3f, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
imd_8:   ; coin_B (16 x 12 Pixel, 2 Zeichenspalten, 1 Feinlagen)
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $03, $0f, $08, $09, $39, $38, $39, $39
        .byte $08, $0f, $0f, $03, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $c0, $f0, $30, $90, $9c, $3c, $9c, $9c, $30, $f0, $f0, $c0
        .byte $00, $00, $00, $00, $00, $00, $00, $00
imm_8:
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $fc, $f0, $f7, $f6, $c6, $c7, $c6, $c6
        .byte $f7, $f0, $f0, $fc, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $3f, $0f, $cf, $6f, $63, $c3, $63, $63, $cf, $0f, $0f, $3f
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
imd_9:   ; coin_S (16 x 12 Pixel, 2 Zeichenspalten, 1 Feinlagen)
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $03, $0f, $0c, $09, $39, $3c, $3f, $39
        .byte $0c, $0f, $0f, $03, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $c0, $f0, $30, $90, $fc, $3c, $9c, $9c, $30, $f0, $f0, $c0
        .byte $00, $00, $00, $00, $00, $00, $00, $00
imm_9:
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $fc, $f0, $f3, $f6, $c6, $c3, $c0, $c6
        .byte $f3, $f0, $f0, $fc, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $3f, $0f, $cf, $6f, $03, $c3, $63, $63, $cf, $0f, $0f, $3f
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
imd_10:   ; coin_R (16 x 12 Pixel, 2 Zeichenspalten, 1 Feinlagen)
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $03, $0f, $08, $09, $39, $38, $38, $39
        .byte $09, $0f, $0f, $03, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $c0, $f0, $30, $90, $9c, $3c, $7c, $3c, $90, $f0, $f0, $c0
        .byte $00, $00, $00, $00, $00, $00, $00, $00
imm_10:
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $fc, $f0, $f7, $f6, $c6, $c7, $c7, $c6
        .byte $f6, $f0, $f0, $fc, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $3f, $0f, $cf, $6f, $63, $c3, $83, $c3, $6f, $0f, $0f, $3f
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
imd_11:   ; coin_L (16 x 12 Pixel, 2 Zeichenspalten, 1 Feinlagen)
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $03, $0f, $09, $09, $39, $39, $39, $39
        .byte $08, $0f, $0f, $03, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $c0, $f0, $f0, $f0, $fc, $fc, $fc, $fc, $10, $f0, $f0, $c0
        .byte $00, $00, $00, $00, $00, $00, $00, $00
imm_11:
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $fc, $f0, $f6, $f6, $c6, $c6, $c6, $c6
        .byte $f7, $f0, $f0, $fc, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $3f, $0f, $0f, $0f, $03, $03, $03, $03, $ef, $0f, $0f, $3f
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
imd_12:   ; coin_F (16 x 12 Pixel, 2 Zeichenspalten, 1 Feinlagen)
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $03, $0f, $08, $09, $39, $38, $39, $39
        .byte $09, $0f, $0f, $03, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $c0, $f0, $10, $f0, $fc, $3c, $fc, $fc, $f0, $f0, $f0, $c0
        .byte $00, $00, $00, $00, $00, $00, $00, $00
imm_12:
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $fc, $f0, $f7, $f6, $c6, $c7, $c6, $c6
        .byte $f6, $f0, $f0, $fc, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $3f, $0f, $ef, $0f, $03, $c3, $03, $03, $0f, $0f, $0f, $3f
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
imd_13:   ; coin_K (16 x 12 Pixel, 2 Zeichenspalten, 1 Feinlagen)
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $03, $0f, $09, $09, $38, $38, $38, $39
        .byte $09, $0f, $0f, $03, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $c0, $f0, $90, $30, $7c, $fc, $7c, $3c, $90, $f0, $f0, $c0
        .byte $00, $00, $00, $00, $00, $00, $00, $00
imm_13:
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $fc, $f0, $f6, $f6, $c7, $c7, $c7, $c6
        .byte $f6, $f0, $f0, $fc, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $3f, $0f, $6f, $cf, $83, $03, $83, $c3, $6f, $0f, $0f, $3f
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
imd_14:   ; coin_plus (16 x 12 Pixel, 2 Zeichenspalten, 1 Feinlagen)
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $03, $0f, $0f, $0e, $3e, $38, $38, $3e
        .byte $0e, $0f, $0f, $03, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $c0, $f0, $f0, $70, $7c, $1c, $1c, $7c, $70, $f0, $f0, $c0
        .byte $00, $00, $00, $00, $00, $00, $00, $00
imm_14:
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $fc, $f0, $f0, $f1, $c1, $c7, $c7, $c1
        .byte $f1, $f0, $f0, $fc, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $3f, $0f, $0f, $8f, $83, $e3, $e3, $83, $8f, $0f, $0f, $3f
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
imd_15:   ; coin_P (16 x 12 Pixel, 2 Zeichenspalten, 1 Feinlagen)
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $03, $0f, $08, $09, $39, $38, $39, $39
        .byte $09, $0f, $0f, $03, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $c0, $f0, $30, $90, $9c, $3c, $fc, $fc, $f0, $f0, $f0, $c0
        .byte $00, $00, $00, $00, $00, $00, $00, $00
imm_15:
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $fc, $f0, $f7, $f6, $c6, $c7, $c6, $c6
        .byte $f6, $f0, $f0, $fc, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $3f, $0f, $cf, $6f, $63, $c3, $03, $03, $0f, $0f, $0f, $3f
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
imd_16:   ; coin_M (16 x 12 Pixel, 2 Zeichenspalten, 1 Feinlagen)
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $03, $0f, $09, $08, $38, $39, $39, $39
        .byte $09, $0f, $0f, $03, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $c0, $f0, $c0, $80, $0c, $4c, $cc, $cc, $c0, $f0, $f0, $c0
        .byte $00, $00, $00, $00, $00, $00, $00, $00
imm_16:
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $fc, $f0, $f6, $f7, $c7, $c6, $c6, $c6
        .byte $f6, $f0, $f0, $fc, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $3f, $0f, $3f, $7f, $f3, $b3, $33, $33, $3f, $0f, $0f, $3f
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
imd_17:   ; coin_G (16 x 12 Pixel, 2 Zeichenspalten, 1 Feinlagen)
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $03, $0f, $0c, $09, $39, $39, $39, $39
        .byte $0c, $0f, $0f, $03, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $c0, $f0, $30, $90, $fc, $1c, $9c, $9c, $30, $f0, $f0, $c0
        .byte $00, $00, $00, $00, $00, $00, $00, $00
imm_17:
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $fc, $f0, $f3, $f6, $c6, $c6, $c6, $c6
        .byte $f3, $f0, $f0, $fc, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $3f, $0f, $cf, $6f, $03, $e3, $63, $63, $cf, $0f, $0f, $3f
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
imd_18:   ; coin_D (16 x 12 Pixel, 2 Zeichenspalten, 1 Feinlagen)
        .byte $00, $00, $00, $00, $00, $00, $00, $00, $03, $0f, $08, $09, $39, $39, $39, $39
        .byte $08, $0f, $0f, $03, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
        .byte $00, $00, $00, $00, $c0, $f0, $70, $30, $9c, $9c, $9c, $3c, $70, $f0, $f0, $c0
        .byte $00, $00, $00, $00, $00, $00, $00, $00
imm_18:
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $fc, $f0, $f7, $f6, $c6, $c6, $c6, $c6
        .byte $f7, $f0, $f0, $fc, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $3f, $0f, $8f, $cf, $63, $63, $63, $c3, $8f, $0f, $0f, $3f
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
im_dlo:   .byte <imd_0, <imd_1, <imd_2, <imd_3, <imd_4, <imd_5, <imd_6, <imd_7, <imd_8, <imd_9, <imd_10, <imd_11, <imd_12, <imd_13, <imd_14, <imd_15, <imd_16, <imd_17, <imd_18
im_dhi:   .byte >imd_0, >imd_1, >imd_2, >imd_3, >imd_4, >imd_5, >imd_6, >imd_7, >imd_8, >imd_9, >imd_10, >imd_11, >imd_12, >imd_13, >imd_14, >imd_15, >imd_16, >imd_17, >imd_18
im_mlo:   .byte <imm_0, <imm_1, <imm_2, <imm_3, <imm_4, <imm_5, <imm_6, <imm_7, <imm_8, <imm_9, <imm_10, <imm_11, <imm_12, <imm_13, <imm_14, <imm_15, <imm_16, <imm_17, <imm_18
im_mhi:   .byte >imm_0, >imm_1, >imm_2, >imm_3, >imm_4, >imm_5, >imm_6, >imm_7, >imm_8, >imm_9, >imm_10, >imm_11, >imm_12, >imm_13, >imm_14, >imm_15, >imm_16, >imm_17, >imm_18
im_h:   .byte 6, 6, 6, 6, 8, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12
im_nc:   .byte 2, 3, 5, 7, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2
im_smask:   .byte 7, 7, 7, 7, 7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
im_stride:   .byte 22, 22, 22, 22, 24, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28
im_sset:   .byte 44, 66, 110, 154, 48, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56
im_used:   .byte 1, 1, 1, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 3, 4, 5, 5, 5, 5, 5, 5, 5, 6, 7, 7, 7, 7, 7, 7, 7, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2
