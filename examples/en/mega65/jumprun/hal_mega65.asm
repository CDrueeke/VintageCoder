; =====================================================================================================================
; hal_mega65.asm - hardware layer (HAL) of the MEGA65 for "Hugo & Gumbo"  (stage 1: C64 code in VIC-II mode)
; =====================================================================================================================
;
; The game logic (lib/core.asm) knows no hardware. This file translates its wishes into accesses to
; the chips of the MEGA65.
;
; INTERFACE (fully described in core.asm; here, what this HAL provides and needs)
;   for the core:            hal_sfx (A = effect 0..7), hal_put_tile (X = column, Y = row, A = tile), hal_score_draw,
;                            sfx_on (byte: not 0 while an effect is sounding; the music does not read it here)
;   for the main program:    hal_init, hal_wait_frame, hal_blank, hal_unblank, hal_title, hal_load_level (A = level),
;                            hal_update_sprites (reads h_px_l/h/h_py/h_fr and e_px_l/h/e_py/e_fr); also SCREEN, COLOR and the
;                            table skyc (used directly by show_title)
;   internal helpers:        hal_draw_level (draw all 240 tiles from LMAP, uses get_tile of the core), hal_hud, put_bcd,
;                            copy1000, sfx_update, irq
;   The HAL reads from the core: lvl, sc0..sc2 and the output values; from lib/levels.inc: lvl_mc1/2, lvl_sky0..5, txt_hud, txt_lv_lo/hi,
;   CH_DIGIT0, CH_STAR; from jumprun.asm: maps_lo/hi; from the export: title_chars, title_attrs, lv1_tiles, lv1_colors.  It only changes
;   its own zero-page cells and may change A, X, Y and the flags freely in all routines.
;   Differences to the other systems (sprites, sound, start): see core.asm, section "Differences".
;
; It is the sister of hal_c64.asm: the MEGA65 contains a C64-compatible video chip
; (VIC-IV, which works like the C64 VIC in VIC-II mode) and several SID sound chips, so the bulk
; (raster bars, tiles, sprites, effect table) stays the same line for line.  But the MEGA65 does not start in a
; C64 state, but in the BASIC 65 environment.  The ADAPTATIONS compared with the C64 are therefore:
;
;   * CPU 45GS02 instead of 6510: the instruction set is a superset of the 6502; our code only uses 6502 instructions.  Only the
;     start is new (map/eom, taz), with which hal_init puts the address space into a plain 64 KB RAM state.
;   * $01 = $35: neither BASIC nor KERNAL ROM is visible, the RAM is free up to $FFFF, I/O at $D000.
;     Consequence: there is NO KERNAL interrupt entry.  The processor reads the vector directly from RAM ($FFFE/$FFFF),
;     and the interrupt routine must save its registers itself and end with rti (see irq).
;   * VIC-IV unlock ($D02F = $47, $53) and resetting the VIC-III/IV registers ($D030, $D031, $D054, $D058, $D05E),
;     because BASIC 65 may have switched the video chip to another mode (e.g. 80 columns).
;   * VIC BANK 1 ($4000-$7FFF) instead of bank 0: the art data (character set, sprites) lies at other addresses because of the longer program and
;     the VIC's bank 0 character ROM trap.  Screen $4400, character set $6000, sprites $6800.
;   * SOUND EFFECTS on the SECOND SID ($D420-$D438).  The music has the first SID ($D400) with all three voices
;     to itself; the effect SID therefore needs no "borrowed" voice 3 and the music never has to pause.
;
;   VIC-II mode (screen)
;     * Text MULTICOLOR MODE, 40 x 25 characters. Each character is 4 x 8 wide pixels in size (one pixel = 2 screen dots wide,
;       hence 160 x 200 visible dots). For each pixel two bits give the choice:
;           00 = background color 0   ($D021)     01 = multicolor 1 ($D022)
;           10 = multicolor 2          ($D023)     11 = character color from color RAM ($D800, only colors 0-7)
;       The color RAM byte switches a character between multicolor (bit 3 set) and hires (bit 3 cleared).
;     * The OWN CHARACTER SET is at $6000 (2 KB); the screen memory at $4400 (1000 bytes).
;     * SPRITES: hero and enemy each consist of TWO sprites on top of each other:
;           - a MULTICOLOR sprite for the areas (shirt, skin, trousers): sprite 1 (hero) and sprite 3 (enemy),
;           - a HIRES sprite for outline, eye and mouth (fine details, one color): sprite 0 (hero), sprite 2 (enemy).
;       At the same position the LOWER sprite number is IN FRONT, so the hires overlays are sprites 0 and 2.
;     * SKY: six background colors on top of each other (raster bars). A raster interrupt changes $D021 at fixed lines.
;
;   SID (sound)
;     * SID 1 ($D400): voices 1 + 2 + 3 play the music (lib/music.inc, tracker export).
;     * SID 2 ($D420): sound effects (voice 3 of this chip: $D42E-$D434; volume $D438 = $D420 + $18).  Unlike
;       on the C64 the music does not pause during an effect; sfx_on remains only as the state flag of the effect.
;
; MEMORY PLAN (MEGA65, VIC bank 1: the VIC sees $4000-$7FFF; $D018 and the sprite pointers count from $4000)
;     $4400-$47E7  screen memory              $47F8-$47FF  sprite pointers (block number = address in the bank / 64)
;     $6000-$67FF  character set              $6800-$71FF  sprites (hero 14+14 frames, enemy 6+6 frames)
;     $D800-$DBE7  color RAM                  $9000-$90EF  level map (LMAP, used by the game logic)
;   (The program itself is at $2001-$2F2E, i.e. below the VIC bank; the bank data is placed by jumprun.asm
;   with .org at the fixed addresses.)
;
; POINTERS IN THE ZERO PAGE AREA: $50-$56 (sp, cp: screen and color RAM pointers; pt0-pt2: scratch memory of this file).
; =====================================================================================================================

