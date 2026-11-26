; =====================================================================================================================
; sound.asm - Klangmodul (4 SID): Effekte mit Prioritaeten, Titelmusik, zwoelf Stimmen, ein Aufruf je Bild
; =====================================================================================================================
; Herkunft: examples/c64/troxx/sound.asm (Stand Commit 4697d074 der Referenz; Schnittstelle gleich: init, play, update, silence, Fx.*); der Klangmodul ist
;   systemspezifisch (hier vier SID-Chips des MEGA65) und daher nicht in der Kopienliste.
; ZWECK     Jede der zwoelf SID-Stimmen spielt einen Strom (Stream): Kopf aus drei Byte (Wellenform ohne Gate, Attack/Decay, Sustain/Release), dann
;           Eintraege zu zwei Byte (Notennummer, Dauer in Bildern); Note 0 = Pause (Gate aus), $FF = Ende, $FE = von vorn (Musik).  Stimmen 0..5 (SID 1 und 2)
;           spielen die Titelmusik (sechs Stimmen: Bass, Arpeggio, Melodie, Begleitung, Oberstimme, Schlagwerk), Stimmen 6..11 (SID 3 und 4) die Effekte;
;           ein Effekt besteht aus bis zu sechs Stroemen (Tabelle EFX_STREAM) und ersetzt einen laufenden Strom nur, wenn seine Prioritaet groesser ist.
;           Waehrend der Titelmusik sind die Effekte gesperrt, bis music_stop aufgerufen wird; die Hintergrundmusik im Spiel gibt es nicht (ruhiges Spiel).
; BEDIENUNG sound.init einmal; sound.play mit A = Effektnummer (Fx.*; veraendert nur A, X, Y, nicht T0..T7); sound.update einmal je Bild
;           (veraendert A, X, Y); music_start / music_stop fuer die Titelmusik; silence stoppt alles.
; HARDWARE  SID 1 $D400, SID 2 $D420, SID 3 $D440, SID 4 $D460 (der MEGA65 hat vier SID-Chips); je Stimme 7 Register (Frequenz lo/hi, Pulsbreite lo/hi,
;           Steuerung, AD, SR), $D418 + 32 n Lautstaerke.  Steuerung: Bit 0 Gate, Bit 4 Dreieck, 5 Saege, 6 Rechteck, 7 Rauschen.  Frequenzwert =
;           Tonhoehe * 2^24 / Takt (PAL 985248 Hz, NTSC 1022727 Hz); Tabellen FLO/FHI fuer 64 Noten C2..D7 je Norm (Notennummer n = MIDI-Nummer - 35).
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
    destroy                             ; Linie zerstoert: Knistern (4)
.endenum

.const NVOICE   = 12                    ; Stimmen: 0..5 Musik, 6..11 Effekte

; Zustand im RAM ab SND (je Stimme ein Byte; Stimmen 0..11)
.const S_PTRLO  = SND                   ; +0..11 Leseposition (niedrig)
.const S_PTRHI  = SND + 12              ; +12..23 (hoch)
.const S_TIME   = SND + 24              ; +24..35 Bilder bis zum naechsten Eintrag
.const S_PRIO   = SND + 36              ; +36..47 Prioritaet des laufenden Stroms (0 = ruhig)
.const S_BASELO = SND + 48              ; +48..59 Anfang der Eintraege (fuer $FE)
.const S_BASEHI = SND + 60              ; +60..71
.const S_WAVE   = SND + 72              ; +72..83 Wellenform ohne Gate
.const S_NORM   = SND + 84              ; 0 (PAL) oder 64 (NTSC): Versatz in den Frequenztabellen
.const S_MUSIC  = SND + 85              ; 1 = Titelmusik laeuft
.const S_N      = SND + 86              ; Notennummer (Hilfe)
.const S_V      = SND + 87              ; Stimme (Hilfe)
.const S_P      = SND + 88              ; Prioritaet des Effekts (Hilfe)
.const S_LO     = SND + 89              ; Frequenz niedrig (Hilfe)
.const S_E      = SND + 90              ; Effektnummer (Hilfe)
.const S_B      = SND + 91              ; Abstand in EFX_STREAM (Hilfe)
.const S_F      = SND + 92              ; Zaehler des Filterlaufs (Titelmusik)
.const SPTR     = $7f                   ; Zeropage: Zeiger in den Strom (2 Byte)
.const SID1     = $d400
.const SID_CTRL = 4                     ; Registerabstaende in der Stimme
.const SID_AD   = 5
.const SID_SR   = 6

