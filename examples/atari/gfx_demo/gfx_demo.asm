; =====================================================================================================================
; gfx_demo.asm - Zeichensatz, Bildschirm und Kachelkarte aus den Editor-Exporten laden (Atari 8 Bit, Textmodus)
; =====================================================================================================================
;
; ZWECK
;   Zeigt, wie Grafikdaten aus dem Projekt (graphics/afont.vcchar = Zeichensatz, graphics/atitle.vcscreen = 40x24-Titelbildschirm,
;   graphics/alevel.vctiles = 20x12-Karte aus 2x2-Kacheln) mit der Makrobibliothek lib/gfx.inc auf den Atari-Textbildschirm
;   (ANTIC-Modus 2, 40x24) gebracht werden.  Das Betriebssystem bleibt aktiv; Display-Liste, Zeichensatz und Farben werden
;   über die OS-Schattenregister gesetzt (Einzelheiten und Makros: Kopf von lib/gfx.inc).
;
; VORBEREITUNG
;   Die Exporte build/export/afont.asm, atitle.asm, alevel.asm werden vor dem Übersetzen aus den Grafikdateien erzeugt
;   (`vcc build` oder `vcc media export graphics/<name>.<endung>`, Projektordner = aktuelles Verzeichnis).  Sie liefern die
;   Labels afont, atitle_chars, alevel_map, alevel_tiles sowie die Konstanten afont_len, *_bg/_fg, alevel_width/_height.
;
; ABLAUF UND FASSUNGEN (Schalter -D DEMO_MAP=1)
;   * Ohne Schalter: Zeichensatz nach gfx_charset ($4400) kopieren, die 960 Zeichencodes des Titelbildschirms nach gfx_screen ($4000)
;     kopieren, Farben setzen.
;   * Mit -D DEMO_MAP=1: Zeichensatz kopieren, Bildschirm mit Zeichen 0 (Leerzeichen) löschen, die Karte alevel mit ihrer Kachelliste
;     zeichnen (20x12 Kacheln = 40x24 Zeichen), Farben setzen.
;   Danach schaltet gfx_set_display die Display-Liste und den Zeichensatz ein; das Programm bleibt in einer Endlosschleife.
;
; SPEICHERPLAN
;   $2000 Programm, Bibliothekscode (gfx_runtime), Display-Liste (gfx_dlist, 32 Bytes ausgerichtet) und die Exportdaten
;   (Zeichensatz 1024 Bytes, Bildschirm 960 Bytes, Kachelsatz 16 Bytes, Karte 240 Bytes) direkt hintereinander ab $2000.
;   Zur Laufzeit angelegt (nicht in der .xex): Bildschirm $4000-$43BF, Zeichensatz $4400-$47FF.  Nullseite $CB-$D0.
; BAUEN:  vcc build . (Projektordner) oder, nach dem Export:  vcc asm gfx_demo.asm --target atari8 --cpu sally -o gfx_demo.xex [-D DEMO_MAP=1]
; =====================================================================================================================
.target atari8
.cpu sally { illegals = deny }

.org $2000
.include "lib/gfx.inc"

start:  gfx_load_charset afont, gfx_charset, afont_len   ; 1024 Bytes Zeichensatz an die 1-KB-Grenze $4400
.ifdef DEMO_MAP
        gfx_clear 0                 ; Bildschirm mit Leerzeichen füllen (die Karte überschreibt ihn dann ganz)
        gfx_draw_map alevel_map, alevel_tiles, alevel_width, alevel_height
        gfx_colors alevel_bg, alevel_fg
.else
        gfx_load_screen atitle_chars        ; 960 Bildschirmcodes des Titels nach $4000
        gfx_colors atitle_bg, atitle_fg
.endif
        gfx_set_display                     ; Display-Liste + Zeichensatz (wirkt im nächsten Vertical Blank)
done:   jmp done

gfx_runtime                         ; Unterprogramme der Bibliothek (hier, damit sie nicht ausgeführt werden)
gfx_dlist                           ; Display-Liste (aus gfx_set_display referenziert)
.include "build/export/afont.asm"
.include "build/export/atitle.asm"
.include "build/export/alevel.asm"
.run start