#include <mega65/gs4502.asm>   // macro mega65_enable_vic4 (unlocks the VIC-IV, see hal_init)
.const SCREEN = $4400                  ; screen in VIC bank 1 ($4000-$7FFF): bank start $4000 + $0400 from $D018
.const COLOR  = $d800                  ; color RAM window; visible with $01 = $35 (the color memory is fixed, not in the VIC bank)
; pointers for (indirect),y accesses and scratch memory of the HAL (zero page; the game logic uses $10-$43, the music $F7-$FA)
.const sp  = $50                 ; pointer (2 bytes) to source or screen cell
.const cp  = $52                 ; pointer (2 bytes) to target or color RAM cell
.const pt0 = $54                 ; scratch byte 0 (tile number, intermediate values)
.const pt1 = $55                 ; scratch byte 1 (row; bit 8 of the hero x coordinate)
.const pt2 = $56                 ; scratch byte 2 (column; bit 8 of the enemy x coordinate)

; Sprite pointer: the VIC reads sprite data from (pointer * 64) within the bank, so here from $4000 + pointer * 64.
; Example: pointer $A0 -> $A0 * 64 = $2800 in the bank = $6800 in the memory of the MEGA65.
; The four sprite sets follow one another: hero base 14 frames ($A0..$AD), hero overlay 14 ($AE..$BB), enemy base 6 ($BC..$C1),
; enemy overlay 6 ($C2..$C7).
; (The pointer values are the same as on the C64, because bank 1 is shifted against bank 0 only by $4000, which the VIC
; does not count: in the VIC address space the sprites remain at $2800 ff.)
.const HERO_BASE_PTR  = $a0      ; $2800 in the bank = $6800 in memory
.const HERO_OVER_PTR  = $ae      ; $2b80 in the bank = $6b80
.const ENEMY_BASE_PTR = $bc      ; $2f00 in the bank = $6f00
.const ENEMY_OVER_PTR = $c2      ; $3080 in the bank = $7080

