; {{name}} – C64: Rahmenfarbe läuft im Takt der Rasterzeile
.target c64
.cpu 6510 { illegals = deny }
.org $0801
.basicstub start

start:  sei
.frame: lda #$80                ; auf Rasterzeile $80 warten
.wait:  cmp $d012
        bne .wait
        inc $d020               ; Rahmenfarbe weiterschalten
        jmp .frame
