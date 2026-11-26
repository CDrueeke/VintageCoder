; {{name}} - VIC-20: border color follows the raster line (VIC-I)
.target vic20
.cpu nmos6502 { illegals = deny }
.org $1001
.basicstub start

start:  sei
        ldx #0
.frame: lda #$40                ; wait for raster line $40 (register counts in steps of two)
.wait:  cmp vic1.raster
        bne .wait
        inx
        txa
        and #$07                ; border color 0-7
        ora #$08                ; bit 3 = 1: normal display, black background
        sta vic1.color
        jmp .frame
