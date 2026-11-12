; tabs.inc - ERZEUGT von tools/gen.py (nicht von Hand aendern)
; ovrow_lo/hi: Adresse der Zeile 0..24 in der Overlay-Tabelle (OV + Zeile * 40); die Bildschirmzeile liegt $1400 darueber ($8000 - $6C00).
ovrow_lo:       .byte $00, $28, $50, $78, $a0, $c8, $f0, $18, $40, $68, $90, $b8, $e0, $08, $30, $58, $80, $a8, $d0, $f8, $20, $48, $70, $98, $c0
ovrow_hi:       .byte $6c, $6c, $6c, $6c, $6c, $6c, $6c, $6d, $6d, $6d, $6d, $6d, $6d, $6e, $6e, $6e, $6e, $6e, $6e, $6e, $6f, $6f, $6f, $6f, $6f
; rowlv: Zeilenanfang in der Levelkarte (Zeile 0..11 * 18)
rowlv:          .byte 0, 18, 36, 54, 72, 90, 108, 126, 144, 162, 180, 198
; qtab: Bildschirmcode je Viertelblock-Maske (Bit 0 oben links, 1 oben rechts, 2 unten links, 3 unten rechts).  Mit dem Zeichensatz des PET geprueft: es gibt
; die Viertelbloecke 126 (oben links), 124 (oben rechts), 123 (unten links), 108 (unten rechts), 97 (linke Haelfte), 98 (untere Haelfte), 127 (oben links + unten rechts);
; alle anderen Muster entstehen als Umkehrschrift (+128) des Gegenmusters.
qtab:           .byte 32, 126, 124, 226, 123, 97, 255, 236, 108, 127, 225, 251, 98, 252, 254, 160
