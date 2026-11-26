; =====================================================================================================================
; hal_plus4.asm - hardware layer (HAL) of the Commodore Plus/4 for "Hugo & Gumbo"
; =====================================================================================================================
;
; The game logic (lib/core.asm) knows no hardware. This file translates its requests into accesses to the TED,
; the only chip of the Plus/4 for graphics AND sound.
;
; INTERFACE (described in full in core.asm; here: what this HAL provides and needs)
;   for the core:            hal_sfx (A = effect 0..7), hal_put_tile (X = column, Y = row, A = tile), hal_score_draw,
;                            sfx_on (byte: non-zero while an effect is sounding)
;   for the main program:    hal_init, hal_wait_frame, hal_blank, hal_unblank, hal_title, hal_load_level (A = level),
;                            hal_update_sprites (reads h_px_l/h/h_py/h_fr and e_px_l/h/e_py/e_fr); also SCREEN, BGS, CH_BLANK,
;                            TED_MC1/TED_MC2 and the table skyc (used directly by show_title)
;   internal helpers:        hal_draw_level (draw all 240 tiles from LMAP, uses get_tile of the core), hal_hud, put0/put23,
;                            put_bcd, set_ptrs, put_cell, restore, compose, char_addr, forget_figures, sfx_update, music_play, irq
;   The HAL reads from the core: lvl, sc0..sc2 and the output values; from lib/levels.inc: lvl_mc1/2, lvl_sky0..5, char_col, txt_hud,
;   txt_lv_lo/hi, CH_DIGIT0, CH_STAR, hero_lo/hi, enemy_lo/hi, sfx_tab, mus1_lo/hi, mus2_lo/hi; from the
;   export: title_chars, hg_c0..hg_c3, levels_screens_lo/hi.  It changes only its own zero page cells and may change A, X, Y and the flags
;   freely in all routines.  The music belongs to the HAL here (music_play), not to a library of its own.
;   Differences to the other systems (sprites, sound, start): see core.asm, section "Differences".
;
;   SCREEN: multicolor text 40 x 25 (characters 4 x 8 wide pixels; one pixel is 2 dots wide, so 160 x 200 pixels are visible).
;       Each pixel has two bits and so selects one of four colors:
;           00 = background ($FF15)   01 = color 1 ($FF16)   10 = color 2 ($FF17)   11 = character color (from color RAM)
;       Multicolor applies per cell only if bit 3 of the color RAM byte is set (otherwise the character is drawn "hires" with two colors).
;       A color RAM byte on the TED is already a color WITH luminance:
;           color value = luminance * 16 + 8 + hue      (luminance 0..7, hue 0..7 for the character color, +8 = multicolor)
;       The registers $FF15-$FF17, in contrast, hold the full color (luminance * 16 + hue 0..15) without the multicolor bit.
;       A custom character set (256 characters = 2 KB) lives in RAM at $4000 (no ROM character set), the screen at $0C00,
;       the color RAM at $0800 (1000 bytes each; on the TED the color RAM always lies $0400 before the screen memory).
;       The sky consists of six bars: a raster interrupt changes the background color $FF15 each time.
;
;   FIGURES: The TED has NO sprites.  Hero and enemy are CHARACTER SPRITES:
;       From pre-shifted images (by 0..3 multicolor dots to the right, precomputed by the generator tools/gen_jumprun_plus4.py), characters
;       of the character set ("pool", codes $E0-$F4) are redefined in every frame in which a figure has moved:
;       dot by dot the image is blended with the background (background character of the cell: dots that the
;       image covers replace the background dots; where the image is transparent, the background stays), and the cells
;       on the screen are set to the pool characters.  For this the background is kept in a copy ($9400, 960 bytes), because the
;       screen itself shows the pool characters right now and so "forgets" what was underneath.
;       The hero occupies 3 x 4 cells (12 pool characters $E0-$EB), the enemy 3 x 3 (9 pool characters $EC-$F4): 21 characters in all.
;       Because the characters always belong to whole cells, the fine position is reached like this: horizontally by pre-shifting the
;       image (0..3 dots), vertically by the line offset when blending (0..7 pixel lines).
;
;   SOUND: The TED has two voices (square wave; voice 2 also noise) and a common volume.  Voice 1 = melody, voice 2 = bass;
;       sound effects borrow voice 2 (the bass pauses during an effect).
;       Pitch: a 10-bit value N per voice gives the frequency  f = 110840.5 / (1024 - N)  Hz  (PAL TED); the larger N, the higher
;       the tone.  The generator converts the note frequencies to N with exactly this formula.
;
; MEMORY MAP:   $1001 program  $4000 character set  $4800 tables and figure images  $6800 image data  $9000 level map  $9400 background copy
;               $0800 color RAM  $0C00 screen  $FF00-$FF3F TED registers (always visible, even with the ROM switched off)
;               The Plus/4 has 64 KB RAM; after switching off the ROM ($FF3F) all of this RAM belongs to the program, also $FFFA-$FFFF.
; ZERO PAGE:    $80-$B3 game logic, $C0-$E3 this file (the Plus/4 needs no zero page any more after the operating system is switched off).
;               The interrupt uses NO zero page cell: it interrupts the main program at any point in the middle of its
;               calculation with the pointers, and it must not change any of them.
; =====================================================================================================================

