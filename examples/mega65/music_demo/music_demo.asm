; =====================================================================================================================
; music_demo.asm - MEGA65: Tracker-Song auf vier SID-Chips (12 Stimmen)
; =====================================================================================================================
; Was das Programm zeigt
;   Der MEGA65 hat vier SID-Chips (SID 1-4 bei $D400, $D420, $D440, $D460, je 3 Stimmen = 12 Stimmen).  Das Programm spielt den mit dem
;   Tracker komponierten Song music/tune4.vcsong (Profil mega65.sid4.song: 12 Spuren je Zeile, 4 Instrumente, 2 Muster zu 32 Zeilen,
;   Reihenfolge 0, 1, 0, 1 und dann wieder von vorn) in einer Endlosschleife.  Zu hören ist die Musik; auf dem Bildschirm ändert sich nichts.
;   Gelernt wird: Song exportieren, Spielroutine einbinden, einmal pro Bild aufrufen.
;
; Bedienung / Ablauf
;   * Der Song muss vor dem Assemblieren exportiert werden: `vcc media export music/tune4.vcsong` erzeugt build/export/tune4.asm
;     (Labels tune4_order, tune4_patterns, tune4_ins und die Konstanten tune4_speed, tune4_rows, tune4_patsize).  `vcc build` im
;     Projektordner erledigt den Export selbst.
;   * Bauen von Hand: vcc asm music_demo.asm --target mega65 --cpu 45gs02 -o music_demo.prg   (Ladeadresse $2000, kein BASIC-Starter:
;     im Emulator/Monitor laden und mit `g 2000` bzw. `SYS 8192` starten).  Für NTSC-Maschinen -D NTSC=1 angeben (andere Frequenztabelle).
;   * Hauptschleife: einmal je Bild (Rasterzeile $FF, also bei 50 Hz PAL / 60 Hz NTSC) ruft sie `music_play` auf.  Ein Aufruf ist ein
;     Takt des Players; der Song hat 6 Takte (Bilder) je Zeile (tune4_speed) und 32 Zeilen je Muster.
;
; Hardware / Bibliothek
;   lib/music.inc (erzeugt aus examples/c64/music_demo/lib/music.inc.in durch tools/gen_music.py): Makros music_init, music_stop,
;   music_runtime und das Unterprogramm music_play.  Es beschreibt die SID-Register $D400-$D47F und führt ein Abbild davon (mus_sid).
;   $D012 = Rasterzeile (Bit 0-7), mit der die Schleife auf den Bildwechsel wartet.
;
; Speicher
;   Programm und Spielroutine ab $2000 (ca. 2,4 KB mit den Songdaten); Zeropage $F7-$FA (Zeiger der Spielroutine); sonst nichts.
;   Das Programm läuft ohne KERNAL-Unterbrechungen (SEI) und kehrt nie zurück.
; =====================================================================================================================
.target mega65
.cpu 45gs02 { illegals = deny }

.org $2000
.include "lib/music.inc"            ; definiert nur Konstanten und Makros, erzeugt hier noch keine Bytes

start:  sei                         ; keine Unterbrechungen: das Programm taktet die Musik selbst über die Rasterzeile
        music_init tune4_order, tune4_patterns, tune4_ins, tune4_speed, tune4_rows, tune4_patsize   ; Zeiger setzen, SIDs löschen, Lautstärke 15
frame:  lda $d012                   ; auf Rasterzeile $FF warten, dann einen Takt spielen
        cmp #$ff
        bne frame
        jsr music_play              ; ein Player-Takt (Zeilenwechsel alle tune4_speed Takte)
wait:   lda $d012                   ; Zeile verlassen, damit pro Bild nur einmal gespielt wird
        cmp #$ff
        beq wait
        bra frame

music_runtime                       ; Spielroutine einmal hinter dem Hauptprogramm ablegen (wird nicht "durchlaufen")
.include "build/export/tune4.asm"   ; Songdaten aus dem Export (Reihenfolge, Muster, Instrumente)
