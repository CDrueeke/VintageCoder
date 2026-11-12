; =====================================================================================================================
; music_demo.asm - POKEY-Musik mit dem Original-RMT-Player (Raster Music Tracker)
; =====================================================================================================================
;
; ZWECK
;   Spielt den mit dem Tracker komponierten POKEY-Song music/atune.vcsong (4 Kanäle, 2 Muster zu je 32 Zeilen, Geschwindigkeit 6,
;   Reihenfolge 0 1 0 1, danach Wiederholung) in einer Endlosschleife ab.  Der Song wird als RMT-Modul exportiert und vom
;   Original-RMT-Player abgespielt; die Makros dafür stehen in lib/rmt.inc, der Player in lib/rmtplayr.asm (nicht ändern,
;   fremde Bibliothek).
;
; VORBEREITUNG
;   Vorher exportieren:  vcc media export music/atune.vcsong     (RMT-Modul als ASM: build/export/atune.asm, Label atune_rmt)
;   (`vcc build` erledigt das automatisch.)  Der zweite Song music/tune_pokey.vcsong wird hier nicht benutzt.
;
; ABLAUF
;   1. rmt_start: Player mit dem Modul bei Zeile 0 initialisieren (Rückgabe in A: Instrumentgeschwindigkeit des Moduls, hier 1).
;   2. rmt_vbi: hängt rmt_vbi_entry in den Deferred-VBI ein.  Ab jetzt spielt das Betriebssystem in jedem Bild (50 Hz PAL) den
;      Player an; das Hauptprogramm hat nichts mehr zu tun und läuft in `done` im Kreis.
;   Benutzte Hardware: POKEY ($D200-$D208, vom Player geschrieben), OS-Routinen SETVBV $E45C / XITVBV $E462.
;
; SPEICHERPLAN
;   $2000-$201A Hauptprogramm und rmt_vbi_entry;  $30E0-$33FF Player-Tabellen (belegt
;   sind laut Build $3182-$33FF);  $3400-$394D Player-Code (PLAYER = $3400, muss an einer Seitengrenze liegen);  $4000-$4181 RMT-Modul.
;   Nullseite $CB-$DD: ausschließlich vom Player.
; BAUEN: vcc build . (Projektordner)
; =====================================================================================================================
.target atari8
.cpu sally { illegals = deny }

.org $2000
.include "lib/rmt.inc"

start:  rmt_start atune_rmt, 0      ; Modul + Startzeile (liefert die Instrumentgeschwindigkeit in A, hier 1 = jedes Bild)
        rmt_vbi                     ; ab jetzt spielt der Deferred-VBI die Musik
done:   jmp done

rmt_vbi_code                        ; Deferred-VBI-Routine rmt_vbi_entry (steht hinter done, wird nicht durchlaufen)
.include "lib/rmtplayr.asm"         ; Player: Tabellen ab $30E0, Code ab $3400 (RMT_PLAYER nicht gesetzt -> Standard $3400)
.org $4000                          ; Modul hinter dem Player
.include "build/export/atune.asm"
.run start
