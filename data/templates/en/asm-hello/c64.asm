; {{name}} - C64: border color follows the raster line
.target c64
.cpu 6510 { illegals = deny }
.org $0801
.basicstub start

start:  sei
.frame: lda #$80                ; wait for raster line $80
.wait:  cmp $d012
        bne .wait
        inc $d020               ; advance border color
        jmp .frame
