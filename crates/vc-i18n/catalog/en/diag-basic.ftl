# SPDX-License-Identifier: GPL-3.0-or-later
# English catalog; comments for translators are in the German file of the same name.

diag-basic-b183-bad-exponent = invalid exponent in '{ $lit }'

diag-basic-b183-bad-number = invalid number '{ $lit }'

diag-basic-b183-number-range-max = number '{ $lit }' is out of range (max. 9.99...E97)

diag-basic-b183-number-range = number '{ $lit }' is out of range

diag-basic-b100-brace-missing = '{ "}" }' is missing

diag-basic-b100-char-ref = { $text }

diag-basic-b184-atascii-range = character '{ $c }' is not in the ATASCII range (use { "{" }{ "$" }xx{ "}" })

diag-basic-b185-too-many-vars = more than { $n } variables

diag-basic-b182-label-name = '{ $id }' is not a valid label name

diag-basic-b182-label-expected = label expected

diag-basic-b181-rparen-missing = ')' is missing

diag-basic-b180-string-compare-expected = string expected after the comparison operator

diag-basic-b181-expr-incomplete = expression is incomplete

diag-basic-b100-hex-too-big = hex number is too big

diag-basic-b184-string-too-long = string is longer than 255 characters

diag-basic-b100-unexpected-char = unexpected character '{ $c }'

diag-basic-b185-dialect-missing = { $name } is missing

diag-basic-b180-flag-statement-missing = flag statement is missing

diag-basic-b182-reserved-word = reserved word '{ $id }' at this position

diag-basic-b180-assign-expected = '=' expected after '{ $id }'

diag-basic-b180-statement-expected = statement expected

diag-basic-b180-then-expected = THEN expected

diag-basic-b180-then-target-expected = statement or line number expected after THEN

diag-basic-b180-for-var-expected = loop variable expected

diag-basic-b180-equals-expected = '=' expected

diag-basic-b180-to-expected = TO expected

diag-basic-b182-variable-expected = variable expected

diag-basic-b180-goto-gosub-expected = GOTO or GOSUB expected

diag-basic-b180-proc-params = PROC/EXEC parameters are not supported in Turbo BASIC XL (variables are global)

diag-basic-b182-name-expected = name expected

diag-basic-b181-lparen-expected = '(' expected

diag-basic-b185-empty-line = empty line

diag-basic-b185-line-too-long-atari = line is too long (more than 255 bytes)

diag-basic-b100-no-line-number = line does not start with a line number

diag-basic-b185-line-number-max-atari = line number { $num } is greater than 32767

diag-basic-b100-line-number-max-cbm = line number { $num } is greater than 63999

diag-basic-b102-line-too-long = line is too long (more than 250 bytes)

diag-basic-b100-petscii-char = character '{ $c }' has no PETSCII equivalent

diag-basic-b100-petscii-char-hint = character '{ $c }' has no PETSCII equivalent (use { "{" }{ "$" }xx{ "}" })

diag-basic-b110-label-undefined = label @{ $name } is not defined

diag-basic-b121-include-nesting = #include "{ $name }": nested too deeply or circular

diag-basic-b120-include-not-found = #include "{ $name }": file not found (for exported files: build or export first)

diag-basic-b100-line-number-max = line number { $digits } is greater than { $max }

diag-basic-b100-line-number-auto-max = line number { $num } is greater than { $max } (too many lines or step too large)

diag-basic-b100-line-number-order = line number { $num } is not greater than the previous one ({ $last })

diag-basic-b111-label-duplicate = label @{ $name } is defined twice

diag-basic-b112-label-at-end = label @{ $name } is at the end with no statement after it

diag-basic-b100-label-name-missing = label name after '@' is missing

diag-basic-b100-bad-name = invalid name '{ $word }' (a dot is only allowed in constants such as asm.label)

diag-basic-b100-bin-too-big = binary number is too big

diag-basic-b101-line-long = line is longer than { $max } characters: it can no longer be edited in the editor

diag-basic-b102-line-too-long-split = line is too long (more than 250 bytes) - split the line

