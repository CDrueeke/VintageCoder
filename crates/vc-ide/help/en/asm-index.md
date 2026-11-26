# Assembler: cheat sheet and error codes

A short version for looking things up. The detailed explanation is on the pages [Assembler](help:assembler), [Expressions and functions](help:asm-expr), [Directives](help:asm-directives), [Text and character sets](help:asm-text), [Labels, macros, scopes](help:asm-macros) and [System library](help:asm-library).

The VCASM language is free to use: anyone may implement it in their own tools, see [License and acknowledgements](help:license). The build reports which licenses appear in the included files, see [License notice when translating](help:license-build) (messages `L001` and `L002`).

## Structure of a line

```
label:  instruction operand ; comment
        .directive arguments
NAME =  expression          ; constant (or variable, if created with .var)
*=      $c000               ; set the program counter (like .org)
```

## Numbers

| Notation | Meaning |
|---|---|
| `255` `$ff` `0xff` `%11111111` | decimal, hexadecimal, hexadecimal, binary |
| `'a'` | ASCII value of a character |
| `2.5` `1e3` | floating-point number (only for calculating, see [Expressions](help:asm-expr)) |
| `<x` `>x` | low byte, high byte of x |
| `*` | current program counter |

## Addressing modes (short form)

| Notation | Mode | Example |
|---|---|---|
| `#value` | immediate | `lda #$00` |
| `$12` / `$1234` | zero page / absolute (the assembler chooses the shortest form) | `lda $d020` |
| `$12,x` `$12,y` `$1234,x` `$1234,y` | indexed | `sta $0400,x` |
| `($12,x)` | indirect, pre-indexed | `lda ($fb,x)` |
| `($12),y` | indirect, post-indexed | `lda ($fb),y` |
| `($1234)` | indirect (`jmp` only) | `jmp ($0314)` |
| `a` | accumulator | `asl a` |
| `lda.z` `lda.a` | force the form (zero page / absolute) | `lda.a $10` |

## Directives from A to Z

| Directive | Short description | Page |
|---|---|---|
| `.align n[, skip \| fillbyte]` | to a multiple of n: pad (zero bytes or fill byte) or jump ahead with `skip`, new segment | [Directives](help:asm-directives) |
| `.assert cond[, "text"]` | check a condition, otherwise E207 | [Directives](help:asm-directives) |
| `.atascii "..."` | text as ATASCII | [Text](help:asm-text) |
| `.bank n` | choose ROM bank (0-1023) | [Cartridges](help:cartridges) |
| `.basicstub target[, line][, "text"]` | put a BASIC `SYS` line in front | [Directives](help:asm-directives) |
| `.be16` `.dbyt` | 16 bits, high byte first | [Directives](help:asm-directives) |
| `.be32` | 32 bits, high byte first | [Directives](help:asm-directives) |
| `.branchlong auto/off` | extend branches that are too far automatically | [Directives](help:asm-directives) |
| `.break` | leave a loop | [Directives](help:asm-directives) |
| `.byte` `.db` | bytes and strings | [Directives](help:asm-directives) |
| `.cartridge "type", "NAME"` | create a cartridge instead of a program | [Cartridges](help:cartridges) |
| `.const` `.equ` | constant: `.const NAME = expression` | [Directives](help:asm-directives) |
| `.continue` | next loop pass | [Directives](help:asm-directives) |
| `.cpu 6510 { illegals = ... }` | how to handle undocumented instructions | [Directives](help:asm-directives) |
| `.cycles [name] [max=n]` ... `.endcycles` | count the cycles of a block | [Directives](help:asm-directives) |
| `.dword` `.long` | 32 bits, low byte first (`.long` stores **four** bytes) | [Directives](help:asm-directives) |
| `.encoding "name"` | choose the text encoding (also as a block `{ ... }`) | [Text](help:asm-text) |
| `.enum` ... `.endenum` | enumeration | [Macros](help:asm-macros) |
| `.error "text"` | create error E207 | [Directives](help:asm-directives) |
| `.exitmacro` | end the running macro early | [Macros](help:asm-macros) |
| `.fill n[, expr[, var]]` | fill n bytes / table | [Directives](help:asm-directives) |
| `.for` ... `.endfor` (old: `.next`) | loop | [Directives](help:asm-directives) |
| `.function name(p) = expr` | your own function for expressions | [Expressions](help:asm-expr) |
| `.hex "aa bb"` | bytes from hex digits | [Directives](help:asm-directives) |
| `.if` `.elseif` `.else` `.endif` | conditional translation | [Directives](help:asm-directives) |
| `.ifdef` `.ifndef name` | name defined / not defined | [Directives](help:asm-directives) |
| `.incbin "file"[, off[, len]]` | include a binary file | [Directives](help:asm-directives) |
| `.include "file"` `#include <path>` | include a file or media file (`.vcsprite` ...) (`as name`: own namespace) | [Directives](help:asm-directives) |
| `.lohitable n, expr[, var]` (old: `.lohifill`) | word table: first the low bytes, then the high bytes | [Directives](help:asm-directives) |
| `.macro` ... `.endmacro` | define a macro | [Macros](help:asm-macros) |
| `.namespace name {` ... `}` | named namespace | [Macros](help:asm-macros) |
| `.org addr` / `*= addr` | set the address, new segment | [Directives](help:asm-directives) |
| `.page [n]` | old name of `.align n, skip` (to the next page boundary, without fill bytes) | [Directives](help:asm-directives) |
| `.petscii "..."` | text as PETSCII | [Text](help:asm-text) |
| `.relocate addr` ... `.endrelocate` (old: `.pseudopc`, `.endpseudopc`, `.realpc`) | code for a different run-time address | [Directives](help:asm-directives) |
| `.repeat n[, var]` ... `.endrepeat` | block n times | [Directives](help:asm-directives) |
| `.retaddr target` (old: `.rta`) | address minus 1 (for `rts` jump tables) | [Directives](help:asm-directives) |
| `.run addr` | start address (Atari) | [Directives](help:asm-directives) |
| `.screencode "..."` (old: `.screen`) | text as screen code | [Text](help:asm-text) |
| `.segment "NAME"[, from, to]` | name a segment and check the range | [Cartridges](help:cartridges) |
| `.set name = expr` | change a variable | [Directives](help:asm-directives) |
| `.struct` ... `.endstruct` | structure with field names | [Macros](help:asm-macros) |
| `.target name` | note the target system in the source | [Directives](help:asm-directives) |
| `.text "..."` | text in the encoding of `.encoding` | [Text](help:asm-text) |
| `.textlen "..."` (old: `.ptext`) | text with a length byte in front | [Text](help:asm-text) |
| `.textnull "..."` (old: `.null`) | text with a zero byte at the end | [Text](help:asm-text) |
| `.textshift "..."` (old: `.shift`) | text, last character with bit 7 | [Text](help:asm-text) |
| `.var name = expr` | create a variable | [Directives](help:asm-directives) |
| `.warning "text"` | create warning W200 | [Directives](help:asm-directives) |
| `.word` `.dw` | 16-bit values, low byte first | [Directives](help:asm-directives) |
| `{` ... `}` / `name: {` ... `}` | scope (namespace) | [Macros](help:asm-macros) |

