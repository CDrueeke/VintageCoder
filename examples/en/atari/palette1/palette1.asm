; =====================================================================================================================
; palette1.asm - Color chart measurement program, page 1: color values 128..254 (3 scanlines each)
; =====================================================================================================================
;
; PURPOSE
;   Shows the colors of the Atari (GTIA) as horizontal bars for measuring/comparing (e.g. with emulator screenshots).
;   Page 0 (palette0.asm) shows the color values 0..126, page 1 (this file) the values 128..254; both files are
;   identical except for the counting range of Y (0..63 and 64..127).  Only even color values are shown: bit 0 of a
;   color register has no effect, the luminance is in bits 3-1 (8 levels), the hue in bits 7-4
;   (16 hues; this page: hues 8-15).  Each bar is 3 raster lines high, 64 bars = 192 lines.
;
; WHAT YOU SEE (top to bottom, PAL)
;   black | 3 white lines (color value $0E = marker) | 64 bars of color value 2*Y | 3 white lines | 1 black line | rest black
;   The white markers delimit the chart; the chart starts at VCOUNT $10 (raster line 32).
;
; WHAT THE CODE DOES
;   * NMIEN = 0 (no VBI/DLI, the OS no longer writes the shadow registers to the hardware) and DMACTL = 0 (screen DMA
;     off, ANTIC delivers no playfield data): only the background color COLBK is visible then.  SEI is also set
;     (IRQs off); the program never returns to the OS.
;   * For each frame the loop `frame` polls until VCOUNT (line / 2) has the value $10, and then draws line by line.
;   * Each raster line is synchronized with STA WSYNC (CPU waits until the end of the line); the color value is written
;     to COLBK only after the WSYNC and then stays until two more WSYNCs have passed -> 3 lines per bar.
;
; REGISTERS: ANTIC NMIEN $D40E, DMACTL $D400, VCOUNT $D40B, WSYNC $D40A;  GTIA COLBK $D01A.
; MEMORY:    program from $2000, no data, no zero page (registers A, X, Y are used).
; BUILD:     vcc asm examples/atari/palette1/palette1.asm --target atari8 --cpu sally -o palette1.xex
; =====================================================================================================================
.target atari8
.cpu sally { illegals = deny }
.org $2000
start:  sei                     ; IRQs off (NMIs are switched off by NMIEN further below)
        lda #0
        sta antic.nmien
        sta antic.dmactl        ; screen DMA off: only COLBK visible
frame:  lda antic.vcount        ; wait for line 32 (VCOUNT = $10): fixed start line for each frame
        cmp #$10
        bne frame
        ldx #3                  ; 3 white lines as upper marker
mark1:  lda #$0e
        sta antic.wsync
        sta gtia.colbk
        dex
        bne mark1
        ldy #64                 ; Y = bar number; color value = 2*Y
bars:   tya                     ; A = bar number Y
        asl a                   ; A = 2*Y = color value (bit 0 has no effect, so only even values; max. 126 or 254)
        sta antic.wsync
        sta gtia.colbk          ; color value 2*Y, stays for the next two WSYNCs
        sta antic.wsync
        sta antic.wsync
        iny
        cpy #128                ; end value (exclusive)
        bne bars
        ldx #3                  ; 3 white lines as lower marker
mark2:  lda #$0e
        sta antic.wsync
        sta gtia.colbk
        dex
        bne mark2
        lda #0                  ; back to black until the next frame
        sta antic.wsync
        sta gtia.colbk
        jmp frame
.run start