.const SCREEN  = $0c00
.const COLORRAM= $0800
.const CHARSET = $4000
.const BGS     = $9400            ; copy of the background (screen codes), 40 x 24 = 960 bytes, same layout as the screen
.const POOL_H  = $e0              ; pool characters of the hero ($E0-$EB), of the enemy ($EC-$F4)
.const POOL_E  = $ec

; cell colors of the figures.  Multicolor = bit 3 (+8) is already included: value = luminance * 16 + 8 + hue.
.const cell_skin = $6f           ; cell colors of the figures (multicolor = bit 3): yellow (skin) = luminance 6, hue 7
.const cell_red  = $4a           ; red (shirt)  = luminance 4, hue 2
.const cell_blue = $5e           ; blue (trousers) = luminance 5, hue 6
.const cell_green= $4d           ; green (enemy) = luminance 4, hue 5
.const TOP     = 4                ; raster line of the first text row (see sky bars); the bars are 32 raster lines = 4 text rows high each

; ---- TED registers
.const TED_BG0   = $ff15          ; background color 0 (pair 00)
.const TED_MC1   = $ff16          ; color 1 (pair 01)
.const TED_MC2   = $ff17          ; color 2 (pair 10)
.const TED_BORDER= $ff19          ; border color
.const TED_CTL1  = $ff06          ; bit 4 screen on, bit 3 25 rows, bits 0-2 vertical offset (3)
.const TED_CTL2  = $ff07          ; bit 7 reverse OFF (so that all 256 characters can be selected), bit 4 multicolor, bit 3 40 columns
.const TED_CHAR  = $ff12          ; bit 2: character set ROM(1)/RAM(0); bits 0-1: pitch of voice 1 (high)
.const TED_CHBASE= $ff13          ; bits 2-7: address of the character set (bits 10-15); bit 1: set here as "fast clock outside
                                  ; the screen" (ted.chip.json calls bit 1 "ROM status", bit 0 "clock status": check in the manual)
.const TED_SCRBASE=$ff14          ; bits 3-7: address (bits 11-15) of the color RAM; the characters lie $0400 behind it ($08: colors $0800, characters $0C00)
.const TED_IRQ   = $ff09          ; interrupt requests (writing a 1 clears)
.const TED_IRQEN = $ff0a          ; bit 1: enable raster interrupt, bit 0: raster line bit 8
.const TED_RLO   = $ff0b          ; raster line that triggers the interrupt (low 8 bits)
.const TED_RASTER= $ff1d          ; current raster line (low 8 bits)
.const TED_V1LO  = $ff0e          ; voice 1 pitch low (high: $FF12 bits 0-1)
.const TED_V2LO  = $ff0f          ; voice 2 pitch low
.const TED_V2HI  = $ff10          ; voice 2 pitch high (bits 0-1)
.const TED_VOL   = $ff11          ; bits 0-3 volume, bit 4 voice 1 on, bit 5 voice 2 on, bit 6 voice 2 noise
.const TED_ROMOFF= $ff3f          ; write: switch ROM off (RAM visible)  [writing to $FF3E switches the ROM on again]

; ---- zero page of this file
.const sp   = $c0                 ; pointer to screen cell (2 bytes)
.const bp   = $c2                 ; pointer to the background copy
.const cp   = $c4                 ; pointer to the color RAM
.const dp   = $c6                 ; general pointer (2 bytes)
.const pt0  = $c8                 ; scratch memory (short-lived, used differently by each routine)
.const pt1  = $c9
.const pt2  = $ca
.const pt3  = $cb
.const src  = $cc                 ; pointer to the figure image (2 bytes)
.const gp   = $ce                 ; pointer to the character data of the background character (2 bytes)
.const op   = $d0                 ; pointer to the character data of the pool character (2 bytes)
.const cp_col = $d2               ; parameters of the composition (compose/restore): left cell, top cell, offset 0..7 in the cell,
.const cp_row = $d3
.const cp_v   = $d4
.const cp_pool= $d5               ; ... first pool character, cell rows, image rows
.const cp_nr  = $d6
.const cp_img = $d7
.const cp_c0  = $d8               ; colors per cell row (4 bytes)
.const cr     = $dc               ; current cell row
.const cc     = $dd               ; current cell column
.const ly     = $de               ; current pixel line 0..7
.const kb     = $df               ; image line of pixel line 0
.const anym   = $e0               ; OR of all masks of the cell: 0 = the cell contains no dot of the figure
.const bgc    = $e1               ; background character of the cell
.const fig    = $e2               ; image byte of the current pixel line (4 pixels)
.const msk    = $e3               ; mask for it

