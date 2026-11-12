; hello.bas - CBM-II (6x0/7x0): natives BASIC ohne Zeilennummern, mit Marke (Label)
;
; Zweck: kleinstes BASIC-Beispiel fuer die CBM-II-Reihe (BASIC-Dialekt cbm_v2). Die Zeilen werden beim Bauen nummeriert; `@ende` wird
;   durch die Zeilennummer ersetzt. Aufbau: `vcc build examples/cbm2/basic_hello` (oder `vcc basic hello.bas --target cbm2 -o hello.prg`).
;   Das PRG hat die Ladeadresse $0003 (BASIC-Programme liegen auf dem CBM-II in Bank 1 ab $0003). Start im Emulator z. B.
;   `xcbm2 -model 610 -autostart hello.prg` (laut Hilfeseite "PET/CBM und CBM-II" mit VICE 3.10 auf dem 610 geprueft).
;   Assemblerprogramme gibt es fuer den CBM-II noch nicht (Bankbehandlung fehlt), deshalb hier nur BASIC.
;
; Ablauf: Bildschirm loeschen (CHR$(147)), "HALLO VON CBM-II" ausgeben, die Zahlen 1 bis 10 aufsummieren (Ergebnis 55) und "SUMME= 55"
;   ausgeben. Danach haelt `GOTO @ende` das Programm in einer Endlosschleife (STOP-Taste beendet es).
;   Der Text steht hier in Grossbuchstaben im Quelltext.
;
; Hinweis: `;` ist nur am Zeilenanfang ein Kommentar (kommt nicht ins Programm); die REM-Zeile unten bleibt im Programm.
;   Variablen: T = Summe, I = Schleifenzaehler.
REM CBM-II - natives BASIC (CBM-II), ohne Zeilennummern
PRINT CHR$(147);"HALLO VON CBM-II"
T=0
FOR I=1 TO 10
T=T+I
NEXT I
PRINT "SUMME=";T
@ende:
GOTO @ende
