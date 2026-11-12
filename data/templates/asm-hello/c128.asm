; {{name}} – C128 (C128-Modus): Rahmenfarbe läuft im Takt der Rasterzeile
.target c128
.cpu 8502 { illegals = deny }
.org $1c01
.basicstub start

start:  sei
.frame: lda #$80                ; auf Rasterzeile $80 warten
.wait:  cmp $d012
        bne .wait
        inc $d020               ; Rahmenfarbe weiterschalten
        jmp .frame
