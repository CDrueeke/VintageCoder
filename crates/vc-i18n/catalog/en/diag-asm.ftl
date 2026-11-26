# SPDX-License-Identifier: GPL-3.0-or-later
# English catalog; comments for translators are in the German file of the same name.

diag-asm-e201-cpu = { $mn } does not exist on CPU { $cpu }

diag-asm-e201-illegal-denied = undocumented opcode { $mn } (illegals=deny)

diag-asm-e202-expression = { $inner } in '{ $expr }'

diag-asm-e202-list-for-number = list instead of number in '{ $expr }' (pick elements with [i] or use .byte/.word for the whole list)

diag-asm-e202-for-in-not-list = .for ... in expects a list or range(...), not '{ $expr }'

diag-asm-e202-list-not-allowed = '{ $op }' is not allowed for lists

diag-asm-e202-not-finite = result is not a finite number

diag-asm-e202-integer-only = '{ $op }' is only allowed for integers

diag-asm-e202-bad-hex = invalid hex number

diag-asm-e202-number-too-large = number too large

diag-asm-e202-bad-float = invalid floating-point number

diag-asm-e202-bad-char-literal = invalid character literal

diag-asm-e202-unexpected-char = unexpected character '{ $ch }'

diag-asm-e202-call-paren-missing = ')' missing after { $name }(...)

diag-asm-e202-fn-arity = { $name }() expects { $expected } argument(s), got { $got }

diag-asm-e202-fn-arity-min = { $name }() expects at least { $expected } argument(s), got { $got }

diag-asm-e202-fn-recursion = function '{ $name }' calls itself

diag-asm-e202-fn-too-deep = functions nested too deeply

diag-asm-e202-in-function = { $inner } (in function { $name })

diag-asm-e202-len-needs-list = len() expects a list

diag-asm-e202-fn-needs-numbers = { $name }() expects numbers, not a list

diag-asm-e202-range-step-zero = range: step must not be 0

diag-asm-e202-range-too-long = range: more than 65536 elements

diag-asm-e202-list-too-long = list too long (at most 65536 elements)

diag-asm-e202-list-op = '{ $op }' is not allowed for lists (only list + list and list * number)

diag-asm-e202-div-zero = division by 0

diag-asm-e202-index-not-list = indexing is only allowed on lists

diag-asm-e202-bracket-missing = ']' missing

diag-asm-e202-paren-missing = ')' missing

diag-asm-e202-index-missing = index missing

diag-asm-e202-index-out-of-range = index { $index } outside the list (length { $len })

diag-asm-e202-incomplete = incomplete expression

diag-asm-e202-next-zero = next(n): n must be greater than 0

diag-asm-e202-unknown-symbol = unknown symbol '{ $name }'

diag-asm-e202-unexpected-operator = unexpected operator '{ $op }'

diag-asm-e202-empty = expression missing

diag-asm-e202-trailing = unexpected characters at end of expression

diag-asm-e203-value-range = value { $value } outside { $lo }..{ $hi }

diag-asm-e203-value-16bit = value { $value } outside 16 bits

diag-asm-e203-value-32bit = value { $value } outside 32 bits

diag-asm-e203-org-range = .org outside 0..65535

diag-asm-e203-bank-range = .bank outside 0..1023

diag-asm-e203-segment-order = .segment: start is after the end

diag-asm-e203-fill-length = .fill: invalid length

diag-asm-e203-lohifill-length = .lohitable: invalid length

diag-asm-e203-page-size = .align: size must be greater than 0

diag-asm-e203-page-address = .align: address outside 0..65535

diag-asm-e203-incbin-range = .incbin: range outside the file

diag-asm-e203-basicstub-line = .basicstub: line number { $line } outside 0–63999

diag-asm-e203-basicstub-too-long = .basicstub: BASIC line too long ({ $len } bytes, at most 200)

diag-asm-e203-branch-range = branch too far ({ $off } bytes)

