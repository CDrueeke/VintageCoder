; script2.inc - Eingabeskript fuer den Testbetrieb (-D SCRIPT=2, mit -D LIVES0=1 -D TESTSCORE=$55): Titelbild: Start mit den Voreinstellungen (ein
; Spieler, Anschluss 2); das Spiel steht still (die Figur bleibt am Startknoten), bis der Laeufer trifft; danach Namenseingabe: B, dann Stelle
; weiter, C, dann Stelle weiter, Leerzeichen (runter von A), Feuer: Name "BC ".
        .byte 30, 0, 0
        .byte 2, 0, 16                  ; Feuer: Start
        .byte 255, 0, 0                 ; bis zum Spielende nichts druecken (das Spielende liegt bei etwa Bild 1100)
        .byte 255, 0, 0
        .byte 255, 0, 0
        .byte 255, 0, 0
        .byte 135, 0, 0
        .byte 2, 0, 1                   ; hoch: A -> B
        .byte 20, 0, 0
        .byte 2, 0, 8                   ; rechts: zweite Stelle
        .byte 20, 0, 0
        .byte 2, 0, 1                   ; hoch
        .byte 20, 0, 0
        .byte 2, 0, 1                   ; hoch: C
        .byte 20, 0, 0
        .byte 2, 0, 8                   ; rechts: dritte Stelle
        .byte 20, 0, 0
        .byte 2, 0, 2                   ; runter: A -> Leerzeichen
        .byte 20, 0, 0
        .byte 2, 0, 16                  ; Feuer: fertig, speichern
        .byte 120, 0, 0
        .byte 0
