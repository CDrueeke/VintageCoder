; SPDX-License-Identifier: GPL-3.0-only
; SPDX-FileCopyrightText: 2002-2009 Radek Sterba / Raster / C.P.U.
; rmt.inc - Atari 8 bit: playback of RMT modules (Raster Music Tracker) with the original RMT player.
;
; The player (rmtplayr.asm) is the RMT player 1.28 (Radek Sterba / Raster, GPLv3) in the notation of this assembler;
; it is byte for byte identical to the original build. Songs from the tracker panel (POKEY)
; are exported as an RMT module (vcc media export ... -> <name>_rmt) and can therefore also be loaded in the Raster Music Tracker.
;
;   .const RMT_PLAYER = $3400        ; optional (page boundary!); the tables lie before it: mono from PLAYER-$320, stereo from PLAYER-$3C0
;   .const RMT_STEREO = 0            ; optional: 0 = 4 tracks, 1 = 8 tracks (2 POKEY), 2/3 = 4 tracks mixed
;   .include "lib/rmt.inc"
;   ...
;   rmt_start atune_rmt, 0           ; module (label) and start line; afterwards hook in the music tick:
;   rmt_vbi                          ; RMT playback every frame (deferred VBI); rmt_stop switches everything off
;   ...
;   .include "lib/rmtplayr.asm"      ; ONCE, at an address without collision (tables before PLAYER, code from PLAYER)
;
; In "Hugo & Gumbo" (jumprun.asm) only the macro rmt_start is used: the vertical-blank interrupt (vbi) is in hal_atari.asm and
; calls the player (+3) together with the sound effects; rmt_stop, rmt_vbi and rmt_vbi_code stay unused there.
;
; Entry points of the player: RASTERMUSICTRACKER (init, X/Y = module, A = start line; returns the instrument speed),
; +3 = play (once per frame), +9 = all off.  Zero page used: $CB-$DD.

.macro rmt_start(module, line)
        ldx #<module
        ldy #>module
        lda #line
        jsr RASTERMUSICTRACKER
.endmacro

.macro rmt_stop
        jsr RASTERMUSICTRACKER+9
.endmacro

.macro rmt_vbi                      ; set deferred VBI to rmt_vbi_entry (the caller creates rmt_vbi_entry, see rmt_vbi_code)
        lda #7
        ldx #>rmt_vbi_entry
        ldy #<rmt_vbi_entry
        jsr $e45c
.endmacro

.macro rmt_vbi_code                 ; routine of the deferred VBI
rmt_vbi_entry:
        jsr RASTERMUSICTRACKER+3
        jmp $e462
.endmacro
