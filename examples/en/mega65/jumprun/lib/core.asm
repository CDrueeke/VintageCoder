; NOTE: This file is its own copy of the game logic for this example project (originally one shared file for all systems;
; each system maintains its copy independently; changes here do not affect the other systems.
; =====================================================================================================================
; core.asm - game logic of the example "Hugo & Gumbo"  (6502 assembler, platform neutral)
; =====================================================================================================================
;
; WHAT THIS FILE DOES
;   It contains everything in the game that is independent of the hardware:
;     * the AUTOPILOT, which guides the hero Hugo through a level without a player (walk, jump, climb, wait),
;     * the MOVEMENT of the hero (walking, jump as a sine curve, climbing),
;     * the ARTIFICIAL INTELLIGENCE of the enemy Gumbo (chases Hugo, climbs ladders, but can NEVER jump),
;     * small helpers: read the tile at a position, collect coins, count points.
;   What it does NOT do: it never writes to hardware registers. Everything the hardware needs is in the
;   "hardware layer" (HAL) of the respective system (e.g. hal_c64.asm).
;
; HOW IT IS INCLUDED
;   The includer (jumprun.asm of the respective system) first defines two constants and then includes this file:
;     ZP    = first zero-page address for the variables of this file (it uses ZP up to ZP_END-1, i.e. 52 bytes)
;     LMAP  = address of 240 bytes of RAM for the level map (20 columns x 12 rows, one byte per tile)
;   Everything else is outside (tables, HAL routines); the assembler resolves the references, so the order of the
;   .include lines does not matter.  Zero-page layout per system (the areas must not overlap):
;     C64, MEGA65:  core $10-$43   HAL $50-$56   tile library (C64 only) $58-$5F   music (lib/music.inc) $F7-$FA
;     Atari:        core $80-$B3   HAL $B4-$BB   RMT player $CB-$DD
;     Plus/4:       core $80-$B3   HAL $C0-$E3
;
; INTERFACE BETWEEN CORE, MAIN PROGRAM AND HAL  (applies equally to C64, Atari, Plus/4 and MEGA65; the head of each HAL file
; repeats the part that concerns it)
;
;   A) What the CORE expects from the HAL.  It calls these routines; afterwards it assumes NO register and NO flag is preserved.
;      The HAL, on the other hand, must never change the zero-page cells ZP .. ZP_END-1 (it has its own, see above).
;        hal_sfx          A = effect number 0..7: start a sound effect, a running one is replaced.  Numbers: 0 jump, 1 landing,
;                         2 step, 3 climbing (Hugo), 4 coin, 5 climbing (Gumbo), 6 level change jingle, 7 fanfare (finale).
;                         The core starts 2, 3, 4 and 5 itself; 0 and 1 come from the scripts (command SFX), 6 and 7 from the main program.
;        hal_put_tile     X = column 0..19, Y = row 0..11, A = tile number: put a tile (2 x 2 characters) on the screen.
;                         The core only calls it with A = 0 (empty) to delete a collected coin.
;        hal_score_draw   no input: write the score from sc2:sc1:sc0 (BCD) into the status line.
;        sfx_on           memory byte of the HAL (not a subroutine): not 0 while an effect is sounding.  The core only reads it,
;                         so that step and climbing sounds do not cut off a running effect.
;   B) What the CORE expects as data (tables from lib/levels.inc; on the C64 partly from the export of the tile set):
;        tile_attr        property per tile number (0 empty, 1 solid, 2 ladder).  Atari, Plus/4, MEGA65: table in lib/levels.inc.
;                         C64: table hg_props from the export of graphics/hg.vctiles (jumprun.asm sets .const tile_attr = hg_props).
;        lvl_hx/hy/hd     start position (x as .word, foot y, facing direction) of the hero per level;  lvl_ex/ey/ed the same for the enemy
;        script_lo/hi     pointer to the autopilot script per level
;   C) What the CORE offers (callers: main program or HAL):
;        core_level_init  A = level number 0..3: set the game state for a level
;        hero_update, enemy_update       one game step each, once per frame
;        core_prepare_sprites            computes the output values h_px_l.. / e_px_l.. for the HAL
;        get_tile         X = column, Y = row -> A = tile number from LMAP (used by hal_draw_level of the HALs for Atari, Plus/4
;                         and MEGA65; the C64 draws with the tile library c64/tiles.asm)
;      Variables of the core that are read or written from outside:
;        lvl (level number 0..3; the HAL reads it in hal_hud), lvl_done (main program reads), sc0/sc1/sc2 (points, BCD; the
;        main program clears them, the HAL reads them), tmr_l/tmr_h (counter of the title screen, main program only), hx../hy/hdir/hmode/
;        hatim and ex../ey/edir/emode/eatim (the title screen sets them by hand), h_px_l/h_px_h/h_py/h_fr and e_px_l/e_px_h/e_py/e_fr
;        (output of the core to hal_update_sprites).
;   D) What the MAIN PROGRAM (not the core) expects from the HAL:
;        hal_init             set up the chips; C64, Plus/4, MEGA65: the interrupts stay disabled, the main program enables them with
;                             cli.  Atari: no sei/cli needed, hal_init enables the NMIs (DLI, vertical blank) itself.
;        hal_wait_frame       wait for the next frame (frame clock 50 Hz PAL; one game step per frame)
;        hal_blank/hal_unblank   screen off / on (hal_blank first waits for the end of a frame)
;        hal_title            bring the title screen into screen memory
;        hal_load_level       A = level number: colors, sky, map into LMAP, tiles and status line
;        hal_update_sprites   bring hero and enemy from h_*/e_* onto the screen
;      In addition, show_title in the main program reaches DIRECTLY into HAL internals: sky color table skyc, color registers of the video chip,
;      screen memory SCREEN/COLOR (blinking notice) and, on the Plus/4, the background copy BGS.  The main program is therefore
;      specific to each system; only the core is really shared.
;
; DIFFERENCES BETWEEN THE SYSTEMS  (details in the head of the respective HAL file)
;   Screen:    C64, MEGA65: VIC-II multicolor text 40 x 25 with color per cell in color RAM.  Plus/4: TED multicolor text, color RAM value with
;              luminance.  Atari: ANTIC mode 4 with ONE color table per level; bit 7 of the character code switches to COLPF3 (grass, bush).
;   Characters: C64, MEGA65: two sprites per character (multicolor areas + hires outline).  Atari: hero = four players, enemy = four
;              missiles.  Plus/4: NO sprites; the characters are assembled as "character sprites" from redefined characters.
;   Conversion of the logical points: C64, MEGA65: 1 point = 1 sprite pixel (x + 24, y + 30 or + 36).  Atari: 1 color clock = 2 points.
;              Plus/4: 1 multicolor pixel = 2 points, fine position via pre-shifted images and line offset.
;   Sky:       six background bars; C64, Plus/4, MEGA65 by raster interrupt, Atari by display list interrupt.
;   Sound:     C64: SID, effects borrow voice 3 (the music pauses there).  MEGA65: music on SID 1, effects on SID 2.
;              Atari: POKEY, music (RMT) on channels 1-3, effects on channel 4.  Plus/4: TED, melody voice 1, bass voice 2, effects
;              borrow voice 2.
;   Frame clock: raster interrupt (C64, MEGA65 at line 251; Plus/4 at line 0) or counter RTCLOK of the operating system (Atari).
;   Start:     BASIC starter at $0801 (C64), $1001 (Plus/4, then ROM off), $2001 (MEGA65, the BASIC 65 environment is cleared away);
;              Atari: XEX file with start address.
;   Tiles:     the C64 draws with the tile library (tile_put, screen_draw_all); the three other systems have their own loop
;              hal_draw_level and the tile list lv1_tiles (4 characters per tile).
;
; COORDINATES
;   All positions are LOGICALLY 320 x 200 points in size, independent of the system (the HAL converts to the hardware).
;     x  = 16 bits (hx_h:hx_l) plus 8 bits of fraction (hx_f), i.e. fixed point 8.8: 0.5 point = $80 in the fraction part.
;          It always means the LEFT EDGE of the character; the character is 24 points wide, its middle is at x + 12.
;     y  = 8 bits, "foot y": the row on which the feet stand = top edge of the tile on which the character stands.
;          A tile is 16 points high; tile row k starts at y = 16 * k. Row 0 is the status line.
;   The tile map has 20 columns (x / 16) and 12 rows (y / 16).
;
; FIXED POINT 8.8 ("how do you calculate with half points?")
;   A value like 1.5 points per frame is stored as the 16-bit number $0180: high byte = whole points (1),
;   low byte = fraction in 1/256 ($80 = 0.5). With every addition the fraction produces a carry
;   into the whole points; this way characters move smoothly with fractions of points per frame.
;
; FLOW PER FRAME (called by the main program):
;   hero_update -> enemy_update -> core_prepare_sprites -> hal_update_sprites
;
; =====================================================================================================================

