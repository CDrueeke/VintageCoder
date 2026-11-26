// SPDX-License-Identifier: MIT-0
// C128: VDC hires prototype (feasibility study for the PRALLBALL extension).
//
// Purpose: show and MEASURE whether the VDC in 640 x 200 bitmap mode is fast enough for a breakout game.  Not a game, no sound, no input.
//   - VDC bitmap 640 x 200: 80 bytes per pixel line ($0000..$3E7F), color attributes per 8 x 8 pixels from $4000 (2000 bytes).
//   - Background image in VDC RAM from $5000 (only with a 64 KB VDC: register 28 bit 4); the balls are restored from the background with the VDC block copy (register 24 bit 7,
//     30, 32, 33) and then redrawn.  Drawing is done in 8 fine positions (pixel offset 0..7 in the byte).
//   - NB balls (16 x 8 pixel wide drawing area, ball 8 x 8) bounce in the free field below the brick wall.
//   - The VIC-II stays dark and only supplies the raster line as a time measure; the processor runs at 2 MHz.
// Result after FRAMES frames in main memory (readable with the monitor):
//   res_done ($C200) = $FF finished, res_vdc64 ($C201) = 1 if 64 KB VDC detected, res_max ($C202/$C203) = largest number of raster lines used per frame
//   (312 lines per frame on PAL), res_sum ($C204..$C206) = sum over all frames, res_frames ($C207/$C208) = counted frames.
// Without a 64 KB VDC the prototype shows only the bricks without balls (the background would need the space) and reports res_vdc64 = 0.

.const VA = $d600                       // VDC: register select / status
.const VD = $d601                       // VDC: data
.const NB = 8                           // number of balls
.const FRAMES = 200                     // frames that are measured
.const BG = $5000                       // background image in VDC RAM (only with 64 KB)
.const ATTR = $4000                     // attributes in VDC RAM

.const res_done   = $c200
.const res_vdc64  = $c201
.const res_max    = $c202
.const res_sum    = $c204
.const res_frames = $c207
.const res_us     = $c210                // microseconds of the last frame (32 bit)
.const res_usmax  = $c214                // largest value (32 bit)

// Zero page (bank 0, free without operating system)
.const zp   = $fb                       // pointer/helpers
.const t0   = $02
.const t1   = $03
.const t2   = $04
.const t3   = $05
.const bi   = $06                       // ball number
.const sh   = $07                       // fine position
.const byt  = $08                       // byte column
.const lin  = $09                       // pixel line
.const fcl  = $0a                       // frame counter low
.const fch  = $0b
.const r0l  = $0c                       // raster line at the start of the frame
.const r0h  = $0d

.org $1c01
.basicstub start

start:  sei
        lda #$3e                        // all RAM of bank 0 plus I/O
        sta $ff00
        lda #$7f
        sta $dc0d                       // CIA interrupts off
        sta $dd0d
        lda $dc0d
        lda $dd0d
        lda #<rti_only
        sta $fffa
        sta $fffe
        lda #>rti_only
        sta $fffb
        sta $ffff
        lda #$0b
        sta $d011                       // VIC-II: display off (only the raster counter runs)
        lda #0
        sta $d015
        lda $d030
        ora #1
        sta $d030                       // 2 MHz
        jsr vdc_detect
        jsr vdc_setup
        jsr draw_wall
        lda res_vdc64
        beq nobg                        // without 64 KB no background image and no balls
        jsr copy_bg
        jsr balls_init
        jsr run
nobg:   lda #$ff
        sta res_done
        jmp *
rti_only: rti

// ---- VDC access
wr:     stx VA                          // X = register, A = value
.w:     bit VA
        bpl .w
        sta VD
        rts
rd:     stx VA                          // X = register -> A
.r:     bit VA
        bpl .r
        lda VD
        rts
// vdc_at: A = address high, Y = address low -> registers 18/19
vdc_at: ldx #18
        jsr wr
        inx
        tya
        jmp wr
put:    ldx #31                         // A = byte to the current VDC address (counts up)
        jmp wr

