; sprite_demo.asm - show a multicolor sprite drawn with the sprite editor on the C64
;
; Purpose: shows how an exported sprite (graphics/hero.vcsprite) gets into a program and how to switch on sprite 0 of the VIC-II,
;   put it into multicolor mode, position it and colour it.
;
; Preparation and use: `vcc build examples/c64/sprite_demo` converts the sprite itself when it is included (or export as ASM in the editor,
;   or `vcc media export graphics/hero.vcsprite`) and delivers the text with the label `hero` and the constants
;   hero_len (128 bytes = 2 frames of 64 bytes each), hero_sprite (7), hero_mc1 (2), hero_mc2 (6) and hero_count (2). Load the PRG, RUN: sprite 0
;   appears as a small figure at screen position X = 100, Y = 100 (coordinates of the sprite registers, not of the text grid). The
;   program stays in an endless loop (SEI without CLI); stop it with RESET. The sprite has two frames; only frame 0 is shown,
;   the pointer stays unchanged (a change to $81 would show frame 1; here both frames are the same).
;
; Hardware (VIC-II, registers as vic2.* from the target data): The sprite pointers sit behind the 1000 characters of screen memory,
;   so the pointer of sprite 0 is at $0400 + $03F8 = $07F8. It holds the sprite address divided by 64, relative to VIC bank 0
;   ($0000-$3FFF).
;   $D015 spena (sprites on), $D01C spmc (sprite multicolor), $D000/$D001 sp0x/sp0y (position), $D027 sp0col (sprite colour),
;   $D025/$D026 spmc0/spmc1 (the two shared multicolor colours of all sprites).
;   $D010 msigx (X bit 8) is not set by the program (0 after power-on).
; Colours (C64 palette): sprite 7 = yellow, multicolor 0 = 2 (red), multicolor 1 = 6 (blue) - the same values as hero_sprite, hero_mc1 and
;   hero_mc2 in the export and in the sprite editor (there the shared colours are called mc1/mc2, in the VIC-II spmc0/spmc1).
;
; Memory map: $0801 BASIC line `10 SYS 2061` and program (40 bytes, ends at $0835), sprite data from $2000 (128 bytes, frame 0 at $2000,
;   frame 1 at $2040). The gap between the end of the program and $2000 is filled by the assembler in the PRG with zero bytes: the PRG is therefore about 6 KB
;   large. Zero page: unused.
; Limits: assumes the standard screen memory $0400 (sprite pointer = screen memory + $3F8) and VIC bank 0.
.target c64
.cpu 6510 { illegals = deny }

.org $0801
.basicstub start

start:  sei
        lda #$80                ; sprite pointer 0: data at $2000 ($2000 / 64 = $80)
        sta $07f8
        lda #1
        sta vic2.spena          ; sprite 0 on (bit 0 = sprite 0)
        sta vic2.spmc           ; multicolor (bit 0 = sprite 0)
        lda #100
        sta vic2.sp0x           ; X = 100 and Y = 100 (sprite coordinates; X bit 8 in $D010 is 0)
        sta vic2.sp0y
        lda #7
        sta vic2.sp0col         ; single colour of the sprite (yellow)
        lda #2
        sta vic2.spmc0          ; shared multicolor colour 0 (red)
        lda #6
        sta vic2.spmc1          ; shared multicolor colour 1 (blue)
loop:   jmp loop

.org $2000                      ; sprite data on a 64-byte boundary: pointer value $80 = $2000 / 64
.include "graphics/hero.vcsprite"
