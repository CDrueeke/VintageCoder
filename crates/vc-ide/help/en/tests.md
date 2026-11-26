# Tests with vcc test

`vcc test` checks a built program using a test file `*.vctest`. There are two levels: **static checks**, which run without an emulator, and **tests in the emulator** (trap mode), which call single subroutines. Both are in the same file. In the user interface, tests run in the **Tests** tab of the tray below the editor (see below). Without the user interface, use the command line.

## The "Tests" tab in the user interface
The tray (**Cmd/Ctrl+J**) has the **Tests** tab. **Alt+F8** (menu Project > Run tests) opens it and runs all tests. The bar at the top of the tab has the button **Run all** (during a run: **Cancel**), the button **Failed**, the selection **Target** (between **Static checks only** and the emulators that were found), the field **Filter tests …** and the checkbox **errors only**; on the right is the summary, for example "3 of 5 passed". The list on the left is sorted by test file, and the right side shows the details of the selected test with the expectation, the measured value, registers and cycles. **Go to source** jumps to the source code, **Open test file** opens the test file, and **Debug this test** starts a debug session with the trap of the test, stopped at the start of the call (**Debug** section, note "Test session"; build first, see [Debugger](help:debugger)). Hardware, mailbox polling and measuring on the Ultimate do not exist in the user interface yet, only on the command line.

![The code editor with the file tests_demo.asm. Below the editor the drawer is open on the tab 'Tests': buttons 'Alle ausfuehren' and 'Fehlgeschlagene', the choice 'Ziel' set to 'VICE x64sc', a filter field, the checkbox 'nur Fehler' and the text '8 von 17 bestanden'. Below that are the static checks with green dots and the word 'bestanden'.](bild:tests-panel "The 'Tests' tab in the drawer with the list of checks and their results.")

## Command line
    vcc test [folder|file.vctest] [--strict] [--backend vice] [--filter TEXT] [--prologue] [--data-dir folder]

Without a path, the current folder is searched (all `*.vctest`, without `build`, `target` and hidden folders). **Build first** (`vcc build`), because the tests read the built program and the `.vcdbg` next to it. The example `examples/c64/tests_demo` shows both levels.

## The test file
    {
      "kind": "test", "target": "c64", "program": "build/tests_demo.prg",
      "static": [ { "check": "size", "max": 1000 }, { "check": "overlap" } ],
      "tests":  [ { "name": "...", "call": "next_color", "expect": [ "[$fb] == 0" ] } ]
    }

`program` is the program file, relative to the test file or to a parent folder.

## Static checks (`static`)
| `check` | Checks |
|---|---|
| `size` | Size in bytes of one segment (`segment`) or of all segments: `min`, `max`, and with `maxEnd` the address after the last byte |
| `zeropage` | `forbid` lists address ranges (`$02-$1f`) that no segment may occupy and no instruction may access |
| `overlap` | Segments that overlap |
| `cycles` | The cycle range from a mark (`label`) or from a cycle block (`block`, in the source `.cycles name` ... `.endcycles`) against `max`. `level` is `hint` (default) or `error` |
| `expr` | Any expression, for example `size("") < $2000 - $0801`, with an optional `name` |

Numbers and expressions can be written as a number or as text (`"$3f"`). The language is the one of conditional breakpoints (registers, flags, `[address]`, symbols of the `.vcdbg`). The cycle range follows all paths of a subroutine up to `rts` or `rti`. For loops, `jsr`, `jmp (...)` and unknown instructions, it reports "cannot be determined" and does not guess a number. CPU cycles are counted. Deductions caused by DMA (bad lines, sprites), interrupts and wait states are not included. Page crossings count in the worst case.

## Tests in the emulator (`tests`, `--backend vice`)
Without `--backend`, these tests appear as "skipped". With `--backend vice`, the runner starts VICE (only one emulator at a time), writes the program image and `setup.mem` to memory for each test, sets the registers, calls the subroutine through a trap in free RAM and evaluates `expect`.

    { "name": "next_color: 15 becomes 0",
      "setup":  { "mem": { "$fb": 15 }, "a": "8", "x": 0 },
      "call":   "next_color",
      "return": "rts",
      "expect": [ "[$fb] == 0", "!fc" ],
      "cycles": { "max": 20 },
      "timeoutMs": 2000 }

- `call`: label or address. `return`: `rts` (default) or `rti` for interrupt routines.
- `setup`: `mem` (address to byte or list of bytes), the registers `a x y sp p` and the flags `fn fv fd fi fz fc` (0 or 1). Default: `sp=$FF`, all flags cleared, I set.
- `expect`: expressions as above, plus `cycles` for the measured duration of the subroutine without the call instruction, including `rts` or `rti`. A test without `expect` and without `cycles` is an error ("checks nothing").
- `cycles.max` is a budget. `level` is `hint` (default) or `error`. `--strict` turns every hint into an error.
- File level: `trap` (address of the trap, default on the C64 `$C000`) and `isolate: true` (VICE restarts for every test).

Between the tests, the program and data are written again, but other memory locations (zero page, screen, chips) are not. A test must set everything it needs in `setup`. The main program does not run. What it sets at startup belongs in `setup`. The cycle measurement uses the timers of CIA2 and writes to its registers while doing so.

## Result
| State | Meaning |
|---|---|
| OK | All expectations are met, the measured value is shown with it |
| FAIL | Expectation violated (with left and right side) or `level: error` exceeded |
| HINT | Cycle budget exceeded without counting as an error |
| Timeout | The trap was not reached after `timeoutMs` |
| skipped | The test cannot run here. The reason is shown with it (for example cleared I flag required) |

The exit code is 1 for failures or timeouts. Hints and skipped tests do not change it.