// Detect a 64 KB VDC: set register 28 bit 4, write different values to $0000 and $8000; with only 16 KB $8000 mirrors to $0000.
vdc_detect:
        ldx #28
        jsr rd
        ora #$10
        ldx #28
        jsr wr
        lda #0
        ldy #0
        jsr vdc_at
        lda #$11
        jsr put
        lda #$80
        ldy #0
        jsr vdc_at
        lda #$22
        jsr put
        lda #0
        ldy #0
        jsr vdc_at
        ldx #31
        jsr rd
        ldy #0
        cmp #$11
        bne +                           // $0000 overwritten: mirroring, only 16 KB
        iny
+       sty res_vdc64
        rts

// vdc_setup: bitmap mode 640 x 200 with attributes, clear the image, attributes white on black
vdc_setup:
        lda #0
        ldx #12
        jsr wr                          // image start $0000
        ldx #13
        jsr wr
        lda #>ATTR
        ldx #20
        jsr wr                          // attribute start $4000
        lda #<ATTR
        ldx #21
        jsr wr
        lda #$00
        ldx #26
        jsr wr                          // background black (low nibble)
        lda #$c7
        ldx #25
        jsr wr                          // bit 7 bitmap, bit 6 attributes, shift 7
        lda #$20
        ldx #10
        jsr wr                          // cursor off
        // Fill the image ($0000..$3E7F = 16000 bytes) with 0: 64 x 250 bytes by block fill
        lda #0
        tay
        jsr vdc_at
        lda #64
        sta t0
        lda #0
        jsr fill_run
        // Attributes ($4000..$47CF = 2000 bytes) white (15): 8 x 250
        lda #>ATTR
        ldy #<ATTR
        jsr vdc_at
        lda #8
        sta t0
        lda #$0f
        jmp fill_run
// fill_run: fill t0 times 250 bytes with the value A (from the current VDC address); register 24 bit 7 = 0 (fill)
fill_run:
        sta t1
        lda #$20
        ldx #24
        jsr wr
.f1:    lda t1
        jsr put                         // write 1 byte, then copy 249 bytes = 250 bytes
        lda #249
        ldx #30
        jsr wr
        dec t0
        bne .f1
        rts

// draw_wall: border and bricks.  Border at the top (lines 0..1) and left/right (byte 0 and 79) complete, 6 rows with 26 bricks of 3 bytes each (24 pixels),
// each row in one cell line (lines (r+2)*8 .. +5), color per brick.
draw_wall:
        // Border at the top: 2 lines x 80 bytes
        lda #0
        tay
        jsr vdc_at
        lda #$ff
        jsr put
        lda #79
        ldx #30
        jsr wr
        lda #0
        ldy #80
        jsr vdc_at
        lda #$ff
        jsr put
        lda #79
        ldx #30
        jsr wr
        // Side wall: byte 0 and byte 79 of every line 2..199 = $ff (two addresses per line, singly)
        lda #2
        sta lin
.sw:    lda lin
        jsr line_addr
        jsr vdc_at
        lda #$ff
        jsr put
        lda lin
        jsr line_addr
        sta t2
        tya
        clc
        adc #79
        tay
        lda t2
        adc #0
        jsr vdc_at
        lda #$ff
        jsr put
        inc lin
        lda lin
        cmp #200
        bne .sw
        // Bricks
        lda #0
        sta bi                          // row 0..5
.rw:    lda bi
        clc
        adc #2
        asl
        asl
        asl
        sta lin                         // first pixel line of the row
        lda #6
        sta t3                          // 6 filled lines
.ln:    lda lin
        jsr line_addr
        sta t2
        tya
        clc
        adc #1                          // byte 1 (border byte 0 to the left)
        tay
        lda t2
        adc #0
        jsr vdc_at
        lda #26
        sta t1                          // 26 bricks
