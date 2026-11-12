// SPDX-License-Identifier: GPL-3.0-or-later
// Testbibliothek (Postfach-Modus, Testkonzept Stufe 3): das Testprogramm prüft sich selbst und schreibt das Ergebnis in ein
// Postfach im RAM. Der Läufer (vcc test, Reiter „Tests“) startet das Programm und liest nur Speicher: Emulator wie Hardware.
//
// Einbinden: #include <c64/test.inc>  (oder c128/test.inc, atari8bit/test.inc; beides bindet dies hier ein)
//
//   test_begin                     ; zuerst: Postfach löschen, Status „läuft“ (verändert A)
//   jsr mein_unterprogramm
//   assert_a_eq  $10               ; A == $10 ?
//   assert_x_eq  0                 ; X == 0 ?
//   assert_y_eq  1
//   assert_mem_eq $0200, 125       ; Byte an Adresse == Wert ?
//   test_end                       ; zuletzt: Ergebnis eintragen, Endlosschleife (jmp *)
//
// Jede Prüfung zählt mit; die erste fehlgeschlagene merkt sich Nummer, Ist-Wert und Sollwert, spätere ändern das nicht.
// Register A, X und Y bleiben bei den Prüfungen erhalten (die Flags nicht). Jede Prüfung kostet rund 10 bis 40 Zyklen.
// Postfach: Adresse TEST_MAILBOX kommt aus den Zieldaten (Feld testMailbox in data/targets/<ziel>.target.json),
// 8 Byte: +0 Status (1 läuft, 2 bestanden, 3 fehlgeschlagen), +1 Anzahl der Prüfungen, +2 Nummer der ersten fehlgeschlagenen
// (0 = keine), +3 Ist-Wert, +4 Sollwert, +5 Art (1 A, 2 X, 3 Y, 4 Speicher), +6/+7 Zwischenspeicher für A und X.
// Debug-Register: Ist TEST_DEBUGREG gesetzt (Zieldaten, Feld testDebugReg: nur C64, $D7FF), schreiben test_begin „läuft“ (1) und test_end
// den Status (2 bestanden, 3 fehlgeschlagen) auch dorthin. Eine Ultimate 64 meldet das über REST (GET /v1/machine:debugreg); der Läufer muss
// dann das Postfach nicht ständig lesen. Auf Emulatoren und anderer Hardware ohne Wirkung ($D7FF ist im C64 unbelegt).
// Der Test kann das Postfach selbst überschreiben, wenn er Speicher verletzt; das Programm gehört dann in den Bericht.

.ifndef TEST_MAILBOX
.error "TEST_MAILBOX fehlt: für dieses Ziel ist in den Zieldaten kein testMailbox festgelegt (C64, C128 und Atari 8 Bit haben eines)"
.endif

.const MB_STATUS = TEST_MAILBOX
.const MB_COUNT  = TEST_MAILBOX + 1
.const MB_FIRST  = TEST_MAILBOX + 2
.const MB_ACTUAL = TEST_MAILBOX + 3
.const MB_EXPECT = TEST_MAILBOX + 4
.const MB_KIND   = TEST_MAILBOX + 5
.const MB_SAVA   = TEST_MAILBOX + 6
.const MB_SAVX   = TEST_MAILBOX + 7

.const TEST_RUNNING = 1
.const TEST_PASS    = 2
.const TEST_FAIL    = 3

.macro test_begin
        lda #0
        sta MB_COUNT
        sta MB_FIRST
        sta MB_ACTUAL
        sta MB_EXPECT
        sta MB_KIND
        sta MB_SAVA
        sta MB_SAVX
        lda #TEST_RUNNING
        sta MB_STATUS
.ifdef TEST_DEBUGREG
        sta TEST_DEBUGREG
.endif
.endmacro

.macro assert_a_eq(v)
        inc MB_COUNT
        cmp #v
        beq @ok
        stx MB_SAVX
        ldx MB_FIRST
        bne @skip
        ldx MB_COUNT
        stx MB_FIRST
        sta MB_ACTUAL
        ldx #v
        stx MB_EXPECT
        ldx #1
        stx MB_KIND
@skip:  ldx MB_SAVX
@ok:
.endmacro

.macro assert_x_eq(v)
        inc MB_COUNT
        cpx #v
        beq @ok
        sta MB_SAVA
        lda MB_FIRST
        bne @skip
        lda MB_COUNT
        sta MB_FIRST
        stx MB_ACTUAL
        lda #v
        sta MB_EXPECT
        lda #2
        sta MB_KIND
@skip:  lda MB_SAVA
@ok:
.endmacro

.macro assert_y_eq(v)
        inc MB_COUNT
        cpy #v
        beq @ok
        sta MB_SAVA
        lda MB_FIRST
        bne @skip
        lda MB_COUNT
        sta MB_FIRST
        sty MB_ACTUAL
        lda #v
        sta MB_EXPECT
        lda #3
        sta MB_KIND
@skip:  lda MB_SAVA
@ok:
.endmacro

.macro assert_mem_eq(addr, v)
        inc MB_COUNT
        sta MB_SAVA
        lda addr
        cmp #v
        beq @ok
        stx MB_SAVX
        ldx MB_FIRST
        bne @skip
        ldx MB_COUNT
        stx MB_FIRST
        sta MB_ACTUAL
        ldx #v
        stx MB_EXPECT
        ldx #4
        stx MB_KIND
@skip:  ldx MB_SAVX
@ok:    lda MB_SAVA
.endmacro

.macro test_end
        lda MB_FIRST
        bne @fail
        lda #TEST_PASS
        bne @set
@fail:  lda #TEST_FAIL
@set:   sta MB_STATUS
.ifdef TEST_DEBUGREG
        sta TEST_DEBUGREG
.endif
@end:   jmp @end
.endmacro
