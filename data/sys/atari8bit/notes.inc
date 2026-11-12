// notes.inc – Notenkonstanten und Frequenztabelle Atari 8 Bit (POKEY).
// ERZEUGT durch tools/gen_notes.py – nicht von Hand ändern (Änderungen im Skript, dann neu erzeugen; `--check` prüft).
// Einbinden: #include <atari8bit/notes.inc>
//
// Notenwerte: NOTE_C0 = 1 … NOTE_A4 = 58 (440 Hz) … NOTE_B7 = 96, gleichstufig, wie im Tracker (C-4 = NOTE_C4).
// Schreibweise englisch: CS = Cis (C#), DB = Des (Db) usw.; B ist das deutsche H, BB das deutsche B (AS und BB sind derselbe Ton).
// Liedformat der Spielroutinen: NOTE_HOLD (0) = Note halten, NOTE_REST (96) = Pause (Gate aus).  Achtung: NOTE_B7 hat denselben Wert
// wie NOTE_REST; als Liednote nutzbar sind daher NOTE_C0 … NOTE_AS7 (1..95).  Als reiner Tabellenindex ist NOTE_B7 gültig.
// Die Tabellen sind mit dem Notenwert indiziert (Eintrag 0 = keine Note):  ldx #NOTE_A4 / lda note_..._lo,x.
// Selbst anlegen lässt sich ein Ton als  NOTE_C4 + 7  (Quinte) oder  NOTE_C4 + 12 * 2  (zwei Oktaven höher).
// PAL ist Standard; mit  -D NTSC=1  (oder .const NTSC = 1 vor dem Include) die NTSC-Tabelle.  Ohne Tabelle (nur Konstanten): NOTES_NO_TABLE definieren.
//
// POKEY-Tabellen: note_pokey8 (AUDF, 8 Bit, 64-kHz-Takt: f = (Takt/28) / (2 (AUDF + 1))),
// note_pokey16_lo / note_pokey16_hi (16 Bit, gekoppelte Kanäle, voller Takt: f = Takt / (2 (AUDF + 7))).  Takt PAL 1773447 Hz, NTSC 1789790 Hz.
// Mit 8 Bit sind Töne unter NOTE_POKEY8_LOWEST nicht darstellbar (Tabelle $FF); die 16-Bit-Tabelle deckt alle 96 Noten ab.
// Reiner Ton ohne Verzerrung (AUDC Bits 5..7 = 10x) vorausgesetzt; mit Verzerrung ändert sich die Frequenz durch die Polynomzähler.
// Beispiel (Kanal 1, 8 Bit):  ldx #NOTE_A4  /  lda note_pokey8,x  /  sta $d200

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
.const NOTE_POKEY8_LOWEST = 37   // NTSC: tiefster mit 8 Bit darstellbarer Ton (darunter steht $FF)
.else
.const NOTE_POKEY8_LOWEST = 36   // PAL: tiefster mit 8 Bit darstellbarer Ton (darunter steht $FF)
.endif

.ifndef NOTES_NO_TABLE
.ifdef NTSC
// NTSC: Takt 1789790 Hz
// POKEY 8 Bit (64 kHz), AUDF-Wert; $FF = nicht darstellbar (tiefer als NOTE_POKEY8_LOWEST)
note_pokey8:
        .byte $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $f3, $e6, $d9, $cc, $c1, $b6, $ac, $a2, $99, $90, $88, $80
        .byte $79, $72, $6c, $66, $60, $5b, $55, $51, $4c, $48, $44, $40
        .byte $3c, $39, $35, $32, $2f, $2d, $2a, $28, $25, $23, $21, $1f
        .byte $1e, $1c, $1a, $19, $17, $16, $15, $13, $12, $11, $10, $0f
        .byte $0e, $0d, $0d, $0c, $0b, $0a, $0a, $09, $09, $08, $08, $07
// POKEY 16 Bit (Kanäle 1+2 oder 3+4 gekoppelt, Takt 1,79 MHz), niedriges Byte (AUDF1 bzw. AUDF3)
note_pokey16_lo:
        .byte $00
        .byte $c1, $c2, $6e, $be, $a7, $21, $24, $a8, $a6, $17, $f4, $38
        .byte $dd, $dd, $34, $db, $d0, $0d, $8e, $50, $4f, $88, $f7, $99
        .byte $6b, $6b, $96, $ea, $64, $03, $c4, $a5, $a4, $c0, $f8, $49
        .byte $b2, $32, $c8, $72, $2f, $fe, $de, $cf, $cf, $dd, $f8, $21
        .byte $56, $96, $e0, $35, $94, $fb, $6c, $e4, $64, $eb, $79, $0d
        .byte $a7, $47, $ed, $97, $46, $fa, $b2, $6e, $2e, $f2, $b9, $83
        .byte $50, $20, $f3, $c8, $a0, $7a, $56, $34, $14, $f5, $d9, $be
        .byte $a5, $8d, $76, $61, $4c, $39, $27, $16, $06, $f7, $e9, $db
