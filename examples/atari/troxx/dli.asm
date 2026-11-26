; =====================================================================================================================
; dli.asm - Display-List-Interrupt und Bildruecklauf: Farbwechsel je Textzeile, Farbbalken im Rand
; =====================================================================================================================
; ZWECK     Alle 24 Textzeilen (und der erste Leerzeilenblock) tragen das DLI-Bit; ANTIC loest den Interrupt am Ende der Zeile aus.  Index dli_i (Zeile, deren
;           Farben der naechste Interrupt setzt):
;             0       Leerzeilenblock oben (dli_top): faehrt 17 Rasterzeilen lang Farbbalken in COLBK (je Zeile WSYNC) und setzt am Ende die Farben der Zeile 0;
;             1..23   Ende der Zeile dli_i - 1 (dli_row): Farben COLBK, COLPF1, COLPF2 der folgenden Zeile aus RC_BK, RC_P1, RC_P2; HSCROL aus RC_H (dli_scr);
;             24      Ende der letzten Zeile (dli_bottom): 16 Rasterzeilen lang Farbbalken unter dem Bild.
;           Alle Schreibzugriffe stehen hinter WSYNC (die neue Zeile beginnt gleich mit den neuen Farben); die Werte holt der Code vorher.
; BILDRUECKLAUF  Der verzoegerte Vertikal-Interrupt (vbi) setzt dli_i = 0, COLPF3 und den ersten Balkenwert, stellt den Bildschirmschoner des
;           Betriebssystems ab und zaehlt die Balkenphase BPHASE weiter.  Das Betriebssystem kopiert vorher seine Schatten (Display-Liste, PRIOR,
;           Spielerfarben, Zeichensatz) in die Chips.
; FARBBALKEN  BAR ist eine Tabelle von 64 Farben (zweimal hintereinander, damit der Zugriff nicht umbrechen muss); oben laeuft der Balken nach rechts
;           (BPHASE steigt), unten gegenlaeufig.  Der Balken lebt vom Hintergrund-Farbregister COLBK, das auch den Rand faerbt.
; =====================================================================================================================

; Vektoren (VDSLST): vbi setzt dli_top; dli_top schaltet auf dli_row; dli_row schaltet vor der Lauftextzeile kurz auf dli_scr und nach der letzten Zeile auf dli_bottom.
; Vor WSYNC steht nur das Noetigste (die Werte der naechsten Zeile liegen schon in DLT_BK / DLT_P1 / DLT_P2): ANTIC stiehlt in den ersten Rasterzeilen einer
;   Textzeile bis zu 40 Takte je Zeile, ein langer Vorspann liesse den Farbwechsel um Rasterzeilen zu spaet kommen.  Die Werte der uebernaechsten Zeile holt
;   der Code erst NACH dem Schreiben (nextrow).

; dli_row: Ende einer Textzeile: Farben der folgenden Zeile setzen.
dli_row:
        pha
        txa
        pha
        ldx DLT_BK
        lda DLT_P2
        sta WSYNC
        stx COLBK
        sta COLPF2
        lda DLT_P1
        sta COLPF1
        jmp nextrow

; dli_scr: wie dli_row, vor der Lauftextzeile: HSCROL schon in der letzten Rasterzeile davor setzen (ANTIC liest ihn zu Beginn jeder Rasterzeile).
dli_scr:
        pha
        txa
        pha
        lda DLT_H
        sta HSCROL
        ldx DLT_BK
        lda DLT_P2
        sta WSYNC
        stx COLBK
        sta COLPF2
        lda DLT_P1
        sta COLPF1
        lda #<dli_row
        sta VDSLST
        lda #>dli_row
        sta VDSLST + 1
        ; weiter mit nextrow

