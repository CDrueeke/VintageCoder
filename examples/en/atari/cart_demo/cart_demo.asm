; =====================================================================================================================
; cart_demo.asm - Atari cartridge: 8 KB standard cartridge or XEGS cartridge with bank switching
; =====================================================================================================================
;
; PURPOSE
;   Shows how an Atari program is built as a cartridge image (.car).  Two versions from one source:
;     * Standard (no switch): 8 KB cartridge, ROM at $A000-$BFFF.  The program only sets the background color and waits.
;     * -D XEGS=1: XEGS cartridge with four 8 KB banks (32 KB).  Banks 0-2 can be mapped to
;       $8000-$9FFF via the bank register $D500; bank 3 sits at a fixed place at $A000-$BFFF and holds the code.
;
; WHAT THE CODE DOES
;   1. SEI (IRQs off; the program does not return to the OS).
;   2. COLBK = $34 (background color hue 3, luminance 4) - a visible sign of life.
;   3. XEGS only: The loop `next` selects bank 0, 1, 2 in turn (writing the bank number to $D500), reads the
;      identifier at $8100 (bank 0 = $A0, bank 1 = $A1, bank 2 = $A2; placed there by the `.byte` lines below) and stores it at
;      $0600+bank.  Then bank 3 is selected again (A = 3 -> $D500).  At the end memory holds $0600-$0602 = $A0 $A1 $A2;
;      this lets you check in the emulator that the bank switching works.
;   4. Endless loop `done`.
;
; STRUCTURE OF THE CARTRIDGE
;   * `.cartridge "8k"` or `.cartridge "xegs"` sets the cartridge type and size; `.bank n` selects, for the XEGS cartridge, the bank
;     the following data (`.org $8100`) belongs to.  The code itself is at `.org $a000`.
;   * The cartridge start vectors ($BFFA-$BFFF) are added by the assembler from `.run start`.
;   * The bank register $D500 is written here as a number (it is not part of the chip tables); $0600 is the freely usable page 6.
;
; BUILD (from the directory of the project folder):
;   vcc asm cart_demo.asm -o build/cart8k.car   --target atari8 --format car --data-dir data
;   vcc asm cart_demo.asm -o build/cartxegs.car --target atari8 --format car -D XEGS=1 --data-dir data
;   (Adjust the path to the source and to `data` depending on the working directory.)
; =====================================================================================================================
.target atari8
.cpu sally { illegals = deny }

.ifdef XEGS
.cartridge "xegs"
.bank 0
.org $8100
        .byte $a0                       ; identifier bank 0
.bank 1
.org $8100
        .byte $a1                       ; identifier bank 1
.bank 2
.org $8100
        .byte $a2                       ; identifier bank 2
.bank 3                                 ; bank 3 = fixed bank at $A000 with the code
.else
.cartridge "8k"
.endif
.org $a000
start:  sei
        lda #$34
        sta gtia.colbk                  ; color the background visibly
.ifdef XEGS
        ldx #0
next:   stx $d500                   ; select bank (X = bank number 0..2)
        lda $8100                   ; identifier byte of the selected bank (mapped at $8000-$9FFF)
        sta $0600,x                 ; store: $0600 + bank
        inx
        cpx #3
        bne next
        lda #3
        sta $d500                   ; select bank 3 again (default state; the code at $A000 is in the fixed bank anyway)
.endif
done:   jmp done
.run start
