; =====================================================================================================================
; gfx_demo.asm - Zeichensatz, Bildschirm und Kachelkarte aus den Editor-Exporten laden (Atari 8 Bit, Textmodus)
; =====================================================================================================================
;
; ZWECK
;   Zeigt, wie Grafikdaten aus dem Projekt (graphics/afont.vcchar = Zeichensatz, graphics/atitle.vcscreen = 40x24-Titelbildschirm,
;   graphics/alevel_tiles.vctiles = Kachelsatz aus 2x2-Kacheln, graphics/alevel.vcscreen = Screen-Bank mit der 20x12-Karte) mit der Makrobibliothek lib/gfx.inc auf den Atari-Textbildschirm
;   (ANTIC-Modus 2, 40x24) gebracht werden.  Das Betriebssystem bleibt aktiv; Display-Liste, Zeichensatz und Farben werden
;   über die OS-Schattenregister gesetzt (Einzelheiten und Makros: Kopf von lib/gfx.inc).
;
; VORBEREITUNG
;   Die Exporte build/export/atitle.asm, alevel_tiles.asm (Kachelsatz) und alevel.asm (Screen-Bank) werden vor dem Übersetzen aus den Grafikdateien
;   erzeugt (`vcc build` oder `vcc media export graphics/<name>.<endung>`, Projektordner = aktuelles Verzeichnis).  Sie liefern die Labels atitle_chars,
;   alevel_tiles_chars (Zielzeichensatz des Kachelsatzes = afont.vcchar, alle 128 Zeichen angepinnt), alevel_tiles_c0.. und alevel_screens_lo/_hi sowie die
;   Konstanten alevel_tiles_chars_len, atitle_bg/_fg und alevel_level_id/_bg/_fg/_tiles_w/_tiles_h.
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
;   Zur Laufzeit angelegt (nicht in der .xex): Bildschirm $4000-$43BF, Zeichensatz $4400-$47FF.  Zeropage $CB-$D0.
; BAUEN:  vcc build . (Projektordner) oder, nach dem Export:  vcc asm gfx_demo.asm --target atari8 --cpu sally -o gfx_demo.xex [-D DEMO_MAP=1]
; =====================================================================================================================
.target atari8
.cpu sally { illegals = deny }

.org $2000
.include "lib/gfx.inc"

start:  gfx_load_charset alevel_tiles_chars, gfx_charset, alevel_tiles_chars_len   ; 1024 Bytes Zeichensatz an die 1-KB-Grenze $4400
.ifdef DEMO_MAP
        gfx_clear 0                 ; Bildschirm mit Leerzeichen füllen (die Karte überschreibt ihn dann ganz)
        gfx_draw_map alevel_level_id        ; Screen der Bank zeichnen (Kachel-Bibliothek, tiles_use siehe unten)
        gfx_colors alevel_level_bg, alevel_level_fg
.else
        gfx_load_screen atitle_chars        ; 960 Bildschirmcodes des Titels nach $4000
        gfx_colors atitle_bg, atitle_fg
.endif
        gfx_set_display                     ; Display-Liste + Zeichensatz (wirkt im nächsten Vertical Blank)
done:   jmp done

gfx_runtime                         ; Unterprogramme der Bibliothek (hier, damit sie nicht ausgeführt werden)
gfx_dlist                           ; Display-Liste (aus gfx_set_display referenziert)
.include "build/export/atitle.asm"
.include "build/export/alevel_tiles.asm"
.include "build/export/alevel.asm"

; Kachel-Bibliothek: Satz alevel_tiles (2x2, ohne Farbspeicher) und Screen-Bank alevel binden, dann die Unterprogramme
        tiles_use alevel_tiles_c0, 0, alevel_tiles_props, alevel_tiles_tw, alevel_tiles_th, alevel_tiles_count, alevel_screens_lo, alevel_screens_hi, alevel_level_tiles_w, alevel_level_tiles_h
        tiles_runtime
.run start
