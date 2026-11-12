// SPDX-License-Identifier: GPL-3.0-or-later
// Eingabe-Bibliothek VC-20 (PRALLBALL). Einbinden: .include "lib/input.asm"
//
// Quellen: Joystick (einziger Anschluss: hoch/runter/links/Feuer an VIA1 $9111 Bit 2..5, RECHTS an VIA2 $9120 Bit 7), Tastatur (VIA2: Spalten
// ueber $9120 (Port B, Ausgang, low = gewaehlt), Zeilen ueber $9121 (Port A, Eingang, low = gedrueckt)).  Ein Paddle ($9008) wird nicht benutzt.
//
// Vertrag (auf allen Systemen gleich): input_poll liefert in A eine Maske aus IN_*; Bit gesetzt = Aktion aktiv.  IN_FIRE wird als Flanke geliefert.
//   input_init        Zustand loeschen, Quelle = IN_SRC_AUTO
//   input_poll        A = Maske (Joystick und Tastatur vereint); veraendert X, Y
//   input_joy / input_key   A = Joystickmaske / Tastaturmaske
//   input_char        A = ASCII-Zeichen einer neu gedrueckten Taste (A-Z, 0-9, Leerzeichen, -, .), $0D RETURN, $14 DEL, sonst 0
//   input_wait_release  wartet, bis keine Aktion mehr aktiv ist
// Tasten: Z links, X rechts, Q hoch, A runter, Leertaste Feuer, P Pause, RUN/STOP Abbruch.
// Zeropage (4 Byte, -D IN_ZP=... verschiebbar): in_state, in_prev, in_pad, in_src
//
// VC-20-Besonderheit: Bit 7 von $9120 ist zugleich die Joystick-Leitung RECHTS.  Fuer die Tastatur muss die ganze Datenrichtung $9122 = $FF (alle Ausgang),
// fuer den Joystick wird Bit 7 kurz auf Eingang gestellt ($7F) und alle Spalten hochgelegt ($FF), damit keine Taste mitspielt.

.const IN_UP      = $01
.const IN_DOWN    = $02
.const IN_LEFT    = $04
.const IN_RIGHT   = $08
.const IN_FIRE    = $10
.const IN_PAUSE   = $20
.const IN_ABORT   = $40

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
        lda #$ff
        sta $9122               // VIA2 Port B: alle Spalten Ausgang (Tastaturabfrage)
        sta $9120
        lda #0
        sta $9123               // VIA2 Port A (Zeilen): Eingang
        // (VIA1 Port A bleibt unberuehrt: Bit 0/1/7 gehoeren dem seriellen Bus, Bit 2..5 sind schon Eingang)
        rts

// Joystick: links/oben/unten/Feuer aus $9111 (low = gedrueckt), rechts aus $9120 Bit 7.
input_joy:
        lda $9111
        eor #$ff                // gedrueckt = 1
        sta in_pad              // Bit 2 oben, 3 unten, 4 links, 5 Feuer
        lda #0
        sta $9122               // Port B ganz auf Eingang: auch Bit 7 (rechts) wird lesbar; die Tastaturspalten liegen dabei hoch (Pull-up)
        lda $9120
        pha
        lda #$ff
        sta $9122               // wieder Ausgang (Tastatur)
        sta $9120
        lda #0
        sta in_state
        lda in_pad
        and #$04                // oben (Bit 2) -> IN_UP
        beq +
        lda #IN_UP
        sta in_state
+       lda in_pad
        and #$08                // unten (Bit 3) -> IN_DOWN
        beq +
        lda #IN_DOWN
        ora in_state
        sta in_state
+       lda in_pad
        and #$10                // links (Bit 4) -> IN_LEFT
        beq +
        lda #IN_LEFT
        ora in_state
        sta in_state
+       lda in_pad
        and #$20                // Feuer (Bit 5) -> IN_FIRE
        beq +
        lda #IN_FIRE
        ora in_state
        sta in_state
+       pla
        bmi +                   // Bit 7 = 1: rechts NICHT gedrueckt (low = gedrueckt)
        lda #IN_RIGHT
        ora in_state
        sta in_state
+       lda in_state
        rts

// in_col: A = Spaltenmaske (low = gewaehlt) -> A = Zeilen (low = gedrueckt)
in_col: sta $9120
        lda $9121
        rts