; ---------------------------------------------------------------- Start
; hal_init: sets up the address space, the VIC, the interrupt and the sprites once at program start.  Does NOT yet enable the
;           interrupt (the main program does that with cli, after the music has been initialized too).  Changes A, X, Y, Z.
;           The NMI vector ($FFFA/$FFFB) points to rti_only (an NMI, e.g. RESTORE, does nothing), as on the Plus/4.
hal_init:
        sei                             ; interrupts off while the chips are set up
        ; ---- MEGA65: switch from the BASIC 65 state to a plain "C64-like" 64 KB RAM with I/O at $D000
        ; map takes its parameters from the four registers A, X, Y, Z (offset and enable bits for the four 8 KB windows
        ; of the lower and upper 32 KB).  With A = X = Y = Z = 0 no window is remapped: every CPU address points to
        ; the same place in bank 0.  For this all four registers must be 0 (taz sets Z from A).
        lda #$00
        tax
        tay
        taz
        map                             ; cancel the memory mapping of the operating system: all 64 KB are RAM (bank 0)
        eom                             ; ends the interrupt lock of map (it prevents interrupts with a half mapping)
        lda #$35
        sta $01                         ; CPU port: RAM everywhere, I/O at $D000 (no ROMs): LORAM = 1, HIRAM = 0, CHAREN = 1
        ; ---- Unlock the VIC-IV: the extended registers (and the freedom to set them) exist only after the key
        ; $47, $53 in this order to $D02F.  Afterwards we reset the video chip settings that BASIC 65 may have
        ; changed, so that the chip behaves like a plain C64 VIC-II.
        mega65_enable_vic4              ; unlock the VIC-IV: first $47, then $53 to $D02F (macro from <mega65/gs4502.asm>)
        lda #0
        sta $d030                       ; no C65 ROM windows (bits for ROM at $8000/$A000/$C000/$E000 and color RAM 2K window off)
        sta $d031                       ; VIC-III: 40 columns (H640 off), no line doubling, C65 fast clock (FAST) off
        sta $d054                       ; VIC-IV: no 16-bit characters, no full-color characters, normal sprites
        lda #40
        sta $d058                       ; line pitch of the text screen: 40 bytes (low byte; BASIC 65 sets 80)
        lda #0
        sta $d059                       ; ... high byte 0: 40 = $0028
        lda #40
        sta $d05e                       ; 40 characters per row (no 80-column text)
        lda #15
        sta $d438                       ; second SID ($D420): volume 15 (it belongs to the sound effects; $D438 = $D420 + $18);
                                        ; music_init sets up the first SID
        lda #$7f                        ; bit 7 = 0, others 1: "clear" -> all CIA interrupt sources off (CIA 1 at $DC0D, CIA 2 at $DD0D)
        sta $dc0d                ; CIA interrupts off (keyboard scan, timers): only the raster interrupt should remain
        sta $dd0d
        lda $dc0d                       ; reading acknowledges pending CIA interrupts
        lda $dd0d
        lda $dd02
        ora #$03
        sta $dd02                ; bits 0-1 of CIA 2 port A as output (VIC bank selection)
        lda $dd00
        and #$fc                        ; clear bank bits ...
        ora #$02                 ; VIC bank 1 ($4000-$7fff): the bits are inverted, %10 selects bank 1 (%11 would be bank 0)
        sta $dd00
        lda #$18                 ; screen: high nibble 1 * $400 = $0400 in the bank = $4400; character set: bits 1-3 = 4 * $800 = $2000 -> $6000
        sta $d018
        lda #$18                 ; multicolor text (bit 4), 40 columns (bit 3)
        sta $d016
        lda #0
        sta $d020                       ; border color black
        sta $d015                ; sprites off
        sta $d01b                ; sprites in front of the background
        sta $d017                       ; no sprite height doubling
        sta $d01d                       ; no sprite width doubling
        lda #%00001010           ; sprites 1 and 3 multicolor
        sta $d01c
        lda #10                  ; multicolor 1 (skin)
        sta $d025
        lda #14                  ; multicolor 2 (trousers, eyes)
        sta $d026
        lda #0
        sta $d027                ; hero overlay black
        lda #2
        sta $d028                ; hero base red
        lda #0
        sta $d029                ; enemy overlay black
        lda #5
        sta $d02a                ; enemy base green
        lda #<irq                       ; interrupt vector in RAM ($FFFE/$FFFF; the operating system is switched off)
        sta $fffe                       ; (without the KERNAL ROM there is no detour via $0314/$0315, the processor jumps directly here)
        lda #>irq
        sta $ffff
        lda #<rti_only                  ; NMI vector ($FFFA/$FFFB) in RAM to a bare rti: an NMI (RESTORE) does nothing
        sta $fffa
        lda #>rti_only
        sta $fffb
        lda #85                         ; first raster interrupt at line 85 (end of the first sky bar)
        sta $d012
        lda $d011
        and #$7f                        ; bit 7 of $D011 = bit 8 of the raster line: 0, because all our lines are less than 256
        sta $d011
        lda #1                          ; $D01A: enable raster interrupt
        sta $d01a                ; raster interrupt on
        lda #0
        sta band
        sta vblank
        sta sfx_on
        rts

