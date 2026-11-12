; =====================================================================================================================
; gfx.inc - Atari 8-Bit: Zeichensätze, Bildschirme und Kachelkarten aus Exporten laden (ANTIC-Modus 2, 40×24 Text)
; =====================================================================================================================
;
; ZWECK
;   Kleine Makrobibliothek, die die Exporte des Editors (vcc media export, build/export/*.asm: Zeichensatz <name>,
;   Bildschirm <name>_chars, Kachelkarte <name>_map/_tiles/_width/_height, Farben <name>_bg/_fg) auf den
;   Textbildschirm bringt.  Das Betriebssystem bleibt aktiv: Farben, Zeichensatz und Display-Liste werden nur über die
;   OS-Schattenregister eingestellt, die das OS im nächsten Vertical Blank in ANTIC/GTIA übernimmt.
;
; BENUTZUNG
;   .include "lib/gfx.inc"
;   ...
;   gfx_load_charset afont, gfx_charset, afont_len   ; 128 Zeichen = 1024 Bytes nach gfx_charset (1-KB-Grenze)
;   gfx_load_screen atitle_chars                     ; 960 Zeichencodes nach gfx_screen
;   gfx_set_display                                  ; Display-Liste (gfx_dlist) + Zeichensatz einschalten
;   gfx_colors atitle_bg, atitle_fg                  ; Hintergrund (COLPF2/Rahmen), Textfarbe (COLPF1: nur die Helligkeit zählt)
;   gfx_draw_map alevel_map, alevel_tiles, alevel_width, alevel_height
;   ...
;   gfx_runtime                                      ; EINMAL, hinter dem Hauptprogramm (wird nicht ausgeführt)
;   gfx_dlist                                        ; EINMAL: Display-Liste (32 Bytes, automatisch ausgerichtet)
;
;   Makros im Überblick (alle verändern A, X, Y und die Nullseite $CB-$D0; keines rettet Register):
;     gfx_copy src, dst, len          Bereich kopieren (Bereiche dürfen sich nicht überlappen), ruft gfx_memcpy
;     gfx_load_charset src, dst, len  = gfx_copy (len kommt aus dem Export als <name>_len)
;     gfx_load_screen chars           = gfx_copy chars -> gfx_screen, 960 Bytes
;     gfx_clear char                  Bildschirmspeicher mit einem Zeichencode füllen (Anmerkung beim Makro beachten)
;     gfx_set_display                 OS-Schatten für Display-Liste (SDLSTL/H) und Zeichensatzseite (CHBAS) setzen
;     gfx_colors bg, fg               OS-Farbschatten COLOR2/COLOR4 (Hintergrund, Rahmen) und COLOR1 (Schrift) setzen
;     gfx_dlist                       legt die Display-Liste an (Makro mit Daten, nicht ausführbar)
;     gfx_draw_map map, tiles, w, h   Kachelkarte (2×2-Kacheln) auf den Bildschirm zeichnen
;     gfx_runtime                     Unterprogramme gfx_memcpy, gfx_map_draw und die Variablen (nicht ausführbar)
;
; SPEICHERPLAN (vor dem Include mit GFX_SCREEN / GFX_CHARSET änderbar)
;   Bildschirm  $4000 (960 Bytes; darf keine 4-KB-Grenze kreuzen, das Low-Byte muss 0 sein: gfx_map_draw prüft es mit .assert)
;   Zeichensatz $4400 (1 KB, an einer 1-KB-Grenze ausgerichtet, damit CHBAS = Seitennummer (High-Byte) passt)
;   Display-Liste: im Programm, 32 Bytes, per `.align 32` ausgerichtet (darf keine 1-KB-Grenze kreuzen)
;   Die Daten von Zeichensatz und Bildschirm liegen im Programmabbild (Export) und werden zur Laufzeit an diese
;   Adressen kopiert; die Zieladressen selbst sind NICHT Teil der .xex-Datei.
;   Nullseite: $CB-$CC Quelle (gfx_src), $CD-$CE Ziel (gfx_dst), $CF-$D0 Länge (gfx_len) - jeweils 16 Bit, Low-Byte zuerst.
;   OS-Schattenregister: SDLSTL $0230/$0231, CHBAS $02F4, COLOR0..COLOR4 $02C4..$02C8 (benutzt: COLOR1, COLOR2, COLOR4).
;
; KONVENTIONEN UND GRENZEN
;   * Bildschirmcode 0 ist das Leerzeichen; Bit 7 eines Zeichencodes invertiert das Zeichen (Zeichen 128-255 des 128er-Zeichensatzes).
;   * Kartengröße: Breite × Höhe höchstens 255 Zellen (der Kartenindex gfx_mi ist ein Byte; Standard 20×12 = 240).  Jede Kachel
;     ist 2×2 Zeichen (4 Bytes im Kachelsatz: links oben, rechts oben, links unten, rechts unten); die Karte füllt also
;     bis zu 40×24 Zeichen ab Bildschirmposition 0/0.  Breite und Höhe dürfen nicht 0 sein.
;   * gfx_draw_map schreibt Operanden in den Code von gfx_map_draw (selbstmodifizierend): gfx_runtime muss im RAM liegen.
;   * Die Makros sind nicht für Interrupts gedacht (gemeinsame Nullseite, kein Register wird gerettet).

.ifdef GFX_SCREEN
.const gfx_screen = GFX_SCREEN
.else
.const gfx_screen = $4000
.endif
.ifdef GFX_CHARSET
.const gfx_charset = GFX_CHARSET
.else
.const gfx_charset = $4400
.endif
.const gfx_sdlstl = $0230       ; OS-Schatten: Display-Liste (Low/High)
.const gfx_chbas  = $02f4       ; OS-Schatten: Zeichensatz-Seite
.const gfx_color0 = $02c4       ; OS-Schatten COLPF0..COLPF3, COLBK = gfx_color4
.const gfx_color1 = $02c5       ; Schatten von COLPF1 (Schriftfarbe, im Modus 2 zählt nur die Helligkeit)
.const gfx_color2 = $02c6       ; Schatten von COLPF2 (Hintergrund des Textfeldes)
.const gfx_color4 = $02c8       ; Schatten von COLBK (Rahmen)
.const gfx_src  = $cb           ; 16 Bit: Quelle
.const gfx_dst  = $cd           ; 16 Bit: Ziel
.const gfx_len  = $cf           ; 16 Bit: Länge

; gfx_copy(src, dst, len): lädt Quelle, Ziel und Länge in die Nullseite und ruft gfx_memcpy.
; Ein: src, dst = Adressen, len = Byteanzahl (Konstanten oder Labels).  Aus: A, X, Y verändert, gfx_src/dst/len zerstört.
.macro gfx_copy(src, dst, len)      ; kopiert len Bytes von src nach dst (Bereiche dürfen nicht überlappen)
        lda #<src
        sta gfx_src
        lda #>src
        sta gfx_src+1
        lda #<dst
        sta gfx_dst
        lda #>dst
        sta gfx_dst+1
        lda #<len
        sta gfx_len
        lda #>len
        sta gfx_len+1
        jsr gfx_memcpy
.endmacro

.macro gfx_load_charset(src, dst, len)   ; Zeichensatz (len aus dem Export: <name>_len) nach dst
        gfx_copy src, dst, len
.endmacro

.macro gfx_load_screen(chars)       ; 960 Zeichencodes nach gfx_screen
        gfx_copy chars, gfx_screen, 960
.endmacro

; gfx_clear(char): füllt den Bildschirmspeicher mit einem Zeichencode.
; Es werden genau 960 Bytes geschrieben (3 volle Seiten + 192 Bytes), damit nichts hinter dem Bildschirm (z. B. bei geändertem
; GFX_CHARSET) überschrieben wird.
; Verändert A, X, Y, gfx_dst.
.macro gfx_clear(char)              ; Bildschirm mit einem Zeichencode füllen
        lda #<gfx_screen
        sta gfx_dst
        lda #>gfx_screen
        sta gfx_dst+1
        ldx #3                      ; 3 volle Seiten = 768 Bytes, danach noch 192 Bytes
        ldy #0
        lda #char
@l:     sta (gfx_dst),y
        iny
        bne @l
        inc gfx_dst+1
        dex
        bne @l
@t:     sta (gfx_dst),y             ; Rest: Y läuft von 0 an weiter (nach der letzten Seite ist Y = 0)
        iny
        cpy #192
        bne @t
.endmacro

; gfx_set_display: Schattenregister auf unsere Display-Liste (Label gfx_dl, siehe gfx_dlist) und den Zeichensatz setzen.
; Wirkt erst im nächsten Vertical Blank (das OS übernimmt SDLSTL und CHBAS dann in ANTIC).  Verändert A.
.macro gfx_set_display              ; Display-Liste und Zeichensatz aktivieren (OS-Schattenregister)
        lda #<gfx_dl
        sta gfx_sdlstl
        lda #>gfx_dl
        sta gfx_sdlstl+1
        lda #>gfx_charset           ; CHBAS = Seitennummer (High-Byte) des Zeichensatzes
        sta gfx_chbas
.endmacro

; gfx_colors(bg, fg): bg -> COLOR2 (Textfeld-Hintergrund) und COLOR4 (Rahmen), fg -> COLOR1 (Schrift, nur Helligkeit).
.macro gfx_colors(bg, fg)           ; Hintergrund/Rahmen und Textfarbe (GTIA-Farbwerte, z. B. <name>_bg/_fg des Exports)
        lda #bg
        sta gfx_color2
        sta gfx_color4
        lda #fg
        sta gfx_color1
.endmacro

; gfx_dlist: Display-Liste, 32 Bytes: 3 x $70 (je 8 leere Zeilen = 24 Rasterzeilen Oberrand), $42 (Modus 2 mit LMS: die
; Bildschirmadresse folgt), Adresse gfx_screen, 23 x $02 (Modus 2), $41 (Sprung und Warten auf Bildende) mit Adresse gfx_dl.
; `.align 32`: 32 Bytes ab einer durch 32 teilbaren Adresse kreuzen nie eine 1-KB-Grenze (ANTIC zählt die Display-Listen-
; Adresse nur innerhalb von 1 KB weiter).
.macro gfx_dlist                    ; Display-Liste: 24 Zeilen ANTIC-Modus 2 ab gfx_screen (Label gfx_dl)
        .align 32
gfx_dl: .byte $70, $70, $70, $42
        .word gfx_screen
        .repeat 23
        .byte $02
        .endrepeat
        .byte $41
        .word gfx_dl
.endmacro

; gfx_draw_map(map, tiles, w, h): trägt Kartenadresse, Kachelsatz, Breite und Höhe in die Operanden bzw. Variablen von
; gfx_map_draw ein (selbstmodifizierender Code) und ruft es auf.  map = w*h Kachelnummern (Bytes, zeilenweise),
; tiles = 4 Zeichencodes je Kachel.  Verändert A, X, Y, gfx_src/dst und die gfx_*-Variablen.
.macro gfx_draw_map(map, tiles, w, h)   ; zeichnet die Karte (2×2-Kacheln) ab Bildschirmposition 0/0
        lda #<map
        sta gfx_p_map+1
        lda #>map
        sta gfx_p_map+2
        lda #<tiles
        sta gfx_p_tl+1
        lda #>tiles
        sta gfx_p_th+1
        lda #w
        sta gfx_mw
        lda #h
        sta gfx_mh
        jsr gfx_map_draw
.endmacro

; gfx_runtime: die Unterprogramme und Variablen.  Einmal hinter dem Hauptprogramm einbinden (wird nie "durchlaufen").
.macro gfx_runtime
; gfx_memcpy: kopiert gfx_len Bytes von (gfx_src) nach (gfx_dst), aufwärts.  Ein: die drei Zeiger in der Nullseite.
; Aus: A, X, Y verändert; gfx_src/gfx_dst zeigen auf die Seite hinter dem zuletzt vollständig kopierten Bereich, gfx_len+1 = 0.
; Länge 0 ist erlaubt.  Arbeitsweise: erst ganze Seiten (Y läuft 0..255), dann der Rest (Low-Byte von gfx_len).
gfx_memcpy:                         ; gfx_src → gfx_dst, gfx_len Bytes
        ldy #0
@page:  lda gfx_len+1
        beq @tail
@l1:    lda (gfx_src),y
        sta (gfx_dst),y
        iny
        bne @l1
        inc gfx_src+1
        inc gfx_dst+1
        dec gfx_len+1
        jmp @page
@tail:  ldx gfx_len
        beq @done
@l2:    lda (gfx_src),y
        sta (gfx_dst),y
        iny
        dex
        bne @l2
@done:  rts

; gfx_map_draw: zeichnet die Karte.  Ein: gfx_mw, gfx_mh (von gfx_draw_map gesetzt) und die Operanden bei gfx_p_map,
; gfx_p_tl, gfx_p_th.  Je Kachel (Spalte gfx_c, Zeile gfx_r): Kachelnummer aus der Karte holen, mal 4 nehmen (Offset im
; Kachelsatz), die Kachelsatzadresse addieren, dann die vier Zeichen auf (2c, 2r), (2c+1, 2r), (2c, 2r+1), (2c+1, 2r+1)
; schreiben.  Der Bildschirmoffset der Zeile 2r kommt aus den Tabellen gfx_off_lo/hi (Zeile * 40).  Aus: A, X, Y verändert.
gfx_map_draw:                       ; Zeiger per selbstmodifizierende Operanden (von gfx_draw_map gesetzt)
        .assert (gfx_screen & 255) == 0
        lda #0
        sta gfx_r
        sta gfx_mi
@row:   lda gfx_r
        asl a                       ; Tabellenindex = 2 * Kachelzeile = Bildzeile
        tax
        lda gfx_off_lo,x            ; Offset der Bildzeile (Zeile * 40)
        sta gfx_o
        lda gfx_off_hi,x
        sta gfx_o+1
        lda #0
        sta gfx_c
@col:   ldx gfx_mi
.const gfx_p_map = *                ; Adresse des folgenden LDA: sein Operand (+1/+2) wird von gfx_draw_map auf die Karte gesetzt
        lda $ffff,x
        sta gfx_t
        inc gfx_mi
        lda gfx_t
        asl a                       ; Kachelnummer * 4 als 16-Bit-Wert in gfx_src
        sta gfx_src
        lda #0
        rol a
        sta gfx_src+1
        asl gfx_src
        rol gfx_src+1
        lda gfx_src
        clc
.const gfx_p_tl = *                 ; Adresse des folgenden ADC: Operand (+1) = Low-Byte des Kachelsatzes (selbstmodifizierend)
        adc #0
        sta gfx_src
        lda gfx_src+1
.const gfx_p_th = *                 ; Adresse des folgenden ADC: Operand (+1) = High-Byte des Kachelsatzes
        adc #0
        sta gfx_src+1
        lda gfx_c                   ; Ziel = gfx_screen + Zeilenoffset + 2 * Spalte
        asl a
        clc
        adc gfx_o
        sta gfx_dst
        lda gfx_o+1
        adc #>gfx_screen
        sta gfx_dst+1
        ldy #0
        lda (gfx_src),y             ; links oben
        sta (gfx_dst),y
        ldy #1
        lda (gfx_src),y             ; rechts oben
        sta (gfx_dst),y
        ldy #2
        lda (gfx_src),y             ; links unten (eine Bildzeile = 40 Zeichen tiefer)
        ldy #40
        sta (gfx_dst),y
        ldy #3
        lda (gfx_src),y             ; rechts unten
        ldy #41
        sta (gfx_dst),y
        inc gfx_c
        lda gfx_c
        cmp gfx_mw
        beq @rowend
        jmp @col
@rowend:
        inc gfx_r
        lda gfx_r
        cmp gfx_mh
        beq @end
        jmp @row
@end:   rts

gfx_r:  .byte 0                     ; aktuelle Kachelzeile
gfx_c:  .byte 0                     ; aktuelle Kachelspalte
gfx_mi: .byte 0                     ; laufender Index in die Karte (8 Bit -> höchstens 255 Kacheln)
gfx_t:  .byte 0                     ; aktuelle Kachelnummer
gfx_mw: .byte 0                     ; Kartenbreite in Kacheln (von gfx_draw_map gesetzt)
gfx_mh: .byte 0                     ; Kartenhöhe in Kacheln
gfx_o:  .word 0                     ; Bildschirmoffset der aktuellen Bildzeile (Zeile * 40)
gfx_off_lo:                         ; Tabelle Zeile * 40 für 24 Bildzeilen, Low-Bytes
        .repeat 24, i
        .byte <(i * 40)
        .endrepeat
gfx_off_hi:                         ; dasselbe, High-Bytes
        .repeat 24, i
        .byte >(i * 40)
        .endrepeat
.endmacro
