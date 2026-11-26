# Assembler

An **assembler** translates programs that you write in readable abbreviations such as `lda #$00` into the numbers that the processor actually executes (the **machine code**). The VintageCoder assembler handles the 6502 family: 6502, 6510 (C64), 7501 (Plus/4), 8502 (C128), "Sally" (Atari) and the 45GS02 of the MEGA65 - that is, all target systems of the development environment. While translating, it calculates with expressions, and it knows macros, loops, namespaces and modules.

**The VCASM language is free.** Anyone can implement the syntax and meaning (instructions, directives, expressions, macros) in their own tools, independent of the license of VintageCoder. What you assemble belongs to you. See [License and acknowledgements](help:license).

## The manual pages

| Page | Contents |
|---|---|
| **this page** | My first program, structure of a line, addressing modes, undocumented instructions |
| [Expressions and functions](help:asm-expr) | Number formats, operators and precedence, `<` and `>`, built-in functions, lists, `.function` |
| [Directives](help:asm-directives) | `.org`, data, constants and variables, `.if`, `.repeat`, `.for`, `.include`, `.basicstub`, `.cycles` ... |
| [Text and character sets](help:asm-text) | `.petscii`, `.screencode`, `.atascii`, `.text`, `.encoding`, special characters `{clr}` |
| [Labels, macros, scopes, structures](help:asm-macros) | local and anonymous labels, `.macro`, `{ }`, `.struct`, `.enum` |
| [Segments, banks, modules](help:cartridges) | `.segment`, `.bank`, `.cartridge` |
| [System library](help:asm-library) | `#include <c64/hardware.asm>` and ready-made macros |
| [Cheat sheet and error codes](help:asm-index) | all directives in one table, all error codes |

The target systems are described in [VIC-20](help:vic20), [C128 and Plus/4](help:c128-plus4) and [MEGA65](help:mega65). To build and run, see [Build and run](help:build).

## My first program

The smallest useful program colors the screen border red and writes a word on the screen. Follow these steps:

1. **Create a project:** *File > New project ...* (Cmd/Ctrl+Alt+N), target system **C64**, template **Assembler**. Or on the command line: `vcc new hallo --target c64 --template asm-hello` (see [Start page, setup assistant and new project](help:wizard)). The template creates the file `src/c64/main.asm`.
2. **Replace the source code:** Open `main.asm` and write the program below into it.
3. **Build and run:** **F7** builds the project, **F5** builds and runs the start program in the emulator (see [Build and run](help:build)). On the command line: `vcc build` in the project folder.

```asm roh
; My first program: red border and a greeting on the screen
.target c64
.basicstub start            ; BASIC line "10 SYS 2061", so that RUN starts the program

start:  lda #2              ; 2 = color red
        sta $d020           ; set the border color
        ldx #0
loop:   lda text,x          ; fetch one character ...
        sta $0400,x         ; ... and write it to screen memory
        inx
        cpx #5              ; five characters copied?
        bne loop            ; no: continue
        rts                 ; back to BASIC

text:   .screencode "hello"     ; text as screen codes
```

Line by line:

| Line | Meaning |
|---|---|
| `; ...` | Comment up to the end of the line (`//` also works). The assembler ignores it |
| `.target c64` | Notes the target system in the source code, so that the IDE recognizes it (see [Directives](help:asm-directives)) |
| `.basicstub start` | Creates the BASIC line `10 SYS 2061`. `RUN` executes it and jumps to `start`. The stub starts automatically at the BASIC start address of the C64, `$0801` (an `.org` is not needed) |
| `start:` | A **label**: a name for the address at this place |
| `lda #2` | Load the number 2 into the accumulator (the working register A). `#` means "the number itself" |
| `sta $d020` | Store A at address `$d020`, the border color register |
| `ldx #0` | Load 0 into the index register X |
| `lda text,x` | Load the byte at address `text + X` |
| `sta $0400,x` | Write it to screen memory (which starts at `$0400`) |
| `inx` / `cpx #5` / `bne loop` | Increase X, compare it with 5, and go back to `loop` as long as it is not equal |
| `rts` | Return to the caller (here BASIC) |
| `.screencode "hello"` | Stores the five characters as **screen codes** (see [Text](help:asm-text)) |

After the build, the folder `build/` contains the program (`main.prg`), a listing (`main.lst`: address, bytes and source code side by side), the symbol table and the memory layout. The listing shows what the assembler made of each line:

```
080d  a9 02      start:  lda #2
080f  8d 20 d0           sta $d020
0812  a2 00              ldx #0
0814  bd 20 08   loop:   lda text,x
```

You can see that the program begins at `$0801`, `lda #2` became `a9 02`, `sta $d020` became `8d 20 d0` (address, low byte first), and `text` was replaced by the address `$0820`. Continue with the pages [Directives](help:asm-directives) and [Expressions](help:asm-expr).

## Structure of a line

A line consists of at most four parts: **label**, **instruction** or **directive**, operand and comment.

