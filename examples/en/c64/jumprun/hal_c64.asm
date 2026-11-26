; =====================================================================================================================
; hal_c64.asm - hardware layer (HAL) of the C64 for "Hugo & Gumbo"
; =====================================================================================================================
;
; The game logic (lib/core.asm) knows no hardware. This file translates its requests into accesses to
; the C64 chips.
;
; INTERFACE (described completely in core.asm; here, what this HAL provides and needs)
;   for the core:            hal_sfx (A = effect 0..7), hal_put_tile (= tile_put of the tile library: X = column, Y = row,
;                            A = tile), hal_score_draw, sfx_on (byte: not 0 while an effect is sounding)
;   for the main program:    hal_init, hal_wait_frame, hal_blank, hal_unblank, hal_title, hal_load_level (A = scene),
;                            hal_update_sprites; also the constants SCREEN and COLOR and the table skyc (used by show_title)
;   The HAL reads from the core: lvl, sc0..sc2 and the output values h_px_l/h/h_py/h_fr, e_px_l/h/e_py/e_fr; from lib/levels.inc: lvl_mc1,
;   lvl_mc2, lvl_sky0..5, txt_hud, txt_lv_lo/hi, CH_DIGIT0, CH_STAR; from the export: title_chars, title_attrs; from the tile
;   library: tile_put, screen_select_a, screen_draw_all.  It changes only its own zero page cells and may change
;   A, X, Y and the flags in all routines.  The music (lib/music.inc) reads sfx_on to skip voice 3 during an effect.
;   Differences from the other systems (sprites, sound, start): see core.asm, section "DIFFERENCES BETWEEN THE SYSTEMS".
;
;   VIC-II (picture)
;     * Text MULTICOLOR MODE, 40 x 25 characters. Each character is 4 x 8 wide pixels in size (one pixel = 2 dots wide,
;       hence 160 x 200 visible dots). Two bits per pixel give the choice:
;           00 = background colour 0  ($D021)     01 = multicolor 1 ($D022)
;           10 = multicolor 2         ($D023)     11 = character colour from colour RAM ($D800, only colours 0-7)
;       The colour RAM byte switches a character between multicolor (bit 3 set) and hires (bit 3 cleared).
;     * The OWN CHARACTER SET is at $2000 (2 KB); the screen memory at $0400 (1000 bytes).
;     * SPRITES: hero and enemy each consist of TWO sprites on top of each other:
;           - a MULTICOLOR sprite for the areas (shirt, skin, trousers): sprite 1 (hero) and sprite 3 (enemy),
;           - a HIRES sprite for outline, eye and mouth (fine details, one colour): sprite 0 (hero), sprite 2 (enemy).
;       At the same position the LOWER sprite number is in FRONT, that is why the hires overlays are sprites 0 and 2.
;     * SKY: six background colours on top of each other (raster bars). A raster interrupt changes $D021 at fixed lines.
;
;   SID (sound)
;     * Voices 1 + 2 + 3 play the music (lib/music.inc, tracker export). Voice 3 is borrowed for SOUND EFFECTS: while
;       an effect sounds (sfx_on <> 0), the music pauses there.
;
; MEMORY MAP (VIC bank 0, the VIC sees addresses $0000-$3FFF; $1000-$1FFF there is the character ROM and therefore unusable)
;     $0400-$07E7  screen memory              $07F8-$07FF  sprite pointers (block number = address / 64)
;     $2000-$27FF  character set              $2800-$31FF  sprites (hero 14+14 frames, enemy 6+6 frames)
;     $D800-$DBE7  colour RAM                 $C000-$C0EF  scene map (LMAP, used by the game logic)
;
; POINTERS IN THE ZERO PAGE AREA: $50-$56 (sp, cp: screen and colour RAM pointers; pt0-pt2: scratch memory of this file).
;   Free of overlap with the game logic ($10-$43), the tile library ($58-$5F) and the music ($F7-$FA).
;   The raster interrupt (irq, music_play, sfx_update) uses only cells of the music and its own memory locations, none of the game logic.
; =====================================================================================================================

.const SCREEN = $0400
.const COLOR  = $d800
; Pointers for (indirect),y accesses and scratch memory of the HAL (zero page; the game logic uses $10-$43, the music $F7-$FA)
.const sp  = $50                 ; pointer (2 bytes)
.const cp  = $52
.const pt0 = $54
.const pt1 = $55
.const pt2 = $56

