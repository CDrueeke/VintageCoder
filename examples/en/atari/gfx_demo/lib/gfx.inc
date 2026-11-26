; =====================================================================================================================
; gfx.inc - Atari 8-bit: load character sets, screens and tile maps from exports (ANTIC mode 2, 40x24 text)
; =====================================================================================================================
;
; PURPOSE
;   Small macro library that brings the exports of the editor (vcc media export, or include the file directly: character set <name>,
;   screen <name>_chars, tile set <name>_c0.., screen bank <bank>_screens_lo/_hi, colors <bank>_<screen>_bg/_fg) onto the
;   text screen.  The operating system stays active: colors, character set and display list are set only through the
;   OS shadow registers, which the OS copies into ANTIC/GTIA in the next vertical blank.
;
; USAGE
;   .include "lib/gfx.inc"
;   ...
;   gfx_load_charset afont, gfx_charset, afont_len   ; 128 characters = 1024 bytes to gfx_charset (1 KB boundary)
;   gfx_load_screen atitle_chars                     ; 960 character codes to gfx_screen
;   gfx_set_display                                  ; switch on display list (gfx_dlist) + character set
;   gfx_colors atitle_bg, atitle_fg                  ; background (COLPF2/border), text color (COLPF1: only the luminance counts)
;   tiles_use alevel_tiles_c0, 0, alevel_tiles_props, alevel_tiles_tw, alevel_tiles_th, alevel_tiles_count, alevel_screens_lo, alevel_screens_hi, alevel_level_tiles_w, alevel_level_tiles_h
;   gfx_draw_map alevel_level_id        ; draw tile map (screen) of the bank: thin wrapper around screen_draw of the tile library
;   ...
;   gfx_runtime                                      ; ONCE, after the main program (is not executed)
;   tiles_runtime                                    ; ONCE, only if tile maps are drawn (subroutines of the tile library, zero page $F7-$FE)
;   gfx_dlist                                        ; ONCE: display list (32 bytes, aligned automatically)
;
;   Macros at a glance (all change A, X, Y and the zero page $CB-$D0; none saves registers):
;     gfx_copy src, dst, len          copy an area (areas must not overlap), calls gfx_memcpy
;     gfx_load_charset src, dst, len  = gfx_copy (len comes from the export as <name>_len)
;     gfx_load_screen chars           = gfx_copy chars -> gfx_screen, 960 bytes
;     gfx_clear char                  fill screen memory with a character code (see the note at the macro)
;     gfx_set_display                 set OS shadow for display list (SDLSTL/H) and character set page (CHBAS)
;     gfx_colors bg, fg               set OS color shadow COLOR2/COLOR4 (background, border) and COLOR1 (text)
;     gfx_dlist                       creates the display list (macro with data, not executable)
;     gfx_draw_map [sn]               draw tile map (screen sn of the bank), wrapper around screen_draw of the tile library
;     gfx_runtime                     subroutine gfx_memcpy (not executable)
;
; MEMORY MAP (changeable before the include with GFX_SCREEN / GFX_CHARSET)
;   Screen      $4000 (960 bytes; must not cross a 4 KB boundary, the low byte must be 0: gfx_map_draw checks it with .assert)
;   Character set $4400 (1 KB, aligned to a 1 KB boundary so that CHBAS = page number (high byte) fits)
;   Display list: in the program, 32 bytes, aligned with `.align 32` (must not cross a 1 KB boundary)
;   The data of character set and screen are in the program image (export) and are copied to these
;   addresses at run time; the target addresses themselves are NOT part of the .xex file.
;   Zero page: $CB-$CC source (gfx_src), $CD-$CE target (gfx_dst), $CF-$D0 length (gfx_len) - each 16 bit, low byte first.
;   OS shadow registers: SDLSTL $0230/$0231, CHBAS $02F4, COLOR0..COLOR4 $02C4..$02C8 (used: COLOR1, COLOR2, COLOR4).
;
; CONVENTIONS AND LIMITS
;   * Screen code 0 is the space; bit 7 of a character code inverts the character (characters 128-255 of the 128-character set).
;   * Map size and tile size: limits of the tile library (header of common/tiles.asm); the map starts at screen position 0/0.
;   * The macros are not meant for interrupts (shared zero page, no register is saved).

.ifdef GFX_SCREEN
.const gfx_screen = GFX_SCREEN
.const TILE_SCREEN = GFX_SCREEN    ; the tile library draws into the same screen
.else
.const gfx_screen = $4000
.endif
.ifdef GFX_CHARSET
.const gfx_charset = GFX_CHARSET
.else
.const gfx_charset = $4400
.endif
#include <atari8bit/tiles.asm>
.const gfx_sdlstl = $0230       ; OS shadow: display list (low/high)
.const gfx_chbas  = $02f4       ; OS shadow: character set page
.const gfx_color0 = $02c4       ; OS shadow COLPF0..COLPF3, COLBK = gfx_color4
.const gfx_color1 = $02c5       ; shadow of COLPF1 (text color, in mode 2 only the luminance counts)
.const gfx_color2 = $02c6       ; shadow of COLPF2 (background of the text field)
.const gfx_color4 = $02c8       ; shadow of COLBK (border)
.const gfx_src  = $cb           ; 16 bit: source
.const gfx_dst  = $cd           ; 16 bit: target
.const gfx_len  = $cf           ; 16 bit: length