diag-asm-e203-address-range = address { $value } outside 0..65535

diag-asm-e203-struct-count = .struct: invalid count for '{ $field }'

diag-asm-e203-repeat-count = .repeat: invalid count

diag-asm-e203-text = { $dir }: { $reason }

diag-asm-e203-text-too-long = text longer than 255 characters

diag-asm-e203-text-empty = empty text

diag-asm-e204-symbol = symbol '{ $name }' defined twice

diag-asm-e204-struct = '{ $name }' defined twice

diag-asm-e204-builtin-function = '{ $name }' is a built-in function

diag-asm-e204-function = function '{ $name }' defined twice

diag-asm-e205 = unknown instruction '{ $mn }'

diag-asm-e206-operand-needed = { $mn } needs an operand

diag-asm-e206-mode-unsupported = { $mn } does not support addressing mode { $mode }

diag-asm-e206-quad-mode = { $mn } does not support addressing mode { $mode }

diag-asm-e206-ptr32-cpu = 32-bit pointers [..] do not exist on this CPU

diag-asm-e206-ptr32 = invalid 32-bit pointer

diag-asm-e206-sp-needs-y = ({ "$" }nn,sp) needs ,y after it

diag-asm-e206-indirect = invalid indirect operand

diag-asm-e206-operand = invalid operand

diag-asm-e207-assert = assertion failed

diag-asm-e207-assert-message = assertion failed: { $message }

diag-asm-e207-user = { $text }

diag-asm-e208-var-syntax = { $dir } expects NAME = expression

diag-asm-e208-var-name = { $dir }: invalid variable name '{ $name }'

diag-asm-e208-string-expected = string in quotation marks expected

diag-asm-e208-strings-expected = { $dir } expects strings in quotation marks

diag-asm-e208-encoding-name = .encoding expects a name in quotation marks

diag-asm-e208-encoding-unknown = .encoding: unknown encoding or file '{ $name }'

diag-asm-e208-encoding-size = .encoding: '{ $name }' must contain exactly 256 bytes

diag-asm-e208-segment-name = .segment expects a name in quotation marks

diag-asm-e208-cartridge-type = .cartridge expects the type in quotation marks (e.g. "ocean")

diag-asm-e208-hex-quoted = .hex expects hex digits in quotation marks

diag-asm-e208-hex-even = .hex: even number of hex digits expected

diag-asm-e208-lohifill-syntax = .lohitable expects: .lohitable count, expression(i)

diag-asm-e208-incbin-name = .incbin expects a file name

diag-asm-e208-const-syntax = .const expects NAME = expression

diag-asm-e208-branchlong = .branchlong expects auto, jmp or off

diag-asm-e208-illegals-value = illegals: unknown value '{ $value }'

diag-asm-e208-unknown-directive = unknown directive '{ $name }'

diag-asm-e208-assignment-expected = assignment expected instead of '{ $text }'

diag-asm-e208-struct-name = { $dir }: invalid name '{ $name }'

diag-asm-e208-enum-item-name = .enum: invalid name '{ $name }'

diag-asm-e208-struct-field-syntax = .struct: field expects 'name type [count]', found '{ $text }'

diag-asm-e208-struct-field-name = .struct: invalid field name '{ $field }'

diag-asm-e208-struct-enum-field = .struct: '{ $ty }' is an enumeration; use .byte (or .word) for the field

diag-asm-e208-struct-field-type = .struct: unknown field type '{ $ty }' (.byte, .word, .dword or a previously defined structure)

diag-asm-e208-function-syntax = .function expects: .function name(param, ...) = expression

diag-asm-e208-function-name = .function: invalid name '{ $name }'

diag-asm-e208-include-alias = { $first } ... as: invalid name '{ $name }' (letters, digits and _ only)

diag-asm-e208-include-name = { $first } expects a file name in quotation marks or <angle brackets>

diag-asm-e208-for-in-list = .for ... in expects a list or range(...)

