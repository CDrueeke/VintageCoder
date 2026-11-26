---
id: tests
titel: Automatic testing
reihenfolge: 10
stufe: Advanced
zeit: 30 minutes
siehe_auch: [debugger, bauen-starten, systeme]
referenz: [tests, debugger]
i18n_status: maschinell
i18n_quelle: 4852ebaa2c9e
---

# Automatic testing

**Goal:** You write a small test file that checks your program automatically: Is it small enough? Does it access forbidden memory locations? Does a subroutine give the right result, and does it need too many cycles? You do not have to start and check by hand every time.

## What you need

- A project that builds without errors. We use the practice program `farbwechsel` from the chapter [The Debugger](kapitel:debugger). You can also open the finished example `examples/c64/tests_demo`.
- For the **static checks**, you do not need an emulator.
- For **tests in the emulator**, you need VICE (the tests currently run only with VICE; all other emulators are not supported).

## Two kinds of checks

A test file has the extension **`.vctest`** and can contain both kinds:

- **Static checks** (`static`) look at the built program without running it: size, memory overlaps, zero-page use, cycles.
- **Tests in the emulator** (`tests`) call single subroutines with given start values and check the result. For this, VintageCoder starts VICE, writes your program into memory and calls the subroutine through a "trap" (a piece of code in free RAM).

Important: The tests check the **built** program. So build first (F7). The IDE reads `build/farbwechsel.prg` and the `.vcdbg` file next to it (it contains the names of your labels).

## Create a test file

1. In the sidebar (heading **Project**), click the icon **New file**. In the dialog **New file**, enter `farbwechsel.vctest` as the name and set the entry **No extension** under **Type**. Because the name already has an extension, VintageCoder should not add a second one. Click **Create**. There is no template for test files in the user interface yet.
2. The new file opens in the editor. Enter this text. It checks the practice program from the Debugger chapter:

```json
{
  "kind": "test", "target": "c64", "program": "build/farbwechsel.prg",
  "static": [
    { "check": "size", "max": 200 },
    { "check": "overlap" },
    { "check": "cycles", "label": "naechste_farbe", "max": 20 }
  ],
  "tests": [
    { "name": "naechste_farbe: 3 becomes 4",
      "setup": { "mem": { "$fb": 3 } },
      "call": "naechste_farbe",
      "expect": [ "[$fb] == 4", "a == 4", "([$d020] & 15) == 4" ] },
    { "name": "naechste_farbe: after 15 comes color 0 again",
      "setup": { "mem": { "$fb": 15 } },
      "call": "naechste_farbe",
      "expect": [ "[$fb] == 16", "a == 0", "fz" ],
      "cycles": { "max": 20 } }
  ]
}
```

What it says:

| Field | Meaning |
|---|---|
| `kind`, `target` | always `"test"`; the target system of the program |
| `program` | the built program file, relative to the test file |
| `static` | the static checks, each with `check` |
| `tests` | the tests in the emulator |
| `setup` | start values: `mem` (address to byte), the registers `a`, `x`, `y`, `sp`, `p` and the flags |
| `call` | the subroutine, as a label or address (with `"return": "rti"` for interrupt routines) |
| `expect` | expressions that must be **true** after the call |
| `cycles` | cycle budget: how long the call may take at most |

In the expressions are the registers (`a x y`), the flags (`fz fc` ...), the memory (`[address]`) and the names from your program. The flag name alone means "set"; with an exclamation mark in front, it means "not set" (`!fc`).

> **Tip:** Each test must prepare everything it needs by itself. Between the tests, the IDE rewrites only the program and data, but not the zero page, the screen or the chip registers. The main program does not run: what it sets at startup belongs in the `setup`. The defaults are `sp=$FF` and cleared flags (only I set).

> **Caution:** A test without `expect` and without `cycles` is an error ("checks nothing").

## Run tests

The tests run in the **tray** below the editor in the **Tests** tab.

![The code editor with the file tests_demo.asm. Below the editor the drawer is open on the tab 'Tests': buttons 'Alle ausfuehren' and 'Fehlgeschlagene', the choice 'Ziel' set to 'VICE x64sc', a filter field, the checkbox 'nur Fehler' and the text '8 von 17 bestanden'. Below that are the static checks with green dots and the word 'bestanden'.](bild:tests-panel "The 'Tests' tab in the drawer with the list of checks and their results.")

1. Open the tray with **Cmd/Ctrl+J** and click the **Tests** tab. Or press **Alt+F8** (menu **Project > Run tests**): This opens the tab and runs all tests.
2. The bar at the top of the tab has these controls: the button **Run all** (during the run also **Cancel**), the button **Failed** (repeats only the failed tests), the selection **Target**, the field **Filter tests ...** and the check box **errors only**. Next to them on the right is the summary, for example "3 of 5 passed".
3. On the left is the list of tests, sorted by test file. On the right are the details of the selected test.

With **Target** you decide where the tests run. The selection offers:

- **Static checks only**: only the static checks, fast and without an emulator.
- **VICE ...** (or another emulator that was found): the tests in the emulator as well.

