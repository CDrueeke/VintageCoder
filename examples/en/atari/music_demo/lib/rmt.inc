; SPDX-License-Identifier: GPL-3.0-only
; SPDX-FileCopyrightText: 2002-2009 Radek Sterba / Raster / C.P.U.
; =====================================================================================================================
; rmt.inc - Atari 8-bit: playback of RMT modules (Raster Music Tracker) with the original RMT player
; =====================================================================================================================
;
; ORIGIN
;   The player (rmtplayr.asm) is the RMT player 1.28 (Radek Sterba / Raster, GPLv3; origin and license are in the header of
;   rmtplayr.asm) in the notation of this assembler; according to that header it is byte for byte identical to the original translation.
;   Songs from the tracker panel (POKEY) are exported as an RMT module (vcc media export ... -> <name>_rmt) and can therefore
;   also be loaded in Raster Music Tracker.  This file contains only the macros that apply the player.
;
; USAGE
;   .const RMT_PLAYER = $3400        ; optional (page boundary, low byte 0!); the tables lie before it: mono from PLAYER-$320, stereo from PLAYER-$3C0
;   .const RMT_STEREO = 0            ; optional: 0 = 4 tracks, 1 = 8 tracks (2 POKEYs), 2/3 = 4 tracks mixed
;   .include "lib/rmt.inc"
;   ...
;   rmt_start atune_rmt, 0           ; module (label) and start row; then hook in the music tick:
;   rmt_vbi                          ; RMT playback every frame (deferred VBI); rmt_stop switches everything off
;   ...
;   rmt_vbi_code                     ; ONCE: creates the routine rmt_vbi_entry (the deferred VBI jumps there)
;   .include "lib/rmtplayr.asm"      ; ONCE, at an address without collision (tables before PLAYER, code from PLAYER)
;
; RUNTIME FLOW
;   rmt_start initializes the player (X/Y = module, A = start row).  rmt_vbi enters rmt_vbi_entry into the deferred VBI vector via the OS routine SETVBV ($E45C; A = 7 =
;   deferred VBI, X = high byte, Y = low byte of the new address); from then on the
;   operating system calls rmt_vbi_entry in every frame (50 Hz PAL, 60 Hz NTSC): play routine of the player (jsr Player+3), then return
;   via XITVBV ($E462).  The OS must stay active for this (VBI not switched off).
;
; ENTRY POINTS OF THE PLAYER (jump table from RASTERMUSICTRACKER = PLAYER)
;   +0 rmt_init: X/Y = module (low/high), A = start row; returns in A the instrument speed of the module (1 = every frame);
;   +3 rmt_play: call once per frame;  +6 rmt_p3 (partial routine of playback, unused here);  +9 rmt_silence (everything off);
;   +12 SetPokey (writes the register values to POKEY, unused here).
;   Zero page used: $CB-$DD (19 bytes, used exclusively by the player).
;
; MACROS
;   rmt_start(module, line)   initialize the player; changes A, X, Y (the player changes further registers as it likes)
;   rmt_stop                  player entry +9: all POKEY channels silent
;   rmt_vbi                   set deferred VBI (SETVBV); changes A, X, Y
;   rmt_vbi_code              creates rmt_vbi_entry (code: jsr Player+3 / jmp $E462); not executable -> place after the main program
; =====================================================================================================================

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