sound: {

; init: Lautstaerke 15 auf allen vier SID, Pulsbreite, Zustand loeschen, Notenversatz je Norm.  Veraendert A, X.
init:
        lda #0
        ldx #95
.clr:   sta SND,x
        dex
        bpl .clr
        ldx #3
.chip:  lda CHIPOFF,x
        tay
        lda #$0f
        sta SID1 + $18,y                ; Lautstaerke
        lda #0
        sta SID1 + $17,y                ; Filter aus
        lda #$08
        sta SID1 + 3,y                  ; Pulsbreite $0800 (Rechteck) fuer alle Stimmen
        sta SID1 + 10,y
        sta SID1 + 17,y
        dex
        bpl .chip
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
        ldx #3
.f:     ldy CHIPOFF,x
        lda #0
        sta SID1 + $17,y                ; Filter aus
        lda #$0f
        sta SID1 + $18,y
        dex
        bpl .f
        ldx #NVOICE - 1
.mute:  lda #0
        sta S_PRIO,x
        ldy VOFF,x
        sta SID1 + SID_CTRL,y
        dex
        bpl .mute
        rts

; start: Strom ab SPTR auf Stimme X mit Prioritaet A starten (Kopf lesen, Gate aus).  Veraendert A, Y.
start:  sta S_PRIO,x
        ldy #0
        lda (SPTR),y
        sta S_WAVE,x
        ldy VOFF,x                      ; Y = Registerabstand der Stimme
        sta SID1 + SID_CTRL,y           ; Gate aus (Neuanschlag im naechsten Bild)
        stx S_V
        ldy #1
        lda (SPTR),y
        pha
        iny
        lda (SPTR),y
        pha
        ldy VOFF,x
        pla
        sta SID1 + SID_SR,y
        pla
        sta SID1 + SID_AD,y
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
        sta S_B                         ; Abstand in EFX_STREAM = Effekt * 6 Stroeme * 2 Byte = Effekt * 12: hier mal 6, dann Stimme * 2
        lda S_B
        asl
        sta S_B
        ldx #5                          ; Effektstimme 5 .. 0 (= Stimme 11 .. 6)
.v:     txa
        asl
        clc
        adc S_B
        tay
        lda EFX_STREAM + 1,y
        beq .nx                         ; kein Strom fuer diese Stimme
        lda S_PRIO + 6,x
        cmp S_P
        bcs .nx                         ; laeuft etwas Gleichwichtiges oder Wichtigeres: ignorieren
        lda EFX_STREAM,y
        sta SPTR
        lda EFX_STREAM + 1,y
        sta SPTR + 1
        txa
        clc
        adc #6
        tax
        lda S_P
        jsr start
        txa
        sec
        sbc #6
        tax
.nx:    dex
        bpl .v
.skip:  rts

; music_start: Titelmusik auf den Stimmen 0..5 starten (Prioritaet 255, Effekte gesperrt).  Veraendert A, X, Y.
music_start:
        jsr silence
        lda #1
        sta S_MUSIC
        lda #$f1                        ; Tiefpassfilter (Resonanz 15) auf Stimme 1 von SID 1, Grenzfrequenz laeuft in update hin und her
        sta SID1 + $17
        lda #$1f
        sta SID1 + $18
        ldx #5
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
.if FX_TITLE
        pha
        lsr
        lsr
        lsr
        ora #1
        sta SID1 + 10                   ; Pulsbreitenmodulation der Begleitstimme (Stimme 2)
        pla
.endif
        clc
        adc #12
        sta SID1 + $16                  ; Grenzfrequenz hoch
.nf:    ldx #NVOICE - 1
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
        sta SID1 + 1,y                  ; Frequenz hoch
        lda S_LO
        sta SID1,y                      ; Frequenz niedrig
        lda S_WAVE,x
        ora #1                          ; Gate an
        sta SID1 + SID_CTRL,y
        jmp .nx
.rest:  lda S_WAVE,x                    ; Pause: Gate aus
        sta SID1 + SID_CTRL,y
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
        ldy VOFF,x
        lda S_WAVE,x
        sta SID1 + SID_CTRL,y           ; Gate aus (Release klingt aus)
        jmp .nx
}

