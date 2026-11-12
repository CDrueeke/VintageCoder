// SPDX-License-Identifier: GPL-3.0-or-later
// Eingabe-Bibliothek PET (PRALLBALL). Einbinden: .include "lib/input.asm"
//
// Der PET hat keinen Joystickanschluss; gespielt wird mit der Tastatur: Z oder , = links, X oder . = rechts, Leertaste = Feuer, P = Pause,
// RUN/STOP = Abbruch, A/Q = runter/hoch (fuer die Namenseingabe).
// Verfahren: Der Bildschirm-Editor (BASIC 4) liest die Tastatur in seinem Interrupt und legt den Code der gerade gehaltenen Taste in $97 ab
//   ($FF = keine Taste).  Der Code ist die ASCII-/PETSCII-Belegung der Taste, nicht die Matrixposition: dadurch laeuft dieselbe Abfrage auf der
//   Grafiktastatur (4032) und der Schreibmaschinentastatur (8032).  Der Editor traegt die Taste aber nur ein, solange der Tastaturpuffer ($9E)
//   leer ist; deshalb leert der Interrupt des Spiels den Puffer in jedem Bild (Schalter in_kbclr).
//   Gilt fuer BASIC 4 (4000er/8000er).  Auf BASIC 2 (2001, 3000er) steht die Taste nach dem gleichen Muster in $97, nicht geprueft.
//
// Vertrag (auf allen Systemen gleich): input_poll liefert in A eine Maske aus IN_*; Bit gesetzt = Aktion aktiv.
// IN_FIRE wird als Flanke geliefert (nur im Aufruf des Tastendrucks), der Rest als Zustand.
//   input_init        in_state/in_prev loeschen, Puffer leeren lassen (in_kbclr = 1)
//   input_poll        A = Maske; verändert X
//   input_char        Zeicheneingabe fuer die Namenseingabe: A = ASCII-Zeichen einer NEU gedrueckten Taste, $0D = RETURN, $14 = DEL, sonst 0;
//                     schaltet das Leeren des Tastaturpuffers fuer die Dauer der Namenseingabe aus (in_kbclr = 0, Aufrufer setzt es danach wieder)
//   input_wait_release  wartet, bis keine Taste mehr gehalten wird
// Zeropage (4 Byte, mit -D IN_ZP=... verschiebbar): in_state, in_prev, in_kbclr, in_ckey

.const IN_UP      = $01
.const IN_DOWN    = $02
.const IN_LEFT    = $04
.const IN_RIGHT   = $08
.const IN_FIRE    = $10
.const IN_PAUSE   = $20
.const IN_ABORT   = $40

.const KEYNOW     = $97         // Code der gehaltenen Taste ($FF = keine), vom Editor-Interrupt (BASIC 4)
.const KEYBUFN    = $9e         // Anzahl Zeichen im Tastaturpuffer

.ifndef IN_ZP
.const IN_ZP = $34
.endif
.const in_state = IN_ZP
.const in_prev  = IN_ZP + 1
.const in_kbclr = IN_ZP + 2
.const in_ckey  = IN_ZP + 3

input_init:
        lda #0
        sta in_state
        sta in_prev
        sta in_ckey
        lda #1
        sta in_kbclr
        rts

// input_poll: Eingabemaske dieses Bildes.  Feuer wird zur FLANKE (nur beim ersten Aufruf nach dem Druck), in_prev merkt sich den Vorzustand.
//   Tasten: Tabelle in_keytab (Taste, Aktion); gesucht wird die gehaltene Taste.  Verändert A, X.
input_poll:
        lda #0
        sta in_state
        lda KEYNOW
        cmp #$ff
        beq .edge               // keine Taste
        ldx #0
.s1:    cmp in_keytab,x
        beq .found
        inx
        inx
        ldy in_keytab,x         // Endemarke: Taste $00 (die Tabelle beginnt nie mit 0)
        bne .s1
        beq .edge
.found: lda in_keytab + 1,x
        sta in_state
.edge:  lda in_state            // Feuer nur bei neuem Druck melden
        tax                     // X = Rohzustand
        and in_prev
        and #IN_FIRE
        beq .nf
        txa
        and #(~IN_FIRE) & $ff   // Feuer wird gehalten: unterdruecken
        sta in_state
.nf:    stx in_prev
        lda in_state
        rts

// Tabelle: Tastencode (ASCII), Aktion.  Z/X und , . bewegen, Leertaste = Feuer, P = Pause, STOP ($03) = Abbruch, A/Q = runter/hoch.
in_keytab:
        .byte $5a, IN_LEFT              // Z
        .byte $2c, IN_LEFT              // ,
        .byte $58, IN_RIGHT             // X
        .byte $2e, IN_RIGHT             // .
        .byte $20, IN_FIRE              // Leertaste
        .byte $50, IN_PAUSE             // P
        .byte $03, IN_ABORT             // RUN/STOP
        .byte $41, IN_DOWN              // A
        .byte $51, IN_UP                // Q
        .byte $00

input_wait_release:
.w:     lda KEYNOW
        cmp #$ff
        bne .w
        rts

// input_char: Zeichen der Namenseingabe.  Liest den Tastaturpuffer des Editors selbst (Anzahl $9E, Zeichen ab $026F, BASIC 4); dafuer darf der Interrupt den
//   Puffer nicht leeren.  GETIN ($FFE4) taugt hier NICHT: Bei RETURN schaltet der Editor auf "Zeile vom Bildschirm lesen" um und liefert den Bildschirminhalt.
//   Liefert A-Z, 0-9, Leerzeichen, '-', '.', RETURN ($0D) und DEL ($14); anderes ergibt 0.  Gehalten wird nicht wiederholt (der Editor tut es ohnehin nicht).
.const KEYBUF = $026f
input_char:
        lda #0
        sta in_kbclr            // Puffer fuellen lassen
        lda KEYBUFN
        beq .none               // nichts gedrueckt
        sei
        lda KEYBUF              // erstes Zeichen entnehmen, Rest nachruecken
        pha
        ldx #0
.sh:    lda KEYBUF + 1,x
        sta KEYBUF,x
        inx
        cpx #9
        bne .sh
        dec KEYBUFN
        cli
        pla
        beq .ic9
        cmp #$0d
        beq .ic9
        cmp #$14
        beq .ic9
        cmp #$20
        beq .ic9
        cmp #$2d
        beq .ic9
        cmp #$2e
        beq .ic9
        cmp #$30
        bcc .none
        cmp #$3a
        bcc .ic9                // Ziffer
        cmp #$41
        bcc .none
        cmp #$5b
        bcc .ic9                // Buchstabe
.none:  lda #0
.ic9:   ora #0                  // Z-Flag nach A setzen (der Aufrufer prueft mit beq auf "keine Taste"; nach einem cmp waere Z irrefuehrend)
        rts