diag-asm-e208-for-syntax = .for expects: .for start, condition, step (e.g. .for i = 0, i < 8, i += 1)

diag-asm-e208-token-unclosed = '{ "{" }' without closing '{ "}" }' (to write a single '{ "{" }', type '{ "{{" }')

diag-asm-e208-token-repeat = { $token }: repeat count 1 to 255 expected

diag-asm-e208-token-hex = { $token }: invalid hex value (00 to ff)

diag-asm-e208-token-number = { $token }: number 0 to 255 expected

diag-asm-e208-token-names = { $token }: names are only available with .petscii/.atascii or .encoding "petscii"/"atascii" (only { "{" }{ "$" }hh{ "}" } and { "{" }nnn{ "}" } here)

diag-asm-e208-token-unknown = unknown special character { $token }

diag-asm-e209-macro-name = macro name '{ $name }' conflicts with an instruction

diag-asm-e209-macro-args = macro '{ $name }' expects { $expected } argument(s), got { $got }

diag-asm-e209-macro-args-range = macro '{ $name }' expects { $min } to { $max } argument(s), got { $got }

diag-asm-e210-outside-loop = { $directive } outside a loop

diag-asm-e210-endcycles = .endcycles without .cycles

diag-asm-e210-exitmacro = .exitmacro outside a macro

diag-asm-e210-macro-param-default = parameter '{ $param }' without a default value follows a parameter with a default value

diag-asm-e210-macro-unclosed = .macro without .endmacro

diag-asm-e210-macro-name-missing = macro name missing

diag-asm-e210-if-unclosed = .if without .endif

diag-asm-e210-repeat-unclosed = .repeat without .endrepeat

diag-asm-e210-block-unclosed = { $directive } without { $end }

diag-asm-e210-for-unclosed = .for without .endfor

diag-asm-e210-unexpected = unexpected { $directive }

diag-asm-e210-cycles-unclosed = .cycles '{ $name }' without .endcycles

diag-asm-e211-include-depth = include nested too deeply (cycle?)

diag-asm-e211-macro-depth = macro '{ $name }' nested too deeply

diag-asm-e212-include-missing = include file '{ $name }' not found

diag-asm-e212-incbin-missing = file '{ $name }' not found

diag-asm-e212-file-error = { $path }: { $error }

diag-asm-e213-scope-name-dot = scope name '{ $name }' must not contain a dot

diag-asm-e213-unmatched-close = '{ "}" }' without opening '{ "{" }'

diag-asm-e213-after-open = nothing may follow '{ "{" }' on the same line

diag-asm-e213-namespace-syntax = .namespace expects: .namespace NAME { "{" }

diag-asm-e213-scope-unclosed = scope '{ $name }' is not closed with '{ "}" }'

diag-asm-e214-before = no matching '{ $label }' label before this

diag-asm-e214-after = no matching '{ $label }' label after this

diag-asm-e215-suffix = suffix .{ $suffix } only applies to zero-page and absolute addressing

diag-asm-e215-form-branch = { $mn }.{ $suffix }: branch instructions have only one form

diag-asm-e215-form-zeropage = { $mn }.{ $suffix }: no zero-page form

diag-asm-e215-form-absolute = { $mn }.{ $suffix }: no absolute form

diag-asm-e216 = variable '{ $name }' was not created with .var

diag-asm-e217 = .for: more than { $limit } iterations

diag-asm-e218 = .basicstub must be at the start of the program (bytes were already output before it)

diag-asm-e300-code-before-org = code before .org

diag-asm-e300-label-before-org = label before .org

diag-asm-e300-pseudopc-before-org = .relocate before .org

diag-asm-e300-no-convergence = symbols do not converge (too many passes)

diag-asm-e301-segment-range = segment '{ $name }' leaves its range ${ $lo }-${ $hi } (${ $from }..${ $to })

diag-asm-e301-overlap = segments overlap: ${ $a } and ${ $b }

