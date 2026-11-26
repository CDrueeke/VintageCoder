; =====================================================================================================================
; text_demo.asm - write text directly into the text screen (GRAPHICS 0) and set colors through OS shadow registers
; =====================================================================================================================
;
; PURPOSE
;   Simplest text output test without CIO: at startup the operating system has set up the text screen (BASIC mode 0, ANTIC mode 2,
;   40 x 24 characters); the program writes 28 character codes directly into screen memory and sets colors.
;   Meant as a simple test for the snapshot in the debugger (the screen memory contents are easy to recognize).
;
; WHAT YOU SEE
;   At the top left in the first text line the text "HALLO ATARI - MOMENTAUFNAHME" (German for "hello Atari - snapshot"; 28 characters), light text (luminance 14) on
;   background hue 9/luminance 4 with border hue 2/luminance 2.  The colors are in OS shadow registers; the
;   operating system copies them into the hardware (GTIA COLPF1/COLPF2/COLBK) in the next vertical blank, so they take effect
;   only in the next frame.  The OS stays active (attract mode may change the colors after some time without a key press).
;
; SCREEN CODES
;   In text mode 2 each cell holds a screen code (not ATASCII): $00 = space, $0D = "-", $10-$19 = digits,
;   $21-$3A = letters A-Z (ATASCII $41-$5A minus $20).  `msg` contains:
;   $28 $21 $2C $2C $2F = HALLO, $00, $21 $34 $21 $32 $29 = ATARI, $00, $0D = -, $00, then MOMENTAUFNAHME.
;
; MEMORY:    program from $2000; text `msg` (28 bytes) directly behind it.  Zero page: SAVMSC $58/$59 (set by the OS, only read).
; OS VARIABLES (constants below): SAVMSC = pointer to screen memory, COLOR1/2/4 = shadows of COLPF1/COLPF2/COLBK.
; BUILD:     vcc asm examples/atari/text_demo/text_demo.asm --target atari8 --cpu sally -o text_demo.xex
; LIMITS:    Assumption: the OS text screen GRAPHICS 0 is active (default after system start).
;            No line breaks, only line 1.  No return to the OS (endless loop).
; =====================================================================================================================
.target atari8
.cpu sally { illegals = deny }

.const SAVMSC = $58             ; pointer to screen memory (2 bytes, low/high; set by the OS)
.const COLOR1 = $02c5           ; text luminance (shadow of COLPF1; in mode 2 only the luminance counts)
.const COLOR2 = $02c6           ; background (shadow of COLPF2)
.const COLOR4 = $02c8           ; border (shadow of COLBK)

.org $2000
start:  lda #$0e                ; color value $0E = hue 0, luminance 14 (light)
        sta COLOR1
        lda #$94                ; hue 9, luminance 4
        sta COLOR2
        lda #$22                ; hue 2, luminance 2
        sta COLOR4
        ldy #0
.copy:  lda msg,y               ; fetch screen code ...
        sta (SAVMSC),y          ; ... and write it into the first text line (Y = column)
        iny
        cpy #28                 ; length of the text
        bne .copy
done:   jmp done

; Text as screen codes (length 28), see header
msg:    .byte $28,$21,$2c,$2c,$2f,$00,$21,$34,$21,$32,$29,$00,$0d,$00,$2d,$2f,$2d,$25,$2e,$34,$21,$35,$26,$2e,$21,$28,$2d,$25
.run start
