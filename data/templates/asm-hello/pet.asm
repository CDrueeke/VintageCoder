; {{name}} – PET/CBM: Text mit CHROUT ausgeben
.target pet
.cpu nmos6502 { illegals = deny }
.org $0401
.basicstub start

.const CHROUT = $ffd2

start:  ldx #0
.loop:  lda text,x
        beq .done
        jsr CHROUT
        inx
        bne .loop
.done:  rts

text:   .petscii "hello from vintagecoder"
        .byte 13, 0