; nextrow: Werte der naechsten Zeile (Index dli_i + 1) in DLT_* holen; vor der letzten Zeile auf dli_bottom, vor der Lauftextzeile auf dli_scr schalten.
nextrow:
        ldx dli_i
        inx
        stx dli_i
        cpx #24
        bcs .bot
        lda RC_BK,x
        sta DLT_BK
        lda RC_P1,x
        sta DLT_P1
        lda RC_P2,x
        sta DLT_P2
        lda RC_H,x
        sta DLT_H
        cpx SCRROW
        bne .out
        lda #<dli_scr                   ; die folgende Zeile ist die Lauftextzeile
        sta VDSLST
        lda #>dli_scr
        sta VDSLST + 1
.out:   pla
        tax
        pla
        rti
.bot:   lda #<dli_bottom                ; hinter der letzten Textzeile: Farbbalken unten
        sta VDSLST
        lda #>dli_bottom
        sta VDSLST + 1
        jmp .out

; dli_top: Leerzeilenblock oben (24 Rasterzeilen): ab Ende der ersten Zeile faehrt der Code 17 Rasterzeilen lang Farbbalken (je Zeile WSYNC, COLBK aus BAR) und
;   setzt dann die Farben der Zeile 0.
dli_top:
        pha
        txa
        pha
        tya
        pha
        lda BPHASE
        and #63
        tax
        ldy #16
.t:     lda BARRAM,x
        sta WSYNC
        sta COLBK
        inx
        dey
        bne .t
        lda RC_P1                       ; Farben der Zeile 0 (die erste Rasterzeile der Zeile 0)
        sta DLT_P1
        lda RC_P2
        tay
        lda RC_BK
        sta WSYNC
        sta COLBK
        sty COLPF2
        lda DLT_P1
        sta COLPF1
        lda #1                          ; naechster Index: Farben der Zeile 1
        sta dli_i
        lda RC_BK + 1
        sta DLT_BK
        lda RC_P1 + 1
        sta DLT_P1
        lda RC_P2 + 1
        sta DLT_P2
        lda #<dli_row
        sta VDSLST
        lda #>dli_row
        sta VDSLST + 1
        pla
        tay
        pla
        tax
        pla
        rti

; dli_bottom: Ende der letzten Textzeile: 16 Rasterzeilen Farbbalken unter dem Bild.
dli_bottom:
        pha
        txa
        pha
        tya
        pha
        lda BPHASE
        eor #63                         ; gegenlaeufig
        and #63
        tax
        ldy #16
.b:     lda BARRAM,x
        sta WSYNC
        sta COLBK
        inx
        dey
        bne .b
        lda #0
        sta COLBK
        pla
        tay
        pla
        tax
        pla
        rti

; vbi: verzoegerter Bildruecklauf-Interrupt: Vektor auf dli_top, Farben der Zeile 0, COLPF3, Balkenphase.
vbi:    lda #<dli_top
        sta VDSLST
        lda #>dli_top
        sta VDSLST + 1
        lda #0
        sta ATRACT
        sta dli_i
        lda PF3COL
        sta COLPF3
        lda RC_P2
        sta COLPF2
        lda RC_P1
        sta COLPF1
.if FX_DLI == 0
        lda #$9e                        ; ohne DLI: eine Farbe fuer Fuellungen und Schrift (die Textzeilen 0 und 23 zeigen dann hellen Grund)
        sta COLPF2
        lda #COL_DONE_LINE
        sta COLPF1
.endif
        inc BPHASE
        lda BPHASE
        and #63
        tax
        lda BARRAM,x
        sta COLBK
        jmp XITVBV

; BAR: 64 Farben, zweimal (128 Byte): weicher Farbverlauf dunkelblau - hellblau - dunkelblau, mit einem schmalen hellen Balken
BAR:
.for k in range(2)
    .for i in range(64)
        .var d = i                                  ; 0..31..0
        .if i >= 32
            .var d = 63 - i
        .endif
        .var lum = d / 8                            ; 0..3
        .if d >= 14 && d <= 17
            .byte $9a - (d - 14) * 0                ; heller Balken (Tuerkis)
        .else
            .byte $70 + lum * 2
        .endif
    .endfor
.endfor
