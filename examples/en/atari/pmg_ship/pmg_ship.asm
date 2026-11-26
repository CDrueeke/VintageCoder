; Shows the four players of the spaceship (graphics/ship.vcpmg) side by side on the Atari 800XL.
; The data is created automatically when graphics/ship.vcpmg is included.
; The colors go into the shadow registers (PCOLR0 to PCOLR3): otherwise the operating system overwrites COLPM0 to COLPM3 in every frame.
; Line 0 of a player lies above the visible picture (visible from about line 32), so the spaceship is drawn from line 48 on.
.target atari8
.cpu sally { illegals = deny }

.const SDMCTL = $022f           ; shadow register of DMACTL (the operating system copies it every frame)
.const GPRIOR = $026f           ; shadow register of PRIOR
.const PCOLR0 = $02c0           ; shadow registers of the player colors COLPM0 to COLPM3 (four bytes)

.org $2000
start:  lda #$40
        sta antic.pmbase        ; player/missile page = $4000 (player 0 is at $4400)
        lda #3
        sta gtia.gractl         ; enable players and missiles
        lda #ship_prior         ; priority and width come from the export (settings in the sprite editor)
        sta GPRIOR
        lda #ship_sizep0
        sta gtia.sizep0
        lda #ship_sizep1
        sta gtia.sizep1
        lda #ship_sizep2
        sta gtia.sizep2
        lda #ship_sizep3
        sta gtia.sizep3
        lda #$3a                ; display list on, player DMA on, single-line resolution, normal field
        sta SDMCTL
        lda #ship_p0
        sta PCOLR0+0
        lda #ship_p1
        sta PCOLR0+1
        lda #ship_p2
        sta PCOLR0+2
        lda #ship_p3
        sta PCOLR0+3
        lda #100                ; four players side by side, 8 pixels wide each
        sta gtia.hposp0
        lda #108
        sta gtia.hposp1
        lda #116
        sta gtia.hposp2
        lda #124
        sta gtia.hposp3
done:   jmp done

.org $4400                      ; player 0; players 1 to 3 follow at a distance of $100
.include "graphics/ship.vcpmg"
.run start
