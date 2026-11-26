; =====================================================================================================================
; hal_atari.asm - hardware layer (HAL) of the Atari 8 bit for "Hugo & Gumbo"
; =====================================================================================================================
;
; The game logic (lib/core.asm) knows no hardware. This file translates its requests into accesses to
; the Atari chips.  The logic calculates in a logical grid of 320 x 200 points; the HAL turns that into color clocks,
; character cells and player lines (1 color clock = 2 logical points, 1 line = 1 logical point in height).
;
; INTERFACE (described completely in core.asm; here, what this HAL provides and needs)
;   for the core:            hal_sfx (A = effect 0..7), hal_put_tile (X = column, Y = row, A = tile), hal_score_draw,
;                            sfx_on (byte: not 0 while an effect is sounding)
;   for the main program:    hal_init, hal_wait_frame, hal_blank, hal_unblank, hal_title, hal_load_level (A = level),
;                            hal_update_sprites (reads h_px_l/h/h_py/h_fr and e_px_l/h/e_py/e_fr); also SCREEN, CH_BLANK, the
;                            shadow registers COLOR0.. and the table skyc (used directly by show_title)
;   internal helpers:        hal_hud, put_bcd,
;                            clear_pm, sfx_update, dli, vbi
;   The HAL reads from the core: lvl, sc0..sc2 and the output values; from lib/levels.inc: lvl_pf0..2, lvl_sky0..5, txt_hud,
;   txt_lv_lo/hi, CH_DIGIT0, CH_STAR, the player images hero_p0..p3 and enemy_m; from the export: title_chars; the tile library (tile_put, screen_select_a, screen_draw_all).  It changes only its own zero page cells and may change A, X, Y and the flags freely in all routines.
;   Differences from the other systems (sprites, sound, start): see core.asm, section "Differences".
;
;   ANTIC (screen)
;     * Text MULTICOLOR MODE 4 (ANTIC mode 4, in BASIC GRAPHICS 12), 40 x 24 characters.  A character is 4 x 8 wide pixels
;       (one pixel = one color clock = 2 hires dots, hence 160 color clocks wide).  Each pixel has two bits:
;           00 = background COLBK       01 = COLPF0       10 = COLPF1       11 = COLPF2  (character bit 7 set: COLPF3 instead of COLPF2)
;       Character codes therefore run only from 0 to 127 (7 bits); bit 7 is not an "inverse" display but the color choice.
;       The color applies to the WHOLE screen (unlike the C64 there is no color RAM per character). For each level
;       COLPF0..2 are set anew; grass and bush carry bit 7 and therefore appear in COLPF3 (green, the same in all levels).
;     * The character set (128 characters = 1 KB) is at $3C00 (it must start on a 1 KB boundary; before it are the tile set tables, which must not lie in screen memory $4000-$43BF), the screen memory
;       (24 * 40 = 960 bytes) at $4000, the display list at $4800.  The screen memory must not cross a 4 KB boundary
;       (ANTIC counts on only 12 address bits); $4000-$43BF lies completely within one 4 KB block.
;     * DISPLAY LIST: program for ANTIC: 24 blank lines, 24 text lines in mode 4 (the first with LMS address), at the end a jump
;       back to the start.  At five lines (3, 7, 11, 15, 19) the DLI bit is set: there ANTIC triggers a
;       DISPLAY LIST INTERRUPT, in which we change the background color.  This makes the sky of six bars
;       (lines 0-3, 4-7, 8-11, 12-15, 16-19, 20-23; each bar is four text lines = 32 scan lines high).
;
;   GTIA + ANTIC (figures): PLAYER-MISSILE GRAPHICS, single-line resolution
;     * Players are stripes 8 color clocks wide that reach over the whole screen height; the bits of a byte of the
;       player memory are the pixels of ONE scan line (bit 7 left).  Missiles are four stripes 2 bits wide that share a
;       common byte per line.  ANTIC fetches the bytes by DMA, GTIA draws them, HPOSPn/HPOSMn set the x position.
;     * The HERO consists of FOUR players on top of each other (each 8 color clocks wide, one color clock = one multicolor dot of the text):
;           P0 = outline and eye (black)   P1 = skin   P2 = shirt   P3 = trousers.
;       Player 0 lies in front of the others (priority), so it draws the outline.  The four stripes have the same x position and
;       their bits never overlap: each pixel belongs to exactly one player, which gives a four-color figure.  A
;       finer "hires overlay" is not possible on the Atari: the smallest pixel of a player is one color clock.
;     * The ENEMY consists of the four MISSILES (each 2 clocks wide), side by side as the "5th player" in color COLPF3 (PRIOR bit 4):
;       so they have the green color of the grass and together form 8 clocks of width, the width of one player.
;     * Player memory: one byte = one screen line.  Page $50 + 0..7: missiles at $5300, P0 $5400, P1 $5500, P2 $5600, P3 $5700.
;       (In single-line resolution a player block is 256 bytes and the memory 2 KB long; PMBASE points to $5000, of which
;       the first $300 bytes are unused.)
;     * The first visible screen line has index 32 in this memory (8 lines of border of normal width + 24 blank lines of the
;       display list).  Everything before and after lies outside the text screen and stays empty.
;
;   POKEY (sound)
;     * Music in RMT format (original player by Raster, lib/rmtplayr.asm, writes all four POKEY channels in every frame).  The song
;       (music/tune.vcsong) uses only tracks 1-3; channel 4 (AUDF4/AUDC4) stays free for the sound effects: after playing the
;       music, the effect part of this file overwrites channel 4 (see vbi).  For the location of the data see "Sound effects" below.
;
;   OPERATING SYSTEM (OS): We use it only a little.  The OS maintains "shadow registers" in RAM (SDMCTL, COLOR0.., PCOLR0.., GPRIOR,
;   CHBAS, SDLSTL), which it copies into the chip registers in every vertical blank; most chip registers cannot be read and
;   a write directly to the chip register would be overwritten by the OS in the next frame.  Only registers WITHOUT a shadow
;   (HPOSPn, GRACTL, PMBASE, NMIEN, WSYNC, POKEY) are written directly.  Effect of a shadow value: only in the next frame.
;
; MEMORY MAP:    $2000-$3FFF program   $3C00 character set   $4000 screen   $4800 display list   $5000-$57FF player memory
;                $5800 level map (LMAP)   $5CE0-$5FFF RMT tables   $6000 RMT player   $7000 tables, screen data, music module
; ZERO PAGE:    $80-$B3 game logic, $B4-$BB this file, $CB-$DD RMT player
;                The interrupts (dli, vbi, sfx_update) use no zero page cell except the player (RMT) itself.
; =====================================================================================================================

