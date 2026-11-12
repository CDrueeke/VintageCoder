// notes.inc – Notenkonstanten und Frequenztabelle Plus/4 (TED).
// ERZEUGT durch tools/gen_notes.py – nicht von Hand ändern (Änderungen im Skript, dann neu erzeugen; `--check` prüft).
// Einbinden: #include <plus4/notes.inc>
//
// Notenwerte: NOTE_C0 = 1 … NOTE_A4 = 58 (440 Hz) … NOTE_B7 = 96, gleichstufig, wie im Tracker (C-4 = NOTE_C4).
// Schreibweise englisch: CS = Cis (C#), DB = Des (Db) usw.; B ist das deutsche H, BB das deutsche B (AS und BB sind derselbe Ton).
// Liedformat der Spielroutinen: NOTE_HOLD (0) = Note halten, NOTE_REST (96) = Pause (Gate aus).  Achtung: NOTE_B7 hat denselben Wert
// wie NOTE_REST; als Liednote nutzbar sind daher NOTE_C0 … NOTE_AS7 (1..95).  Als reiner Tabellenindex ist NOTE_B7 gültig.
// Die Tabellen sind mit dem Notenwert indiziert (Eintrag 0 = keine Note):  ldx #NOTE_A4 / lda note_..._lo,x.
// Selbst anlegen lässt sich ein Ton als  NOTE_C4 + 7  (Quinte) oder  NOTE_C4 + 12 * 2  (zwei Oktaven höher).
// PAL ist Standard; mit  -D NTSC=1  (oder .const NTSC = 1 vor dem Include) die NTSC-Tabelle.  Ohne Tabelle (nur Konstanten): NOTES_NO_TABLE definieren.
//
// TED-Tabellen: note_ted_lo / note_ted_hi, 10-Bit-Tonwert N, f = (Takt / 8) / (1024 - N)  (PAL 886724 Hz, NTSC 894886 Hz Takt).
// Der TED erreicht nur etwa 108 Hz (N = 0) nach unten; Töne unterhalb von NOTE_TABLE_LOWEST stehen um ganze Oktaven nach oben gefaltet.
// Registerlage: Stimme 1 $FF0E (niedrig) / $FF12 Bits 0..1 (hoch), Stimme 2 $FF0F / $FF10.  Bit 2 und höher von $FF12 gehören dem Zeichensatz – nur maskiert schreiben.

.ifndef NOTE_C0

.const NOTE_HOLD = 0
.const NOTE_REST = 96
.const NOTE_FIRST = 1
.const NOTE_LAST = 96