// Tastatur: Matrix 8 Spalten x 8 Zeilen.  Benutzt: Z (Spalte 4/Zeile 1) links, X (3/2) rechts, Q (6/0) hoch, A (2/1) runter, Leertaste (4/0) Feuer,
// P (1/5) Pause, RUN/STOP (3/0) Abbruch.
input_key:
        lda #0
        sta in_pad
        lda #%11101111          // Spalte 4: Leertaste (Zeile 0), Z (Zeile 1)
        jsr in_col
        tax
        and #%00000010
        bne +
        lda #IN_LEFT
        ora in_pad
        sta in_pad
+       txa
        and #%00000001
        bne +
        lda #IN_FIRE
        ora in_pad
        sta in_pad
+       lda #%11110111          // Spalte 3: RUN/STOP (Zeile 0), X (Zeile 2)
        jsr in_col
        tax
        and #%00000100
        bne +
        lda #IN_RIGHT
        ora in_pad
        sta in_pad
+       txa
        and #%00000001
        bne +
        lda #IN_ABORT
        ora in_pad
        sta in_pad
+       lda #%11111101          // Spalte 1: P (Zeile 5)
        jsr in_col
        and #%00100000
        bne +
        lda #IN_PAUSE
        ora in_pad
        sta in_pad
+       lda #%11111011          // Spalte 2: A (Zeile 1)
        jsr in_col
        and #%00000010
        bne +
        lda #IN_DOWN
        ora in_pad
        sta in_pad
+       lda #%10111111          // Spalte 6: Q (Zeile 0)
        jsr in_col
        and #%00000001
        bne +
        lda #IN_UP
        ora in_pad
        sta in_pad
+       lda #$ff
        sta $9120               // Spalten wieder frei
        lda in_pad
        rts

// input_poll: Eingabemaske dieses Bildes: Joystick und Tastatur vereint; IN_FIRE wird zur FLANKE (nur beim neuen Druck).  Veraendert A, X, Y.
input_poll:
        jsr input_joy
        sta in_state
        jsr input_key
        ora in_state
        sta in_state
        tax                     // X = Rohzustand
        and in_prev
        and #IN_FIRE
        beq +
        txa
        and #(~IN_FIRE) & $ff   // Feuer wird gehalten: unterdruecken
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
// input_char: Zeicheneingabe von der Tastatur (Namenseingabe): ASCII einer NEU gedrueckten Taste, $0D RETURN, $14 DEL, sonst 0.  Veraendert X, Y.
//   Die Tabelle in_ktab (Index Spalte * 8 + Zeile) ist die Dekodiertabelle des KERNAL (ROM $EC5E), auf unsere Zeichen verkuerzt.
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
        rol in_pad
        inx
        cpx #8
        bne .ic1
        lda #$ff
        sta $9120
        lda #0
        sta in_ckey                     // nichts gedrueckt: Taste gilt als losgelassen
        rts
.icf:   ldy #0                          // erste gedrueckte Zeile suchen
.ic3:   lsr
        bcc .icg
        iny
        cpy #8
        bne .ic3
        beq .ic2
.icg:   txa
        asl
        asl
        asl
        sty in_pad
        ora in_pad                      // A = Spalte * 8 + Zeile
        tax
        lda #$ff
        sta $9120
        lda in_ktab,x
        beq .icnone
        cmp in_ckey
        beq .icsame
        sta in_ckey
        rts
.icsame: lda #0
        rts
.icnone: sta in_ckey
        rts
in_ckey: .byte 0

// ASCII je Matrixposition (Spalte * 8 + Zeile); 0 = nicht benutzt.  $14 = DEL, $0D = RETURN.
in_ktab:
        .byte '1', '3', '5', '7', '9', 0, 0, $14                 // Spalte 0: 1 3 5 7 9 + Pfund DEL
        .byte 0, 'W', 'R', 'Y', 'I', 'P', 0, $0d                 // Spalte 1: <- W R Y I P * RETURN
        .byte 0, 'A', 'D', 'G', 'J', 'L', 0, 0                   // Spalte 2: CTRL A D G J L ; Cursor rechts
        .byte 0, 0, 'X', 'V', 'N', 0, '/', 0                     // Spalte 3: STOP Shift X V N , / Cursor runter
        .byte ' ', 'Z', 'C', 'B', 'M', '.', 0, 0                 // Spalte 4: Leertaste Z C B M . Shift F1
        .byte 0, 'S', 'F', 'H', 'K', 0, 0, 0                     // Spalte 5: C= S F H K : = F3
        .byte 'Q', 'E', 'T', 'U', 'O', 0, 0, 0                   // Spalte 6: Q E T U O @ ^ F5
        .byte '2', '4', '6', '8', '0', '-', 0, 0                 // Spalte 7: 2 4 6 8 0 - Home F7