; ---------------------------------------------------------------- Start
; hal_init: set up the TED. Afterwards the program runs without an operating system: ROM off, own interrupt vector.  Changes A.
;   The interrupts stay disabled (sei); the main program enables them with cli.
;   The order matters: first block every interrupt with sei (the KERNAL ROM routine of the Plus/4 would otherwise butt in during
;   the switch), then switch the ROM off and only then set the vectors in RAM, because with the ROM visible, writes to
;   $FFFA-$FFFF would have no effect (they would land in the RAM under the ROM, but reads would return the ROM).
hal_init:
        sei
        sta TED_ROMOFF           ; RAM instead of ROM: the whole operating system is gone from now on
                                 ; (Writing is enough, the written value A does not matter.)
        lda #<irq
        sta $fffe
        lda #>irq
        sta $ffff                ; interrupt vector (in RAM): on IRQ the CPU jumps via $FFFE/$FFFF
        lda #<rti_only
        sta $fffa
        sta $fffc
        lda #>rti_only
        sta $fffb
        sta $fffd                ; NMI vector ($FFFA) and reset vector ($FFFC) point to a plain rti: an NMI does nothing
        lda #0
        sta sfx_on               ; no sound effect active
        sta sfx_n
        sta mus_tick             ; music starts at row 0 with a fresh beat
        sta mus_row
        sta frame
        lda #$98                 ; = %10011000
        sta TED_CTL2             ; bit 7: reverse off, bit 4: multicolor on, bit 3: 40 columns, bits 0-2: horizontal offset 0
        lda #$0b                 ; = %00001011
        sta TED_CTL1             ; screen still off (bit 4 = 0), 25 rows (bit 3), vertical offset 3 (bits 0-2, default value)
        lda #$08
        sta TED_SCRBASE          ; screen at $0C00 (and so the color RAM at $0800)
        lda TED_CHAR
        and #$fb                 ; clear bit 2: character set from RAM (the other bits, esp. the pitch of voice 1, stay)
        sta TED_CHAR             ; character set from RAM
        lda #$42                 ; = %01000010: bits 2-7 = $10 -> $10 * 1024 = $4000;  bit 1 = 1: fast clock outside the screen
        sta TED_CHBASE           ; character set at $4000, fast clock outside the screen
        lda #0
        sta TED_BORDER           ; border black
        lda #$38                 ; = %00111000: volume 8 (bits 0-3), voice 1 (bit 4) and voice 2 (bit 5) on, no noise (bit 6)
        sta TED_VOL              ; volume 8, both voices on
        ; sky interrupt: first interrupt in raster line 0
        lda #0
        sta irq_idx              ; number of the next interrupt per frame (0..5)
        sta TED_RLO              ; raster line 0 (low 8 bits) ...
        lda #$02
        sta TED_IRQEN            ; ... enable raster interrupt (bit 1); bit 0 = 0: raster line has only 8 bits, all lines < 256
        lda TED_IRQ
        sta TED_IRQ              ; clear possible old requests (write back the bits read)
        rts                      ; The interrupts stay disabled: the main program enables them with cli.

rti_only: rti                    ; target for the NMI and reset vectors (see hal_init): does nothing

; ---------------------------------------------------------------- Interrupt
; Six interrupts per frame.  Number 0 (raster line 0, above the screen): sky color 0, music, effects, frame counter.
; Number 1..5: one line before the start of bar k the interrupt is triggered; then the routine waits for the start
; of the line (otherwise the color would change in the middle of the line) and sets the background color.
;
; Why like this? The TED has only ONE background color for pixel pair 00.  A sky with a color gradient is made by changing $FF15 several
; times while the picture is drawn: what was drawn above the switching line keeps the old color.
; The interrupt runs six times per frame, 50 times per second; the lines are in irq_line (start of bar) and irq_trig (one line before).
; Why one line before?  Triggering the interrupt and entering it cost a few clock cycles of varying length
; (depending on which instruction the main program was just executing).  If the change simply happened "some time after the line",
; the color change would drift into the line by a varying amount to the right.  So: come early, then wait for the
; exact start of the line (loop .band).
;   The interrupt uses no zero page and shares no cells with the main loop (except frame, sfx_on, the effect and music cells),
;   so it may interrupt it at any point.  Number 0 lies above the screen and has time for music and effect.
irq:    pha                      ; save registers: with the ROM switched off the CPU jumps straight here, no KERNAL saves them
        txa
        pha
        tya
        pha
        lda TED_IRQ
        sta TED_IRQ              ; acknowledge interrupt (otherwise it would trigger again at once)
        ldx irq_idx
        bne .band
        ; --- number 0: above and outside the visible screen, there is time here for the screen work
        lda skyc
        sta TED_BG0              ; sky color of the top bar
        inc frame                ; frame counter: hal_wait_frame waits for it to change
        jsr music_play           ; music one step
        jsr sfx_update           ; sound effect in progress one step
        ldx #1
        jmp .next
.band:  lda irq_line,x           ; line where the bar begins
-       cmp TED_RASTER
        bcs -                    ; the loop runs as long as irq_line >= raster line; so it ends only when the raster line
                                 ; is greater than irq_line (the color then changes at the start of line irq_line + 1)
        lda skyc,x               ; color of bar x ...
        sta TED_BG0              ; ... from now on it applies to the following lines
        inx
        cpx #6
        bcc .next
        ldx #0                   ; after bar 5 the interrupt 0 comes again in the next frame
.next:  stx irq_idx
        lda irq_trig,x           ; raster line of the next interrupt (one before the start of the bar)
        sta TED_RLO
        pla                      ; restore registers (reverse order)
        tay
        pla
        tax
        pla
        rti