> **Info:** Tests on real hardware (Ultimate 64) and the cycle measurement through the Ultimate do not exist in the user interface yet. They run on the command line (see below).

You can do the first check right away: build with **F7**, set **Target** to *Static checks only* and click **Run all**. The three static checks run, and all of them pass:

```
OK  Size all segments <= 200  $20 (32 bytes)
OK  No segment overlap  1 segments
OK  Cycles naechste_farbe <= 20  20  [farbwechsel.asm:18]
```

The tests in the emulator are reported as "skipped", with the reason "needs the emulator". Set **Target** to VICE to run them.

### Read the results

Each test has a state:

| State | Meaning |
|---|---|
| **passed** | all expectations are met; the measured value (for example the cycles) is shown with it |
| **failed** | an expectation is wrong; the message names the left and right side of the comparison |
| **note** | a cycle budget is exceeded, but this does not count as an error (`level: hint`) |
| **timed out** | the trap was not reached in time (`timeoutMs`) |
| **skipped** | the test cannot run here; the reason is shown with it |

Click a test: on the right in the tab, you see the **State**, the message with **expectation and actual value**, the **measured value** ("Measured", for example the cycles) and the state of the registers on return. Three buttons belong to the test: **Go to source** jumps into the source code, **Open test file** opens the test file, and **Debug this test** starts the test in the debugger (see below).

![The Code area with the file tests_demo.asm and the 'Tests' page at the bottom. The list shows checks with green dots ('bestanden', passed), one yellow entry 'fehlgeschlagen' (failed) and two 'uebersprungen' (skipped). At the top it says '8 von 17 bestanden' (8 of 17 passed). On the right are the state, message and measured value of the chosen test and the buttons 'Zur Quelle', 'Testdatei oeffnen' and 'Diesen Test debuggen'.](bild:tests-ergebnis "The 'Tests' page with the list of checks and the result of the chosen test.")

A typical error message looks like this: *Expectation "peekw($0200) == 90000" is not met: left side = 24464 ($5F90), right side = 90000 ($15F90).* This shows you at once that, for example, a carry is missing.

### Examine a test in the debugger

If a test fails and you do not see why, select it and click **Debug this test**. VintageCoder starts a debug session with the "trap" of the test and opens the **Debug** area. There, next to the status, you see the note **Test session**. The program stops right at the start of the call, with the start values from the `setup`. From there you go on step by step and see registers, flags and memory (see [The debugger](kapitel:debugger)). This button does not build again: Build with **F7** first, so that the program matches the test. Debugging in trap mode does not exist for every target system; then VintageCoder reports this.

*[Bild folgt in E13: debug-test-sitzung]*

## Count cycles

How many CPU cycles does a subroutine need? In raster programs, this is the key question. VintageCoder answers it in three ways.

**1. Static** (without an emulator): `{ "check": "cycles", "label": "naechste_farbe", "max": 20 }` calculates the cycles of all paths from the label to `rts` or `rti`. For a subroutine without a branch, an exact value is shown (`20`); with branches, a range (`18 to 19`). For loops, `jsr`, `jmp (...)` and unknown instructions, the check reports "cannot be determined" and does not guess. Only pure CPU cycles are counted; delays from bad lines, sprites, interrupts and wait states are not included.

In the source code, you can also mark a block that the check should measure (example from `examples/c64/tests_demo/tests_demo.asm`):

```asm
irq_handler:
.cycles irq max=24
        pha
        lda COLOR
        sta BORDER
        pla
        rti
.endcycles
```

In the test file, `{ "check": "cycles", "block": "irq" }` checks exactly this block. The result from the real run: `OK  Cycles irq <= 24  20`.

**2. In the emulator**: `"cycles": { "max": 20 }` in a test measures the duration of the call (without the call instruction, but including `rts` or `rti`) and compares it with the budget. The budget `max` is a *note*, unless you give `"level": "error"`.

**3. On real hardware** (only on the command line): With an Ultimate 64, the mode `"mode": "measure"` measures the cycles from address A to address B on the real device, including interrupts and bad lines (see `examples/c64/tests_cycles`). For this, the device must have the value "6510 Only" under *Data Streams > Debug Stream Mode*. The measurement is never estimated: if packets are lost, the IDE reports "skipped".

> **Info:** In the example `tests_demo`, you can see the difference nicely: for `next_color`, the static check reports "18 to 19" cycles, because the code has a branch. The budget `max: 18` therefore gives a **note** ("in the worst case 19 cycles, allowed: at most 18"), and `max: 20` passes.

## More static checks

All checks are in the section `static`:

| `check` | Checks |
|---|---|
| `size` | Size in bytes (`min`, `max`; with `maxEnd` the address after the last byte; with `segment` only one segment) |
| `zeropage` | `forbid`: address ranges such as `"$02-$1f"` that no segment may occupy and no instruction may access |
| `overlap` | Segments that overlap |
| `cycles` | Cycles from a label (`label`) or from a cycle block (`block`) against `max` |
| `expr` | Any expression, for example `size("") < $2000 - $0801`, with an optional `name` |

With `"level": "error"`, a note becomes an error; on the command line, `--strict` turns every note into an error.

