; {{name}} – C64-Steckmodul (8 KB bei $8000). Erzeugen: vcc asm src/c64/module.asm -o build/{{name}}.crt --target c64
.target c64
.cpu 6510 { illegals = deny }
.cartridge "normal", "{{name}}"
.org $8000
        .word cold, cold
        .byte $c3, $c2, $cd, $38, $30       ; "CBM80"
cold:   sei
        lda #$05
        sta $d020
        jmp *
