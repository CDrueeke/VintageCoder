; =====================================================================================================================
; sound.asm - Klangmodul (SID): Effekte mit Prioritaeten, Titelmusik, drei Stimmen, ein Aufruf je Bild
; =====================================================================================================================
; Herkunft: examples/vic20/troxx/sound.asm, Stand Commit 4697d074 (gleiche Schnittstelle: init, play, update, silence, Fx.*); der Klangmodul
;   ist C64-spezifisch (SID statt VIC-I) und daher nicht in der Kopienliste.
; ZWECK     Jede der drei SID-Stimmen spielt einen Strom (Stream): Kopf aus drei Byte (Wellenform ohne Gate, Attack/Decay, Sustain/Release), dann
;           Eintraege zu zwei Byte (Notennummer, Dauer in Bildern); Note 0 = Pause (Gate aus), $FF = Ende, $FE = von vorn (Musik).  Ein Effekt besteht
;           aus bis zu drei Stroemen (je Stimme einer, Tabelle EFX_STREAM); er ersetzt einen laufenden Strom nur, wenn seine Prioritaet groesser ist.
;           Die Titelmusik belegt alle drei Stimmen mit Prioritaet 255 und sperrt die Effekte, bis music_stop aufgerufen wird.
; BEDIENUNG sound.init einmal; sound.play mit A = Effektnummer (Fx.*; veraendert nur A, X, Y, nicht T0..T7); sound.update einmal je Bild
;           (veraendert A, X, Y); music_start / music_stop fuer die Titelmusik; silence stoppt alles.
; HARDWARE  SID $D400 ff.: je Stimme 7 Register (Frequenz lo/hi, Pulsbreite lo/hi, Steuerung, AD, SR); $D418 Lautstaerke.  Steuerung: Bit 0 Gate,
;           Bit 4 Dreieck, 5 Saege, 6 Rechteck, 7 Rauschen.  Frequenzwert = Tonhoehe * 2^24 / Takt (PAL 985248 Hz, NTSC 1022727 Hz); Tabellen FLO/FHI
;           fuer 64 Noten C2..D7 je Norm (Notennummer n = MIDI-Nummer - 35).
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
    destroy                             ; Linie zerstoert: Knistern und fallender Ton (4)
.endenum

; Zustand im RAM ab SND (je Stimme ein Byte; Stimmen 0..2)
.const S_PTRLO  = SND                   ; +0..2 Leseposition (niedrig)
.const S_PTRHI  = SND + 3               ; +3..5 (hoch)
.const S_TIME   = SND + 6               ; +6..8 Bilder bis zum naechsten Eintrag
.const S_PRIO   = SND + 9               ; +9..11 Prioritaet des laufenden Stroms (0 = ruhig)
.const S_BASELO = SND + 12              ; +12..14 Anfang der Eintraege (fuer $FE)
.const S_BASEHI = SND + 15              ; +15..17
.const S_WAVE   = SND + 18              ; +18..20 Wellenform ohne Gate
.const S_NORM   = SND + 21              ; 0 (PAL) oder 64 (NTSC): Versatz in den Frequenztabellen
.const S_MUSIC  = SND + 22              ; 1 = Titelmusik laeuft
.const S_N      = SND + 23              ; Notennummer (Hilfe)
.const S_V      = SND + 24              ; Stimme (Hilfe)
.const S_P      = SND + 25              ; Prioritaet des Effekts (Hilfe)
.const S_LO     = SND + 26              ; Frequenz niedrig (Hilfe)
.const S_E      = SND + 27              ; Effektnummer (Hilfe)
.const S_B      = SND + 28              ; Abstand in EFX_STREAM (Hilfe)
.const S_F      = SND + 29              ; Zaehler des Filterlaufs (Titelmusik)
.const SPTR     = $7a                   ; Zeropage: Zeiger in den Strom (2 Byte)