; movement state of the hero (hmode)
.const HM_STAND = 0                     ; stands (also waiting and after the end of the script)
.const HM_WALK  = 1                     ; walks (hdir determines the direction)
.const HM_JUMP  = 2                     ; jump (32 frames long)
.const HM_CLIMB = 3                     ; climbs (hclimb determines up or down)

; ---- Variables (zero page). The zero page is faster and shorter to address than normal memory (2 instead of 3 bytes,
; ---- 3 instead of 4 cycles); that is why all frequently used variables are located there.
; ---- Naming rule: h... = hero, e... = enemy, t0..t3 = scratch memory for intermediate values.
.const st      = ZP + 0      ; reserved; st is not used anywhere in the core and in the main programs
.const lvl     = ZP + 1      ; number of the running level 0..3 (core_level_init sets it; show_title sets 0 for the title screen)
.const tmr_l   = ZP + 2      ; 16-bit frame counter (tmr_h:tmr_l); the core does not use it, the title screen of the main program
.const tmr_h   = ZP + 3      ; counts down 400 frames with it
; Scratch memory: every routine may overwrite t0..t3 freely.  Their content is valid only WITHIN a routine and across calls
; of helper routines that demonstrably do not touch it; ap_fetch passes coin column/row to do_coin in t0/t1.
.const t0      = ZP + 4
.const t1      = ZP + 5
.const t2      = ZP + 6
.const t3      = ZP + 7
; Autopilot: ap_ptr points to the next script byte, ap_cnt counts the frames the running command still lasts.
.const ap_ptr  = ZP + 8      ; 2 bytes
.const ap_cnt  = ZP + 10     ; frames the running command still lasts; 0 = fetch next command; 255 = script finished
.const lvl_done= ZP + 11     ; 0 = level running, 1 = autopilot has reached END (the main program waits for it)
; Hero: position (fixed point 8.8 in x), movement state and speeds.
;   hmode: 0 stand, 1 walk, 2 jump, 3 climb.   hdir: 0 = looks/walks right, 1 = left.
;   hvx/hvy: jump speed per frame in 8.8 (signed).   jt: step in the jump (0..32).
.const hx_f    = ZP + 12     ; hero x = hx_h:hx_l . hx_f
.const hx_l    = ZP + 13
.const hx_h    = ZP + 14
.const hy      = ZP + 15     ; foot y (during the jump: base line)
.const hy_f    = ZP + 16     ; fraction of hy (1/256 point); only matters during the jump
.const hdir    = ZP + 17     ; 0 right, 1 left
.const hmode   = ZP + 18
.const hatim   = ZP + 19     ; animation counter of the hero (per walk/climb frame + 1; core_prepare_sprites divides it)
.const hvx_l   = ZP + 20     ; jump speed x per frame, 8.8 signed (low / high byte)
.const hvx_h   = ZP + 21
.const hvy_l   = ZP + 22     ; jump speed y per frame, 8.8 signed
.const hvy_h   = ZP + 23
.const jt      = ZP + 24     ; jump step 0..32
.const hclimb  = ZP + 25     ; 0 up, 1 down
; Enemy: same idea as the hero, but with its own state machine (emode: 0 walk, 1 climb, 2 wait).
.const ex_f    = ZP + 26     ; enemy x
.const ex_l    = ZP + 27
.const ex_h    = ZP + 28
.const ey      = ZP + 29
.const ey_f    = ZP + 30
.const edir    = ZP + 31
.const emode   = ZP + 32     ; 0 walk, 1 climb, 2 wait
.const eatim   = ZP + 33     ; animation counter of the enemy
.const eclimb  = ZP + 34     ; climbing direction of the enemy: 0 up, 1 down (copied from want when stepping onto the ladder)
.const ewait   = ZP + 35     ; remaining frames in the state "wait" (emode 2)
.const ecol    = ZP + 36     ; column of the ladder on which the enemy climbs (set by find_ladder)
; Score as a three-digit BCD number (two decimal digits per byte), least significant byte first.
.const sc0     = ZP + 37     ; points BCD (low .. high)
.const sc1     = ZP + 38
.const sc2     = ZP + 39
.const sfx_t1  = ZP + 40     ; common frame counter for the step and climbing sounds of Hugo and Gumbo
; Output for the HAL: the HAL reads these values and uses them to paint the characters (see core_prepare_sprites).
; px = x position (16 bit), py = foot y (8 bit), fr = frame number of the animation (0..13 hero, 0..5 enemy).
.const h_px_l  = ZP + 41     ; output for the HAL
.const h_px_h  = ZP + 42
.const h_py    = ZP + 43
.const h_fr    = ZP + 44
.const e_px_l  = ZP + 45
.const e_px_h  = ZP + 46
.const e_py    = ZP + 47
.const e_fr    = ZP + 48
; Target position the enemy heads for (e.g. foot of a ladder), want: 0 = he wants to go up, 1 = down.
.const tx_l    = ZP + 49     ; target position of the enemy (x of the left edge, 16 bit; set by find_ladder)
.const tx_h    = ZP + 50
.const want    = ZP + 51     ; wanted ladder direction: 0 = up, 1 = down
.const ZP_END  = ZP + 52     ; first free zero-page cell behind the core

