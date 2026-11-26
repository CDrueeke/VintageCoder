; vic4.inc - MEGA65: unlock the VIC-IV, load a 256-color palette into a palette bank and select it for text/bitmap.
; (This file contains no full-color mode/FCM; for that see examples/mega65/prallball.)
;
;   .include "lib/vic4.inc"
;   vic4_unlock                                   ; unlock VIC-IV registers ($47, $53 to $D02F)
;   vic4_load_palette pal_red, pal_green, pal_blue, 1    ; load 256 colors into palette bank 1 (export of a palette)
;   vic4_use_palette 1                            ; select bank 1 for text/bitmap (and sprites)
;   ...
;   vic4_palette_runtime                          ; ONCE after the main program (not executed)
;
; The tables come from the export of a palette (`<name>_red`, `_green`, `_blue`, 256 bytes each with swapped nibbles).
; $D070 selects the palette bank: bits 7-6 = bank mapped in at $D100-$D3FF; bits 5-4 = text/bitmap; bits 3-2 = sprites; bits 1-0 = alternate palette.
; Zero page used: $F7-$FA ($F7/$F8 = source pointer, $F9/$FA = destination pointer when copying).
; Note: The macros contain no SEI/CLI; whoever uses the same palette in an interrupt must lock it themselves.  All macros
; change A (vic4_load_palette also Y and the zero page); the register layout of $D070 comes from the MEGA65 register description.

#include <mega65/gs4502.asm>

; vic4_unlock: unlock VIC-IV registers (prerequisite for $D070).  Changes A.
.macro vic4_unlock
        mega65_enable_vic4                            ; macro from <mega65/gs4502.asm>: $47, $53 to $D02F
.endmacro

; vic4_load_palette(red, green, blue, bank): Copy the three 256-byte tables red/green/blue (nibbles swapped, export of a palette) into the
;   registers $D100/$D200/$D300 of palette bank `bank` (0-3).  Needs vic4_palette_runtime (subroutine vic4_copy_page).
;   Changes A, Y, $F7-$FA and sets $D070 to 0 at the end (bank 0 mapped in, text/bitmap/sprites bank 0).
.macro vic4_load_palette(red, green, blue, bank)
        lda #(bank & 3) << 6        ; map the bank in at $D100-$D3FF
        sta $d070
        lda #<red
        sta $f7
        lda #>red
        sta $f8
        lda #$00
        sta $f9
        lda #$d1
        sta $fa
        jsr vic4_copy_page
        lda #<green
        sta $f7
        lda #>green
        sta $f8
        lda #$d2
        sta $fa
        jsr vic4_copy_page
        lda #<blue
        sta $f7
        lda #>blue
        sta $f8
        lda #$d3
        sta $fa
        jsr vic4_copy_page
        lda #0
        sta $d070                   ; back to bank 0 (this also clears all other bits of $D070!)
.endmacro

; vic4_use_palette(bank): select palette bank `bank` (0-3) for text/bitmap ($D070 bits 5-4); the other bits are kept.  Changes A.
.macro vic4_use_palette(bank)
        lda $d070
        and #%11001111
        ora #(bank & 3) << 4        ; text/bitmap
        sta $d070
.endmacro

; vic4_palette_runtime: the copy subroutine vic4_copy_page, place it ONCE after the main program (not in the flow).
.macro vic4_palette_runtime
vic4_copy_page:                     ; input: ($F7) = source, ($F9) = destination; copies 256 bytes with Y = 0..255 (source arbitrary, here low byte 0 for the destination); changes A, Y
        ldy #0
@l:     lda ($f7),y
        sta ($f9),y
        iny
        bne @l
        rts
.endmacro
