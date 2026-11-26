; =====================================================================================================================
; music_demo.asm - POKEY music with the original RMT player (Raster Music Tracker)
; =====================================================================================================================
;
; PURPOSE
;   Plays the POKEY song music/atune.vcsong composed with the tracker (4 channels, 2 patterns of 32 rows each, speed 6,
;   order 0 1 0 1, then repeat) in an endless loop.  The song is exported as an RMT module and played by the
;   original RMT player; the macros for it are in lib/rmt.inc, the player in lib/rmtplayr.asm (do not change,
;   third-party library).
;
; PREPARATION
;   Export first:  vcc media export music/atune.vcsong     (RMT module as ASM, label atune_rmt)
;   (`vcc build` does this automatically.)  The second song music/tune_pokey.vcsong is not used here.
;
; FLOW
;   1. rmt_start: initialize the player with the module at row 0 (return value in A: instrument speed of the module, here 1).
;   2. rmt_vbi: hooks rmt_vbi_entry into the deferred VBI.  From now on the operating system calls the
;      player in every frame (50 Hz PAL); the main program has nothing more to do and loops in `done`.
;   Hardware used: POKEY ($D200-$D208, written by the player), OS routines SETVBV $E45C / XITVBV $E462.
;
; MEMORY MAP
;   $2000-$201A main program and rmt_vbi_entry;  $30E0-$33FF player tables (occupied
;   according to the build: $3182-$33FF);  $3400-$394D player code (PLAYER = $3400, must lie on a page boundary);  $4000-$4181 RMT module.
;   Zero page $CB-$DD: exclusively used by the player.
; BUILD: vcc build . (project folder)
; =====================================================================================================================
.target atari8
.cpu sally { illegals = deny }

.org $2000
.include "lib/rmt.inc"

start:  rmt_start atune_rmt, 0      ; module + start row (returns the instrument speed in A, here 1 = every frame)
        rmt_vbi                     ; from now on the deferred VBI plays the music
done:   jmp done

rmt_vbi_code                        ; deferred VBI routine rmt_vbi_entry (is placed after done, is not run through)
.include "lib/rmtplayr.asm"         ; player: tables from $30E0, code from $3400 (RMT_PLAYER not set -> default $3400)
.org $4000                          ; module after the player
.include "music/atune.vcsong"
.run start