## Mailbox mode: the program checks itself
With `"mode": "mailbox"` in the test file (instead of the calls in trap mode), the test program checks itself. It includes the test library of the target and writes the result to a mailbox in RAM. The runner starts the program, waits for the end and only reads memory.

```
.include "c64/test.inc"        ; or c128/test.inc, atari8bit/test.inc
        test_begin             ; first (changes A)
        jsr mein_unterprogramm
        assert_a_eq 42         ; check A, X, Y and memory; the registers are preserved
        assert_x_eq 8
        assert_y_eq 0
        assert_mem_eq $fb, $7f
        test_end               ; write the result, then an endless loop
```

```json
{ "kind": "test", "target": "c64", "program": "build/tests_mailbox.prg", "mode": "mailbox",
  "tests": [ { "name": "Self-test", "timeoutMs": 20000, "expect": [ "[$fb] == $7f" ] } ] }
```

- Each entry in `tests` is one run: a freshly started emulator with the program (default time limit 15 s, including startup). `expect` are additional expectations about memory and registers after the end.
- The mailbox (8 bytes) is at `TEST_MAILBOX`. The address is in the target data (`testMailbox`): C64 `$033C` (cassette buffer), C128 `$0B00` (cassette buffer), Atari 8-bit `$0600` (page 6). Plus/4, VIC-20 and MEGA65 have none. The test is skipped there. The program must not overwrite the mailbox.
- Result: passed (with the number of checks), failed (number of the first failed check, type, actual value and expected value), timeout (status and checks so far are shown with it) or failed "no check was run". Cycles are not measured in this mode.
- Waiting: If the program contains the endless loop of `test_end`, the runner stops there (also with atari800, which cannot be paused). Otherwise, or with `vcc test --poll`, it reads the mailbox every 100 ms.
- Tested with VICE (C64) and atari800 (Atari 8-bit). C128 and Xemu are not set up yet (Xemu loads programs through the monitor). Opening a mailbox test in the debugger does not exist (there is no trap, and you can debug the program normally).

## Real hardware, JUnit
Mailbox tests also run on an **Ultimate 64 / Ultimate II+** (REST, C64 programs only): `vcc test --backend u64:HOST[:PORT] [--password PW] --yes`. The device loads the program (reset, load, RUN), and the runner reads the mailbox while the computer is running, at intervals of 100 ms until the time limit. After that, the device keeps running. Trap tests are skipped there, because the hardware provides neither CPU registers nor breakpoints. The network password comes from `--password`, otherwise from the machine with the same address in the settings. Without an address (`--backend u64`), nothing is contacted.

Every run on hardware needs a confirmation: `vcc test` asks in the terminal (`--yes` skips the question, without a terminal `--yes` is required). Runs on hardware do not exist in the user interface yet. MEGA65 hardware (`--backend m65:INTERFACE[@BAUD]`) is prepared but not tested (the device was offline), and has no mailbox in the target data yet.

`vcc test --junit file.xml` also writes JUnit XML for pipelines: failed -> `failure`, timeout -> `error`, skipped -> `skipped`. Hints and measured values are written as `system-out`.

## Cycle measurement on the Ultimate 64 (`"mode": "measure"`)
With the debug stream of the Ultimate 64 (mode "6510 Only"), `vcc test` measures the **cycles from address A to address B** on real hardware and checks them against a budget. For this, the device must have "6510 Only" set under *Data Streams > Debug Stream Mode* (the IDE does not change this).

```json
{ "kind": "test", "target": "c64", "program": "build/tests_cycles.prg", "mode": "measure",
  "tests": [ { "name": "Delay loop", "from": "messen", "to": "ende", "max": 1284, "timeoutMs": 10000 } ] }
```

    vcc test examples/c64/tests_cycles --backend u64:192.168.0.10 --yes

- `from`, `to`: label or address (expression as elsewhere). `max`: budget (maximum), `min`: lower limit. At least one is required. `level`: `error` (default, a verdict) or `hint`. `timeoutMs`: time limit after the program starts (default 20,000).
- Steps for each test: start the debug stream (the video stream is stopped while doing so, both at the same time do not work), reset the device, load and start the program, count cycles from the reset, stop the stream. No configuration values are changed and no flash memory is written.
- The measurement runs from the first **read access** (fetch or read) at `from` to the first one after it at `to`, accurate to the cycle. All wait cycles (bad lines, sprite DMA) **and every interrupt** that runs in between are included. A program with a raster IRQ therefore measures differently depending on the timing. With several runs, "a to b cycles" is shown and the budget applies to the largest value. For exact measurements: disable the IRQ (`sei`), switch off the screen.
- A taken branch already reads the address behind the instruction (PC+2) as a dummy read. If `from` or `to` is exactly there, this counts as an access and the measurement starts or ends too early. Put marks on a jump target or a call instead.
- Result: **OK** with measured cycles, **FAIL** (or **HINT** with `level: hint`) with "N cycles ... exceed the budget of M", **Timeout** if `to` was not reached after `from`. **Invalid** (reported as "skipped", an error with `--strict`) means: packets of the debug stream were lost or did not arrive at all (firewall for incoming UDP). The measurement is then **never estimated**, and you can repeat the run.
- Only on the Ultimate 64 (MEGA65 and Ultimate II+ have no debug stream), only 6502/6510. The safety question before the hardware run stays. The measurement does not exist in the user interface yet.

## Limits
Tests in the emulator run only with `--backend vice`. The default trap address and the cycle measurement exist so far only for the C64 (x64sc, PAL). On the other VICE systems, `trap` must be in the test file, and tests with `cycles` are skipped. atari800, Xemu and hardware are not supported (`--backend` accepts only `vice`).

All `vcc` commands at a glance: [Command line](help:cli).
