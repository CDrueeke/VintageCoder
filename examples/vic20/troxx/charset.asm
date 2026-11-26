; =====================================================================================================================
; charset.asm - Zeichensatz im RAM ($1C00), 64 Zeichen zu 8 Byte (Codes 0..63)
; =====================================================================================================================
; ZWECK     Die eigenen Grafikzeichen liegen hier als Daten und kommen mit dem Programm in den Speicher.  Die Text-Zeichen (A-Z = 1..26,
;           Leerzeichen 32, "-" 45, Ziffern 48..57, ":" 58) kopiert vic.init beim Start aus dem Zeichen-ROM; ihre Plaetze sind hier mit Nullen belegt.
;           Codes: 0 waagerechte Linie, 27 senkrechte Linie, 28 Kreuzung, 29/30 Ecke oben links/rechts, 31/33 Ecke unten links/rechts,
;           34/35 T nach unten/oben, 36/37 T nach rechts/links, 38..41 Fuellmuster, 42/43 Spieler, 44/46 Gegner, 47 Leben, 59 Treffer,
;           60..63 Tabellen (Startknoten, Gegnerzahl, Wahrscheinlichkeit, Typfarben).  Linien sind zwei Punkte dick (Zeilen/Spalten 3 und 4).  Figuren sind drehsymmetrisch gezeichnet.
; =====================================================================================================================

.segment "CHARSET", $1c00, $1dff
.org $1c00

        .byte $00,$00,$00,$ff,$ff,$00,$00,$00   ;  0 waagerechte Linie
        .fill 26 * 8                            ;  1..26 A-Z (aus dem Zeichen-ROM)
        .byte $18,$18,$18,$18,$18,$18,$18,$18   ; 27 senkrechte Linie
        .byte $18,$18,$18,$ff,$ff,$18,$18,$18   ; 28 Kreuzung
        .byte $00,$00,$00,$1f,$1f,$18,$18,$18   ; 29 Ecke oben links
        .byte $00,$00,$00,$f8,$f8,$18,$18,$18   ; 30 Ecke oben rechts
        .byte $18,$18,$18,$1f,$1f,$00,$00,$00   ; 31 Ecke unten links
        .fill 8                                 ; 32 Leerzeichen
        .byte $18,$18,$18,$f8,$f8,$00,$00,$00   ; 33 Ecke unten rechts
        .byte $00,$00,$00,$ff,$ff,$18,$18,$18   ; 34 T nach unten
        .byte $18,$18,$18,$ff,$ff,$00,$00,$00   ; 35 T nach oben
        .byte $18,$18,$18,$1f,$1f,$18,$18,$18   ; 36 T nach rechts
        .byte $18,$18,$18,$f8,$f8,$18,$18,$18   ; 37 T nach links
        .byte $11,$22,$44,$88,$11,$22,$44,$88   ; 38 Fuellmuster: Schraffur
        .byte $22,$00,$88,$00,$22,$00,$88,$00   ; 39 Fuellmuster: Punkte
        .byte $ff,$80,$80,$80,$ff,$08,$08,$08   ; 40 Fuellmuster: Mauer
        .byte $aa,$55,$aa,$55,$aa,$55,$aa,$55   ; 41 Fuellmuster: Schachbrett
        .byte $3c,$7e,$db,$ff,$ff,$db,$7e,$3c   ; 42 Spieler, Bild 1
        .byte $3c,$7e,$ff,$ff,$e7,$ff,$7e,$3c   ; 43 Spieler, Bild 2
        .byte $24,$5a,$ff,$db,$ff,$7e,$24,$42   ; 44 Gegner, Bild 1
        .fill 8                                 ; 45 "-" (aus dem Zeichen-ROM)
        .byte $24,$5a,$ff,$db,$ff,$7e,$42,$24   ; 46 Gegner, Bild 2
        .byte $66,$ff,$ff,$ff,$7e,$3c,$18,$00   ; 47 Leben
        .fill 11 * 8                            ; 48..58 Ziffern und ":" (aus dem Zeichen-ROM)
        .byte $18,$db,$7e,$3c,$3c,$7e,$db,$18   ; 59 Treffer
        ; 60..63: im Spiel nie angezeigt; die 32 Byte nehmen Tabellen auf (spart Programmraum unterhalb von $1C00)
        .assert * == $1de0
; Gegner: Startknoten (oben rechts, oben links, unten rechts, oben Mitte); Gegnerzahl * 5 (Gegner zu 5 Byte) und Laeufer-Wahrscheinlichkeit (von 256) je Stufe 1..9 (ab 10 wie 9)
START_X: .byte 20, 0, 20, 8
START_Y: .byte 0, 0, 20, 0
COUNT:   .byte 5, 10, 10, 15, 15, 15, 20, 20, 20
PROB:    .byte 128, 144, 160, 176, 192, 200, 210, 220, 230
KIND_COL: .byte COL_RED, COL_PURPLE
        .fill $1e00 - *
.assert * == $1e00, "Zeichensatz muss 64 Zeichen umfassen"
