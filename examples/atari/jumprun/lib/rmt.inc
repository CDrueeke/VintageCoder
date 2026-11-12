; rmt.inc – Atari 8-Bit: Wiedergabe von RMT-Modulen (Raster Music Tracker) mit dem Original-RMT-Player.
;
; Der Player (rmtplayr.asm) ist der RMT-Player 1.28 (Radek Sterba / Raster, GPLv3) in der Schreibweise dieses Assemblers;
; er ist Byte für Byte identisch zur Originalübersetzung. Songs aus dem Tracker-Panel (POKEY)
; werden als RMT-Modul exportiert (vcc media export … → <name>_rmt) und sind damit auch im Raster Music Tracker ladbar.
;
;   .const RMT_PLAYER = $3400        ; optional (Seitengrenze!); die Tabellen liegen davor: Mono ab PLAYER-$320, Stereo ab PLAYER-$3C0
;   .const RMT_STEREO = 0            ; optional: 0 = 4 Spuren, 1 = 8 Spuren (2 POKEY), 2/3 = 4 Spuren gemischt
;   .include "lib/rmt.inc"
;   ...
;   rmt_start atune_rmt, 0           ; Modul (Label) und Startzeile; danach music-Takt einhängen:
;   rmt_vbi                          ; jedes Bild RMT-Wiedergabe (Deferred-VBI); rmt_stop schaltet alles aus
;   ...
;   .include "lib/rmtplayr.asm"      ; EINMAL, an einer Adresse ohne Kollision (Tabellen vor PLAYER, Code ab PLAYER)
;
; In "Hugo & Gumbo" (jumprun.asm) wird nur das Makro rmt_start benutzt: Der Bildrücklauf-Interrupt (vbi) steht in hal_atari.asm und
; ruft den Player (+3) zusammen mit den Klangeffekten auf; rmt_stop, rmt_vbi und rmt_vbi_code bleiben dort ungenutzt.
;
; Eingänge des Players: RASTERMUSICTRACKER (init, X/Y = Modul, A = Startzeile; liefert die Instrumentgeschwindigkeit),
; +3 = Play (einmal pro Bild), +9 = alles aus.  Belegte Zeropage: $CB-$DD.

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