; gfx_copy(src, dst, len): loads source, target and length into the zero page and calls gfx_memcpy.
; In: src, dst = addresses, len = number of bytes (constants or labels).  Out: A, X, Y changed, gfx_src/dst/len destroyed.
.macro gfx_copy(src, dst, len)      ; copies len bytes from src to dst (areas must not overlap)
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

.macro gfx_load_charset(src, dst, len)   ; character set (len from the export: <name>_len) to dst
        gfx_copy src, dst, len
.endmacro

.macro gfx_load_screen(chars)       ; 960 character codes to gfx_screen
        gfx_copy chars, gfx_screen, 960
.endmacro

; gfx_clear(char): fills the screen memory with a character code.
; Exactly 960 bytes are written (3 full pages + 192 bytes), so that nothing behind the screen (e.g. with a changed
; GFX_CHARSET) is overwritten.
; Changes A, X, Y, gfx_dst.
.macro gfx_clear(char)              ; fill the screen with a character code
        lda #<gfx_screen
        sta gfx_dst
        lda #>gfx_screen
        sta gfx_dst+1
        ldx #3                      ; 3 full pages = 768 bytes, then another 192 bytes
        ldy #0
        lda #char
@l:     sta (gfx_dst),y
        iny
        bne @l
        inc gfx_dst+1
        dex
        bne @l
@t:     sta (gfx_dst),y             ; rest: Y continues from 0 (after the last page Y = 0)
        iny
        cpy #192
        bne @t
.endmacro

; gfx_set_display: set shadow registers to our display list (label gfx_dl, see gfx_dlist) and the character set.
; Takes effect only in the next vertical blank (the OS then copies SDLSTL and CHBAS into ANTIC).  Changes A.
.macro gfx_set_display              ; activate display list and character set (OS shadow registers)
        lda #<gfx_dl
        sta gfx_sdlstl
        lda #>gfx_dl
        sta gfx_sdlstl+1
        lda #>gfx_charset           ; CHBAS = page number (high byte) of the character set
        sta gfx_chbas
.endmacro

; gfx_colors(bg, fg): bg -> COLOR2 (text field background) and COLOR4 (border), fg -> COLOR1 (text, luminance only).
.macro gfx_colors(bg, fg)           ; background/border and text color (GTIA color values, e.g. <name>_bg/_fg of the export)
        lda #bg
        sta gfx_color2
        sta gfx_color4
        lda #fg
        sta gfx_color1
.endmacro

; gfx_dlist: display list, 32 bytes: 3 x $70 (8 blank lines each = 24 raster lines top border), $42 (mode 2 with LMS: the
; screen address follows), address gfx_screen, 23 x $02 (mode 2), $41 (jump and wait for end of screen) with address gfx_dl.
; `.align 32`: 32 bytes from an address divisible by 32 never cross a 1 KB boundary (ANTIC increments the display list
; address only within 1 KB).
.macro gfx_dlist                    ; display list: 24 lines of ANTIC mode 2 from gfx_screen (label gfx_dl)
        .align 32
gfx_dl: .byte $70, $70, $70, $42
        .word gfx_screen
        .repeat 23
        .byte $02
        .endrepeat
        .byte $41
        .word gfx_dl
.endmacro

; gfx_draw_map(sn): draw tile map (screen sn of the bank).  The map is drawn by the tile library (#include <atari8bit/tiles.asm>, included above);
; this wrapper stays so that existing programs can keep using gfx_draw_map.  Before that, tiles_use must bind the tile set and the screen bank
; (example in gfx_demo.asm).  Without an argument the most recently selected screen is drawn.
.macro gfx_draw_map(sn=255)
        screen_draw sn
.endmacro

; gfx_runtime: the subroutines and variables.  Include once after the main program (is never "run through").
.macro gfx_runtime
; gfx_memcpy: copies gfx_len bytes from (gfx_src) to (gfx_dst), upward.  In: the three pointers in the zero page.
; Out: A, X, Y changed; gfx_src/gfx_dst point to the page after the last completely copied area, gfx_len+1 = 0.
; Length 0 is allowed.  Method: first whole pages (Y runs 0..255), then the rest (low byte of gfx_len).
gfx_memcpy:                         ; gfx_src -> gfx_dst, gfx_len bytes
        ldy #0
@page:  lda gfx_len+1
        beq @tail
@l1:    lda (gfx_src),y
        sta (gfx_dst),y
        iny
        bne @l1
        inc gfx_src+1
        inc gfx_dst+1
        dec gfx_len+1
        jmp @page
@tail:  ldx gfx_len
        beq @done
@l2:    lda (gfx_src),y
        sta (gfx_dst),y
        iny
        dex
        bne @l2
@done:  rts
.endmacro