.const NOTE_C0 = 1
.const NOTE_CS0 = 2
.const NOTE_DB0 = 2
.const NOTE_D0 = 3
.const NOTE_DS0 = 4
.const NOTE_EB0 = 4
.const NOTE_E0 = 5
.const NOTE_F0 = 6
.const NOTE_FS0 = 7
.const NOTE_GB0 = 7
.const NOTE_G0 = 8
.const NOTE_GS0 = 9
.const NOTE_AB0 = 9
.const NOTE_A0 = 10
.const NOTE_AS0 = 11
.const NOTE_BB0 = 11
.const NOTE_B0 = 12
.const NOTE_C1 = 13
.const NOTE_CS1 = 14
.const NOTE_DB1 = 14
.const NOTE_D1 = 15
.const NOTE_DS1 = 16
.const NOTE_EB1 = 16
.const NOTE_E1 = 17
.const NOTE_F1 = 18
.const NOTE_FS1 = 19
.const NOTE_GB1 = 19
.const NOTE_G1 = 20
.const NOTE_GS1 = 21
.const NOTE_AB1 = 21
.const NOTE_A1 = 22
.const NOTE_AS1 = 23
.const NOTE_BB1 = 23
.const NOTE_B1 = 24
.const NOTE_C2 = 25
.const NOTE_CS2 = 26
.const NOTE_DB2 = 26
.const NOTE_D2 = 27
.const NOTE_DS2 = 28
.const NOTE_EB2 = 28
.const NOTE_E2 = 29
.const NOTE_F2 = 30
.const NOTE_FS2 = 31
.const NOTE_GB2 = 31
.const NOTE_G2 = 32
.const NOTE_GS2 = 33
.const NOTE_AB2 = 33
.const NOTE_A2 = 34
.const NOTE_AS2 = 35
.const NOTE_BB2 = 35
.const NOTE_B2 = 36
.const NOTE_C3 = 37
.const NOTE_CS3 = 38
.const NOTE_DB3 = 38
.const NOTE_D3 = 39
.const NOTE_DS3 = 40
.const NOTE_EB3 = 40
.const NOTE_E3 = 41
.const NOTE_F3 = 42
.const NOTE_FS3 = 43
.const NOTE_GB3 = 43
.const NOTE_G3 = 44
.const NOTE_GS3 = 45
.const NOTE_AB3 = 45
.const NOTE_A3 = 46
.const NOTE_AS3 = 47
.const NOTE_BB3 = 47
.const NOTE_B3 = 48
.const NOTE_C4 = 49
.const NOTE_CS4 = 50
.const NOTE_DB4 = 50
.const NOTE_D4 = 51
.const NOTE_DS4 = 52
.const NOTE_EB4 = 52
.const NOTE_E4 = 53
.const NOTE_F4 = 54
.const NOTE_FS4 = 55
.const NOTE_GB4 = 55
.const NOTE_G4 = 56
.const NOTE_GS4 = 57
.const NOTE_AB4 = 57
.const NOTE_A4 = 58
.const NOTE_AS4 = 59
.const NOTE_BB4 = 59
.const NOTE_B4 = 60
.const NOTE_C5 = 61
.const NOTE_CS5 = 62
.const NOTE_DB5 = 62
.const NOTE_D5 = 63
.const NOTE_DS5 = 64
.const NOTE_EB5 = 64
.const NOTE_E5 = 65
.const NOTE_F5 = 66
.const NOTE_FS5 = 67
.const NOTE_GB5 = 67
.const NOTE_G5 = 68
.const NOTE_GS5 = 69
.const NOTE_AB5 = 69
.const NOTE_A5 = 70
.const NOTE_AS5 = 71
.const NOTE_BB5 = 71
.const NOTE_B5 = 72
.const NOTE_C6 = 73
.const NOTE_CS6 = 74
.const NOTE_DB6 = 74
.const NOTE_D6 = 75
.const NOTE_DS6 = 76
.const NOTE_EB6 = 76
.const NOTE_E6 = 77
.const NOTE_F6 = 78
.const NOTE_FS6 = 79
.const NOTE_GB6 = 79
.const NOTE_G6 = 80
.const NOTE_GS6 = 81
.const NOTE_AB6 = 81
.const NOTE_A6 = 82
.const NOTE_AS6 = 83
.const NOTE_BB6 = 83
.const NOTE_B6 = 84
.const NOTE_C7 = 85
.const NOTE_CS7 = 86
.const NOTE_DB7 = 86
.const NOTE_D7 = 87
.const NOTE_DS7 = 88
.const NOTE_EB7 = 88
.const NOTE_E7 = 89
.const NOTE_F7 = 90
.const NOTE_FS7 = 91
.const NOTE_GB7 = 91
.const NOTE_G7 = 92
.const NOTE_GS7 = 93
.const NOTE_AB7 = 93
.const NOTE_A7 = 94
.const NOTE_AS7 = 95
.const NOTE_BB7 = 95
.const NOTE_B7 = 96

.ifdef NTSC
.const NOTE_TABLE_LOWEST = 34   // NTSC: tiefster Ton ohne Oktavfaltung (darunter steht der Eintrag ganze Oktaven höher)
.else
.const NOTE_TABLE_LOWEST = 34   // PAL: tiefster Ton ohne Oktavfaltung (darunter steht der Eintrag ganze Oktaven höher)
.endif

