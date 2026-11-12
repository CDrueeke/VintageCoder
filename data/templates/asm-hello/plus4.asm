; {{name}} – Plus/4: Rahmenfarbe läuft im Takt der Rasterzeile (TED)
.target plus4
.cpu 7501 { illegals = deny }
.org $1001
.basicstub start

start:  sei
.frame: lda #$80                ; auf Rasterzeile $80 warten
.wait:  cmp ted.raster_lo
        bne .wait
        inc ted.border          ; Rahmenfarbe weiterschalten
        jmp .frame
