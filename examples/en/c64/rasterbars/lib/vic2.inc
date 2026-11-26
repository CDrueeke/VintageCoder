; vic2.inc - VIC-II registers (excerpt) and a wait macro for the example rasterbars.asm
;
; The constants deliberately have their own names (vic_*) here, to show including a file with .include. The same
; registers are also available as vic2.raster, vic2.border, vic2.bg0 etc. from the target data (data/chips/vic2.chip.json).
.const vic_raster = $d012       ; current raster line, bits 0-7 (read); bit 8 is in bit 7 of $D011
.const vic_border = $d020       ; border color (color number 0-15)
.const vic_bg0    = $d021       ; background color 0 (color number 0-15)

; wait_line(line) - waits in a loop until the VIC-II has reached raster line `line`.
;   Input:  line = raster line 0-255 (only the low byte is compared; lines from 256 cannot be reached this way)
;   Output: none; changes A (= line) and the flags N, Z, C. X and Y stay untouched.
;   Special note: pure wait loop (active waiting, no interrupt). If the call arrives exactly in the line, the macro returns
;   immediately; if the line is missed (e.g. by a long interrupt), it waits until the next frame.
;   `@w` is a local label valid only in the macro, so that it can be used several times.
.macro wait_line(line)
        lda #line
@w:     cmp vic_raster
        bne @w
.endmacro
