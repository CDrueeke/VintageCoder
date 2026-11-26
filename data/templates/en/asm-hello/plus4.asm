; {{name}} - Plus/4: border color follows the raster line (TED)
.target plus4
.cpu 7501 { illegals = deny }
.org $1001
.basicstub start

start:  sei
.frame: lda #$80                ; wait for raster line $80
.wait:  cmp ted.raster_lo
        bne .wait
        inc ted.border          ; advance border color
        jmp .frame