sound: {

; init: Lautstaerke 15, Pulsbreite, Zustand loeschen, Notenversatz je Norm.  Veraendert A, X.
init:
        lda #$0f
        sta SID_MODE_VOL
        lda #0
        sta SID_RES_FILT
        ldx #31
.clr:   sta SND,x
        dex
        bpl .clr
        lda #$08                        ; Pulsbreite $0800 (Rechteck) fuer alle Stimmen
        sta SID_V1_PW_HI
        sta SID_V2_PW_HI
        sta SID_V3_PW_HI
        lda FLAGS
        and #F_PAL
        bne .pal
        lda #64                         ; NTSC: zweite Tabellenhaelfte
        sta S_NORM
.pal:   jmp silence

; silence: alle Stimmen aus, alle Stroeme gestoppt.  Veraendert A, X.
silence:
        lda #0
        sta S_MUSIC
        sta SID_RES_FILT                ; Filter aus
        lda #$0f
        sta SID_MODE_VOL
        lda #0
        ldx #2
.mute:  sta S_PRIO,x
        sta SID_V1_CTRL
        sta SID_V2_CTRL
        sta SID_V3_CTRL
        dex
        bpl .mute
        rts

; start: Strom ab SPTR auf Stimme X mit Prioritaet A starten (Kopf lesen, Gate aus).  Veraendert A, Y.
start:  sta S_PRIO,x
        ldy #0
        lda (SPTR),y
        sta S_WAVE,x
        lda VOFF,x
        stx S_V
        tay                             ; Y = Registerabstand der Stimme
        lda S_WAVE,x
        sta SID_V1_CTRL,y               ; Gate aus (Neuanschlag im naechsten Bild)
        ldy #1
        lda (SPTR),y
        pha
        iny
        lda (SPTR),y
        pha
        ldy VOFF,x
        pla
        sta SID_V1_SR,y
        pla
        sta SID_V1_AD,y
        lda SPTR                        ; Eintraege beginnen hinter dem Kopf (3 Byte)
        clc
        adc #3
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
        sta S_B
        asl
        clc
        adc S_B
        sta S_B                         ; Abstand in EFX_STREAM = Effekt * 6
        ldx #2                          ; Stimme 2 .. 0
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

; music_start: Titelmusik auf allen drei Stimmen starten (Prioritaet 255, Effekte gesperrt).  Veraendert A, X, Y.
music_start:
        jsr silence
        lda #1
        sta S_MUSIC
        lda #$f1                        ; Tiefpassfilter (Resonanz 15) auf der Bass-Stimme, Grenzfrequenz laeuft in update hin und her
        sta SID_RES_FILT
        lda #$1f
        sta SID_MODE_VOL
        ldx #2
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
update: lda S_MUSIC
        beq .nf
        inc S_F                         ; Filterlauf: Dreieck 0..127 -> Grenzfrequenz
        lda S_F
        bpl .up
        eor #$ff
.up:
.if FX_PWM
        pha
        lsr
        lsr
        lsr
        ora #1
        sta SID_V2_PW_HI                ; Pulsbreitenmodulation der Begleitstimme (Stimme 2)
        pla
.endif
        clc
        adc #12
        sta SID_FILTER_HI
.nf:    ldx #2
.v:     lda S_PRIO,x
        beq .nx
        dec S_TIME,x
        bne .nx
        stx S_V
        lda S_PTRLO,x
        sta SPTR
        lda S_PTRHI,x
        sta SPTR + 1
.read:  ldy #0
        lda (SPTR),y
        cmp #$ff
        beq .stop
        cmp #$fe
        beq .loop
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
        lda VOFF,x
        tay                             ; Y = Registerabstand der Stimme
        lda S_N
        beq .rest
        clc
        adc S_NORM
        tax
        lda FLO - 1,x
        sta S_LO
        lda FHI - 1,x
        ldx S_V
        sta SID_V1_FREQ_HI,y
        lda S_LO
        sta SID_V1_FREQ_LO,y
        lda S_WAVE,x
        ora #1                          ; Gate an
        sta SID_V1_CTRL,y
        jmp .nx
.rest:  lda S_WAVE,x                    ; Pause: Gate aus
        sta SID_V1_CTRL,y
.nx:    dex
        bpl .v
        rts
.loop:  lda S_BASELO,x                  ; Musik: von vorn
        sta SPTR
        lda S_BASEHI,x
        sta SPTR + 1
        jmp .read
.stop:  lda #0
        sta S_PRIO,x
        lda VOFF,x
        tay
        lda S_WAVE,x
        sta SID_V1_CTRL,y               ; Gate aus (Release klingt aus)
        jmp .nx
}

VOFF: .byte 0, 7, 14

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

; ---- Frequenztabellen: 64 Noten je Norm (erst PAL, dann NTSC); Tonhoehe = 440 Hz * 2^((MIDI - 69) / 12)
.var clocks = [985248, 1022727]
FLO:
.for c in clocks
    .for n in range(64)
        .byte round(440 * pow(2, (n + 36 - 69) / 12) * 16777216 / c) & 255
    .endfor
.endfor
FHI:
.for c in clocks
    .for n in range(64)
        .byte round(440 * pow(2, (n + 36 - 69) / 12) * 16777216 / c) >> 8
    .endfor
.endfor

; ---- Effekte: Stroeme (Kopf: Wellenform, AD, SR; dann Note/Dauer; $FF = Ende)
snd_step:
        .byte $80, $00, $00
        nt 55, 1
        .byte $ff
snd_seg:
        .byte $40, $09, $a6
        nt N_C5, 3
        nt N_E5, 3
        .byte $ff
snd_box:
        .byte $40, $09, $a6
        nt N_E5, 4
        nt N_C6, 5
        .byte $ff
