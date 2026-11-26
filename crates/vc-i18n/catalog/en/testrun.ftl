# SPDX-License-Identifier: GPL-3.0-or-later
# Test runner (English)

# ---- Trace (trace.rs)
testrun-trace-analyzer-crashed = Analysis crashed
testrun-trace-no-measure = no measurement
testrun-trace-header = # Cycle  PC  Cycles  Source (cycles count per epoch; a new epoch starts after packet loss)
testrun-trace-epoch = # --- packet loss: new epoch { $epoch } ---
testrun-trace-writer-crashed = Writer thread crashed

# ---- Run and display (suite.rs)
testrun-static-title = Static checks
testrun-suffix-mailbox = · mailbox
testrun-suffix-measure = · cycles
testrun-hw-ultimate-c64-only = The Ultimate hardware only plays C64 programs
testrun-hw-ultimate-prg-only = Only .prg files can be started on the Ultimate
testrun-hw-mega65-only = The MEGA65 hardware only plays MEGA65 programs
testrun-launch-vice-target = VICE does not know the target { $target }
testrun-launch-atari800-target = atari800 only knows the target atari8, not { $target }
testrun-launch-fujisan-target = Fujisan only knows the target atari8, not { $target }
testrun-launch-xemu = Xemu loads programs through the monitor: mailbox mode is not set up for it yet
testrun-launch-unknown-emulator = Emulator "{ $x }" is unknown
testrun-mark-note = Note{ $m }: { $name }
testrun-mark-error = Error: { $name }{ $m }
testrun-mark-timeout = Timeout: { $name }
testrun-mark-skipped = skipped: { $name }
testrun-skip-not-built = the program is not built or not readable (build, then run again)
testrun-skip-trap-on-hardware = Trap mode not available: the hardware provides no CPU registers and no breakpoints. Rebuild as a mailbox test ("mode": "mailbox", test.inc)
testrun-skip-measure-needs-u64 = Cycle measurement (mode measure) needs the Ultimate 64: set "Runs on" to the device or use vcc test --backend u64:HOST
testrun-skip-no-emulator = no emulator chosen: "Runs on" in the header bar
testrun-skip-emulator-unsupported = Emulator "{ $b }" is not supported by the runner yet
testrun-debug-mailbox = In mailbox mode there is no trap: you can debug the program normally (F6)
testrun-debug-trap-target = Debugging in trap mode does not exist for the target { $target } yet

# ---- Trap mode (runner.rs)
testrun-mem-unreadable = memory not readable
testrun-mem-unreadable-here = memory is not readable here
testrun-quoted-error = "{ $src }": { $e }
testrun-byte-range = { $what }: { $v } does not fit in a byte
testrun-image-unreadable = Program image not readable: { $e }
testrun-address-range = { $what }: address { $v } outside 0..65535
testrun-trap-unknown = no trap address is known for the target "{ $target }": give "trap" in the test
testrun-trap-in-program = the trap at { $trap } is inside the program: give a free address as "trap" in the test
testrun-emulator-start-failed = Emulator cannot be started: { $e }
testrun-cancelled = cancelled
testrun-skip-iflag = the test requires a cleared I flag: in trap mode, tests run with the I flag set (otherwise VICE does not reliably reach the trap)
testrun-hang = { $what } not reached after { $ms } ms: the subroutine does not return; the emulator is restarted
testrun-hang-what-prologue = the trap prologue
testrun-hang-what-return = the return
testrun-cycles-not-requested = not requested
testrun-cycles-no-support = this emulator does not measure cycles
testrun-cycles-unmeasurable = not measurable: { $e }
testrun-cycles-beam-near-frame = not measurable: beam time { $beam } is close to one frame ({ $frame } cycles), the position repeats
testrun-cycles-less-than-call = not measurable: { $n } cycles are less than the call instruction
testrun-cycles-mismatch = not measurable: timer ({ $timer } cycles) and beam position ({ $beam } mod { $frame }) do not match
testrun-cycles-timer-unreadable = not measurable: timer not readable
testrun-expect-parse = expect "{ $src }": { $e }
testrun-expect-failed = Expectation "{ $src }" is not met
testrun-expect-compare = : left side = { $lv } ({ $lh }), right side = { $rv } ({ $rh }), required { $op }
testrun-expect-unevaluable = Expectation "{ $src }" cannot be evaluated: { $e }
testrun-expect-cycles-why = "{ $src }": cycles { $why }
testrun-state-on-return = State on return: { $regs }
testrun-budget-cycles-why = Cycle budget: cycles { $why }
testrun-budget-exceeded = { $n } cycles measured{ $sfx }, allowed: at most { $limit } - { $over } too many
testrun-budget-beam-note = (beam time: includes memory refresh and DMA, never below the CPU cycles)
testrun-measured-cycles = { $n } cycles
testrun-measured-cycles-beam = { $n } cycles (beam time)
testrun-test-not-found = Test "{ $test }" not found

