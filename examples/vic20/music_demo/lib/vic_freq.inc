; vic_freq.inc – VIC-I-Registerwerte ($80 | N) für Note 1..96, ERZEUGT von tools/gen_vic_freq.py. Mit .const NTSC = 1 die NTSC-Tabelle.
;
; Je Stimme eine Tabelle zu 96 Byte (8 Zeilen zu 12 Noten = 8 Oktaven, Index = Note - 1; Note 1 = C-0), Namen mus_f_bass, mus_f_alto, mus_f_sopr (Töne) und
; mus_f_nois (Rauschen). Der Wert gehört in $900A-$900D: Bit 7 = Stimme ein, Bits 0..6 = N; Frequenz f = Takt * m / (256 * (127 - N)) mit m = 1 (Bass), 2 (Alt),
; 4 (Sopran). Töne außerhalb des Bereichs einer Stimme sind oktaviert; deshalb sind die ersten Zeilen gleich (Bass 2, Alt 3, Sopran 4 Zeilen). Beim
; Rauschen (N = 24 + Note) bestimmt die Note nur die Klangfarbe. PAL: Takt 1108405 Hz (.else-Zweig), NTSC: 1022727 Hz (Zweig .ifdef NTSC).
; Der Erzeuger schreibt diese Kommentarzeilen nicht; sie wurden von Hand ergänzt (tools/gen_vic_freq.py bei Neuerzeugung entsprechend erweitern).
.ifdef NTSC
mus_f_bass:
        .byte $85,$8c,$92,$98,$9e,$a3,$a9,$ad,$b2,$b6,$ba,$be
        .byte $85,$8c,$92,$98,$9e,$a3,$a9,$ad,$b2,$b6,$ba,$be
        .byte $c2,$c5,$c9,$cc,$cf,$d1,$d4,$d6,$d9,$db,$dd,$df
        .byte $e0,$e2,$e4,$e5,$e7,$e8,$e9,$eb,$ec,$ed,$ee,$ef
        .byte $f0,$f1,$f1,$f2,$f3,$f4,$f4,$f5,$f5,$f6,$f6,$f7
        .byte $f7,$f8,$f8,$f9,$f9,$f9,$fa,$fa,$fa,$fa,$fb,$fb
        .byte $fb,$fb,$fc,$fc,$fc,$fc,$fc,$fc,$fd,$fd,$fd,$fd
        .byte $fd,$fd,$fd,$fd,$fd,$fe,$fe,$fe,$fe,$fe,$fe,$fe
mus_f_alto:
        .byte $85,$8c,$92,$98,$9e,$a3,$a9,$ad,$b2,$b6,$ba,$be
        .byte $85,$8c,$92,$98,$9e,$a3,$a9,$ad,$b2,$b6,$ba,$be
        .byte $85,$8c,$92,$98,$9e,$a3,$a9,$ad,$b2,$b6,$ba,$be
        .byte $c2,$c5,$c9,$cc,$cf,$d1,$d4,$d6,$d9,$db,$dd,$df
        .byte $e0,$e2,$e4,$e5,$e7,$e8,$e9,$eb,$ec,$ed,$ee,$ef
        .byte $f0,$f1,$f1,$f2,$f3,$f4,$f4,$f5,$f5,$f6,$f6,$f7
        .byte $f7,$f8,$f8,$f9,$f9,$f9,$fa,$fa,$fa,$fa,$fb,$fb
        .byte $fb,$fb,$fc,$fc,$fc,$fc,$fc,$fc,$fd,$fd,$fd,$fd
mus_f_sopr:
        .byte $85,$8c,$92,$98,$9e,$a3,$a9,$ad,$b2,$b6,$ba,$be
        .byte $85,$8c,$92,$98,$9e,$a3,$a9,$ad,$b2,$b6,$ba,$be
        .byte $85,$8c,$92,$98,$9e,$a3,$a9,$ad,$b2,$b6,$ba,$be
        .byte $85,$8c,$92,$98,$9e,$a3,$a9,$ad,$b2,$b6,$ba,$be
        .byte $c2,$c5,$c9,$cc,$cf,$d1,$d4,$d6,$d9,$db,$dd,$df
        .byte $e0,$e2,$e4,$e5,$e7,$e8,$e9,$eb,$ec,$ed,$ee,$ef
        .byte $f0,$f1,$f1,$f2,$f3,$f4,$f4,$f5,$f5,$f6,$f6,$f7
        .byte $f7,$f8,$f8,$f9,$f9,$f9,$fa,$fa,$fa,$fa,$fb,$fb