; Sprite pointers: the VIC reads sprite data from (pointer * 64) within the bank. Example: $2800 / 64 = $A0.
; The four sprite sets follow each other: hero base 14 frames ($A0..$AD), hero overlay 14 ($AE..$BB), enemy base 6 ($BC..$C1),
; enemy overlay 6 ($C2..$C7).
.const HERO_BASE_PTR  = $a0      ; $2800 / 64
.const HERO_OVER_PTR  = $ae      ; $2b80 / 64
.const ENEMY_BASE_PTR = $bc      ; $2f00 / 64
.const ENEMY_OVER_PTR = $c2      ; $3080 / 64

; ---------------------------------------------------------------- Start
; hal_init: sets up the VIC, the interrupt and the sprites once at program start.  Does NOT release the interrupt yet
;           (the main program does that with cli, after the music is initialised as well).
hal_init:
        sei                             ; interrupts off while the chips are being set up
        lda #$7f                        ; bit 7 = 0, all others 1: "clear" -> all CIA interrupt sources off (CIA 1 at $DC0D, CIA 2 at $DD0D)
        sta $dc0d                ; CIA interrupts off
        sta $dd0d
        lda $dc0d
        lda $dd0d
        lda $dd00
        ora #$03                 ; VIC bank 0 ($0000-$3fff)
        sta $dd00
        lda #$18                 ; screen $0400, character set $2000
        sta $d018
        lda #$18                 ; multicolor text, 40 columns
        sta $d016
        lda #0
        sta $d020
        sta $d015                ; sprites off
        sta $d01b                ; sprites in front of the background
        sta $d017
        sta $d01d
        lda #%00001010           ; sprites 1 and 3 multicolor (the base sprites; the hires overlays 0 and 2 stay single colour)
        sta $d01c
        lda #10                  ; sprite multicolor 1 (light red: skin), valid for all multicolor sprites
        sta $d025
        lda #14                  ; sprite multicolor 2 (light blue: trousers, eyes)
        sta $d026
        lda #0
        sta $d027                ; hero overlay black
        lda #2
        sta $d028                ; hero base red (colour for the bit pair 11, e.g. the shirt)
        lda #0
        sta $d029                ; enemy overlay black
        lda #5
        sta $d02a                ; enemy base green (bit pair 11)
        lda #<irq                       ; bend the KERNAL IRQ vector $0314/$0315 to our routine
        sta $0314
        lda #>irq
        sta $0315
        lda #85                         ; first raster interrupt at line 85 (end of the first sky bar)
        sta $d012
        lda $d011
        and #$7f                        ; bit 7 of $D011 = bit 8 of the raster line: 0, because all our lines are below 256
        sta $d011
        lda #1                          ; $D01A: enable the raster interrupt
        sta $d01a                ; raster interrupt on
        lda #0
        sta band
        sta vblank
        sta sfx_on
        rts

; ---------------------------------------------------------------- Interrupt: sky bars and frame clock
; The raster interrupt fires six times per frame, each time at a raster line from irq_line (85, 119, 153, 187, 221, 251).
; On every call the background colour is set to the next bar colour: line 51 is the top screen line,
; the bars are 34 lines high each.  irq_ci distributes the colours (bars 1..5, then bar 0 again for the next frame).
; On the LAST call (line 251, below the visible picture) what should run exactly once per frame happens in addition:
;   * music_play   the music one step further (write SID registers),
;   * sfx_update   the running sound effect one step further,
;   * vblank = 1   signal to the main program: "the next frame may be calculated" (see hal_wait_frame).
; Acknowledge: writing 1 to $D019 clears the raster interrupt flag.  Exit: $EA81 restores the registers saved by the KERNAL entry
; and returns with rti.  The vector $0314/$0315 is jumped to by the KERNAL IRQ (ROMs are on).
; band = number of the bar whose colour is set now (0..5); irq_ci gives the entry in skyc: 1,2,3,4,5 and finally 0.
irq:    lda #1
        sta $d019
        ldx band
        lda irq_ci,x
        tay
        lda skyc,y
        sta $d021
        cpx #5
        bne +
        jsr music_play                  ; music: one step (write SID registers)
        jsr sfx_update
        lda #1
        sta vblank
+       inx
        cpx #6
        bcc +
        ldx #0
+       stx band
        lda irq_line,x
        sta $d012
        jmp $ea81                       ; KERNAL exit: restore registers and rti

