; =====================================================================================================================
; sound.asm - Klangmodul (TED): Effekte mit Prioritaeten, Titelmusik, zwei Stimmen, ein Aufruf je Bild
; =====================================================================================================================
; Herkunft: examples/vic20/troxx/sound.asm, Stand Commit 4697d074 und examples/c64/troxx/sound.asm (Stand d173cb4a): gleiche Schnittstelle (init, play,
;   update, silence, music_start, music_stop, Fx.*); der Klangmodul ist Plus/4-spezifisch (TED statt SID) und daher nicht in der Kopienliste.
; ZWECK     Jede der zwei TED-Stimmen spielt einen Strom (Stream): Kopf aus einem Byte (0 Rechteck, 1 Rauschen; Rauschen gibt es nur auf Stimme 2),
;           dann Eintraege zu zwei Byte (Notennummer, Dauer in Bildern); Note 0 = Pause (Stimme aus), $FF = Ende, $FE = von vorn (Musik).  Ein Effekt
;           besteht aus bis zu zwei Stroemen (je Stimme einer, Tabelle EFX_STREAM); er ersetzt einen laufenden Strom nur, wenn seine Prioritaet groesser
;           ist.  Die Titelmusik belegt beide Stimmen mit Prioritaet 255 und sperrt die Effekte, bis music_stop aufgerufen wird.  Der TED kennt keine
;           Huellkurve: Toene haben eine feste Lautstaerke (eine fuer beide Stimmen, $FF11 Bit 0-3), die Gestalt entsteht durch kurze Notenfolgen.
; BEDIENUNG sound.init einmal; sound.play mit A = Effektnummer (Fx.*; veraendert nur A, X, Y, nicht T0..T7); sound.update einmal je Bild (veraendert A, X, Y);
;           music_start / music_stop fuer die Titelmusik; silence stoppt alles.
; HARDWARE  TED: Stimme 1 = $FF0E (Bit 0-7) und $FF12 Bit 0-1; Stimme 2 = $FF0F und $FF10 Bit 0-1; $FF11: Bit 0-3 Lautstaerke (0..8), Bit 4 Stimme 1 an,
;           Bit 5 Stimme 2 an, Bit 6 Stimme 2 als Rauschen.  Frequenz = Takt / (8 * (1024 - N)) mit Takt PAL 886724 * 8 / 8 -> f = 110840,5 Hz / (1024 - N)
;           (PAL), 111860,8 Hz / (1024 - N) (NTSC); Tabellen NLO/NHI fuer 64 Noten je Norm (Notennummer n = MIDI-Nummer - 35).  Die tiefste Frequenz ist
;           108 Hz (N = 0): Noten unter A2 gibt es nicht (sie werden auf N = 0 begrenzt), darum liegen Bass und Tieftoene hoeher als beim C64.
; =====================================================================================================================

.enum Fx
    step                                ; kurzes Klicken bei jedem Schritt (Prioritaet 1)
    seg                                 ; Segment abgeschlossen: zwei steigende Toene (2)
    box                                 ; Rechteck gefuellt: zwei Toene (3)
    chain                               ; Kettenreaktion: drei Toene (4)
    hit                                 ; Treffer: fallendes Rauschen (5)
    clear                               ; Stufe geschafft: Fanfare (6)
    over                                ; Spielende: fallende Toene (5)
    bonus                               ; Stern aufgenommen (3)
    ice                                 ; Eisblock aufgenommen (3)
    destroy                             ; Gegner zerstoert eine Linie: Knistern und fallender Ton (3)
.endenum

; Zustand im RAM ab SND (je Stimme ein Byte; Stimmen 0 und 1)
.const S_PTRLO  = SND                   ; +0, +1 Leseposition (niedrig)
.const S_PTRHI  = SND + 2               ; +2, +3 (hoch)
.const S_TIME   = SND + 4               ; +4, +5 Bilder bis zum naechsten Eintrag
.const S_PRIO   = SND + 6               ; +6, +7 Prioritaet des laufenden Stroms (0 = ruhig)
.const S_BASELO = SND + 8               ; +8, +9 Anfang der Eintraege (fuer $FE)
.const S_BASEHI = SND + 10              ; +10, +11
.const S_WAVE   = SND + 12              ; +12, +13 Kopf: 0 Rechteck, 1 Rauschen
.const S_NORM   = SND + 14              ; 0 (PAL) oder 64 (NTSC): Versatz in den Frequenztabellen
.const S_MUSIC  = SND + 15              ; 1 = Titelmusik laeuft
.const S_N      = SND + 16              ; Notennummer (Hilfe)
.const S_V      = SND + 17              ; Stimme (Hilfe)
.const S_P      = SND + 18              ; Prioritaet des Effekts (Hilfe)
.const S_LO     = SND + 19              ; Frequenz niedrig (Hilfe)
.const S_E      = SND + 20              ; Effektnummer (Hilfe)
.const S_B      = SND + 21              ; Abstand in EFX_STREAM (Hilfe)
.const S_CTL    = SND + 22              ; $FF11 ohne Lautstaerke: Bit 4 Stimme 1, Bit 5 Stimme 2, Bit 6 Rauschen
.const VOLUME   = 8                     ; Lautstaerke (0..8)

