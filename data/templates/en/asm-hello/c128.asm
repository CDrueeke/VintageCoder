; {{name}} - C128 (C128 mode): border color follows the raster line
.target c128
.cpu 8502 { illegals = deny }
.org $1c01
.basicstub start

start:  sei
.frame: lda #$80                ; wait for raster line $80
.wait:  cmp $d012
        bne .wait
        inc $d020               ; advance border color
        jmp .frame