mus_f_nois:
        .byte $99,$9a,$9b,$9c,$9d,$9e,$9f,$a0,$a1,$a2,$a3,$a4
        .byte $a5,$a6,$a7,$a8,$a9,$aa,$ab,$ac,$ad,$ae,$af,$b0
        .byte $b1,$b2,$b3,$b4,$b5,$b6,$b7,$b8,$b9,$ba,$bb,$bc
        .byte $bd,$be,$bf,$c0,$c1,$c2,$c3,$c4,$c5,$c6,$c7,$c8
        .byte $c9,$ca,$cb,$cc,$cd,$ce,$cf,$d0,$d1,$d2,$d3,$d4
        .byte $d5,$d6,$d7,$d8,$d9,$da,$db,$dc,$dd,$de,$df,$e0
        .byte $e1,$e2,$e3,$e4,$e5,$e6,$e7,$e8,$e9,$ea,$eb,$ec
        .byte $ed,$ee,$ef,$f0,$f1,$f2,$f3,$f4,$f5,$f6,$f7,$f8
.else
mus_f_bass:
        .byte $bd,$82,$89,$90,$96,$9c,$a1,$a7,$ac,$b0,$b5,$b9
        .byte $bd,$82,$89,$90,$96,$9c,$a1,$a7,$ac,$b0,$b5,$b9
        .byte $bd,$c1,$c4,$c7,$ca,$cd,$d0,$d3,$d5,$d8,$da,$dc
        .byte $de,$e0,$e2,$e3,$e5,$e6,$e8,$e9,$ea,$eb,$ec,$ed
        .byte $ee,$ef,$f0,$f1,$f2,$f3,$f3,$f4,$f5,$f5,$f6,$f6
        .byte $f7,$f7,$f8,$f8,$f8,$f9,$f9,$f9,$fa,$fa,$fa,$fb
        .byte $fb,$fb,$fb,$fc,$fc,$fc,$fc,$fc,$fc,$fd,$fd,$fd
        .byte $fd,$fd,$fd,$fd,$fd,$fd,$fe,$fe,$fe,$fe,$fe,$fe
mus_f_alto:
        .byte $bd,$82,$89,$90,$96,$9c,$a1,$a7,$ac,$b0,$b5,$b9
        .byte $bd,$82,$89,$90,$96,$9c,$a1,$a7,$ac,$b0,$b5,$b9
        .byte $bd,$82,$89,$90,$96,$9c,$a1,$a7,$ac,$b0,$b5,$b9
        .byte $bd,$c1,$c4,$c7,$ca,$cd,$d0,$d3,$d5,$d8,$da,$dc
        .byte $de,$e0,$e2,$e3,$e5,$e6,$e8,$e9,$ea,$eb,$ec,$ed
        .byte $ee,$ef,$f0,$f1,$f2,$f3,$f3,$f4,$f5,$f5,$f6,$f6
        .byte $f7,$f7,$f8,$f8,$f8,$f9,$f9,$f9,$fa,$fa,$fa,$fb
        .byte $fb,$fb,$fb,$fc,$fc,$fc,$fc,$fc,$fc,$fd,$fd,$fd
mus_f_sopr:
        .byte $bd,$82,$89,$90,$96,$9c,$a1,$a7,$ac,$b0,$b5,$b9
        .byte $bd,$82,$89,$90,$96,$9c,$a1,$a7,$ac,$b0,$b5,$b9
        .byte $bd,$82,$89,$90,$96,$9c,$a1,$a7,$ac,$b0,$b5,$b9
        .byte $bd,$82,$89,$90,$96,$9c,$a1,$a7,$ac,$b0,$b5,$b9
        .byte $bd,$c1,$c4,$c7,$ca,$cd,$d0,$d3,$d5,$d8,$da,$dc
        .byte $de,$e0,$e2,$e3,$e5,$e6,$e8,$e9,$ea,$eb,$ec,$ed
        .byte $ee,$ef,$f0,$f1,$f2,$f3,$f3,$f4,$f5,$f5,$f6,$f6
        .byte $f7,$f7,$f8,$f8,$f8,$f9,$f9,$f9,$fa,$fa,$fa,$fb
mus_f_nois:
        .byte $99,$9a,$9b,$9c,$9d,$9e,$9f,$a0,$a1,$a2,$a3,$a4
        .byte $a5,$a6,$a7,$a8,$a9,$aa,$ab,$ac,$ad,$ae,$af,$b0
        .byte $b1,$b2,$b3,$b4,$b5,$b6,$b7,$b8,$b9,$ba,$bb,$bc
        .byte $bd,$be,$bf,$c0,$c1,$c2,$c3,$c4,$c5,$c6,$c7,$c8
        .byte $c9,$ca,$cb,$cc,$cd,$ce,$cf,$d0,$d1,$d2,$d3,$d4
        .byte $d5,$d6,$d7,$d8,$d9,$da,$db,$dc,$dd,$de,$df,$e0
        .byte $e1,$e2,$e3,$e4,$e5,$e6,$e7,$e8,$e9,$ea,$eb,$ec
        .byte $ed,$ee,$ef,$f0,$f1,$f2,$f3,$f4,$f5,$f6,$f7,$f8
.endif