; ---------------------------------------------------------------- Tables
; Sine table for the jump. A jump lasts 32 frames (t = 0..32). In every frame the character is
;     40 * sin(pi * t / 32)
; points above its base line. At t = 0 and t = 32 the sine is 0 (take-off and landing on the base line),
; at t = 16 it is 1 (highest point, 40 points high = two and a half tiles).  The numbers are computed
; at assembly time (.repeat and the function sin), so no table has to be typed by hand.
; Example: t = 8 gives 40 * sin(pi/4) = 28.28, rounded 28.
sinetab:                     ; jump arc: 40 * sin(pi * t / 32), t = 0..32
    .repeat 33, i
        .byte round(40 * sin(i * pi / 32))
    .endrepeat
; row20[k] = k * 20: start index of tile row k in the level map (20 tiles per row).  This gives
; index = row20[row] + column; with at most 11 * 20 + 19 = 239 the index fits in a byte.
row20:
    .repeat 12, i
        .byte i * 20
    .endrepeat
; Order of the walking frames (frame numbers in the sprite set): 1, 2, 3, 2 - step, middle, step, middle.
walkseq:
        .byte 1,2,3,2
; The same for the enemy (frames 0, 1, 2, 1).
enemyseq:
        .byte 0,1,2,1

; ---------------------------------------------------------------- Start a level
; Input:    A = level number 0..3
; Effect:   sets all hero and enemy variables to the start values of this level (from lib/levels.inc),
;           clears the movement state and points the autopilot at the start of the script of this level.
;           The score (sc0..sc2) stays untouched: the main program clears it only at the start of a whole run.
; Note:     The level map (LMAP) is NOT loaded by this core but by the HAL (hal_load_level), because only it knows where the map comes from.
; Changes:  A, X, Y, flags; all variables of the hero and the enemy; calls nothing.
; Example:  level 0 starts the hero at x = 16, y = 160 (stands on the grass), looking right.
core_level_init:
        sta lvl                         ; remember which level is running (0..3)
        tax                             ; X = level number: index for all tables with one byte per level
        lda #0
        sta lvl_done
        sta hmode
        sta hatim
        sta hy_f
        sta hx_f
        sta ex_f
        sta ey_f
        sta emode
        sta eatim
        sta sfx_t1
        sta ap_cnt
        txa
        asl a                           ; level number * 2: the tables with 16-bit values (lvl_hx, lvl_ex) have two bytes per level
        tay                             ; Y = level number * 2 = index into the .word tables
        lda lvl_hx,y                    ; x start of the hero, low byte
        sta hx_l
        lda lvl_hx+1,y                  ; ... high byte (x can be above 255)
        sta hx_h
        lda lvl_ex,y
        sta ex_l
        lda lvl_ex+1,y
        sta ex_h
        lda lvl_hy,x                    ; foot y of the hero (one byte per level, therefore index X)
        sta hy
        lda lvl_hd,x
        sta hdir
        lda lvl_ey,x
        sta ey
        lda lvl_ed,x
        sta edir
        lda script_lo,x                 ; point the autopilot at the start of the script of this level (pointer low ...
        sta ap_ptr
        lda script_hi,x                 ; ... and high)
        sta ap_ptr+1
        rts

