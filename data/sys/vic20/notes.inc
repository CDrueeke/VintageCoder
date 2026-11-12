// notes.inc – Notenkonstanten und Frequenztabelle VC-20 (VIC-I).
// ERZEUGT durch tools/gen_notes.py – nicht von Hand ändern (Änderungen im Skript, dann neu erzeugen; `--check` prüft).
// Einbinden: #include <vic20/notes.inc>
//
// Notenwerte: NOTE_C0 = 1 … NOTE_A4 = 58 (440 Hz) … NOTE_B7 = 96, gleichstufig, wie im Tracker (C-4 = NOTE_C4).
// Schreibweise englisch: CS = Cis (C#), DB = Des (Db) usw.; B ist das deutsche H, BB das deutsche B (AS und BB sind derselbe Ton).
// Liedformat der Spielroutinen: NOTE_HOLD (0) = Note halten, NOTE_REST (96) = Pause (Gate aus).  Achtung: NOTE_B7 hat denselben Wert
// wie NOTE_REST; als Liednote nutzbar sind daher NOTE_C0 … NOTE_AS7 (1..95).  Als reiner Tabellenindex ist NOTE_B7 gültig.
// Die Tabellen sind mit dem Notenwert indiziert (Eintrag 0 = keine Note):  ldx #NOTE_A4 / lda note_..._lo,x.
// Selbst anlegen lässt sich ein Ton als  NOTE_C4 + 7  (Quinte) oder  NOTE_C4 + 12 * 2  (zwei Oktaven höher).
// PAL ist Standard; mit  -D NTSC=1  (oder .const NTSC = 1 vor dem Include) die NTSC-Tabelle.  Ohne Tabelle (nur Konstanten): NOTES_NO_TABLE definieren.
//
// VIC-I-Tabellen: note_vic_sopr ($900C), note_vic_alto ($900B), note_vic_bass ($900A); Wert = $80 | N (Bit 7 = Stimme an), f = Takt * m / (256 * (127 - N)).
// m = 4 (Sopran), 2 (Alt), 1 (Bass); Takt PAL 1108405 Hz, NTSC 1022727 Hz.  Jede Stimme hat nur etwa 7 Oktaven Umfang:
// Töne außerhalb (Bass tief, Sopran sehr tief/hoch) sind um ganze Oktaven in den Bereich verschoben – die Tonart bleibt, die Lage nicht.
// Das Rauschen ($900D) hat keine Tonhöhe und hier keine Tabelle.

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

.ifndef NOTES_NO_TABLE
.ifdef NTSC
// NTSC: Takt 1022727 Hz
// VIC-I-Registerwerte ($80 | N), Sopran ($900C)
note_vic_sopr:
        .byte $00
        .byte $85, $8c, $92, $98, $9e, $a3, $a9, $ad, $b2, $b6, $ba, $be
        .byte $85, $8c, $92, $98, $9e, $a3, $a9, $ad, $b2, $b6, $ba, $be
        .byte $85, $8c, $92, $98, $9e, $a3, $a9, $ad, $b2, $b6, $ba, $be
        .byte $85, $8c, $92, $98, $9e, $a3, $a9, $ad, $b2, $b6, $ba, $be
        .byte $c2, $c5, $c9, $cc, $cf, $d1, $d4, $d6, $d9, $db, $dd, $df
        .byte $e0, $e2, $e4, $e5, $e7, $e8, $e9, $eb, $ec, $ed, $ee, $ef
        .byte $f0, $f1, $f1, $f2, $f3, $f4, $f4, $f5, $f5, $f6, $f6, $f7
        .byte $f7, $f8, $f8, $f9, $f9, $f9, $fa, $fa, $fa, $fa, $fb, $fb
// VIC-I-Registerwerte ($80 | N), Alt ($900B)
note_vic_alto:
        .byte $00
        .byte $85, $8c, $92, $98, $9e, $a3, $a9, $ad, $b2, $b6, $ba, $be
        .byte $85, $8c, $92, $98, $9e, $a3, $a9, $ad, $b2, $b6, $ba, $be
        .byte $85, $8c, $92, $98, $9e, $a3, $a9, $ad, $b2, $b6, $ba, $be
        .byte $c2, $c5, $c9, $cc, $cf, $d1, $d4, $d6, $d9, $db, $dd, $df
        .byte $e0, $e2, $e4, $e5, $e7, $e8, $e9, $eb, $ec, $ed, $ee, $ef
        .byte $f0, $f1, $f1, $f2, $f3, $f4, $f4, $f5, $f5, $f6, $f6, $f7
        .byte $f7, $f8, $f8, $f9, $f9, $f9, $fa, $fa, $fa, $fa, $fb, $fb
        .byte $fb, $fb, $fc, $fc, $fc, $fc, $fc, $fc, $fd, $fd, $fd, $fd