sound: {

; init: Zustand loeschen, Notenversatz je Norm.  Veraendert A, X.
init:
        lda #0
        ldx #31
.clr:   sta SND,x
        dex
        bpl .clr
        lda FLAGS
        and #F_PAL
        bne .pal
        lda #64                         ; NTSC: zweite Tabellenhaelfte
        sta S_NORM
.pal:   jmp silence

; out: $FF11 aus S_CTL und der Lautstaerke setzen.  Veraendert A.
out:    lda S_CTL
        ora #VOLUME
        sta TED_SOUND
        rts

; silence: beide Stimmen aus, alle Stroeme gestoppt.  Veraendert A, X.
silence:
        lda #0
        sta S_MUSIC
        sta S_CTL
        sta S_PRIO
        sta S_PRIO + 1
        jmp out

; start: Strom ab SPTR auf Stimme X mit Prioritaet A starten (Kopf lesen, Stimme aus).  Veraendert A, Y.
start:  sta S_PRIO,x
        ldy #0
        lda (SPTR),y
        sta S_WAVE,x
        lda SPTR                        ; Eintraege beginnen hinter dem Kopf (1 Byte)
        clc
        adc #1
        sta S_PTRLO,x
        sta S_BASELO,x
        lda SPTR + 1
        adc #0
        sta S_PTRHI,x
        sta S_BASEHI,x
        lda #1
        sta S_TIME,x                    ; im naechsten update den ersten Eintrag lesen
        rts

; play: A = Effektnummer.  Veraendert A, X, Y (nicht T0..T7).
play:   ldx S_MUSIC
        bne .skip                       ; waehrend der Titelmusik keine Effekte
        sta S_E
        tax
        lda EFX_PRIO,x
        sta S_P
        lda S_E
        asl
        asl
        sta S_B                         ; Abstand in EFX_STREAM = Effekt * 4
        ldx #1                          ; Stimme 1 .. 0
.v:     txa
        asl
        clc
        adc S_B
        tay
        lda EFX_STREAM + 1,y
        beq .nx                         ; kein Strom fuer diese Stimme
        lda S_PRIO,x
        cmp S_P
        bcs .nx                         ; laeuft etwas Gleichwichtiges oder Wichtigeres: ignorieren
        lda EFX_STREAM,y
        sta SPTR
        lda EFX_STREAM + 1,y
        sta SPTR + 1
        lda S_P
        jsr start
.nx:    dex
        bpl .v
.skip:  rts

; music_start: Titelmusik auf beiden Stimmen starten (Prioritaet 255, Effekte gesperrt).  Veraendert A, X, Y.
music_start:
        jsr silence
        lda #1
        sta S_MUSIC
        ldx #1
.m:     lda MUSIC_LO,x
        sta SPTR
        lda MUSIC_HI,x
        sta SPTR + 1
        lda #255
        jsr start
        dex
        bpl .m
        rts

; music_stop: Titelmusik beenden.  Veraendert A, X.
music_stop:
        jmp silence

; update: einmal je Bild.  Veraendert A, X, Y.
update: ldx #1
.v:     lda S_PRIO,x
        bne .act
        jmp .nx
.act:   dec S_TIME,x
        beq .step
        jmp .nx
.step:  stx S_V
        lda S_PTRLO,x
        sta SPTR
        lda S_PTRHI,x
        sta SPTR + 1
.read:  ldy #0
        lda (SPTR),y
        cmp #$ff
        bne .nostop
        jmp .stop
.nostop:
        cmp #$fe
        bne .noloop
        jmp .loop
.noloop:
        sta S_N
        iny
        lda (SPTR),y
        sta S_TIME,x                    ; Dauer
        lda SPTR
        clc
        adc #2
        sta S_PTRLO,x
        lda SPTR + 1
        adc #0
        sta S_PTRHI,x
        lda S_N
        beq .rest
        clc
        adc S_NORM
        tay
        lda NLO - 1,y
        sta S_LO
        lda NHI - 1,y
        tay                             ; Y = hohe zwei Bit
        cpx #0
        bne .v2
        lda S_LO                        ; Stimme 1
        sta TED_VOICE1_LO
        lda TED_BITMAP_BASE
        and #$fc
        sta S_LO
        tya
        ora S_LO
        sta TED_BITMAP_BASE
        lda S_CTL
        ora #$10
        sta S_CTL
        jmp .nx
.v2:    lda S_LO                        ; Stimme 2
        sta TED_VOICE2_LO
        sty TED_VOICE2_HI
        lda S_CTL
        and #$9f                        ; Stimme 2 und Rauschen aus, dann nach Kopf wieder an
        ora #$20
        ldy S_WAVE + 1
        beq .sq
        ora #$40
.sq:    sta S_CTL
        jmp .nx
.rest:  cpx #0                          ; Pause: Stimme aus
        bne .r2
        lda S_CTL
        and #$ef
        sta S_CTL
        jmp .nx
.r2:    lda S_CTL
        and #$9f
        sta S_CTL
        jmp .nx
.loop:  lda S_BASELO,x                  ; Musik: von vorn
        sta SPTR
        lda S_BASEHI,x
        sta SPTR + 1
        jmp .read
.stop:  lda #0
        sta S_PRIO,x
        cpx #0
        bne .s2
        lda S_CTL
        and #$ef
        sta S_CTL
        jmp .nx
.s2:    lda S_CTL
        and #$9f
        sta S_CTL
.nx:    dex
        bmi .end
        jmp .v
.end:   jmp out
}