irq_line: .byte 85,119,153,187,221,251     ; raster line of interrupt no. 0..5 (distance 34, the last one is below the picture)
irq_ci:   .byte 1,2,3,4,5,0               ; index in skyc that is set at interrupt no. 0..5
band:     .byte 0                         ; number of the next interrupt
vblank:   .byte 0                         ; 1 = end of frame reported (set by the interrupt, cleared by hal_wait_frame)
skyc:     .byte 3,3,3,3,3,14              ; background colours of the six bars (hal_load_level / show_title overwrite them for each scene)

; hal_wait_frame: waits until the interrupt reports the next end of frame (so the game logic runs exactly 50 times per second).
;   Clears vblank first, so it always waits for the NEXT end of frame (up to a whole frame).  Changes only A.
hal_wait_frame:
        lda #0
        sta vblank
-       lda vblank
        beq -
        rts


; hal_blank / hal_unblank: screen off/on (bit 4 of $D011 = DEN: display enable).  hal_blank also switches off all sprites
; (hal_unblank does not; hal_update_sprites switches the sprites on again).  While a new scene is being built, the
; screen stays black this way, no half-finished picture is seen.  hal_blank waits for the end of the frame first, so that the switch is
; flipped in the invisible part of the picture.  Both change only A.
hal_blank:
        jsr hal_wait_frame       ; switch the screen off only in the blanking area
        lda #$0b                        ; $D011 = $0B: bit 4 (DEN) cleared = screen off; bit 3 = 25 rows; Y scroll 3
        sta $d011
        lda #0
        sta $d015                       ; sprite enable: all sprites off (bits 0-3 would be sprites 0-3)
        rts
hal_unblank:
        lda #$1b                        ; $D011 = $1B: DEN set = screen on, 25 rows (bit 3), Y scroll 3 (standard)
        sta $d011
        rts

; ---------------------------------------------------------------- Tiles
; A tile consists of 2 x 2 characters (16 x 16 dots).  Tile (column c, row k) lies on the screen in the character cells
; (2c, 2k), (2c+1, 2k), (2c, 2k+1), (2c+1, 2k+1).  Drawing is done by the tile library (#include <c64/tiles.asm>, included in jumprun.asm
; and bound with tiles_use to the tile set hg and the screen bank levels):
;   hal_put_tile   = tile_put: place a tile.  Input X = column (0..19), Y = row (0..11), A = tile number.  Character and colour of the tile
;                    (colour RAM value: bit 3 = multicolor, bits 0-2 = character colour) come from the tables of the tile set (hg_c0..hg_c3, hg_col).
;   screen_select_a: A = scene number; copies the map of the scene to LMAP (TILE_MAPRAM): the game logic deletes coins from this copy.
;   screen_draw_all: draws the whole scene from LMAP (12 rows x 20 columns).  Takes several frames; therefore with the screen switched off.
.const hal_put_tile = tile_put

; hal_title: copies the title picture drawn in the screen editor (title_chars, title_attrs: 1000 bytes each) into the screen
;   and the colour RAM.  copy1000 copies 1000 bytes from (sp) to (cp): three full pages of 256 bytes plus 232 bytes.
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
        inc sp+1
        inc cp+1
        dex
        bne .pg
        ldy #0
-       lda (sp),y
        sta (cp),y
        iny
        cpy #232                        ; ... plus the last 232 bytes: together 1000
        bne -
        rts

; ---------------------------------------------------------------- Load scene
; hal_load_level: A = scene number.  Sets the multicolor colours 1 and 2 of the scene ($D022/$D023: this way the same tile
;   characters look different on meadow, in the cave, in the castle and in the sky), the six sky bars (skyc),
;   copies the tile map to LMAP (screen_select_a), draws it (screen_draw_all) and the status line.
hal_load_level:
        sta pt0
        tax
        lda lvl_mc1,x
        sta $d022
        lda lvl_mc2,x
        sta $d023
        lda lvl_sky0,x
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
        ; Copy the map to LMAP and draw the scene (scene number = screen number of the bank levels)
        lda pt0
        jsr screen_select_a
        jsr screen_draw_all
        jmp hal_hud

; hal_hud: status line (row 0: score, scene number and lives) and scrolling text (row 24: name of the scene).
;   The font consists of characters of the own character set (numbers from CH_FONT), the colour < 8 switches the character to hires.
hal_hud:
        ldx #0
