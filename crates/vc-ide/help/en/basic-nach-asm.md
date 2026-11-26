# Converting BASIC to assembler

The command **Convert BASIC to assembler** (menu **File**, without a shortcut) is meant to translate the open BASIC program (`.bas`) into assembler source. In this version it is in the command list, but it is **not implemented yet**: it reports that it follows in a later stage.

The conversion already exists on the command line: `vcc basic2asm program.bas --target c64 -o program.asm`. The switches, the stages and the limits are on the page [BASIC: tools](help:basic-tools).

The conversion is mechanical: it replaces BASIC statements with calls to subroutines. Check the result by hand; not every BASIC program can be converted completely. See also [BASIC](help:basic).
