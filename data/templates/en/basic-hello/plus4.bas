REM {{name}} - native BASIC 3.5 (Plus/4), without line numbers
PRINT CHR$(147);"HELLO FROM {{name}}"
T=0
FOR I=1 TO 10
T=T+I
NEXT I
PRINT "SUM=";T
@ende:
GOTO @ende
