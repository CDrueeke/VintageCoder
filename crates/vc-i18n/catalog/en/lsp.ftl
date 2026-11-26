# SPDX-License-Identifier: GPL-3.0-or-later
# English catalog; comments for translators are in the German file of the same name.

lsp-cli-help-title = vc-lsp { $version } - language server for VintageCoder assembler
lsp-cli-help-protocol = Speaks the Language Server Protocol over stdin and stdout (messages with Content-Length).
lsp-cli-help-note = Normally started by the editor, not by hand: without input it waits for messages and prints nothing.
lsp-cli-usage = Usage:  vc-lsp [--help] [--version]
lsp-cli-help-editor = Setting up the editor: command "vc-lsp" without arguments, files with the extension .asm.
lsp-cli-help-exit = Exit code: 0 after an orderly exit (shutdown, exit), otherwise 1.
lsp-cli-unknown-option = vc-lsp: unknown option "{ $option }"

lsp-error-method-unsupported = not supported: { $method }

diag-lsp-e000-no-data = Hardware database (data/) not found - set initializationOptions.dataDir or VC_DATA

diag-lsp-e000-load = { $detail }

lsp-diag-context-hint = { $message } – probably defined in the including file

lsp-hover-table-head =
    | Mode | Opcode | Bytes | Cycles |
    |---|---|---|---|
lsp-hover-flags = Flags: `{ $flags }`
lsp-hover-tag-unstable = ⚠ unstable
lsp-hover-tag-undocumented = ⚠ undoc.
lsp-hover-line-chosen = This line: **{ $mode }** (chosen by the assembler) · bytes `{ $bytes }` · { $cycles ->
    [one] 1 cycle
   *[other] { $cycles } cycles
 }
lsp-hover-line-open = This line: mode still open ({ $why })
lsp-hover-line-ambiguous = This line: several modes (macro or repeat), none highlighted
lsp-hover-why-no-run = no assembler run yet
lsp-hover-why-errors = the file contains errors
lsp-hover-why-no-code = the assembler produced no code for the line
lsp-hover-why-ambiguous = the instruction in this line cannot be matched uniquely

lsp-hover-register =
    { "" }**{ $chip }.{ $reg }** — `{ $addr }` · { $access }

    { $doc }

    { "" }*{ $name }*
lsp-hover-register-short = Register **{ $chip }.{ $reg }** – { $doc }
lsp-hover-access-read = read
lsp-hover-access-write = write
lsp-hover-access-rw = read/write
lsp-hover-defined-in = Defined in `{ $file }`:{ $line }

lsp-hover-also = Also: { $names }
lsp-completion-directive = Directive
lsp-completion-alias = Old name of { $name }

lsp-codeaction-canonical = Use the canonical name: { $old } → { $new }

lsp-codeaction-canonical-all = Make all directive names in the file canonical ({ $count })
lsp-completion-line = Line { $line }
lsp-completion-macro = Macro({ $params })

lsp-value-table-head =
    | Format | Value |
    |---|---|
lsp-value-row-dec = | Decimal | { $v } |
lsp-value-row-hex = | Hex | `{ $v }` |
lsp-value-row-bin = | Binary | `{ $v }` |
lsp-value-row-signed8 = | Signed (8 bit) | { $v } |
lsp-value-row-signed16 = | Signed (16 bit) | { $v } |
lsp-value-row-unsigned8 = | 8 bit unsigned | { $v } (`{ $hex }`) |
lsp-value-row-unsigned16 = | 16 bit unsigned | { $v } |
lsp-value-row-ascii = | Character (ASCII) | `'{ $v }'` |
lsp-value-row-lowbyte = | Low byte (`<`) | `{ $hex }` = { $dec } |
lsp-value-row-highbyte = | High byte (`>`) | `{ $hex }` = { $dec } |