diag-basic-b103-reserved-word = '{ $name }' is a reserved word of the structured language

diag-basic-b104-label-duplicate = label '{ $name }' is defined twice

diag-basic-b105-line-numbers-cbm = line numbers above 63999 - reduce step or use pack

diag-basic-b105-line-numbers-atari = line numbers above 32767 - reduce step

diag-basic-b106-else-without-if = else without a matching multi-line if

diag-basic-b106-endif-without-if = endif without if

diag-basic-b106-wend-without-while = wend without while

diag-basic-b106-until-without-repeat = until without repeat

diag-basic-b106-elseif-without-if = elseif without a matching multi-line if (or after else)

diag-basic-b106-do-syntax = do expects nothing, 'while condition' or 'until condition'

diag-basic-b106-loop-syntax = loop expects nothing, 'while condition' or 'until condition'

diag-basic-b106-loop-without-do = loop without do

diag-basic-b106-exit-outside = exit outside of while, repeat or do

diag-basic-b106-endproc-without-proc = endproc without proc

diag-basic-b106-block-not-closed = block is not closed

diag-basic-b107-label-undefined = label '@{ $name }' is not defined

diag-basic-b108-const-syntax = const expects NAME = value

diag-basic-b108-const-value = const expects a single number or string

diag-basic-b109-proc-name-missing = procedure name is missing

diag-basic-b109-proc-head-syntax = expected name(parameter, ...)

diag-basic-b109-proc-param-name = invalid parameter name '{ $p }'

diag-basic-b109-proc-nested = proc must not be inside another proc

diag-basic-b109-proc-undefined = procedure '{ $name }' is not defined

diag-basic-b109-proc-arg-count = procedure '{ $name }' expects { $expected ->
        [one] 1 argument
       *[other] { $expected } arguments
    }, given: { $given }

diag-basic-b110-include-read = { $path }: { $error }

diag-basic-b110-include-depth = include is nested too deeply

diag-basic-b110-include-not-found = include file '{ $name }' not found

diag-basic-b111-directive-without-if = { $word } without #if

diag-basic-b111-else-after-else = #elseif/#else after #else

diag-basic-b111-include-system = { "{" }system{ "}" } in the include path, but the target system is unknown

diag-basic-b111-if-without-endif = #if without #endif

diag-basic-b111-cond-char = character '{ $c }' is not allowed in the condition

diag-basic-b111-cond-incomplete = expression is incomplete

diag-basic-b111-cond-rparen = ')' is missing

diag-basic-b111-cond-unexpected = unexpected '{ $t }'

diag-basic-b111-cond-unknown-name = unknown name '{ $t }' (known: SYSTEM_C64, SYSTEM_C128, SYSTEM_PLUS4, SYSTEM_VIC20, SYSTEM_MEGA65, SYSTEM_ATARI and -D constants)

diag-basic-b112-restore-label = restore with a label does not exist in BASIC V2 (only from BASIC 3.5): reset without a label and read in order

diag-basic-b113-blob-syntax = blob expects: blob "file.asm" [with "song.asm"]

diag-basic-b130-separator = separator _ is only allowed singly between two digits

diag-basic-b131-separator = separator _ is only allowed singly between two digits

diag-basic-b131-not-binary-digit = '{ $c }' is not a binary digit (only 0 and 1)

diag-basic-b130-not-hex-digit = '{ $c }' is not a hex digit (0-9, A-F)

diag-basic-b132-no-digits = { $prefix } without digits

diag-basic-b130-not-digit-after = '{ $c }' is not a hex digit (put a space before a keyword)

diag-basic-b131-not-digit-after = '{ $c }' is not a binary digit

diag-basic-b133-hex-bits = hex number above { $bits } bits (largest value { $max })

diag-basic-b133-bin-bits = binary number above { $bits } bits (largest value { $max })

diag-basic-b134-address-range = address { $v } is outside 0-65535

diag-basic-b135-value-range = value { $v } is outside 0-255 ({ $kw } writes one byte)

