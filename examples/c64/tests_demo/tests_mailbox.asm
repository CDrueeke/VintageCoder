; tests_mailbox.asm – Postfach-Modus von `vcc test`: das Programm prüft sich selbst
;
; Zweck: zeigt die Testbibliothek (Testkonzept Stufe 3). Das Testprogramm ruft zwei kleine Unterprogramme auf, vergleicht die Ergebnisse
;   mit Sollwerten (assert_*-Makros) und trägt das Ergebnis in ein Postfach im RAM ein. Der Läufer startet das Programm, wartet auf die
;   Endlosschleife von test_end und liest nur den Speicher: im Emulator (`vcc test --backend vice`) wie auf einer Ultimate 64
;   (`--backend u64:HOST`). Die Testdatei dazu ist tests_mailbox.vctest (mode "mailbox", Größe, Überlappung und ein Lauf).
;
; Bedienung: `vcc build examples/c64/tests_demo`, dann `vcc test examples/c64/tests_demo --backend vice`. Ohne Emulator meldet der
;   Läufer den Test als „übersprungen“, die statischen Prüfungen laufen trotzdem.
;
; Ablauf: test_begin (Postfach löschen, Status „läuft“) – 21 durch `double` verdoppeln und gegen 42 prüfen – X = 5 durch `plus3`
;   um drei erhöhen und gegen 8 prüfen – prüfen, dass A dabei erhalten blieb – Speicherzelle $FB mit $7F beschreiben und lesen –
;   test_end (Status „bestanden“ oder „fehlgeschlagen“ ins Postfach, danach Endlosschleife). Vier Prüfungen.
;
; Postfach: TEST_MAILBOX (C64: $033C, Kassettenpuffer, 8 Bytes; Wert aus data/targets/c64.target.json), Aufbau und Statuswerte siehe
;   data/sys/common/test.inc. Die Makros lassen A, X und Y unverändert (die Flags nicht). Ist TEST_DEBUGREG gesetzt (C64: $D7FF),
;   meldet test_end den Status zusätzlich dorthin (die Ultimate 64 liest es über REST).
; Speicherplan: $0801 BASIC-Zeile `10 SYS 2061`, Programm ab $080D, Postfach $033C-$0343, Zeropage: $FB (Testzelle). Größe der Segmente 240 Bytes
;   (Prüfung in tests_mailbox.vctest: höchstens $0100).
.target c64
.cpu 6510 { illegals = deny }
.include "c64/test.inc"             ; Makros test_begin, assert_a_eq, assert_x_eq, assert_y_eq, assert_mem_eq, test_end

.org $0801
.basicstub start

start:  sei
        test_begin                  ; Postfach löschen, Status = läuft (verändert A)
        lda #21
        jsr double
        assert_a_eq 42              ; 21 * 2
        ldx #5
        jsr plus3
        assert_x_eq 8               ; 5 + 3
        assert_a_eq 42              ; A blieb erhalten
        lda #$7f
        sta $fb
        assert_mem_eq $fb, $7f
        test_end                    ; Ergebnis ins Postfach, danach Endlosschleife

; double – verdoppelt A (ASL). Eingabe/Ausgabe: A. Verändert die Flags N, Z, C.
double: asl
        rts

; plus3 – erhöht X um 3. Eingabe/Ausgabe: X. Verändert die Flags N und Z.
plus3:  inx
        inx
        inx
        rts