# ---- Mailbox mode (mailbox.rs)
testrun-kind-mem = assert_mem_eq (memory)
testrun-kind-unknown = unknown kind
testrun-mailbox-none-for-target = no mailbox (testMailbox) is set in the target data for the target "{ $target }"
testrun-mailbox-unreadable = Mailbox not readable: { $e }
testrun-mailbox-state = status { $status }, { $count } check(s) so far
testrun-mailbox-state-unknown = unknown
testrun-mailbox-hint-status0 = (status 0: test_begin was never executed, the program does not start or hangs before)
testrun-mailbox-timeout = the test program did not end after { $ms } ms: { $st }{ $hint }; the emulator is stopped
testrun-mailbox-incomplete = Mailbox read incompletely
testrun-mailbox-fail = Check no. { $number } of { $of } failed ({ $kind }): actual value { $actual } ({ $actualhex }), expected { $expected } ({ $expectedhex })
testrun-mailbox-no-checks = the program ran through but did not execute any check (assert_… missing)
testrun-mailbox-unfinished = mailbox shows status { $status } after the end: the program overwrote the mailbox or did not reach test_end
testrun-mailbox-expect-unmet = Expectation "{ $src }" is not met after the end
testrun-mailbox-expect-after = all { $checks } checks of the program passed, but { $b }
testrun-mailbox-checks =
    { $n ->
        [one] { $n } check
       *[other] { $n } checks
    }

# ---- Cycle measurement on hardware (cycles.rs)
testrun-measure-mem-unreadable = memory is not readable in a hardware measurement
testrun-measure-addr-range = "{ $src }" at { $v } is outside $0000..$FFFF
testrun-measure-text-range = { $a } to { $b } cycles ({ $n } measurements)
testrun-measure-text-one = { $a } cycles
testrun-measure-span = { $from } to { $to }
testrun-measure-lost = , { $n } packet(s) lost
testrun-measure-invalid-no-packets = invalid: no packets of the debug stream received (firewall for incoming UDP, mode "6510 Only" on the device, network?); nothing estimated
testrun-measure-invalid-loss = invalid: packet loss during measurement { $span } ({ $n } measurement(s) discarded{ $lost }); nothing estimated, repeat the run
testrun-measure-not-reached-after = { $span } was not passed in { $ms } ms: { $from } reached, { $to } not reached afterwards
testrun-measure-never-reached = { $span } was not passed in { $ms } ms: { $from } never reached
testrun-measure-over = { $hi } cycles of { $span } exceed the budget of { $mx }
testrun-measure-under = { $lo } cycles of { $span } are below the lower limit of { $mn }
testrun-measure-extra-invalid = ({ $n } more measurement(s) invalid because of packet loss and discarded)
testrun-measure-addr-unevaluable = Address cannot be evaluated: { $e }
testrun-measure-limit-unevaluable = Limit cannot be evaluated: { $e }
testrun-measure-impossible = Measurement not possible: { $e }
testrun-measure-u64-only = The debug stream (cycle measurement) is only available on the Ultimate 64

testrun-hw-quoted-name = "{ $name }"
