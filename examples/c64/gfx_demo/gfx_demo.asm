; gfx_demo.asm – Zeichensatz, Bildschirm und Kachelkarte aus den Editor-Exporten laden (C64)
;
; Zweck: zeigt, wie Grafikdaten aus den Editoren (Zeichensatz-, Bildschirm- und Kachel-Editor) in ein Programm kommen: Die Daten
;   liegen als Projektdateien in graphics/ (font.vcchar, title.vcscreen, level_tiles.vctiles, level.vcscreen); beim Bauen entstehen
;   daraus Assemblerquelltexte build/export/*.asm, die dieses Programm einbindet. Die Hilfsmakros stehen in lib/gfx.inc.
;
; Vorbereitung: `vcc build examples/c64/gfx_demo` erledigt den Export selbst. Von Hand: im Editor exportieren oder
;   `vcc media export graphics/<name>.vc<art>` (title, level_tiles = Kachelsatz, level = Screen-Bank). Dann PRG laden, RUN.
;
; Zwei Fassungen (Auswahl mit -D, z. B. `vcc asm gfx_demo.asm --target c64 -D DEMO_MAP=1 -o …`):
;   ohne Schalter  Titelbildschirm: Der Zeichensatz (graphics/font.vcchar, ein eigener Satz mit 256 Zeichen) wird nach $2800 kopiert
;                  und aktiviert, die 1000 Zeichencodes und die 1000 Farben von „title“ kommen in Bildschirm und Farb-RAM, Rahmen und
;                  Hintergrund erhalten die Farben des Bildschirms (14 und 6). Zu sehen ist ein kleines Testbild mit einem Kasten
;                  oben links und einigen Zeichen des eigenen Satzes.
;   -D DEMO_MAP=1  Kachelkarte: der Bildschirm wird mit Leerzeichen gelöscht, Rahmen und Hintergrund bekommen die Farben der Karte
;                  (14 und 6), und die Kachelkarte „level“ (20 x 12 Kacheln zu 2 x 2 Zeichen = 40 x 24 Zeichen, Kacheln: leer, Stein,
;                  Gras, Kiste) wird mit der Kachel-Bibliothek gezeichnet. Das Programm bleibt danach in einer Endlosschleife.
;   Wegen des eigenen Zeichensatzes erscheinen statt normaler Schrift die selbst gezeichneten Glyphen (Hires-Zeichen zu 8 x 8 Punkten).
;
; Hardware: VIC-II $D018 (Zeiger auf Bildschirmspeicher und Zeichensatz), $D020 (Rahmenfarbe), $D021 (Hintergrundfarbe 0),
;   Bildschirmspeicher $0400, Farb-RAM $D800, KERNAL-Zelle $0288 (Seite des Bildschirmspeichers für die Textausgabe).
; Speicherplan: Programm, BASIC-Zeile `10 SYS 2061` und alle Exportdaten liegen in einem Block von $0801 bis $22A3 (6819 Bytes; der
;   Export `level` enthält auch einen Zeichensatz `level_chars` von 2048 Bytes, den dieses Programm nicht benutzt). Der Zeichensatz
;   wird zur Laufzeit nach $2800-$2FFF kopiert (VIC-Bank 0), also hinter dem Programmende ($22A3). Zeropage $F7-$FE (siehe gfx.inc).
; Grenzen: Interrupts bleiben gesperrt (SEI ohne CLI), das Programm kehrt nicht zurück; beenden nur mit RESET.
.target c64
.cpu 6510 { illegals = deny }

.org $0801
.basicstub start

.include "lib/gfx.inc"          ; Makros gfx_*, bindet auch die Kachel-Bibliothek c64/tiles.asm ein

start:  sei
        gfx_load_charset level_tiles_chars, $2800   ; Zielzeichensatz des Kachelsatzes = font.vcchar (alle 256 Zeichen angepinnt)
        gfx_charset_at $2800                        ; VIC-II auf den Zeichensatz bei $2800 zeigen lassen
        gfx_screen_at gfx_screen                    ; Bildschirmspeicher bei $0400 (Zeiger setzen, KERNAL-Seite $0288 anpassen)
.ifdef DEMO_MAP
        gfx_clear 32, level_level_bg                ; Bildschirm: Leerzeichen (Code 32), Farb-RAM: Hintergrundfarbe der Karte (unsichtbar)
        gfx_colors level_level_border, level_level_bg
        gfx_draw_map level_level_id                 ; Screen 0 der Bank „level“ mit der Kachel-Bibliothek zeichnen
.else
        gfx_load_screen title_chars, title_attrs    ; 1000 Zeichencodes -> $0400, 1000 Farben -> $D800
        gfx_colors title_border, title_bg
.endif
done:   jmp done                ; Endlosschleife: das Bild bleibt stehen

gfx_runtime                     ; Unterprogramm gfx_memcpy der Kopier-Makros (hinter dem Hauptprogramm, wird nicht ausgeführt)
.include "build/export/title.asm"       ; Daten: title_chars, title_attrs (je 1000 Bytes) und Konstanten title_border, title_bg
.include "build/export/level_tiles.asm" ; Kachelsatz level_tiles (2x2): Tabellen _c0.._c3, _col, _props, level_tiles_chars (2048 Bytes)
.include "build/export/level.asm"       ; Screen-Bank level: Kartenfeld, Adresstabellen level_screens_lo/_hi und Konstanten level_*

; Kachel-Bibliothek: Satz level_tiles (2x2) und Screen-Bank level binden, dann die Unterprogramme
        tiles_use level_tiles_c0, level_tiles_col, level_tiles_props, level_tiles_tw, level_tiles_th, level_tiles_count, level_screens_lo, level_screens_hi, level_level_tiles_w, level_level_tiles_h
        tiles_runtime