.ifndef NOTES_NO_TABLE
.ifdef NTSC
// NTSC: Takt 894886 Hz
// TED-Tonwert N (10 Bit), niedriges Byte: N = 1024 - Takt/8 / f
note_ted_lo:
        .byte $00
        .byte $a9, $d9, $06, $31, $59, $7f, $a3, $c5, $e5, $07, $40, $76
        .byte $a9, $d9, $06, $31, $59, $7f, $a3, $c5, $e5, $07, $40, $76
        .byte $a9, $d9, $06, $31, $59, $7f, $a3, $c5, $e5, $07, $40, $76
        .byte $a9, $d9, $06, $31, $59, $7f, $a3, $c5, $e5, $04, $20, $3b
        .byte $54, $6c, $83, $98, $ad, $c0, $d2, $e3, $f3, $02, $10, $1e
        .byte $2a, $36, $42, $4c, $56, $60, $69, $71, $79, $81, $88, $8f
        .byte $95, $9b, $a1, $a6, $ab, $b0, $b4, $b9, $bd, $c0, $c4, $c7
        .byte $cb, $ce, $d0, $d3, $d6, $d8, $da, $dc, $de, $e0, $e2, $e4
// TED-Tonwert N, obere 2 Bit
note_ted_hi:
        .byte $00
        .byte $00, $00, $01, $01, $01, $01, $01, $01, $01, $00, $00, $00
        .byte $00, $00, $01, $01, $01, $01, $01, $01, $01, $00, $00, $00
        .byte $00, $00, $01, $01, $01, $01, $01, $01, $01, $00, $00, $00
        .byte $00, $00, $01, $01, $01, $01, $01, $01, $01, $02, $02, $02
        .byte $02, $02, $02, $02, $02, $02, $02, $02, $02, $03, $03, $03
        .byte $03, $03, $03, $03, $03, $03, $03, $03, $03, $03, $03, $03
        .byte $03, $03, $03, $03, $03, $03, $03, $03, $03, $03, $03, $03
        .byte $03, $03, $03, $03, $03, $03, $03, $03, $03, $03, $03, $03
.else
// PAL: Takt 886724 Hz
// TED-Tonwert N (10 Bit), niedriges Byte: N = 1024 - Takt/8 / f
note_ted_lo:
        .byte $00
        .byte $b1, $e0, $0d, $37, $5f, $85, $a9, $ca, $ea, $10, $49, $7e
        .byte $b1, $e0, $0d, $37, $5f, $85, $a9, $ca, $ea, $10, $49, $7e
        .byte $b1, $e0, $0d, $37, $5f, $85, $a9, $ca, $ea, $10, $49, $7e
        .byte $b1, $e0, $0d, $37, $5f, $85, $a9, $ca, $ea, $08, $24, $3f
        .byte $58, $70, $87, $9c, $b0, $c3, $d4, $e5, $f5, $04, $12, $20
        .byte $2c, $38, $43, $4e, $58, $61, $6a, $73, $7b, $82, $89, $90
        .byte $96, $9c, $a2, $a7, $ac, $b1, $b5, $b9, $bd, $c1, $c5, $c8
        .byte $cb, $ce, $d1, $d3, $d6, $d8, $db, $dd, $df, $e1, $e2, $e4
// TED-Tonwert N, obere 2 Bit
note_ted_hi:
        .byte $00
        .byte $00, $00, $01, $01, $01, $01, $01, $01, $01, $00, $00, $00
        .byte $00, $00, $01, $01, $01, $01, $01, $01, $01, $00, $00, $00
        .byte $00, $00, $01, $01, $01, $01, $01, $01, $01, $00, $00, $00
        .byte $00, $00, $01, $01, $01, $01, $01, $01, $01, $02, $02, $02
        .byte $02, $02, $02, $02, $02, $02, $02, $02, $02, $03, $03, $03
        .byte $03, $03, $03, $03, $03, $03, $03, $03, $03, $03, $03, $03
        .byte $03, $03, $03, $03, $03, $03, $03, $03, $03, $03, $03, $03
        .byte $03, $03, $03, $03, $03, $03, $03, $03, $03, $03, $03, $03
.endif
.endif

.endif
