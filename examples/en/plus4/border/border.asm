; =====================================================================================================================
; border.asm - border color change in step with the raster line (Commodore Plus/4 and C16, TED)
; =====================================================================================================================
;
; WHAT THE EXAMPLE SHOWS
;   The smallest TED program with a raster reference: it waits in a loop for a certain raster line of the screen and advances
;   the border color there.  Because this happens in the middle of the picture, the border above and below this line has a different
;   color; since the color changes continuously, the picture is restless.  The program runs forever.
;   You learn from it: the BASIC starter (.basicstub), waiting for a raster line by reading $FF1D (polling) and the
;   color registers of the TED.
;
; BUILD AND START
;   vcc asm border.asm --target plus4 --cpu 7501 -o border.prg      (or vcc build in the project folder, see border.vcprj)
;   Start in the emulator e.g. with xplus4 -autostart border.prg; on a real computer load the program and type RUN.
;   Only a reset ends it: the interrupts are off, so the RUN/STOP key is no longer polled.
;
; WHAT THE CODE DOES ON THE HARDWARE
;   - sei: disable all interrupts.  The interrupt routine of the KERNAL (keyboard scan, cursor blinking, clock) would otherwise run in the
;     middle of the wait loop and shift the timing; the program does not need it here.
;   - Background color 0 ($FF15) to black.  (The name SCREEN of the constant has nothing to do with the screen memory, it names the
;     register of the background color.)
;   - Then forever: wait until the current raster line is $80 (= 128); increase the border color register $FF19 by 1.
;   The loop (a few instructions) is probably shorter than a raster line.  Then the condition "line = $80" is met several times in a row
;   as long as the beam is still in line $80: the color is probably increased several times in this one line (a
;   colored stripe appears) and not exactly once per frame.  The example wants to show the effect, not a clean single step per frame; if you
;   need exactly one step per frame, wait after the increase until the raster line changes.
;
; TED REGISTERS USED
;   $FF1D  raster line, bits 0-7 (read only; bit 8 is in $FF1C and is ignored here: $80 is below 256)
;   $FF19  border color: bits 0-3 hue, bits 4-6 luminance (TED color value = hue + 16 * luminance, see plus4/hardware.asm).
;          Increasing $FF19 by 1 therefore goes through the 16 hues and then through the luminance levels.
;   $FF15  background color 0 (only set to black here)
;
; MEMORY MAP
;   $1001      BASIC starter (one line "10 SYS 4109", 12 bytes), then at $100D = 4109 the program (19 bytes).  No data, no zero page.
;
; KNOWN LIMITS
;   The example is written for the Plus/4; the C16 has the same TED, the behavior there is not checked here.  The line number
;   $80 is meant for PAL; NTSC was not examined.
; =====================================================================================================================
.target plus4
.cpu 7501
.const RASTER = $ff1d       ; current raster line, bits 0-7 (read only; TED_RASTER_LO in plus4/hardware.asm)
.const BORDER = $ff19       ; border color: hue + 16 * luminance (TED_BORDERCOLOR)
.const SCREEN = $ff15       ; background color 0 (TED_BGCOLOR) - not the screen memory

.org $1001                  ; start of BASIC memory on the Plus/4 (load address of the .prg file)
.basicstub start            ; creates "10 SYS <start>": RUN thereby jumps to start

start:  sei                 ; no interrupts: the KERNAL interrupt routine would disturb the wait loop
        lda #$00
        sta SCREEN          ; background black, so that only the border shows color
.frame: lda #$80            ; wait for raster line $80 (A would be kept even without this reload, cmp and inc do not change A)
.wait:  cmp RASTER          ; is the beam already in line $80?
        bne .wait           ; no: keep polling (active waiting, "polling")
        inc BORDER          ; yes: advance the color (read-modify-write directly in the register)
        jmp .frame          ; and start over
