// SPDX-License-Identifier: GPL-3.0-or-later
// Eingabe-Bibliothek C64 (PRALLBALL). Einbinden: .include "lib/input.asm"
//
// Quellen: Joystick Port 2 (CIA1 $DC00), Tastatur (Z links, X rechts, Q hoch, A runter, Leertaste Feuer, P Pause, RUN/STOP Abbruch),
// Paddle Port 2 (SID $D419, ⚠ nicht auf echter Hardware geprüft).
//
// Vertrag (auf allen Systemen gleich): input_poll liefert in A eine Maske aus IN_*; Bit gesetzt = Aktion aktiv.
// IN_FIRE wird als Flanke geliefert (nur im Aufruf des Tastendrucks), der Rest als Zustand.
//   input_init        in_state/in_prev löschen, Quelle = IN_SRC_AUTO
//   input_select      A = IN_SRC_AUTO/JOY/KEY/PADDLE
//   input_poll        A = Maske (Joystick und Tastatur vereint bei IN_SRC_AUTO); verändert X, Y
//   input_joy         A = Joystickmaske (ohne Pause/Abbruch)
//   input_key         A = Tastaturmaske
//   input_paddle      A = Position 0..255, C=1 gültig
//   input_wait_release  wartet, bis keine Aktion mehr aktiv ist
// Zeropage (4 Byte, mit -D IN_ZP=... verschiebbar): in_state, in_prev, in_pad, in_src

#include <c64/hardware.asm>

.const IN_UP      = $01
.const IN_DOWN    = $02
.const IN_LEFT    = $04
.const IN_RIGHT   = $08
.const IN_FIRE    = $10
.const IN_PAUSE   = $20
.const IN_ABORT   = $40

.const IN_SRC_AUTO   = 0
.const IN_SRC_JOY    = 1
.const IN_SRC_KEY    = 2
.const IN_SRC_PADDLE = 3

.ifndef IN_ZP
.const IN_ZP = $f8
.endif
.const in_state = IN_ZP
.const in_prev  = IN_ZP + 1
.const in_pad   = IN_ZP + 2
.const in_src   = IN_ZP + 3

input_init:
        lda #0
        sta in_state
        sta in_prev
        sta in_pad
        sta in_src
        rts

input_select:
        sta in_src
        rts

// Joystick Port 2: Bit 0..4 von $DC00, low = aktiv. Gleiche Bitlage wie IN_*.
//   Warum: Port A der CIA 1 ($DC00) ist doppelt belegt: die Ausgabebits waehlen Tastaturspalten, die Eingabeleitungen fuehren
//   zugleich die Joystickkontakte von Port 2.  Waehrend DDRA = $FF (Ausgang) liest man den eigenen Ausgangswert zurueck,
//   deshalb schalten wir Port A kurz auf Eingang.  Die Kontakte schliessen nach Masse: gedrueckt = Bit 0, daher das eor #$ff.
//   Bitbelegung: 0 oben, 1 unten, 2 links, 3 rechts, 4 Feuer - identisch zu IN_UP..IN_FIRE, es ist keine Umrechnung noetig.
input_joy:
        lda #$00
        sta CIA1_DDRA           // Port A auf Eingang: wir lesen den Joystick
        lda CIA1_PRA
        ldx #$ff
        stx CIA1_DDRA           // zurück auf Ausgang (Tastaturspalten)
        eor #$ff
        and #$1f
        rts

// Tastatur: Spalte in $DC00 wählen (low), Zeile in $DC01 lesen (low = gedrückt).
//   Tastaturmatrix 8 x 8: PA0..PA7 (Ausgang, low = Spalte aktiv) waehlen eine Spalte, PB0..PB7 (Eingang, low = Taste gedrueckt)
//   liefern die Zeilen.  Benutzte Tasten (Spalte/Zeile): Z = PA1/PB4, X = PA2/PB7, P = PA5/PB1, Leertaste = PA7/PB4,
//   RUN/STOP = PA7/PB7.  Rueckgabe in A und in in_pad.
input_key:
        lda #$ff
        sta CIA1_DDRA
        lda #$00
        sta CIA1_DDRB
        lda #0
        sta in_pad              // Zwischenmaske
        lda #%11111101          // Spalte 1: Z (Zeile 4)
        sta CIA1_PRA
        lda CIA1_PRB
        tax                     // Zeilen merken: Spalte 1 liefert Z (Zeile 4) und A (Zeile 2)
        and #%00010000
        bne +
        lda #IN_LEFT
        ora in_pad
        sta in_pad