; ---------------------------------------------------------------- Read a tile
; get_tile: input X = column (0..19), Y = row (0..11).  Output A = tile number at this position (0 = empty/sky).
;           Uses X (and A, carry); Y is preserved.  The map is in LMAP (RAM), because collected coins are
;           deleted there.  Index = row20[row] + column.
get_tile:
        txa
        clc
        adc row20,y                     ; A = column + 20 * row = index into the level map
        tax
        lda LMAP,x                      ; tile number from the map in RAM
        rts
; get_attr: input as get_tile.  Output A = property of the tile: 0 = empty/decoration (you fall through),
;           1 = solid (floor, platform, wall), 2 = ladder.  Columns outside 0..19 count as wall (1), so that
;           the enemy never runs over the edge of the screen.  Uses X and Y is preserved.
get_attr:
        cpx #20                         ; column >= 20? (outside the screen)
        bcc +
        lda #1
        rts
+       jsr get_tile
        tax
        lda tile_attr,x                 ; look up the property of this tile number
        rts

; ---------------------------------------------------------------- Autopilot
; The autopilot is a tiny interpreter. The script of a level is a sequence of commands (bytes):
;
;     byte(s)            command  meaning
;     0                  END      level done (hero stands at the door); lvl_done becomes 1
;     1  d  n            WALK     d = direction (0 right, 1 left), walk for n frames (1.5 points per frame)
;     2  d  dx dy        JUMP     jump: d = direction, dx = distance in points (0..255, unsigned), dy = height difference
;                                 (signed byte -128..127, negative = upwards); always lasts 32 frames
;     3  d  n            CLIMB    d = 0 up, 1 down, climb for n frames (1 point per frame)
;     4  n               WAIT     stand still for n frames
;     5  k               SFX      play sound effect k (takes no time)
;     6  col row         COIN     collect coin at tile (col,row): delete the tile, 50 points, sound (takes no time)
;
; Limit: the duration n of WALK, CLIMB and WAIT should be 1..254.  n = 255 counts immediately as "script finished".  n = 0 is a command
; without duration: hero_update skips it and fetches the next one at once (otherwise ap_cnt would count down to 255).
;
; The scripts are generated by tools/jumprun_art.py: there every path is simulated and checked beforehand (does the hero really land
; on a solid tile after every jump?) before the bytes are written to lib/levels.inc.
;
; ap_byte: reads the next script byte into A and advances ap_ptr by one (16-bit pointer; on overflow of the
;          low byte the high byte is incremented).  Uses Y (= 0); flags after the pointer increment.
ap_byte:                     ; fetch the next script byte
        ldy #0
        lda (ap_ptr),y                  ; script byte (indirect via the pointer, Y = 0)
        inc ap_ptr                      ; pointer low + 1; on overflow (zero) increment the high byte
        bne +
        inc ap_ptr+1
+       rts

