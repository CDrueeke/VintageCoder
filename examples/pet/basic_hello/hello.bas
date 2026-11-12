; hello.bas - PET/CBM: natives BASIC ohne Zeilennummern, mit Marke (Label)
;
; Zweck: kleinstes BASIC-Beispiel fuer den PET (BASIC 2 oder 4, Dialekt cbm_v2). Es zeigt, dass der Quelltext ohne Zeilennummern
;   auskommt: die Zeilen werden beim Bauen mit 10, 20, 30 ... nummeriert und `@ende` wird durch die Zeilennummer ersetzt.
;   Aufbau: `vcc build examples/pet/basic_hello` (oder `vcc basic hello.bas --target pet -o hello.prg`); das PRG hat die Ladeadresse
;   $0401 (BASIC-Anfang des PET). Start im Emulator z. B. `xpet -model 4032 -autostart hello.prg` (laut Hilfeseite "PET/CBM" auf dem 4032
;   geprueft), sonst laden und RUN.
;
; Ablauf: Bildschirm loeschen (CHR$(147)), "HALLO VON VINTAGECODER" ausgeben, die Zahlen 1 bis 10 aufsummieren (Ergebnis 55) und
;   "SUMME= 55" ausgeben. Danach haelt `GOTO @ende` das Programm in einer Endlosschleife (STOP-Taste beendet es), damit READY. nicht
;   erscheint. Der Text steht klein geschrieben im Quelltext und erscheint auf den PET-Modellen 2001 bis 4032 im Grundzustand als
;   Grossbuchstaben (die REM-Zeilen unten gehoeren zum Programm und erscheinen im Listing).
;
; Hinweis: `;` ist nur am Zeilenanfang ein Kommentar (kommt nicht ins Programm); `REM` bleibt im Programm und kostet Speicher.
;   Variablen: T = Summe, I = Schleifenzaehler (beide Gleitkomma).
REM PET - natives BASIC (PET/CBM), ohne Zeilennummern
REM Text klein schreiben: erscheint im Grundzustand der PET-Modelle als Grossbuchstaben
PRINT CHR$(147);"hallo von vintagecoder"
T=0
FOR I=1 TO 10
T=T+I
NEXT I
PRINT "summe=";T
@ende:
GOTO @ende
