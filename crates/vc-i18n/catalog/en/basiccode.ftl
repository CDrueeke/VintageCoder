# SPDX-License-Identifier: GPL-3.0-or-later
# English catalog; comments for translators are in the German file of the same name.
# Machine-translated, to be reviewed.

basiccode-b150 = A keyword is hidden inside a variable name (for example `TOTAL` contains `TO`). The interpreter splits the name there. Pick a name without a keyword.
basiccode-b151 = A `FOR` has no `NEXT`, or the other way round. Loops must come in pairs and may only be nested, never crossed.
basiccode-b152 = There is a `GOSUB` but no `RETURN` anywhere in the program (or the other way round). Subroutines need both.
basiccode-b153 = A jump goes to a line number that does not exist. The error only shows at run time (`UNDEF'D STATEMENT`). Jumping to labels (`@name`) is safer.
basiccode-b154 = Two names are the same variable to the interpreter because only the first characters count (two on Commodore). Their values overwrite each other.
basiccode-b155 = A string is not closed at the end of the line. That is allowed and stays open until the line ends, but it is often a forgotten quotation mark.
basiccode-b156 = A keyword gets more or fewer arguments than the dialect allows. The expected number is in the message.
basiccode-b157 = The word is a keyword of another dialect and does not belong to the chosen one. The interpreter would read it as a variable or stop with a syntax error.
basiccode-b158 = A colon follows `THEN`. In Commodore BASIC everything after `THEN` belongs to the condition; the colon is usually superfluous.
basiccode-b159 = A function needs its argument in parentheses, and they are missing here.
basiccode-b160 = The files of the program mix line numbers and labels. Building works, but it is hard to read; convert uniformly with “Convert to labels” or “Convert to numbers”.
basiccode-b161 = The conversion or renumbering is not possible this way (for example computed jump targets, or a range that would change the order). Nothing was changed.
basiccode-b162 = The jump target is computed (for example `GOTO A*10`). When converting, the program cannot tell which line is meant; you have to adjust this spot yourself.
basiccode-b163 = The program reads or writes line numbers at run time (for example `RESTORE`, `ERL`). After converting or renumbering those numbers no longer match.
basiccode-b164 = Note: this number sits where a line number could be. When converting it is treated as a line number.
basiccode-b165 = A marker line `;@name` has no statement after it. The marker then has no target.
basiccode-b166 = An included file has a different mode (numbers or labels) than the other files. Converting cannot work uniformly this way.
basiccode-b167 = In this BASIC the apostrophe is not a comment character. The rest of the line is read as program text. Write comments with `REM`.
basiccode-b168 = A marker line (`;@name`) is invalid or used twice. Labels must be unique in the program.
basiccode-b170 = The chosen dialect is not intended for the target system of the project. The build runs, but the result may not suit the machine.
basiccode-b171 = A required part of the dialect is missing (for example an extension). The build cannot continue.
basiccode-b172 = The name for `#define` is a keyword. That would replace every occurrence of the word in the program; pick another name.
basiccode-b173 = The line numbers are not enough for the DATA lines of the machine code. Reduce the step or shorten the program.
basiccode-b174 = The `#define` line is malformed or the name is already taken. Expected: `#define NAME value`; the name consists of letters, digits and `_`.
basiccode-b175 = The structure of `#if`, `#else`, `#endif` is wrong: a part is missing, doubled, or the condition cannot be read.
basiccode-b176 = A `#define` refers to itself, or through too many levels to others. Replacing would never end.
basiccode-b177 = Something went wrong including machine code with `#asm` (syntax, missing label, too large, address not stable). The message names the reason.
basiccode-b178 = The dialect in `#dialect` or in the project settings is unknown. The message lists the known identifiers.
basiccode-b180 = A statement is malformed here in Atari BASIC (for example `THEN`, `TO` or an equals sign is missing).
basiccode-b181 = An expression in Atari BASIC is incomplete or the parentheses do not match.
basiccode-b182 = A name is invalid in Atari BASIC: not an allowed label name, a reserved word, or a name is missing.
basiccode-b183 = A number is invalid or out of range in Atari BASIC (largest number about 9.99E97).
basiccode-b184 = A text contains a character that does not exist in ATASCII, or is longer than 255 characters. With `{ "{" }{ "$" }xx{ "}" }` you write a character as a code.
basiccode-b185 = A limit of Atari BASIC is exceeded: too many variables, empty line, line too long, or line number above 32767.
basiccode-b190 = A `#numbering` line is malformed. Expected: `#numbering start=100 step=5`.
basiccode-b191 = Start or step of the line numbers are invalid: the step must be at least 1, and the start must not exceed the largest value of the system.
basiccode-b192 = At this step the line numbers are not enough for the program. Choose a smaller step or fewer lines.
basiccode-b193 = An included medium (sprite, character set, song …) could not be read or converted. The message names the reason.
basiccode-b194 = The path under `build/export/` is outdated. Include the medium directly, for example `#include "graphics/hero.vcsprite"`; the build creates the DATA lines itself.
basiccode-b195 = The program is larger than the free memory of the target system. Shorten it, use `#asm` for machine code, or turn on the “Build compact” option.
basiccode-b196 = Included media are present as DATA lines, but the dialect has no `RESTORE` to a line. `READ` reads strictly in order; the order of the media matters.
basiccode-anzeigen = Show explanation of { $code }
basiccode-abschaltbar = Can be turned off in the project settings (BASIC, “Disabled messages”).
basiccode-fest = An error cannot be turned off.
