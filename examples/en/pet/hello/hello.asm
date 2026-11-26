; hello.asm - PET Hello (Commodore PET/CBM): print text with the KERNAL routine CHROUT
;
; Purpose: the smallest assembler program for the PET/CBM series. It shows .target pet, the BASIC start line at $0401 (.basicstub), a
;   text loop with a terminator byte and the call of the KERNAL jump table (CHROUT).
;
; Usage: build with `vcc build examples/pet/hello` (or `vcc asm hello.asm --target pet --cpu nmos6502 -o hello.prg`), load the PRG
;   (VICE: `xpet -model 4032 -autostart hello.prg`; tested on 2001, 4032 and 8032 with VICE 3.10 according to the help page "PET/CBM") and type RUN.
;   You see the text HELLO FROM VINTAGECODER, then BASIC reports READY again because the program ends with RTS.
;
; Sequence: the program prints the characters of `text` one by one through CHROUT until the end byte 0 is reached. In the text, a
;   line feed (RETURN, code 13) comes before the 0 so that READY appears on the next line. The loop relies on CHROUT
;   preserving the X register (this works on the tested models); the text may be at most 256 bytes long because X serves as the index
;   and the loop ends when it wraps to 0.
;
; Text: `.petscii` converts to PETSCII codes. Lowercase source text gives the codes $41-$5A, which the PET models 2001 to
;   4032 show as capital letters in their default state (capitals/graphics); according to the help page the 8032 starts
;   in upper/lower case mode, where the same codes probably appear as lowercase letters (not verified).
;
; Hardware and memory: KERNAL jump table CHROUT = $FFD2 (print a character, A = character code; constant also in pet/kernal.asm). Screen
;   from $8000 (addressed by CHROUT only). BASIC programs start at $0401 on the PET: the line `10 SYS 1037` is there; the
;   machine program starts at $040D (= 1037). Zero page: unused. The PET has no 6510 port at $00/$01
;   and no color; hence .cpu nmos6502.
; Limits: spike state of the PET target data (according to the header of pet/hardware.asm only screen memory and CHROUT are confirmed in the emulator).
.target pet
.cpu nmos6502 { illegals = deny }
.org $0401                      ; BASIC program start of the PET (BASIC 2 and 4)
.basicstub start                ; generates `10 SYS 1037` (address of start)

.const CHROUT = $ffd2           ; KERNAL: print the character in A (jump table, same address as on the C64)

start:  ldx #0                  ; X = index into the text
.loop:  lda text,x
        beq .done               ; end byte 0: done
        jsr CHROUT              ; print character (changes A; X is preserved, see above)
        inx
        bne .loop               ; X runs 0..255; the loop ends when it wraps to 0 (at most 256 characters)
.done:  rts                     ; back to BASIC (READY.)

text:   .petscii "hello from vintagecoder"      ; PETSCII codes ($41-$5A for letters)
        .byte 13, 0                             ; line break (RETURN) and end byte