diag-basic-b136-hex-keyword = hex number is next to the keyword { $kw }: add a space (digits A-F could belong to the word)

diag-basic-b137-int-range = { $v } does not fit in an integer variable (-32768 ... 32767)

diag-basic-b138-jump-target = hex/binary as a jump target: line numbers are created when building, use a label (@name)

diag-basic-b139-c-hex = 0x... is not BASIC: write hex numbers as { "$" }FF

diag-basic-b140-classic-literal = { $t } is not converted in the classic path, the dialect has no hex/binary numbers (build as native BASIC: vcc basic { $file }, or write a decimal number)

diag-basic-b131-no-short-name = no free short name is left for the name '{ $key }' (too many variables)

diag-basic-w313-names-collide = { $names } differ only after the first { $sig } characters ('{ $nat }'); they are kept apart here ({ $shorts }). A BASIC without this conversion would see only one variable

diag-basic-w312-same-variable = { $a } and { $b } are the same variable in BASIC (only the first 2 characters count)

diag-basic-i110-float-var = variable { $name } is a floating-point number in the original; stage 1 uses 16-bit integers

diag-basic-e321-string-dim = string variable { $name } needs a DIM with a fixed size (Atari BASIC: DIM { $name }(n), otherwise error 9)

diag-basic-e303-too-big-16 = number { $n } is too big for 16 bits

diag-basic-e303-data-too-big-16 = DATA number { $n } is too big for 16 bits

diag-basic-e303-mbf-too-big = number { $v } is too big for the BASIC number format

diag-basic-e303-bcd-too-big = number { $v } is too big for the BASIC number format (at most 9.99999999E+97)

diag-basic-e303-number-invalid = number { $s } is invalid

diag-basic-i111-big-numbers = numbers above 32767 are treated as 16-bit patterns (addresses for PEEK/POKE/SYS); in calculations and in PRINT they appear negative

diag-basic-i112-ti-16bit = TI returns only the lower 16 bits of the system clock

diag-basic-i113-int-noop = INT has no effect in stage 1 (integers only)

diag-basic-e304-one-dim-stage1 = stage 1 supports one-dimensional arrays only

diag-basic-e304-one-dim = only one-dimensional arrays are implemented

diag-basic-e304-dim-string-one-size = DIM of a string needs exactly one size: DIM A$(n)

diag-basic-e304-substring = substring: A$(n) or A$(n,m)

diag-basic-e305-one-arg = { $name } expects exactly one argument

diag-basic-e305-open-args = OPEN expects #channel, number, number, name

diag-basic-e305-arg-count = { $n }: wrong number of arguments

diag-basic-e306-function = function { $name } is not translated

diag-basic-e307-assign = { $name } cannot be assigned

diag-basic-e201-jump-target = jump target: line { $n } does not exist

diag-basic-e320-assign = types do not match (TYPE MISMATCH): string and number

diag-basic-e320-open-name = OPEN: the file name must be a string (TYPE MISMATCH)

diag-basic-e320-number-expected = number expected, but got a string (TYPE MISMATCH)

diag-basic-e320-string-expected = string, but a number was expected (TYPE MISMATCH)

diag-basic-e320-string-var-number = string variable { $name }$, but a number was expected (TYPE MISMATCH)

diag-basic-e320-string-ops = only + and comparisons are allowed with strings (TYPE MISMATCH)

diag-basic-e320-compare = a string and a number cannot be compared (TYPE MISMATCH)

diag-basic-e320-arith = you cannot calculate with a string (TYPE MISMATCH)

diag-basic-e320-expects-string = { $n } expects a string (TYPE MISMATCH)

diag-basic-e320-dec-string = DEC expects a string (TYPE MISMATCH)

diag-basic-e320-instr = INSTR(a$,b$) or INSTR(a$,b$,n) with strings (TYPE MISMATCH)

diag-basic-e320-returns-string = { $n } returns a string, but a number is expected here (TYPE MISMATCH)

diag-basic-e320-concat = a string and a number cannot be joined (TYPE MISMATCH)

diag-basic-e320-str-number = STR$ expects a number (TYPE MISMATCH)

