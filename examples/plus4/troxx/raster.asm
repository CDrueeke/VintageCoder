; =====================================================================================================================
; raster.asm - Rasterinterrupt: Bildtakt, wandernde Farbbalken im Rand, Farbbaender im Bild, Bildlauf im Titelbild
; =====================================================================================================================
; ZWECK     Der TED loest in einer vorgegebenen Rasterzeile einen Interrupt aus; der Handler fuehrt einen Eintrag einer Tabelle aus und stellt die naechste
;           Zeile ein.  Die Tabelle (je Norm eine) laeuft einmal je Bild und beginnt in Zeile 204 (unter dem Bild, das die Zeilen 4..203 belegt).
;           Eintragsarten:
;             BAR    Rahmenfarbe auf Balken ARG (Farbverlauf GRAD, laeuft mit RB_VAR + 1 weiter): wandernde Balken im unteren (Zeile 204..) und oberen Rand
;             FRAME  wie BAR fuer den Balken 0, dazu FRAMEC + 1 (Bildtakt, auf den ted.wait_frame wartet)
;             BASE   feste Rahmenfarbe; im Spiel auch Hintergrund 0 zurueck auf die Grundfarbe
;             GBAND  Spiel: Mehrfarben-Hintergrund 1 und 2 der Rechteckzeile ARG (Schatten und Glanz der Fuellungen im Ton der Zeile; FX_SHADE)
;             TBAND  Titelbild: Hintergrund 0 auf das Hintergrundband ARG (Farbbaender hinter dem Logo)
;             SON    Titelbild: Bildlauf an (38 Spalten, Versatz RB_VAR + 4) ab Zeile 51 (Textzeile 6);  SOFF  Bildlauf aus ab Zeile 59
;           Eintraege, deren Farbe mitten im Bild wechselt (BAR, FRAME, BASE, TBAND, SON, SOFF), werden eine Zeile frueher ausgeloest und warten dann auf den
;           genauen Zeilenanfang (sonst wackelte die Kante je nach dem Befehl, den der Interrupt unterbrach); GBAND wechselt in der waagerechten Linie
;           einer Rechteckzeile (dort sind alle Zeichen einfarbig und nutzen die Mehrfarben-Hintergruende nicht) und braucht das nicht.
; HARDWARE  $FF0A Bit 1 = Rasterinterrupt, Bit 0 = Bit 8 der Vergleichszeile, $FF0B = Vergleichszeile (Bits 0-7), $FF09 Bit 1 = Quittung (Schreiben von 1);
;           $FF1D = laufende Rasterzeile.  Der IRQ-Vektor des KERNAL ($0314/$0315, bei eingeblendetem ROM) zeigt auf den Handler; das ROM hat A, X, Y schon
;           gerettet, der Handler holt sie selbst zurueck und endet mit RTI.  Diskettenzugriffe laufen mit gesperrtem Interrupt (pause/resume).
;           Die Sichtgrenzen (am Emulator gemessen): PAL zeigt im Rand unten die Zeilen 204..251 und oben 276..311 (und 0..3), NTSC unten 204..227 und oben
;           248..261 (und 0..3); daher je Norm eine Tabelle.
; =====================================================================================================================

; Zustand im RAM (RB_VAR, main.asm): +0 laufender Eintrag, +1 Phase, +2 Ende (Eintragsnummer + 1 des letzten), +3 Modus (0 Spiel, 1 Titelbild),
;   +4 Bildlauf-Versatz, +5 erster Eintrag der Norm
.const K_BAR   = 0
.const K_FRAME = 1
.const K_BASE  = 2
.const K_GBAND = 3
.const K_TBAND = 4
.const K_SON   = 5
.const K_SOFF  = 6

.const RB_NBOT_PAL  = 12                ; Balken unten PAL: Zeilen 204, 208, ... 248
.const RB_NTOP_PAL  = 9                 ; oben PAL: Zeilen 276, 280, ... 308
.const RB_NBOT_NTSC = 6                 ; unten NTSC: 204 .. 224
.const RB_NTOP_NTSC = 3                 ; oben NTSC: 248, 252, 256
.const RB_NMID      = 15                ; Eintraege im Bildbereich (siehe MID_*)
.const RB_PAL_N     = RB_NBOT_PAL + RB_NTOP_PAL + RB_NMID
.const RB_NTSC_N    = RB_NBOT_NTSC + RB_NTOP_NTSC + RB_NMID