+       txa
        and #%00000100          // A = runter (fuer die Namenseingabe)
        bne +
        lda #IN_DOWN
        ora in_pad
        sta in_pad
+       lda #%11111011          // Spalte 2: X (Zeile 7)
        sta CIA1_PRA
        lda CIA1_PRB
        and #%10000000
        bne +
        lda #IN_RIGHT
        ora in_pad
        sta in_pad
+       lda #%11011111          // Spalte 5: P (Zeile 1)
        sta CIA1_PRA
        lda CIA1_PRB
        and #%00000010
        bne +
        lda #IN_PAUSE
        ora in_pad
        sta in_pad
+       lda #%01111111          // Spalte 7: Leertaste (Zeile 4), RUN/STOP (Zeile 7)
        sta CIA1_PRA
        lda CIA1_PRB
        tax
        and #%00010000
        bne +
        lda #IN_FIRE
        ora in_pad
        sta in_pad
+       txa
        and #%01000000          // Q = hoch (Spalte 7, Zeile 6; fuer die Namenseingabe)
        bne +
        lda #IN_UP
        ora in_pad
        sta in_pad
+       txa
        and #%10000000
        bne +
        lda #IN_ABORT
        ora in_pad
        sta in_pad
+       lda #$ff
        sta CIA1_PRA            // Tastaturspalten wieder frei
        lda in_pad
        rts

// Paddle Port 2 ($D419 = POTX), ⚠ ungeprüft. Port 2 wählen: CIA1 PRA Bit 7..6 = %01.
//   Warum die Wartezeit: Der SID misst den Paddlewiderstand, indem er einen Kondensator auflaedt; der Wert ist erst nach etwa
//   256 Takten pro Messschritt gueltig.  Die Paddlezuordnung (Bits 7..6 von $DC00: %01 = Port 1, %10 = Port 2) ist ⚠ ungeprueft.
input_paddle:
        lda #$ff
        sta CIA1_DDRA
        lda #%01111111
        sta CIA1_PRA
        ldx #$80
-       dex                     // ≈ 500 µs Wartezeit, bis die Kondensatoren gemessen sind
        bne -
        lda SID_POTX
        eor #$ff                // Drehung: links = klein
        sta in_pad
        lda #$ff
        sta CIA1_PRA
        lda in_pad
        sec
        rts

// input_poll: Eingabemaske dieses Bildes.  Quelle nach in_src: Joystick und Tastatur werden bei IN_SRC_AUTO mit ODER vereinigt,
//   damit der Spieler einfach das benutzen kann, was er zur Hand hat.  IN_FIRE wird zur FLANKE: nur im ersten Aufruf, nachdem der
//   Knopf gedrueckt wurde, kommt das Bit durch; solange er gehalten wird, ist es unterdrueckt (in_prev merkt sich den Vorzustand).
//   Veraendert A, X, Y.  Etwa 120 Takte (Schaetzung).
input_poll:
        lda in_src
        cmp #IN_SRC_KEY
        beq .keyonly
        jsr input_joy
        sta in_state
        lda in_src
        cmp #IN_SRC_JOY
        beq .edge
        jsr input_key
        ora in_state
        jmp .store
.keyonly:
        jsr input_key
.store: sta in_state
.edge:  lda in_state            // Feuer nur bei neuem Druck melden
        tax                     // X = Rohzustand
        and in_prev
        and #IN_FIRE
        beq +
        txa
        and #(~IN_FIRE) & $ff   // Feuer wird gehalten: unterdrücken
        sta in_state
+       stx in_prev
        lda in_state
        rts

