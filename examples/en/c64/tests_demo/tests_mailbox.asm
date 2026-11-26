; tests_mailbox.asm - mailbox mode of `vcc test`: the program tests itself
;
; Purpose: shows the test library (test concept stage 3). The test program calls two small subroutines, compares the results
;   with expected values (assert_* macros) and writes the result into a mailbox in RAM. The runner starts the program, waits for the
;   endless loop of test_end and only reads memory: in the emulator (`vcc test --backend vice`) as on an Ultimate 64
;   (`--backend u64:HOST`). The test file for it is tests_mailbox.vctest (mode "mailbox", size, overlap and one run).
;
; Usage: `vcc build examples/c64/tests_demo`, then `vcc test examples/c64/tests_demo --backend vice`. Without an emulator the
;   runner reports the test as "skipped", the static checks run anyway.
;
; Flow: test_begin (clear the mailbox, status "running") - double 21 with `double` and check against 42 - raise X = 5 by three with
;   `plus3` and check against 8 - check that A was preserved - write $7F to memory cell $FB and read it back -
;   test_end (status "passed" or "failed" into the mailbox, then endless loop). Four checks.
;
; Mailbox: TEST_MAILBOX (C64: $033C, cassette buffer, 8 bytes; value from data/targets/c64.target.json), layout and status values see
;   data/sys/common/test.inc. The macros leave A, X and Y unchanged (not the flags). If TEST_DEBUGREG is set (C64: $D7FF),
;   test_end reports the status there as well (the Ultimate 64 reads it via REST).
; Memory map: $0801 BASIC line `10 SYS 2061`, program from $080D, mailbox $033C-$0343, zero page: $FB (test cell). Size of the segments 240 bytes
;   (check in tests_mailbox.vctest: at most $0100).
.target c64
.cpu 6510 { illegals = deny }
.include "c64/test.inc"             ; macros test_begin, assert_a_eq, assert_x_eq, assert_y_eq, assert_mem_eq, test_end

.org $0801
.basicstub start

start:  sei
        test_begin                  ; clear the mailbox, status = running (changes A)
        lda #21
        jsr double
        assert_a_eq 42              ; 21 * 2
        ldx #5
        jsr plus3
        assert_x_eq 8               ; 5 + 3
        assert_a_eq 42              ; A was preserved
        lda #$7f
        sta $fb
        assert_mem_eq $fb, $7f
        test_end                    ; result into the mailbox, then endless loop

; double - doubles A (ASL). Input/output: A. Changes the flags N, Z, C.
double: asl
        rts

; plus3 - raises X by 3. Input/output: X. Changes the flags N and Z.
plus3:  inx
        inx
        inx
        rts
