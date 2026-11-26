; =====================================================================================================================
; input.asm - Eingabe: Joystick ueber PIA und TRIG (zwei Anschluesse)
; =====================================================================================================================
; Herkunft: examples/c64/troxx/input.asm (Stand Commit d173cb4a); fuer den Atari neu geschrieben, nicht in der Kopienliste.
; ZWECK     Liefert die Joystickmaske (IN_*, 1 = gedrueckt) eines Anschlusses, des aktiven Spielers oder beider Anschluesse zusammen.
;           Der aktive Anschluss folgt der Wahl im Titelbild (JOYMODE): 0 ein Joystick an Anschluss 1 (Standard, beim Atari ueblich), 1 ein Joystick an
;           Anschluss 2, 2 zwei Joysticks (Spieler 1 an Anschluss 1, Spieler 2 an Anschluss 2).  Ziel ist der XL/XE mit zwei Anschluessen.
; HARDWARE  PIA Port A ($D300): Anschluss 1 in den unteren vier Bit, Anschluss 2 in den oberen vier Bit; je Anschluss Bit 0 hoch, 1 runter, 2 links,
;           3 rechts, aktiv niedrig.  Feuer: TRIG0 ($D010, Anschluss 1) und TRIG1 ($D011, Anschluss 2), aktiv niedrig.  Zusaetzlich zaehlt die
;           START-Taste (CONSOL Bit 0) im Titelbild und in den Zwischenbildern als Feuer.
; =====================================================================================================================

input: {

; setup: nichts einzurichten (das Betriebssystem hat die PIA schon eingestellt).  Veraendert nichts.
setup:  rts

; port_of: X = Anschluss (1 oder 2) des aktiven Spielers laut JOYMODE.  Veraendert A, X.
port_of:
        lda JOYMODE
        beq .p1                         ; 0: ein Joystick, Anschluss 1
        cmp #1
        beq .p2                         ; 1: ein Joystick, Anschluss 2
        ldx CUR                         ; 2: zwei Joysticks, Spieler 1 an Anschluss 1, Spieler 2 an Anschluss 2
        inx
        rts
.p1:    ldx #1
        rts
.p2:    ldx #2
        rts

; read_port: A = Maske (IN_*, 1 = gedrueckt) des Joysticks an Anschluss X (1 oder 2).  Veraendert A, Y.
.ifdef SCRIPT
; Testbetrieb (-D SCRIPT=n): die Masken beider Anschluesse kommen aus der Skripttabelle SCRIPT_TAB (Eintraege: Bilder, Maske Anschluss 1,
;   Maske Anschluss 2; Dauer 0 beendet das Skript, danach ist nichts gedrueckt).  script_step schaltet je Bild (aus tick) weiter.
read_port:
        cpx #1
        bne .p2s
        lda SCR_M1
        rts
.p2s:   lda SCR_M2
        rts

; script_step: je Bild einmal: Rest der Dauer herunterzaehlen, beim Ablauf den naechsten Eintrag laden.  Veraendert A, X.
script_step:
        lda SCR_REM
        beq .next
        dec SCR_REM
        bne .out
.next:  ldx SCR_IDX
        lda SCRIPT_TAB,x
        beq .end
        sta SCR_REM
        lda SCRIPT_TAB + 1,x
        sta SCR_M1
        lda SCRIPT_TAB + 2,x
        sta SCR_M2
        inx
        inx
        inx
        stx SCR_IDX
        rts
.end:   lda #0
        sta SCR_M1
        sta SCR_M2
.out:   rts
.else
read_port:
        lda PORTA
        cpx #1
        beq .m
        lsr                             ; Anschluss 2: obere vier Bit nach unten
        lsr
        lsr
        lsr
.m:     eor #$0f                        ; gedrueckt = 1
        and #$0f
        cpx #1
        beq .f1
        ldy TRIG1
        jmp .f
.f1:    ldy TRIG0
.f:     bne .out
        ora #IN_FIRE
.out:   rts
.endif

; read_joy: A = Maske des Joysticks des aktiven Spielers.  Veraendert A, X, Y.
read_joy:
        jsr port_of
        jmp read_port

; read_any: A = Maske beider Anschluesse zusammen (und START als Feuer; Titelbild und Zwischenbilder).  Veraendert A, X, Y, T7.
read_any:
        ldx #1
        jsr read_port
        sta T7
        ldx #2
        jsr read_port
        ora T7
        sta T7
.ifndef SCRIPT
        lda CONSOL
        and #1
        bne .nos
        lda T7
        ora #IN_FIRE
        rts
.nos:
.endif
        lda T7
        rts
}