.const SCREEN  = $4000
.const CHARSET = $3C00
.const DLIST   = $4800
.const PM_M    = $5300
.const PM_P0   = $5400            ; P1 = $5500, P2 = $5600, P3 = $5700

; ---- Operating system shadow registers (copied into the chips by the OS in the vertical blank)
.const SDMCTL  = $022f            ; DMA: bit 5 display list, bit 4 single line, bit 3 players, bit 2 missiles, bits 0-1 width (2 = normal)
.const SDLSTL  = $0230            ; address of the display list (low, high)
.const GPRIOR  = $026f            ; priority: bit 4 = missiles as 5th player (COLPF3), bit 0 = players in front of playfield
.const PCOLR0  = $02c0            ; player colors 0..3
.const COLOR0  = $02c4            ; COLPF0..COLPF3
.const COLOR4  = $02c8            ; COLBK
.const CHBAS   = $02f4            ; page of the character set
.const VDSLST  = $0200            ; vector of the display list interrupt
.const RTCLOK  = $14              ; frame counter (low), counts 50 (PAL) or 60 (NTSC) times per second
.const SETVBV  = $e45c            ; OS routine: set vector (A = 7: deferred vertical-blank interrupt, X = high, Y = low)
.const XITVBV  = $e462            ; end of a deferred vertical-blank interrupt