rti_only: rti                          ; target of the NMI vector (see hal_init)

; ---------------------------------------------------------------- Interrupt: sky bars and frame clock
; The raster interrupt fires six times per frame, each time at a raster line from irq_line (85, 119, 153, 187, 221, 251).
; On every call the background color is set to the next bar color: line 51 is the top screen row,
; the bars are 34 lines high each.  irq_ci distributes the colors (bars 1..5, then bar 0 again for the next frame).
; On the LAST call (line 251, below the visible picture) additionally what should run exactly once per frame happens:
;   * music_play   the music one step further (write SID registers),
;   * sfx_update   the running sound effect one step further,
;   * vblank = 1   signal to the main program: "the next frame may be computed" (see hal_wait_frame).
; Acknowledge: writing 1 to $D019 clears the raster interrupt flag.
; band = number of the bar whose color is set now (0..5); irq_ci gives the entry in skyc: 1,2,3,4,5 and finally 0.
; Difference to the C64: here there is no KERNAL entry that saves the registers, and no "jmp $EA81".  The routine
; pushes A, X and Y onto the stack itself (the processor has saved only the status register and the return address), gets them back at the
; end and returns with rti.  (We do not use the Z register of the 45GS02, it does not have to be saved.)
irq:    pha                             ; save registers (without KERNAL the routine must do this itself)
        txa
        pha
        tya
        pha
        lda #1
        sta $d019                       ; acknowledge raster interrupt
        ldx band                        ; X = number of the bar that starts now
        lda irq_ci,x                    ; color index of this bar ...
        tay
        lda skyc,y                      ; ... look it up in the color table of the current level ...
        sta $d021                       ; ... and set it as background color
        cpx #5                          ; last call of the frame (below the visible area)?
        bne +
        jsr music_play                  ; music: one step (write SID registers)
        jsr sfx_update
        lda #1
        sta vblank
+       inx                             ; next bar; after the sixth start again from the beginning
        cpx #6
        bcc +
        ldx #0
+       stx band
        lda irq_line,x                  ; raster line of the next interrupt
        sta $d012
        pla                             ; restore registers
        tay
        pla
        tax
        pla
        rti

irq_line: .byte 85,119,153,187,221,251  ; raster lines of the six interrupts per frame
irq_ci:   .byte 1,2,3,4,5,0             ; color index that is set at the respective call
band:     .byte 0                       ; running number of the next interrupt (0..5)
vblank:   .byte 0                       ; 1 = a frame is over (set by the interrupt, cleared by hal_wait_frame)
skyc:     .byte 3,3,3,3,3,14            ; the six sky colors (hal_load_level overwrites them per level)

; hal_wait_frame: waits until the interrupt reports the next end of frame (so the game logic runs exactly 50 times per second).
;   Clears vblank first, so it always waits for the NEXT end of frame (up to a whole frame).  Changes only A.
hal_wait_frame:
        lda #0
        sta vblank
-       lda vblank
        beq -
        rts


; hal_blank / hal_unblank: screen off/on (bit 4 of $D011 = DEN: display enable).  hal_blank also switches all sprites off
; (hal_unblank does not; hal_update_sprites switches the sprites on again).  While a new level is being built the
; screen stays black this way, no half-finished picture is seen.  hal_blank first waits for the end of the frame, so that the switch is flipped in the
; invisible part of the picture.  Both change only A.
hal_blank:
        jsr hal_wait_frame       ; switch the screen off only in the blanking area
        lda #$0b                        ; $D011 = $0B: bit 4 (DEN) cleared = screen off; bit 3 = 25 rows; Y scroll 3
        sta $d011
        lda #0
        sta $d015                       ; sprite enable: all sprites off (bits 0-3 would be sprites 0-3)
        rts
