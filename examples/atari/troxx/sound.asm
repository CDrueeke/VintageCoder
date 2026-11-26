; =====================================================================================================================
; sound.asm - Klangmodul (POKEY): Effekte mit Prioritaeten, Titelmusik, vier Stimmen, ein Aufruf je Bild
; =====================================================================================================================
; Herkunft: gleiche Schnittstelle wie examples/c64/troxx/sound.asm (Stand Commit d173cb4a): init, play, update, silence, music_start, music_stop, Fx.*; der
;   Klangmodul ist Atari-spezifisch (POKEY statt SID) und daher nicht in der Kopienliste.
; ZWECK     Jede der vier POKEY-Stimmen (Kanal 1..4) spielt einen Strom: Kopf aus drei Byte, dann Eintraege zu zwei Byte (Notennummer, Dauer in Bildern);
;           Note 0 = Pause, $FF = Ende, $FE = von vorn (Musik).  Kopf: Byte 0 = AUDC-Wert beim Anschlag (Verzerrung in Bit 5..7, Anfangslautstaerke in Bit
;           0..3; $A0 reiner Ton, $80 Rauschen, $C0 4-Bit-Poly), Byte 1 = Abklingen: alle n Bilder ein Lautstaerkeschritt (0 = haelt), Byte 2 = Sockel (Lautstaerke,
;           bis zu der es abklingt).  Ein POKEY hat keine Huellkurven; das Abklingen ersetzt sie.  Ein Effekt besteht aus bis zu vier Stroemen (EFX_STREAM);
;           er ersetzt einen laufenden Strom nur, wenn seine Prioritaet groesser ist.  Die Titelmusik belegt alle vier Stimmen (Prioritaet 255) und sperrt
;           die Effekte, bis music_stop aufgerufen wird.
; STIMMEN DER TITELMUSIK  Stimme 0 (Kanal 1) Melodie, 1 (Kanal 2) Begleitung als gebrochener Dreiklang, 2 (Kanal 3) Bass (eine Oktave hoeher notiert, weil der
;           8-Bit-Teiler bei 64 kHz nicht unter 125 Hz kommt), 3 (Kanal 4) Schlagzeug aus Rauschen.  FX_VIB schaltet ein leichtes Vibrato der Melodie ein.
; HARDWARE  POKEY $D200: AUDFn (Teiler), AUDCn (Verzerrung + Lautstaerke), AUDCTL = 0 (64 kHz, Kanaele einzeln), SKCTL = 3.  Tonhoehe f = Takt / (2 * (n + 1)) mit
;           Takt 63337 Hz (PAL) bzw. 63921 Hz (NTSC); Tabelle DIVT fuer 64 Noten C2..D7 je Norm (Notennummer n = MIDI-Nummer - 35).
; =====================================================================================================================

.enum Fx
    step                                ; kurzes Klicken bei jedem Schritt (Prioritaet 1)
    seg                                 ; Segment abgeschlossen: zwei steigende Toene (2)
    box                                 ; Rechteck gefuellt: zwei Toene (3)
    chain                               ; Kettenreaktion: schnelle Tonleiter, rauer Klang, Rauschstoss (4)
    hit                                 ; Treffer: fallendes Rauschen (5)
    clear                               ; Stufe geschafft: Fanfare (6)
    over                                ; Spielende: fallende Toene (5)
    bonus                               ; Stern aufgenommen (3)
    ice                                 ; Eisblock aufgenommen (3)
    destroy                             ; Linie zerstoert: Knistern und fallender Ton (3)
.endenum

