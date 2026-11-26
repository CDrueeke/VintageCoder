; script1.inc - Eingabeskript fuer den Testbetrieb (-D SCRIPT=1): Titelbild: zwei Spieler, zwei Joysticks, Tempo 7, Start; dann Spieler 1 (Anschluss 1)
; laeuft nach rechts und hoch.  Eintraege: Bilder, Maske Anschluss 1, Maske Anschluss 2 (IN_UP 1, IN_DOWN 2, IN_LEFT 4, IN_RIGHT 8, IN_FIRE 16).
        .byte 60, 0, 0                  ; warten
        .byte 2, 0, 8                   ; rechts (Anschluss 2): Spieler 2
        .byte 20, 0, 0
        .byte 2, 0, 2                   ; runter: Zeile Joystick
        .byte 20, 0, 0
        .byte 2, 0, 8                   ; rechts: Anschluss 1
        .byte 20, 0, 0
        .byte 2, 8, 0                   ; rechts (Anschluss 1): zwei Joysticks
        .byte 20, 0, 0
        .byte 2, 2, 0                   ; runter: Zeile Tempo
        .byte 20, 0, 0
        .byte 2, 0, 8                   ; Tempo 6
        .byte 20, 0, 0
        .byte 2, 0, 8                   ; Tempo 7
        .byte 20, 0, 0
        .byte 120, 0, 0                 ; Titelbild stehen lassen (Bildschirmfoto)
        .byte 2, 0, 16                  ; Feuer: Start
        .byte 200, 0, 0                 ; "player 1 ready"
        .byte 90, 8, 0                  ; Spieler 1 (Anschluss 1) nach rechts
        .byte 60, 1, 0                  ; hoch
        .byte 0
