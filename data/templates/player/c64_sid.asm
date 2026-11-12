; Abspielprogramm für Tracker-Lieder (C64, SID, 3 Stimmen) – wird von „Auf Hardware hören“ zusammengesetzt.
; Das Lied kommt als build/export-Datei „song.asm“ (Label song_*), die Spielroutine steht in music.inc.
.target c64
.cpu 6510 { illegals = deny }

.org $0801
.basicstub start

.include "music.inc"

start:  sei
        music_init song_order, song_patterns, song_ins, song_speed, song_rows, song_patsize
        ; Rasterzeilen-IRQ: genau ein Takt je Bild (PAL 50 Hz, NTSC 60 Hz)
        lda #$7f
        sta $dc0d
        sta $dd0d
        lda $dc0d
        lda $dd0d
        lda #$01
        sta $d01a
        lda #$00
        sta $d012
        lda $d011
        and #$7f
        sta $d011
        lda #<irq
        sta $0314
        lda #>irq
        sta $0315
        cli
loop:   jmp loop

irq:    lda #$01
        sta $d019
        jsr music_play
        jmp $ea81

music_runtime
.include "song.asm"
