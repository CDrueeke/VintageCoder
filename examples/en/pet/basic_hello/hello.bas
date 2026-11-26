; hello.bas - PET/CBM: native BASIC without line numbers, with a label
;
; Purpose: smallest BASIC example for the PET (BASIC 2 or 4, dialect cbm_v2). It shows that the source needs no line numbers:
;   the lines are numbered 10, 20, 30 ... at build time and `@ende` is replaced by the line number.
;   Build: `vcc build examples/pet/basic_hello` (or `vcc basic hello.bas --target pet -o hello.prg`); the PRG has the load address
;   $0401 (BASIC start of the PET). Start in the emulator e.g. `xpet -model 4032 -autostart hello.prg` (checked on the 4032 according to the
;   "PET/CBM" help page), otherwise load and RUN.
;
; Flow: clear the screen (CHR$(147)), print "HELLO VINTAGECODER", add up the numbers 1 to 10 (result 55) and print
;   "SUM= 55". Then `GOTO @ende` keeps the program in an endless loop (the STOP key ends it) so that READY. does not
;   appear. The text is written in lower case in the source and appears as capitals on the PET models 2001 to 4032 in the default state
;   (the REM lines below belong to the program and appear in the listing).
;
; Note: `;` is a comment only at the start of a line (does not go into the program); `REM` stays in the program and costs memory.
;   Variables: T = sum, I = loop counter (both floating point).
REM PET - native BASIC (PET/CBM), without line numbers
REM Write text in lower case: appears as capitals in the default state of the PET models
PRINT CHR$(147);"hello vintagecoder"
T=0
FOR I=1 TO 10
T=T+I
NEXT I
PRINT "sum=";T
@ende:
GOTO @ende
