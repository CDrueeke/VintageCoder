; {{name}} – Atari-8-KB-Modul bei $A000. Erzeugen: vcc asm src/atari8/module.asm -o build/{{name}}.car --target atari8 --format car
.target atari8
.cpu sally { illegals = deny }
.cartridge "8k"
.org $a000
start:  sei
        lda #$34
        sta gtia.colbk
        jmp *
.run start