; ---- Chip registers
.const HPOSP0  = $d000            ; horizontal position players 0..3 ($d000-$d003) and missiles 0..3 ($d004-$d007), in color clocks
.const GRACTL  = $d01d            ; bit 0 enable missiles, bit 1 enable players
.const PMBASE  = $d407            ; page of the player memory
.const WSYNC   = $d40a            ; write: wait for the end of the line
.const COLBK   = $d01a
.const NMIEN   = $d40e            ; bit 7 DLI, bit 6 vertical-blank interrupt
.const AUDF4   = $d206
.const AUDC4   = $d207

; Pointers for (indirect),y accesses and scratch memory of the HAL (zero page; the game logic uses $80-$B3, the RMT player $CB-$DD)
.const sp  = $b4                  ; pointer (2 bytes)
.const dp  = $b6                  ; second pointer (2 bytes), mostly the destination
.const pt0 = $b8                  ; scratch memory (used differently by each routine, see there)
.const pt1 = $b9
.const pt2 = $ba
.const pt3 = $bb

; ---------------------------------------------------------------- Start
; hal_init: set up the chips.  The screen stays dark (SDMCTL = 0) until hal_unblank switches it on.  Changes A, X, Y.
;   Note: display list (DLIST), screen, character set and skyc colors must be ready before hal_unblank; the display list
;   is copied by the main program (jumprun.asm, start) before the call of hal_init.
;   All color values are Atari colors: high nibble = hue (0 gray, 2 orange, 3 red-orange, 8 blue, 12 green ...),
;   low nibble = brightness (0..14, even values).
hal_init:
        lda #0
        sta SDMCTL               ; no DMA: screen off
        sta sfx_on               ; no sound effect active
        lda #<DLIST
        sta SDLSTL               ; pointer to the display list (the OS copies it to DLISTL/H in the next frame)
        lda #>DLIST
        sta SDLSTL+1
        lda #>CHARSET
        sta CHBAS                ; only the page: $3C -> character set at $3C00
        lda #>(PM_M - $300)           ; player memory: page $50 (missiles lie $300 behind it)
        sta PMBASE
        lda #3
        sta GRACTL               ; enable players and missiles
        lda #$11
        sta GPRIOR               ; players in front of the playfield, missiles as 5th player
        lda #$00
        sta PCOLR0               ; P0 outline: black
        lda #$3c
        sta PCOLR0+1             ; P1 skin
        lda #$36
        sta PCOLR0+2             ; P2 shirt: red
        lda #$86
        sta PCOLR0+3             ; P3 trousers: blue
        lda #$c6
        sta COLOR0+3             ; COLPF3: green (grass, bush and the enemy)
        ; Hook in the display list interrupt and the frame tick
        lda #<dli
        sta VDSLST               ; DLI vector: the OS jumps here from the NMI
        lda #>dli
        sta VDSLST+1
        lda #7
        ldx #>vbi
        ldy #<vbi
        jsr SETVBV               ; set deferred vertical-blank vector (VVBLKD) to vbi
        lda #$c0
        sta NMIEN                ; enable DLI (bit 7) and vertical-blank interrupt (bit 6)
        jsr clear_pm
        rts

