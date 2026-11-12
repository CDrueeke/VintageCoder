// SPDX-License-Identifier: GPL-3.0-or-later
// Eingabe-Bibliothek Plus/4 (PRALLBALL). Einbinden: .include "lib/input.asm"
//
// Quellen: Joystick (TED-Tastaturlatch $FF08, beide Anschlüsse vereint), Tastatur (Z links, X rechts, Q hoch, A runter, Leertaste Feuer,
// P Pause, RUN/STOP Abbruch).  Ein Paddle gibt es am Plus/4 nicht; input_paddle liefert C=0 (ungültig).
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
.const IN_ZP = $8c
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

// Joystick: Der Plus/4 hat keine CIA; die Joystickkontakte liegen im selben Latch wie die Tastaturzeilen ($FF08).
//   Zugriff: Ins Register $FF08 wird eine Maske geschrieben, die einen Joystickanschluss ausblendet (low = Anschluss lesen); danach liefert $FF08
//   beim Lesen die Kontakte: Bit 0 oben, 1 unten, 2 links, 3 rechts (low = gedrueckt), Feuer bei Bit 6 (Anschluss, Maske $FB) bzw. Bit 7
//   (Anschluss, Maske $FD); wir werten beide Bits fuer beide Anschluesse.  Welcher Anschluss "Joystick 1/2" heisst, ist je nach Quelle unterschiedlich; wir lesen DESHALB BEIDE und
//   vereinen sie (wie bei C64 Joystick und Tastatur).  Damit keine Tastatur-Taste mitspielt, wird vorher $FD30 = $FF (keine Spalte) gesetzt.
input_joy:
        lda #$ff
        sta $fd30               // keine Tastaturspalte waehlen
        lda #$fb
        sta $ff08               // Anschluss A waehlen
        lda $ff08
        eor #$ff                // gedrueckt = 1
        sta in_pad              // Bits 0..3 Richtungen, Bits 6 und 7 Feuer (je nach Anschluss)
        lda #$fd
        sta $ff08               // Anschluss B waehlen
        lda $ff08
        eor #$ff
        ora in_pad              // beide Anschluesse vereinen: Richtungen und Feuer
        tax
        lda #$ff
        sta $ff08               // Latch wieder frei
        txa
        and #$c0                // Feuer: Bit 6 oder Bit 7 (die genaue Zuordnung je Anschluss ist nicht verifiziert: beide gelten)
        beq +
        lda #IN_FIRE
+       sta in_pad
        txa
        and #$0f                // Richtungen: Bit 0 oben, 1 unten, 2 links, 3 rechts = IN_UP, IN_DOWN, IN_LEFT, IN_RIGHT
        ora in_pad
        rts

// Tastatur: Spalte in $FD30 waehlen (low = gewaehlt), danach $FF08 schreiben (loest das Einlesen aus) und die Zeilen aus $FF08 lesen (low = gedrueckt).
//   Matrix 8 Spalten x 8 Zeilen (vom Tastaturdekodierer des KERNAL ueberprueft): Z = Spalte 1/Zeile 4, A = 1/2, X = 2/7, P = 5/1,
//   Q = 7/6, Leertaste = 7/4, RUN/STOP = 7/7.  Rueckgabe in A und in_pad.
input_key:
        lda #0
        sta in_pad              // Zwischenmaske
        lda #%11111101          // Spalte 1: A (Zeile 2), Z (Zeile 4)
        jsr in_col
        tax
        and #%00010000
        bne +
        lda #IN_LEFT
        ora in_pad
        sta in_pad
+       txa
        and #%00000100
        bne +
        lda #IN_DOWN
        ora in_pad
        sta in_pad
+       lda #%11111011          // Spalte 2: X (Zeile 7)
        jsr in_col
        and #%10000000
        bne +
        lda #IN_RIGHT
        ora in_pad
        sta in_pad
+       lda #%11011111          // Spalte 5: P (Zeile 1)
        jsr in_col
        and #%00000010
        bne +
        lda #IN_PAUSE
        ora in_pad
        sta in_pad
+       lda #%01111111          // Spalte 7: Leertaste (Zeile 4), Q (Zeile 6), RUN/STOP (Zeile 7)
        jsr in_col
        tax
        and #%00010000
        bne +
        lda #IN_FIRE
        ora in_pad
        sta in_pad
+       txa
        and #%01000000
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
        sta $fd30               // Spalten wieder frei
        lda in_pad
        rts

// in_col: A = Spaltenmaske (low = gewaehlt) -> A = Zeilen (low = gedrueckt).
in_col: sta $fd30
        sta $ff08               // Einlesen ausloesen
        lda $ff08
        rts

// Ein Paddle gibt es am Plus/4 nicht.
input_paddle:
        lda #$80
        sta in_pad
        clc                     // C=0: nicht gueltig
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
//   Die Tabelle in_ktab hat den Index Spalte * 8 + Zeile (Spalte = Bit von $FD30, Zeile = Bit von $FF08) und entspricht der Dekodiertabelle
//   des KERNAL (ROM $E02B), nur auf unsere Zeichen verkuerzt.
// Zustand: in_ckey (zuletzt gelesene Taste, damit nur neue Druecke zaehlen)
// ---------------------------------------------------------------------------------------------------------------------
input_char:
        ldx #0                          // X = Spalte 0..7
        lda #%11111110
        sta in_pad                      // Spaltenmaske (eine 0 wandert nach links)
.ic1:   lda in_pad
        jsr in_col
        cmp #$ff
        bne .icf                        // in dieser Spalte ist eine Taste gedrueckt
.ic2:   sec
        rol in_pad                      // Maske fuer die naechste Spalte
        inx
        cpx #8
        bne .ic1
        lda #$ff
        sta $fd30
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
        sta $fd30                       // Spalten wieder frei
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
        .byte $14, $0d, 0, 0, 0, 0, 0, 0                         // Spalte 0: DEL, RETURN, Pfund, HELP, F1, F2, F3, @
        .byte '3', 'W', 'A', '4', 'Z', 'S', 'E', 0               // Spalte 1: 3 W A 4 Z S E Shift
        .byte '5', 'R', 'D', '6', 'C', 'F', 'T', 'X'             // Spalte 2
        .byte '7', 'Y', 'G', '8', 'B', 'H', 'U', 'V'             // Spalte 3
        .byte '9', 'I', 'J', '0', 'M', 'K', 'O', 'N'             // Spalte 4
        .byte 0, 'P', 'L', 0, '.', 0, '-', 0                     // Spalte 5: unten P L oben . : - ,
        .byte 0, 0, 0, 0, 0, 0, 0, 0                             // Spalte 6: links * ; rechts ESC = + /
        .byte '1', 0, 0, '2', ' ', 0, 'Q', 0                     // Spalte 7: 1 HOME CTRL 2 Leertaste C= Q RUN/STOP
