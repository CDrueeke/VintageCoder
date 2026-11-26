; =====================================================================================================================
; colorbars.asm - raster bars for the Atari 800XL (PAL): background color = raster line
; =====================================================================================================================
;
; WHAT YOU SEE
;   A color gradient over the whole screen height: the background color (COLBK) changes continuously from line to line.  Because
;   a color value consists of hue (bits 7-4) and luminance (bits 3-1, bit 0 unused), luminance steps visibly run
;   through, and the hue changes every 32 raster lines.  This is probably visible mainly in the border and wherever
;   the OS text screen does not cover the background itself with COLPF2.  The file contains no display list interrupt:
;   the color is set by the CPU in a loop (see below).
;
; WHAT THE CODE DOES
;   1. NMIEN = 0 switches off the vertical blank and display list NMIs.  This stops the OS screen update
;      (it would otherwise copy the shadow COLOR4 = $02C8 to COLBK and overwrite the bars).
;   2. Endless loop, exactly one pass per raster line:
;        - read VCOUNT (ANTIC line counter, line / 2, 0..155 on PAL),
;        - STA WSYNC: the CPU is halted until the end of the line, after which the loop always starts at the same
;          place in the next line (no flicker at the line start),
;        - write the VCOUNT value read as the color value to COLBK.
;      VCOUNT grows only every 2 lines, so each color value is 2 raster lines wide.  The value is used directly as the color
;      (hue = upper four bits, luminance = bits 3-1).
;
; REGISTERS: ANTIC NMIEN $D40E, VCOUNT $D40B, WSYNC $D40A;  GTIA COLBK $D01A (names from the target atari8).
; MEMORY:    program from $2000 (17 bytes), no data, no zero page.
; BUILD:     vcc asm examples/atari/colorbars/colorbars.asm --target atari8 --cpu sally -o colorbars.xex
; LIMITS:    No way back to the operating system (endless loop); reset ends the program.
; =====================================================================================================================
.target atari8
.cpu sally { illegals = deny }

.org $2000
start:  lda #0
        sta antic.nmien         ; VBI/DLI off: the OS no longer overwrites COLBK
.line:  ldx antic.vcount        ; X = line / 2 (changes only every 2 raster lines)
        stx antic.wsync         ; wait until the end of the line (the value written does not matter)
        stx gtia.colbk          ; color = VCOUNT (line / 2), applies from the start of the next line
        jmp .line
.run start