; ap_fetch: fetches and starts the next command when the previous one has expired (ap_cnt = 0).
;   Timeless commands (SFX, COIN) it executes at once and fetches the next one right away (jump to .again),
;   until a command comes that needs time (WALK, JUMP, CLIMB, WAIT) or the script ends (END).
;   Afterwards ap_cnt holds how many frames the new command lasts, and hmode says what the hero is doing.
;   Changes A, X, Y, Flags, t0/t1 and the variables of the hero; calls ap_byte, ap_jump, do_coin, hal_sfx.
;   The commands are tested in numerical order (cmp #1, #2, ...); if none matches, only COIN is left.
ap_fetch:
.again: jsr ap_byte
        cmp #0
        bne +
        ; END: level done
        lda #1
        sta lvl_done
        lda #255                        ; 255 = "forever": hero_update does nothing more until the level changes
        sta ap_cnt
        lda #HM_STAND
        sta hmode
        rts
+       cmp #1
        bne +
        ; WALK d n
        jsr ap_byte
        sta hdir
        jsr ap_byte
        sta ap_cnt
        lda #HM_WALK
        sta hmode
        rts
+       cmp #2
        bne +
        jmp ap_jump
+       cmp #3
        bne +
        ; CLIMB d n
        jsr ap_byte
        sta hclimb
        jsr ap_byte
        sta ap_cnt
        lda #HM_CLIMB
        sta hmode
        rts
+       cmp #4
        bne +
        ; WAIT n
        jsr ap_byte
        sta ap_cnt
        lda #HM_STAND
        sta hmode
        rts
+       cmp #5
        bne +
        ; SFX k
        jsr ap_byte
        jsr hal_sfx
        jmp .again
+       ; COIN col row: column to t0, row to t1, do_coin does the rest
        jsr ap_byte
        sta t0
        jsr ap_byte
        sta t1
        jsr do_coin
        jmp .again

; ap_jump: starts a jump.  Parameters from the script: direction d, distance dx (points, unsigned), height dy (signed).
;   The jump lasts exactly 32 frames.  So that the hero ends up EXACTLY dx points further and dy points higher/lower,
;   the speed per frame is chosen like this:
;       vx = dx * 256 / 32 = dx * 8     (in 8.8 fixed point, i.e. dx * 8 / 256 points per frame)
;       vy = dy * 256 / 32 = dy * 8
;   Example: dx = 70, dy = -32:  vx = 560 = $0230  (2.19 points per frame),  vy = -256 = $FF00  (-1 point per frame).
;   Multiplying by 8 is done by shifting the 16-bit number left three times (asl/rol).  When facing left, vx
;   is negated (two's complement: 0 - vx).  For negative dy, vy gets $FF as the high byte (sign extension).
;   The vertical base line hy moves linearly from the start to the target height; hero_update lays the sine arc over it.
;   Value range: dx 0..255 gives |vx| up to 2040 = $07F8; dy -128..127 gives vy from -1024 to 1016.  Both fit in 16 bits.
;   Changes A, t0, t1, hdir, hvx, hvy, jt, hy_f, ap_cnt, hmode; on return (rts) the jump begins with the next hero_update.
ap_jump:
        jsr ap_byte
        sta hdir
        jsr ap_byte
        sta t0                   ; dx
        jsr ap_byte
        sta t1                   ; dy (signed)
        ; vx = dx * 8
        lda t0
        sta hvx_l
        lda #0
        sta hvx_h
        asl hvx_l                       ; vx * 2 (16 bit: asl shifts into the carry, rol brings it into the high byte)
        rol hvx_h
        asl hvx_l
        rol hvx_h
        asl hvx_l
        rol hvx_h
        lda hdir                        ; facing left? Then negate the speed
        beq +
        sec                      ; left: negate
        lda #0
        sbc hvx_l                       ; 0 - vx = two's complement (subtraction with carry, hence sec first)
        sta hvx_l
        lda #0
        sbc hvx_h
        sta hvx_h
+       ; vy = dy * 8 with sign extension
        lda t1
        sta hvy_l
        lda #0
        bit t1                          ; dy negative? (bit 7 of t1 sets the N flag; A stays 0)
        bpl +
        lda #$ff
+       sta hvy_h
        asl hvy_l
        rol hvy_h
        asl hvy_l
        rol hvy_h
        asl hvy_l
        rol hvy_h
        lda #0
        sta jt
        sta hy_f
        lda #32                         ; a jump lasts exactly 32 frames
        sta ap_cnt
        lda #HM_JUMP
        sta hmode
        rts

; do_coin: collects a coin.  Input t0 = column, t1 = row of the coin tile.  Changes A, X, Y, flags.
;   1. Set the tile in the level map (LMAP) to 0 (empty) so that it never shows up again,
;   2. delete the same tile on the screen (hal_put_tile),
;   3. add 50 points - in decimal mode (sed), so that the score can be read directly as decimal digits
;      (BCD: $50 + $50 = $00 with carry, the high byte gets +1),
;   4. redraw the score and start the coin sound (effect 4) (the call of hal_sfx is a jump, not a subroutine
;      call: its rts returns right to the caller of do_coin).
;   Limit: the score is a three-byte BCD number (000000..999999); an overflow beyond 999999 is not caught (unreachable with 50 points per
;   coin and a few coins per level).  The NMOS CPUs (6510, Sally, 7501) do not clear the D flag on an interrupt:
;   if an IRQ falls into the short sed ... cld, its routine (music, adc) would run in decimal mode.  So do_coin disables the IRQs there
;   (php/sei ... plp).  On the Atari (.const NO_SEI_BCD) this is dropped: there DLI and VBI run as NMI, which sei does not block; the
;   interrupt routines are in the operating system or in the RMT player (that the OS clears the D flag in the NMI is not documented here).
do_coin:
        ldx t0
        ldy t1
        txa
        clc
        adc row20,y
        tax
        lda #0
        sta LMAP,x
        ldx t0
        ldy t1
        lda #0
        jsr hal_put_tile
.ifndef NO_SEI_BCD
        php                             ; remember the I flag (the main loop runs with interrupts enabled) ...
        sei                             ; ... and disable IRQs during decimal mode
.endif
        sed                             ; decimal mode: adc/sbc now calculate with BCD digits
        clc
        lda sc0
        adc #$50                        ; +50 points (BCD $50)
        sta sc0
        lda sc1
        adc #0
        sta sc1
        lda sc2
        adc #0
        sta sc2
        cld                             ; decimal mode off again (otherwise all following additions are wrong!)
.ifndef NO_SEI_BCD
        plp                             ; I flag as before (plp also restores D = 0)
.endif
        jsr hal_score_draw
        lda #4
        jmp hal_sfx

; ---------------------------------------------------------------- Hero
; hero_update: called once per frame.  The flow:
;   1. If the running command has expired (ap_cnt = 0), fetch the next one (ap_fetch).
;   2. Depending on hmode, perform one step (walk / climb / jump / stand).
;   3. Count ap_cnt down; at 0 the next frame fetches the next command.
;   The value 255 in ap_cnt means "level finished": the hero stands still until the main program starts the next level.
;
;   Walking:  x changes by 1.5 points per frame (hero_walk), a step sound every 12 frames (effect 2).
;   Climbing: y changes by 1 point per frame, a climbing sound every 10 frames (effect 3).
;   Jumping:  x and y run with the speeds computed in ap_jump; jt counts 0..32.  The DISPLAY height is
;             hy minus sinetab[jt] (see core_prepare_sprites), the base line hy itself moves in a straight line.
;   Input: none (state in the variables).  Changes all registers and flags, t0/t1, the hero variables; calls ap_fetch, hero_walk, hal_sfx.
hero_update:
        lda ap_cnt
        bne +
.fetch: jsr ap_fetch
        lda ap_cnt
        beq .fetch                      ; command without duration (n = 0): skip it and fetch the next one
+       lda ap_cnt
        cmp #255                        ; level finished? Then do nothing more
        beq .done                       ; end of the level: stay standing
        lda hmode
        cmp #HM_WALK
        beq .walk
        cmp #HM_JUMP
        beq .jump
        cmp #HM_CLIMB
        beq .climb
        dec ap_cnt                      ; stand / wait
        jmp .done
.walk:  jsr hero_walk
        dec ap_cnt
        inc hatim
        ; step sound every 12 frames
        inc sfx_t1                      ; counter for the step sound
        lda sfx_t1
        cmp #12                         ; every 12 frames a step click (effect 2), but only if no other effect is sounding
        bcc .done
        lda #0
        sta sfx_t1
        lda sfx_on
        bne .done
        lda #2
        jsr hal_sfx
        jmp .done
.climb: lda hclimb
        bne +
        dec hy                          ; up (y grows downwards)
        jmp ++
+       inc hy                          ; down
+      dec ap_cnt
        inc hatim                       ; advance the climbing animation
        inc sfx_t1
        lda sfx_t1
        cmp #10                         ; every 10 frames a climbing blip (effect 3), if no other effect is sounding
        bcc .done
        lda #0
        sta sfx_t1
        lda sfx_on
        bne .done
        lda #3
        jsr hal_sfx
        jmp .done
.jump:  clc                             ; x (24 bit hx_h:hx_l:hx_f) += vx (16 bit signed): first fraction and whole points ...
        lda hx_f
        adc hvx_l
        sta hx_f
        lda hx_l
        adc hvx_h
        sta hx_l
        lda hvx_h
        bmi +                           ; ... then the high byte: vx negative -> add $FF, otherwise 0 (sign extension to 24 bit)
        lda #0
        beq ++                          ; always taken (A = 0); the carry from the adc above is preserved
+       lda #$ff                        ; negative speed: high byte = $FF
+      adc hx_h                         ; plus carry from the addition of hx_l
        sta hx_h
        clc                             ; y (16 bit hy:hy_f) += vy
        lda hy_f
        adc hvy_l
        sta hy_f
        lda hy
        adc hvy_h                       ; base line += vy; two's complement addition also works for negative vy
        sta hy
        inc jt                          ; next step in the jump (0..32)
        dec ap_cnt
        bne .done
        lda #HM_STAND
        sta hmode
.done:  rts


; hero_walk: one walking step of 1.5 points: adds (left: subtracts) $0180 to the 24-bit position hx_h:hx_l:hx_f.
;   Input hdir (0 right, 1 left).  Changes A, flags and the position.  No edge test: the script makes sure the hero stays in the level.
;   $0180 = 1.5 in 8.8.  The fraction ($80 = 0.5) overflows with every second step and so gives alternately
;   1 and 2 whole points: 1.5 on average.  The carries are passed on with adc/sbc without an intermediate clc/sec.
hero_walk:
        lda hdir
        bne .l
        clc
        lda hx_f
        adc #$80                        ; +0.5 point in the fraction (carry possible) ...
        sta hx_f
        lda hx_l
        adc #1                          ; ... + 1 whole point + possible carry = 1.5 points per frame
        sta hx_l
        lda hx_h
        adc #0
        sta hx_h
        rts
.l:     sec
        lda hx_f
        sbc #$80                        ; to the left: -0.5 in the fraction ...
        sta hx_f
        lda hx_l
        sbc #1                          ; ... and -1 whole point
        sta hx_l
        lda hx_h
        sbc #0
        sta hx_h
        rts

; ---------------------------------------------------------------- Enemy "Gumbo": walks, climbs ladders, NEVER jumps
;
; Gumbo has his OWN physics, which differs from Hugo's (that is the joke of the demo):
;   * he only walks on solid tiles and stops at every edge (he never runs into the void),
;   * he stops before walls (solid tiles at body height) - he cannot jump over obstacles,
;   * he climbs ladders, but only if a ladder is within his reach,
;   * he is slower than Hugo (0.6 instead of 1.5 points per frame when walking, 0.5 when climbing).
; The state machine (emode):  0 = walk/decide,  1 = climb,  2 = wait ("grumble") after a failure.
;
; enemy_step: tries ONE walking step in direction edir.  Output: carry = 1 on success, 0 if blocked.
;   Check at the "front point" = middle of the character +/- 9 points (depending on direction):
;     1. column = front point / 16 (shift right four times; outside 0..19 counts as blocked),
;     2. floor: the tile in the row of the feet (ey / 16) must be solid (property 1),
;     3. wall: the tile one row higher ((ey-1) / 16) must NOT be solid.
;   Then change the position by 154/256 = 0.6 points.
;   Changes A, X, Y, t0, t1, t2 and (on success) ex_h:ex_l:ex_f; calls get_attr.
enemy_step:                     ; C = 1 if the step succeeded
        clc
        lda ex_l
        adc #12                         ; middle of the character = left edge + 12
        sta t0
        lda ex_h
        adc #0
        sta t1
        lda edir
        bne .left
        clc
        lda t0
        adc #9                          ; front point right: middle + 9 points
        sta t0
        lda t1
        adc #0
        sta t1
        jmp .col
.left:  sec
        lda t0
        sbc #9                          ; front point left: middle - 9 points
        sta t0
        lda t1
        sbc #0
        sta t1
        bmi .blocked                    ; negative (left of the screen edge): blocked
.col:   lda t1                          ; column = x / 16: shift the 16-bit value t1:t0 right four times, the result is in t0
        lsr a
        ror t0
        lsr a
        ror t0
        lsr a
        ror t0
        lsr a
        ror t0
        lda t0
        cmp #20
        bcs .blocked
        sta t2                   ; remember the column
        tax
        lda ey                   ; floor under the feet: row ey/16
        lsr a
        lsr a
        lsr a
        lsr a
        tay
        jsr get_attr
        cmp #1
        bne .blocked
        ldx t2
        lda ey                   ; wall in the way: row (ey-1)/16
        sec
        sbc #1
        lsr a
        lsr a
        lsr a
        lsr a
        tay
        jsr get_attr
        cmp #1
        beq .blocked
        ; step: 154/256 points per frame
        lda edir
        bne .sl
        clc
        lda ex_f
        adc #154                        ; 154 / 256 = 0.6 points per frame (the enemy is slower than the hero)
        sta ex_f
        lda ex_l
        adc #0
        sta ex_l
        lda ex_h
        adc #0
        sta ex_h
        sec
        rts
.sl:    sec
        lda ex_f
        sbc #154
        sta ex_f
        lda ex_l
        sbc #0
        sta ex_l
        lda ex_h
        sbc #0
        sta ex_h
        sec
        rts
.blocked:
        clc
        rts

; chk_ladder: is there a suitable ladder in column X?  Input X = column, want = 0 (up) or 1 (down).
;   For up, the tile above the feet (row (ey-1)/16) must be a ladder, for down the tile at foot height (ey/16).
;   Output: carry = 1 if yes.  X stays the column (find_ladder needs it afterwards).  Changes A, Y, t2; calls get_attr.
chk_ladder:
        stx t2
        lda want
        bne .down
        lda ey
        sec
        sbc #1
        jmp .row
.down:  lda ey
.row:   lsr a
        lsr a
        lsr a
        lsr a
        tay
        ldx t2
        jsr get_attr
        ldx t2
        cmp #2
        beq +
        clc
        rts
+       sec
        rts

; find_ladder: looks for the nearest suitable ladder, from the enemy to the left AND right, always further away (distance 0..19).
;   Middle column = (ex + 12) / 16.  On success: carry = 1, ecol = ladder column and tx = target position for the left edge
;   of the character: column * 16 - 4, so that the character's middle (+12) lies on the ladder's middle (+8).  At equal distance LEFT is
;   checked first.  Failure: carry = 0 (ecol and tx then stay unchanged).  Changes A, X, Y, t0, t1, t3; calls chk_ladder.
find_ladder:
        clc
        lda ex_l
        adc #12
        sta t0
        lda ex_h
        adc #0
        lsr a
        ror t0
        lsr a
        ror t0
        lsr a
        ror t0
        lsr a
        ror t0
        lda t0
        sta t3                   ; middle column
        ldx #0                   ; distance
.dist:  stx t1
        lda t3
        sec
        sbc t1                          ; column to the left of the enemy (middle column - distance)
        bcc +                    ; outside on the left
        tax
        jsr chk_ladder
        bcs .found
+       lda t1
        beq +                           ; distance 0: the middle column itself has already been checked
        clc
        adc t3                          ; column to the right of the enemy (middle column + distance)
        cmp #20
        bcs +
        tax
        jsr chk_ladder
        bcs .found
+       ldx t1
        inx
        cpx #20
        bcc .dist
        clc
        rts
.found: stx ecol
        txa
        asl a
        asl a
        asl a
        asl a                           ; column * 16 (low byte) ...
        sta tx_l
        lda ecol
        lsr a
        lsr a
        lsr a
        lsr a                           ; ... and the high byte of column * 16 (= column / 16)
        sta tx_h
        sec
        lda tx_l
        sbc #4                          ; minus 4: the character's middle (x + 12) should fall on the ladder's middle (column * 16 + 8)
        sta tx_l
        lda tx_h
        sbc #0
        sta tx_h
        sec
        rts


; enemy_update: once per frame.  Decision of the enemy:
;   emode 2 (wait):    count ewait down, then decide again - the character stands and "wobbles" (animation runs).
;   emode 1 (climb):   0.5 point per frame (the fraction in ey_f overflows every second frame), at the end of the ladder
;                      (next tile in the climbing direction is no longer a ladder) back to walking mode.
;   emode 0 (walk):    a) Is the hero at about the same height (difference < 8 points in y)? Then towards him, up to
;                         24 points distance.  If the way is blocked (edge, wall), Gumbo waits 25 frames.
;                      b) Otherwise look for a ladder (find_ladder).  If there is none, he waits 40 frames (he cannot
;                         jump!).  If there is one, he walks to it; when he stands less than 2 points
;                         in front of it, he steps onto the ladder (x exactly on the ladder's middle) and climbs.
;   Changes all registers and flags, t0..t3 and the enemy variables; calls enemy_step, find_ladder, get_attr, hal_sfx.
;   eatim controls the animation and only counts in frames in which the enemy moves or waits (emode 2); when he stops
;   next to the hero (.idle) and in frames in which he only decides (ladder chosen, stepping on), the animation also stands still.
enemy_update:
        lda emode
        beq .walk
        cmp #1
        beq .climb
        ; wait (frustrated): briefly on the spot, then decide again
        dec ewait
        bne +
        lda #0
        sta emode                       ; waiting time over: walk/decide again
+       inc eatim
        rts
.climb: inc eatim
        clc
        lda ey_f
        adc #$80                        ; 0.5 points per frame
        sta ey_f
        bcc .cdone                      ; carry only every second frame: only then does ey move by a whole point
        lda eclimb
        bne .down
        dec ey
        lda ey
        sec
        sbc #1
        lsr a
        lsr a
        lsr a
        lsr a
        tay
        ldx ecol
        jsr get_attr
        cmp #2
        beq .cdone
        lda #0
        sta emode                       ; reached the top (no more ladder): back to walking mode (without climbing sound)
        rts
.down:  inc ey
        lda ey
        lsr a
        lsr a
        lsr a
        lsr a
        tay
        ldx ecol
        jsr get_attr
        cmp #2
        beq .cdone
        lda #0
        sta emode
.cdone: ; climbing sound: every 20 frames effect 5, if no other effect is sounding (Hugo and Gumbo share sfx_t1)
        inc sfx_t1
        lda sfx_t1
        cmp #20
        bcc +
        lda #0
        sta sfx_t1
        lda sfx_on
        bne +
        lda #5
        jsr hal_sfx
+       rts
.walk:  ; same level?  Absolute value of hy - ey less than 8 points (during a jump hy is the base line, not the display height)
        lda hy
        sec
        sbc ey
        bcs +
        eor #$ff                        ; absolute value of the height difference: negate a negative number (two's complement: invert, +1)
        adc #1
+       cmp #8                          ; difference less than 8 points: same level
        bcs .other
        ; same level: towards the hero, up to 24 points
        lda hx_l
        sec
        sbc ex_l
        sta t0
        lda hx_h
        sbc ex_h
        sta t1
        bmi .toleft
        lda #0
        sta edir                        ; hero to the right of Gumbo: to the right
        lda t1
        bne .go
        lda t0
        cmp #24                         ; closer than 24 points: Gumbo stands still (it is "his turn")
        bcc .idle
        jmp .go
.toleft:
        lda #1
        sta edir                        ; hero to the left of Gumbo: to the left
        sec                             ; distance = absolute value of (t1:t0): 16-bit two's complement
        lda #0
        sbc t0
        sta t0
        lda #0
        sbc t1
        sta t1
        bne .go
        lda t0
        cmp #24
        bcc .idle
.go:    jsr enemy_step
        bcs +
        lda #25                         ; blocked (edge or wall): wait 25 frames and grumble
        sta ewait
        lda #2
        sta emode
        rts
+       inc eatim
        rts
.idle:  rts
.other: ; other level: look for a ladder
        lda hy
        cmp ey
        bcc +
        lda #1                          ; hero lower (hy >= ey, y grows downwards) -> ladder down
        jmp ++
+       lda #0                          ; hero higher -> ladder up
+      sta want
        jsr find_ladder
        bcs +
        lda #40                  ; no ladder: stand and grumble
        sta ewait
        lda #2
        sta emode
        rts
+       lda ex_l                        ; distance to the ladder: ex - tx (16 bit signed)
        sec
        sbc tx_l
        sta t0
        lda ex_h
        sbc tx_h
        sta t1
        bmi .tright
        lda t1
        bne .tl
        lda t0
        cmp #2
        bcc .atladder
.tl:    lda #1
        sta edir
        jmp .go
.tright:
        lda t1
        cmp #$ff
        bne .tr
        lda t0
        cmp #$fe
        bcs .atladder
.tr:    lda #0
        sta edir
        jmp .go
.atladder:                              ; distance to the ladder less than 2 points:
        lda tx_l                        ; step onto the ladder (x exactly on the ladder's middle, fraction 0)
        sta ex_l
        lda tx_h
        sta ex_h
        lda #0
        sta ex_f
        lda want
        sta eclimb
        lda #1
        sta emode
        rts

; ---------------------------------------------------------------- Prepare output for the HAL
; core_prepare_sprites: converts the game state into the values the HAL needs for drawing
;   h_px_l/h (x), h_py (y), h_fr (frame number) for the hero and e_px_l/h, e_py, e_fr for the enemy.
;   * During the jump the display height is computed: y = base line hy - sinetab[jt].
;   * The frame number depends on the state: hero stand 0, walk 1/2/3/2 (advances every 4 frames), jump 4,
;     climb 5/6 (switches every 8 frames); when facing left, 7 is added (second half of the sprite set =
;     mirrored images).  Enemy: walk 0/1/2/1 (advances every 8 frames), climb 0/2 (switches every 8 frames); when facing
;     left + 3.
;   Input: the game state.  Output: h_px_l/h/h_py/h_fr and e_px_l/h/e_py/e_fr (see above, "Output for the HAL").
;   Changes A, X, flags.  Is also called by the title screen of the main program after it has set positions and state by hand.
core_prepare_sprites:
        lda hx_l
        sta h_px_l
        lda hx_h
        sta h_px_h
        lda hy
        ldx hmode
        cpx #HM_JUMP
        bne +
        ldx jt
        sec
        sbc sinetab,x                   ; display height = base line - sine value (y grows downwards, so the character rises)
+       sta h_py
        ; hero frame: stand 0, walk (1,2,3,2), jump 4, climb 5/6; plus 7 when facing left
        lda hmode
        bne +
        lda #0
        jmp .hdir
+       cmp #HM_WALK
        bne +
        lda hatim
        lsr a
        lsr a
        and #3
        tax
        lda walkseq,x                   ; walk phase -> frame number (1, 2, 3, 2)
        jmp .hdir
+       cmp #HM_JUMP
        bne +
        lda #4
        jmp .hdir
+       lda hatim                ; climb
        lsr a
        lsr a
        lsr a
        and #1
        clc
        adc #5
.hdir:  ldx hdir
        beq +
        clc
        adc #7
+       sta h_fr
        ; enemy
        lda ex_l
        sta e_px_l
        lda ex_h
        sta e_px_h
        lda ey
        sta e_py
        lda emode
        cmp #1
        bne +
        lda eatim                       ; climb: frames 0 and 2
        lsr a
        lsr a
        lsr a
        and #1
        asl a
        jmp .edir
+       lda eatim
        lsr a
        lsr a
        lsr a
        and #3
        tax
        lda enemyseq,x                  ; enemy phase -> frame number (0, 1, 2, 1)
.edir:  ldx edir
        beq +
        clc
        adc #3
+       sta e_fr
        rts