irq_idx: .byte 0                 ; number of the next interrupt (0 = start of frame, 1..5 = bars)
frame:   .byte 0                 ; frame counter, counted up by interrupt 0
irq_line: .byte 0, TOP+32, TOP+64, TOP+96, TOP+128, TOP+160     ; raster line in which bar k begins
irq_trig: .byte 0, TOP+31, TOP+63, TOP+95, TOP+127, TOP+159     ; raster line in which the interrupt for it is triggered (one before)
skyc:    .byte 0,0,0,0,0,0       ; sky colors of the six bars (TED color value for $FF15), set by hal_load_level / .colors

; hal_wait_frame: wait for the next frame: the interrupt counts frame up.  Changes only A.
;   This ties the main program to the frame clock (50 frames per second): exactly one step of the game logic per frame.
hal_wait_frame:
        lda frame
-       cmp frame
        beq -                    ; frame is a byte that the interrupt changes in the background
        rts

; hal_blank / hal_unblank: switch the screen off and on.  When off, the TED outputs only the border color; so the
; rebuild of a picture (writing cells, redefining characters) is not visible.  First wait for the end of the frame, so the screen is not
; switched off in the middle of the picture.  Both change only A.  $0B resp. $1B: bit 3 = 25 rows, bits 0-2 = vertical offset 3, bit 4 = screen on.
hal_blank:
        jsr hal_wait_frame
        lda #$0b
        sta TED_CTL1             ; screen off
        rts
hal_unblank:
        lda #$1b
        sta TED_CTL1             ; screen on (bit 4)
        rts

; ---------------------------------------------------------------- Cells
; row starts: screen + 40 * row.  Tables, because the 6502 cannot multiply.
row40_lo:
    .repeat 25, i
        .byte <(SCREEN + i * 40)
    .endrepeat
row40_hi:
    .repeat 25, i
        .byte >(SCREEN + i * 40)
    .endrepeat

; set_ptrs: X = column, Y = row -> sp (screen), bp (background copy), cp (color RAM) point to the cell.  Changes A, flags.
;   All three areas have the same layout (row * 40 + column); they differ only in the high address byte:
;   BGS - SCREEN = $9400 - $0C00 (background copy), COLORRAM - SCREEN = -$0400 (color RAM).
set_ptrs:
        txa
        clc
        adc row40_lo,y
        sta sp                   ; low byte is the same for all three pointers
        sta bp
        sta cp
        lda row40_hi,y
        adc #0                   ; carry from the addition of the column
        sta sp+1
        clc
        adc #(>BGS) - (>SCREEN)  ; background copy: high address shifted
        sta bp+1
        lda sp+1
        sec
        sbc #(>SCREEN) - (>COLORRAM)   ; color RAM: high address shifted (4 pages lower)
        sta cp+1
        rts

; put_cell: Y = offset relative to (sp), A = character code: write screen, background copy and color of the cell.  Changes A, X.
;   The color comes from the table char_col (one color RAM value per character code, with the multicolor bit set).
put_cell:
        sta (sp),y
        sta (bp),y
        tax
        lda char_col,x
        sta (cp),y
        rts

; hal_put_tile: X = column (0..19), Y = row (0..11), A = tile number: write the four characters of the tile (from the exported tile list) into the
;   2 x 2 cells.  Changes all registers, sp, bp, cp, pt0.  The tile set hg (export graphics/hg.vctiles) applies to all four levels.
;   The game thinks in tiles of 2 x 2 characters (20 x 12 tiles = 40 x 24 cells); the character tables hg_c0..hg_c3 of the tile set
;   hold the four character codes per tile (top left, top right, bottom left, bottom right), one byte per tile in each table.  Offset 40 = next row.
hal_put_tile:
        sta pt0                  ; tile number = index into the four character tables hg_c0..hg_c3
        txa
        asl a
        tax                      ; cell column = 2 * column
        tya
        asl a
        tay                      ; cell row = 2 * row
        jsr set_ptrs
        ldx pt0
        lda hg_c0,x
        ldy #0
        jsr put_cell
        ldx pt0
        lda hg_c1,x
        ldy #1
        jsr put_cell
        ldx pt0
        lda hg_c2,x
        ldy #40
        jsr put_cell
        ldx pt0
        lda hg_c3,x
        ldy #41
        jmp put_cell

; hal_draw_level: draw all 20 x 12 tiles from the level map LMAP onto the screen (get_tile reads them from the game logic).
;   pt3 = row, pt2 = column.  Changes all registers, sp, bp, cp, pt0..pt3.
hal_draw_level:
        lda #0
        sta pt3                  ; row
.row:   lda #0
        sta pt2                  ; column
.col:   ldx pt2
        ldy pt3
        jsr get_tile
        ldx pt2
        ldy pt3
        jsr hal_put_tile
        inc pt2
        lda pt2
        cmp #20
        bcc .col
        inc pt3
        lda pt3
        cmp #12
        bcc .row
        rts

; hal_title: title picture, 960 character codes (24 rows * 40), cell by cell with cell color (from char_col) onto screen, background copy
;   and color RAM.  Changes all registers, sp, bp, cp, dp, pt1, pt2.
;   Row 24 (the 25th) stays untouched; the status lines of the game are in rows 0 and 23.
hal_title:
        lda #<title_chars
        sta dp
        lda #>title_chars
        sta dp+1
        lda #0
        sta pt2                  ; row