; Display list: 3 x 8 blank lines (24 lines of top border), then 24 text lines mode 4; at lines 3, 7, 11, 15, 19 the DLI bit ($80).
;   Command bytes of ANTIC:  $70 = 8 empty scan lines (background color)   $44 = mode 4 + bit 6 (LMS: the 16-bit address
;   of the screen memory follows)   $04 = mode 4, memory simply continues (40 bytes per line)   $84 = like $04, but with DLI bit 7
;   $41 = JVB: jump to the following address and wait for the end of the frame (display list from the start).
;   The list lies in the program area and is copied to DLIST ($4800) at start (see jumprun.asm), presumably so that the
;   XEX file needs no segment of its own at $4800.  (A display list must not cross a 1 KB boundary; the 32 bytes
;   at $4800 do not.)
;   The loop counts i = 0..22 for text lines 1..23; line 0 is the $44 before it.  So i = 2 is text line 3, etc.
dlist_data:
        .byte $70,$70,$70
        .byte $44,<SCREEN,>SCREEN
        .repeat 23, i
        .if i == 2 || i == 6 || i == 10 || i == 14 || i == 18
        .byte $84
        .else
        .byte $04
        .endif
        .endrepeat
        .byte $41,<DLIST,>DLIST
dlist_end:

; ---------------------------------------------------------------- Interrupts
; DLI: triggered at the end of a band line and sets the background color for the next bar.  WSYNC waits until the end of the line,
; so that the color changes exactly with the next line (otherwise a color wedge appears in the middle of the line).
;   ANTIC reports the DLI at the start of the LAST scan line (of 8) of the marked text line.  The color from the table is fetched into A
;   before WSYNC, so that after waking up only a single write access (sta COLBK) remains.
;   Registers: A and X are saved (pha / txa pha ...) and restored at the end; Y stays untouched.  rti ends the interrupt.
;   dli_i = number of the next bar (1..5): bar 0 is the color that the OS copies from COLOR4 to COLBK.
;   Duration: only a few clocks plus the wait for WSYNC; uses no zero page.
dli:    pha
        txa
        pha
        ldx dli_i
        lda skyc,x
        sta WSYNC
        sta COLBK
        inx
        stx dli_i
        pla
        tax
        pla
        rti
dli_i:  .byte 1                             ; number of the bar that the next DLI sets (vbi sets 1)
skyc:   .byte $9c,$9a,$98,$96,$94,$92       ; background color per bar (initial values; hal_load_level / show_title overwrite them for each level;
                                            ; bar 0 is set by hal_unblank via COLOR4, bars 1-5 by the DLI)

; VBI (deferred vertical-blank interrupt, 50/60 times per second): reset bar counter, music, effects.
;   "Deferred" means: the OS has already done its own work (copy shadow registers, increment RTCLOK) and saved
;   the registers; we end with XITVBV, which restores them.  The bar counter is set to 1 because bar 0 runs via COLOR4.
;   RASTERMUSICTRACKER+3 is the "play" entry of the RMT player (+0 = init, +9 = stop); it writes the POKEY channels 1-4.
vbi:    lda #1
        sta dli_i                ; next DLI starts again at bar 1
        jsr RASTERMUSICTRACKER+3 ; RMT player: one step of the music (writes all four POKEY channels)
        jsr sfx_update           ; effect afterwards: overwrites channel 4
        jmp XITVBV               ; OS end of the deferred VBI: restores the registers (no rti here)

; hal_wait_frame: wait for the next frame; RTCLOK counts up in every frame.  Changes only A.
;   The loop reads the counter once and waits until it has changed (the OS increments it in the vertical blank).  So the
;   game logic runs exactly once per frame: 50 times per second (PAL).
hal_wait_frame:
        lda RTCLOK
-       cmp RTCLOK
        beq -
        rts

; hal_blank / hal_unblank: screen off/on (hal_blank waits 2 frames and then deletes all figures; hal_unblank changes only A).
; While a new level is being built the screen stays one color, no half-finished picture is seen.
; With SDMCTL = 0 ANTIC fetches no more data (no picture, no players); hal_blank first waits for a frame end, so that the switch
; does not fall in the middle of the frame.  The new value takes effect only in the next vertical blank.
hal_blank:
        jsr hal_wait_frame
        lda #0
        sta SDMCTL               ; screen and DMA off
        sta COLOR4               ; background black: without DMA ANTIC shows only COLBK (otherwise a one-color intermediate picture in sky color)
        jsr hal_wait_frame       ; both values take effect in the next vertical blank; only then the old picture is gone and the rebuild invisible
        jmp clear_pm