diag-asm-e301-overlap-bank = segments overlap (bank { $bank }): ${ $a } and ${ $b }

diag-asm-e301-beyond-ffff = segment at ${ $start } exceeds { "$" }FFFF

diag-asm-w104 = { $mn } is an undocumented opcode (illegals=warn)

diag-asm-w105 = { $mn } is an unstable undocumented opcode

diag-asm-w200 = { $text }

diag-asm-w201 = cycle budget '{ $name }' exceeded: worst case { $max } > { $limit } cycles

diag-asm-w202 = floating-point number { $value } truncated to { $to } (use int(), round() or floor()) in '{ $expr }'

diag-asm-w203 = branch too far for { $mn }: long form ({ $len } bytes) used

diag-asm-w204 = .basicstub is at ${ $pc }, the BASIC start address of the target system is ${ $start } (BASIC will not find the line there)

diag-asm-w205 = .basicstub has no effect on this target (no BASIC with SYS); use .run label to set the start

diag-asm-w206 = .run has no effect on this target (a PRG has no start vector); use .basicstub label to set the start

diag-asm-w207 = No start vector set; add one with .run label


asm-out-empty = no output

asm-out-banks = multiple banks (.bank) exist only in the cartridge formats crt/car/rom

asm-cart-segment-window = bank { $bank }: segment ${ $from }..${ $to } is not inside the window ${ $lo }-${ $hi }

asm-cart-normal-bank = standard cartridge: bank 0 only (choose ocean or magicdesk for several banks)

asm-cart-normal-window = standard cartridge: code must be in $8000-$BFFF

asm-cart-normal-outside = standard cartridge: code outside $8000-$BFFF

asm-cart-max-banks = { $kind }: at most { $max } banks (8 KB)

asm-cart-easyflash-max-banks = easyflash: at most 64 banks (8 KB ROML + 8 KB ROMH each)

asm-cart-easyflash-romh-twice = easyflash: bank { $bank } uses ROMH at both $A000 and $E000

asm-cart-easyflash-bank0-romh = easyflash: bank 0 needs ROMH code (start vectors at $FFFC in the window $E000-$FFFF)

asm-cart-actionreplay-banks = actionreplay: exactly 4 banks (8 KB each at $8000)

asm-cart-ultimax-bank = ultimax: bank 0 only

asm-cart-ultimax-romh = ultimax: ROMH code with reset vector at $FFFC missing in the window $E000-$FFFF

asm-cart-ultimax-outside = ultimax: code only in $8000-$9FFF or $E000-$FFFF

asm-cart-unknown-type-c64 = unknown cartridge type '{ $kind }' (normal, 16k, ocean, magicdesk, easyflash, actionreplay, ultimax)

asm-cart-atari-run = Atari cartridge: start address missing (.run or own vectors at $BFFA)

asm-cart-atari-banks = multiple banks: choose cartridge type xegs

asm-cart-atari-outside = Atari cartridge: code outside ${ $lo }-$BFFF

asm-cart-unknown-type-atari = unknown cartridge type '{ $kind }' (8k, 16k, xegs)

asm-cart-xegs-banks = XEGS: at most 16 banks (128 KB)

asm-cart-unknown-type-vic20 = unknown VIC-20 cartridge type '{ $kind }' (a0, 20, 40, 60)

asm-cart-vic20-banks = VIC-20 cartridge: banks (.bank) are not possible

asm-cart-vic20-window = VIC-20 cartridge: code must be in ${ $from }-${ $to }

asm-cart-vic20-outside = VIC-20 cartridge: code outside ${ $from }-${ $to }

asm-cart-vic20-id = VIC-20 cartridge at $A000: signature A0CBM missing at $A004 (.byte $41,$30,$c3,$c2,$cd; the start vectors .word cold, warm come before it)
diag-asm-e219-media = { $error }
diag-asm-w208-legacy-export = Outdated path '{ $path }': include the media file directly, e.g. .include "{ $media }"