lsp-mnemonic-adc = Add with carry
lsp-mnemonic-and = AND with accumulator
lsp-mnemonic-asl = Arithmetic shift left
lsp-mnemonic-bcc = Branch if carry clear
lsp-mnemonic-bcs = Branch if carry set
lsp-mnemonic-beq = Branch if equal (Z=1)
lsp-mnemonic-bit = Test bits
lsp-mnemonic-bmi = Branch if negative
lsp-mnemonic-bne = Branch if not equal (Z=0)
lsp-mnemonic-bpl = Branch if positive
lsp-mnemonic-brk = Software interrupt
lsp-mnemonic-bvc = Branch if no overflow
lsp-mnemonic-bvs = Branch if overflow
lsp-mnemonic-clc = Clear carry
lsp-mnemonic-cld = Decimal mode off
lsp-mnemonic-cli = Enable interrupts
lsp-mnemonic-clv = Clear overflow
lsp-mnemonic-cmp = Compare accumulator
lsp-mnemonic-cpx = Compare X register
lsp-mnemonic-cpy = Compare Y register
lsp-mnemonic-dec = Decrement memory
lsp-mnemonic-dex = Decrement X
lsp-mnemonic-dey = Decrement Y
lsp-mnemonic-eor = Exclusive OR with accumulator
lsp-mnemonic-inc = Increment memory
lsp-mnemonic-inx = Increment X
lsp-mnemonic-iny = Increment Y
lsp-mnemonic-jmp = Jump
lsp-mnemonic-jsr = Call subroutine
lsp-mnemonic-lda = Load accumulator
lsp-mnemonic-ldx = Load X register
lsp-mnemonic-ldy = Load Y register
lsp-mnemonic-lsr = Logical shift right
lsp-mnemonic-nop = No operation
lsp-mnemonic-ora = OR with accumulator
lsp-mnemonic-pha = Push accumulator onto the stack
lsp-mnemonic-php = Push status register onto the stack
lsp-mnemonic-pla = Pull accumulator from the stack
lsp-mnemonic-plp = Pull status register from the stack
lsp-mnemonic-rol = Rotate left
lsp-mnemonic-ror = Rotate right
lsp-mnemonic-rti = Return from interrupt
lsp-mnemonic-rts = Return from subroutine
lsp-mnemonic-sbc = Subtract with borrow
lsp-mnemonic-sec = Set carry
lsp-mnemonic-sed = Decimal mode on
lsp-mnemonic-sei = Disable interrupts
lsp-mnemonic-sta = Store accumulator
lsp-mnemonic-stx = Store X register
lsp-mnemonic-sty = Store Y register
lsp-mnemonic-tax = A to X
lsp-mnemonic-tay = A to Y
lsp-mnemonic-tsx = Stack pointer to X
lsp-mnemonic-txa = X to A
lsp-mnemonic-txs = X to stack pointer
lsp-mnemonic-tya = Y to A
lsp-mnemonic-asr = Arithmetic shift right, sign is kept (45GS02)
lsp-mnemonic-asw = Shift a 16-bit value in memory left (45GS02)
lsp-mnemonic-row = Rotate a 16-bit value in memory left (45GS02)
lsp-mnemonic-inw = Increment a 16-bit value in the zero page (45GS02)
lsp-mnemonic-dew = Decrement a 16-bit value in the zero page (45GS02)
lsp-mnemonic-phw = Push a 16-bit value (constant or memory) onto the stack (45GS02)
lsp-mnemonic-bbr0 = Branch if bit 0 of the zero-page byte is clear (45GS02; BBR1 to BBR7 likewise)
lsp-mnemonic-bbr1 = Branch if bit 1 of the zero-page byte is clear (45GS02)
lsp-mnemonic-bbr2 = Branch if bit 2 of the zero-page byte is clear (45GS02)
lsp-mnemonic-bbr3 = Branch if bit 3 of the zero-page byte is clear (45GS02)
lsp-mnemonic-bbr4 = Branch if bit 4 of the zero-page byte is clear (45GS02)
lsp-mnemonic-bbr5 = Branch if bit 5 of the zero-page byte is clear (45GS02)
lsp-mnemonic-bbr6 = Branch if bit 6 of the zero-page byte is clear (45GS02)
lsp-mnemonic-bbr7 = Branch if bit 7 of the zero-page byte is clear (45GS02)
lsp-mnemonic-bbs0 = Branch if bit 0 of the zero-page byte is set (45GS02; BBS1 to BBS7 likewise)
lsp-mnemonic-bbs1 = Branch if bit 1 of the zero-page byte is set (45GS02)
lsp-mnemonic-bbs2 = Branch if bit 2 of the zero-page byte is set (45GS02)
lsp-mnemonic-bbs3 = Branch if bit 3 of the zero-page byte is set (45GS02)
lsp-mnemonic-bbs4 = Branch if bit 4 of the zero-page byte is set (45GS02)
lsp-mnemonic-bbs5 = Branch if bit 5 of the zero-page byte is set (45GS02)
lsp-mnemonic-bbs6 = Branch if bit 6 of the zero-page byte is set (45GS02)
lsp-mnemonic-bbs7 = Branch if bit 7 of the zero-page byte is set (45GS02)
lsp-mnemonic-rmb0 = Clear bit 0 in the zero-page byte (45GS02; RMB1 to RMB7 likewise)
lsp-mnemonic-rmb1 = Clear bit 1 in the zero-page byte (45GS02)
lsp-mnemonic-rmb2 = Clear bit 2 in the zero-page byte (45GS02)
lsp-mnemonic-rmb3 = Clear bit 3 in the zero-page byte (45GS02)
lsp-mnemonic-rmb4 = Clear bit 4 in the zero-page byte (45GS02)
lsp-mnemonic-rmb5 = Clear bit 5 in the zero-page byte (45GS02)
lsp-mnemonic-rmb6 = Clear bit 6 in the zero-page byte (45GS02)
lsp-mnemonic-rmb7 = Clear bit 7 in the zero-page byte (45GS02)
lsp-mnemonic-smb0 = Set bit 0 in the zero-page byte (45GS02; SMB1 to SMB7 likewise)
lsp-mnemonic-smb1 = Set bit 1 in the zero-page byte (45GS02)
lsp-mnemonic-smb2 = Set bit 2 in the zero-page byte (45GS02)
lsp-mnemonic-smb3 = Set bit 3 in the zero-page byte (45GS02)
lsp-mnemonic-smb4 = Set bit 4 in the zero-page byte (45GS02)
lsp-mnemonic-smb5 = Set bit 5 in the zero-page byte (45GS02)
lsp-mnemonic-smb6 = Set bit 6 in the zero-page byte (45GS02)
lsp-mnemonic-smb7 = Set bit 7 in the zero-page byte (45GS02)
lsp-mnemonic-bra = Always branch (45GS02)
lsp-mnemonic-bsr = Call subroutine with a relative 16-bit jump (45GS02)
lsp-mnemonic-lbcc = Long branch (16-bit) if carry clear (45GS02)
lsp-mnemonic-lbcs = Long branch (16-bit) if carry set (45GS02)
lsp-mnemonic-lbeq = Long branch (16-bit) if equal (45GS02)
lsp-mnemonic-lbne = Long branch (16-bit) if not equal (45GS02)
lsp-mnemonic-lbmi = Long branch (16-bit) if negative (45GS02)
lsp-mnemonic-lbpl = Long branch (16-bit) if positive (45GS02)
lsp-mnemonic-lbvc = Long branch (16-bit) if no overflow (45GS02)
lsp-mnemonic-lbvs = Long branch (16-bit) if overflow (45GS02)
lsp-mnemonic-lbra = Long branch (16-bit), always (45GS02)
lsp-mnemonic-cle = Clear extended stack flag: 16-bit stack pointer (45GS02)
lsp-mnemonic-see = Set extended stack flag: 8-bit stack pointer (45GS02)
lsp-mnemonic-cpz = Compare Z register (45GS02)
lsp-mnemonic-ldz = Load Z register (45GS02)
lsp-mnemonic-stz = Store Z register (45GS02)
lsp-mnemonic-inz = Increment Z (45GS02)
lsp-mnemonic-dez = Decrement Z (45GS02)
lsp-mnemonic-phx = Push X onto the stack (45GS02)
lsp-mnemonic-phy = Push Y onto the stack (45GS02)
lsp-mnemonic-phz = Push Z onto the stack (45GS02)
lsp-mnemonic-plx = Pull X from the stack (45GS02)
lsp-mnemonic-ply = Pull Y from the stack (45GS02)
lsp-mnemonic-plz = Pull Z from the stack (45GS02)
lsp-mnemonic-taz = A to Z (45GS02)
lsp-mnemonic-tza = Z to A (45GS02)
lsp-mnemonic-tab = A to B, the base page register (45GS02)
lsp-mnemonic-tba = B (base page register) to A (45GS02)
lsp-mnemonic-tsy = High byte of the stack pointer to Y (45GS02)
lsp-mnemonic-tys = Y to the high byte of the stack pointer (45GS02)
lsp-mnemonic-trb = Clear and test accumulator bits in memory (45GS02)
lsp-mnemonic-tsb = Set and test accumulator bits in memory (45GS02)
lsp-mnemonic-neg = Negate accumulator, two's complement (45GS02)
lsp-mnemonic-eom = No operation ($EA); also prefix for 32-bit pointers (45GS02)
lsp-mnemonic-map = Set memory map from A, X, Y, Z (45GS02)
lsp-mnemonic-ldq = Load the 32-bit value Q (Z:Y:X:A) (45GS02)
lsp-mnemonic-stq = Store the 32-bit value Q (45GS02)
lsp-mnemonic-adcq = Add to Q with carry (45GS02)
lsp-mnemonic-sbcq = Subtract from Q with borrow (45GS02)
lsp-mnemonic-andq = Q AND memory (45GS02)
lsp-mnemonic-orq = Q OR memory (45GS02)
lsp-mnemonic-eorq = Q exclusive OR memory (45GS02)
lsp-mnemonic-cpq = Compare Q (45GS02)
lsp-mnemonic-bitq = Test bits of Q (45GS02)
lsp-mnemonic-aslq = Shift Q left (45GS02)
lsp-mnemonic-lsrq = Shift Q right, logical (45GS02)
lsp-mnemonic-rolq = Rotate Q left (45GS02)
lsp-mnemonic-rorq = Rotate Q right (45GS02)
lsp-mnemonic-asrq = Shift Q right, arithmetic (45GS02)
lsp-mnemonic-inq = Increment Q (45GS02)
lsp-mnemonic-deq = Decrement Q (45GS02)
lsp-mnemonic-negq = Negate Q (45GS02)
lsp-mnemonic-lax = Load A and X (undocumented)
lsp-mnemonic-sax = Store A AND X (undocumented)
lsp-mnemonic-dcp = DEC, then CMP (undocumented)
lsp-mnemonic-isc = INC, then SBC (undocumented)
lsp-mnemonic-slo = ASL, then ORA (undocumented)
lsp-mnemonic-rla = ROL, then AND (undocumented)
lsp-mnemonic-sre = LSR, then EOR (undocumented)
lsp-mnemonic-rra = ROR, then ADC (undocumented)
lsp-mnemonic-anc = AND, bit 7 to carry (undocumented)
lsp-mnemonic-alr = AND, then LSR (undocumented)
lsp-mnemonic-arr = AND, then ROR (undocumented)
lsp-mnemonic-axs = (A AND X) minus operand to X (undocumented)
lsp-mnemonic-las = Memory AND stack pointer to A, X, SP (undocumented)
lsp-mnemonic-xaa = unstable: X to A, AND operand (undocumented)
lsp-mnemonic-lxa = unstable: load A and X (undocumented)
lsp-mnemonic-sha = unstable: store A AND X AND (H+1) (undocumented)
lsp-mnemonic-shx = unstable: store X AND (H+1) (undocumented)
lsp-mnemonic-shy = unstable: store Y AND (H+1) (undocumented)
lsp-mnemonic-tas = unstable: A AND X to SP (undocumented)
lsp-mnemonic-jam = The processor halts (undocumented)

