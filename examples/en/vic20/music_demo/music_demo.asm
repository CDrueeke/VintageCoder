; =====================================================================================================================
; music_demo.asm - VIC-20: plays a song composed with the tracker through the VIC-I (3 tones + noise)
; =====================================================================================================================
;
; PURPOSE
;   The example shows the shortest way from a tracker song to music on the VIC-20: the song music/tune.vcsong (profile vic20.vic.song) is exported at
;   include time (tune_order, tune_patterns, tune_ins and the constants tune_speed, tune_rows, tune_patsize), the
;   player routine lib/music.inc reads it, and an IRQ hook calls it once per frame. You learn from it: how a song export is included,
;   how to hook in behind the KERNAL IRQ, and how the VIC-I with its four voices and the shared volume is addressed.
;
; WHAT YOU SEE AND HEAR
;   A short loop (32 rows of 6 frames each, just under 4 seconds in PAL): bass, alto and soprano with the instruments "Ton" (tone) and "Kurz" (short), plus
;   noise bursts ("Tick") as drums. The screen stays as BASIC left it; the program prints nothing, the cursor keeps
;   blinking because the KERNAL IRQ keeps running.
;
; BUILD AND RUN
;   Project music_demo.vcprj (VIC-20 PAL, memory configuration "unexpanded", CPU 6502): "vcc build examples/vic20/music_demo" first exports the song
;   and builds build/music_demo.prg. Start with LOAD and RUN (the BASIC line "10 SYS" is made by
;   .basicstub); stop with reset. For NTSC choose the table with "-D NTSC=1" (or .const NTSC = 1 before including lib/vic_freq.inc); the
;   tempo also depends on the TV standard (see below).
;
; WHAT THE CODE DOES ON THE HARDWARE
;   start disables interrupts, prepares the player (music_init: self-modification of the addresses, clear state, voices off, volume 15),
;   redirects the IRQ vector CINV ($0314/$0315) to irq and enables interrupts again. After that the main loop does nothing. Every IRQ tick of the
;   KERNAL (timer 1 of VIA 2, $9124: PAL 50 Hz, NTSC 60 Hz) calls irq: there music_play runs and then jumps on to the original KERNAL routine
;   (keyboard, cursor, clock, return with RTI). The KERNAL IRQ entry has already saved A, X and Y, so music_play may use them freely.
;
;   Registers: VIC-I $900A bass, $900B alto, $900C soprano, $900D noise (bit 7 = on, bits 0..6 = pitch) and $900E (volume 0..15 in bits 0..3,
;   auxiliary color in bits 4..7, is preserved). The VIC-I has no envelopes per voice; the volume applies to all voices together (see
;   lib/music.inc).
;
; MEMORY MAP (VIC-20 without expansion)
;   $1001  BASIC line "10 SYS" (.basicstub), start at $100D      zero page $FB-$FE: player pointers (music.inc)
;   then program (start, irq, old_irq), music_runtime (player routine and variables), frequency tables (4 x 96 bytes, lib/vic_freq.inc),
;   song data (tune_order, tune_patterns 256 bytes, tune_ins 96 bytes); together currently $1001-$14C1 (1217 bytes). Screen $1E00 and color RAM $9600 stay untouched.
;
; KNOWN LIMITS
;   One song, no control (no stop, no pause); the song speed depends on the TV standard because one frame is counted per IRQ tick;
;   the frequency table is valid only for the chosen standard (PAL without -D NTSC=1).
; =====================================================================================================================
.target vic20
.cpu 6502

.org $1001                          ; BASIC start of the basic machine ($1001, project setting "unexpanded")
.basicstub start                    ; creates "10 SYS <start>" so that RUN starts the program

.include "lib/music.inc"

start:  sei                         ; disable interrupts while the player and IRQ vector are rebuilt
        music_init tune_order, tune_patterns, tune_ins, tune_speed, tune_rows, tune_patsize   ; prepare the player with the export data
        lda $0314                   ; remember the KERNAL IRQ and hook in behind it
        sta old_irq
        lda $0315
        sta old_irq+1
        lda #<irq
        sta $0314
        lda #>irq
        sta $0315                   ; from here on the KERNAL IRQ calls irq first
        cli
loop:   jmp loop                    ; endless loop: everything else happens in the IRQ

; irq: hooked IRQ routine, once per KERNAL tick (PAL 50 Hz / NTSC 60 Hz). A, X, Y are already saved by the KERNAL entry.
irq:    jsr music_play              ; one player step: advance the row, envelopes, write VIC-I registers
        jmp (old_irq)               ; continue in the original KERNAL routine (keyboard, cursor, clock, RTI)

old_irq: .word 0                    ; remembered old IRQ vector (set by start)

music_runtime                       ; player routine and variables, ONCE behind the main program (not executed)
.include "lib/vic_freq.inc"         ; frequency tables (note 1..96 -> register value) for bass, alto, soprano and noise, PAL or NTSC with NTSC=1
.include "music/tune.vcsong"    ; song data from the tracker export (made at build time from music/tune.vcsong, not in the source tree)
