; =====================================================================================================================
; border.asm - Border color in step with the raster line (C128, 40-column screen of the VIC-IIe)
; =====================================================================================================================
;
; WHAT THIS EXAMPLE SHOWS
;   The smallest "raster program": the processor waits in a loop until the electron beam of the VIC-IIe reaches raster line $80
;   (= 128, about the middle of the screen), and then advances the border color.  So the border has a different color from line $80
;   on than before it, and because the color keeps moving on with every frame (50 Hz, PAL), it flickers in many colors.  You learn:
;   how to poll the raster line ($D012), write a VIC register directly, disable interrupts and start a machine program with a BASIC line.
;
; BUILD AND RUN
;   vcc build .  (in the border folder)     or     vcc asm border.asm --target c128 --cpu 8502 -o border.prg
;   Start in 40-column mode (key 40/80 DISPLAY or x128 -40col), otherwise the monitor shows only the VDC screen.  Start with RUN.
;   The program never ends by itself; leave it only with a reset (the IRQ is disabled, the KERNAL no longer polls the keyboard).
;
; HARDWARE (names and addresses as on the C64: data/sys/c64/hardware.asm, VIC-IIe)
;   $D012  Raster line (read: current line, only the low 8 bits; bit 8 is in $D011 bit 7 and is ignored here.
;          Line $80 occurs only once per frame, because a PAL frame has only 312 lines and $180 = 384 is never reached.)
;   $D020  Border color (only the low 4 bits count = 16 colors; INC therefore cycles through all colors)
;   $D021  Background color 0 (here only set to black so that the screen looks calm)
;
; MEMORY
;   $1C01        BASIC program start of the C128 (BASIC 7).  The line "10 SYS <address of start>" created by the assembler (here SYS 7181)
;                occupies $1C01-$1C0C and starts the program with RUN.
;   $1C0D-$1C1F  the machine code (19 bytes); the file, including the load address, has 33 bytes.
;   Neither zero page nor any other RAM is used and the MMU is not switched: the configuration stays as after
;   start ($00: system ROMs and I/O visible), so the VIC registers are directly reachable.
;
; TIMING (why the effect looks the way it does)
;   The loop needs only about 17 cycles (LDA 2 + CMP 4 + BNE 2 + INC 6 + JMP 3), whereas a raster line takes about 63 cycles
;   (1 MHz).  After the INC the beam is usually still in line $80, so the loop finds the line again at once and advances the
;   color a few more times (roughly 3 to 4 times per frame).  In line $80 itself this produces a few fine color stripes, after that
;   the last color applies until the end of the frame.  Exactly one change per frame could only be achieved by waiting for a
;   different line after the INC.
; =====================================================================================================================
.target c128
.cpu 8502
.const RASTER = $d012                   ; VIC-IIe: current raster line (low 8 bits), read only
.const BORDER = $d020                   ; VIC-IIe: border color
.const SCREEN = $d021                   ; VIC-IIe: background color 0 (the name matches the KERNAL call SCREEN, $FFED, by coincidence)

.org $1c01                              ; BASIC start of the C128 (in C64 mode it would be $0801)
.basicstub start                        ; creates "10 SYS <start>" so that RUN starts the machine program

start:  sei                             ; disable IRQs: otherwise the KERNAL interrupt routine (keyboard scan, cursor) would interrupt the
                                        ; loop and distort the timing
        lda #$00
        sta SCREEN                      ; background black
.frame: lda #$80                        ; target line (A is kept by CMP; reloading it on every pass is not necessary)
.wait:  cmp RASTER                      ; compare with the current line (Z=1 when the beam has reached it)
        bne .wait                       ; wait until line $80 begins
        inc BORDER                      ; advance the border color by 1 (after 15 comes 0 again)
        jmp .frame                      ; continue endlessly
