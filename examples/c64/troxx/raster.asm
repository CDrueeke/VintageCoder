; =====================================================================================================================
; raster.asm - Rasterinterrupt: wandernde Farbbalken im oberen und unteren Rand (Kupferbalken)
; =====================================================================================================================
; ZWECK     Der VIC-II loest in einer vorgegebenen Rasterzeile einen Interrupt aus; der Handler setzt die Rahmenfarbe ($D020) und stellt die
;           naechste Zeile ein.  So entsteht im Rand oberhalb (Zeilen 16..44) und unterhalb (ab Zeile 252) des Bildes eine Folge von Farbbalken,
;           die mit jedem Bild einen Schritt weiterlaeuft; neben dem Bild (Zeile 48 bis 251) gilt die feste Rahmenfarbe.  Die Spiellogik bleibt bei
;           der Rasterabfrage in vic.wait_frame (Zeile 251); der Interrupt aendert nur den Rahmen.
; ABLAUF    RB_LINE (niedriges Byte der Rasterzeile), RB_HI (Bit 8 der Zeile in $D011 Bit 7) und RB_BAR (Nummer des Balkens, $FF = feste Rahmenfarbe)
;           bilden eine Tabelle aufsteigender Zeilen.  PAL hat unten Platz bis Zeile 288, NTSC (262 Zeilen) nur bis 260: RB_COUNT je Norm.
; HARDWARE  $D01A Bit 0 = Rasterinterrupt, $D012 / $D011 Bit 7 = Vergleichszeile, $D019 Bit 0 = Quittung (Schreiben von 1); der IRQ-Vektor des
;           KERNAL ($0314/$0315, bei eingeblendetem KERNAL) zeigt auf den Handler; die Timer-Interrupts der CIAs sind aus ($DC0D, $DD0D = $7F).
;           Der Handler zieht die vom KERNAL gesicherten Register selbst vom Stapel und endet mit RTI.  Diskettenzugriffe laufen mit gesperrtem
;           Rasterinterrupt (raster.pause / raster.resume), sonst stoert er das Zeitverhalten des seriellen Busses.
; =====================================================================================================================

; Zustand im RAM (RB_VAR, main.asm): +0 laufender Eintrag, +1 Phase, +2 Anzahl der Eintraege, +3 Modus (0 Spiel, 5 Titelbild), +4 Bildlauf-Versatz
.const RB_NTOP  = 8
.const RB_BASE  = COL_LIGHTBLUE         ; feste Rahmenfarbe

