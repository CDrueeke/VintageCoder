; rasterbars.asm - raster bars on the C64: the border color changes from raster line to raster line
;
; Purpose: shows include (lib/vic2.inc), macro (wait_line), repetition (.repeat) when generating a table and conditional assembly
;   (.if DEBUG). The bars are made without an interrupt: the program waits actively for raster lines and writes the border color $D020
;   line by line, so that each raster line gets a different color.
;
; Usage: `vcc build examples/c64/rasterbars` (the project file sets DEBUG = 1), load the PRG, RUN. With `vcc asm ... -D DEBUG=0`
;   or `-D DEBUG=1` you choose the version yourself; without it the assembler reports E202 (unknown symbol DEBUG).
;   Quit only with RESET (SEI without CLI, endless loop).
;
; Flow per frame (PAL: 50 times per second):
;   1. wait_line $30: wait until raster line $30 (48) is reached.
;   2. increase phase by one (animation: the bar colors move from frame to frame).
;   3. 16 bars (BARS): for each bar wait for the next raster line (Y remembers the old line, .sync waits for a different one),
;      write the color `colors[bar_index + phase]` to $D020 and count on. The bars lie on lines $31 to $40.
;   4. Back to 1. The border color of the last bar stays until the next frame.
;   With DEBUG = 1 the same color is also written to the background color $D021, so the bars are also visible in the screen window
;   (without DEBUG only where the border is visible).
;   Timing: Each bar is one raster line high. This only works because the loop per bar is shorter than a line
;   (PAL 63 cycles; loop about 35 cycles, with DEBUG about 39). Badlines (lines in which the VIC-II steals cycles from the CPU) can
;   probably shift the color changes by a few cycles and make single bars visibly jitter.
;
; Hardware: VIC-II $D012 (read raster line), $D020 (border color), $D021 (background color 0).
; Memory map: $0801 BASIC line `10 SYS 2061`, program from $080D (about 50 bytes of code), directly behind it the table `colors`
;   (256 bytes). Zero page: $FB = phase (animation phase, freely usable zero page cell of the C64).
; Table colors: 256 entries (i * 3 + 2) & 15, i.e. all 16 C64 colors in an order with step 3; neighboring bars therefore
;   have clearly different colors. The index bar_index + phase overflows in the accumulator (8 bit) and fits exactly 256 entries.
; Limits: no clean end (only RESET); active wait loops without cycle counting; the line times given apply to PAL.
.target c64
.cpu 6510 { illegals = deny }
.include "lib/vic2.inc"             ; vic_raster, vic_border, vic_bg0 and the macro wait_line

.const BARS = 16                ; number of bars = number of raster lines colored per frame
.const phase = $fb              ; zero page: animation phase

.org $0801
.basicstub start

start:  sei                     ; no IRQ: the KERNAL would disturb the line timing
        lda #0
        sta vic_bg0             ; background black
        sta phase               ; phase 0
frame:  wait_line $30           ; once per frame: start shortly before the visible area
        inc phase
        ldx #0                  ; X = bar index 0..BARS-1
.bar:   ldy vic_raster
.sync:  cpy vic_raster          ; wait for the next raster line
        beq .sync
        txa
        clc
        adc phase               ; color = bar index + phase
        tay
        lda colors,y
        sta vic_border
.if DEBUG
        sta vic_bg0             ; in the debug build also color the background
.endif
        inx
        cpx #BARS
        bne .bar
        jmp frame

; 256 color values; .repeat counts i from 0 to 255 and generates one .byte each
colors: .repeat 256, i
        .byte (i * 3 + 2) & 15
.endrepeat
