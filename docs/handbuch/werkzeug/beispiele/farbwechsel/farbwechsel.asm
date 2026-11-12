; Rahmenfarbe weiterschalten – kleines Übungsprogramm für Debugger und Tests (C64)
.target c64
.cpu 6510 { illegals = deny }
.const BORDER  = $d020
.const ZAEHLER = $fb            ; Zeropage: wie oft schon weitergeschaltet

.org $0801
.basicstub start

start:  lda #0
        sta ZAEHLER
schleife:
        jsr naechste_farbe
        jmp schleife

; erhöht den Zähler und zeigt dessen unteres Halbbyte als Rahmenfarbe
naechste_farbe:
        inc ZAEHLER
        lda ZAEHLER
        and #15
        sta BORDER
        rts
