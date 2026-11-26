; {{name}} - VIC-20 cartridge (8 KB at $A000, autostart). Create: vcc asm src/vic20/module.asm --target vic20 -o build/{{name}}.a0
; The image is raw (no load address); the extension names the memory block: .a0 = $A000, .20/.40/.60 = $2000/$4000/$6000.
.target vic20
#include <vic20/kernal.asm>
.cpu nmos6502 { illegals = deny }
.cartridge "a0", "{{name}}"
.org $A000
        .word cold, cold                ; cold and warm start
        .byte $41, $30, $c3, $c2, $cd   ; signature "A0CBM" at $A004: the system starts the module instead of BASIC

cold:   jsr RAMTAS                      ; the system is not initialized yet: memory, vectors, I/O, screen
        jsr RESTOR
        jsr IOINIT
        jsr CINT
        jsr CLRCHN                      ; default devices: input keyboard, output screen
        cli
        ldx #0
.next:  lda msg,x
        beq .done
        jsr CHROUT
        inx
        bne .next
.done:  jmp .done

msg:    .petscii "{clr}hello from module"
        .byte 0
