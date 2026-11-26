; =====================================================================================================================
; regs_test.asm - self-test of the generated Atari register names
; =====================================================================================================================
;
; PURPOSE
;   Checks that the register names (antic.*, gtia.*, pokey.*) of the target atari8, generated from data/chips/*.chip.json, give
;   the correct addresses when assembled.  The program writes known test values one after another into one register each of
;   ANTIC, GTIA and POKEY and then stays in an endless loop; in the emulator monitor
;   the values can be read in the hardware registers (most registers are write-protected, see the note).
;   (The file lib/atari.inc next to it is the generated constant list, which this program does not include: the
;   names come from .target atari8.)
;
; WHAT IS WRITTEN (value -> register, address)
;   $00 -> ANTIC NMIEN  $D40E  (VBI/DLI off, so that the OS does not overwrite the color registers)
;   $5A -> GTIA COLBK   $D01A  (background color: hue 5, luminance 10)
;   $C4 -> GTIA COLPF2  $D018  (playfield color 2: hue 12, luminance 4)
;   $33 -> GTIA COLPM0  $D012  (color of player/missile 0)
;   $11 -> POKEY AUDF1  $D200  (frequency divider channel 1)
;   $A8 -> POKEY AUDC1  $D201  (bits 7-5 = 101 distortion "pure tone", bit 4 = 0, bits 3-0 = volume 8)
;   $4F -> POKEY AUDCTL $D208  (bits 6, 3, 2, 1, 0: channel 1 at 1.79 MHz, channels 3+4 as 16 bit, high-pass channel 1 and 2,
;                               15 kHz base clock)
;   Note: The write registers D200/D201/D208 are write pages, the read pages of the same addresses are POT0/POT1/ALLPOT.
;   The values are deliberately chosen to be distinguishable, not meant musically; AUDF1/AUDC1 produce a tone (depending on the clock).
;
; MEMORY: program from $2000, no data, no zero page.
; BUILD:    vcc asm examples/atari/regs_test/regs_test.asm --target atari8 --cpu sally -o regs_test.xex
; =====================================================================================================================
.target atari8
.cpu sally { illegals = deny }

.org $2000
start:  lda #0
        sta antic.nmien         ; NMI off: the OS no longer copies the shadow registers into the hardware
        lda #$5A
        sta gtia.colbk
        lda #$C4
        sta gtia.colpf2
        lda #$33
        sta gtia.colpm0
        lda #$11
        sta pokey.audf1
        lda #$A8
        sta pokey.audc1
        lda #$4F
        sta pokey.audctl
done:   jmp done                ; endless loop: the state stays for the monitor
.run start
