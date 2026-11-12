; =====================================================================================================================
; jumprun.asm - "Hugo & Gumbo": selbstlaufende Jump-and-Run-Demo fuer den C64 (Hauptprogramm)
; =====================================================================================================================
;
; ABLAUF (endlos):   Titelbild (8 Sekunden)  ->  Bild 1 Wiese -> Bild 2 Hoehle -> Bild 3 Burg -> Bild 4 Wolken
;                    ->  Finale (Fanfare)  ->  wieder das Titelbild.
;   Es gibt keine Steuerung: der "Autopilot" (core.asm) fuehrt den Helden Hugo, die Spiellogik laesst den Gegner Gumbo
;   hinterherlaufen.  Musik und Klangeffekte laufen die ganze Zeit.
;
; AUFBAU DER QUELLEN
;   jumprun.asm              dieses Hauptprogramm (Ablaufsteuerung, Titelbild)
;   hal_c64.asm              Hardware-Schicht: VIC-II (Bild, Sprites), SID (Klang), Interrupt
;   ../../common/jumprun/    core.asm = Spiellogik, fuer alle Systeme gleich
;   lib/music.inc            Spielroutine fuer die Tracker-Musik (SID, 3 Stimmen; hier fuer Klangeffekte angepasst)
;   lib/levels.inc           Tabellen: Farben, Startpositionen, Autopilot-Skripte, Texte
;                            (ERZEUGT von tools/gen_jumprun_c64.py aus den Kunstdaten tools/jumprun_art.py)
;   graphics/*, music/       Projektdateien der Editoren: Zeichensatz, Kachelsatz hg, Screen-Bank levels (4 Bilder), Titelbild, Sprites, Song
;                            (beim Bauen exportiert die IDE sie nach build/export/*.asm, die unten eingebunden werden)
;
; SPEICHERKARTE
;   $0801-$1F81  Programm (BASIC-Starter + Code; Stand dieser Fassung bis $16BD)     $1F82-$1FFF  Tabellen des Kachelsatzes (hg_props ...)
;   $2000-$27FF  Zeichensatz (aus chars.vcchar)         $2800-$31FF  Sprites (4 Saetze, siehe unten)
;   $3200-...    Titelbild, Karten der Bilder, Musikdaten, danach die Unterprogramme der Kachel-Bibliothek (tiles_runtime)
;   $C000-$C0EF  Bildkarte im RAM (LMAP)
;   Nullseite:   $10-$43 Spiellogik (core.asm), $50-$56 HAL, $58-$5F Kachel-Bibliothek, $F7-$FA Musik
;   Bildspeicher (Bildschirm $0400, Sprite-Zeiger $07F8, Farb-RAM $D800) und VIC-Bank 0: siehe hal_c64.asm.
; =====================================================================================================================
.target c64                             ; Zielsystem: Commodore 64
.cpu 6510 { illegals = deny }           ; CPU 6510; undokumentierte Befehle sind verboten
.branchlong auto                        ; zu weite bedingte Spruenge verlaengert der Assembler selbst (Warnung W203)

.const ZP   = $10                       ; Nullseiten-Variablen der Spiellogik beginnen bei $10
.const LMAP = $c000              ; Bildkarte im RAM (240 Byte), Muenzen werden daraus geloescht

; Kachel-Bibliothek: zeichnet die Kacheln (tile_put) und die Bilder (screen_select_a, screen_draw_all); die Daten kommen aus dem Export
; von graphics/hg.vctiles (Kachelsatz) und graphics/levels.vcscreen (Screen-Bank), die Bindung steht unten bei tiles_use.
.const TILE_MAPRAM = LMAP               ; screen_select_a kopiert die Karte eines Bildes nach LMAP (Muenzen werden dort geloescht)
.const TILE_ZP = $58                    ; Nullseite $58-$5F fuer die Bibliothek (die Musik nutzt $F7-$FA)
#include <c64/tiles.asm>

.org $0801                              ; BASIC-Startadresse
.basicstub start                        ; erzeugt die BASIC-Zeile "10 SYS <Adresse von start>", damit RUN das Programm startet

.include "lib/music.inc"

; Programmstart (per BASIC-Zeile "SYS"): Hardware einrichten, Musik starten (music_init richtet die Spielroutine auf die
; exportierten Songdaten ein).  Es gibt keine Rueckkehr nach BASIC; das Programm endet nie.
start:  jsr hal_init                    ; VIC, Sprites und Interrupt einrichten (Interrupts bleiben gesperrt)
        music_init tune_order, tune_patterns, tune_ins, tune_speed, tune_rows, tune_patsize
        cli                             ; Interrupts jetzt freigeben: ab hier laufen Musik und Bildtakt
        jmp main                        ; Hauptablauf

; ---------------------------------------------------------------- Hauptablauf
; cur = Nummer des laufenden Bildes (0..3)
cur:    .byte 0


; main: der Hauptablauf.  Punktestand auf 0, Titelbild zeigen, die vier Bilder nacheinander spielen, Finale, von vorn.
main:   lda #0
        sta sc0
        sta sc1
        sta sc2
        jsr show_title
        lda #0
        sta cur
.next:  lda cur
        jsr play_level
        inc cur
        lda cur
        cmp #4
        bcc .next
        ; Finale: Fanfare, dann zurueck zum Titelbild
        lda #7
        jsr hal_sfx
        ldx #100
        jsr wait_x
        jsr hal_blank
        ldx #20
        jsr wait_x
        jmp main

; wait_x: wartet X Bilder (X * 1/50 Sekunden, PAL).  X = 0 ergaebe 256 Bilder.  X wird um den Aufruf herum gerettet, weil die
;   HAL-Schnittstelle nicht zusichert, dass hal_wait_frame X erhaelt (diese Fassung aendert nur A).  Aendert A, X (= 0) und Flags.
wait_x:                          ; X Bilder warten
-       txa
        pha
        jsr hal_wait_frame
        pla
        tax
        dex
        bne -
        rts

; ---------------------------------------------------------------- Titelbild (8 Sekunden = 400 Bilder)
; show_title: zeigt das Titelbild mit den Himmelsfarben und Mehrfarben des ersten Bildes, dem blinkenden Hinweis
;   "DEMO - NICHT SPIELBAR" (Zeile 18; Blinken = abwechselnd Textfarbe und Himmelsfarbe in der Farb-RAM-Zeile) und beiden
;   Figuren, die leicht auf und ab wippen.  Der Zaehler tmr_h:tmr_l zaehlt 400 Bilder herunter.
;   Dieses Unterprogramm greift DIREKT auf VIC-Register ($D022/$D023), die Himmelstabelle skyc und Bildschirm/Farb-RAM der HAL
;   zu (die Farbwahl entspricht dem Anfang von hal_load_level); es ist daher nicht systemneutral.  Aendert alle Register.
show_title:
        jsr hal_blank
        lda #0
        sta lvl
        tax
        lda lvl_mc1,x
        sta $d022
        lda lvl_mc2,x
        sta $d023
        lda lvl_sky0,x
        sta skyc
        lda lvl_sky1,x
        sta skyc+1
        lda lvl_sky2,x
        sta skyc+2
        lda lvl_sky3,x
        sta skyc+3
        lda lvl_sky4,x
        sta skyc+4
        lda lvl_sky5,x
        sta skyc+5
        jsr hal_title
        ; Hinweis "DEMO - NICHT SPIELBAR" (Zeile 18, Spalte 9, 21 Zeichen), blinkt ueber die Farbe
        ldx #0
-       lda txt_prompt,x
        sta SCREEN+18*40+9,x
        inx
        cpx #21
        bne -
        ; Held links, Gegner rechts, beide stehen: die Zustandsvariablen des Kerns werden von Hand gesetzt, core_level_init laeuft hier nicht
        lda #24
        sta hx_l
        lda #0
        sta hx_h
        sta hx_f
        sta hdir
        sta hmode
        sta hatim
        sta ex_f
        sta eatim
        sta emode
        lda #160
        sta hy
        sta ey
        lda #<272
        sta ex_l
        lda #>272
        sta ex_h
        lda #1
        sta edir
        jsr core_prepare_sprites
        jsr hal_update_sprites
        jsr hal_unblank
        lda #<400                ; 400 Bilder
        sta tmr_l
        lda #>400
        sta tmr_h
.tf:    jsr hal_wait_frame
        ; Blinken: Bit 5 des Zaehlers wechselt alle 32 Bilder zwischen gelb (7) und der Himmelsfarbe cyan (3); beide Werte unter 8 = Hires-Text
        lda tmr_l
        and #$20
        beq +
        lda #7
        jmp ++
+       lda #3
+      ldx #0
-       sta COLOR+18*40+9,x
        inx
        cpx #21
        bne -
        ; Figuren wippen leicht: Bit 4 des Zaehlers wechselt alle 16 Bilder, Fuss-y = 160 oder 161
        lda tmr_l
        lsr a
        lsr a
        lsr a
        lsr a
        and #1
        clc
        adc #160
        sta hy
        sta ey
        jsr core_prepare_sprites
        jsr hal_update_sprites
        lda tmr_l                       ; 16-Bit-Zaehler herunterzaehlen: bei niedrigem Byte 0 zuerst das hohe vermindern (Borgen)
        bne +
        dec tmr_h
+       dec tmr_l
        lda tmr_l
        ora tmr_h                       ; beide Bytes 0? Dann sind die 400 Bilder um.
        bne .tf
        rts

; ---------------------------------------------------------------- Ein Bild abspielen: A = 0..3
; play_level: Bildschirm aus -> Spiellogik fuer dieses Bild initialisieren -> Bild und Farben laden -> Bildschirm an ->
;   je Bild: auf Bildende warten, Held bewegen, Gegner bewegen, Werte fuer die HAL aufbereiten, Sprites setzen ->
;   wenn der Autopilot "END" erreicht hat (lvl_done), Jingle spielen (ausser nach dem letzten Bild), 70 Bilder warten.
;   Eingang: A = Bildnummer 0..3 (zugleich in cur).  Aendert alle Register.  Die drei Bilder Wartezeit vor hal_unblank geben dem
;   Interrupt Zeit, die gerade gesetzten Sprites und Farben zu uebernehmen, bevor das fertige Bild erscheint.
play_level:
        pha
        jsr hal_blank
        pla
        jsr core_level_init
        lda lvl
        jsr hal_load_level
        jsr core_prepare_sprites
        jsr hal_update_sprites
        ldx #3
        jsr wait_x
        jsr hal_unblank
        lda #%00001111
        sta $d015                       ; Sprites 0-3 (Held und Gegner, je Overlay + Basis) an; hal_update_sprites setzt dasselbe nochmals
.f:     jsr hal_wait_frame
        jsr hero_update
        jsr enemy_update
        jsr core_prepare_sprites
        jsr hal_update_sprites
        lda lvl_done
        beq .f
        ; Tuer erreicht: kurz warten, Jingle, naechstes Bild
        lda cur
        cmp #3
        beq +
        lda #6
        jsr hal_sfx
+       ldx #70
        jsr wait_x
        rts

; ---------------------------------------------------------------- Hardware-Schicht, Spiellogik, Tabellen und Musik
; Die Reihenfolge ist egal (der Assembler loest Vorwaertsverweise auf); wichtig ist nur, dass der Code vor $1F82 endet, wo die Tabellen
; des Kachelsatzes beginnen (siehe unten, .org $2000 - 126).
.include "hal_c64.asm"
.include "../../common/jumprun/core.asm"
.include "lib/levels.inc"

; Kacheleigenschaften der Spiellogik (core.asm liest tile_attr,x): 0 leer, 1 tragend (hg_p_solid), 2 Leiter (hg_p_ladder) - aus dem Kachelsatz
.const tile_attr = hg_props
.assert hg_p_solid == 1 && hg_p_ladder == 2, "core.asm erwartet 1 = tragend, 2 = Leiter"

music_runtime                          ; Unterprogramme der Musik (music_play ...), einmal hinter dem Hauptprogramm

; ---------------------------------------------------------------- Daten an festen Adressen
; Der VIC sieht nur $0000-$3FFF der Bank, deshalb liegen Zeichensatz und Sprites genau hier. Die .assert-Zeilen pruefen, dass die
; exportierten Spritesaetze lueckenlos aneinanderschliessen (Held 14 Bilder = 896 Byte, Gegner 6 Bilder = 384 Byte).
; Der Zeichensatz ist der Zielzeichensatz hg_chars des Kachelsatzes (alle 256 Zeichen von chars.vcchar sind angepinnt: Kachelzeichen, Vollflaeche,
; Schrift). Der Export legt vor hg_chars die Tabellen des Satzes (6 x 21 Byte); deshalb beginnt die Datei 126 Byte vor $2000.
.org $2000 - 126
.include "build/export/hg.asm"
.assert hg_chars == $2000
.org $2800
.include "build/export/hero_base.asm"
.include "build/export/hero_over.asm"
.include "build/export/enemy_base.asm"
.include "build/export/enemy_over.asm"
.assert hero_over == $2b80
.assert enemy_base == $2f00
.assert enemy_over == $3080
.org $3200
.include "build/export/title.asm"
.include "build/export/levels.asm"
.include "build/export/tune.asm"

; Bindung der Bibliothek an Kachelsatz und Screen-Bank, dann die Unterprogramme (tile_put = hal_put_tile, screen_select_a, screen_draw_all)
        tiles_use hg_c0, hg_col, hg_props, hg_tw, hg_th, hg_count, levels_screens_lo, levels_screens_hi, levels_lv1_tiles_w, levels_lv1_tiles_h
        tiles_runtime
