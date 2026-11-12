; gfx.inc - ERZEUGT von tools/gen.py (nicht von Hand aendern)
; gfx_chars: drei eigene Mehrfarbzeichen zu je 8 Byte (video_init kopiert sie nach CHARSET + 64 * 8): 64 Stein links, 65 Stein rechts, 66 Wand (zugleich Logoblock).
; VIC-I-Mehrfarbe (Farb-RAM-Bit 3 gesetzt): je zwei Bit = ein doppelt breiter Punkt, 4 Punkte je Zeile: 00 Hintergrund, 01 Rahmenfarbe (schwarz = Schatten),
; 10 Zeichenfarbe (Farb-RAM Bits 0..2 = Steinfarbe), 11 Hilfsfarbe (Bits 4..7 von $900E, weiss = Licht).  Licht oben/links, Schatten unten/rechts, Fugen leer.
gfx_chars:
        .byte $3f, $3a, $3a, $3a, $3a, $3a, $15, $00
        .byte $fc, $a4, $a4, $a4, $a4, $a4, $54, $00
        .byte $ff, $e9, $e9, $e9, $e9, $e9, $e9, $55
