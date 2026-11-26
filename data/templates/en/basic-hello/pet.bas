REM {{name}} - native BASIC (PET/CBM), without line numbers
REM Write text in lower case: it appears as upper case in the default state of the PET models
PRINT CHR$(147);"hello from vintagecoder"
T=0
FOR I=1 TO 10
T=T+I
NEXT I
PRINT "sum=";T
@ende:
GOTO @ende
