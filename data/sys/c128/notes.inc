// notes.inc – Notenkonstanten und Frequenztabelle C128 (SID).
// ERZEUGT durch tools/gen_notes.py – nicht von Hand ändern (Änderungen im Skript, dann neu erzeugen; `--check` prüft).
// Einbinden: #include <c128/notes.inc>
//
// Notenwerte: NOTE_C0 = 1 … NOTE_A4 = 58 (440 Hz) … NOTE_B7 = 96, gleichstufig, wie im Tracker (C-4 = NOTE_C4).
// Schreibweise englisch: CS = Cis (C#), DB = Des (Db) usw.; B ist das deutsche H, BB das deutsche B (AS und BB sind derselbe Ton).
// Liedformat der Spielroutinen: NOTE_HOLD (0) = Note halten, NOTE_REST (96) = Pause (Gate aus).  Achtung: NOTE_B7 hat denselben Wert
// wie NOTE_REST; als Liednote nutzbar sind daher NOTE_C0 … NOTE_AS7 (1..95).  Als reiner Tabellenindex ist NOTE_B7 gültig.
// Die Tabellen sind mit dem Notenwert indiziert (Eintrag 0 = keine Note):  ldx #NOTE_A4 / lda note_..._lo,x.
// Selbst anlegen lässt sich ein Ton als  NOTE_C4 + 7  (Quinte) oder  NOTE_C4 + 12 * 2  (zwei Oktaven höher).
// PAL ist Standard; mit  -D NTSC=1  (oder .const NTSC = 1 vor dem Include) die NTSC-Tabelle.  Ohne Tabelle (nur Konstanten): NOTES_NO_TABLE definieren.
//
// SID-Tabellen: note_sid_lo / note_sid_hi, 16-Bit-Frequenzzähler Fn = f * 2^24 / Takt (PAL 985248 Hz, NTSC 1022727 Hz).
// Oberhalb von NOTE_TABLE_HIGHEST ist der Zähler voll (65535); höhere Noten klingen zu tief.  Registerlage wie beim C64 ($D400 ff.).
// Beispiel (Stimme 1):  ldx #NOTE_A4  /  lda note_sid_lo,x  /  sta $d400  /  lda note_sid_hi,x  /  sta $d401

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
.const NOTE_TABLE_HIGHEST = 96   // NTSC: höchster Ton ohne Kappung bei 65535 (darüber liefert die Tabelle 65535)
.else
.const NOTE_TABLE_HIGHEST = 95   // PAL: höchster Ton ohne Kappung bei 65535 (darüber liefert die Tabelle 65535)
.endif

.ifndef NOTES_NO_TABLE
.ifdef NTSC
// NTSC: Takt 1022727 Hz
// SID-Frequenzzähler (16 Bit), niedriges Byte: Fn = f * 2^24 / Takt
note_sid_lo:
        .byte $00
        .byte $0c, $1c, $2d, $3f, $52, $66, $7b, $92, $aa, $c3, $de, $fa
        .byte $18, $38, $5a, $7e, $a4, $cc, $f7, $24, $54, $86, $bc, $f5
        .byte $31, $71, $b4, $fc, $48, $98, $ed, $48, $a7, $0c, $78, $e9
        .byte $62, $e2, $69, $f8, $90, $30, $db, $8f, $4e, $19, $f0, $d3
        .byte $c4, $c3, $d1, $f0, $1f, $61, $b6, $1e, $9d, $32, $df, $a6
        .byte $88, $86, $a3, $e0, $3f, $c2, $6b, $3d, $3a, $64, $be, $4c
        .byte $0f, $0c, $46, $bf, $7d, $84, $d6, $7a, $73, $c8, $7d, $97
        .byte $1e, $18, $8b, $7f, $fb, $07, $ac, $f4, $e7, $8f, $f9, $2f
// SID-Frequenzzähler, hohes Byte
note_sid_hi:
        .byte $00
        .byte $01, $01, $01, $01, $01, $01, $01, $01, $01, $01, $01, $01
        .byte $02, $02, $02, $02, $02, $02, $02, $03, $03, $03, $03, $03
        .byte $04, $04, $04, $04, $05, $05, $05, $06, $06, $07, $07, $07
        .byte $08, $08, $09, $09, $0a, $0b, $0b, $0c, $0d, $0e, $0e, $0f
        .byte $10, $11, $12, $13, $15, $16, $17, $19, $1a, $1c, $1d, $1f
        .byte $21, $23, $25, $27, $2a, $2c, $2f, $32, $35, $38, $3b, $3f
        .byte $43, $47, $4b, $4f, $54, $59, $5e, $64, $6a, $70, $77, $7e
        .byte $86, $8e, $96, $9f, $a8, $b3, $bd, $c8, $d4, $e1, $ee, $fd
.else
// PAL: Takt 985248 Hz
// SID-Frequenzzähler (16 Bit), niedriges Byte: Fn = f * 2^24 / Takt
note_sid_lo:
        .byte $00
        .byte $16, $27, $39, $4b, $5f, $74, $8a, $a1, $ba, $d4, $f0, $0e
        .byte $2d, $4e, $71, $96, $be, $e7, $14, $42, $74, $a9, $e0, $1b
        .byte $5a, $9c, $e2, $2d, $7b, $cf, $27, $85, $e8, $51, $c1, $37
        .byte $b4, $38, $c4, $59, $f7, $9d, $4e, $0a, $d0, $a2, $81, $6d
        .byte $67, $70, $89, $b2, $ed, $3b, $9c, $13, $a0, $45, $02, $da
        .byte $ce, $e0, $11, $64, $da, $76, $39, $26, $40, $89, $04, $b4
        .byte $9c, $c0, $23, $c8, $b4, $eb, $72, $4c, $80, $12, $08, $68
        .byte $39, $80, $45, $90, $68, $d6, $e3, $99, $00, $24, $10, $ff
// SID-Frequenzzähler, hohes Byte
note_sid_hi:
        .byte $00
        .byte $01, $01, $01, $01, $01, $01, $01, $01, $01, $01, $01, $02
        .byte $02, $02, $02, $02, $02, $02, $03, $03, $03, $03, $03, $04
        .byte $04, $04, $04, $05, $05, $05, $06, $06, $06, $07, $07, $08
        .byte $08, $09, $09, $0a, $0a, $0b, $0c, $0d, $0d, $0e, $0f, $10
        .byte $11, $12, $13, $14, $15, $17, $18, $1a, $1b, $1d, $1f, $20
        .byte $22, $24, $27, $29, $2b, $2e, $31, $34, $37, $3a, $3e, $41
        .byte $45, $49, $4e, $52, $57, $5c, $62, $68, $6e, $75, $7c, $83
        .byte $8b, $93, $9c, $a5, $af, $b9, $c4, $d0, $dd, $ea, $f8, $ff
.endif
.endif

.endif
