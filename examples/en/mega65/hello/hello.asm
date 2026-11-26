; =====================================================================================================================
; hello.asm - MEGA65: smallest example of the special features of the 45GS02 and the VIC-IV
; =====================================================================================================================
; What the program shows
;   1. The border turns red: writing to $D020 (vic2.border), the VIC-II border colour register (colour 2 = red).
;   2. The VIC-IV is unlocked (mega65_enable_vic4: first $47, then $53 to $D02F).  The border does not need this - $D020 is
;      a VIC-II register and always available -; but the example shows the step you need before any access to the extended
;      VIC-IV registers ($D030 ff., see data/sys/mega65/vic4.asm).
;   3. A 32-bit pointer [$40],z in the zero page writes a byte into chip RAM bank 1 (address $0001_0000), which the CPU cannot reach with
;      16-bit addresses.  Then the pointer is increased by 1 as a whole 32-bit word with the Q instructions LDQ/INQ/STQ.
;
; Use / flow
;   Build: vcc asm hello.asm --target mega65 --cpu 45gs02 -o hello.prg   (or `vcc build` in the project folder).
;   Start: load the PRG file on the MEGA65 or in Xemu and start it with RUN.  The BASIC starter (.basicstub, "10 SYS <start>") calls
;   `start`.  Only the red border is visible; the program does not return to BASIC but stays in an endless loop.
;
; Memory / registers
;   Program from $2001 (start of BASIC-65 memory; BASIC starter + code, a few dozen bytes).
;   Zero page: $40-$43 = 32-bit pointer (low byte first).  Z register = index of the 32-bit pointer.  The base page register B
;   must be 0 (normally the case; this program does not change B).
;   $0001_0000 = $FF is written.  Caution: according to examples/mega65/prallball/lib/sound.asm the area from $10000 is used by the KERNAL DOS
;   ($10000-$107FF); after the example, do not continue working in BASIC without resetting the machine.
;   Library: <mega65/gs4502.asm> (macro mega65_enable_vic4; includes <mega65/hardware.asm> with the register names).
; =====================================================================================================================
.target mega65
.cpu 45gs02 { illegals = deny }
#include <mega65/gs4502.asm>   // macro mega65_enable_vic4

.org $2001                          ; BASIC-65 loads programs from $2001
.basicstub start                    ; creates the BASIC line "10 SYS <start>" so that RUN starts the program

start:  mega65_enable_vic4          ; unlock VIC-IV registers ($47, then $53 to $D02F)
        lda #$02
        sta vic2.border             ; border red
        ldz #$00                    ; Z = 0: index for the 32-bit pointer
        lda #$00
        sta $40                     ; zero page pointer $40-$43 = $0001_0000 (32-bit address; byte 0 = $00, 1 = $00, 2 = $01, 3 = $00)
        sta $41
        lda #$01
        sta $42
        lda #$00
        sta $43
        lda #$ff
        sta [$40],z               ; write byte to $1_0000 (chip RAM bank 1, reachable only via the 32-bit pointer)
        ldq $40                     ; load the pointer as a 32-bit value (Q register)
        inq                         ; ... increase it by one
        stq $40                     ; ... and write it back: the pointer now points to $0001_0001
done:   bra done                    ; endless loop (no return to BASIC)