lsp-directive-bank =
    Select the ROM bank for the cartridge format (0-1023). Each bank has its own address space.

    Usage:

    - `.bank 1`, then `.org $8000` sets the address in this bank
lsp-directive-segment =
    Name a segment and optionally check the allowed range. Usages:

    - `.segment "CODE", $8000, $9fff` – name it and check the range

    - `.segment "CODE"` – name only

    - `.segment` – clear name and range
lsp-directive-cartridge =
    Write a cartridge instead of a program: `.cartridge "type", "NAME"`. Types:

    - C64: `normal`, `ocean`, `magicdesk`, `easyflash`, `actionreplay`, `ultimax`

    - Atari: `8k`, `16k`, `xegs`

    - VIC-20: `a0`, `20`, `40`, `60` (memory block; raw image with the extension .a0/.20/.40/.60)
lsp-directive-org =
    Set the program counter and start a new segment. Usages:

    - `.org $0801`

    - `*= $0801` – same meaning

    - `.org next(1024)` – to the next address divisible by 1024
lsp-directive-byte = Write bytes: `.byte 1, $ff, "text"`
lsp-directive-word = Write 16-bit values (little endian): `.word label`
lsp-directive-text = Write ASCII text: `.text "hello"`
lsp-directive-petscii =
    Write text as PETSCII (C64). Special characters go in curly braces inside the string:

    - `{ "{" }clr{ "}" }` – name, `{ "{" }rvs on{ "}" }` – name with a space

    - `{ "{" }down*2{ "}" }` – repetition

    - `{ "{" }$c1{ "}" }` or `{ "{" }147{ "}" }` – code in hexadecimal or decimal
