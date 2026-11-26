; =====================================================================================================================
; input.asm - Eingabe: Joystick ueber CIA 1 (zwei Anschluesse)
; =====================================================================================================================
; Herkunft: examples/vic20/troxx/player.asm (read_joy), fuer den C64 neu geschrieben (zwei Anschluesse, Joystickwahl); nicht in der Kopienliste.
; ZWECK     Liefert die Joystickmaske (IN_*, 1 = gedrueckt) eines Anschlusses, des aktiven Spielers oder beider Anschluesse zusammen.
;           Der aktive Anschluss folgt der Wahl im Titelbild (JOYMODE): 0 ein Joystick an Anschluss 2 (Standard), 1 ein Joystick an Anschluss 1,
;           2 zwei Joysticks (Spieler 1 an Anschluss 1, Spieler 2 an Anschluss 2).
; HARDWARE  CIA 1: Anschluss 2 liegt auf Port A ($DC00), Anschluss 1 auf Port B ($DC01); Bit 0 hoch, 1 runter, 2 links, 3 rechts, 4 Feuer,
;           aktiv niedrig.  Beide Datenrichtungsregister stehen auf Eingang (setup); die Tastatur wird nicht benutzt.  Die Tastaturmatrix liegt auf
;           denselben Leitungen: eine gedrueckte Taste kann einen Joystick-Kontakt vortaeuschen (bei Anschluss 1 genauso wie bei 2).
; =====================================================================================================================

input: {

; setup: Datenrichtungen der Eingabe setzen (einmal beim Start).  Veraendert A.
setup:
        lda #0
        sta CIA1_DDRA                   ; Port A = Eingang: Joystick Anschluss 2
        sta CIA1_DDRB                   ; Port B = Eingang: Joystick Anschluss 1
        rts

; port_of: X = Anschluss (1 oder 2) des aktiven Spielers laut JOYMODE.  Veraendert A, X.
port_of:
        lda JOYMODE
        beq .p2                         ; 0: ein Joystick, Anschluss 2
        cmp #1
        beq .p1                         ; 1: ein Joystick, Anschluss 1
        ldx CUR                         ; 2: zwei Joysticks, Spieler 1 an Anschluss 1, Spieler 2 an Anschluss 2
        inx
        rts
.p1:    ldx #1
        rts
.p2:    ldx #2
        rts

; read_port: A = Maske (IN_*, 1 = gedrueckt) des Joysticks an Anschluss X (1 oder 2).  Veraendert A.
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
        cpx #1
        bne .p2
        lda CIA1_PRB
        jmp .m
.p2:    lda CIA1_PRA
.m:     eor #$ff                        ; gedrueckt = 1
        and #$1f
        rts
.endif

; read_joy: A = Maske des Joysticks des aktiven Spielers.  Veraendert A, X.
read_joy:
        jsr port_of
        jmp read_port

; read_any: A = Maske beider Anschluesse zusammen (Titelbild und Zwischenbilder).  Veraendert A, X, T7.
read_any:
        ldx #1
        jsr read_port
        sta T7
        ldx #2
        jsr read_port
        ora T7
        rts

}
