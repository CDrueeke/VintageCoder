// BASIC→Assembler (basic2asm), Atari 8 Bit: die KERNAL-Aufrufe, die die gemeinsame Laufzeit (data/sys/common/b2a) benutzt,
// als kleine Schicht über dem Betriebssystem. Namen und Schnittstelle wie beim C64-KERNAL; X und Y bleiben erhalten.
//   CHROUT  A = Zeichen (13 = Zeilenende, wird zu ATASCII $9B), Ausgabe über CIO-Kanal 0 (Editor)
//   CHRIN   liest ein Zeichen vom Editor (Eingabe einer Zeile, Echo); Zeilenende kommt als 13 zurück, Carry gelöscht
//   GETIN   Taste ohne Warten (A = 0, wenn keine gedrückt ist), ATASCII, über den Tastaturtreiber K:
//   PLOT    Carry gesetzt: X = Zeile, Y = Spalte des Cursors; Carry gelöscht: Cursor auf X = Zeile, Y = Spalte setzen (Spalten ab dem linken Rand)
//   RDTIM   A = niedrig, X = mitte, Y = hoch (Uhr RTCLOK, 1/60 s)    SETTIM   dasselbe in umgekehrter Richtung
.const CIO_GETCHR = $07
.const CIO_PUTCHR = $0B

CHROUT:   cmp #13
          bne .go
          lda #$9B
.go:      sta rt_os_a
          stx rt_os_x
          sty rt_os_y
          ldx rt_chan
          lda #CIO_PUTCHR
          sta ICCOM,x
          lda #0
          sta ICBLL,x
          sta ICBLH,x
          lda rt_os_a
          jsr CIOV
          lda rt_os_a
          ldx rt_os_x
          ldy rt_os_y
          rts

CHRIN:    stx rt_os_x
          sty rt_os_y
          ldx #0
          lda #CIO_GETCHR
          sta ICCOM
          lda #0
          sta ICBLL
          sta ICBLH
          jsr CIOV
          cmp #$9B
          bne .k
          lda #13
.k:       ldx rt_os_x
          ldy rt_os_y
          clc
          rts

GETIN:    lda CH
          cmp #$FF
          bne .key
          lda #0
          rts
.key:     stx rt_os_x
          sty rt_os_y
          jsr .callk
          ldx rt_os_x
          ldy rt_os_y
          rts
// K:-Treiber: Tabelle bei $E420 (öffnen, schließen, lesen, schreiben, …), jeder Eintrag ist die Adresse minus 1
.callk:   lda $E425
          pha
          lda $E424
          pha
          rts

// Spalten zählen ab dem linken Rand (LMARGN), wie die Tabulatoren von Atari-BASIC
PLOT:     bcc .set
          lda COLCRS
          sec
          sbc LMARGN
          bcs .ok
          lda #0
.ok:      tay
          ldx ROWCRS
          rts
.set:     stx ROWCRS
          tya
          clc
          adc LMARGN
          sta COLCRS
          rts

RDTIM:    lda RTCLOK+2
          ldx RTCLOK+1
          ldy RTCLOK
          rts

SETTIM:   sta RTCLOK+2
          stx RTCLOK+1
          sty RTCLOK
          rts

rt_os_a:  .byte 0
rt_os_x:  .byte 0
rt_os_y:  .byte 0
rt_chan:  .byte 0               // IOCB-Offset für CHROUT: 0 = E: (Editor), $60 = Kanal 6 (S:, PRINT #6)
