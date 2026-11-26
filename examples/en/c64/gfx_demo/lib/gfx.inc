; gfx.inc - C64: load character sets, screens and tile maps from exports.
;
; Usage:
;   .include "lib/gfx.inc"
;   ...
;   gfx_load_charset font, $2800          ; 2048 bytes (label from the ASM export)
;   gfx_charset_at $2800                  ; point the VIC at the new character set
;   gfx_load_screen title_chars, title_attrs
;   gfx_colors title_border, title_bg
;   tiles_use level_tiles_c0, level_tiles_col, level_tiles_props, level_tiles_tw, level_tiles_th, level_tiles_count, level_screens_lo, level_screens_hi, level_level_tiles_w, level_level_tiles_h
;   gfx_draw_map level_level_id            ; draw the tile map (screen) of the bank: thin wrapper around screen_draw of the tile library
;   ...
;   gfx_runtime                           ; ONCE, at a place that is not executed (e.g. behind the main program)
;   tiles_runtime                         ; ONCE, only if tile maps are drawn (subroutines of the tile library)
;
; Tile maps are drawn by the tile library (#include <c64/tiles.asm>, included below): design the tile set (.vctiles) and screen bank (.vcscreen)
; in the editor, include the file (`.include "graphics/<name>.vctiles"`) and bind them with tiles_use; commands and switches are described in the header of common/tiles.asm.
;
; Overview of the macros:
;   gfx_copy src, dst, len      general copy (uses gfx_memcpy)                      gfx_clear char, color      fill text and color
;   gfx_load_charset src, dst   copy a 2048-byte character set                      gfx_colors border, bg      border and background
;   gfx_charset_at addr         $D018: set the character set address                gfx_draw_map [sn]          draw tile map
;   gfx_screen_at addr          $D018 and $0288: set the screen address             gfx_load_screen chars, attrs  load screen + color RAM
;   gfx_vic_bank addr           select the VIC bank (16 KB)                         gfx_load_screen_chars chars   load the character codes only
; All macros expect constants (labels/numbers) as arguments. They change A and the flags; gfx_clear also X, the copy macros
; (gfx_copy, gfx_load_*) also X, Y and the zero page cells gfx_src, gfx_dst, gfx_len. An interrupt that uses the same zero page group
; must not run between the macro call and gfx_memcpy.
;
; Zero page used: $F7-$FE (pointers; the library uses the same group, never at the same time as gfx_memcpy):
;   gfx_len $F7/$F8 (length), gfx_src $FB/$FC (source), gfx_dst $FD/$FE (destination); $F9/$FA are left untouched by these macros.
; The screen is at gfx_screen (default $0400, changeable before the include with .const GFX_SCREEN = $0800 or -D GFX_SCREEN=$0800;
; must be aligned to 256 bytes). Color RAM is fixed at $D800. Map size: limits of the tile library.

.ifdef GFX_SCREEN
.const gfx_screen = GFX_SCREEN
.const TILE_SCREEN = GFX_SCREEN    ; the tile library draws into the same screen
.else
.const gfx_screen = $0400
.endif
#include <c64/tiles.asm>
.const gfx_colorram = $d800     ; color RAM (only bits 0-3 of each byte count)
.const gfx_len  = $f7           ; 16 bit: length when copying
.const gfx_src  = $fb           ; 16 bit: source
.const gfx_dst  = $fd           ; 16 bit: destination

; --- Copying ---------------------------------------------------------------------------------------------------
; gfx_copy(src, dst, len) - set zero page pointers and length and call gfx_memcpy (needs gfx_runtime somewhere in the program).
.macro gfx_copy(src, dst, len)      ; copies len bytes from src to dst (len at most 65535, areas must not overlap)
        lda #<src
        sta gfx_src
        lda #>src
        sta gfx_src+1
        lda #<dst
        sta gfx_dst
        lda #>dst
        sta gfx_dst+1
        lda #<len
        sta gfx_len
        lda #>len
        sta gfx_len+1
        jsr gfx_memcpy
.endmacro

; --- Character set --------------------------------------------------------------------------------------------------
; gfx_load_charset(src, dst) - src = label of the character set (e.g. from the export), dst = target address in RAM.
.macro gfx_load_charset(src, dst)   ; 256 characters = 2048 bytes; dst aligned to 2 KB and inside the VIC area (not $1000-$1FFF/$9000-$9FFF)
        gfx_copy src, dst, 2048
.endmacro

; gfx_charset_at(addr) - addr = address of the character set (aligned to 2 KB). In $D018, bits 1-3 are the pointer to the character set
;   in 2 KB steps inside the VIC bank; bits 4-7 (screen pointer) are kept.
.macro gfx_charset_at(addr)         ; set the character set address in VIC $d018 (bank 0: $0000-$3FFF; otherwise gfx_vic_bank first)
        lda $d018
        and #%11110001
        ora #((addr & $3fff) >> 11) << 1
        sta $d018
.endmacro

; gfx_screen_at(addr) - addr = address of the screen memory (aligned to 1 KB). In $D018, bits 4-7 are the screen pointer in
;   1 KB steps inside the VIC bank; bits 0-3 (character set pointer) are kept. The KERNAL writes its text output
;   to the page held in $0288 (here the upper address half of addr), so that PRINT and the cursor use the same screen.
.macro gfx_screen_at(addr)          ; set the screen address in VIC $d018 (1 KB grid inside the bank); KERNAL output follows via $0288
        lda $d018
        and #%00001111
        ora #((addr & $3fff) >> 10) << 4
        sta $d018
        lda #>addr
        sta $0288
.endmacro

; gfx_vic_bank(addr) - switch the VIC-II to another 16 KB bank. The bank bits are inverted in bits 0-1 of $DD00
;   (port A of CIA 2): bank 0 = %11, bank 3 = %00, hence 3 - (addr >> 14). Bits 0-1 of $DD02 (data direction) must be outputs.
;   The other bits of $DD00 (serial bus among others) are kept. Addresses afterwards (screen, character set) are bank-relative.
.macro gfx_vic_bank(addr)           ; select the VIC bank: addr = $0000, $4000, $8000 or $c000
        lda $dd02
        ora #3
        sta $dd02
        lda $dd00
        and #%11111100
        ora #3 - (addr >> 14)
        sta $dd00
.endmacro

; --- Screen ---------------------------------------------------------------------------------------------------
.macro gfx_load_screen_chars(chars) ; 1000 character codes to gfx_screen
        gfx_copy chars, gfx_screen, 1000
.endmacro

.macro gfx_load_screen(chars, attrs) ; character codes to gfx_screen, colors to color RAM
        gfx_copy chars, gfx_screen, 1000
        gfx_copy attrs, gfx_colorram, 1000
.endmacro

; gfx_colors(border, bg) - color numbers 0-15 as constants (border $D020, background color 0 $D021).
.macro gfx_colors(border, bg)       ; border and background color (e.g. from <name>_border/_bg of the export)
        lda #border
        sta $d020
        lda #bg
        sta $d021
.endmacro

; gfx_clear(char, color) - fills all 1000 cells of gfx_screen with the screen code `char` and color RAM with `color`.
;   A single loop with X = 0..255 writes four overlapping 256-byte windows ($000, $100, $200, $2E8); the last window
;   covers cells 744-999, so the 1000 cells are reached without a second loop (some cells are written twice).
.macro gfx_clear(char, color)       ; fill the screen with character and color
        ldx #0
@l:     lda #char
        sta gfx_screen,x
        sta gfx_screen+$100,x
        sta gfx_screen+$200,x
        sta gfx_screen+$2e8,x
        lda #color
        sta gfx_colorram,x
        sta gfx_colorram+$100,x
        sta gfx_colorram+$200,x
        sta gfx_colorram+$2e8,x
        inx
        bne @l
.endmacro

; --- Tile map ---------------------------------------------------------------------------------------------------
; The tile library draws the map. The wrapper stays so that existing programs can keep using gfx_draw_map;
; tiles_use must bind the tile set and the screen bank first. Characters and color RAM are produced as before from screen position 0/0.
.macro gfx_draw_map(sn=255)         ; draw screen sn of the bank (without argument: the one selected last)
        screen_draw sn
.endmacro

; --- Run time routines (include once) -----------------------------------------------------------------------------
; gfx_memcpy - subroutine behind gfx_copy. Input: gfx_src (source), gfx_dst (destination), gfx_len (length), all 16 bit in zero page.
;   First copies len/256 full pages (Y runs 0..255, then the upper address bytes advance), then the rest (len & 255, counted
;   down via X). Changes A, X, Y and the flags as well as gfx_src, gfx_dst (advance by the copied whole pages) and
;   gfx_len+1 (0 afterwards). No overlap handling: source and destination must not overlap. Not interrupt-safe if
;   an interrupt uses the same zero page cells.
.macro gfx_runtime
gfx_memcpy:                         ; gfx_src -> gfx_dst, gfx_len bytes
        ldy #0
@page:  lda gfx_len+1
        beq @tail                   ; no full page left
@l1:    lda (gfx_src),y
        sta (gfx_dst),y
        iny
        bne @l1
        inc gfx_src+1
        inc gfx_dst+1
        dec gfx_len+1
        jmp @page
@tail:  ldx gfx_len                 ; rest below 256 bytes
        beq @done
@l2:    lda (gfx_src),y
        sta (gfx_dst),y
        iny
        dex
        bne @l2
@done:  rts
.endmacro
