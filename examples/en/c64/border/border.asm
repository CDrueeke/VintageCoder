; border.asm - cycle the border color in step with the raster line (C64) - first VintageCoder example
;
; Purpose: the smallest useful C64 assembler program. It shows the structure of a source file (.target, .cpu, .const, .org, .basicstub),
;   local labels (.frame, .wait), waiting for a certain raster line of the VIC-II and - on purpose - an undocumented
;   instruction (LAX), for which the assembler issues warning W104.
;
; Usage: build with `vcc build examples/c64/border` (or `vcc asm border.asm --target c64 --cpu 6510 -o border.prg`), load the PRG
;   and start it with RUN. You can only stop it with RESET: the program never returns, and because the interrupt stays disabled
;   (SEI without CLI), the RUN/STOP key does not react either.
;
; Flow: background color to black, then endlessly: wait until the VIC-II draws raster line $80 (128), then raise the border color
;   by one. One pass of the loop takes 21 cycles (LDA# 2 + CMP abs 4 + BNE not taken 2 + INC abs 6 + LAX abs 4 + JMP 3),
;   a PAL raster line takes 63 cycles. So the color is advanced several times (about three) within the one line $80;
;   after that the compare no longer matches, and the program waits until line $80 of the next frame. Probably the border flickers
;   quickly through the colors, with a color edge at raster line $80 (not verified on hardware).
;
; Hardware (VIC-II, see c64.inc or data/chips/vic2.chip.json):
;   $D012  raster line, bits 0-7 (bit 8 is in bit 7 of $D011; the low byte is enough here because $80 < 256)
;   $D020  border color (color number 0-15; only the lower four bits count, so INC wraps from 15 to 0)
;   $D021  background color 0
;
; Memory map: $0801 BASIC line `10 SYS 2061`, program from $080D (= 2061), together 34 bytes without load address. Zero page: unused.
; Special feature: LAX is an undocumented 6510 instruction (loads A and X at the same time). It is only here to demonstrate warning W104 and
;   has no effect on the flow, because A is reloaded in .frame anyway. `illegals = warn` assembles it with a warning;
;   `deny` would reject it as an error, `allow` would assemble it without a message.
.target c64
.cpu 6510 { illegals = warn }
.const RASTER = $d012           ; VIC-II: current raster line (read: bits 0-7)
.const BORDER = $d020           ; VIC-II: border color
.const SCREEN = $d021           ; Attention, misleading name: this is background color 0, not the screen memory ($0400)

.org $0801                      ; BASIC start address of the C64
.basicstub start                ; creates `10 SYS <address of start>` so that RUN starts the machine program

start:  sei                     ; disable interrupts: the KERNAL IRQ would only cost time and disturb the loop
        lda #$00
        sta SCREEN              ; background black
.frame: lda #$80                ; wait for raster line $80
.wait:  cmp RASTER              ; compare with the low byte of the current raster line
        bne .wait               ; while not equal: keep waiting
        inc BORDER              ; advance the color
        lax $d020               ; deliberately undocumented: load A and X (W104)
        jmp .frame
