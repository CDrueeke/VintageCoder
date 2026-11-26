; script1.inc - Eingabeskript fuer den Testbetrieb (-D SCRIPT=1): Titelbild: zwei Spieler, gleichzeitig (zwei Joysticks), Tempo 6, Start; dann Figur 0
; (Anschluss 1) laeuft nach rechts, Figur 1 (Anschluss 2) nach links, danach beide hoch.  Eintraege: Bilder, Maske Anschluss 1, Maske Anschluss 2
; (IN_UP 1, IN_DOWN 2, IN_LEFT 4, IN_RIGHT 8, IN_FIRE 16); Dauer 0 beendet das Skript.
        .byte 60, 0, 0                  ; warten
        .byte 2, 0, 8                   ; rechts (Anschluss 2): zwei Spieler
        .byte 20, 0, 0
        .byte 2, 0, 2                   ; runter: Zeile Ablauf
        .byte 20, 0, 0
        .byte 2, 0, 8                   ; rechts: gleichzeitig
        .byte 20, 0, 0
        .byte 2, 0, 2                   ; runter: Zeile Joystick (fest: zwei Joysticks)
        .byte 20, 0, 0
        .byte 2, 0, 2                   ; runter: Zeile Tempo
        .byte 20, 0, 0
        .byte 2, 0, 8                   ; Tempo 6
        .byte 20, 0, 0
        .byte 120, 0, 0                 ; Titelbild stehen lassen (Bildschirmfoto)
        .byte 2, 0, 16                  ; Feuer: Start
        .byte 200, 0, 0                 ; "bereit"
        .byte 90, 8, 4                  ; Figur 0 nach rechts, Figur 1 nach links
        .byte 60, 1, 1                  ; beide hoch
        .byte 0