; Zustand im RAM ab SND (je Stimme ein Byte, Stimmen 0..3)
.const S_PTRLO  = SND                   ; +0..3 Leseposition (niedrig)
.const S_PTRHI  = SND + 4               ; +4..7 (hoch)
.const S_TIME   = SND + 8               ; +8..11 Bilder bis zum naechsten Eintrag
.const S_PRIO   = SND + 12              ; +12..15 Prioritaet des laufenden Stroms (0 = ruhig)
.const S_BASELO = SND + 16              ; +16..19 Anfang der Eintraege (fuer $FE)
.const S_BASEHI = SND + 20              ; +20..23
.const S_AUDC   = SND + 24              ; +24..27 laufender AUDC-Wert
.const S_INIT   = SND + 28              ; +28..31 AUDC-Wert beim Anschlag
.const S_RATE   = SND + 32              ; +32..35 Abklingen: Bilder je Schritt
.const S_FLOOR  = SND + 36              ; +36..39 Sockel
.const S_CNT    = SND + 40              ; +40..43 Zaehler des Abklingens
.const S_NORM   = SND + 44              ; 0 (PAL) oder 64 (NTSC): Versatz in der Teilertabelle
.const S_MUSIC  = SND + 45              ; 1 = Titelmusik laeuft
.const S_N      = SND + 46              ; Notennummer (Hilfe)
.const S_V      = SND + 47              ; Stimme (Hilfe)
.const S_P      = SND + 48              ; Prioritaet des Effekts (Hilfe)
.const S_E      = SND + 49              ; Effektnummer (Hilfe)
.const S_B      = SND + 50              ; Abstand in EFX_STREAM (Hilfe)
.const S_F      = SND + 51              ; Zaehler (Vibrato)
.const S_DIV    = SND + 52              ; Teiler der Melodie (fuer das Vibrato)
.const S_SND    = SND + 53              ; 1 = Melodie klingt
.const SPTR     = $f8                   ; Zeropage: Zeiger in den Strom (2 Byte)

sound: {

; init: Kanaele einzeln (AUDCTL = 0), SKCTL = 3, Zustand loeschen, Teilerversatz je Norm.  Veraendert A, X.
init:
        lda #0
        sta AUDCTL
        lda #3
        sta SKCTL
        lda #0
        ldx #63
.clr:   sta SND,x
        dex
        bpl .clr
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
        sta S_SND
        sta AUDCTL
        ldx #3
.mute:  sta S_PRIO,x
        sta S_AUDC,x
        dex
        bpl .mute
        ldx #6
.m2:    sta AUDC1,x
        dex
        dex
        bpl .m2
        rts

; start: Strom ab SPTR auf Stimme X mit Prioritaet A starten (Kopf lesen, stumm schalten).  Veraendert A, Y.
start:  sta S_PRIO,x
        ldy #0
        lda (SPTR),y
        sta S_INIT,x
        iny
        lda (SPTR),y
        sta S_RATE,x
        iny
        lda (SPTR),y
        sta S_FLOOR,x
        lda S_INIT,x
        and #$e0
        sta S_AUDC,x                    ; stumm (Lautstaerke 0), Verzerrung gesetzt
        stx S_V
        lda VOFF,x
        tay
        lda S_AUDC,x
        sta AUDC1,y
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
        asl
        asl
        sta S_B                         ; Abstand in EFX_STREAM = Effekt * 8
        ldx #3                          ; Stimme 3 .. 0
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

; music_start: Titelmusik auf allen vier Stimmen starten (Prioritaet 255, Effekte gesperrt).  Veraendert A, X, Y.
music_start:
        jsr silence
        lda #1
        sta S_MUSIC
        ldx #3
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
update:
.if FX_VIB
        lda S_SND
        beq .nv
        inc S_F
        lda S_F
        and #8
        beq .v0                         ; Vibrato: Teiler der Melodie alle 8 Bilder um 1 versetzt
        lda S_DIV
        clc
        adc #1
        jmp .vw
.v0:    lda S_DIV
.vw:    sta AUDF1
.nv:
.endif
        ldx #3
.v:     lda S_PRIO,x
        bne .act
        jmp .nx
.act:   lda S_RATE,x                    ; Abklingen
        beq .noenv
        dec S_CNT,x
        bne .noenv
        sta S_CNT,x
        lda S_AUDC,x
        and #15
        cmp S_FLOOR,x
        bcc .noenv
        beq .noenv
        dec S_AUDC,x
        ldy VOFF,x
        lda S_AUDC,x
        sta AUDC1,y
.noenv: dec S_TIME,x
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
        ldy VOFF,x                      ; Y = Registerabstand der Stimme
        lda S_N
        beq .rest
        clc
        adc S_NORM
        tax
        lda DIVT - 1,x                  ; Teiler
        ldx S_V
        sta AUDF1,y
        cpx #0
        bne .nm
        sta S_DIV
        lda S_MUSIC
        sta S_SND
.nm:    lda S_INIT,x                    ; Anschlag: volle Lautstaerke
        sta S_AUDC,x
        sta AUDC1,y
        lda S_RATE,x
        sta S_CNT,x
        jmp .nx
.rest:  lda S_INIT,x                    ; Pause: Lautstaerke 0
        and #$e0
        sta S_AUDC,x
        sta AUDC1,y
        jmp .nx
.loop:  lda S_BASELO,x                  ; Musik: von vorn
        sta SPTR
        lda S_BASEHI,x
        sta SPTR + 1
        jmp .read
.stop:  lda #0
        sta S_PRIO,x
        sta S_AUDC,x
        ldy VOFF,x
        sta AUDC1,y                     ; stumm
.nx:    dex
        bmi .end
        jmp .v
.end:   rts
}