.trow:  ldx #0
        ldy pt2
        jsr set_ptrs
        ldy #0
-       sty pt1                  ; remember column counter Y (put_cell/char_col use the registers freely)
        lda (dp),y
        jsr put_cell
        ldy pt1
        iny
        cpy #40
        bne -
        lda dp                   ; next row of the source
        clc
        adc #40
        sta dp
        bcc +
        inc dp+1
+       inc pt2
        lda pt2
        cmp #24
        bcc .trow
        jmp forget_figures

; The figure characters have vanished from the screen (picture redrawn): forget the state.
;   h_nr/e_nr = 0 means "no cells to restore"; hk/ek = $FF in the first byte matches no real x position, so the
;   next hal_update_sprites certainly composes anew.
forget_figures:
        lda #0
        sta h_nr
        sta e_nr
        lda #$ff
        sta hk
        sta ek
        rts

; ---------------------------------------------------------------- Load level: A = level number
; hal_load_level: colors (color 1, color 2, sky bars), map to LMAP, draw picture and status line.  Changes all registers.
hal_load_level:
        tax
        lda lvl_mc1,x
        sta TED_MC1              ; color 1 / color 2 of the level (pairs 01 and 10 in the character set)
        lda lvl_mc2,x
        sta TED_MC2
        lda lvl_sky0,x           ; six sky bar colors (the interrupt sets them one after another in $FF15)
        sta skyc
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
        lda levels_screens_lo,x  ; tile map of the level (240 bytes = 20 x 12) from the screen bank levels ...
        sta dp
        lda levels_screens_hi,x
        sta dp+1
        ldy #0
-       lda (dp),y
        sta LMAP,y               ; ... copy to LMAP: there the game logic may change it (coins are collected)
        iny
        cpy #240
        bne -
        jsr hal_draw_level
        jsr forget_figures
        jmp hal_hud

; put0: A = character, X = column in row 0;  put23: the same in row 23 (offset 920). Screen, background copy and color.
;   (Row 23 begins at 23 * 40 = 920.)  Y is used as the index for char_col.
put0:   sta SCREEN,x
        sta BGS,x
        tay
        lda char_col,y
        sta COLORRAM,x
        rts
put23:  sta SCREEN+920,x
        sta BGS+920,x
        tay
        lda char_col,y
        sta COLORRAM+920,x
        rts

; hal_hud: status line (row 0) and scrolling text (row 23, the last; it is the lower character row of tile row 11 and
;   overwrites the tiles there).  Writes to screen, background copy and color RAM.  Changes all registers, pt0, pt1, dp.
hal_hud:
        ldx #0
-       lda txt_hud,x            ; "HUGO 000000   BILD 1/4   LEBEN" (German text of the generated data): 30 characters
        sta pt0
        inx
        lda pt0
        jsr put0                 ; column X = text index + 1
        cpx #30
        bne -
        lda lvl
        clc
        adc #CH_DIGIT0+1         ; level number = lvl + 1 as a digit character
        ldx #20
        jsr put0                 ; level number
        lda #CH_STAR
        ldx #31
        jsr put0                 ; three stars = three lives, columns 31..33
        inx
        jsr put0
        inx
        jsr put0
        ldx lvl
        lda txt_lv_lo,x          ; name of the level (25 characters)
        sta dp
        lda txt_lv_hi,x
        sta dp+1
        ldy #0
-       lda (dp),y
        sta pt0
        tya
        clc
        adc #7                   ; column 7 ... 31
        tax
        lda pt0
        sty pt1
        jsr put23
        ldy pt1
        iny
        cpy #25
        bne -
        jmp hal_score_draw

; hal_score_draw: score (six BCD digits in sc2:sc1:sc0) in row 0 from column 6.  Changes A, X, Y.
hal_score_draw:
        lda sc2
        ldx #6
        jsr put_bcd
        lda sc1
        ldx #8
        jsr put_bcd
        lda sc0
        ldx #10
put_bcd:                         ; A = BCD, X = column of the first digit (6, 8, 10); changes A, X, Y
        pha
        lsr a
        lsr a
        lsr a
        lsr a                    ; upper digit
        clc
        adc #CH_DIGIT0
        jsr put0
        inx
        pla
        and #$0f                 ; lower digit
        clc
        adc #CH_DIGIT0
        jmp put0

; ---------------------------------------------------------------- Character sprites
; state per figure: hk/ek = last state (x low, x high, y, image) -> compose again only on a change;
; h_ocol/h_orow/h_nr = cells that the figure last occupied (h_nr = 0: none).
;   Composing costs a lot of computing time (21 pool characters times 8 pixel lines); when both figures stand still, it is not needed.
hk:      .byte $ff, 0, 0, 0
ek:      .byte $ff, 0, 0, 0
h_ocol:  .byte 0
h_orow:  .byte 0
h_nr:    .byte 0                 ; number of occupied cell rows (0 = nothing to restore)
e_ocol:  .byte 0
e_orow:  .byte 0
e_nr:    .byte 0
hero_cols: .byte cell_skin, cell_red, cell_blue, cell_blue      ; cell color per cell row of the hero (head, shirt, trousers, trousers)

