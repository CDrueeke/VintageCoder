; {{name}} – VC-20: Rahmenfarbe läuft im Takt der Rasterzeile (VIC-I)
.target vic20
.cpu nmos6502 { illegals = deny }
.org $1001
.basicstub start

start:  sei
        ldx #0
.frame: lda #$40                ; auf Rasterzeile $40 warten (Register zählt in Zweierschritten)
.wait:  cmp vic1.raster
        bne .wait
        inx
        txa
        and #$07                ; Rahmenfarbe 0-7
        ora #$08                ; Bit 3 = 1: normale Darstellung, Hintergrund schwarz
        sta vic1.color
        jmp .frame