snd_chain:
        .byte $40, $09, $a6
        nt N_C5, 3
        nt N_E5, 3
        nt N_C6, 6
        .byte $ff
snd_chain1:
        .byte $40, $09, $a6
        nt N_E4, 3
        nt N_A4, 3
        nt N_E5, 6
        .byte $ff
snd_chain0:
        .byte $20, $00, $a8
        nt N_C4, 2
        nt N_E4, 2
        nt N_G4, 2
        nt N_C5, 2
        nt N_E5, 2
        nt N_G5, 4
        .byte $ff
snd_destroy:
        .byte $80, $00, $f6
        nt 60, 1
        nt 52, 1
        nt 58, 1
        nt 46, 2
        nt 40, 2
        nt 30, 3
        .byte $ff
snd_destroy1:
        .byte $20, $00, $a6
        nt N_G4, 2
        nt N_E4, 2
        nt N_C4, 2
        nt N_G3, 3
        .byte $ff
snd_hit:
        .byte $80, $00, $f9
        nt 45, 4
        nt 38, 4
        nt 32, 5
        nt 26, 6
        .byte $ff
snd_hit1:
        .byte $20, $00, $f8
        nt N_G3, 4
        nt N_E3, 4
        nt N_C3, 5
        nt N_A2, 6
        .byte $ff
snd_clear2:
        .byte $40, $09, $a8
        nt N_C5, 4
        nt N_E5, 4
        nt N_G5, 4
        nt N_C6, 8
        nt N_G5, 3
        nt N_C6, 12
        .byte $ff
snd_clear1:
        .byte $40, $09, $a8
        nt N_C4, 4
        nt N_E4, 4
        nt N_G4, 4
        nt N_C5, 8
        nt N_G4, 3
        nt N_C5, 12
        .byte $ff
snd_clear0:
        .byte $10, $08, $f8
        nt N_C3, 12
        nt N_G3, 11
        nt N_C3, 12
        .byte $ff
snd_over2:
        .byte $20, $09, $a8
        nt N_G5, 8
        nt N_D5, 8
        nt N_A4, 16
        .byte $ff
snd_over0:
        .byte $10, $08, $f8
        nt N_E3, 16
        nt N_A2, 16
        .byte $ff
snd_bonus:
        .byte $40, $08, $a4
        nt N_C6, 2
        nt N_E6, 2
        nt N_G6, 2
        nt N_C7, 4
        .byte $ff
snd_ice:
        .byte $10, $08, $a8
        nt N_G6, 2
        nt N_C6, 2
        nt N_G6, 2
        nt N_C6, 2
        nt N_E6, 2
        nt N_A5, 2
        nt N_E5, 6
        .byte $ff

; Effekt -> Stroeme je Stimme (Stimme 0, 1, 2; 0 = keiner), Prioritaet
EFX_STREAM:
        .word 0, 0, snd_step            ; step
        .word 0, 0, snd_seg             ; seg
        .word 0, 0, snd_box             ; box
        .word snd_chain0, snd_chain1, snd_chain ; chain
        .word 0, snd_hit1, snd_hit      ; hit
        .word snd_clear0, snd_clear1, snd_clear2 ; clear
        .word snd_over0, 0, snd_over2   ; over
        .word 0, 0, snd_bonus           ; bonus
        .word 0, 0, snd_ice             ; ice
        .word 0, snd_destroy1, snd_destroy ; destroy
EFX_PRIO:
        .byte 1, 2, 3, 4, 5, 6, 5, 3, 3, 4

; ---- Titelmusik (a-Moll, Folge Am F C G; 32 Schritte zu 6 Bildern, wird wiederholt): Stimme 0 Bass, 1 Arpeggio, 2 Melodie
MUSIC_LO: .byte <mus_bass, <mus_arp, <mus_mel
MUSIC_HI: .byte >mus_bass, >mus_arp, >mus_mel
.macro bar_bass(r)
        nt r, 6
        nt r, 6
        nt r + 12, 6
        nt r, 6
        nt r, 6
        nt r, 6
        nt r + 12, 6
        nt r, 6
.endmacro
.macro bar_arp(a, b, c)
        nt a, 6
        nt b, 6
        nt c, 6
        nt b, 6
        nt a, 6
        nt b, 6
        nt c, 6
        nt b, 6
.endmacro
mus_bass:
        .byte $10, $09, $a8
        bar_bass N_A2 + 0 - 0
        bar_bass N_F2
        bar_bass N_C3
        bar_bass N_G2
        .byte $fe
mus_arp:
        .byte $40, $06, $00
        bar_arp N_A3, N_C4, N_E4
        bar_arp N_F3, N_A3, N_C4
        bar_arp N_C4, N_E4, N_G4
        bar_arp N_G3, N_B3, N_D4
        .byte $fe
mus_mel:
        .byte $20, $08, $98
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