raster: {

; start: Interrupt einrichten und freigeben.  Veraendert A, X.
start:  sei
        lda #$7f
        sta CIA1_ICR                    ; Timer-Interrupts der CIAs aus
        sta CIA2_ICR
        lda CIA1_ICR                    ; offene Meldungen quittieren
        lda CIA2_ICR
        lda #<irq
        sta $0314
        lda #>irq
        sta $0315
        lda FLAGS
        and #F_PAL
        beq .ntsc
        lda #26                         ; PAL: 16 + 10 Eintraege
        bne .n
.ntsc:  lda #19                         ; NTSC: 16 + 3
.n:     sta RB_VAR + 2
        lda #0
        sta RB_VAR
        sta RB_VAR + 1
        lda RB_LINE
        sta VIC_RASTER
        lda VIC_CTRL1
        and #$7f
        sta VIC_CTRL1
        lda #1
        sta VIC_IRQ_FLAGS
        sta VIC_IRQ_ENABLE
        cli
        rts

; pause: Rasterinterrupt sperren (vor Diskettenzugriffen).  Veraendert nichts ausser dem Interruptstatus.
pause:  sei
        pha
        lda #0
        sta VIC_IRQ_ENABLE
        lda #RB_BASE
        sta VIC_BORDERCOLOR
        pla
        rts

; resume: Rasterinterrupt wieder freigeben.
resume: pha
        lda #1
        sta VIC_IRQ_FLAGS
        sta VIC_IRQ_ENABLE
        pla
        cli
        rts

; irq: Eintrag RB_KIND des laufenden Eintrags ausfuehren (Randbalken, feste Rahmenfarbe, Hintergrundband, Bildlauf-Anfang, Bildlauf-Ende), naechste Zeile einstellen.
irq:    lda #1
        sta VIC_IRQ_FLAGS
        ldx RB_VAR
        lda RB_KIND,x
        beq .bar
        cmp #1
        beq .base
        cmp #2
        beq .bg
        cmp #3
        beq .scr
        lda #$18                        ; Bildlauf-Ende: 40 Spalten, Mehrfarben erlaubt, kein Versatz
        sta VIC_CTRL2
        jmp .next
.scr:   lda RB_VAR + 4                  ; Bildlauf-Anfang: 38 Spalten (Raender verdeckt), Versatz 0..7
        ora #$10
        sta VIC_CTRL2
        jmp .next
.bg:    lda RB_ARG,x                    ; Hintergrundband: Farbe aus der Tabelle des Modus (0 Spiel, 5 Titelbild)
        clc
        adc RB_VAR + 3
        tay
        lda BGT,y
        sta VIC_BGCOLOR0
        jmp .next
.bar:   lda RB_VAR + 5                  ; Rasterfarbblitz nach einer Zerstoerung: rot/weiss statt Farbverlauf
        beq .grad
        txa
        and #1
        tay
        lda FLASHB,y
        bne .set
.grad:  lda RB_ARG,x
        clc
        adc RB_VAR + 1
        and #15
        tay
        lda GRAD,y
        bne .set
.base:  lda #RB_BASE
.set:   sta VIC_BORDERCOLOR
.next:  inx
        cpx RB_VAR + 2
        bcc .nx
        ldx #0
        inc RB_VAR + 1                  ; ein Schritt je Bild
        lda RB_VAR + 5
        beq .nx
        dec RB_VAR + 5
.nx:    stx RB_VAR
        lda VIC_CTRL1
        and #$7f
        ora RB_HI,x
        sta VIC_CTRL1
        lda RB_LINE,x
        sta VIC_RASTER
        pla
        tay
        pla
        tax
        pla
        rti
}

; Farbverlauf der Balken (16 Schritte, hin und zurueck): blau - hellblau - cyan - weiss
GRAD:   .byte COL_BLUE, COL_BLUE, COL_LIGHTBLUE, COL_LIGHTBLUE, COL_CYAN, COL_CYAN, COL_WHITE, COL_WHITE
        .byte COL_WHITE, COL_WHITE, COL_CYAN, COL_CYAN, COL_LIGHTBLUE, COL_LIGHTBLUE, COL_BLUE, COL_BLUE

; Hintergrundfarben der fuenf Baender: Spiel (alle blau) und Titelbild (oben und unten schwarz)
FLASHB: .byte COL_RED, COL_WHITE
BGT:    .byte COL_BLUE, COL_BLUE, COL_BLUE, COL_BLUE, COL_BLUE
        .byte COL_BLACK, COL_BLUE, COL_BLUE, COL_BLUE, COL_BLACK

; Eintraege (aufsteigende Zeilen): Art 0 Balken (Arg = Nummer), 1 feste Rahmenfarbe, 2 Hintergrundband (Arg = Band), 3 Bildlauf an, 4 Bildlauf aus
RB_LINE:
.for i in range(RB_NTOP)
        .byte 16 + 4 * i
.endfor
        .byte 48, 56, 97, 106, 120, 160, 200, 236
.for i in range(10)
        .byte (252 + 4 * i) & 255
.endfor
RB_HI:
.for i in range(RB_NTOP + 8)
        .byte 0
.endfor
.for i in range(10)
        .byte ((252 + 4 * i) >> 8) << 7
.endfor
RB_KIND:
.for i in range(RB_NTOP)
        .byte 0
.endfor
        .byte 1, 2, 3, 4, 2, 2, 2, 2
.for i in range(10)
        .byte 0
.endfor
RB_ARG:
.for i in range(RB_NTOP)
        .byte i
.endfor
        .byte 0, 0, 0, 0, 1, 2, 3, 4
.for i in range(10)
        .byte i
.endfor
