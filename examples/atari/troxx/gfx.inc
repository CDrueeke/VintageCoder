; gfx.inc - ERZEUGT von tools/gen_gfx.py (nicht von Hand aendern).  Zeichen 64..127 des Zeichensatzes (Modus 4).
; Schrift der Seitenleiste: Code = 64 + Bildschirmcode (Ziffern 80..89, Buchstaben 97..122, Leerzeichen 64); Grafikzeichen in den freien Codes.
.const TXT_BASE = 64
.const CH_HLINE = 65
.const CH_VLINE = 66
.const CH_CROSS = 67
.const CH_TL = 68
.const CH_TR = 69
.const CH_BL = 70
.const CH_BR = 71
.const CH_TDOWN = 72
.const CH_TUP = 73
.const CH_TRIGHT = 74
.const CH_TLEFT = 75
.const CH_HLINE_D = 76
.const CH_VLINE_D = 77
.const CH_CROSS_D = 78
.const CH_TDOWN_D = 79
.const CH_TUP_D = 90
.const CH_TRIGHT_D = 91
.const CH_TLEFT_D = 92
.const CH_FTL = 93
.const CH_FT = 94
.const CH_FTR = 95
.const CH_FL = 96
.const CH_FR = 102
.const CH_FBL = 103
.const CH_FB = 106
.const CH_FBR = 107
.const CH_FILL0 = 109
.const CH_FILL1 = 113
.const CH_FILL2 = 119
.const CH_FILL3 = 122
.const CH_HEART = 123
.const CH_STAR = 124
.const CH_ICE = 125
.const CH_BLOCK = 126
.const NGLYPH_FREE = 1                ; ungenutzte Codes
GFX_CHARS:
        .byte $00, $00, $00, $00, $00, $00, $00, $00           ; 64
        .byte $00, $00, $55, $55, $55, $55, $00, $00           ; 65
        .byte $14, $14, $14, $14, $14, $14, $14, $14           ; 66
        .byte $14, $14, $55, $55, $55, $55, $14, $14           ; 67
        .byte $00, $00, $15, $15, $15, $15, $14, $14           ; 68
        .byte $00, $00, $54, $54, $54, $54, $14, $14           ; 69
        .byte $14, $14, $15, $15, $15, $15, $00, $00           ; 70
        .byte $14, $14, $54, $54, $54, $54, $00, $00           ; 71
        .byte $00, $00, $55, $55, $55, $55, $14, $14           ; 72
        .byte $14, $14, $55, $55, $55, $55, $00, $00           ; 73
        .byte $14, $14, $15, $15, $15, $15, $14, $14           ; 74
        .byte $14, $14, $54, $54, $54, $54, $14, $14           ; 75
        .byte $00, $00, $aa, $aa, $aa, $aa, $00, $00           ; 76
        .byte $28, $28, $28, $28, $28, $28, $28, $28           ; 77
        .byte $28, $28, $aa, $aa, $aa, $aa, $28, $28           ; 78
        .byte $00, $00, $aa, $aa, $aa, $aa, $28, $28           ; 79
        .byte $fc, $cc, $cc, $cc, $cc, $cc, $fc, $00           ; 80
        .byte $30, $f0, $30, $30, $30, $30, $fc, $00           ; 81
        .byte $f0, $0c, $0c, $30, $c0, $c0, $fc, $00           ; 82
        .byte $f0, $0c, $0c, $30, $0c, $0c, $f0, $00           ; 83
        .byte $cc, $cc, $cc, $fc, $0c, $0c, $0c, $00           ; 84
        .byte $fc, $c0, $c0, $f0, $0c, $0c, $f0, $00           ; 85
        .byte $3c, $c0, $c0, $f0, $cc, $cc, $30, $00           ; 86
        .byte $fc, $0c, $0c, $30, $30, $30, $30, $00           ; 87
        .byte $30, $cc, $cc, $30, $cc, $cc, $30, $00           ; 88
        .byte $30, $cc, $cc, $3c, $0c, $0c, $f0, $00           ; 89
        .byte $28, $28, $aa, $aa, $aa, $aa, $00, $00           ; 90
        .byte $28, $28, $2a, $2a, $2a, $2a, $28, $28           ; 91
        .byte $28, $28, $a8, $a8, $a8, $a8, $28, $28           ; 92
        .byte $aa, $bf, $bf, $bf, $bf, $bf, $bf, $bf           ; 93
        .byte $aa, $ff, $ff, $ff, $ff, $ff, $ff, $ff           ; 94
        .byte $aa, $fd, $fd, $fd, $fd, $fd, $fd, $fd           ; 95
        .byte $bf, $bf, $bf, $bf, $bf, $bf, $bf, $bf           ; 96
        .byte $30, $cc, $cc, $fc, $cc, $cc, $cc, $00           ; 97
        .byte $f0, $cc, $cc, $f0, $cc, $cc, $f0, $00           ; 98
        .byte $3c, $c0, $c0, $c0, $c0, $c0, $3c, $00           ; 99
        .byte $f0, $cc, $cc, $cc, $cc, $cc, $f0, $00           ; 100
        .byte $fc, $c0, $c0, $f0, $c0, $c0, $fc, $00           ; 101
        .byte $fd, $fd, $fd, $fd, $fd, $fd, $fd, $fd           ; 102
        .byte $bf, $bf, $bf, $bf, $bf, $bf, $bf, $95           ; 103
        .byte $cc, $cc, $cc, $fc, $cc, $cc, $cc, $00           ; 104
        .byte $fc, $30, $30, $30, $30, $30, $fc, $00           ; 105
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $55           ; 106
        .byte $fd, $fd, $fd, $fd, $fd, $fd, $fd, $55           ; 107
        .byte $c0, $c0, $c0, $c0, $c0, $c0, $fc, $00           ; 108
        .byte $dd, $dd, $77, $77, $dd, $dd, $77, $77           ; 109
        .byte $cc, $fc, $fc, $cc, $cc, $cc, $cc, $00           ; 110
        .byte $30, $cc, $cc, $cc, $cc, $cc, $30, $00           ; 111
        .byte $f0, $cc, $cc, $f0, $c0, $c0, $c0, $00           ; 112
        .byte $ff, $dd, $ff, $ff, $ff, $dd, $ff, $ff           ; 113
        .byte $f0, $cc, $cc, $f0, $f0, $cc, $cc, $00           ; 114
        .byte $3c, $c0, $c0, $30, $0c, $0c, $f0, $00           ; 115
        .byte $fc, $30, $30, $30, $30, $30, $30, $00           ; 116
        .byte $cc, $cc, $cc, $cc, $cc, $cc, $fc, $00           ; 117
        .byte $cc, $cc, $cc, $cc, $cc, $30, $30, $00           ; 118
        .byte $7f, $7f, $7f, $55, $f7, $f7, $f7, $55           ; 119
        .byte $cc, $cc, $cc, $30, $cc, $cc, $cc, $00           ; 120
        .byte $cc, $cc, $cc, $30, $30, $30, $30, $00           ; 121
        .byte $dd, $dd, $77, $77, $dd, $dd, $77, $77           ; 122
        .byte $00, $cc, $fc, $fc, $fc, $30, $00, $00           ; 123
        .byte $00, $28, $28, $aa, $28, $28, $82, $00           ; 124
        .byte $00, $aa, $bd, $bd, $95, $95, $55, $00           ; 125
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff           ; 126
        .byte $00, $00, $00, $00, $00, $00, $00, $00           ; 127