## The program checks itself: mailbox mode

> **Info:** Mailbox mode is not fully built in the user interface yet: the hardware query and the runs on an Ultimate exist only on the command line (see below). It is also not yet checked whether and how the user interface runs mailbox tests in the emulator. Rely on the command line for this mode. In mailbox mode there is no trap: You can debug the program like any other with **F6**.

Sometimes it is easier to write the checks *in* the program. For this, there is **mailbox mode**: your test program includes the test library of the target system, runs check commands and writes the result into a small mailbox in RAM. The IDE starts the program, waits for the end and reads only the mailbox. The example `examples/c64/tests_demo/tests_mailbox.asm`:

```asm
.target c64
.cpu 6510 { illegals = deny }
.include "c64/test.inc"

.org $0801
.basicstub start

start:  sei
        test_begin
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
        test_end                    ; result into the mailbox, then an endless loop

double: asl
        rts

plus3:  inx
        inx
        inx
        rts
```

The matching test file has only the name of the program and `"mode": "mailbox"`:

```json
{
  "kind": "test", "target": "c64", "program": "build/tests_mailbox.prg", "mode": "mailbox",
  "static": [
    { "check": "size", "max": "$0100" },
    { "check": "overlap" }
  ],
  "tests": [
    { "name": "double, plus3 and memory (self-test)", "timeoutMs": 20000 }
  ]
}
```

The check commands: `test_begin` at the start, then `assert_a_eq`, `assert_x_eq`, `assert_y_eq` and `assert_mem_eq address, value`, and `test_end` at the end. The registers are preserved by the check commands.

The mailbox is at `$033C` on the C64, at `$0B00` on the C128 and at `$0600` on the Atari 8-bit (the libraries are called `c64/test.inc`, `c128/test.inc`, `atari8bit/test.inc`). The program must not overwrite this location. Plus/4, VIC-20 and MEGA65 have no mailbox; there, the test is skipped. The mode is checked with VICE (C64) and atari800 (Atari 8-bit).

Mailbox tests also run on a real **Ultimate 64**, but only on the command line (see below). Because the device is reset and its memory is overwritten, `vcc test` asks **every** time you run on hardware. You can skip this question only with `--yes`.

## On the command line

The same tests run without the user interface, for example in an automatic check (pipeline):

```bash
vcc build examples/c64/tests_demo
vcc test examples/c64/tests_demo
vcc test examples/c64/tests_demo --backend vice
```

The first call builds, the second runs only the static checks (tests in the emulator are "skipped"), the third adds VICE. The exit code is 1 for errors or timeouts; notes and skipped tests do not change it. With `--junit file.xml`, `vcc test` also writes JUnit XML. More options: `--filter TEXT`, `--strict`, `--poll`.

In the example `tests_demo`, the output without an emulator looks like this (shortened):

```
tests_demo/tests_demo.vctest  (c64, build/tests_demo.prg)
  OK       Size (without name) <= $0100  $37 (55 bytes)
  OK       Zero page free $02-$1f  no access
  OK       No segment overlap  1 segments
  HINT     Cycles next_color <= 18  18 to 19  [tests_demo.asm:27]
  SKIPPED  Cycles clear_text <= 100  [tests_demo.asm:38]
           cannot be determined: loop at $082F (tests_demo.asm:39)
  SKIPPED  next_color: 15 becomes 0
           needs the emulator (trap mode): `vcc test --backend vice`
10 passed, 0 failed, 0 timed out, 1 hint, 7 skipped
```

## Common stumbling blocks

| Problem | Cause and solution |
|---|---|
| All tests "skipped" | In **Target**, "Static checks only" is chosen, or VICE was not found. Choose VICE in the field **Target**; if VICE is not there, VintageCoder did not find it (the start page shows the detected emulators). |
| The checks do not find the program | You did not build, or `program` points to the wrong file. Build with F7 and check the path (relative to the test file or to a parent folder). |
| Names from the program are not recognized | The `.vcdbg` file next to the program is out of date: build again. |
| A test fails although the program seems right | The test forgot something in the `setup`. The main program does not run, and the zero page and chips keep their values from the previous test. |
| "Cycles ... cannot be determined" | A loop, `jsr` or `jmp (...)` is in the way. Split the subroutine into pieces without a loop, or measure in the emulator. |
| Timeout | The trap was not reached, for example because the subroutine ends in an endless loop. Check `call` and `timeoutMs`. |
| A test with `cycles` is skipped on Plus/4, VIC-20 or C128 | The cycle measurement in the emulator and the default address of the trap exist only for the C64 so far (PAL, x64sc). For the other VICE systems, `trap` must be in the test file, and tests with `cycles` are skipped. |
| Measurement on the Ultimate "invalid" | Packets of the debug stream were lost (for example a firewall for incoming UDP). Repeat. |

## What next?

- The practice program `farbwechsel` comes from the chapter [The Debugger](kapitel:debugger).
- The special features of the target systems (mailbox, emulators): [Target systems compared](kapitel:systeme).
- To look up all fields: [Tests with vcc test](hilfe:tests).
