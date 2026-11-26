; script3.inc - Eingabeskript fuer den Testbetrieb (-D SCRIPT=3, dazu -D LIVES0=1 -D TESTSCORE=$90): Titelbild: Feuer = Start; die Figur steht still, bis ein Gegner sie trifft (Spielende);
; dann Namenseingabe: zweimal hoch, rechts, einmal hoch, rechts, Feuer.  Eintraege: Bilder, Maske Anschluss 1, Maske Anschluss 2 (IN_UP 1, IN_DOWN 2, IN_LEFT 4, IN_RIGHT 8, IN_FIRE 16).
        .byte 60, 0, 0                  ; warten
        .byte 2, 16, 0                  ; Feuer: Start
        .byte 2, 0, 0
        .byte 255, 0, 0                 ; warten bis zum Spielende
        .byte 255, 0, 0                 ; warten bis zum Spielende
        .byte 255, 0, 0                 ; warten bis zum Spielende
        .byte 255, 0, 0                 ; warten bis zum Spielende
        .byte 255, 0, 0                 ; warten bis zum Spielende
        .byte 255, 0, 0                 ; warten bis zum Spielende
        .byte 255, 0, 0                 ; warten bis zum Spielende
        .byte 255, 0, 0                 ; warten bis zum Spielende
        .byte 255, 0, 0                 ; warten bis zum Spielende
        .byte 255, 0, 0                 ; warten bis zum Spielende
        .byte 255, 0, 0                 ; warten bis zum Spielende
        .byte 255, 0, 0                 ; warten bis zum Spielende
        .byte 255, 0, 0                 ; warten bis zum Spielende
        .byte 255, 0, 0                 ; warten bis zum Spielende
        .byte 255, 0, 0                 ; warten bis zum Spielende
        .byte 255, 0, 0                 ; warten bis zum Spielende
        .byte 255, 0, 0                 ; warten bis zum Spielende
        .byte 255, 0, 0                 ; warten bis zum Spielende
        .byte 255, 0, 0                 ; warten bis zum Spielende
        .byte 255, 0, 0                 ; warten bis zum Spielende
        .byte 255, 0, 0                 ; warten bis zum Spielende
        .byte 255, 0, 0                 ; warten bis zum Spielende
        .byte 255, 0, 0                 ; warten bis zum Spielende
        .byte 255, 0, 0                 ; warten bis zum Spielende
        .byte 255, 0, 0                 ; warten bis zum Spielende
        .byte 255, 0, 0                 ; warten bis zum Spielende
        .byte 2, 1, 0
        .byte 20, 0, 0
        .byte 2, 1, 0
        .byte 20, 0, 0
        .byte 2, 8, 0
        .byte 20, 0, 0
        .byte 2, 1, 0
        .byte 20, 0, 0
        .byte 2, 8, 0
        .byte 20, 0, 0
        .byte 2, 16, 0                  ; Feuer: Name fertig
        .byte 250, 0, 0
        .byte 0, 0, 0                   ; Ende