raster: {

; start: Interrupt einrichten und freigeben.  Veraendert A, X.
start:  sei
        lda CINV
        sta OLDIRQ
        lda CINV + 1
        sta OLDIRQ + 1
        lda #<irq
        sta CINV
        lda #>irq
        sta CINV + 1
        lda FLAGS
        and #F_PAL
        beq .ntsc
        lda #RB_PAL_N
        ldx #0
        beq .n                          ; (immer: X = 0)
.ntsc:  lda #RB_PAL_N + RB_NTSC_N
        ldx #RB_PAL_N
.n:     stx RB_VAR + 5
        stx RB_VAR
.if FX_RASTER
        sta RB_VAR + 2
.else
        inx                             ; nur der Bildtakt: der erste Eintrag der Norm (Zeile 204), mit fester Rahmenfarbe
        stx RB_VAR + 2
        dex
        lda #3
        sta RB_ARG,x                    ; Balken 3 des Farbverlaufs = BORDER_COLOR (Blau, Helligkeit 3); die Phase bleibt stehen
.endif
        lda #0
        sta RB_VAR + 1
        sta RB_VAR + 3
        lda TED_IRQ_FLAGS
        sta TED_IRQ_FLAGS               ; offene Meldungen quittieren
        jsr line
        cli
        rts

; line: Vergleichszeile und Freigabe des Eintrags RB_VAR einstellen.  Veraendert A, X.
line:   ldx RB_VAR
        lda RB_LINE,x
        sta TED_RASTER
        lda RB_HI,x
        ora #2
        sta TED_IRQ_ENABLE
        rts

; pause: Rasterinterrupt sperren (vor Diskettenzugriffen).  Veraendert nichts ausser dem Interruptstatus und dem Rahmen.
pause:  sei
        pha
        lda #0
        sta TED_IRQ_ENABLE
        lda OLDIRQ
        sta CINV                        ; waehrend des Diskettenzugriffs gilt wieder der Interruptvektor des KERNALs
        lda OLDIRQ + 1
        sta CINV + 1
        lda #BORDER_COLOR
        sta TED_BORDERCOLOR
        lda TED_CTRL2
        ora #8
        sta TED_CTRL2                   ; 40 Spalten
        pla
        rts

; resume: Rasterinterrupt wieder freigeben.  Veraendert nichts.
resume: pha
        txa
        pha
        lda #<irq
        sta CINV
        lda #>irq
        sta CINV + 1
        lda TED_IRQ_FLAGS
        sta TED_IRQ_FLAGS
        jsr line
        pla
        tax
        pla
        cli
        rts

; irq: Eintrag RB_KIND des laufenden Eintrags ausfuehren, naechste Zeile einstellen.
irq:    lda TED_IRQ_FLAGS
        sta TED_IRQ_FLAGS               ; alle gemeldeten Quellen quittieren (das KERNAL kann bei Diskettenzugriffen eigene freigeben)
        and #2
        bne .ras
        pla                             ; kein Rasterinterrupt: sofort zurueck
        tay
        pla
        tax
        pla
        rti
.ras:   ldx RB_VAR
        lda RB_KIND,x
        cmp #K_GBAND
        beq .gband
        cmp #K_TBAND
        beq .tband
        cmp #K_SON
        beq .son
        cmp #K_SOFF
        bne .bar
        jmp .soff
.bar:   pha                             ; BAR, FRAME, BASE: Rahmenfarbe, auf den Zeilenanfang warten
        lda RB_LINE,x
.w1:    cmp TED_RASTER_LO
        beq .w1
        pla
        cmp #K_BASE
        beq .base
        pha
        lda RB_ARG,x
        clc
        adc RB_VAR + 1
        and #15
        tay
        lda GRAD,y
        sta TED_BORDERCOLOR
        pla
        cmp #K_FRAME
        bne .next
        inc FRAMEC                      ; Bildtakt
        lda FRAMEC
        and #1
        bne .next
.if FX_RASTER
        inc RB_VAR + 1                  ; die Balken laufen alle zwei Bilder einen Schritt weiter
.endif
        jmp .next
.base:  lda #BORDER_COLOR
        sta TED_BORDERCOLOR
        lda RB_VAR + 3
        bne .next                       ; Titelbild: Hintergrund 0 setzt TBAND 0
        lda #BG0_COLOR
        sta TED_BGCOLOR
        jmp .next
.gband:
.if FX_SHADE
        lda RB_VAR + 3
        bne .next                       ; im Titelbild nicht
        ldy RB_ARG,x
        lda BAND_BG1,y
        sta TED_COLOR1
        lda BAND_BG2,y
        sta TED_COLOR2
.endif
        jmp .next
.tband: lda RB_VAR + 3
        beq .next                       ; nur im Titelbild
        lda RB_LINE,x
.w2:    cmp TED_RASTER_LO
        beq .w2
        ldy RB_ARG,x
        lda TBG,y
        sta TED_BGCOLOR
        jmp .next
.son:   lda RB_VAR + 3
        beq .next
        lda RB_LINE,x
.w3:    cmp TED_RASTER_LO
        beq .w3
        lda RB_VAR + 4                  ; Bildlauf-Anfang: 38 Spalten (Raender verdeckt), Versatz 0..7
        and #7
        ora #$90
        sta TED_CTRL2
        jmp .next
.soff:  lda RB_VAR + 3
        beq .next
        lda RB_LINE,x
.w4:    cmp TED_RASTER_LO
        beq .w4
        lda #$98                        ; Bildlauf-Ende: 40 Spalten, kein Versatz
        sta TED_CTRL2
.next:  inx
        cpx RB_VAR + 2
        bcc .nx
        ldx RB_VAR + 5
.nx:    stx RB_VAR
        lda RB_LINE,x
        sta TED_RASTER
        lda RB_HI,x
        ora #2
        sta TED_IRQ_ENABLE
        pla
        tay
        pla
        tax
        pla
        rti
}