VOFF: .byte 0, 2, 4, 6

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

; ---- Teilertabelle: 64 Noten je Norm (erst PAL, dann NTSC); Tonhoehe = 440 Hz * 2^((MIDI - 69) / 12), Teiler = Takt / (2 f) - 1, auf 1..255 begrenzt
.var pclocks = [1773447 / 28, 1789773 / 28]
DIVT:
.for c in pclocks
    .for n in range(64)
        .byte min(255, max(1, round(c / (2 * 440 * pow(2, (n + 36 - 69) / 12)) - 1)))
    .endfor
.endfor

; ---- Effekte: Stroeme (Kopf: AUDC beim Anschlag, Abklingen, Sockel; dann Note/Dauer; $FF = Ende)
snd_step:
        .byte $80 + 6, 1, 0
        nt 50, 1
        .byte $ff
snd_seg:
        .byte $a0 + 10, 3, 2
        nt N_C5, 3
        nt N_E5, 3
        .byte $ff
snd_box:
        .byte $a0 + 11, 4, 2
        nt N_E5, 4
        nt N_C6, 6
        .byte $ff
snd_chain:                              ; Stimme 0: hohe Tonleiter, rau ($C0)
        .byte $c0 + 12, 3, 3
        nt N_C5, 2
        nt N_E5, 2
        nt N_G5, 2
        nt N_C6, 2
        nt N_E6, 2
        nt N_G6, 6
        .byte $ff
snd_chain1:                             ; Stimme 1: dieselbe Tonleiter eine Terz tiefer, versetzt
        .byte $a0 + 9, 3, 2
        nt 0, 1
        nt N_A4, 2
        nt N_C5, 2
        nt N_E5, 2
        nt N_A5, 2
        nt N_C6, 2
        nt N_E6, 5
        .byte $ff
snd_chain2:                             ; Stimme 2: tiefe Quinte, Poly
        .byte $c0 + 11, 4, 3
        nt N_C4, 4
        nt N_G4, 4
        nt N_C5, 4
        nt N_G5, 6
        .byte $ff
snd_chain3:                             ; Stimme 3: Rauschstoss
        .byte $80 + 10, 2, 0
        nt 30, 4
        nt 40, 4
        nt 50, 4
        nt 58, 6
        .byte $ff
snd_hit:
        .byte $80 + 14, 3, 0
        nt 45, 4
        nt 38, 4
        nt 32, 5
        nt 26, 8
        .byte $ff
snd_hit1:
        .byte $a0 + 12, 4, 0
        nt N_G3, 5
        nt N_E3, 5
        nt N_C3, 6
        nt N_A2 + 2, 8
        .byte $ff
snd_clear2:
        .byte $a0 + 10, 6, 5
        nt N_C5, 4
        nt N_E5, 4
        nt N_G5, 4
        nt N_C6, 8
        nt N_G5, 3
        nt N_C6, 14
        .byte $ff
snd_clear1:
        .byte $a0 + 8, 6, 4
        nt N_C4, 4
        nt N_E4, 4
        nt N_G4, 4
        nt N_C5, 8
        nt N_G4, 3
        nt N_C5, 14
        .byte $ff