; hal_update_sprites: called every frame; sets hero and enemy to the state the game requests (h_px_l/h_px_h/h_py/h_fr
; and e_*, provided by core_prepare_sprites).  Steps:
;   1. nothing changed?  -> return at once
;   2. restore the old cells of both figures from the background copy (restore)
;   3. blend enemy, then hero anew (compose); the hero comes last so that he lies IN FRONT of the enemy: both use different
;      pool characters, but where the cells overlap, the cell written later overwrites the one written earlier on the screen.
;      compose always reads the background from the copy BGS, never from the screen, because the screen holds pool characters.
;   Changes all registers, the scratch cells and the parameters cp_*.  Runs in the main loop, not in the interrupt.  Figures whose
;   top edge would lie above the screen edge (foot y less than the image height of the figure: 13 resp. 21) are not drawn.
hal_update_sprites:
        lda h_px_l               ; has anything changed?
        cmp hk
        bne .chg
        lda h_px_h
        cmp hk+1
        bne .chg
        lda h_py
        cmp hk+2
        bne .chg
        lda h_fr
        cmp hk+3
        bne .chg
        lda e_px_l
        cmp ek
        bne .chg
        lda e_px_h
        cmp ek+1
        bne .chg
        lda e_py
        cmp ek+2
        bne .chg
        lda e_fr
        cmp ek+3
        bne .chg
        rts
.chg:   lda h_px_l               ; remember new state
        sta hk
        lda h_px_h
        sta hk+1
        lda h_py
        sta hk+2
        lda h_fr
        sta hk+3
        lda e_px_l
        sta ek
        lda e_px_h
        sta ek+1
        lda e_py
        sta ek+2
        lda e_fr
        sta ek+3
        ; 1. reset the old cells to the background
        lda e_nr
        beq +
        sta cp_nr
        lda e_ocol
        sta cp_col
        lda e_orow
        sta cp_row
        jsr restore
+       lda h_nr
        beq +
        sta cp_nr
        lda h_ocol
        sta cp_col
        lda h_orow
        sta cp_row
        jsr restore
        ; 2. compose the enemy anew (foot y - 13 = top line of the image)
        ;    The y coordinate of the game logic is the foot line of the figure; the 13-line-high enemy image begins 13 lines higher up.
+       lda #0
        sta e_nr
        lda e_py
        sec
        sbc #13
        bcc .hero                ; above the screen edge: do not draw
        pha
        lsr a
        lsr a
        lsr a
        sta cp_row               ; top cell row = y / 8
        pla
        and #7
        sta cp_v                 ; offset within the cell = y mod 8 (pixel lines downward)
        lda e_px_h
        lsr a
        lda e_px_l
        ror a                    ; x / 2 = multicolor dots (16-bit value shifted right by one bit: x is in dots, one pixel = 2)
        clc
        adc #2                   ; column 2 of the 12-dot-wide source image = left edge of the picture
        jsr cell_shift           ; cp_col = cell, A = shift 0..3
        sta pt0
        lda e_fr
        asl a
        asl a
        ora pt0                  ; level number * 4 + shift = index into the table of the pre-shifted images
        tax
        lda enemy_lo,x
        sta src
        lda enemy_hi,x
        sta src+1
        lda #POOL_E
        sta cp_pool
        lda #3
        sta cp_nr                ; 3 cell rows
        sta e_nr
        lda #13
        sta cp_img               ; 13 image lines
        lda #cell_green
        sta cp_c0                ; all cell rows of the enemy green
        sta cp_c0+1
        sta cp_c0+2
        sta cp_c0+3
        lda cp_col
        sta e_ocol               ; remember the occupancy so that it can be restored the next time
        lda cp_row
        sta e_orow
        jsr compose
        ; 3. hero (lies in front of the enemy)
.hero:  lda #0
        sta h_nr
        lda h_py
        sec
        sbc #21                  ; the hero image is 21 lines high
        bcc .done
        pha
        lsr a
        lsr a
        lsr a
        sta cp_row
        pla
        and #7
        sta cp_v
        lda h_px_h
        lsr a
        lda h_px_l
        ror a
        clc
        adc #2
        jsr cell_shift
        sta pt0
        lda h_fr
        asl a
        asl a
        ora pt0
        tax
        lda hero_lo,x
        sta src
        lda hero_hi,x
        sta src+1
        lda #POOL_H
        sta cp_pool
        lda #4
        sta cp_nr                ; 4 cell rows (21 lines + up to 7 lines of offset = up to 28 lines)
        sta h_nr
        lda #21
        sta cp_img
        ldx #3
-       lda hero_cols,x          ; cell colors per cell row: skin, shirt, trousers, trousers
        sta cp_c0,x
        dex
        bpl -
        lda cp_col
        sta h_ocol
        lda cp_row
        sta h_orow
        jsr compose
.done:  rts

; cell_shift: A = multicolor dot (x) -> cp_col = A / 4, A = remainder 0..3
;   A cell is 4 multicolor dots wide.  The whole cells determine where the 3 pool columns stand; the remainder (0..3) selects the
;   image pre-shifted to the right by that many dots.
cell_shift:
        pha
        lsr a
        lsr a
        sta cp_col
        pla
        and #3
        rts

; restore: reset cells (cp_col, cp_row, 3 columns, cp_nr rows) to the background (character and color)
;   Cells outside the screen (row >= 24, column >= 40) are skipped: figures may half disappear at the edge.
restore:
        lda #0
        sta cr