hal_unblank:
        lda #$1b                        ; $D011 = $1B: DEN set = screen on, 25 rows (bit 3), Y scroll 3 (default)
        sta $d011
        rts

; ---------------------------------------------------------------- Tiles
; A tile consists of 2 x 2 characters (16 x 16 dots).  Tile (column c, row k) lies on the screen
; in the character cells (2c, 2k), (2c+1, 2k), (2c, 2k+1), (2c+1, 2k+1).  The address of character row 2k is
; $4400 + 80 * k (each character row has 40 bytes, so a tile row has 80).  The tables row80_lo/hi supply these addresses.
row80_lo:
    .repeat 12, i
        .byte <(SCREEN + i * 80)
    .endrepeat
row80_hi:
    .repeat 12, i
        .byte >(SCREEN + i * 80)
    .endrepeat

; hal_put_tile: puts a tile on the screen.  Input X = column (0..19), Y = row (0..11), A = tile number.
;   Steps: (1) pointer sp to the upper left character cell = row80[row] + 2 * column,
;          (2) pointer cp to the same cell in color RAM = sp + $9400 (color RAM $D800 minus screen $4400),
;          (3) fetch the four character numbers of the tile from lv1_tiles (4 bytes per tile: top left, top right,
;              bottom left, bottom right) and write them at offsets 0, 1, 40, 41,
;          (4) write the color of the tile (lv1_colors, one byte per tile) into all four color RAM cells.
;   The tile description comes from the tilemap export of lv1; it is the same for all four levels.  Changes A, X, Y, sp, cp, pt0.
hal_put_tile:
        sta pt0                         ; remember the tile number
        lda row80_lo,y                  ; sp = address of character row 2 * row
        sta sp
        lda row80_hi,y
        sta sp+1
        txa
        asl a                           ; column * 2 = character column of the tile (each tile is 2 characters wide)
        clc
        adc sp
        sta sp
        bcc +
        inc sp+1
+       lda sp                          ; cp = sp, but in color RAM
        sta cp
        lda sp+1
        clc
        adc #(>COLOR) - (>SCREEN) ; color RAM = screen + $9400 ($D800 - $4400); only the high byte changes
        sta cp+1
        lda pt0
        asl a
        asl a
        tax
        ldy #0
        lda lv1_tiles,x                 ; character top left of the tile (X = tile number * 4)
        sta (sp),y
        iny
        lda lv1_tiles+1,x               ; top right
        sta (sp),y
        ldy #40                         ; one character row lower (40 cells)
        lda lv1_tiles+2,x               ; bottom left
        sta (sp),y
        iny
        lda lv1_tiles+3,x               ; bottom right
        sta (sp),y
        ldx pt0
        lda lv1_colors,x                ; color of the tile (color RAM value: bit 3 = multicolor, bits 0-2 = character color)
        ldy #0
        sta (cp),y
        iny
        sta (cp),y
        ldy #40
        sta (cp),y
        iny
        sta (cp),y
        rts

; hal_draw_level: draws the whole level from the map in RAM (LMAP): 12 rows x 20 columns, per tile get_tile + hal_put_tile.
;   Takes several frames (240 tiles); that is why it runs with the screen switched off.  pt1 = row, pt2 = column.
;   Changes A, X, Y, sp, cp, pt0..pt2.
hal_draw_level:
        lda #0
        sta pt1                  ; row
.row:   lda #0
        sta pt2                  ; column
.col:   ldx pt2                         ; get_tile (game logic): returns the tile number at (X, Y) in A
        ldy pt1
        jsr get_tile
        ldx pt2
        ldy pt1
        jsr hal_put_tile
        inc pt2
        lda pt2
        cmp #20
        bcc .col
        inc pt1
        lda pt1
        cmp #12
        bcc .row
        rts

