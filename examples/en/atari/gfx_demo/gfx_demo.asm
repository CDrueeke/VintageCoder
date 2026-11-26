; =====================================================================================================================
; gfx_demo.asm - load character set, screen and tile map from the editor exports (Atari 8-bit, text mode)
; =====================================================================================================================
;
; PURPOSE
;   Shows how graphics data from the project (graphics/afont.vcchar = character set, graphics/atitle.vcscreen = 40x24 title screen,
;   graphics/alevel_tiles.vctiles = tile set of 2x2 tiles, graphics/alevel.vcscreen = screen bank with the 20x12 map) is brought with the macro library lib/gfx.inc onto the Atari text screen
;   (ANTIC mode 2, 40x24).  The operating system stays active; display list, character set and colors are set
;   through the OS shadow registers (details and macros: header of lib/gfx.inc).
;
; PREPARATION
;   The files graphics/atitle.vcscreen, alevel_tiles.vctiles (tile set) and alevel.vcscreen (screen bank) are converted from the graphics files when they are included
;   (`vcc build` or `vcc media export graphics/<name>.<extension>`, project folder = current directory).  They provide the labels atitle_chars,
;   alevel_tiles_chars (target character set of the tile set = afont.vcchar, all 128 characters pinned), alevel_tiles_c0.. and alevel_screens_lo/_hi as well as the
;   constants alevel_tiles_chars_len, atitle_bg/_fg and alevel_level_id/_bg/_fg/_tiles_w/_tiles_h.
;
; FLOW AND VERSIONS (switch -D DEMO_MAP=1)
;   * Without the switch: copy the character set to gfx_charset ($4400), copy the 960 character codes of the title screen to gfx_screen ($4000),
;     set colors.
;   * With -D DEMO_MAP=1: copy the character set, clear the screen with character 0 (space), draw the map alevel with its tile list
;     (20x12 tiles = 40x24 characters), set colors.
;   Then gfx_set_display switches on the display list and the character set; the program stays in an endless loop.
;
; MEMORY MAP
;   $2000 program, library code (gfx_runtime), display list (gfx_dlist, aligned to 32 bytes) and the export data
;   (character set 1024 bytes, screen 960 bytes, tile set 16 bytes, map 240 bytes) directly one after another from $2000.
;   Created at run time (not in the .xex): screen $4000-$43BF, character set $4400-$47FF.  Zero page $CB-$D0.
; BUILD:  vcc build . (project folder) or, after the export:  vcc asm gfx_demo.asm --target atari8 --cpu sally -o gfx_demo.xex [-D DEMO_MAP=1]
; =====================================================================================================================
.target atari8
.cpu sally { illegals = deny }

.org $2000
.include "lib/gfx.inc"

start:  gfx_load_charset alevel_tiles_chars, gfx_charset, alevel_tiles_chars_len   ; copy 1024 bytes of character set to the 1 KB boundary $4400
.ifdef DEMO_MAP
        gfx_clear 0                 ; fill the screen with spaces (the map then overwrites it completely)
        gfx_draw_map alevel_level_id        ; draw a screen of the bank (tile library, tiles_use see below)
        gfx_colors alevel_level_bg, alevel_level_fg
.else
        gfx_load_screen atitle_chars        ; 960 screen codes of the title to $4000
        gfx_colors atitle_bg, atitle_fg
.endif
        gfx_set_display                     ; display list + character set (takes effect in the next vertical blank)
done:   jmp done

gfx_runtime                         ; subroutines of the library (here, so that they are not executed)
gfx_dlist                           ; display list (referenced from gfx_set_display)
.include "graphics/atitle.vcscreen"
.include "graphics/alevel_tiles.vctiles"
.include "graphics/alevel.vcscreen"

; Tile library: bind set alevel_tiles (2x2, without color memory) and screen bank alevel, then the subroutines
        tiles_use alevel_tiles_c0, 0, alevel_tiles_props, alevel_tiles_tw, alevel_tiles_th, alevel_tiles_count, alevel_screens_lo, alevel_screens_hi, alevel_level_tiles_w, alevel_level_tiles_h
        tiles_runtime
.run start
