; {{name}} – VC-20-Steckmodul (8 KB bei $A000, Autostart). Erzeugen: vcc asm src/vic20/module.asm --target vic20 -o build/{{name}}.a0
; Das Abbild ist roh (ohne Ladeadresse); die Endung nennt den Speicherblock: .a0 = $A000, .20/.40/.60 = $2000/$4000/$6000.
.target vic20
#include <vic20/kernal.asm>
.cpu nmos6502 { illegals = deny }
.cartridge "a0", "{{name}}"
.org $A000
        .word cold, cold                ; Kalt- und Warmstart
        .byte $41, $30, $c3, $c2, $cd   ; Kennung "A0CBM" bei $A004: das System startet das Modul statt BASIC

cold:   jsr RAMTAS                      ; das System ist noch nicht initialisiert: Speicher, Vektoren, E/A, Bildschirm
        jsr RESTOR
        jsr IOINIT
        jsr CINT
        jsr CLRCHN                      ; Standardgeräte: Eingabe Tastatur, Ausgabe Bildschirm
        cli
        ldx #0
.next:  lda msg,x
        beq .done
        jsr CHROUT
        inx
        bne .next
.done:  jmp .done

msg:    .petscii "{clr}hallo vom modul"
        .byte 0