diag-basic-e320-string-expr = string expression expected (TYPE MISMATCH)

diag-basic-e320-fn-number = FN{ $name } expects a number (TYPE MISMATCH)

diag-basic-e308-input-multi = INPUT with several variables is not translated in stage 1 (use one INPUT per variable)

diag-basic-e311-get-string = GET into a string does not exist in Atari BASIC (GET #n,A reads one byte into a number)

diag-basic-e311-unsupported = { $k } is not translated (files, LIST/NEW/CONT and DEF FN are not part of the conversion)

diag-basic-e311-plus-strings = Atari BASIC has no + for strings (A$(LEN(A$)+1) = B$ appends)

diag-basic-e311-not-in-atari = { $n } does not exist in Atari BASIC (use the substring A$(n,m))

diag-basic-e311-atari-only = '{ $w }' is Atari BASIC and is not translated (channel and file commands, XIO, TRAP, NOTE/POINT, PADDLE and others)

diag-basic-e314-for-int = FOR variable { $name }% must not be an integer variable (a SYNTAX ERROR in BASIC V2)

diag-basic-e309-poke-range = POKE value must be 0..255

diag-basic-e310-run-line = RUN with a line number is not translated

diag-basic-e312-dim-size-range = DIM A$(n): n must be 1 to 32767 (Atari BASIC: otherwise error 9)

diag-basic-e312-dim-size-number = DIM A$(n) needs a number as the size in basic2asm

diag-basic-e312-dim-number = DIM needs a number as the size in stage 1

diag-basic-e313-redim = array { $name } is dimensioned twice (REDIM'D ARRAY)

diag-basic-e315-fn-string = FN{ $name }: string functions and parameters are not translated

diag-basic-e315-fn-string-plain = string functions are not translated

diag-basic-e316-fn-twice = FN{ $name } is defined twice

diag-basic-e317-fn-undefined = FN{ $name } is not defined anywhere with DEF FN (UNDEF'D FUNCTION)

diag-basic-e322-data-entry = DATA entry '{ $entry }' is not a number (Atari BASIC: error 8 when reading)

diag-basic-e340-proc-twice = procedure { $n } is defined twice

diag-basic-e341-open-blocks = { $n ->
        [one] 1 block
       *[other] { $n } blocks
    } of Turbo BASIC XL not closed at the end of the program (ENDIF, WEND, UNTIL, LOOP or ENDPROC is missing)

diag-basic-e334-tan = TAN does not exist in Atari BASIC (use SIN(x)/COS(x))

diag-basic-e342-else = ELSE without a matching IF (or a second ELSE)

diag-basic-e342-endif = ENDIF without a matching IF

diag-basic-e342-wend = WEND without a matching WHILE

diag-basic-e342-until = UNTIL without a matching REPEAT

diag-basic-e342-loop = LOOP without a matching DO

diag-basic-e342-exit = EXIT outside of WHILE, REPEAT and DO

diag-basic-e342-proc-nested = PROC inside a procedure

diag-basic-e342-endproc = ENDPROC without a matching PROC

diag-basic-e343-on-exec-undefined = ON ... EXEC: procedure { $n } is not defined

diag-basic-e343-exec-undefined = EXEC: procedure { $n } is not defined

diag-basic-e330-target = target '{ $target }' is not implemented (available: { $available })

diag-basic-e331-float-target = floating point (stage 2) is not yet implemented for the target { $t } (available: { $available }); the program needs it

diag-basic-e300-stage = stage { $stage } is not implemented; stages 1 (integer), 2 (floating point) and 3 (strings) are available

diag-basic-e301-needs-stage = { $what } needs stage 2 (floating point); the highest allowed stage is { $max }

diag-basic-e302-needs-stage = { $what } needs stage 3 (strings); the highest allowed stage is { $max }

diag-basic-e344-turbo-target = Turbo BASIC XL exists only for the target atari8

diag-basic-e192-embed-read = embed: { $path }: { $error }

diag-basic-e192-embed-loop = embed: include loop at { $name }

diag-basic-e192-embed-data-dir = embed needs the data folder (B2aOptions::data_dir)

diag-basic-e191-roundtrip = round trip gives a different syntax tree (first difference in line { $at })

diag-basic-i150-atari-bcd = Atari: numbers are 6-byte BCD of the OS floating-point package (as in Atari BASIC); TAN does not exist; PRINT prints numbers like Atari BASIC without a sign position; error messages are named as on CBM (e.g. ILLEGAL QUANTITY)

diag-basic-i150-atari-int = Atari: numbers are 16-bit integers instead of BCD with 10 digits; PRINT prints numbers like Atari BASIC without a sign position; error messages are named as on CBM (e.g. ILLEGAL QUANTITY)

diag-basic-e190-unsupported = { $k } cannot be written back (statement is not translated)

diag-basic-e190-line = line { $n }: { $e }

diag-basic-e100-no-line-number = line does not start with a line number

diag-basic-e101-unknown-token = unknown token ${ $b }

diag-basic-e101-unexpected-byte = unexpected character ${ $b }

diag-basic-e102-line-twice = line number { $num } appears twice

diag-basic-e103-line = line { $num }: { $e }

diag-basic-e103-sym-expected = '{ $c }' expected

diag-basic-e103-line-no-invalid = line number { $s } is invalid

diag-basic-e103-line-no-expected = line number expected

diag-basic-e103-not-sign = NOT followed by a sign (NOT -1) is error 11 in Atari BASIC; use parentheses: NOT (-1)

diag-basic-e103-fn-name-expected = name expected after FN

diag-basic-e103-fn-one-arg = FN expects exactly one argument

diag-basic-e103-rand-one-arg = RAND expects exactly one argument

diag-basic-e103-kw-not-allowed = { $k } is not allowed here

diag-basic-e103-expr-expected-found = expression expected, found { $found }

diag-basic-e103-expr-expected = expression expected

diag-basic-e103-variable-expected = variable expected

diag-basic-e103-unexpected = unexpected: { $found }

diag-basic-e103-equals-expected = '=' expected

diag-basic-e103-stmt-expected-found = statement expected, found { $found }

diag-basic-e103-stmt-expected = statement expected

diag-basic-e103-name-after-expected = name expected after { $k }

diag-basic-e103-move-three = -MOVE expects 3 numbers

diag-basic-e103-numbers-expected = { $k } expects { $want } numbers

diag-basic-e103-semicolon-channel = ';' expected after the channel number

diag-basic-e103-semicolon-prompt = ';' expected after the input prompt

diag-basic-e103-to-after-go = TO expected after GO

diag-basic-e103-proc-name-on-exec = procedure name expected after ON ... EXEC

diag-basic-e103-to-expected = TO expected

diag-basic-e103-goto-gosub-after-on = GOTO or GOSUB expected after ON

diag-basic-e103-then-or-goto = THEN or GOTO expected after IF

diag-basic-e103-loop-var-expected = loop variable expected

diag-basic-e103-array-name-expected = array name expected

diag-basic-e103-fn-after-def = FN expected after DEF

diag-basic-e103-fn-name-after-fn = function name expected after FN

diag-basic-e103-param-name-expected = parameter name expected

diag-basic-e103-not-a-statement = { $k } is not a statement here


diag-basic-b170-dialect-system = dialect '{ $id }' is not intended for target system '{ $system }'

diag-basic-b178-dialect-unknown = unknown BASIC dialect '{ $id }' (known: { $known })

diag-basic-b160-mixed-mode =
    line numbers and labels are mixed ({ $numbers ->
        [one] 1 line
       *[other] { $numbers } lines
    } with a number, { $labels } with a label)

diag-basic-b161-convert-failed = conversion to labels is not possible: { $count } computed jump target(s)

diag-basic-b161-range-labels = renumbering a range only works in line-number mode (some lines have no number)

diag-basic-b161-range-order = renumbering the range would change the order of the lines (line { $num } after { $last })

diag-basic-b162-computed-target = computed jump target after { $word }: { $text }

diag-basic-b163-runtime-line = { $word } returns the line number at run time; after a conversion or renumbering the numbers no longer match

diag-basic-b153-target-missing = jump target { $num } is not a program line

diag-basic-b165-marker-orphan = marker ;@{ $name } has no statement after it

diag-basic-b166-include-mode = { $file }: mode { $mode } does not match the other files of the include set

diag-basic-b168-marker-invalid = invalid or duplicate marker line: { $text }

diag-basic-b150-keyword-in-name = '{ $name }' contains the keyword { $keyword } and is split up

diag-basic-b151-for-without-next = FOR { $var } has no matching NEXT

diag-basic-b151-next-without-for = NEXT { $var } has no matching FOR

diag-basic-b152-gosub-without-return = GOSUB without any RETURN in the program

diag-basic-b152-return-without-gosub = RETURN without any GOSUB in the program

diag-basic-b154-variable-clash = { $name } and { $other } are the same variable to the interpreter (only the first { $chars } characters count)

diag-basic-b155-string-open = string not closed at the end of the line

diag-basic-b156-arg-count = { $word } with { $count } argument(s); expected: { $expected }

diag-basic-b157-keyword-unknown = { $word } is not part of dialect { $dialect }

diag-basic-b159-function-paren = { $word } needs an argument in parentheses

diag-basic-b167-quote-not-comment = ' is not a comment character in this BASIC (comments: REM)

diag-basic-b172-define-keyword = #define: '{ $name }' is a keyword and cannot be used as a name

diag-basic-b174-define-syntax = #define expects a name made of letters, digits and _ : { $text }

diag-basic-b174-define-twice = #define: '{ $name }' is already defined

diag-basic-b175-if-condition = #if: condition '{ $text }' not readable (allowed: SYSTEM_C64, a defined name, a leading !)

diag-basic-b175-else-without-if = #else without #if

diag-basic-b175-else-twice = second #else in the same #if

diag-basic-b175-endif-without-if = #endif without #if

diag-basic-b175-if-not-closed = #if without #endif

diag-basic-b176-define-cycle = #define: '{ $name }' refers to itself (cyclic or more than eight levels)

diag-basic-b177-blob-syntax = #asm expects: #asm "file.asm" [with "song.asm"]

diag-basic-b177-symbol-missing = asm.{ $name }: the machine code does not define a symbol '{ $name }'

diag-basic-b177-too-large = program with machine code ends at ${ $end }: too large

diag-basic-b177-address-unstable = address of the machine code does not settle (BASIC text and numbers influence each other)

diag-basic-b177-code-too-large = machine code ({ $bytes } bytes) does not fit below { "$" }A000

diag-basic-b177-blob-here = #asm needs the build with machine code (vcc basic, or building in the IDE with the assembler attached)

diag-basic-b173-blob-range = line numbers are not enough for the DATA lines of the machine code ({ $lines } lines after { $last }, at most { $max })

diag-basic-b190-numbering-syntax = invalid line '{ $text }' (expected: #numbering start=100 step=5)

diag-basic-b191-numbering-range = line numbers: start { $start } and step { $step } are invalid (step at least 1, start at most { $max })

diag-basic-b192-numbering-overflow = { $count } lines need numbers up to { $last } with this step (largest allowed: { $max }); use a smaller step, e.g. { $suggestion }
diag-basic-b150-keyword-in-name-more = '{ $name }' contains the keyword { $keyword } and is split up (also on line { $lines })

diag-basic-b177-blob-renamed = #blob is now called #asm: #asm "file.asm" [with "song.asm"]

diag-basic-b193-media-failed = #include "{ $name }": { $reason }

diag-basic-b194-legacy-path = #include "{ $name }": outdated path; better include the medium directly: #include "{ $source }"

diag-basic-b195-program-too-big = The program takes { $size } bytes, { $free } bytes are free ({ $system }); it does not fit in memory. Use machine code with #asm or fewer DATA lines

diag-basic-b196-media-restore-order = { $count } media included, but { $dialect } has no RESTORE to a line: READ reads strictly in order; include the media in reading order
