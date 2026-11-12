; =====================================================================================================================
; jumprun.asm - "Hugo & Gumbo": selbstlaufende Jump-and-Run-Demo fuer den MEGA65 (Hauptprogramm)
; =====================================================================================================================
;
; ABLAUF (endlos):   Titelbild (8 Sekunden)  ->  Bild 1 Wiese -> Bild 2 Hoehle -> Bild 3 Burg -> Bild 4 Wolken
;                    ->  Finale (Fanfare)  ->  wieder das Titelbild.
;   Es gibt keine Steuerung: der "Autopilot" (core.asm) fuehrt den Helden Hugo, die Spiellogik laesst den Gegner Gumbo
;   hinterherlaufen.  Musik und Klangeffekte laufen die ganze Zeit.
;
; AUFBAU DER QUELLEN
;   jumprun.asm              dieses Hauptprogramm (Ablaufsteuerung, Titelbild)
;   hal_mega65.asm           Hardware-Schicht: VIC-II-Modus des VIC-IV (Bild, Sprites), zwei SIDs (Klang), Interrupt
;   ../../common/jumprun/    core.asm = Spiellogik, fuer alle Systeme gleich
;   lib/music.inc            Spielroutine fuer die Tracker-Musik (SID 1 bei $D400, 3 Stimmen; die Klangeffekte laufen am
;                            MEGA65 auf dem zweiten SID, deshalb muss die Musik nie aussetzen)
;   lib/levels.inc           Tabellen: Kacheleigenschaften, Farben, Startpositionen, Autopilot-Skripte, Texte
;                            (ERZEUGT von tools/gen_jumprun_c64.py aus den Kunstdaten tools/jumprun_art.py; die Kunstdaten sind
;                            die des C64-Beispiels, nur Zielsystem und Profile sind MEGA65)
;   graphics/*, music/       Projektdateien der Editoren: Zeichensatz chars.vcchar, Kachelkarten lv1..lv4.vctiles, Titelbild title.vcscreen,
;                            Sprites (hero_*, enemy_*.vcsprite), Song tune.vcsong
;                            (beim Bauen exportiert die IDE sie nach build/export/*.asm, die unten eingebunden werden)
;   Anders als beim C64 gibt es hier keine Kachel-Bibliothek: hal_mega65.asm zeichnet die Kacheln selbst (hal_draw_level) und
;   kennt nur die Kachelliste lv1_tiles/lv1_colors; die Bildkarten lv1..lv4 liegen als 240-Byte-Karten vor (maps_lo/hi unten).
;
; DIESE VERSION ist "Stufe 1" fuer den MEGA65: reiner C64-Code (6502-Befehle), der im VIC-II-Modus des neuen Bildchips
;   (VIC-IV) laeuft.  Der MEGA65 startet anders als der C64: BASIC 65 hat den Speicher ueber MAP umgeblendet und den Bildchip
;   anders eingestellt; hal_init (hal_mega65.asm) stellt deshalb zuerst einen schlichten 64-KB-RAM-Zustand mit Ein-/Ausgabe
;   bei $D000 her ($01 = $35) und schaltet den VIC-IV auf 40 Spalten zurueck.  Daher auch die anderen Adressen als am C64:
;   das Programm beginnt bei $2001 (BASIC-65-Startadresse), und die Daten, die der VIC lesen muss, liegen in VIC-Bank 1.
;
; SPEICHERKARTE
;   $2001-$3FFF  Programm (BASIC-Starter + Code; Stand dieser Fassung bis $2F2E)    $6000-$67FF  Zeichensatz (aus chars.vcchar)
;   $6800-$71FF  Sprites (4 Saetze, siehe unten)        $7200-$80D1  Titelbild, Kachelkarten, Musikdaten (Stand dieser Fassung; die Daten
;                                                                    ab $8000 liegen ausserhalb der VIC-Bank, werden aber nur von der CPU gelesen)
;   $9000-$90EF  Bildkarte im RAM (LMAP)                $4400-$47E7  Bildschirm (VIC-Bank 1 = $4000-$7FFF), Sprite-Zeiger $47F8
;   Nullseite: $10-$43 Spiellogik, $50-$56 HAL, $F7-$FA Musik        Farb-RAM: $D800 (Hires-Text: Farbwert < 8)
;   Zwischen $4000 und $5FFF liegt nur der Bildschirm bei $4400; der Rest der Bank 1 bis $5FFF ist unbelegt.
; =====================================================================================================================
.target mega65                          ; Zielsystem: MEGA65 (Chip- und Speicherkarte des MEGA65 fuer den Assembler und die IDE)
.cpu 45gs02 { illegals = deny }         ; CPU 45GS02 (6502-kompatibel; zusaetzlich map, eom, taz u. a.); undokumentierte Befehle sind verboten
.branchlong auto                        ; zu weite bedingte Spruenge verlaengert der Assembler selbst (Warnung W203)

.const ZP   = $10                       ; Nullseiten-Variablen der Spiellogik beginnen bei $10
.const LMAP = $9000              ; Bildkarte im RAM (240 Byte), Muenzen werden daraus geloescht

.org $2001                              ; BASIC-65-Startadresse (am C64 ist es $0801): ein PRG wird hierhin geladen
.basicstub start                        ; erzeugt die BASIC-Zeile "10 SYS <Adresse von start>", damit RUN das Programm startet
                                        ; (der Code laeuft also zunaechst in der BASIC-65-Umgebung; hal_init raeumt sie als Erstes weg)

.include "lib/music.inc"

; Programmstart: Hardware einrichten, Musik starten (music_init richtet die Spielroutine auf die exportierten Songdaten ein).
start:  jsr hal_init                    ; Speicherabbildung (MAP), $01, VIC-IV-Freischaltung, VIC-Bank 1, Sprites, Interrupt-Vektor, SID 2
        music_init tune_order, tune_patterns, tune_ins, tune_speed, tune_rows, tune_patsize
        cli                             ; Interrupts jetzt freigeben: ab hier laufen Musik und Bildtakt (Vektor $FFFE im RAM, siehe hal_init)
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
;   Dieses Unterprogramm greift DIREKT auf VIC-Register ($D022/$D023), die Himmelstabelle skyc und Bildschirm/Farb-RAM der HAL zu
;   (die Farbwahl entspricht dem Anfang von hal_load_level) und setzt die Zustandsvariablen des Kerns von Hand (core_level_init
;   laeuft hier nicht); es ist daher nicht systemneutral.  Aendert alle Register.
show_title:
        jsr hal_blank
        lda #0
        sta lvl
        tax
        lda lvl_mc1,x
        sta $d022                       ; Mehrfarbe 1 des ersten Bildes
        lda lvl_mc2,x
        sta $d023                       ; Mehrfarbe 2 des ersten Bildes
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
        sta SCREEN+18*40+9,x            ; Zeile 18, Spalte 9 (Bildschirm $4400 + 18 * 40 + 9)
        inx
        cpx #21
        bne -
        ; Held links, Gegner rechts, beide stehen
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
        ; Blinken: Bit 5 des Zaehlers (tmr_l and $20) wechselt alle 32 Bilder zwischen gelb (7) und der Himmelsfarbe cyan (3)
        lda tmr_l
        and #$20
        beq +
        lda #7
        jmp ++
+       lda #3
+      ldx #0
-       sta COLOR+18*40+9,x             ; Farb-RAM der Hinweiszeile (Farbwerte 7 / 3, beide unter 8: Hires-Text)
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
        sta $d015                       ; Sprites 0-3 an (Held und Gegner, je Overlay + Basis); hal_update_sprites setzt dasselbe nochmals
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
; Die Reihenfolge ist egal (der Assembler loest Vorwaertsverweise auf); wichtig ist nur, dass der Code vor $4000 (VIC-Bank 1) endet,
; weil dort der Bildschirm ($4400) und die Sprite-Zeiger ($47F8) liegen; derzeit endet er bei $2F2E.
.include "hal_mega65.asm"
.include "../../common/jumprun/core.asm"
.include "lib/levels.inc"

maps_lo: .byte <lv1_map, <lv2_map, <lv3_map, <lv4_map     ; Kachelkarte je Bild (240 Byte, aus build/export/lv?.asm); hal_load_level
maps_hi: .byte >lv1_map, >lv2_map, >lv3_map, >lv4_map     ; kopiert die gewaehlte nach LMAP

music_runtime                           ; Unterprogramme der Musik (music_play ...), einmal hinter dem Hauptprogramm

; ---------------------------------------------------------------- Daten an festen Adressen
; Der VIC sieht nur die 16 KB seiner Bank 1 ($4000-$7FFF), deshalb liegen Zeichensatz ($6000) und Sprites ($6800) genau hier; der Bildschirm
; steht bei $4400 (siehe hal_init, $D018). Die .assert-Zeilen pruefen, dass die
; exportierten Spritesaetze lueckenlos aneinanderschliessen (Held 14 Bilder = 896 Byte, Gegner 6 Bilder = 384 Byte).
.org $6000
.include "build/export/chars.asm"
.org $6800
.include "build/export/hero_base.asm"
.include "build/export/hero_over.asm"
.include "build/export/enemy_base.asm"
.include "build/export/enemy_over.asm"
.assert hero_over == $6b80
.assert enemy_base == $6f00
.assert enemy_over == $7080
.org $7200
.include "build/export/title.asm"
.include "build/export/lv1.asm"
.include "build/export/lv2.asm"
.include "build/export/lv3.asm"
.include "build/export/lv4.asm"
.include "build/export/tune.asm"
