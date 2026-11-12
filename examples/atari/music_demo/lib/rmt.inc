; =====================================================================================================================
; rmt.inc - Atari 8-Bit: Wiedergabe von RMT-Modulen (Raster Music Tracker) mit dem Original-RMT-Player
; =====================================================================================================================
;
; HERKUNFT
;   Der Player (rmtplayr.asm) ist der RMT-Player 1.28 (Radek Sterba / Raster, GPLv3; Herkunft und Lizenz stehen im Kopf von
;   rmtplayr.asm) in der Schreibweise dieses Assemblers; laut dessen Kopf ist er Byte für Byte identisch zur Originalübersetzung.
;   Songs aus dem Tracker-Panel (POKEY) werden als RMT-Modul exportiert (vcc media export ... -> <name>_rmt) und sind damit
;   auch im Raster Music Tracker ladbar.  Diese Datei enthält nur die Makros, die den Player anwenden.
;
; BENUTZUNG
;   .const RMT_PLAYER = $3400        ; optional (Seitengrenze, Low-Byte 0!); die Tabellen liegen davor: Mono ab PLAYER-$320, Stereo ab PLAYER-$3C0
;   .const RMT_STEREO = 0            ; optional: 0 = 4 Spuren, 1 = 8 Spuren (2 POKEY), 2/3 = 4 Spuren gemischt
;   .include "lib/rmt.inc"
;   ...
;   rmt_start atune_rmt, 0           ; Modul (Label) und Startzeile; danach den Musiktakt einhängen:
;   rmt_vbi                          ; jedes Bild RMT-Wiedergabe (Deferred-VBI); rmt_stop schaltet alles aus
;   ...
;   rmt_vbi_code                     ; EINMAL: legt die Routine rmt_vbi_entry an (der Deferred-VBI springt dorthin)
;   .include "lib/rmtplayr.asm"      ; EINMAL, an einer Adresse ohne Kollision (Tabellen vor PLAYER, Code ab PLAYER)
;
; ABLAUF ZUR LAUFZEIT
;   rmt_start initialisiert den Player (X/Y = Modul, A = Startzeile).  rmt_vbi trägt über die OS-Routine SETVBV ($E45C; A = 7 =
;   Deferred-VBI, X = High-, Y = Low-Byte der neuen Adresse) rmt_vbi_entry in den Deferred-VBI-Vektor ein; ab dann ruft das
;   Betriebssystem in jedem Bild (50 Hz PAL, 60 Hz NTSC) rmt_vbi_entry auf: Play-Routine des Players (jsr Player+3), dann Rückkehr
;   über XITVBV ($E462).  Das OS muss dafür aktiv bleiben (VBI nicht abgeschaltet).
;
; EINGÄNGE DES PLAYERS (Sprungtabelle ab RASTERMUSICTRACKER = PLAYER)
;   +0 rmt_init: X/Y = Modul (Low/High), A = Startzeile; liefert in A die Instrumentgeschwindigkeit des Moduls (1 = jedes Bild);
;   +3 rmt_play: einmal pro Bild aufrufen;  +6 rmt_p3 (Teilroutine des Abspielens, hier ungenutzt);  +9 rmt_silence (alles aus);
;   +12 SetPokey (schreibt die Registerwerte nach POKEY, hier ungenutzt).
;   Belegte Zeropage: $CB-$DD (19 Bytes, ausschließlich vom Player benutzt).
;
; MAKROS
;   rmt_start(module, line)   Player initialisieren; verändert A, X, Y (Player verändert weitere Register nach Belieben)
;   rmt_stop                  Player-Eingang +9: alle POKEY-Kanäle stumm
;   rmt_vbi                   Deferred-VBI setzen (SETVBV); verändert A, X, Y
;   rmt_vbi_code              legt rmt_vbi_entry an (Code: jsr Player+3 / jmp $E462); nicht ausführbar -> hinter das Hauptprogramm
; =====================================================================================================================

.macro rmt_start(module, line)
        ldx #<module
        ldy #>module
        lda #line
        jsr RASTERMUSICTRACKER
.endmacro

.macro rmt_stop
        jsr RASTERMUSICTRACKER+9
.endmacro

.macro rmt_vbi                      ; Deferred-VBI auf rmt_vbi_entry setzen (der Aufrufer legt rmt_vbi_entry an, siehe rmt_vbi_code)
        lda #7
        ldx #>rmt_vbi_entry
        ldy #<rmt_vbi_entry
        jsr $e45c
.endmacro

.macro rmt_vbi_code                 ; Routine des Deferred-VBI
rmt_vbi_entry:
        jsr RASTERMUSICTRACKER+3
        jmp $e462
.endmacro
