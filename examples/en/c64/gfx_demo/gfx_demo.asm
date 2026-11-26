; gfx_demo.asm - load character set, screen and tile map from the editor exports (C64)
;
; Purpose: shows how graphics data from the editors (character set, screen and tile editor) get into a program: the data
;   are project files in graphics/ (font.vcchar, title.vcscreen, level_tiles.vctiles, level.vcscreen); building turns
;   them into assembler text where this program includes them. The helper macros are in lib/gfx.inc.
;
; Preparation: `vcc build examples/c64/gfx_demo` does the export itself. By hand: export in the editor or
;   `vcc media export graphics/<name>.vc<kind>` (title, level_tiles = tile set, level = screen bank). Then load the PRG, RUN.
;
; Two versions (selected with -D, e.g. `vcc asm gfx_demo.asm --target c64 -D DEMO_MAP=1 -o ...`):
;   no switch      title screen: the character set (graphics/font.vcchar, a custom set with 256 characters) is copied to $2800
;                  and activated, the 1000 character codes and the 1000 colors of "title" go into screen and color RAM, border and
;                  background get the colors of the screen (14 and 6). The result is a small test picture with a box
;                  at the top left and a few characters of the custom set.
;   -D DEMO_MAP=1  tile map: the screen is cleared with spaces, border and background get the colors of the map
;                  (14 and 6), and the tile map "level" (20 x 12 tiles of 2 x 2 characters = 40 x 24 characters, tiles: empty, stone,
;                  grass, crate) is drawn with the tile library. The program then stays in an endless loop.
;   Because of the custom character set, the hand-drawn glyphs appear instead of normal text (hires characters of 8 x 8 dots).
;
; Hardware: VIC-II $D018 (pointer to screen memory and character set), $D020 (border color), $D021 (background color 0),
;   screen memory $0400, color RAM $D800, KERNAL cell $0288 (page of the screen memory for text output).
; Memory map: program, BASIC line `10 SYS 2061` and all export data are in one block from $0801 to $22A3 (6819 bytes; the
;   export `level` also contains a character set `level_chars` of 2048 bytes, which this program does not use). The character set
;   is copied at run time to $2800-$2FFF (VIC bank 0), i.e. behind the end of the program ($22A3). Zero page $F7-$FE (see gfx.inc).
; Limits: interrupts stay disabled (SEI without CLI), the program does not return; leave only with RESET.
.target c64
.cpu 6510 { illegals = deny }

.org $0801
.basicstub start

.include "lib/gfx.inc"          ; macros gfx_*, also includes the tile library c64/tiles.asm

start:  sei
        gfx_load_charset level_tiles_chars, $2800   ; target character set of the tile set = font.vcchar (all 256 characters pinned)
        gfx_charset_at $2800                        ; make the VIC-II point to the character set at $2800
        gfx_screen_at gfx_screen                    ; screen memory at $0400 (set the pointer, adjust KERNAL page $0288)
.ifdef DEMO_MAP
        gfx_clear 32, level_level_bg                ; screen: spaces (code 32), color RAM: background color of the map (invisible)
        gfx_colors level_level_border, level_level_bg
        gfx_draw_map level_level_id                 ; draw screen 0 of the bank "level" with the tile library
.else
        gfx_load_screen title_chars, title_attrs    ; 1000 character codes -> $0400, 1000 colors -> $D800
        gfx_colors title_border, title_bg
.endif
done:   jmp done                ; endless loop: the picture stays

gfx_runtime                     ; subroutine gfx_memcpy of the copy macros (behind the main program, not executed)
.include "graphics/title.vcscreen"       ; data: title_chars, title_attrs (1000 bytes each) and constants title_border, title_bg
.include "graphics/level_tiles.vctiles" ; tile set level_tiles (2x2): tables _c0.._c3, _col, _props, level_tiles_chars (2048 bytes)
.include "graphics/level.vcscreen"       ; screen bank level: map field, address tables level_screens_lo/_hi and constants level_*

; Tile library: bind set level_tiles (2x2) and screen bank level, then the subroutines
        tiles_use level_tiles_c0, level_tiles_col, level_tiles_props, level_tiles_tw, level_tiles_th, level_tiles_count, level_screens_lo, level_screens_hi, level_level_tiles_w, level_level_tiles_h
        tiles_runtime