.rr:    lda cp_row
        clc
        adc cr
        cmp #24
        bcs .rnext               ; below the last row
        sta pt0
        lda #0
        sta cc
.rc:    lda cp_col
        clc
        adc cc
        cmp #40
        bcs .rcn                 ; right of the screen
        tax
        ldy pt0
        jsr set_ptrs
        ldy #0
        lda (bp),y               ; background character from the copy ...
        sta (sp),y               ; ... back onto the screen
        tax
        lda char_col,x           ; and its color back into the color RAM
        sta (cp),y
.rcn:   inc cc
        lda cc
        cmp #3
        bcc .rc
.rnext: inc cr
        lda cr
        cmp cp_nr
        bcc .rr
        rts

; compose: blend the image (pointer src, 3 bytes per line, cp_img lines) at the cells (cp_col, cp_row), shifted down by cp_v pixel lines,
; with the background in cp_nr cell rows and write it into the pool characters from cp_pool.
;   Blending per pixel line:  mask = all pairs of the image that are not 00;  new = (background AND NOT mask) OR image.
;   The cell color is set to the figure color of the cell row if the cell contains any dots at all.
;
; An example for one pixel line (four pairs per byte, the highest pair on the left):
;     background    01 10 11 00        image 00 11 00 01
;     mask          00 11 00 11        (pair covered -> 11, transparent -> 00)
;     background AND NOT mask = 01 00 11 00;  OR image = 01 11 11 01
;   Note: where the image has 11, the character color results, that is the cell color -- and it is the same for the whole cell.  So
;   the color belongs to the CELL, not to the pixel; the generator assigns the figure colors to cell rows (head/shirt/trousers) so that this fits.
;   The outline of the figure (color 1 = 01), in contrast, stays freely selectable per pixel and is the same in all cells.
;
; The pool characters: cell (column cc, row cr) of the figure -> character cp_pool + 3 * cr + cc.  So each figure cell has ITS own
; pool character whose 8 data bytes (8 pixel lines) are rewritten here; on the screen the cell holds this code.
compose:
        lda #0
        sta cr
.cr:    lda cr
        asl a
        asl a
        asl a
        sec
        sbc cp_v
        sta kb                   ; image line of pixel line 0 of this cell row (negative = 8-bit number above 127)
        lda #0
        sta cc
.cc:    lda cp_row
        clc
        adc cr
        cmp #24
        bcc +
        jmp .cnext               ; cell below the screen: leave out
+       sta pt0                  ; cell row on the screen
        lda cp_col
        clc
        adc cc
        cmp #40
        bcc +
        jmp .cnext               ; cell outside on the right: leave out
+       tax
        ldy pt0
        jsr set_ptrs
        lda cr                   ; pool character = cp_pool + 3 * cell row + column
        asl a
        clc
        adc cr
        clc
        adc cc
        clc
        adc cp_pool
        sta pt1
        ldy #0
        lda (bp),y
        sta bgc                  ; background character of the cell (from the copy, not from the screen!)
        lda pt1
        sta (sp),y               ; the cell now shows the pool character
        lda bgc                  ; gp = character data of the background character
        jsr char_addr
        lda dp
        sta gp
        lda dp+1
        sta gp+1
        lda pt1                  ; op = character data of the pool character
        jsr char_addr
        lda dp
        sta op
        lda dp+1
        sta op+1
        lda #0
        sta anym                 ; so far no dot of the figure in this cell
        ldy #0                   ; Y = pixel line 0..7 of the cell
.py:    tya
        clc
        adc kb                   ; image line k of this pixel line
        cmp cp_img
        bcs .tr                  ; outside the image: transparent (also "negative" k, which appear as a number above 127)
        sta pt2
        asl a
        clc
        adc pt2                  ; 3 * k
        clc
        adc cc                   ; + column
        sty ly
        tay
        lda (src),y              ; the image byte (four pairs) of this line and column
        ldy ly
        jmp .have
.tr:    lda #0
.have:  sta fig
        lsr a
        ora fig
        and #$55                 ; per pair the low bit holds "pair is not 00" (upper OR lower bit of the pair)
        sta msk
        asl a
        ora msk
        sta msk                  ; mask: 11 for every pair that the image covers
        ora anym
        sta anym                 ; remember whether the cell contains anything of the figure at all
        lda msk
        eor #$ff                 ; NOT mask
        and (gp),y               ; background without the pairs covered by the image
        ora fig                  ; put the image in
        sta (op),y               ; write into the pool character
        iny
        cpy #8
        bne .py
        lda anym                 ; cell color
        beq .bgcol               ; no dot: the cell keeps the color of the background character
        ldx cr
        lda cp_c0,x              ; figure color of this cell row
        jmp .setc
.bgcol: ldx bgc
        lda char_col,x
.setc:  ldy #0
        sta (cp),y
.cnext: inc cc
        lda cc
        cmp #3
        bcs +
        jmp .cc                  ; next cell (column) in the same cell row
+       inc cr
        lda cr
        cmp cp_nr
        bcs +
        jmp .cr                  ; next cell row
+       rts