.br:    lda #$ff
        jsr put
        lda #$ff
        jsr put
        lda #$fe                        // last column leaves a 1 pixel gap
        jsr put
        dec t1
        bne .br
        inc lin
        dec t3
        bne .ln
        // Attributes of this row: cell line (bi+2): $4000 + (bi+2)*80 + 1 .. 78; color per brick = ((brick + row) mod 14) + 1
        lda bi
        clc
        adc #2
        jsr cell_addr                   // A = high, Y = low
        sta t2
        tya
        clc
        adc #1
        tay
        lda t2
        adc #0
        jsr vdc_at
        lda #0
        sta t1                          // brick number
.ac:    lda t1
        clc
        adc bi
.m14:   cmp #14
        bcc +
        sbc #14
        jmp .m14
+       clc
        adc #1                          // color 1..14
        jsr put
        jsr put
        jsr put                         // three cells per brick
        inc t1
        lda t1
        cmp #26
        bne .ac
        inc bi
        lda bi
        cmp #6
        beq .rwe
        jmp .rw
.rwe:   rts

// line_addr: pixel line A -> VDC address (A * 80) in A (high) / Y (low).  Lines 0..199.
line_addr:
        sta zp + 2
        lda #0
        sta zp + 3                          // zp + 3:zp + 2 = line
        // * 80 = * 64 + * 16
        lda zp + 2
        sta zp
        lda zp + 3
        sta zp + 1                      // zp = line
        asl zp
        rol zp + 1
        asl zp
        rol zp + 1
        asl zp
        rol zp + 1
        asl zp
        rol zp + 1                      // * 16
        lda zp
        sta zp + 2
        lda zp + 1
        sta zp + 3                          // zp + 3:zp + 2 = * 16
        asl zp
        rol zp + 1
        asl zp
        rol zp + 1                      // * 64
        lda zp
        clc
        adc zp + 2
        tay
        lda zp + 1
        adc zp + 3
        rts
// cell_addr: cell line A (0..24) -> ATTR + A * 80 in A / Y
cell_addr:
        jsr line_addr                   // A * 80 (line count 0..24 is enough)
        clc
        adc #>ATTR
        rts

// copy_bg: copy display $0000..$3E7F to BG (block copy, 64 x 250 bytes)
copy_bg:
        lda #$a0
        ldx #24
        jsr wr                          // block copy
        lda #>BG
        ldy #<BG
        jsr vdc_at                      // destination
        lda #0
        ldx #32
        jsr wr                          // source $0000
        ldx #33
        jsr wr
        lda #64
        sta t0
.cb:    lda #250
        ldx #30
        jsr wr                          // copy 250 bytes (source and destination keep counting)
        dec t0
        bne .cb
        lda #$20
        ldx #24
        jmp wr

// ---- Balls: state (NB pieces) and movement
balls_init:
        ldx #NB - 1
.bi:    txa
        asl
        asl
        asl
        asl
        sta bxl,x                       // X start 16 px apart
        txa
        lsr
        lsr
        lsr
        lsr
        sta bxh,x
        lda #90
        sta byy,x
        txa
        and #3
        clc
        adc #1
        sta bdx,x                       // dx 1..4
        txa
        and #1
        beq +
        lda #$ff                        // dx negative for every second ball
        sta bdh,x
        lda bdx,x
        eor #$ff
        clc
        adc #1
        sta bdx,x
        jmp .pp1
+       lda #0
        sta bdh,x

.pp1:  txa
        and #1
        clc
        adc #1
        sta bdy,x                       // dy 1..2
        lda #1
        sta bsh,x                       // sign of dy: 1 = downwards
        dex
        bmi .bie
        jmp .bi
.bie:
        // Shifted ball images: 8 fine positions x 8 lines x 2 bytes (high, low), from the 8 pixel pattern
        ldy #0
        sty sh
.sf:    ldx #0
.sr:    lda ballpat,x
        sta t0
        lda #0
        sta t1                          // 16-bit value t0:t1, shifted right by sh
        ldy sh
        beq .sd
.sx:    lsr t0
        ror t1
        dey
        bne .sx
