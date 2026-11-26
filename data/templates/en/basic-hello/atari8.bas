REM {{name}} - native Atari BASIC, without line numbers
PRINT "HELLO FROM {{name}}"
T=0
FOR I=1 TO 10
T=T+I
NEXT I
PRINT "SUM=";T
@ende:
GOTO @ende
