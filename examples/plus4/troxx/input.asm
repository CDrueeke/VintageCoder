; =====================================================================================================================
; input.asm - Eingabe: Joystick ueber die Tastaturmatrix des TED (zwei Anschluesse)
; =====================================================================================================================
; Herkunft: examples/c64/troxx/input.asm (Stand Commit d173cb4a), fuer den Plus/4 neu geschrieben (Joystick ueber die Tastaturmatrix); nicht in der Kopienliste.
; ZWECK     Liefert die Joystickmaske (IN_*, 1 = gedrueckt) eines Anschlusses, des aktiven Spielers oder beider Anschluesse zusammen.
;           Der aktive Anschluss folgt der Wahl im Titelbild (JOYMODE): 0 ein Joystick an Anschluss 2 (Standard), 1 ein Joystick an Anschluss 1,
;           2 zwei Joysticks (Spieler 1 an Anschluss 1, Spieler 2 an Anschluss 2).
; HARDWARE  Die Joysticks des Plus/4 liegen an der Tastaturmatrix: $FD30 = $FF waehlt keine Tastenspalte, dann waehlt ein Wert in $FF08 die Leitung des
;           Anschlusses (Bit 2 = 0, also $FB: Anschluss 1; Bit 1 = 0, also $FD: Anschluss 2) und das Lesen von $FF08 liefert (aktiv niedrig) Bit 0 hoch,
;           1 runter, 2 links, 3 rechts, und das Feuer: Bit 7 oder Bit 6 (beide werden ausgewertet).  Die Tastatur wird nicht benutzt; gedrueckte Tasten
;           koennen aber einen Joystick-Kontakt vortaeuschen, weil die Tastaturmatrix dieselben Leitungen nutzt.  Im Emulator gilt dieselbe Belegung.
;           [Stand: Leerlaufwerte $FF und die Belegung aus der Plus/4-Dokumentation; die Kontakte beider Anschluesse im Handtest pruefen.]
; =====================================================================================================================

input: {

; setup: nichts einzurichten (die Matrix wird bei jeder Abfrage gewaehlt).  Veraendert nichts.
setup:  rts

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
        lda #$ff
        sta PIO_KEYBOARD                ; keine Tastenspalte waehlen
        cpx #1
        bne .p2
        lda #$fb                        ; Anschluss 1
        sta TED_KEYLATCH
        lda TED_KEYLATCH
        jmp .m
.p2:    lda #$fd                        ; Anschluss 2
        sta TED_KEYLATCH
        lda TED_KEYLATCH
.m:     eor #$ff                        ; gedrueckt = 1
        tax
        and #$c0                        ; Feuer: Bit 6 oder Bit 7 (die Belegung je Anschluss ist nicht gesichert, darum beide)
        beq .d
        txa
        and #$0f
        ora #IN_FIRE
        rts
.d:     txa
        and #$0f
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