; Farbverlauf der Balken (16 Schritte, hin und zurueck): Blau mit Helligkeit 0..7 und zurueck
GRAD:
.var lums = [0, 1, 2, 3, 4, 5, 6, 7, 7, 6, 5, 4, 3, 2, 1, 0]
.for i in range(16)
        .byte (lums[i] << 4) | 6
.endfor

; Hintergrundbaender des Titelbilds (Hintergrund 0)
TBG:    .byte $06, $16, $26, $36, $26, $16, $06

; Eintraege im Bildbereich (Zeilen 1..187; "Zeile" = Zeile des Interrupts, bei genauen Eintraegen eine vor der Zielzeile)
.ifndef SON_L
.const SON_L = 49
.endif
.ifndef SOFF_L
.const SOFF_L = 57
.endif
.var mid_line = [1, 3, 22, 27, SON_L, 54, SOFF_L, 67, 86, 91, 118, 123, 150, 155, 187]
.var mid_kind = [K_BASE, K_TBAND, K_GBAND, K_TBAND, K_SON, K_GBAND, K_SOFF, K_TBAND, K_GBAND, K_TBAND, K_GBAND, K_TBAND, K_GBAND, K_TBAND, K_TBAND]
.var mid_arg  = [0, 0, 0, 1, 0, 1, 0, 2, 2, 3, 3, 4, 4, 5, 6]

; Tabellen: erst PAL (RB_PAL_N Eintraege), dann NTSC.  Zeile (niedrige 8 Bit), Bit 8, Art, Argument.
RB_LINE:
        .byte 203                       ; PAL: FRAME (Zielzeile 204)
.for i in range(RB_NBOT_PAL - 1)
        .byte (204 + 4 * (i + 1) - 1) & 255
.endfor
.for i in range(RB_NTOP_PAL)
        .byte (276 + 4 * i - 1) & 255
.endfor
.for i in range(RB_NMID)
        .byte mid_line[i]
.endfor
        .byte 203                       ; NTSC
.for i in range(RB_NBOT_NTSC - 1)
        .byte (204 + 4 * (i + 1) - 1) & 255
.endfor
.for i in range(RB_NTOP_NTSC)
        .byte (248 + 4 * i - 1) & 255
.endfor
.for i in range(RB_NMID)
        .byte mid_line[i]
.endfor
RB_HI:
.for i in range(RB_NBOT_PAL)
        .byte 0
.endfor
.for i in range(RB_NTOP_PAL)
        .byte (276 + 4 * i - 1) >> 8
.endfor
.for i in range(RB_NMID)
        .byte 0
.endfor
.for i in range(RB_NBOT_NTSC)
        .byte 0
.endfor
.for i in range(RB_NTOP_NTSC)
        .byte (248 + 4 * i - 1) >> 8
.endfor
.for i in range(RB_NMID)
        .byte 0
.endfor
RB_KIND:
        .byte K_FRAME
.for i in range(RB_NBOT_PAL - 1 + RB_NTOP_PAL)
        .byte K_BAR
.endfor
.for i in range(RB_NMID)
        .byte mid_kind[i]
.endfor
        .byte K_FRAME
.for i in range(RB_NBOT_NTSC - 1 + RB_NTOP_NTSC)
        .byte K_BAR
.endfor
.for i in range(RB_NMID)
        .byte mid_kind[i]
.endfor
RB_ARG:
.for i in range(RB_NBOT_PAL + RB_NTOP_PAL)
        .byte i * 2
.endfor
.for i in range(RB_NMID)
        .byte mid_arg[i]
.endfor
.for i in range(RB_NBOT_NTSC + RB_NTOP_NTSC)
        .byte i * 2
.endfor
.for i in range(RB_NMID)
        .byte mid_arg[i]
.endfor
