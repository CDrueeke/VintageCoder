; =====================================================================================================================
; sound.asm - Klangmodul: Effekte mit Prioritaeten, ein Aufruf je Bild
; =====================================================================================================================
; ZWECK     Spielt Effekte aus Listen (SND_DATA).  Eine Liste besteht aus Eintraegen zu drei Byte: Register (Abstand von $9000), Wert, Dauer in
;           Bildern; Register 0 beendet die Liste und schaltet alle Stimmen aus.  Wert 0 schaltet die Stimme aus.  Bei Bass, Alt und Sopran ist der Wert eine Notennummer 1..8
;           (C5 bis C6 der Sopranstimme; Bass klingt zwei, Alt eine Oktave tiefer); das Modul setzt daraus den Registerwert der Norm
;           (SCALE, Bit 7 = Stimme ein).  Beim Rauschen ($900D) gilt der Wert unveraendert.
; BEDIENUNG sound.init einmal; sound.play mit A = Effektnummer (Fx.*); sound.update einmal je Bild.  Ein neuer Effekt ersetzt den laufenden nur,
;           wenn seine Prioritaet groesser ist.
; HARDWARE  VIC-I $900A Bass, $900B Alt, $900C Sopran, $900D Rauschen (je Bit 7 = ein, Bit 0-6 = Teiler), $900E Lautstaerke 0..15 (Bit 0-3)
;           und Hilfsfarbe (Bit 4-7).
; =====================================================================================================================

.enum Fx
    step                                ; kurzes Klicken bei jedem Schritt (Prioritaet 1)
    seg                                 ; Segment abgeschlossen: zwei steigende Toene (2)
    box                                 ; Rechteck gefuellt: zwei Toene (3)
    chain                               ; Kettenreaktion: drei Toene (4)
    hit                                 ; Treffer: fallendes Rauschen (5)
    clear                               ; Stufe geschafft: Fanfare (6)
    over                                ; Spielende: fallende Toene (5)
    destroy                             ; Linie zerstoert: kurz fallender Ton (3)
.endenum

.macro snd(reg, val, dur)
        .byte (reg) - VIC_ORIGIN_X, val, dur
.endmacro

sound: {

; init: Lautstaerke 15, Stimmen aus, Zustand loeschen, Notenversatz je Norm.  Veraendert A, X.
init:
        lda #$1f
        sta VIC_VOLUME
        jsr silence
        sta SND
        sta SND + 1
        sta SND + 3
        lda FLAGS
        and #F_PAL
        eor #F_PAL                      ; PAL: 0, NTSC: 1
        asl
        asl
        asl                             ; PAL: 0, NTSC: 8 (zweite Reihe in SCALE)
        sta SND + 2
        rts

; silence: alle Stimmen aus.  A = 0 danach.  Veraendert A, X.
silence:
        lda #0
        ldx #3
.mute:  sta VIC_BASS,x                  ; $900A..$900D
        dex
        bpl .mute
        rts

; play: A = Effektnummer.  Veraendert A, X.
play:   tax
        lda SND + 3
        cmp EFF_PRIO,x
        bcs .skip                       ; laufender Effekt gleich wichtig oder wichtiger: ignorieren
        lda EFF_PRIO,x
        sta SND + 3
        lda EFF_START,x
        sta SND
        lda #1                          ; im naechsten update den ersten Eintrag lesen
        sta SND + 1
.skip:  rts

; update: einmal je Bild.  Veraendert A, X, Y.
update:
        lda SND + 3
        beq .done                       ; ruhig
        dec SND + 1
        bne .done                       ; Eintrag laeuft noch
        ldx SND
        ldy SND_DATA,x                  ; Register (Abstand von $9000)
        beq .stop                       ; Ende der Liste
        lda SND_DATA + 1,x              ; Wert
        beq .write                      ; 0 = aus
        cpy #VIC_NOISE - VIC_ORIGIN_X
        beq .write                      ; Rauschen: Wert roh
        clc
        adc SND + 2                     ; Notennummer 1..8 (NTSC: + 8) -> Tonwert
        tax
        lda SCALE - 1,x
.write: sta VIC_ORIGIN_X,y              ; $9000 + Register
        ldx SND
        lda SND_DATA + 2,x
        sta SND + 1                     ; Dauer
        txa
        clc
        adc #3
        sta SND
.done:  rts
.stop:  lda #0
        sta SND + 3
        jmp silence
}

; ---- Effektlisten
SND_DATA:
snd_step:
        snd VIC_NOISE, $f0, 1           ; ein Bild hohes Rauschen
        .byte 0
snd_seg:
        snd VIC_SOPRANO, 3, 3
        snd VIC_SOPRANO, 5, 3
        .byte 0
snd_box:
        snd VIC_SOPRANO, 5, 4
        snd VIC_SOPRANO, 8, 5
        .byte 0
snd_chain:
        snd VIC_SOPRANO, 3, 3
        snd VIC_SOPRANO, 5, 3
        snd VIC_SOPRANO, 8, 6
        .byte 0
snd_hit:
        snd VIC_NOISE, $f0, 5
        snd VIC_NOISE, $e0, 6
        snd VIC_NOISE, $c0, 8
        .byte 0
snd_destroy:
        snd VIC_ALTO, 6, 2
        snd VIC_ALTO, 4, 2
        snd VIC_ALTO, 2, 3
        .byte 0
snd_clear:
        snd VIC_SOPRANO, 3, 6
        snd VIC_SOPRANO, 8, 20
        .byte 0
snd_over:
        snd VIC_SOPRANO, 5, 8
        snd VIC_SOPRANO, 3, 8
        snd VIC_SOPRANO, 1, 16
        .byte 0
EFF_START:
        .byte snd_step - SND_DATA, snd_seg - SND_DATA, snd_box - SND_DATA, snd_chain - SND_DATA
        .byte snd_hit - SND_DATA, snd_clear - SND_DATA, snd_over - SND_DATA, snd_destroy - SND_DATA
EFF_PRIO:
        .byte 1, 2, 3, 4, 5, 6, 5, 3
