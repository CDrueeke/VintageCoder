# SPDX-License-Identifier: GPL-3.0-or-later
# English catalog; comments for translators are in the German file of the same name.

basic-dialect-not-atari = { $path }: not an Atari dialect

basic-dialect-key-missing = { $key } is missing

basic-dialect-name-missing = name is missing

basic-dialect-code-invalid = invalid code

basic-dialect-ops-missing = ops is missing

basic-dialect-keyword-required = { $path }: required keyword { $need } is missing

basic-prg-too-short = file is too short

basic-prg-end-missing = program end ($0000) is missing

basic-prg-line-truncated = truncated line

basic-prg-line-end-missing = line end (0) is missing

basic-prg-link-mismatch = link pointer in line { $num } is wrong (${ $next }, expected ${ $expect })

basic-detok-structured-header = ' Converted back from PRG (vcc basic --detokenize --structured)

basic-atari-vntp-offset = unexpected VNTP offset ${ $offset }

basic-atari-length-mismatch = length does not match (header: { $head }, file: { $file })

basic-atari-pointers-invalid = pointers in the file header are invalid

basic-atari-vvt-mismatch = variable value table does not match the name table

basic-atari-line-length = invalid line length at ${ $offset }

basic-calc-unknown-word = unknown word { $w }

basic-calc-bad-number = invalid number { $s }

basic-calc-unexpected-char = unexpected character '{ $c }'

basic-calc-rparen-missing = ')' is missing

basic-calc-number-expected = number expected

basic-calc-overflow = overflow

basic-calc-div-zero = division by 0

basic-calc-shift-range = shift count 0-63

basic-calc-empty = empty

basic-calc-trailing = extra text after the expression

basic-b2a-what-number = number { $s }

basic-b2a-what-string-expr = string in an expression

basic-b2a-what-string-var-named = string variable { $name }$

basic-b2a-what-string-array = string array

basic-b2a-what-division = division '/' (returns fractions; integer with --int-div)

basic-b2a-what-power = power '^'

basic-b2a-what-name = { $name }

basic-b2a-what-string-var = string variable

basic-b2a-what-string-var-hyphen = string variable

basic-b2a-what-string-assign = string assignment

basic-b2a-what-for-string = FOR with a string variable

basic-b2a-what-data-text = DATA with text

basic-b2a-what-data-number = DATA number { $s }

basic-b2a-asm-facexp = exponent of the FAC: 0 = number is 0

basic-b2a-asm-line = line { $n }{ $labels }: { $text }

basic-b2a-asm-target = Target: { $t }, stage { $stage } ({ $kind }), style compact. Each BASIC line is a comment before its block (label L<line>).

basic-b2a-asm-kind-str-float = strings and floating point

basic-b2a-asm-kind-str-int = strings and integer

basic-b2a-asm-kind-float-os = floating point via the OS package

basic-b2a-asm-kind-float-rom = floating point via the BASIC ROM

basic-b2a-asm-kind-int = integer

basic-b2a-asm-degflg = DEGFLG: radians (DEG/RAD set it via POKE 251)

basic-b2a-asm-end-program = end of program

basic-b2a-asm-clr = CLR: set all variables to 0, empty the stack

basic-b2a-asm-str-cap = { $shown } (length, capacity { $cap })

basic-b2a-asm-str-addr = { $shown } (length, address)

basic-b2a-asm-end = the program ends here

basic-b2a-asm-end-heap = the string heap lies above (up to RT_HEAPTOP)

basic-b2a-asm-assert = program too big for { $t }: it must end below RT_HEAPTOP

basic-b2a-asm-def-fn = line { $n }: DEF FN{ $name }({ $shown }) - parameter { $shown } is valid only during the call


# I3b
basic-names-kind-array = { $kind }, array
basic-names-kind-function = Function
basic-names-kind-text = Text
basic-names-kind-integer = Integer
basic-names-kind-number = Number
basic-embed-end = End

basic-dialect-base-cycle = Dialect { $id }: base chain too deep or cyclic