; ---- Notennummern (n = MIDI - 35): C2 = 1 ... D7 = 64
.const N_F2 = 6
.const N_G2 = 8
.const N_A2 = 10
.const N_C3 = 13
.const N_D3 = 15
.const N_E3 = 17
.const N_F3 = 18
.const N_G3 = 20
.const N_A3 = 22
.const N_B3 = 24
.const N_C4 = 25
.const N_D4 = 27
.const N_E4 = 29
.const N_F4 = 30
.const N_G4 = 32
.const N_A4 = 34
.const N_B4 = 36
.const N_C5 = 37
.const N_D5 = 39
.const N_E5 = 41
.const N_F5 = 42
.const N_G5 = 44
.const N_A5 = 46
.const N_C6 = 49
.const N_E6 = 53
.const N_G6 = 56
.const N_C7 = 61

.macro nt(n, d)
        .byte n, d
.endmacro

; ---- Tonhoehentabellen: 64 Noten je Norm (erst PAL, dann NTSC); N = 1024 - Takt / Tonhoehe, begrenzt auf 0..1020 (tiefster Ton 108 Hz);
;      Tonhoehe = 440 Hz * 2^((MIDI - 69) / 12)
.var clocks = [110840.5, 111860.8]
NLO:
.for c in clocks
    .for n in range(64)
        .byte min(1020, max(0, round(1024 - c / (440 * pow(2, (n + 36 - 69) / 12))))) & 255
    .endfor
.endfor
NHI:
.for c in clocks
    .for n in range(64)
        .byte min(1020, max(0, round(1024 - c / (440 * pow(2, (n + 36 - 69) / 12))))) >> 8
    .endfor
.endfor

; ---- Effekte: Stroeme (Kopf: Wellenform; dann Note/Dauer; $FF = Ende)
snd_step:                               ; Rauschen auf Stimme 2, ein Bild
        .byte 1
        nt 62, 1
        .byte $ff
snd_seg:
        .byte 0
        nt N_C5, 3
        nt N_E5, 3
        .byte $ff
snd_box:
        .byte 0
        nt N_E5, 4
        nt N_C6, 5
        .byte $ff
snd_chain:
        .byte 0
        nt N_C5, 3
        nt N_E5, 3
        nt N_C6, 6
        .byte $ff
snd_chain1:
        .byte 0
        nt N_E4, 3
        nt N_A4, 3
        nt N_E5, 6
        .byte $ff
snd_hit:                                ; tiefer fallender Ton auf Stimme 1 ...
        .byte 0
        nt N_G3, 4
        nt N_E3, 4
        nt N_C3, 5
        nt N_A2, 6
        .byte $ff
