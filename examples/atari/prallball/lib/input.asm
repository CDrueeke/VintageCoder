// SPDX-License-Identifier: GPL-3.0-or-later
// Eingabe-Bibliothek Atari 8 Bit (PRALLBALL). Einbinden: .include "lib/input.asm"
//
// Quellen: Joystick Port 1 (PIA PORTA $D300, Bits 0..3; Feuerknopf TRIG0 $D010), Tastatur (POKEY KBCODE $D209 / OS-Zeichen CH $02FC),
// Paddle Port 1 (POKEY POT0 $D200, ⚠ nicht auf echter Hardware geprüft).
//
// Vertrag (auf allen Systemen gleich): input_poll liefert in A eine Maske aus IN_*; Bit gesetzt = Aktion aktiv.  IN_FIRE wird als Flanke geliefert.
//   input_init        Zustand löschen, Quelle = IN_SRC_AUTO
//   input_select      A = IN_SRC_AUTO/JOY/KEY/PADDLE
//   input_poll        A = Maske; verändert X, Y
//   input_joy         A = Joystickmaske (ohne Pause/Abbruch)
//   input_key         A = Tastaturmaske (Z links, X rechts, Q hoch, A runter, Leertaste Feuer, P Pause, ESC Abbruch)
//   input_char        A = ASCII-Zeichen einer neu gedrückten Taste (A-Z, 0-9, Leerzeichen, -, .), $0D RETURN, $14 DEL, sonst 0
//   input_paddle      A = Position 0..255, C=1 gültig
//   input_wait_release  wartet, bis keine Aktion mehr aktiv ist
// Zeropage (4 Byte, mit -D IN_ZP=... verschiebbar): in_state, in_prev, in_pad, in_src
//
// Atari-Besonderheiten:
//  * Der Joystick-Port zeigt gedrückt = 0 (PORTA Bit 0 oben, 1 unten, 2 links, 3 rechts); TRIG0 = 0 bei Feuer.
//  * Die Tastatur lesen wir über das Betriebssystem: es legt den Tastencode eines Drucks in CH ($02FC) ab ($FF = keine Taste); wir lesen CH und
//    löschen es (auf $FF), das ergibt „neu gedrückt“ ohne Wiederholung durch das OS-Tastaturfeld.  Gedrückt gehaltene Tasten (Z/X zum Steuern)
//    brauchen den Dauerzustand: dafür fragen wir zusätzlich SKSTAT ($D20F, Bit 2 = 0: eine Taste ist gedrückt) und KBCODE ($D209) direkt.
.const PORTA   = $d300
.const TRIG0   = $d010
.const POT0    = $d200
.const POTGO   = $d20b
.const KBCODE  = $d209
.const SKSTAT  = $d20f
.const CH_OS   = $02fc                  // OS-Tastaturpuffer (1 Zeichen, $FF = leer)

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
.const IN_ZP = $d0
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
        sta CH_OS
        sta in_ckey
        rts

input_select:
        sta in_src
        rts

// Joystick: PORTA Bit 0..3 = oben, unten, links, rechts (0 = gedrückt) -> die Bitlage entspricht IN_UP..IN_RIGHT.
// Feuer: TRIG0 = 0 bei gedrücktem Knopf.
input_joy:
        lda PORTA
        eor #$ff
        and #$0f                        // Richtungen 1 = aktiv
        ldx TRIG0
        bne +
        ora #IN_FIRE
+       rts

// Tastatur (Dauerzustand): SKSTAT Bit 2 = 0 solange eine Taste gedrückt ist; KBCODE enthält dann den Code dieser Taste (Bit 6 = Shift,
// Bit 7 = Control).  Tastencodes der benutzten Tasten (ohne Shift/Control): Z = $17, X = $16, Q = $2F, A = $3F, P = $0A, Leertaste = $21,
// ESC = $1C.  Es wird nur EINE gleichzeitig gedrückte Taste erkannt (Hardware: KBCODE kennt nur die zuletzt erkannte Taste).
input_key:
        lda #0
        sta in_pad
        lda SKSTAT
        and #%00000100
        bne .ikend                      // Bit 2 gesetzt = keine Taste
        lda KBCODE
        and #$3f
        ldx #0
.ik1:   cmp in_kcode,x
        beq .ikf
        inx
        cpx #7
        bne .ik1
        beq .ikend
.ikf:   lda in_kmask,x
        sta in_pad
.ikend: lda in_pad
        rts
