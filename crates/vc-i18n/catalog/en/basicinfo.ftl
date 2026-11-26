# SPDX-License-Identifier: GPL-3.0-or-later
# English catalog; comments for translators are in the German file of the same name.
# Machine-translated, to be reviewed.

basicinfo-define = `{ $name }` is defined with `#define` (line { $line }): `{ $value }`
basicinfo-define-text = Before numbering, the name is replaced by the value (not inside strings and after `REM`).
basicinfo-dir-define = `#define NAME value`
basicinfo-dir-define-text = Defines a name. Before numbering, it is replaced by the value in the program text (not inside strings and after `REM`).
basicinfo-dir-if = `#if CONDITION`
basicinfo-dir-if-text = Keeps the following lines only if the condition holds: `SYSTEM_C64` (target system), or a defined name (true if its value is not 0). Excluded lines are shown grey.
basicinfo-dir-else = `#else`
basicinfo-dir-else-text = The following lines apply if the condition of the matching `#if` does not hold.
basicinfo-dir-endif = `#endif`
basicinfo-dir-endif-text = Ends the block of `#if`.
basicinfo-dir-asm = `#asm "file.asm" [with "…"]`
basicinfo-dir-asm-text = Includes machine code: the code is assembled and placed behind the BASIC text (MEGA65 and Atari: as DATA lines). In the program, `asm.name` gives the address of the label `name`.
basicinfo-dir-blob = `#blob` no longer exists: the directive is now called `#asm`.
basicinfo-dir-include = `#include "file"`
basicinfo-dir-include-text = Includes a file of the project; a medium (sprite, character set, song …) is turned into DATA lines.
basicinfo-dir-numbering = `#numbering start=N step=N`
basicinfo-dir-numbering-text = Start and step of the line numbers for this file; takes precedence over the project setting.
basicinfo-dir-dialect = `#dialect <id>`
basicinfo-dir-dialect-text = BASIC dialect of this file; takes precedence over the project and target system settings.
basicinfo-dir-longnames = `#longnames`
basicinfo-dir-longnames-text = Switches on long variable names; they are shortened to two characters when building (mapping in the `.vars.txt` file).