input_wait_release:
-       jsr input_poll
        and #$ff
        bne -
        rts

// ---------------------------------------------------------------------------------------------------------------------
// input_char: Zeicheneingabe von der Tastatur (fuer die Namenseingabe).  Liefert in A das ASCII-Zeichen einer NEU gedrueckten
//   Taste (Grossbuchstaben A-Z, Ziffern 0-9, Leerzeichen, '-' und '.'), $0D fuer RETURN, $14 fuer DEL (Loeschen), sonst 0.
//   Gehalten wird nicht wiederholt: erst nach dem Loslassen zaehlt dieselbe Taste wieder.  Veraendert X, Y.
//   Warum die Tabelle: Die Matrix ist 8 Spalten (PA0..PA7, low = gewaehlt) x 8 Zeilen (PB0..PB7, low = gedrueckt); der Tabellenindex ist
//   Spalte * 8 + Zeile.  Tasten, die wir nicht brauchen, stehen als 0 in der Tabelle.
// Zustand: in_ckey (zuletzt gelesene Taste, damit nur neue Druecke zaehlen)
// ---------------------------------------------------------------------------------------------------------------------
input_char:
        lda #$ff
        sta CIA1_DDRA
        lda #$00
        sta CIA1_DDRB
        ldx #0                          // X = Spalte 0..7
        lda #%11111110
        sta in_pad                      // Spaltenmaske (eine 0 wandert nach links)
.ic1:   lda in_pad
        sta CIA1_PRA
        lda CIA1_PRB
        cmp #$ff
        bne .icf                        // in dieser Spalte ist eine Taste gedrueckt
.ic2:   sec
        rol in_pad                      // Maske fuer die naechste Spalte
        inx
        cpx #8
        bne .ic1
        lda #$ff
        sta CIA1_PRA
        lda #0
        sta in_ckey                     // nichts gedrueckt: Taste gilt als losgelassen
        rts
.icf:   ldy #0                          // erste gedrueckte Zeile suchen
.ic3:   lsr
        bcc .icg                        // Bit war 0 = gedrueckt
        iny
        cpy #8
        bne .ic3
        beq .ic2                        // (kommt nicht vor)
.icg:   txa
        asl
        asl
        asl
        sty in_pad                      // Spaltenmaske wird nicht mehr gebraucht
        ora in_pad                      // A = Spalte * 8 + Zeile
        tax
        lda #$ff
        sta CIA1_PRA                    // Spalten wieder frei
        lda in_ktab,x                   // ASCII oder 0
        beq .icnone                     // nicht benutzte Taste
        cmp in_ckey
        beq .icsame                     // dieselbe Taste wie zuvor: nicht noch einmal melden
        sta in_ckey
        rts
.icsame: lda #0
        rts
.icnone: sta in_ckey                    // (A = 0)
        rts
in_ckey: .byte 0

// ASCII je Matrixposition (Spalte * 8 + Zeile); 0 = nicht benutzt.  $14 = DEL, $0D = RETURN.
in_ktab:
        .byte $14, $0d, 0, 0, 0, 0, 0, 0                         // PA0: DEL, RETURN, Cursor links/rechts, F7, F1, F3, F5, Cursor hoch/runter
        .byte '3', 'W', 'A', '4', 'Z', 'S', 'E', 0               // PA1: 3 W A 4 Z S E Shift-links
        .byte '5', 'R', 'D', '6', 'C', 'F', 'T', 'X'             // PA2
        .byte '7', 'Y', 'G', '8', 'B', 'H', 'U', 'V'             // PA3
        .byte '9', 'I', 'J', '0', 'M', 'K', 'O', 'N'             // PA4
        .byte 0, 'P', 'L', '-', '.', 0, 0, 0                     // PA5: + P L - . : @ ,
        .byte 0, 0, 0, 0, 0, 0, 0, 0                             // PA6: nicht benutzt
        .byte '1', 0, 0, '2', ' ', 0, 'Q', 0                     // PA7: 1 <- Ctrl 2 Leertaste C= Q RUN/STOP