;   $3E = %00111110: bit 5 display list, bit 4 single-line players, bit 3 players, bit 2 missiles, bits 1-0 = 10 normal width
;   (40 characters = 160 color clocks).  Only with bit 5 (display list) is anything visible at all.
;   The sky color of the top bar (skyc, set by hal_load_level / show_title) goes into COLOR4 here: color and DMA
;   are taken over in the same vertical blank, so the new picture appears in one go.
hal_unblank:
        lda skyc
        sta COLOR4               ; COLBK for the top bar (the DLI fetches the others)
        lda #$3e
        sta SDMCTL               ; display list, players, missiles, single line, normal width
        rts

; clear_pm: clear all player and missile memory (figures invisible) and reset the remembered lines.  Changes A, X.
;   The loop runs with X from 0 to 255 (overflow to 0 ends it) and clears one byte in each of the five pages per pass.
;   old_top / old_etop = start line at which the figures were last drawn (see hal_update_sprites).
clear_pm:
        lda #0
        tax
-       sta PM_M,x
        sta PM_P0,x
        sta PM_P0+$100,x
        sta PM_P0+$200,x
        sta PM_P0+$300,x
        inx
        bne -
        sta old_top
        sta old_etop
        rts

; ---------------------------------------------------------------- Tiles
; A tile = 2 x 2 characters.  Tile (c, k) lies in the cells (2c, 2k) ... (2c+1, 2k+1); a text line has 40 bytes.  Drawing is done by the
; tile library (#include <atari8bit/tiles.asm>, included in jumprun.asm and bound with tiles_use to the tile set hg and the screen bank levels):
;   hal_put_tile   = tile_put: place one tile.  Input X = column (0..19), Y = row (0..11), A = tile number.  Grass and bush already carry in the
;                    character tables of the set the character bit 7 (cell flag "sel" in the tile set): color COLPF3 = green.
;   screen_select_a: A = level number; copies the map of the level to LMAP (TILE_MAPRAM): the game logic deletes coins from this copy.
;   screen_draw_all: draws the whole level from LMAP (12 rows x 20 columns).  Takes several frames; therefore with the screen switched off.
;   Unlike on the C64 there is no color RAM here: the color is carried by bit 7 of the character code alone.
.const hal_put_tile = tile_put

; hal_title: copy the title screen, 960 character codes (24 lines * 40), into the screen memory.  Changes A, X, Y, sp, dp.
;   The source is title_chars (drawn in the screen editor, file graphics/title.vcscreen).  Copied are 3 full pages
;   (3 * 256 = 768 bytes) and then 192 more bytes: 960 together.  sp = source, dp = destination; both pointers are incremented page by page.
hal_title:
        lda #<title_chars
        sta sp
        lda #>title_chars
        sta sp+1
        lda #<SCREEN
        sta dp
        lda #>SCREEN
        sta dp+1
        ldx #3
.pg:    ldy #0
-       lda (sp),y
        sta (dp),y
        iny
        bne -
        inc sp+1
        inc dp+1
        dex
        bne .pg
        ldy #0
-       lda (sp),y
        sta (dp),y
        iny
        cpy #192
        bne -
        rts

; ---------------------------------------------------------------- Load level: A = level number
; hal_load_level: colors (COLPF0..2 and the sky bars), map to LMAP, draw level and status line.  Changes all registers.
;   The same tile characters look different on the meadow, in the cave, in the castle and in the sky, because for each level
;   COLPF0, COLPF1 and COLPF2 change (shadow registers COLOR0..2).  COLPF3 (green) always stays the same.
;   The first bar (sky0) also goes to COLOR4 = COLBK, because the OS puts it there for the top bar;
;   bars 1..5 are fetched by the DLI from skyc.  The map (240 bytes = 20 x 12 tiles) is copied to LMAP by screen_select_a of the tile library;
;   the game logic later deletes collected coins from it.
hal_load_level:
        sta pt0
        tax
        lda lvl_pf0,x
        sta COLOR0
        lda lvl_pf1,x
        sta COLOR0+1
        lda lvl_pf2,x
        sta COLOR0+2
        lda lvl_sky0,x
        sta skyc                 ; COLOR4 (COLBK) is set only by hal_unblank: during the pause the background stays black
        lda lvl_sky1,x
        sta skyc+1
        lda lvl_sky2,x
        sta skyc+2
        lda lvl_sky3,x
        sta skyc+3
        lda lvl_sky4,x
        sta skyc+4
        lda lvl_sky5,x
        sta skyc+5
        lda pt0                  ; level number = screen number of the bank levels
        jsr screen_select_a      ; copy map to LMAP
        jsr screen_draw_all
        jmp hal_hud

; hal_hud: status line (row 0) and scrolling text (row 23, the last)
;   Row 0 (30 characters from column 1): "HUGO 000000   BILD 1/4   LEBEN"; the digits are overwritten afterwards:
;   score in columns 6-11 (hal_score_draw), the level number in column 20 (text index 19 + column 1), three stars as lives
;   in columns 31-33.  The digit characters are CH_DIGIT0 + digit.  Row 23 (index 23 * 40 = 920) shows the name of the
;   level (25 characters from column 7), pointer from txt_lv_lo/hi.  Row 23 is the lower character row of tile row 11 and overwrites
;   the tiles there.  Unlike on the C64 there are no color RAM writes: the colors apply to the whole screen.
hal_hud:
        ldx #0
-       lda txt_hud,x
        sta SCREEN+1,x
        inx
        cpx #30
        bne -
        lda lvl
        clc
        adc #CH_DIGIT0+1
        sta SCREEN+20            ; level number
        lda #CH_STAR
        sta SCREEN+31
        sta SCREEN+32
        sta SCREEN+33
        ldx lvl
        lda txt_lv_lo,x
        sta sp
        lda txt_lv_hi,x
        sta sp+1
        ldy #0
-       lda (sp),y
        sta SCREEN+920+7,y       ; row 23 = index 23 * 40 = 920
        iny
        cpy #25
        bne -
        jmp hal_score_draw

; hal_score_draw: score as six digits (three BCD bytes sc2, sc1, sc0, most significant first) into the status line (columns 6-11).
;   Changes A, X.  The end (put_bcd) is run through directly by the third call, so the rts there serves all three calls.
hal_score_draw:
        lda sc2
        ldx #0
        jsr put_bcd
        lda sc1
        ldx #2
        jsr put_bcd
        lda sc0
        ldx #4
put_bcd:                         ; A = BCD, X = column offset from column 6 (0, 2, 4); changes A
        pha
        lsr a
        lsr a
        lsr a
        lsr a                    ; upper digit (tens)
        clc
        adc #CH_DIGIT0
        sta SCREEN+6,x
        pla
        and #$0f                 ; lower digit (ones)
        clc
        adc #CH_DIGIT0
        sta SCREEN+7,x
        rts

; ---------------------------------------------------------------- Figures
; Tables for access to the animation frames.  The hero has 14 frames (7 to the right, 7 mirrored to the left) of
; 21 lines (bytes) per player; the enemy 6 frames (3 + 3 mirrored) of 13 bytes each.  frame21_lo/hi = frame number * 21 as a 16-bit value
; (offset into hero_p0..p3), frame13_lo = frame number * 13 (the highest frame 5 * 13 = 65 fits in a byte; the high byte of the address
; comes from the carry during the addition).
frame21_lo:
    .repeat 14, i
        .byte <(i * 21)
    .endrepeat
frame21_hi:
    .repeat 14, i
        .byte >(i * 21)
    .endrepeat
frame13_lo:
    .repeat 6, i
        .byte <(i * 13)
    .endrepeat
herotab_lo: .byte <hero_p0,<hero_p1,<hero_p2,<hero_p3       ; start of the frame table per player
herotab_hi: .byte >hero_p0,>hero_p1,>hero_p2,>hero_p3
old_top:  .byte 0                ; start line of the hero in player memory at the last drawing (for erasing)
old_etop: .byte 0                ; the same for the enemy (start line in the missile page PM_M)

; hal_update_sprites: write hero and enemy from h_px/h_py/h_fr and e_px/e_py/e_fr into the player memories.
;   Horizontal: color clock = x / 2 + 48 (left border of the screen) + 2 (column 2 of the 12-point-wide image = start of the 8-clock-wide player)
;   Vertical:   index in player memory = 32 + foot y - height; hero 21 lines (foot in line 20) -> h_py + 12; enemy 13 lines -> e_py + 19.
;   Flow: (1) set x position, (2) erase the old lines of all four players (the figure moves in y; the bytes at the
;   old place would otherwise remain as a shadow), (3) copy the 21 bytes of the current frame of each player to the new place.
;   The enemy is treated the same way, only with 13 bytes, a common missile page and four HPOS registers.
;   Changes A, X, Y, sp, dp, pt0.  The players are moved by writing to memory (ANTIC fetches them by DMA): there is no
;   sprite register for y.  The height is not range-checked (h_py + 12 must stay below 256, always the case here).
hal_update_sprites:
        ; ---- Hero: position
        lda h_px_h
        lsr a                    ; bit 0 of the high byte into the carry ...
        lda h_px_l
        ror a                    ; ... and shift it in again from the left: 16-bit value / 2, result fits in a byte (0..160)
        clc
        adc #50                  ; 48 (left border) + 2 (column 2 of the image)
        sta HPOSP0
        sta HPOSP0+1
        sta HPOSP0+2
        sta HPOSP0+3             ; all four players the same: they lie exactly on top of each other
        ; ---- Hero: erase old lines, write new ones
        ; Loop .clr: X = player 0..3.  dp points to page (PM_P0 + X) at line old_top (dp low), 21 bytes are erased
        ; (Y = 20 to 0; bpl ends when Y falls below 0).
        ldx #0
.clr:   lda #>PM_P0
        stx pt0                  ; player k
        clc
        adc pt0
        sta dp+1
        lda old_top
        sta dp
        ldy #20
        lda #0
-       sta (dp),y
        dey
        bpl -
        inx
        cpx #4
        bne .clr
        lda h_py
        clc
        adc #12
        sta old_top              ; new start line = foot y + 12 (see above)
        ; Loop .put: sp = start of the table of player k + frame number * 21; dp = destination in page k; copy 21 bytes.
        ldx #0
.put:   stx pt0
        lda h_fr
        tay
        lda herotab_lo,x
        clc
        adc frame21_lo,y
        sta sp
        lda herotab_hi,x
        adc frame21_hi,y
        sta sp+1
        lda #>PM_P0
        clc
        adc pt0
        sta dp+1
        lda old_top
        sta dp
        ldy #20
-       lda (sp),y
        sta (dp),y
        dey
        bpl -
        ldx pt0
        inx
        cpx #4
        bne .put
        ; ---- Enemy: missiles side by side (2 clocks each)
        ;   Missiles 0..3 are 2 clocks apart (HPOSM0 = $d004 = HPOSP0+4 ...); during the addition the carry stays 0,
        ;   because the position stays below 256, so adc needs no clc.
        lda e_px_h
        lsr a
        lda e_px_l
        ror a
        clc
        adc #50
        sta HPOSP0+4
        adc #2
        sta HPOSP0+5
        adc #2
        sta HPOSP0+6
        adc #2
        sta HPOSP0+7
        ; All four missiles share one page (PM_M): one byte per line, 2 bits per missile.  Erase the old 13 lines ...
        lda #>PM_M
        sta dp+1
        lda old_etop
        sta dp
        ldy #12
        lda #0
-       sta (dp),y
        dey
        bpl -
        ; ... new start line = foot y + 19, then copy the 13 bytes of the image from enemy_m (frame number * 13) into it.
        lda e_py
        clc
        adc #19
        sta old_etop
        sta dp
        ldx e_fr
        lda frame13_lo,x
        clc
        adc #<enemy_m
        sta sp
        lda #>enemy_m
        adc #0
        sta sp+1
        ldy #12
-       lda (sp),y
        sta (dp),y
        dey
        bpl -
        rts

; ---------------------------------------------------------------- Sound effects (POKEY channel 4)
; Effects 0 jump, 1 landing, 2 step, 3 climbing, 4 coin, 5 Gumbo climbs, 6 level-change jingle, 7 fanfare.
; Entry of 4 bytes each: AUDF start value (smaller value = higher pitch), change per frame (signed), AUDC (sound type * 32 + volume:
; $A0 = pure tone, $80 = noise), duration in frames.
;   AUDF4 ($D206) is the divider of the pitch, AUDC4 ($D207) contains the sound type in bits 7-5 and the volume in bits 3-0
;   (bit 4 = volume only, never set here).  The change is added to AUDF at every frame, so the tone glides:
;   253 = -3 (pitch rises), 250 = -6, 255 = -1, 4 = +4 (pitch falls), 0 = stays.
sfx_tab:
        .byte 100, 253, $a8, 22        ; 0 jump: AUDF falls = pitch rises
        .byte 40, 4, $8a, 7            ; 1 landing: noise
        .byte 20, 0, $86, 2            ; 2 step: click
        .byte 60, 250, $a6, 4          ; 3 climbing
        .byte 40, 253, $aa, 10         ; 4 coin
        .byte 120, 4, $a8, 12          ; 5 Gumbo climbs
        .byte 70, 255, $aa, 40         ; 6 level-change jingle
        .byte 90, 255, $aa, 60         ; 7 fanfare
sfx_on:  .byte 0                 ; not 0 while an effect is sounding (core.asm reads this)
sfx_f:   .byte 0                 ; current AUDF value
sfx_d:   .byte 0                 ; change per frame (signed, added to sfx_f)
sfx_c:   .byte 0                 ; AUDC value (sound type and volume)
sfx_n:   .byte 0                 ; remaining frames

; hal_sfx: start effect.  A = effect number 0..7; number * 4 is the index into sfx_tab.  A new effect replaces a running one.
;   Changes A, X.  Unlike on the C64 there is no gate: the effect begins with the next vbi (sfx_update writes AUDF4/AUDC4).
;   sfx_on becomes 0 first (sfx_update then does nothing), the four bytes are set, sfx_on becomes 1 again at the end.
hal_sfx:
        asl a
        asl a                    ; number * 4
        tax
        lda #0
        sta sfx_on               ; effect off while setting (the VBI calls sfx_update)
        lda sfx_tab,x
        sta sfx_f
        lda sfx_tab+1,x
        sta sfx_d
        lda sfx_tab+2,x
        sta sfx_c
        lda sfx_tab+3,x
        sta sfx_n
        lda #1
        sta sfx_on
        rts

; sfx_update: called in every frame by the VBI (after the music player, so that it overwrites channel 4).  Writes AUDF4 and AUDC4,
; advances the tone divider and counts down the duration; when it has run out, AUDC4 = 0 (volume 0) is set.  Changes A.
sfx_update:
        lda sfx_on
        beq +
        lda sfx_f
        sta AUDF4
        lda sfx_c
        sta AUDC4
        clc
        lda sfx_f
        adc sfx_d
        sta sfx_f
        dec sfx_n
        bne +
        lda #0
        sta sfx_on
        sta AUDC4                ; channel 4 silent again
+       rts
