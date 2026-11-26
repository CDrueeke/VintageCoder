; tests_cycles.asm - cycle measurement on the Ultimate 64 (vcc test --backend u64:HOST): debug stream, mode "6510 Only"
;
; Purpose: device under test for the measurement `"mode": "measure"` of `vcc test`. What is measured is the CPU cycles from the first fetch of the instruction
;   at the label `messen` to the first one at the label `ende`, directly on the hardware (Ultimate 64, debug stream, mode "6510 Only"; set on the device
;   under Data Streams -> Debug Stream Mode). The test file tests_cycles.vctest sets the budget.
;
; Usage: `vcc build examples/c64/tests_cycles`, then `vcc test examples/c64/tests_cycles --backend u64:HOST --yes` (HOST = address
;   of the Ultimate). Without a device, `vcc test` only reports the static size check (program at most $0100 bytes). The program also starts
;   in the emulator and ends in an endless loop (RESET only).
;
; Sequence: the screen is switched off (DEN = 0, no bad lines) and the IRQ is disabled so that the measurement is exact. Then the
;   program waits for the end of the frame (raster line $FA) - DEN takes effect only in the next frame, and the current one still has bad lines until then. Then
;   the measured wait loop (`dex`/`bne`, 256 passes) and the endless loop at the label `ende` run.
; Measured section messen..ende:
;   LDX (2) + 255 * (DEX 2 + BNE taken 3) + (DEX 2 + BNE not taken 2) + JMP (3) = 1284 cycles.
;   The test file checks budget 1284 (must pass) and budget 1283 with level "hint" (must fail narrowly and so shows that the
;   measurement is accurate to the cycle). 1284 cycles are about 20 PAL raster lines (63 cycles each); they stay in the same frame (from line $FA).
; Neither `messen` nor `ende` directly follows a BNE: a taken branch makes a dummy read of the address behind it
;   (PC+2); that would be an "access" to the label and would start or end the measurement too early.
;
; Hardware: VIC-II $D011 (control register 1) and $D012 (raster line). Memory map: $0801 BASIC line `10 SYS 2061`, program from $080D
;   (39 bytes with load address 41). Zero page: unused.
.target c64
.cpu 6510 { illegals = deny }

.org $0801
.basicstub start

start:  sei                     ; no interrupt may run into the measurement
        lda #$0b                ; DEN = 0: screen off, no bad lines from the next frame (value $0B: RSEL = 1, YSCROLL = 3, bit 4 = 0)
        sta $d011
bild:   lda $d012               ; wait for the end of the frame (line $FA): the current frame still has bad lines
        cmp #$fa
        bne bild
        jmp messen              ; not directly behind the BNE (dummy read, see above)
messen: ldx #0                  ; start of measurement: first fetch of this instruction
warte:  dex                     ; X runs from 0 through 255 back to 0: 256 passes
        bne warte
        jmp ende                ; 3 cycles, the target is the next instruction
ende:   jmp ende                ; endless loop: the first access to `ende` ends the measurement