snd_clear0:
        .byte $a0 + 10, 0, 0
        nt N_C3 + 12, 12
        nt N_G3 + 12, 11
        nt N_C3 + 12, 14
        .byte $ff
snd_clear3:
        .byte $80 + 8, 1, 0
        nt 1, 4
        nt 58, 4
        nt 1, 4
        nt 58, 4
        nt 1, 4
        nt 58, 4
        nt 1, 8
        .byte $ff
snd_over2:
        .byte $a0 + 10, 7, 4
        nt N_G5, 8
        nt N_D5, 8
        nt N_A4, 18
        .byte $ff
snd_over0:
        .byte $a0 + 10, 0, 0
        nt N_E3 + 2, 16
        nt N_A2 + 12, 18
        .byte $ff
snd_bonus:
        .byte $a0 + 11, 2, 3
        nt N_C6, 2
        nt N_E6, 2
        nt N_G6, 2
        nt N_C7, 5
        .byte $ff
snd_ice:
        .byte $a0 + 10, 2, 2
        nt N_G6, 2
        nt N_C6, 2
        nt N_G6, 2
        nt N_C6, 2
        nt N_E6, 2
        nt N_A5, 2
        nt N_E5, 6
        .byte $ff

snd_destroy1:                           ; Stimme 1: fallender rauer Ton
        .byte $c0 + 11, 2, 2
        nt N_A5, 2
        nt N_E5, 2
        nt N_B4, 2
        nt N_F4, 3
        .byte $ff
snd_destroy3:                           ; Stimme 3: Knistern (schnell wechselnde Rauschteiler)
        .byte $80 + 12, 1, 3
        nt 58, 1
        nt 36, 1
        nt 52, 1
        nt 30, 1
        nt 56, 1
        nt 40, 1
        nt 50, 1
        nt 26, 2
        .byte $ff

; Effekt -> Stroeme je Stimme (Stimme 0, 1, 2, 3; 0 = keiner); Prioritaet in EFX_PRIO
EFX_STREAM:
        .word 0, 0, 0, snd_step                     ; step
        .word snd_seg, 0, 0, 0                      ; seg
        .word snd_box, 0, 0, 0                      ; box
        .word snd_chain, snd_chain1, snd_chain2, snd_chain3   ; chain
        .word 0, 0, snd_hit1, snd_hit               ; hit
        .word snd_clear2, snd_clear1, snd_clear0, snd_clear3  ; clear
        .word snd_over2, 0, snd_over0, 0            ; over
        .word snd_bonus, 0, 0, 0                    ; bonus
        .word snd_ice, 0, 0, 0                      ; ice
        .word 0, snd_destroy1, 0, snd_destroy3      ; destroy
EFX_PRIO:
        .byte 1, 2, 3, 4, 5, 6, 5, 3, 3, 3

; ---- Titelmusik (a-Moll, Folge Am F C G; 32 Schritte zu 6 Bildern, wird wiederholt): Stimme 0 Melodie, 1 Arpeggio, 2 Bass, 3 Schlagzeug
MUSIC_LO: .byte <mus_mel, <mus_arp, <mus_bass, <mus_drum
MUSIC_HI: .byte >mus_mel, >mus_arp, >mus_bass, >mus_drum
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
.macro bar_drum()
        nt 1, 6
        nt 58, 6
        nt 44, 6
        nt 58, 6
        nt 1, 6
        nt 58, 6
        nt 44, 6
        nt 58, 6
.endmacro
mus_bass:
        .byte $a0 + 11, 0, 0
        bar_bass N_A2 + 12
        bar_bass N_F2 + 12
        bar_bass N_C3 + 12
        bar_bass N_G2 + 12
        .byte $fe
mus_arp:
        .byte $a0 + 8, 4, 2
        bar_arp N_A3, N_C4, N_E4
        bar_arp N_F3, N_A3, N_C4
        bar_arp N_C4, N_E4, N_G4
        bar_arp N_G3, N_B3, N_D4
        .byte $fe
mus_mel:
        .byte $a0 + 10, 12, 6
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
mus_drum:
        .byte $80 + 7, 1, 0
        bar_drum
        bar_drum
        bar_drum
        bar_drum
        .byte $fe