.sd:    lda sh
        asl
        asl
        asl
        asl                             // sh * 16
        sta t2
        txa
        asl                             // line * 2
        clc
        adc t2
        tay
        lda t0
        sta ballimg,y
        lda t1
        sta ballimg + 1,y
        inx
        cpx #8
        bne .sr
        inc sh
        lda sh
        cmp #8
        bne .sf
        // draw the first image: balls at their start positions
        rts

// run: FRAMES frames: wait for line 251, move all balls (restore the old position from the background, draw the new one), measure raster lines.
run:    lda #0
        sta fcl
        sta fch
        sta res_max
        sta res_max + 1
        sta res_sum
        sta res_sum + 1
        sta res_sum + 2
        ldx #7
        lda #0
.cl:    sta res_us,x
        dex
        bpl .cl
.fr:    lda $d012
        cmp #251
        bne .fr
.fr2:   lda $d012                       // wait out of line 251 (otherwise a frame would be counted twice)
        cmp #251
        beq .fr2
        // Start of measurement: line 252 +- 1; reference cleanly via $d012/$d011 bit 7
        lda $d012
        sta r0l
        lda $d011
        and #$80
        sta r0h
        lda #$ff
        sta $dc04                       // CIA1 timer A: start value $FFFF, runs freely (clock phi2)
        sta $dc05
        sta $dc06                       // timer B: counts underflows of A
        sta $dc07
        lda #$41
        sta $dc0f                       // B: start, counts A underflows
        lda #$01
        sta $dc0e                       // A: start
        lda #0
        sta bi
.bl:    ldx bi
        jsr ball_erase
        ldx bi
        jsr ball_move
        ldx bi
        jsr ball_draw
        inc bi
        lda bi
        cmp #NB
        bne .bl
        lda #0
        sta $dc0e                       // stop the timers
        sta $dc0f
        // Microseconds = $FFFFFFFF - (B:A) ; as 32 bit in res_us (little endian)
        sec
        lda #$ff
        sbc $dc04
        sta res_us
        lda #$ff
        sbc $dc05
        sta res_us + 1
        lda #$ff
        sbc $dc06
        sta res_us + 2
        lda #$ff
        sbc $dc07
        sta res_us + 3
        lda res_us + 3
        cmp res_usmax + 3
        bcc .um
        bne .us
        lda res_us + 2
        cmp res_usmax + 2
        bcc .um
        bne .us
        lda res_us + 1
        cmp res_usmax + 1
        bcc .um
        bne .us
        lda res_us
        cmp res_usmax
        bcc .um
.us:    ldx #3
.uc:    lda res_us,x
        sta res_usmax,x
        dex
        bpl .uc
.um:
        // used lines = (end - start) mod 312
        lda $d012
        sta t0
        lda $d011
        and #$80
        beq +
        lda #1
        bne .pp2
+       lda #0

.pp2:  sta t1                          // t1:t0 = end
        lda r0h
        beq +
        lda #1
        bne .pp3
+       lda #0

.pp3:  sta t3                          // t3:r0l = start
        sec
        lda t0
        sbc r0l
        sta t0
        lda t1
        sbc t3
        sta t1                          // t1:t0 = end - start
        bpl +
        clc
        lda t0
        adc #<312
        sta t0
        lda t1
        adc #>312
        sta t1                          // wrap around the frame end
+       // Sum and maximum
        clc
        lda res_sum
        adc t0
        sta res_sum
        lda res_sum + 1
        adc t1
        sta res_sum + 1
        bcc +
        inc res_sum + 2
+       lda t1
        cmp res_max + 1
        bcc .nm
        bne .sm
        lda t0
        cmp res_max
        bcc .nm
.sm:    lda t0
        sta res_max
        lda t1
        sta res_max + 1
.nm:    inc fcl
        bne +
        inc fch
+       lda fcl
        sta res_frames
        lda fch
        sta res_frames + 1
        lda fch
        bne .done
        lda fcl
        cmp #FRAMES
        beq .done
        jmp .fr
.done:  rts