// VIC-I-Registerwerte ($80 | N), Bass ($900A)
note_vic_bass:
        .byte $00
        .byte $85, $8c, $92, $98, $9e, $a3, $a9, $ad, $b2, $b6, $ba, $be
        .byte $85, $8c, $92, $98, $9e, $a3, $a9, $ad, $b2, $b6, $ba, $be
        .byte $c2, $c5, $c9, $cc, $cf, $d1, $d4, $d6, $d9, $db, $dd, $df
        .byte $e0, $e2, $e4, $e5, $e7, $e8, $e9, $eb, $ec, $ed, $ee, $ef
        .byte $f0, $f1, $f1, $f2, $f3, $f4, $f4, $f5, $f5, $f6, $f6, $f7
        .byte $f7, $f8, $f8, $f9, $f9, $f9, $fa, $fa, $fa, $fa, $fb, $fb
        .byte $fb, $fb, $fc, $fc, $fc, $fc, $fc, $fc, $fd, $fd, $fd, $fd
        .byte $fd, $fd, $fd, $fd, $fd, $fe, $fe, $fe, $fe, $fe, $fe, $fe
.else
// PAL: Takt 1108405 Hz
// VIC-I-Registerwerte ($80 | N), Sopran ($900C)
note_vic_sopr:
        .byte $00
        .byte $bd, $82, $89, $90, $96, $9c, $a1, $a7, $ac, $b0, $b5, $b9
        .byte $bd, $82, $89, $90, $96, $9c, $a1, $a7, $ac, $b0, $b5, $b9
        .byte $bd, $82, $89, $90, $96, $9c, $a1, $a7, $ac, $b0, $b5, $b9
        .byte $bd, $82, $89, $90, $96, $9c, $a1, $a7, $ac, $b0, $b5, $b9
        .byte $bd, $c1, $c4, $c7, $ca, $cd, $d0, $d3, $d5, $d8, $da, $dc
        .byte $de, $e0, $e2, $e3, $e5, $e6, $e8, $e9, $ea, $eb, $ec, $ed
        .byte $ee, $ef, $f0, $f1, $f2, $f3, $f3, $f4, $f5, $f5, $f6, $f6
        .byte $f7, $f7, $f8, $f8, $f8, $f9, $f9, $f9, $fa, $fa, $fa, $fb
// VIC-I-Registerwerte ($80 | N), Alt ($900B)
note_vic_alto:
        .byte $00
        .byte $bd, $82, $89, $90, $96, $9c, $a1, $a7, $ac, $b0, $b5, $b9
        .byte $bd, $82, $89, $90, $96, $9c, $a1, $a7, $ac, $b0, $b5, $b9
        .byte $bd, $82, $89, $90, $96, $9c, $a1, $a7, $ac, $b0, $b5, $b9
        .byte $bd, $c1, $c4, $c7, $ca, $cd, $d0, $d3, $d5, $d8, $da, $dc
        .byte $de, $e0, $e2, $e3, $e5, $e6, $e8, $e9, $ea, $eb, $ec, $ed
        .byte $ee, $ef, $f0, $f1, $f2, $f3, $f3, $f4, $f5, $f5, $f6, $f6
        .byte $f7, $f7, $f8, $f8, $f8, $f9, $f9, $f9, $fa, $fa, $fa, $fb
        .byte $fb, $fb, $fb, $fc, $fc, $fc, $fc, $fc, $fc, $fd, $fd, $fd
// VIC-I-Registerwerte ($80 | N), Bass ($900A)
note_vic_bass:
        .byte $00
        .byte $bd, $82, $89, $90, $96, $9c, $a1, $a7, $ac, $b0, $b5, $b9
        .byte $bd, $82, $89, $90, $96, $9c, $a1, $a7, $ac, $b0, $b5, $b9
        .byte $bd, $c1, $c4, $c7, $ca, $cd, $d0, $d3, $d5, $d8, $da, $dc
        .byte $de, $e0, $e2, $e3, $e5, $e6, $e8, $e9, $ea, $eb, $ec, $ed
        .byte $ee, $ef, $f0, $f1, $f2, $f3, $f3, $f4, $f5, $f5, $f6, $f6
        .byte $f7, $f7, $f8, $f8, $f8, $f9, $f9, $f9, $fa, $fa, $fa, $fb
        .byte $fb, $fb, $fb, $fc, $fc, $fc, $fc, $fc, $fc, $fd, $fd, $fd
        .byte $fd, $fd, $fd, $fd, $fd, $fd, $fe, $fe, $fe, $fe, $fe, $fe
.endif
.endif

.endif
