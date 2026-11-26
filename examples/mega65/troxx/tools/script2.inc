; script2.inc - Eingabeskript fuer den Testbetrieb (-D SCRIPT=2): zwei Spieler abwechselnd mit einem Joystick (Anschluss 2): Titelbild: Spieler 2, Start;
; "spieler 1 bereit": Feuer weiterreichen, dann nach rechts laufen.  Danach tut sich nichts mehr (der Spieler wechselt nach einem Treffer).
        .byte 60, 0, 0
        .byte 2, 0, 8                   ; rechts: zwei Spieler (abwechselnd, ein Joystick Anschluss 2)
        .byte 20, 0, 0
        .byte 120, 0, 0
        .byte 2, 0, 16                  ; Feuer: Start
        .byte 120, 0, 0                 ; "spieler 1 bereit"
        .byte 20, 0, 0
        .byte 2, 0, 16                  ; Feuer: Joystick weitergegeben, los
        .byte 20, 0, 0
        .byte 100, 0, 8                 ; nach rechts
        .byte 60, 0, 1                  ; hoch
        .byte 0