// ball_erase: restore the cell area of ball X from the background (8 lines x 2 bytes by block copy)
ball_erase:
        lda bxl,x
        sta t0
        lda bxh,x
        sta t1
        lsr t1
        ror t0
        lsr t1
        ror t0
        lsr t1
        ror t0                          // byte column = X / 8
        lda t0
        sta byt
        lda byy,x
        sta lin
        lda #$a0
        ldx #24
        jsr wr                          // block copy
        lda #8
        sta t3
.ee:    lda lin
        jsr line_addr                   // A = high, Y = low (line start)
        sta t2
        tya
        clc
        adc byt
        tay
        lda t2
        adc #0
        sta t2                          // destination = line start + byte
        pha
        tya
        pha
        lda t2
        jsr vdc_at                      // destination in R18/19
        pla
        tay
        pla                             // A = destination high, Y = destination low; source = destination + BG
        clc
        adc #>BG
        ldx #32
        jsr wr
        tya
        ldx #33
        jsr wr
        lda #2
        ldx #30
        jsr wr                          // copy 2 bytes
        inc lin
        dec t3
        bne .ee
        rts

// ball_move: move ball X by (dx, dy); bounce off walls (X 8..615, Y 72..184)
ball_move:
        clc
        lda bxl,x
        adc bdx,x
        sta bxl,x
        lda bxh,x
        adc bdh,x
        sta bxh,x
        // Limits: x < 8 or x > 615 (= $0267) -> reverse direction
        lda bxh,x
        bmi .bn                         // negative (below 0)
        cmp #2
        bcc .bx                         // < 512
        bne .bn                         // >= 768: too far
        lda bxl,x
        cmp #$67
        bcc .bx
.bn:    sec                             // reverse dx, X back to a valid range
        lda #0
        sbc bdx,x
        sta bdx,x
        lda bdh,x
        eor #$ff
        sta bdh,x
        lda bdx,x
        bne +
        inc bdh,x                       // (-0 -> new high byte)
+       clc
        lda bxl,x
        adc bdx,x
        sta bxl,x
        lda bxh,x
        adc bdh,x
        sta bxh,x
        clc
        lda bxl,x
        adc bdx,x
        sta bxl,x
        lda bxh,x
        adc bdh,x
        sta bxh,x
.bx:    lda bsh,x
        beq .up
        clc
        lda byy,x
        adc bdy,x
        sta byy,x
        cmp #184
        bcc .yd
        lda #0
        sta bsh,x
        rts
.up:    sec
        lda byy,x
        sbc bdy,x
        sta byy,x
        cmp #72
        bcs .yd
        lda #1
        sta bsh,x
.yd:    rts

// ball_draw: draw ball X: fine position = X mod 8, 8 lines x 2 bytes from ballimg
ball_draw:
        lda bxl,x
        and #7
        asl
        asl
        asl
        asl
        sta sh                          // sh * 16 = offset in the image
        lda bxl,x
        sta t0
        lda bxh,x
        sta t1
        lsr t1
        ror t0
        lsr t1
        ror t0
        lsr t1
        ror t0
        lda t0
        sta byt
        lda byy,x
        sta lin
        // Block copy off, so that R31 writes normally
        lda #$20
        ldx #24
        jsr wr
        lda #0
        sta t3                          // line counter
.dd:    lda lin
        jsr line_addr
        sta t2
        tya
        clc
        adc byt
        tay
        lda t2
        adc #0
        jsr vdc_at
        lda sh
        clc
        adc t3
        adc t3                          // sh*16 + line * 2
        tax
        lda ballimg,x
        jsr put
        lda ballimg + 1,x
        jsr put
        inc lin
        inc t3
        lda t3
        cmp #8
        bne .dd
        rts

ballpat: .byte $3c, $7e, $ff, $ff, $ff, $ff, $7e, $3c
bxl:    .fill NB, 0
bxh:    .fill NB, 0
byy:    .fill NB, 0
bdx:    .fill NB, 0
bdh:    .fill NB, 0
bdy:    .fill NB, 0
bsh:    .fill NB, 0
ballimg: .fill 128, 0                   // 8 fine positions x 8 lines x 2 bytes