snd_hit1:                               ; ... und fallendes Rauschen auf Stimme 2
        .byte 1
        nt 45, 4
        nt 38, 4
        nt 32, 5
        nt 26, 6
        .byte $ff
snd_clear:
        .byte 0
        nt N_C5, 4
        nt N_E5, 4
        nt N_G5, 4
        nt N_C6, 8
        nt N_G5, 3
        nt N_C6, 12
        .byte $ff
snd_clear1:
        .byte 0
        nt N_C4, 4
        nt N_E4, 4
        nt N_G4, 4
        nt N_C5, 8
        nt N_G4, 3
        nt N_C5, 12
        .byte $ff
snd_over:
        .byte 0
        nt N_G5, 8
        nt N_D5, 8
        nt N_A4, 16
        .byte $ff
snd_over1:
        .byte 0
        nt N_E3, 16
        nt N_A2, 16
        .byte $ff
snd_bonus:
        .byte 0
        nt N_C6, 2
        nt N_E6, 2
        nt N_G6, 2
        nt N_C7, 4
        .byte $ff
snd_ice:
        .byte 0
        nt N_G6, 2
        nt N_C6, 2
        nt N_G6, 2
        nt N_C6, 2
        nt N_E6, 2
        nt N_A5, 2
        nt N_E5, 6
        .byte $ff

snd_destroy:                            ; fallender hoher Ton auf Stimme 1 (kurz, hell: nicht mit dem Treffer zu verwechseln)
        .byte 0
        nt N_A5, 2
        nt N_E5, 2
        nt N_C5, 2
        nt N_G4, 3
        .byte $ff
snd_destroy1:                           ; Knistern: Rauschen in wechselnder Hoehe auf Stimme 2
        .byte 1
        nt 58, 1
        nt 40, 1
        nt 55, 1
        nt 36, 1
        nt 50, 1
        nt 30, 2
        .byte $ff

; Effekt -> Stroeme je Stimme (Stimme 1 = TED-Stimme 1, Stimme 2; 0 = keiner), Prioritaet
EFX_STREAM:
        .word 0, snd_step               ; step (Rauschen: Stimme 2)
        .word snd_seg, 0                ; seg
        .word snd_box, 0                ; box
        .word snd_chain, snd_chain1     ; chain
        .word snd_hit, snd_hit1         ; hit
        .word snd_clear, snd_clear1     ; clear
        .word snd_over, snd_over1       ; over
        .word snd_bonus, 0              ; bonus
        .word snd_ice, 0                ; ice
        .word snd_destroy, snd_destroy1 ; destroy (Rauschen: Stimme 2)
EFX_PRIO:
        .byte 1, 2, 3, 4, 5, 6, 5, 3, 3, 3

; ---- Titelmusik (a-Moll, Folge Am F C G; 32 Schritte zu 6 Bildern, wird wiederholt): Stimme 1 Melodie, Stimme 2 Bass im Wechsel mit dem Akkord (Arpeggio)
MUSIC_LO: .byte <mus_mel, <mus_bass
MUSIC_HI: .byte >mus_mel, >mus_bass
.macro bar_bass(r, a, b, c)             ; Bass (eine Oktave hoeher als beim C64: tiefer als 108 Hz geht nicht) im Wechsel mit Akkordtoenen
        nt r, 6
        nt a, 6
        nt r + 12, 6
        nt b, 6
        nt r, 6
        nt c, 6
        nt r + 12, 6
        nt b, 6
.endmacro
mus_mel:
        .byte 0
        nt N_E5, 12
        nt N_C5, 6
        nt N_D5, 6
        nt N_E5, 12
        nt N_A5, 12
        nt N_A5, 12
        nt N_G5, 6
        nt N_F5, 6
        nt N_E5, 12
        nt N_C5, 12
        nt N_G5, 12
        nt N_E5, 6
        nt N_D5, 6
        nt N_C5, 12
        nt N_E5, 12
        nt N_D5, 12
        nt N_B4, 6
        nt N_D5, 6
        nt N_G5, 24
        .byte $fe
mus_bass:
        .byte 0
        bar_bass N_A2 + 12, N_C4 + 12, N_E4 + 12, N_C4 + 12
        bar_bass N_F2 + 12, N_A3 + 12, N_C4 + 12, N_A3 + 12
        bar_bass N_C3 + 12, N_E4 + 12, N_G4 + 12, N_E4 + 12
        bar_bass N_G2 + 12, N_B3 + 12, N_D4 + 12, N_B3 + 12
        .byte $fe
