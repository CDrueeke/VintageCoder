; script4.inc - Eingabeskript fuer den Testbetrieb (-D SCRIPT=4): Titelbild: Feuer = Start; Spieler 1 (Anschluss 1) laeuft rechts, dann hoch; Feuer = Pause (stehen lassen,
; Bildschirmfoto), Feuer = weiter, noch ein Stueck rechts.  Eintraege: Bilder, Maske Anschluss 1, Maske Anschluss 2 (IN_UP 1, IN_DOWN 2, IN_LEFT 4, IN_RIGHT 8, IN_FIRE 16).
        .byte 60, 0, 0                  ; warten
        .byte 2, 16, 0                  ; Feuer: Start
        .byte 160, 0, 0                 ; "READY"
        .byte 120, 8, 0                 ; rechts
        .byte 60, 1, 0                  ; hoch (nur an Knoten)
        .byte 60, 8, 0
        .byte 3, 0, 0
        .byte 2, 16, 0                  ; Feuer: Pause
        .byte 3, 0, 0
        .byte 250, 0, 0                 ; Pause stehen lassen
        .byte 250, 0, 0
        .byte 2, 16, 0                  ; Feuer: weiter
        .byte 3, 0, 0
        .byte 120, 8, 0
        .byte 0, 0, 0                   ; Ende
