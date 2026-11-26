; =====================================================================================================================
; farben.bas - Native Atari BASIC without line numbers: labels instead of GOTO numbers
; =====================================================================================================================
; When building, the lines are numbered (10, 20, 30 ...) and "@name" is replaced by the line number.  Semicolon lines are
; source comments and do not end up in the program.
;
; WHAT YOU SEE
;   A text screen (GRAPHICS 0) with a black background and border.  In an endless loop the background cycles through
;   the 16 hues (0-15) at luminance 4; in the middle (column 8, row 10) is the text "ATARI BASIC AND LABELS".
;   Shows: label syntax (@neu: / GOTO @neu), POKE to operating system color shadows, SETCOLOR, POSITION, FOR loops.
;
; BUILD AND RUN
;   vcc build .  -> tokenized Atari BASIC program (target atari8, numbering start values 10, 10); it runs in
;   Atari BASIC (built into the 800XL) and is started there with RUN.
;
; VARIABLES: C = hue 0..15 (loop counter), W = counter of the delay loop.
GRAPHICS 0
; address 710 = COLOR2 (shadow of COLPF2, background of the text field), 712 = COLOR4 (shadow of COLBK, border): both black
POKE 710,0 : POKE 712,0
@neu:
FOR C=0 TO 15
  ; SETCOLOR register,hue,luminance: register 2 = background (COLOR2), luminance 4 (0..14, even values)
  SETCOLOR 2,C,4
  ; empty delay loop (short pause per color) so that the color change stays visible
  FOR W=1 TO 20 : NEXT W
NEXT C
; text in column 8, row 10 of the text screen; "?" is the short form of PRINT
POSITION 8,10 : ? "ATARI BASIC AND LABELS"
; endless loop: back to the label, the program runs until BREAK or RESET
GOTO @neu