// POKEY 16 Bit, hohes Byte (AUDF2 bzw. AUDF4)
note_pokey16_hi:
        .byte $00
        .byte $d5, $c9, $be, $b3, $a9, $a0, $97, $8e, $86, $7f, $77, $71
        .byte $6a, $64, $5f, $59, $54, $50, $4b, $47, $43, $3f, $3b, $38
        .byte $35, $32, $2f, $2c, $2a, $28, $25, $23, $21, $1f, $1d, $1c
        .byte $1a, $19, $17, $16, $15, $13, $12, $11, $10, $0f, $0e, $0e
        .byte $0d, $0c, $0b, $0b, $0a, $09, $09, $08, $08, $07, $07, $07
        .byte $06, $06, $05, $05, $05, $04, $04, $04, $04, $03, $03, $03
        .byte $03, $03, $02, $02, $02, $02, $02, $02, $02, $01, $01, $01
        .byte $01, $01, $01, $01, $01, $01, $01, $01, $01, $00, $00, $00
.else
// PAL: Takt 1773447 Hz
// POKEY 8 Bit (64 kHz), AUDF-Wert; $FF = nicht darstellbar (tiefer als NOTE_POKEY8_LOWEST)
note_pokey8:
        .byte $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
        .byte $f1, $e4, $d7, $cb, $bf, $b4, $aa, $a1, $98, $8f, $87, $7f
        .byte $78, $71, $6b, $65, $5f, $5a, $55, $50, $4b, $47, $43, $3f
        .byte $3c, $38, $35, $32, $2f, $2c, $2a, $27, $25, $23, $21, $1f
        .byte $1d, $1c, $1a, $18, $17, $16, $14, $13, $12, $11, $10, $0f
        .byte $0e, $0d, $0c, $0c, $0b, $0a, $0a, $09, $09, $08, $07, $07
// POKEY 16 Bit (Kanäle 1+2 oder 3+4 gekoppelt, Takt 1,79 MHz), niedriges Byte (AUDF1 bzw. AUDF3)
note_pokey16_lo:
        .byte $00
        .byte $ce, $ea, $b1, $1a, $1a, $ab, $c2, $5a, $6b, $ed, $dc, $30
        .byte $e3, $f1, $55, $09, $0a, $52, $de, $aa, $b2, $f3, $6a, $14
        .byte $ee, $f5, $27, $81, $01, $a5, $6b, $51, $55, $76, $b2, $07
        .byte $74, $f7, $90, $3d, $fd, $cf, $b2, $a5, $a7, $b8, $d5, $00
        .byte $36, $78, $c5, $1b, $7b, $e4, $56, $cf, $50, $d8, $67, $fc
        .byte $98, $39, $df, $8a, $3a, $ef, $a7, $64, $25, $e9, $b0, $7b
        .byte $48, $19, $ec, $c2, $9a, $74, $50, $2f, $0f, $f1, $d5, $ba
        .byte $a1, $89, $72, $5d, $49, $36, $25, $14, $04, $f5, $e7, $d9
// POKEY 16 Bit, hohes Byte (AUDF2 bzw. AUDF4)
note_pokey16_hi:
        .byte $00
        .byte $d3, $c7, $bc, $b2, $a8, $9e, $95, $8d, $85, $7d, $76, $70
        .byte $69, $63, $5e, $59, $54, $4f, $4a, $46, $42, $3e, $3b, $38
        .byte $34, $31, $2f, $2c, $2a, $27, $25, $23, $21, $1f, $1d, $1c
        .byte $1a, $18, $17, $16, $14, $13, $12, $11, $10, $0f, $0e, $0e
        .byte $0d, $0c, $0b, $0b, $0a, $09, $09, $08, $08, $07, $07, $06
        .byte $06, $06, $05, $05, $05, $04, $04, $04, $04, $03, $03, $03
        .byte $03, $03, $02, $02, $02, $02, $02, $02, $02, $01, $01, $01
        .byte $01, $01, $01, $01, $01, $01, $01, $01, $01, $00, $00, $00
.endif
.endif

.endif