; char_addr: A = character code -> dp = CHARSET + 8 * code
;   Each character occupies 8 bytes (one per pixel line); 8 * code is a 16-bit result, hence three shifts left by one bit.
char_addr:
        sta dp
        lda #0
        sta dp+1
        asl dp
        rol dp+1
        asl dp
        rol dp+1
        asl dp
        rol dp+1
        lda dp+1
        clc
        adc #>CHARSET
        sta dp+1
        rts

; ---------------------------------------------------------------- Sound effects (TED voice 2)
; entry of 6 bytes each (sfx_tab in lib/levels.inc): pitch low/high, change per frame low/high, noise, duration in frames.
;   An effect is a tone whose value N (10 bits, see f = 110840.5 / (1024 - N) above) changes by a fixed amount
;   in every frame (glissando: jump, fall, beep); noise replaces the square wave of voice 2 with noise (bit 6 in $FF11).
sfx_on:  .byte 0                 ; 0 = no effect active (then the bass plays on voice 2)
sfx_lo:  .byte 0                 ; current N
sfx_hi:  .byte 0
sfx_dl:  .byte 0                 ; change per frame (16 bits signed, kept separately here)
sfx_dh:  .byte 0
sfx_nz:  .byte 0                 ; 1 = noise
sfx_n:   .byte 0                 ; frames remaining

; hal_sfx: A = effect number 0..7: start the effect.  Changes A, X, pt0, pt1.  Effects: 0 jump, 1 landing, 2 step, 3 climb, 4 coin,
;   5 Gumbo climbs, 6 level change jingle, 7 fanfare.  The effect begins with the next interrupt no. 0 (sfx_update).
hal_sfx:
        sta pt0
        asl a                    ; number * 6
        sta pt1
        asl a
        clc
        adc pt1
        tax
        lda #0
        sta sfx_on               ; off while setting (the interrupt looks at it)
        lda sfx_tab,x
        sta sfx_lo
        lda sfx_tab+1,x
        sta sfx_hi
        lda sfx_tab+2,x
        sta sfx_dl
        lda sfx_tab+3,x
        sta sfx_dh
        lda sfx_tab+4,x
        sta sfx_nz
        lda sfx_tab+5,x
        sta sfx_n
        lda #1
        sta sfx_on
        rts

; sfx_update: called by interrupt no. 0 each frame (zero page is not used).  Changes A, X (the interrupt saves the registers).
sfx_update:
        lda sfx_on
        beq .rts
        lda sfx_lo
        sta TED_V2LO             ; current pitch to voice 2
        lda sfx_hi
        sta TED_V2HI
        lda #$38                 ; volume 8, both voices on
        ldx sfx_nz
        beq +
        lda #$78                 ; bit 6: voice 2 as noise
+       sta TED_VOL
        clc
        lda sfx_lo               ; N := N + change (16 bits; negative change = two's complement, the carry is dropped)
        adc sfx_dl
        sta sfx_lo
        lda sfx_hi
        adc sfx_dh
        and #3                   ; only 2 bits high: N is a 10-bit value
        sta sfx_hi
        dec sfx_n
        bne .rts
        lda #0
        sta sfx_on               ; effect over: switch the bass on again
        lda mus_n2l              ; restore the last played bass pitch ...
        sta TED_V2LO
        lda mus_n2h
        sta TED_V2HI
        lda #$38                 ; ... and switch the noise off (bit 6 = 0)
        sta TED_VOL
.rts:   rts

; ---------------------------------------------------------------- Music (two voices, tables from lib/levels.inc)
; One row lasts 6 frames; 128 rows, then from the start. An entry with high byte $FF = no new note (the old one keeps sounding).
;   The tables mus1_lo/mus1_hi (melody) and mus2_lo/mus2_hi (bass) have 128 entries each (N low resp. high).  The generator
;   writes $FFFF for "no new note", so high = $FF; a real N has high 0..3, so the sign test (bmi) is enough.
mus_tick: .byte 0                ; frames since the last row step (0..5)
mus_row:  .byte 0                ; current row 0..127
mus_n2l:  .byte 0                ; last played bass pitch (restored after an effect)
mus_n2h:  .byte 0
; music_play: called by interrupt no. 0 each frame; advances one music row every 6 frames.  Changes A, X.
music_play:
        inc mus_tick
        lda mus_tick
        cmp #6
        bcc .mrts                ; not yet time for the next step
        lda #0
        sta mus_tick
        ldx mus_row
        lda mus1_hi,x
        bmi .v2                  ; voice 1: no new note
        lda mus1_lo,x
        sta TED_V1LO
        lda TED_CHAR             ; the high bits of voice 1 are in $FF12 bits 0-1, next to them are character set bits there: keep them
        and #$fc
        ora mus1_hi,x
        sta TED_CHAR
.v2:    lda mus2_hi,x
        bmi .adv                 ; voice 2 (bass): no new note
        sta mus_n2h              ; remember new note so that an effect can restore it afterwards
        lda mus2_lo,x
        sta mus_n2l
        lda sfx_on
        bne .adv                 ; if an effect is running, voice 2 belongs to it: only remember the bass, do not play it
        lda mus_n2l
        sta TED_V2LO
        lda mus_n2h
        sta TED_V2HI
.adv:   inx
        cpx #128
        bcc +
        ldx #0                   ; after the last row from the start
+       stx mus_row
.mrts:  rts
