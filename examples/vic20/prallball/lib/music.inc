; music.inc - ERZEUGT von tools/gen.py (nicht von Hand aendern): gekuerztes Titellied (16 Takte, 24 Muster) aus dem C64-Lied (examples/c64/prallball/lib/music.inc)
; Format (siehe examples/c64/prallball/lib/sound.asm; examples/vic20/prallball/lib/sound.asm liest es): Liedkopf mit 5 Byte = Bilder je Zeile, Takte, Wiederholen, Abstand der Mustertabelle
; (16 Bit, vom Liedkopf aus); dann je Takt 3 Musternummern (Spur 0 Melodie = Sopran, 1 Bass, 2 Arpeggio = Alt und Schlagzeug = Rauschen); dann die Muster
; zu je 16 Zeilen (1/16-Noten): 0 = halten, 1..95 = Note, 96 = Pause, 128 + n = Schlagzeug n (nur Spur 2).
song:
        .byte 6, 16, 1, <53, >53       ; Bilder je Zeile, Takte, Wiederholen, Abstand der Mustertabelle
        .byte 0, 0, 1
        .byte 0, 0, 2
        .byte 3, 4, 5
        .byte 6, 7, 8
        .byte 9, 10, 11
        .byte 12, 13, 11
        .byte 14, 15, 11
        .byte 16, 17, 18
        .byte 9, 10, 11
        .byte 19, 13, 11
        .byte 20, 21, 11
        .byte 22, 23, 18
        .byte 9, 10, 11
        .byte 12, 13, 11
        .byte 14, 15, 11
        .byte 16, 17, 18
song_patterns:
        .byte 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        .byte 51, 54, 58, 63, 51, 54, 58, 63, 51, 54, 58, 63, 51, 54, 58, 63
        .byte 59, 63, 66, 71, 59, 63, 66, 71, 59, 63, 66, 71, 59, 63, 66, 71
        .byte 61, 0, 0, 0, 0, 0, 0, 0, 66, 0, 0, 0, 0, 0, 0, 0
        .byte 30, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        .byte 54, 58, 61, 66, 54, 58, 61, 66, 54, 58, 61, 66, 54, 58, 61, 66
        .byte 56, 0, 0, 0, 0, 0, 0, 0, 61, 0, 0, 0, 0, 0, 0, 0
        .byte 25, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        .byte 49, 53, 56, 61, 49, 53, 56, 61, 49, 53, 56, 61, 49, 53, 56, 61
        .byte 58, 0, 0, 63, 0, 0, 66, 0, 65, 0, 63, 0, 58, 0, 0, 0
        .byte 27, 0, 27, 39, 0, 0, 34, 0, 27, 0, 27, 39, 0, 0, 34, 32
        .byte 128, 0, 130, 0, 129, 0, 130, 131, 128, 0, 128, 0, 129, 0, 130, 0
        .byte 63, 0, 0, 59, 0, 0, 63, 0, 61, 0, 59, 0, 58, 0, 0, 0
        .byte 35, 0, 35, 47, 0, 0, 42, 0, 35, 0, 35, 47, 0, 0, 42, 40
        .byte 61, 0, 0, 66, 0, 0, 70, 0, 68, 0, 66, 0, 61, 0, 0, 0
        .byte 30, 0, 30, 42, 0, 0, 37, 0, 30, 0, 30, 42, 0, 0, 37, 35
        .byte 65, 0, 0, 68, 0, 0, 65, 0, 63, 0, 0, 0, 61, 0, 0, 0
        .byte 25, 0, 25, 37, 0, 0, 32, 0, 25, 0, 25, 37, 0, 0, 32, 30
        .byte 128, 0, 130, 0, 129, 0, 130, 131, 128, 0, 128, 0, 129, 132, 133, 133
        .byte 63, 0, 0, 66, 0, 0, 63, 0, 61, 0, 59, 0, 63, 0, 0, 0
        .byte 59, 0, 0, 63, 0, 0, 68, 0, 66, 0, 63, 0, 59, 0, 0, 0
        .byte 32, 0, 32, 44, 0, 0, 39, 0, 32, 0, 32, 44, 0, 0, 39, 37
        .byte 65, 0, 0, 62, 0, 0, 65, 0, 70, 0, 68, 0, 65, 0, 0, 0
        .byte 34, 0, 34, 46, 0, 0, 41, 0, 34, 0, 34, 46, 0, 0, 41, 39