; hal_title: copies the title screen drawn in the screen editor (title_chars, title_attrs: 1000 bytes each) into the screen
;   ($4400) and the color RAM ($D800).  copy1000 copies 1000 bytes from (sp) to (cp): three full pages of 256 bytes plus 232 bytes.
;   Changes A, X, Y, sp, cp.
hal_title:
        lda #<title_chars
        sta sp
        lda #>title_chars
        sta sp+1
        lda #<SCREEN
        sta cp
        lda #>SCREEN
        sta cp+1
        jsr copy1000
        lda #<title_attrs
        sta sp
        lda #>title_attrs
        sta sp+1
        lda #<COLOR
        sta cp
        lda #>COLOR
        sta cp+1
copy1000:                        ; 1000 bytes from (sp) to (cp)
        ldx #3                          ; three full pages (3 * 256 = 768 bytes) ...
.pg:    ldy #0
-       lda (sp),y
        sta (cp),y
        iny
        bne -
        inc sp+1                        ; next page in source and target
        inc cp+1
        dex
        bne .pg
        ldy #0
-       lda (sp),y
        sta (cp),y
        iny
        cpy #232                        ; ... plus the last 232 bytes: 1000 in total
        bne -
        rts

; ---------------------------------------------------------------- Load level: colors, map, status line
; hal_load_level: A = level number.  Sets the multicolor colors 1 and 2 of the level ($D022/$D023: so the same tile
;   characters look different on the meadow, in the cave, in the castle and in the sky), the six sky bars (skyc),
;   copies the tile map to LMAP ($9000), draws it and the status line.  Changes all registers.
hal_load_level:
        sta pt0
        tax                             ; X = level number as table index
        lda lvl_mc1,x
        sta $d022
        lda lvl_mc2,x
        sta $d023
        lda lvl_sky0,x                  ; the six sky colors of the level into the table of the interrupt
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
        ; copy the map to LMAP
        lda maps_lo,x                   ; pointer to the map data of this level (table in the main program)
        sta sp
        lda maps_hi,x
        sta sp+1
        ldy #0
-       lda (sp),y
        sta LMAP,y
        iny
        cpy #240                        ; 240 bytes = 20 columns * 12 rows
        bne -
        jsr hal_draw_level
        jmp hal_hud

; hal_hud: status line (row 0: "HUGO 000000   BILD n/4   LEBEN ***", German text from levels.inc) and scrolling text (row 24: name of the level).
;   The letters are characters of the own character set (numbers from CH_FONT).  The color value 1 (white) has bit 3 cleared:
;   the character is drawn as HIRES text (full sharpness, 8 x 8 dots), not as multicolor.
hal_hud:
        ldx #0
-       lda txt_hud,x
        sta SCREEN+1,x                  ; row 0, from column 1: 30 characters of text
        lda #1                          ; color value 1 = white, bit 3 off = hires
        sta COLOR+1,x
        inx
        cpx #30
        bne -
        ldx lvl
        txa
        clc
        adc #CH_DIGIT0+1                ; digit of the level (1..4) as a character
        sta SCREEN+20            ; level number
        lda #CH_STAR                    ; three life stars
        sta SCREEN+31
        sta SCREEN+32
        sta SCREEN+33
        lda #7
        sta COLOR+31                    ; yellow, also below 8 = hires
        sta COLOR+32
        sta COLOR+33
        ldx lvl                         ; row 24: name of the level (pointer from the table of texts)
        lda txt_lv_lo,x
        sta sp
        lda txt_lv_hi,x
        sta sp+1
        ldy #0
-       lda (sp),y
        sta SCREEN+960+7,y              ; 960 = 24 * 40: start of the last row; text from column 7
        lda #1                   ; character color white: color value below 8 = hires text (from 8 on the hires font would be read as multicolor)
        sta COLOR+960+7,y
        iny
        cpy #25
        bne -
        jmp hal_score_draw


