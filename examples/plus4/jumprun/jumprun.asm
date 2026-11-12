; =====================================================================================================================
; jumprun.asm - "Hugo & Gumbo": selbstlaufende Jump-and-Run-Demo fuer den Commodore Plus/4 (Hauptprogramm)
; =====================================================================================================================
;
; ABLAUF (endlos):   Titelbild (8 Sekunden) -> Bild 1 Wiese -> Bild 2 Hoehle -> Bild 3 Burg -> Bild 4 Wolken
;                    -> Finale (Fanfare) -> wieder das Titelbild.   Keine Steuerung: der Autopilot (core.asm) fuehrt Hugo.
;   Der Gegner Gumbo laeuft Hugo nach (ebenfalls in core.asm).  Musik und Klangeffekte laufen die ganze Zeit im Hintergrund, aus dem
;   Rasterinterrupt der HAL; das Hauptprogramm selbst rechnet nur ein Bild nach dem anderen (Takt: 50 Bilder je Sekunde, PAL).
;
; AUFBAU DER QUELLEN
;   jumprun.asm              dieses Hauptprogramm (Ablaufsteuerung, Titelbild)
;   hal_plus4.asm            Hardware-Schicht: TED (Bild, Zeichen-Sprites, Klang), Rasterinterrupt
;   ../../common/jumprun/    core.asm = Spiellogik, fuer alle Systeme gleich
;   lib/levels.inc           Tabellen, Figurenbilder, Musik (ERZEUGT von tools/gen_jumprun_plus4.py)
;   graphics/*               Projektdateien der Editoren: Zeichensatz chars.vcchar, Kachelkarten lv1..lv4.vctiles, Titelbild title.vcscreen
;                            (beim Bauen nach build/export/*.asm exportiert und unten eingebunden)
;
; SPEICHERKARTE
;   $1001-$3FFF  Programm (BASIC-Starter + Code)   $4000-$47FF  Zeichensatz   $4800-...  Tabellen, Figurenbilder, Musik
;   $6800-...    Titelbild und Kachelkarten        $9000 Bildkarte (LMAP)   $9400 Hintergrundkopie   $0800 Farb-RAM   $0C00 Bildschirm
;   Nullseite: $80-$B3 Spiellogik (ZP), $C0-$E3 HAL.   Der Plus/4 startet BASIC-Programme bei $1001 (BASIC-Speicher ohne Erweiterung);
;   unterhalb von $4000 ist deshalb Platz fuer ungefaehr 12 KB Code, der Zeichensatz muss auf eine 1-KB-Grenze ($4000 = 16 * 1024).
;   Der TED sieht den RAM nur als Ganzes: Bild, Zeichensatz und Farb-RAM liegen im selben RAM wie Code und Daten; es gibt keine
;   "Bank" wie beim C64.  Die Tabellen ab $4800 und die Bilddaten ab $6800 sind nur gewoehnliche Daten.
;
; FARBEN: TED-Farbwert = Helligkeit * 16 + Farbton.  Im Farb-RAM kommt Bit 3 (+8) fuer "Mehrfarbe" dazu (Farbton dann nur 0..7).
;   Alle Farbwerte der Tabellen in lib/levels.inc entstehen im Generator tools/gen_jumprun_plus4.py nach dieser Formel.
; =====================================================================================================================
.target plus4                           ; Zielsystem: Commodore Plus/4 (BASIC-Start bei $1001)
.cpu 7501                               ; CPU 7501 (wie 6502, im TED-Rechner; hier ohne Besonderheiten)
.branchlong auto                        ; zu weite bedingte Spruenge verlaengert der Assembler selbst (Warnung W203)

.const ZP   = $80                       ; Nullseiten-Variablen der Spiellogik beginnen bei $80 (core.asm legt sie ab hier an, bis $B3)
.const LMAP = $9000                     ; Bildkarte im RAM (240 Byte = 20 x 12 Kacheln); Muenzen werden daraus geloescht

.org $1001                              ; BASIC-Startadresse des Plus/4
.basicstub start                        ; erzeugt die BASIC-Zeile "10 SYS <Adresse von start>", damit RUN das Programm startet

; Programmstart (per BASIC-Zeile "SYS"): Hardware einrichten (das Betriebssystem wird abgeschaltet), Interrupts freigeben, Hauptablauf.
;   Ab dem cli laufen Musik und Himmelsbalken (Rasterinterrupt) von selbst; hal_init laesst die Interrupts bewusst gesperrt,
;   bis alles eingerichtet ist.  Es gibt keine Rueckkehr nach BASIC: das ROM ist abgeschaltet.
start:  jsr hal_init
        cli
        jmp main

; ---------------------------------------------------------------- Hauptablauf
; cur = Nummer des laufenden Bildes (0..3)
cur:    .byte 0

; main: Punktestand auf 0, Titelbild, vier Bilder nacheinander, Finale, von vorn.
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
        lda #7                          ; Finale: Fanfare (Klangeffekt 7, 60 Bilder)
        jsr hal_sfx
        ldx #100                        ; 2 Sekunden stehen lassen
        jsr wait_x
        jsr hal_blank                   ; Bild aus, kurze Pause (0,4 s), dann beginnt alles von vorn mit dem Titelbild
        ldx #20
        jsr wait_x
        jmp main

; wait_x: wartet X Bilder (X * 1/50 Sekunden, PAL).  X = 0 ergaebe 256 Bilder.  X wird um den Aufruf herum gerettet, weil die
;   HAL-Schnittstelle nicht zusichert, dass hal_wait_frame X erhaelt (diese Fassung aendert nur A).  Aendert A, X (= 0) und Flags.
wait_x: txa
        pha
        jsr hal_wait_frame
        pla
        tax
        dex
        bne wait_x
        rts

; ---------------------------------------------------------------- Titelbild (8 Sekunden = 400 Bilder)
; Farben des ersten Bildes, Hinweistext (blinkt: abwechselnd Text und Leerzeichen), beide Figuren wippen.
;   Das Bild wird bei ausgeschaltetem Bild aufgebaut (hal_blank), damit man den Neuaufbau nicht sieht.
;   Das Blinken geschieht hier durch Umschreiben von Bildschirm UND Hintergrundkopie: die Kopie muss mitgehen, damit die
;   Zeichen-Sprites (die den Hintergrund daraus lesen) nicht den alten Text wieder auf den Bildschirm zurueckschreiben.
;   Dieses Unterprogramm greift DIREKT auf Interna der HAL zu (SCREEN, BGS, skyc, TED-Farbregister ueber .colors) und setzt die
;   Zustandsvariablen des Kerns von Hand (core_level_init laeuft hier nicht); es ist daher nicht systemneutral.  Aendert alle Register.
show_title:
        jsr hal_blank
        lda #0
        sta lvl                         ; das Titelbild benutzt Farben und Himmel von Bild 1
        jsr .colors
        jsr hal_title                   ; 24 Zeilen Titelbild nach Bildschirm, Hintergrundkopie und Farb-RAM; Figurenzustand vergessen
        lda #0                          ; Held links, Gegner rechts, beide stehen
        sta hx_h
        sta hx_f
        sta hdir
        sta hmode
        sta hatim
        sta ex_f
        sta eatim
        sta emode
        lda #24                         ; Held: x = 24, Fuss-y = 160 (Koordinaten der Spiellogik, ein Pixel = 2 Bildpunkte)
        sta hx_l
        lda #160
        sta hy
        sta ey
        lda #<272                       ; Gegner: x = 272 (16 Bit), blickt nach links (edir = 1)
        sta ex_l
        lda #>272
        sta ex_h
        lda #1
        sta edir
        jsr core_prepare_sprites        ; Spiellogik: Bildnummern und Lage der Figuren fuer die HAL bereitstellen (h_px_l usw.)
        jsr hal_update_sprites          ; Zeichen-Sprites einmal zusammensetzen (noch bei ausgeschaltetem Bild)
        jsr hal_unblank
        lda #<400                       ; 400 Bilder = 8 Sekunden
        sta tmr_l
        lda #>400
        sta tmr_h
.tf:    jsr hal_wait_frame
        lda tmr_l                       ; Blinken: Bit 5 des Zaehlers waehlt Text oder Leerzeichen
        and #$20                        ; 32 Bilder an, 32 Bilder aus
        beq .blank
        ldx #20                         ; Text "DEMO - NICHT SPIELBAR" (21 Zeichen) in Zeile 18, ab Spalte 9
-       lda txt_prompt,x
        sta SCREEN+18*40+9,x
        sta BGS+18*40+9,x
        dex
        bpl -
        jmp .wip
.blank: ldx #20                         ; dieselbe Stelle mit Leerzeichen fuellen
        lda #CH_BLANK
-       sta SCREEN+18*40+9,x
        sta BGS+18*40+9,x
        dex
        bpl -
.wip:   lda tmr_l                       ; Figuren wippen leicht (Bit 4 des Zaehlers)
        lsr a
        lsr a
        lsr a
        lsr a
        and #1
        clc
        adc #160                        ; Fuss-y = 160 oder 161: die Figuren hopsen um eine Pixelzeile
        sta hy
        sta ey
        jsr core_prepare_sprites
        jsr hal_update_sprites          ; setzt nur dann neu zusammen, wenn sich y geaendert hat (hal_update_sprites vergleicht)
        lda tmr_l                       ; 16-Bit-Zaehler herunterzaehlen: bei tmr_l = 0 zuerst den Uebertrag nach tmr_h
        bne +
        dec tmr_h
+       dec tmr_l
        lda tmr_l
        ora tmr_h
        bne .tf
        rts
; .colors: Farbregister und Himmelsbalken des Bildes lvl setzen (das Titelbild benutzt die Farben des ersten Bildes)
;   Farbe 1 / Farbe 2 gehen in die TED-Register ($FF16/$FF17) und gelten fuer das ganze Bild; die sechs Balkenfarben werden in
;   skyc abgelegt, aus denen der Rasterinterrupt der HAL sie nach $FF15 (Hintergrund) schreibt.
.colors:                                ; Eingang lvl (Bildnummer); aendert A, X
        ldx lvl
        lda lvl_mc1,x
        sta TED_MC1
        lda lvl_mc2,x
        sta TED_MC2
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
        rts

; ---------------------------------------------------------------- Ein Bild abspielen: A = 0..3
; play_level: baut das Bild bei ausgeschaltetem Bildschirm auf, schaltet es ein und fuehrt dann je Bild (1/50 s) einen Schritt von
;   Held (Autopilot) und Gegner aus, bis die Spiellogik lvl_done meldet (Hugo hat die Tuer erreicht).
;   Eingang: A = Bildnummer 0..3 (zugleich in cur).  Aendert alle Register.
play_level:
        pha
        jsr hal_blank
        pla
        jsr core_level_init             ; Spiellogik: Startpositionen, Skript des Bildes (A = Bildnummer)
        lda lvl
        jsr hal_load_level              ; Farben, Himmel, Karte (LMAP), Kacheln, Statuszeilen
        jsr core_prepare_sprites
        jsr hal_update_sprites
        ldx #3                          ; 3 Bilder warten, bevor das fertige Bild gezeigt wird
        jsr wait_x
        jsr hal_unblank
.f:     jsr hal_wait_frame              ; auf den Bildtakt des Rasterinterrupts warten
        jsr hero_update                 ; Spiellogik: ein Schritt des Autopiloten
        jsr enemy_update                ; Spiellogik: ein Schritt des Gegners
        jsr core_prepare_sprites        ; Logik-Koordinaten in die Werte der HAL umrechnen
        jsr hal_update_sprites          ; Zeichen-Sprites neu zusammensetzen, falls sich etwas bewegt hat
        lda lvl_done
        beq .f
        lda cur                         ; Tuer erreicht: Jingle (nicht nach dem letzten Bild), kurz warten
        cmp #3
        beq +
        lda #6
        jsr hal_sfx
+       ldx #70                         ; 1,4 Sekunden Schlusspause
        jsr wait_x
        rts

; Quellen einbinden: die HAL (TED) und die gemeinsame Spiellogik.  Beide liegen im Programmbereich unterhalb von $4000
; (Stand dieser Fassung: Programm $1001-$1DD6, Zeichensatz $4000-$47FF, Tabellen $4800-$5EEC, Bilddaten $6800-$7123).
.include "hal_plus4.asm"
.include "../../common/jumprun/core.asm"

; Tabelle der Kachelkarten der vier Bilder (Adressen der exportierten Karten; hal_load_level kopiert eine davon nach LMAP)
maps_lo: .byte <lv1_map, <lv2_map, <lv3_map, <lv4_map
maps_hi: .byte >lv1_map, >lv2_map, >lv3_map, >lv4_map

; ---- Daten an festen Adressen (die IDE exportiert die Dateien aus graphics/ beim Bauen nach build/export/)
.org $4000                              ; Zeichensatz: 256 Zeichen * 8 Byte; der TED holt ihn von hier (Register $FF13 = $42);
.include "build/export/chars.asm"       ;   die Zeichen $E0-$F4 sind der Pool der Zeichen-Sprites und werden zur Laufzeit ueberschrieben
.org $4800                              ; Tabellen und Figurenbilder (vom Generator erzeugt), Musik
.include "lib/levels.inc"
.org $6800                              ; Titelbild (title_chars) und die Kachelkarten lv1..lv4 (lv1_tiles = Kachelliste, lv?_map = Karten)
.include "build/export/title.asm"
.include "build/export/lv1.asm"
.include "build/export/lv2.asm"
.include "build/export/lv3.asm"
.include "build/export/lv4.asm"