in_kcode: .byte $17, $16, $2f, $3f, $21, $0a, $1c           // Z X Q A Leertaste P ESC (ohne Shift/Control)
in_kmask: .byte IN_LEFT, IN_RIGHT, IN_UP, IN_DOWN, IN_FIRE, IN_PAUSE, IN_ABORT

// Paddle: POT0 liefert 0..228 (links ... rechts? auf dem Paddle drehend); ⚠ ungeprüft.  POTGO startet die nächste Abtastung.
input_paddle:
        lda POT0
        sta in_pad
        sta POTGO
        lda in_pad
        sec
        rts

// input_poll: Quelle nach in_src; bei IN_SRC_AUTO werden Joystick und Tastatur mit ODER vereinigt.  IN_FIRE wird zur Flanke.
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
.edge:  lda in_state
        tax
        and in_prev
        and #IN_FIRE
        beq +
        txa
        and #(~IN_FIRE) & $ff
        sta in_state
+       stx in_prev
        lda in_state
        rts

input_wait_release:
-       jsr input_poll
        and #$ff
        bne -
        rts

// input_char: Zeicheneingabe für die Namenseingabe.  Nutzt den OS-Tastaturpuffer CH: Das OS trägt dort den Tastencode jedes NEUEN Drucks ein;
// wir wandeln ihn per Tabelle (Index = Tastencode ohne Shift/Control, 0..63) in ASCII und löschen CH.  Nicht benutzte Tasten ergeben 0.
//   Tastencodes (Atari): L=$00 J=$01 ;=$02 K=$05 +=$06 *=$07 O=$08 P=$0A U=$0B RETURN=$0C I=$0D -=$0E =$0F V=$10 C=$12 B=$15 X=$16 Z=$17
//   4=$18 3=$1A 6=$1B 5=$1D 2=$1E 1=$1F ,=$20 Leertaste=$21 .=$22 N=$23 M=$25 /=$26 R=$28 E=$2A Y=$2B T=$2D W=$2E Q=$2F 9=$30 0=$32 7=$33
//   BACKSPACE=$34 8=$35 G=$3D H=$39 D=$3A S=$3E A=$3F F=$38
input_char:
        ldx CH_OS
        cpx #$ff
        beq .icdirect                   // der OS-Puffer ist leer: Hardware direkt abfragen (siehe unten)
        lda #$ff
        sta CH_OS                       // Taste uebernehmen (CH leeren)
        txa
        and #$3f
        tax
        cpx in_ckey
        beq .icnone                     // dieselbe Taste hat die Direktabfrage schon gemeldet: nicht doppelt
        stx in_ckey
        lda in_ktab,x                   // ASCII oder 0
        rts
// Direktabfrage der Tastaturhardware (falls die Tastatur-Unterbrechung des OS nicht arbeitet oder die Taste noch nicht im Puffer steht):
//   SKSTAT Bit 2 = 0, solange eine Taste gedrueckt ist; KBCODE enthaelt dann ihren Code.  in_ckey merkt die zuletzt gemeldete Taste, damit eine gehaltene
//   Taste nur einmal zaehlt (erst nach dem Loslassen wieder).
.icdirect:
        lda SKSTAT
        and #%00000100
        beq .icdown
        lda #$ff
        sta in_ckey                     // keine Taste gedrueckt: naechster Druck zaehlt neu
.icnone:
        lda #0
        rts
.icdown:
        lda KBCODE
        and #$3f
        cmp in_ckey
        beq .icnone                     // schon gemeldet
        sta in_ckey
        tax
        lda in_ktab,x
        rts
in_ckey: .byte $ff                     // zuletzt gemeldeter Tastencode ($FF = keine)

// Atari-Tastencode (0..63) -> ASCII.  Lücken = 0.  Gültig: Buchstaben A-Z, Ziffern 0-9, Leerzeichen ($21), '-' ($0E), '.' ($22), RETURN ($0C) -> $0D,
// BACKSPACE ($34) -> $14.
in_ktab:
        .byte 'L', 'J', ';',  0,  0, 'K',  0,  0, 'O',  0, 'P', 'U', $0d, 'I', '-',  0       // $00..$0f
        .byte 'V',  0, 'C',  0,  0, 'B', 'X', 'Z', '4',  0, '3', '6',  0, '5', '2', '1'       // $10..$1f
        .byte  0, ' ', '.', 'N',  0, 'M',  0,  0, 'R',  0, 'E', 'Y',  0, 'T', 'W', 'Q'       // $20..$2f
        .byte '9',  0, '0', '7', $14, '8',  0,  0, 'F', 'H', 'D',  0,  0, 'G', 'S', 'A'       // $30..$3f