; Registerabstand der Stimme (Stimme 0..11 = SID n = v div 3, Stimme v mod 3 im Chip) und der Chips
VOFF:   .for v in range(12)
    .byte (v / 3) * 32 + (v % 3) * 7
.endfor
CHIPOFF: .byte 0, 32, 64, 96

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
snd_box1:
        .byte $20, $08, $a6
        nt N_C5, 4
        nt N_G5, 5
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
snd_hit2:
        .byte $40, $00, $f8
        nt N_C3, 4
        nt N_A2, 4
        nt N_F2, 10
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
snd_clear3:
        .byte $40, $08, $a8
        nt N_E6, 4
        nt N_G6, 4
        nt N_C7, 8
        nt N_G6, 3
        nt N_C7, 12
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
snd_destroy:
        .byte $80, $00, $a9
        nt 50, 1
        nt 44, 1
        nt 38, 2
        nt 30, 3
        .byte $ff
snd_destroy1:
        .byte $20, $00, $a8
        nt N_A3, 2
        nt N_F3, 2
        nt N_D3, 4
        .byte $ff

; Effekt -> Stroeme je Effektstimme (Stimmen 6..11; 0 = keiner), Prioritaet.  Je Effekt sechs Woerter.
EFX_STREAM:
        .word 0, 0, snd_step, 0, 0, 0                                   ; step
        .word 0, 0, snd_seg, 0, 0, 0                                    ; seg
        .word 0, 0, snd_box, 0, 0, snd_box1                             ; box
        .word snd_chain0, snd_chain1, snd_chain, 0, 0, 0                ; chain
        .word 0, snd_hit1, snd_hit, 0, 0, snd_hit2                      ; hit
        .word snd_clear0, snd_clear1, snd_clear2, 0, 0, snd_clear3      ; clear
        .word snd_over0, 0, snd_over2, 0, 0, 0                          ; over
        .word 0, 0, snd_bonus, 0, 0, 0                                  ; bonus
        .word 0, 0, snd_ice, 0, 0, 0                                    ; ice
        .word 0, snd_destroy1, snd_destroy, 0, 0, 0                     ; destroy
EFX_PRIO:
        .byte 1, 2, 3, 4, 5, 6, 5, 3, 3, 4

; ---- Titelmusik (a-Moll, Folge Am F C G; 32 Schritte zu 6 Bildern, wird wiederholt): Stimme 0 Bass, 1 Arpeggio, 2 Melodie (SID 1);
;      3 Begleitung (Akkordgrundton), 4 Oberstimme (Arpeggio eine Oktave hoeher, versetzt), 5 Schlagwerk (Rauschen) (SID 2)
MUSIC_LO: .byte <mus_bass, <mus_arp, <mus_mel, <mus_pad, <mus_arp2, <mus_drum
MUSIC_HI: .byte >mus_bass, >mus_arp, >mus_mel, >mus_pad, >mus_arp2, >mus_drum
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
.macro bar_pad(r)
        nt r, 48
.endmacro
.macro bar_drum
        nt 40, 3
        nt 0, 9
        nt 30, 3
        nt 0, 9
        nt 40, 3
        nt 0, 3
        nt 36, 3
        nt 0, 3
        nt 30, 3
        nt 0, 9
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
mus_pad:
        .byte $40, $4a, $a8
        bar_pad N_A3
        bar_pad N_F3
        bar_pad N_C4
        bar_pad N_G3
        .byte $fe
mus_arp2:
        .byte $20, $05, $60
        bar_arp N_A4, N_C5, N_E5
        bar_arp N_F4, N_A4, N_C5
        bar_arp N_C5, N_E5, N_G5
        bar_arp N_G4, N_B4, N_D5
        .byte $fe
mus_drum:
        .byte $80, $00, $f4
        bar_drum
        bar_drum
        bar_drum
        bar_drum
        .byte $fe

