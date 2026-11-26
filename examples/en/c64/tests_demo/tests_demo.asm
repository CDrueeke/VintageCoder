; tests_demo.asm - example for `vcc test`: static checks and tests in the emulator (size, zero page, overlap, cycle range)
;
; Purpose: shows what the test file tests_demo.vctest is used for. The program itself is only a test subject; it contains three small
;   subroutines that can be used to demonstrate different checks:
;     irq_handler  straight-line code (PHA, LDA, STA, PLA, RTI): the cycles can be determined exactly, in the source additionally given a
;                  `.cycles` block with a budget of 24 cycles (actually 3 + 3 + 4 + 4 + 6 = 20 cycles)
;     next_color   forward branch: the cycle range goes from the shortest to the longest path (here 18 to 19 cycles)
;     clear_text   loop: the check reports "not determinable" instead of guessing; measured in the emulator about 3079 cycles
;   The main program `start` calls clear_text and then jumps into irq_handler like an interrupt (see "Limits"); next_color
;   is only used by the tests. After that the program ends in an endless loop.
;
; Usage (in the repository root directory):
;   vcc build examples/c64/tests_demo              ; builds tests_demo.prg and tests_mailbox.prg
;   vcc test examples/c64/tests_demo               ; only the static checks (without emulator)
;   vcc test examples/c64/tests_demo --backend vice   ; additionally the tests in the emulator (VICE, only one emulator at a time)
; The checks are in tests_demo.vctest: size (segment and whole program, with end address), zeropage (forbidden: $02-$1F), overlap,
; cycles (cycle block `irq`, labels irq_handler, next_color, clear_text, start; the budget 18 for next_color is tight and
; shows the note "budget exceeded" - the longer path needs 19 cycles; for clear_text and start the check reports "not
; determinable" because of the loop or JSR), expr (program size below $2000) and four tests in the emulator (next_color with 15 and 3,
; irq_handler with return by RTI, clear_text). The second test subject in the project, tests_mailbox.asm, has its own test file.
;
; Memory map: $0801 BASIC line `10 SYS 2061`, program from $080D (64 bytes with load address). Zero page: $FB = COLOR (current color).
;   The test file forbids $02-$1F; $FB is allowed. Screen memory $0400-$04FF is written by clear_text, border color $D020
;   by irq_handler.
; Limits: irq_handler ends with RTI and therefore must not be called with JSR; `start` therefore puts the return address and status
;   on the stack itself and jumps there with JMP. The emulator test calls irq_handler through the test case with `"return": "rti"`.
;   next_color is never called by the program itself, only by the tests.
.target c64
.cpu 6510 { illegals = deny }
.const BORDER = $d020           ; VIC-II: border color
.const COLOR  = $fb             ; zero page: current color (free according to the test file)

.org $0801
.basicstub start

start:  sei
        jsr clear_text          ; loop: cycles not determinable
        lda #>back              ; imitate an interrupt by hand so that the RTI of irq_handler returns correctly:
        pha                     ; put the return address (high, then low) and the status register on the stack (like a real IRQ),
        lda #<back              ; RTI fetches them back in the order P, low, high and lands exactly at `back`
        pha
        php
        jmp irq_handler
back:   jmp *

; irq_handler - raster interrupt, straight-line code: exactly determinable.
;   Input: COLOR ($FB) = color. Output: border color $D020 = COLOR. A is preserved (PHA/PLA), no X/Y.
;   Ends with RTI, so it is meant as an interrupt routine (return with status). Without budget overrun: 20 cycles, budget 24.
irq_handler:
.cycles irq max=24              ; the assembler adds up the cycles up to .endcycles and warns (W201) if the worst case is more than 24
        pha
        lda COLOR
        sta BORDER
        pla
        rti
.endcycles

; next_color - forward branch: the range goes from the shortest to the longest path.
;   Input: COLOR ($FB) = current color. Output: COLOR = next color 0..15 (from 15 back to 0). Changes A and the flags.
;   Path 1 (color = 15): 18 cycles; path 2 (otherwise): 19 cycles.
next_color:
        lda COLOR
        cmp #15
        bne .inc                ; not equal to 15: count up
        lda #0                  ; otherwise start over
        sta COLOR
        rts
.inc:   inc COLOR
        rts

; clear_text - loop: the check reports "not determinable" instead of guessing.
;   Clears the first 256 characters of the screen ($0400-$04FF) with the space ($20). Changes A and X (X = 0 at the end, Z set).
;   One pass: LDX 2 + 256 * (LDA# 2 + STA abs,X 5 + INX 2) + 255 * BNE taken 3 + BNE not taken 2 + RTS 6 = 3079 cycles.
clear_text:
        ldx #0
.loop:  lda #$20                ; screen code of the space
        sta $0400,x
        inx
        bne .loop               ; X runs from 0 to 255 and then to 0: 256 passes
        rts