lsp-directive-screencode = Write text as screen codes of the target system (C64 family: VIC/TED, Atari: ANTIC)
lsp-directive-atascii =
    Write text as ATASCII (Atari; the line end becomes $9B). Special characters go in curly braces inside the string:

    - `{ "{" }clr{ "}" }`, `{ "{" }eol{ "}" }` – name

    - `{ "{" }$14{ "}" }` or `{ "{" }20{ "}" }` – code in hexadecimal or decimal
lsp-directive-encoding =
    Text encoding for `.text`, `.textnull`, `.textlen`, `.textshift`. Names: `ascii`, `petscii`, `screen`, `atascii` or a 256-byte table. Usages:

    - `.encoding "petscii"` – applies until the next switch

    - `.encoding "petscii" { "{" }` ... `{ "}" }` – inside the block only
lsp-directive-textnull = Text with a terminating zero byte: `.textnull "hello"` (also `.null`)
lsp-directive-textlen = Text with a leading length byte: `.textlen "hello"` (also `.ptext`)
lsp-directive-textshift = Text whose last character has bit 7 set: `.textshift "end"` (also `.shift`)
lsp-directive-retaddr = Word with the value minus 1 (for jumps via RTS): `.retaddr routine` (also `.rta`)
lsp-directive-be16 = 16-bit value, high byte first: `.be16 $1234` (also `.dbyt`)
lsp-directive-be32 = 32-bit value, high byte first: `.be32 $12345678`
lsp-directive-dword = 32-bit value, low byte first: `.dword $12345678` (also `.long`; `.long` writes four bytes here)
lsp-directive-hex = Bytes from hex digits: `.hex "aa bb0c"`
lsp-directive-var =
    Create a variable that can change: `.var x = 0` (may hold decimal numbers and lists like `[1, 2]`). To change it:

    - `x = x + 1`

    - `x += 2` (also `-=`, `*=` ...)

    - `.set x = ...`
