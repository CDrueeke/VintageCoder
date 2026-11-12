REM {{name}} - natives BASIC 3.5 (C16), ohne Zeilennummern
PRINT CHR$(147);"HALLO VON {{name}}"
T=0
FOR I=1 TO 10
T=T+I
NEXT I
PRINT "SUMME=";T
@ende:
GOTO @ende
