; =====================================================================================================================
; pal_demo.asm - MEGA65: load a 256-color palette into the VIC-IV
; =====================================================================================================================
; What the program shows
;   The VIC-IV of the MEGA65 has four palettes (banks) of 256 colors each, with 8 bits each of red, green and blue (the C64 has only 16 fixed colors).
;   The program loads the palette graphics/pal65.vcpal created in the palette window of the editor (256 entries: 0-15 the C64 colors,
;   then gradients) into palette bank 1 and selects it for text/bitmap (vic4_use_palette).  It shows the way: export the palette -> include the tables <name>_red/_green/_blue -> copy them into the registers with vic4_load_palette.
;   Hardly anything is visible on the screen (entries 0-15 are the C64 colors); check the result in the monitor/emulator
;   by reading the register windows $D100-$D3FF (for this, bank 1 stays mapped in at the end and is used for text/bitmap).
;
; Usage / flow
;   * Export first: `vcc media export graphics/pal65.vcpal` creates an assembler file (pal65_red, pal65_green, pal65_blue, 256
;     bytes each; red/green/blue are stored with swapped nibbles, as the registers require).  You do not need it to build: `.include "graphics/pal65.vcpal"` converts the palette itself.
;   * Build by hand: vcc asm pal_demo.asm --target mega65 --cpu 45gs02 -o pal_demo.prg   (load address $2000, no BASIC starter:
;     load it in the emulator/monitor and start it with `g 2000` or `SYS 8192`).  The program then stays in an endless loop.
;
; Hardware / library
;   lib/vic4.inc: vic4_unlock ($47, $53 nach $D02F), vic4_load_palette, vic4_use_palette, vic4_palette_runtime.
;   $D070 (palette select, according to the MEGA65 register description; not included in data/chips): bits 7-6 = bank mapped in at $D100-$D3FF,
;   bits 5-4 = bank for text/bitmap, bits 3-2 = bank for sprites, bits 1-0 = alternate palette.
;   $D100/$D200/$D300 = red/green/blue registers of the mapped-in bank (256 bytes each).
;
; Memory
;   Program from $2000 (approx. 870 bytes with the three tables of 256 bytes each); zero page $F7-$FA (source and destination pointers when copying).
; =====================================================================================================================
.target mega65
.cpu 45gs02 { illegals = deny }

.org $2000
.include "lib/vic4.inc"             ; only defines macros, generates no bytes here yet

start:  sei                         ; no interruption while $D070 is being switched
        vic4_unlock                 ; unlock VIC-IV registers (otherwise $D070 has no effect)
        vic4_load_palette pal65_red, pal65_green, pal65_blue, 1   ; copy 3 x 256 bytes into the windows $D100/$D200/$D300 of bank 1
        vic4_use_palette 1          ; select bank 1 for text/bitmap
        lda #$50
        sta $d070                   ; bits 7-6 = 1: keep bank 1 mapped in at $D100-$D3FF (for reading back); bits 5-4 = 1: text/bitmap
                                    ; show bank 1 (set the whole register; vic4_load_palette set it to 0 at the end)
done:   bra done                    ; endless loop

vic4_palette_runtime                ; place the copy subroutine once after the main program
.include "graphics/pal65.vcpal"   ; the tables pal65_red, pal65_green, pal65_blue