-       lda txt_hud,x
        sta SCREEN+1,x
        lda #1
        sta COLOR+1,x
        inx
        cpx #30
        bne -
        ldx lvl
        txa
        clc
        adc #CH_DIGIT0+1                ; digit of the scene (1..4) as a character
        sta SCREEN+20            ; scene number
        lda #CH_STAR
        sta SCREEN+31
        sta SCREEN+32
        sta SCREEN+33
        lda #7
        sta COLOR+31
        sta COLOR+32
        sta COLOR+33
        ldx lvl
        lda txt_lv_lo,x
        sta sp
        lda txt_lv_hi,x
        sta sp+1
        ldy #0
-       lda (sp),y
        sta SCREEN+960+7,y
        lda #1                   ; character colour white: colour value below 8 = hires text (from 8 on the hires font would be read as multicolor)
        sta COLOR+960+7,y
        iny
        cpy #25
        bne -
        jmp hal_score_draw


; hal_score_draw: writes the score (sc2 sc1 sc0 as BCD) as six digits into the status line (columns 6..11).
;   put_bcd splits a BCD byte into two digits: upper half (shift right four times) and lower half (and #$0f);
;   character number = CH_DIGIT0 + digit.
hal_score_draw:
        lda sc2                         ; most significant BCD byte: columns 6-7
        ldx #0
        jsr put_bcd
        lda sc1                         ; columns 8-9
        ldx #2
        jsr put_bcd
        lda sc0                         ; least significant BCD byte: columns 10-11 (the rts of put_bcd also ends hal_score_draw)
        ldx #4
put_bcd:                         ; A = BCD, X = column offset from column 6 (0, 2, 4); changes A
        pha
        lsr a
        lsr a
        lsr a
        lsr a
        clc
        adc #CH_DIGIT0
        sta SCREEN+6,x
        pla
        and #$0f
        clc
        adc #CH_DIGIT0
        sta SCREEN+7,x
        rts

; ---------------------------------------------------------------- Sprites
; hal_update_sprites: sets the sprite registers from the values of the game logic (h_px/h_py/h_fr, e_px/e_py/e_fr).
;   Conversion logical -> VIC coordinates:
;     x_vic = x + 24      (the visible screen starts at x = 24)
;     y_vic = y_foot - 20 + 50  = h_py + 30   for the hero  (sprite is 21 lines high, the feet are 20 lines below the
;                                              sprite start; the first screen line is raster line 50)
;     y_vic = e_py + 36   for the enemy  (foot 14 lines below the sprite start)
;   x can be above 255: the ninth bit is in $D010 (one bit per sprite).  Both sprites of a figure get
;   the same position; the sprite pointers choose the frame (frame number + base).  Switches all four sprites on at the end.
;   Changes A, X and the scratch cells pt0-pt2.  Called once per frame from the main loop (not from the interrupt).
hal_update_sprites:
        ; Hero: x + 24, y + 30 (foot y 20 lines below the sprite start, border at line 50)
        clc
        lda h_px_l
        adc #24
        sta pt0
        lda h_px_h
        adc #0
        sta pt1                  ; bit 8 of the x coordinate of the hero (0 or 1)
        lda pt0
        sta $d000                       ; x (low 8 bits) of sprite 0 (hero overlay) ...
        sta $d002                       ; ... and sprite 1 (hero base)
        lda h_py
        clc
        adc #30
        sta $d001
        sta $d003
        ; Enemy: foot 14 lines below the sprite start
        clc
        lda e_px_l
        adc #24
        sta pt0
        lda e_px_h
        adc #0
        sta pt2                         ; bit 8 of the x coordinate of the enemy
        lda pt0
        sta $d004                       ; sprite 2 (enemy overlay) and sprite 3 (enemy base)
        sta $d006
        lda e_py
        clc
        adc #36
        sta $d005
        sta $d007
        lda #0
        ldx pt1
        beq +
        ora #%00000011
+       ldx pt2
        beq +
        ora #%00001100
+       sta $d010                       ; bit 8 of the x coordinates (bits 0-1 hero, 2-3 enemy)
        lda h_fr                        ; sprite pointer = frame number + base pointer of the respective set (overlay: sprite 0 / 2, base: 1 / 3)
        clc
        adc #HERO_OVER_PTR
        sta $07f8
        lda h_fr
        clc
        adc #HERO_BASE_PTR
        sta $07f9
        lda e_fr
        clc
        adc #ENEMY_OVER_PTR
        sta $07fa
        lda e_fr
        clc
        adc #ENEMY_BASE_PTR
        sta $07fb
        lda #%00001111
        sta $d015
        rts

; ---------------------------------------------------------------- Sound effects (SID voice 3: $D40E-$D414)
; An effect is a short sequence of sounds, described by 9 bytes:
;     0,1  start frequency (lo, hi)  16 bit: frequency in Hz = value * 985248 / 2^24 = value * 0.0587 (PAL).  Example $0C00 = 3072 = 180 Hz
;     2,3  frequency change per frame (lo, hi), signed: positive = tone rises, negative = falls
;     4    control byte: waveform ($11 triangle, $21 sawtooth, $41 pulse, $81 noise), bit 0 = gate (tone on)
;     5    duration in frames                    6 AD = attack/decay   7 SR = sustain level/release   8 pulse width (hi)
; hal_sfx copies the values into the SID registers and switches the tone on; sfx_update changes the frequency per frame and
; switches the tone off when the duration has run out (gate off).  sfx_on tells the music that voice 3 is currently in use.
; Effect  0 jump (rising tone)   1 landing (short noise)   2 step (click)   3 climbing (blip)
;         4 coin (fast upward sound)   5 Gumbo climbs (bubble)   6 scene change jingle   7 fanfare
sfx_tab:
        .byte $00,$0c,$60,$00,$41,22,$08,$a4,$08     ; 0 jump: rising tone
        .byte $00,$14,$80,$ff,$81,7,$00,$a5,$08      ; 1 landing: noise
        .byte $00,$28,$00,$00,$81,2,$00,$90,$08      ; 2 step: click
        .byte $00,$18,$00,$02,$11,4,$00,$a4,$08      ; 3 climbing: blip
        .byte $00,$40,$00,$03,$41,10,$00,$f0,$08     ; 4 coin: fast upward glide
        .byte $00,$0a,$b0,$ff,$11,12,$00,$a8,$08     ; 5 Gumbo: bubble
        .byte $00,$20,$00,$01,$41,40,$08,$f4,$08     ; 6 scene change jingle
        .byte $00,$18,$00,$02,$41,60,$08,$f6,$08     ; 7 fanfare
sfx_on:   .byte 0                         ; not 0 while an effect is sounding (core.asm and the music read it)
sfx_cnt:  .byte 0                         ; remaining frames of the effect
sfx_fl:   .byte 0                         ; current frequency (lo)
sfx_fh:   .byte 0                         ; current frequency (hi)
sfx_dl:   .byte 0                         ; frequency change per frame (lo, 16 bit signed)
sfx_dh:   .byte 0                         ; ... (hi)
sfx_ctrl: .byte 0                         ; control byte (waveform + gate) of the effect

; hal_sfx: A = effect number 0..7.  Row in sfx_tab = number * 9 (number*8 + number).  The gate is first switched off and then on again,
; so that the envelope starts from the beginning each time.  Changes A, X and pt0.  sfx_on first becomes 0 (sfx_update then does nothing while
; the values are being set) and finally 1 again.
hal_sfx:
        sta pt0
        asl a
        asl a
        asl a
        clc
        adc pt0
        tax
        lda #0
        sta $d412                ; gate off (restart)
        sta sfx_on               ; effect off while setting (the interrupt calls sfx_update)
        lda sfx_tab,x
        sta sfx_fl
        sta $d40e
        lda sfx_tab+1,x
        sta sfx_fh
        sta $d40f
        lda sfx_tab+2,x
        sta sfx_dl
        lda sfx_tab+3,x
        sta sfx_dh
        lda sfx_tab+4,x
        sta sfx_ctrl
        lda sfx_tab+5,x
        sta sfx_cnt
        lda sfx_tab+6,x
        sta $d413
        lda sfx_tab+7,x
        sta $d414
        lda #0
        sta $d410
        lda sfx_tab+8,x
        sta $d411
        lda sfx_ctrl
        sta $d412                ; gate on
        lda #1
        sta sfx_on
        rts

; sfx_update: called once per frame by the interrupt.  Adds the frequency change, counts the duration down and ends the effect.
;   Changes A (the interrupt entry saves the registers).
sfx_update:
        lda sfx_on
        beq +
        clc
        lda sfx_fl
        adc sfx_dl                      ; frequency += change per frame (16 bit)
        sta sfx_fl
        sta $d40e
        lda sfx_fh
        adc sfx_dh
        sta sfx_fh
        sta $d40f
        dec sfx_cnt
        bne +
        lda sfx_ctrl
        and #$fe                        ; clear gate (bit 0): the tone fades out with the release
        sta $d412
        lda #0
        sta sfx_on
+       rts