; hal_score_draw: writes the score (sc2 sc1 sc0 as BCD) as six digits into the status line (columns 6..11).
;   put_bcd splits a BCD byte into two digits: upper half (shift right four times) and lower half (and #$0f);
;   character number = CH_DIGIT0 + digit.
hal_score_draw:
        lda sc2                         ; most significant two digits from column 6
        ldx #0
        jsr put_bcd
        lda sc1                         ; middle two digits from column 8
        ldx #2
        jsr put_bcd
        lda sc0                         ; least significant two digits from column 10
        ldx #4
put_bcd:                         ; A = BCD, X = column offset from column 6 (0, 2, 4); changes A
        pha
        lsr a
        lsr a
        lsr a
        lsr a
        clc
        adc #CH_DIGIT0
        sta SCREEN+6,x                  ; tens digit
        pla
        and #$0f
        clc
        adc #CH_DIGIT0
        sta SCREEN+7,x                  ; ones digit
        rts

; ---------------------------------------------------------------- Sprites
; hal_update_sprites: sets the sprite registers from the values of the game logic (h_px/h_py/h_fr, e_px/e_py/e_fr).
;   Conversion logical -> VIC coordinates (unchanged compared with the C64 in the VIC-II mode of the MEGA65):
;     x_vic = x + 24      (the visible screen starts at x = 24)
;     y_vic = y_foot - 20 + 50  = h_py + 30   for the hero  (sprite is 21 lines high, the feet are 20 lines below the
;                                              sprite start; the first screen row is raster line 50)
;     y_vic = e_py + 36   for the enemy  (foot 14 lines below the sprite start)
;   x can be above 255: the ninth bit is in $D010 (one bit per sprite).  Both sprites of a character get
;   the same position; the sprite pointers (SCREEN+$3F8 ff.) choose the image (frame number + base).
;   The sprite pointers are at the end of the screen memory, i.e. at $4400 + $3F8 = $47F8, not at $07F8 as on the C64.
;   Switches all four sprites on at the end.  Changes A, X and the scratch cells pt0..pt2.  Called from the main loop.
hal_update_sprites:
        ; hero: x + 24, y + 30 (foot y 20 lines below the sprite start, border at line 50)
        clc
        lda h_px_l
        adc #24
        sta pt0
        lda h_px_h
        adc #0
        sta pt1                  ; bit 8
        lda pt0
        sta $d000                       ; sprite 0 (hero overlay) x
        sta $d002                       ; sprite 1 (hero base) x
        lda h_py
        clc
        adc #30
        sta $d001                       ; y of the two hero sprites
        sta $d003
        ; enemy: foot 14 lines below the sprite start
        clc
        lda e_px_l
        adc #24
        sta pt0
        lda e_px_h
        adc #0
        sta pt2
        lda pt0
        sta $d004                       ; sprite 2 (enemy overlay) x
        sta $d006                       ; sprite 3 (enemy base) x
        lda e_py
        clc
        adc #36
        sta $d005                       ; y of the two enemy sprites
        sta $d007
        lda #0
        ldx pt1
        beq +
        ora #%00000011
+       ldx pt2
        beq +
        ora #%00001100
+       sta $d010                       ; bit 8 of the x coordinates (bits 0-1 hero, 2-3 enemy)
        lda h_fr                        ; frame number of the hero + base of the overlay/base set = sprite pointer
        clc
        adc #HERO_OVER_PTR
        sta SCREEN+$3f8                 ; pointer sprite 0
        lda h_fr
        clc
        adc #HERO_BASE_PTR
        sta SCREEN+$3f9                 ; pointer sprite 1
        lda e_fr
        clc
        adc #ENEMY_OVER_PTR
        sta SCREEN+$3fa                 ; pointer sprite 2
        lda e_fr
        clc
        adc #ENEMY_BASE_PTR
        sta SCREEN+$3fb                 ; pointer sprite 3
        lda #%00001111
        sta $d015                       ; switch sprites 0-3 on
        rts

; ---------------------------------------------------------------- Sound effects (second SID, voice 3: $D42E-$D434)
; An effect is a short sequence of sounds, described by 9 bytes:
;     0,1  start frequency (lo, hi)  16 bit: frequency in Hz = value * 985248 / 2^24 = value * 0.0587 (SID clock PAL, as on the C64).
;          Example $0C00 = 3072 = 180 Hz
;     2,3  frequency change per frame (lo, hi), signed: positive = tone rises, negative = falls
;     4    control byte: waveform ($11 triangle, $21 sawtooth, $41 pulse, $81 noise), bit 0 = gate (tone on)
;     5    duration in frames                    6 AD = attack/decay   7 SR = sustain level/release   8 pulse width (hi)
; hal_sfx copies the values into the SID registers and switches the tone on; sfx_update changes the frequency per frame and
; switches the tone off after the duration has run out (gate off).  sfx_on shows that an effect is currently running.
; Register layout: the second SID starts at $D420, voice 3 is, as with the first SID, 14 bytes (2 * 7) behind the start:
;     $D42E/$D42F frequency   $D430/$D431 pulse width   $D432 control byte (gate)   $D433 AD   $D434 SR   ($D438 volume)
; On the C64 the same registers are at $D40E-$D414 in the single SID, where the music has to pause during the effect.
; Effect  0 jump (rising tone)   1 landing (short noise)   2 step (click)   3 climbing (blip)
;         4 coin (fast upward sound)   5 Gumbo climbs (bubble)   6 level change jingle   7 fanfare
; The table is byte for byte the same as on the C64; only the register addresses of the HAL routines differ.
sfx_tab:
        .byte $00,$0c,$60,$00,$41,22,$08,$a4,$08     ; 0 jump: rising tone
        .byte $00,$14,$80,$ff,$81,7,$00,$a5,$08      ; 1 landing: noise
        .byte $00,$28,$00,$00,$81,2,$00,$90,$08      ; 2 step: click
        .byte $00,$18,$00,$02,$11,4,$00,$a4,$08      ; 3 climbing: blip
        .byte $00,$40,$00,$03,$41,10,$00,$f0,$08     ; 4 coin: fast upward glide
        .byte $00,$0a,$b0,$ff,$11,12,$00,$a8,$08     ; 5 Gumbo: bubble
        .byte $00,$20,$00,$01,$41,40,$08,$f4,$08     ; 6 level change jingle
        .byte $00,$18,$00,$02,$41,60,$08,$f6,$08     ; 7 fanfare
sfx_on:   .byte 0                       ; 1 = effect running
sfx_cnt:  .byte 0                       ; remaining frames
sfx_fl:   .byte 0                       ; current frequency (lo)
sfx_fh:   .byte 0                       ; current frequency (hi)
sfx_dl:   .byte 0                       ; change per frame (lo)
sfx_dh:   .byte 0                       ; change per frame (hi)
sfx_ctrl: .byte 0                       ; control byte of the effect (waveform + gate)

; hal_sfx: A = effect number 0..7.  Row in sfx_tab = number * 9 (number*8 + number).  The gate is first switched off and then on again,
; so that the envelope starts from the beginning each time.  Changes A, X and pt0.  sfx_on first becomes 0, finally 1 again (as on the C64).
hal_sfx:
        sta pt0
        asl a
        asl a
        asl a
        clc
        adc pt0
        tax                             ; X = offset of the table row
        lda #0
        sta $d432                ; gate off (restart)
        sta sfx_on               ; effect off while setting (the interrupt calls sfx_update)
        lda sfx_tab,x
        sta sfx_fl
        sta $d42e                       ; frequency lo
        lda sfx_tab+1,x
        sta sfx_fh
        sta $d42f                       ; frequency hi
        lda sfx_tab+2,x
        sta sfx_dl
        lda sfx_tab+3,x
        sta sfx_dh
        lda sfx_tab+4,x
        sta sfx_ctrl
        lda sfx_tab+5,x
        sta sfx_cnt
        lda sfx_tab+6,x
        sta $d433                       ; AD
        lda sfx_tab+7,x
        sta $d434                       ; SR
        lda #0
        sta $d430                       ; pulse width lo
        lda sfx_tab+8,x
        sta $d431                       ; pulse width hi
        lda sfx_ctrl
        sta $d432                ; gate on
        lda #1
        sta sfx_on
        rts

; sfx_update: called once per frame by the interrupt.  Adds the frequency change, counts the duration down and ends the effect.
;   Changes A (the interrupt saves the registers).
sfx_update:
        lda sfx_on
        beq +                           ; no effect active: nothing to do
        clc
        lda sfx_fl
        adc sfx_dl                      ; frequency += change per frame (16 bit)
        sta sfx_fl
        sta $d42e
        lda sfx_fh
        adc sfx_dh
        sta sfx_fh
        sta $d42f
        dec sfx_cnt
        bne +
        lda sfx_ctrl
        and #$fe                        ; clear the gate (bit 0): the tone fades out with the release
        sta $d432
        lda #0
        sta sfx_on
+       rts