```
start:  lda #$00        ; label with colon
        sta vic2.border ; chip register as symbols
.loop:  dex             ; local label is valid until the next global one
        bne .loop
```

- **Label:** a name with a colon at the start of the line. `start:` names the address. `.loop:` is a *local* label (see [Labels](help:asm-macros)). Without a colon, the assembler treats the word as an instruction.
- **Instruction:** an abbreviation of the processor (`lda`, `sta`, `jmp` ...). Uppercase and lowercase do not matter. For the CPU instructions, a card with the description appears when you point at the word with the mouse.
- **Directive:** starts with a period and is an instruction to the assembler (see [Directives](help:asm-directives)).
- **Assignment:** `NAME = expression` defines a constant, `*= address` sets the address (like `.org`).
- **Comment:** `;` or `//` up to the end of the line (not inside quotation marks).
- A line contains only **one** instruction. There is no separator such as `:` between several instructions.
- Chip registers are available as `chip.register` (`vic2.border`); the description appears when you point at it with the mouse.

## Addressing modes

The 6502 knows different ways to give the operand:

| Notation | Name | Meaning |
|---|---|---|
| `#$12` | immediate | the number itself |
| `$12` | zero page | address in the first 256 bytes (shorter and faster) |
| `$1234` | absolute | any 16-bit address |
| `$12,x` `$12,y` `$1234,x` `$1234,y` | indexed | address plus register X or Y |
| `($12,x)` | indirect, pre-indexed | pointer in the zero page at `$12 + X` |
| `($12),y` | indirect, post-indexed | pointer in the zero page at `$12`, plus Y |
| `($1234)` | indirect | jump pointer (only `jmp`) |
| `a` or nothing | accumulator | `asl a` or `asl` shifts the accumulator |

The **zero page** (page 0, addresses `$00`-`$FF`) is a memory area that is especially fast to reach. For `$12` or a label with a small value, the assembler automatically chooses the shorter zero-page form. The 45GS02 of the MEGA65 knows more modes, see [MEGA65](help:mega65).

### Forcing the address mode

For forward references (the label is further down), the size of the value is known only in the second pass, and the assembler first plans for the long form. This can shift addresses. With a suffix, you fix the form, for example for exact raster timing or self-modifying code:

```asm
lda.z zeiger     ; zero page (2 bytes); also .zp
lda.a $10        ; absolute (3 bytes), although the value is < 256; also .abs
sta.a $d020,x
zeiger = $fb
;= a5 fb ad 10 00 9d 20 d0
```

A suffix works only with direct addressing (with `,x` / `,y`). Jump instructions, `#value` and indirect forms report **E215**, and so does a form that the instruction does not have (`jmp.z`).

```asm fehler
 jmp.z $10
```

```text
error[E215]: JMP.z: no zero-page form
```

### Other notations

| Notation | Note |
|---|---|
| `blt target` / `bge target` | Alias for `bcc` / `bcs` ("less than", "greater than or equal" after `cmp`) |
| `asl` without operand | Same as `asl a` |
| `nop` on the MEGA65 | Becomes `eom` (`$EA`) |

## Undocumented instructions (6502)

The 6502 family has instructions that were never officially described but work (`lax`, `dcp`, `slo` ...). Whether you may use them is controlled by `illegals = deny | warn | allow` in the `.cpu` header: `.cpu 6510 { illegals = warn }`. With `deny`, such an instruction is the error **E201**, with `warn` it is the warning **W104**. Especially unreliable ones (`xaa`) always warn with **W105**. Common alternative names are recognized: `aso`=`slo`, `lse`=`sre`, `isb`=`ins`=`isc`, `dcm`=`dcp`, `sbx`=`axs`, `asr`=`alr`, `ane`=`xaa`, `axa`=`sha`, `sxa`=`shx`, `sya`=`shy`, `xas`=`tas`, `kil`=`hlt`=`jam`.

## How the assembler works

- **Several passes:** The assembler translates the source code several times until all addresses are fixed. For this reason, you can use labels before their definition. The number of passes is shown in the [build log](help:build).
- **Output:** The bytes are in **segments** (one for each `.org`). Depending on the target system, the result is a PRG (load address + data), an Atari XEX, a cartridge (`.crt`, `.car`) or a disk.
- **Output files:** listing (`.lst`), symbol table (`.sym.txt`), VICE labels (`.vs`), debug info (`.vcdbg`).
- **Command line:** `vcc asm file.asm -o output --target c64 --format prg` translates a single file. `-I folder` adds to the include search (`vcc asm` also includes media files of the project), `-D NAME=value` defines a constant (for `.ifdef`/`.if`), `--illegals deny|warn|allow` sets the default. `vcc build` builds a whole project.
- **Error messages:** `error[E...]` and `warning[W...]` with file and line. The [cheat sheet](help:asm-index) explains the codes.

If you point the mouse at a directive, an instruction or a chip register, a card with the matching description appears.