lsp-directive-set = Change a variable: `.set x = x + 1` (also `x = ...`, `x += ...`)
lsp-directive-for =
    Loop, each ending with `.endfor`, also `.next` (at most 65536 passes). Usages:

    - `.for i = 0, i < 8, i += 1` – start value, condition, step

    - `.for i in range(n)` – 0 to n-1

    - `.for x in [1, 2, 3]` – values of a list
lsp-directive-endfor = End of `.for` (also `.next`)
lsp-directive-break = Leaves the innermost loop (`.for` or `.repeat`)
lsp-directive-continue = Jumps to the next pass of the innermost loop
lsp-directive-function = One-line function for expressions: `.function lerp(a, b, t) = a + (b - a) * t / 255`; valid from this line on, no recursion
lsp-directive-exitmacro = Ends the running macro early
lsp-directive-fill =
    Fill bytes. Usages:

    - `.fill 4, $ea` – four bytes with a fixed value

    - `.fill 256, i * 2` – the expression may use the loop variable `i` (0-based)

    - `.fill 8, k + 10, k` – third argument: another name for the loop variable
lsp-directive-lohitable = Table of 16-bit values, first all low bytes, then all high bytes: `.lohitable 25, $0400 + 40 * i` (also `.lohifill`)
lsp-directive-branchlong =
    Extend conditional jumps that are too far automatically (inverted branch + JMP, on the 45GS02 LBxx; warning W203). Usages:

    - `.branchlong auto` – switch on

    - `.branchlong off` – switch off (default)