## Operators

From highest to lowest precedence: signs `- + ! ~ < >` in front of a value; `* / %`; `+ -`; `<< >>`; `&`; `^`; bitwise OR; `< <= > >=`; `== !=`; `&&`; `||`. Comparisons give 1 or 0. The details are under [Expressions](help:asm-expr).

## Functions

`sin cos tan asin acos atan atan2 sqrt pow exp log log2 log10 int floor ceil round abs sign min max len range next align` and `pi`. Your own functions use `.function`. Table under [Expressions](help:asm-expr).

## Error codes (E) and warnings (W)

Errors stop the build, warnings do not. The message gives the file, line and code.

| Code | Meaning | Typical cause |
|---|---|---|
| **E201** | undocumented opcode not allowed, or the instruction does not exist on this CPU | `lax` with `illegals = deny`; instruction of another CPU |
| **E202** | error in an expression | unknown symbol, missing parenthesis, division by 0, wrong number of arguments for a function, list instead of number |
| **E203** | value out of range | `lda #256`, `.org` outside 0-65535, branch too far, `.incbin` beyond the end of the file |
| **E204** | name defined twice | two identical labels, constant with a new value, label in `.repeat` without a scope |
| **E205** | unknown instruction | typo, label without a colon, macro not defined |
| **E206** | operand does not fit the instruction | `lda` without an operand, `sta #1`, addressing mode does not exist |
| **E207** | `.assert` failed or `.error` reached | your own checks |
| **E208** | wrong notation of a directive | missing quotation marks, invalid name, unknown directive, unknown special character `{...}` |
| **E209** | macro: wrong number of arguments or name like an instruction | `m 1, 2` for one parameter |
| **E210** | faulty block structure | `.if` without `.endif`, `.macro` without `.endmacro`, `.endcycles` without `.cycles`, `.break` outside a loop |
| **E211** | nested too deeply | macro calls itself, include cycle |
| **E212** | file not found | wrong path for `.include`, `.incbin`, `.encoding` |
| **E219** | medium cannot be included | media file belongs to another target system or is not readable (`.include "graphics/hero.vcsprite"`) |
| **E213** | scope error | `}` without `{`, scope not closed, period in the scope name |
| **E214** | anonymous label missing | `bne -` without a `-` before it, `beq +` without a `+` after it |
| **E215** | addressing mode suffix does not fit | `jmp.z`, `lda.z #1` |
| **E216** | `.set` on something that is not a variable | `.set` on a constant |
| **E217** | loop runs too long | `.for` with more than 65536 passes |
| **E218** | `.basicstub` not at the start of the program | bytes before `.basicstub` |
| **E300** | code or label before `.org`; symbols do not converge | `nop` in the first line; a condition depends on its own address |
| **E301** | segment overlaps or leaves its range | two `.org` with overlapping content, `.segment` with a range that is too small |
| **W104** | undocumented opcode (illegals = warn) | `lax $d020` |
| **W105** | unstable undocumented opcode | `xaa` |
| **W200** | `.warning` reached | your own message |
| **W201** | cycle budget exceeded | `.cycles ... max=n` |
| **W202** | floating-point number was truncated | `.byte 7 / 2.0` |
| **W203** | branch that was too far was extended automatically | `.branchlong auto` |
| **W204** | `.basicstub` not at the BASIC start address of the target system | `.org $0801` on the C128 |
| **W208** | old include path `build/export/...` | instead `.include "graphics/hero.vcsprite"` |

The BASIC translation ([BASIC](help:basic)) also assigns numbers from E301 on for its own errors (for example `E320 TYPE MISMATCH`). The texts on the BASIC page apply there. Errors when creating a cartridge (`crt`, `car`, `rom`, `a0`) appear without a code as plain text, for example "standard cartridge: code must be in $8000-$BFFF"; see [Cartridges](help:cartridges).