lsp-directive-struct =
    Structure with fields of the form `name type [count]`; defines `Sprite.field` (offset) and `Sprite.size`. Field types:

    - `.byte`, `.word`, `.dword`

    - a structure defined earlier

    Layout: `.struct Sprite` ... `.endstruct`
lsp-directive-endstruct = End of `.struct`
lsp-directive-enum =
    Enumeration: `.enum Color` ... `.endenum`; defines `Color.name`. Entries:

    - `name` – counts up from 0 or from the last value

    - `name = value` – sets the value
lsp-directive-endenum = End of `.enum`
lsp-directive-align =
    Align to a multiple. Usages:

    - `.align 256` – fill with zero bytes

    - `.align 4, $ea` – fill with a fill byte (here NOP)

    - `.align 256, skip` – jump without fill bytes and start a new segment (formerly `.page`)
lsp-directive-const = Define a constant: `.const NAME = expression`
lsp-directive-include = Include a file: `.include "lib/vic2.inc"`
lsp-directive-incbin =
    Include a binary file. Usages:

    - `.incbin "file.bin"` – whole file

    - `.incbin "file.bin", offset` – from an offset

    - `.incbin "file.bin", offset, length` – a section
lsp-directive-if = Conditional assembly: `.if expression` ... `.elseif` ... `.else` ... `.endif`
lsp-directive-ifdef = Assemble only if the symbol is defined
lsp-directive-ifndef = Assemble only if the symbol is not defined
lsp-directive-else = Alternative branch of `.if`
lsp-directive-elseif = Further condition of `.if`
lsp-directive-endif = End of `.if`
lsp-directive-scope = Open a scope: `.scope` ... `.endscope` or `{ "{" }` ... `{ "}" }`
lsp-directive-endscope = End of `.scope` and `.namespace` (also `{ "}" }`)
lsp-directive-namespace = Open a named namespace: `.namespace name { "{" }` ... `{ "}" }`
lsp-directive-macro =
    Define a macro, end with `.endmacro`; `@label` becomes unique for each call. Usages:

    - `.macro name(param, ...)`

    - `.macro name(param, opt=value)` – parameter with a default value (after the required parameters)

    Call: `name value, value` or `name(value, value)`
lsp-directive-endmacro = End of a macro definition
lsp-directive-repeat =
    Repeat a block, end with `.endrepeat`. Usages:

    - `.repeat 8` – eight times

    - `.repeat 8, i` – with loop variable `i` (from 0)
lsp-directive-endrepeat = End of `.repeat`
lsp-directive-relocate = Assemble code for a different run-time address (until `.endrelocate`); formerly `.pseudopc`
lsp-directive-endrelocate = End of `.relocate` (also `.realpc`, `.endpseudopc`)
lsp-directive-cycles = Cycle budget block: `.cycles name max=40` ... `.endcycles`; warning W201 when exceeded
lsp-directive-endcycles = End of `.cycles`
lsp-directive-basicstub =
    Put the BASIC line `SYS <target>` in front. Usages:

    - `.basicstub start` – line 10

    - `.basicstub start, 1000` – own line number 0-63999

    - `.basicstub start, 1000, "text"` – appends `:REM TEXT` (lowercase letters become PETSCII uppercase letters, `{ "{" }name{ "}" }` special characters are allowed)

    Without a previous `.org` the stub starts at the BASIC start address of the target system (C64 $0801, C128 $1C01, Plus/4 and VIC-20 $1001, MEGA65 $2001); with `.org` at another address, W204 warns
lsp-directive-run = Start address for XEX files (Atari): `.run start`
lsp-directive-assert = Check a condition: `.assert expression, "message"`
lsp-directive-error = Raise an error
lsp-directive-warning = Raise a warning
lsp-directive-target = Note the target system in the source (`.target c64` or `.target atari8`); the assembler ignores the line, the IDE reads it when the setting is "automatic".
lsp-directive-cpu =
    Handling of undocumented instructions: `.cpu 6510 { "{" } illegals = warn { "}" }`. Values:

    - `deny` – error

    - `warn` – warning

    - `allow` – allowed
diag-export-wird-beim-bauen-erzeugt = The file '{ $name }' is created when building, from '{ $quelle }'.
