; =====================================================================================================================
; jumprun.asm - "Hugo & Gumbo": selbstlaufende Jump-and-Run-Demo fuer den Atari 8 Bit (Hauptprogramm)
; =====================================================================================================================
;
; ABLAUF (endlos):   Titelbild (8 Sekunden) -> Bild 1 Wiese -> Bild 2 Hoehle -> Bild 3 Burg -> Bild 4 Wolken
;                    -> Finale (Fanfare) -> wieder das Titelbild.   Keine Steuerung: der Autopilot (core.asm) fuehrt den Helden
;                    Hugo, die Spiellogik laesst den Gegner Gumbo hinterherlaufen.  Musik und Klangeffekte laufen die ganze Zeit.
;
; AUFBAU DER QUELLEN
;   jumprun.asm              dieses Hauptprogramm (Ablaufsteuerung, Titelbild)
;   hal_atari.asm            Hardware-Schicht: ANTIC/GTIA (Bild, Spieler/Missiles), POKEY (Klang), Interrupts
;   ../../common/jumprun/    core.asm = Spiellogik, fuer alle Systeme gleich
;   lib/rmt.inc, rmtplayr.asm  Musikspieler (Raster Music Tracker); rmtplayr.asm ist der unveraenderte Original-Player (fremde Bibliothek)
;   lib/levels.inc           Tabellen und Spielerdaten (ERZEUGT von tools/gen_jumprun_atari.py)
;   graphics/*, music/       Projektdateien der Editoren: Zeichensatz chars.vcchar, Titelbild title.vcscreen, Kachelkarten lv1..lv4.vctiles,
;                            Song tune.vcsong (beim Bauen nach build/export/*.asm exportiert und unten eingebunden)
;
; ZEITTAKT: Das Hauptprogramm wartet in jedem Durchgang auf den naechsten Bildruecklauf (hal_wait_frame); so laeuft alles im
;   Bildtakt des Fernsehers, 50 Mal je Sekunde (PAL; bei NTSC 60 Mal, die Demo laeuft dort entsprechend schneller).
;   Alle Zeitangaben in diesem Programm ("400 Bilder = 8 Sekunden") gelten fuer PAL.
;
; SPEICHERKARTE
;   $2000-$3FFF  Programm      $4000 Bildschirm   $4400 Zeichensatz   $4800 Display-Liste   $5000-$57FF Spielerspeicher
;   $5800 Bildkarte (LMAP)     $5CE0-$5FFF RMT-Tabellen   $6000 RMT-Player   $7000 Tabellen, Bilddaten, Musikmodul
;   Nullseite: $80-$B3 Spiellogik, $B4-$BB HAL, $CB-$DD RMT-Player
;   (Stand dieser Fassung: Programm $2000-$31B8, Zeichensatz $4400-$47FF, RMT-Player $6000-$6610, Daten $7000-$7ACA.  Die Display-Liste
;   steht nicht in der Datei, sondern wird beim Start aus dlist_data nach $4800 kopiert; Bildschirm und Spielerspeicher sind unbelegt,
;   sie werden zur Laufzeit gefuellt.)
; =====================================================================================================================
.target atari8                          ; Zielsystem: Atari 8 Bit; das Ergebnis ist eine ausfuehrbare Datei im XEX-Format (Segmente + Startadresse)
.cpu sally { illegals = deny }          ; CPU "Sally" (6502C im Atari); undokumentierte Befehle sind verboten
.branchlong auto                        ; zu weite bedingte Spruenge verlaengert der Assembler selbst (Warnung W203)

.const ZP   = $80                       ; Nullseiten-Variablen der Spiellogik (belegt $80-$B3)
.const NO_SEI_BCD = 1                   ; core.asm: do_coin sperrt hier keine IRQs (die Interrupts sind NMIs, siehe do_coin)
.const LMAP = $5800                     ; Bildkarte im RAM (240 Byte = 20 x 12 Kacheln), Muenzen werden daraus geloescht
.const RMT_PLAYER = $6000               ; Spielroutine der Musik (Tabellen davor)

.org $2000                              ; Programmanfang: ueber dem Bereich, den DOS und OS fuer sich brauchen
.include "lib/rmt.inc"                  ; Makros fuer den Musikspieler (rmt_start ...)

; Programmstart (die XEX-Datei startet hier, siehe .run am Ende): Display-Liste an ihren festen Platz kopieren, Hardware einrichten,
; Musik starten.  Anders als bei den anderen Systemen gibt es kein sei/cli: das Betriebssystem laeuft weiter, hal_init haengt sich ein.
;   Die Schleife kopiert dlist_data (Laenge dlist_end - dlist_data = 3 + 3 + 23 + 3 = 32 Byte) rueckwaerts von X = 31 bis 0; bpl endet bei
;   X = $FF.  Danach richtet hal_init die Chips ein, rmt_start uebergibt das Musikmodul an den Spieler (X/Y = Adresse, A = Startzeile).
;   Von nun an ruft der Bildruecklauf-Interrupt der HAL die Musik in jedem Bild auf, hier ist nichts weiter zu tun.
start:  ldx #dlist_end - dlist_data - 1
-       lda dlist_data,x
        sta DLIST,x
        dex
        bpl -
        jsr hal_init
        rmt_start tune_rmt, 0           ; Musik ab Zeile 0 (die HAL ruft den Spieler in jedem Bild auf)
        jmp main

cur:    .byte 0                         ; Nummer des laufenden Bildes (0..3); main zaehlt sie hoch, nach dem Bild 3 folgt das Finale

; main: Punktestand auf 0, Titelbild, vier Bilder nacheinander, Finale, von vorn.
;   Der Punktestand ist dreistellig BCD (sc0 niedrig, sc2 hoch).  Nach dem vierten Bild (cur = 4) folgt das Finale: Fanfare (Effekt 7),
;   2 Sekunden (100 Bilder) stehenbleiben, Bildschirm aus, noch 20 Bilder warten, dann von vorn mit dem Titelbild.
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
        lda #7                          ; Finale: Fanfare
        jsr hal_sfx
        ldx #100
        jsr wait_x
        jsr hal_blank
        ldx #20
        jsr wait_x
        jmp main

; wait_x: wartet X Bilder; X wird um den Bildaufruf herum gerettet.
;   X dient als Zaehler; damit ein Register-Verbrauch der aufgerufenen Routine ihn nicht zerstoert, liegt er waehrend des
;   Aufrufs auf dem Stapel (pha / pla).  X = 0 wuerde 256 Bilder bedeuten (dex wickelt von 0 auf 255 um).
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
;   Ablauf: Bildschirm aus; Farben von Bild 0 (Wiese) einstellen (dieselben Register wie hal_load_level, aber ohne Karte und
;   Statuszeile); Titelzeichen in den Bildschirm kopieren; Spielzustand von Held und Gegner so setzen, dass beide stehen (hmode, emode = 0);
;   Sprites zeichnen; Bildschirm an; 400 Bilder lang blinken und wippen lassen.
;   Anders als auf dem C64 gibt es kein Farb-RAM: das Blinken des Hinweises geschieht durch Austauschen der Zeichen (Text <-> Leerzeichen).
;   Dieses Unterprogramm greift DIREKT auf die Schattenregister COLOR0.., die Himmelstabelle skyc und den Bildschirm SCREEN der HAL zu
;   und setzt die Zustandsvariablen des Kerns von Hand (core_level_init laeuft hier nicht).  Aendert alle Register.
show_title:
        jsr hal_blank
        lda #0
        sta lvl
        tax                             ; X = 0 = Bildnummer fuer die Farbtabellen
        lda lvl_pf0,x
        sta COLOR0                      ; COLPF0 (Schattenregister)
        lda lvl_pf1,x
        sta COLOR0+1                    ; COLPF1
        lda lvl_pf2,x
        sta COLOR0+2                    ; COLPF2
        lda lvl_sky0,x
        sta skyc                        ; Balken 0 (hal_unblank setzt ihn als COLOR4/COLBK; bis dahin bleibt der Hintergrund schwarz)
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
        lda #0                          ; Held links, Gegner rechts, beide stehen
        sta hx_h
        sta hx_f
        sta hdir
        sta hmode
        sta hatim
        sta ex_f
        sta eatim
        sta emode
        lda #24
        sta hx_l                        ; Held x = 24 (logische Punkte, linke Kante)
        lda #160
        sta hy                          ; Fuss-y = 160 = Oberkante von Kachelzeile 10 (die Grasreihe des Titelbildes)
        sta ey
        lda #<272
        sta ex_l                        ; Gegner x = 272 (24 Punkte breit: rechte Kante bei 296)
        lda #>272
        sta ex_h
        lda #1
        sta edir                        ; Gegner blickt nach links, zu Hugo
        jsr core_prepare_sprites
        jsr hal_update_sprites
        jsr hal_unblank
        lda #<400
        sta tmr_l
        lda #>400
        sta tmr_h                       ; 16-Bit-Zaehler: 400 Bilder = 8 Sekunden
; Schleife je Bild: warten, Hinweistext zeigen oder loeschen, Figuren wippen lassen, Zaehler herunterzaehlen.
;   Blinken: Bit 5 des Zaehlers wechselt alle 32 Bilder (0,64 s Text sichtbar, 0,64 s unsichtbar).  Die 21 Zeichen von
;   txt_prompt (oder 21 Leerzeichen) kommen nach Zeile 18, Spalte 9 (zentriert: (40 - 21) / 2 = 9).
.tf:    jsr hal_wait_frame
        lda tmr_l                       ; Blinken: Bit 5 des Zaehlers waehlt Text oder Leerzeichen
        and #$20
        beq +
        ldx #20
-       lda txt_prompt,x
        sta SCREEN+18*40+9,x
        dex
        bpl -
        jmp ++
+       ldx #20
        lda #CH_BLANK
-       sta SCREEN+18*40+9,x
        dex
        bpl -
+       lda tmr_l                       ; Figuren wippen leicht (Bit 4 des Zaehlers)
        lsr a
        lsr a
        lsr a
        lsr a
        and #1                          ; wechselt alle 16 Bilder zwischen 0 und 1
        clc
        adc #160                        ; Fuss-y 160 oder 161: die Figuren heben und senken sich um einen Punkt
        sta hy
        sta ey
        jsr core_prepare_sprites
        jsr hal_update_sprites
        lda tmr_l                       ; 16-Bit-Zaehler herunterzaehlen: bei niedrigem Byte = 0 erst das hohe vermindern (Borgen)
        bne +
        dec tmr_h
+       dec tmr_l
        lda tmr_l
        ora tmr_h                       ; beide Bytes 0? Dann sind die 400 Bilder um.
        bne .tf
        rts

; ---------------------------------------------------------------- Ein Bild abspielen: A = 0..3
; play_level (Eingang A = Bildnummer 0..3, aendert alle Register): Bildschirm aus -> Spiellogik fuer dieses Bild initialisieren ->
;   Bild und Farben laden -> Bildschirm an ->
;   je Bild: auf Bildende warten, Held bewegen, Gegner bewegen, Werte fuer die HAL aufbereiten, Sprites setzen ->
;   wenn der Autopilot "END" erreicht hat (lvl_done), Jingle spielen (ausser nach dem letzten Bild), 70 Bilder warten.
;   Das eine Bild Wartezeit vor hal_unblank ist nur eine kurze Pause im schwarzen Bild (hal_blank hat Bildschirm und Hintergrund
;   schon abgeschaltet); SDMCTL und COLOR4 aus hal_unblank werden im naechsten Bildruecklauf gemeinsam uebernommen.
play_level:
        pha                             ; Bildnummer retten (hal_blank verwendet A)
        jsr hal_blank
        pla
        jsr core_level_init             ; Spiellogik: Positionen, Skript, lvl setzen
        lda lvl
        jsr hal_load_level
        jsr core_prepare_sprites
        jsr hal_update_sprites
        ldx #1
        jsr wait_x
        jsr hal_unblank
.f:     jsr hal_wait_frame
        jsr hero_update
        jsr enemy_update
        jsr core_prepare_sprites
        jsr hal_update_sprites
        lda lvl_done
        beq .f
        lda cur                         ; Tuer erreicht: Jingle (nicht nach dem letzten Bild), kurz warten
        cmp #3
        beq +
        lda #6
        jsr hal_sfx
+       ldx #70
        jsr wait_x
        rts

; ---------------------------------------------------------------- Einbinden der weiteren Quellen und Daten
; Reihenfolge im Speicher: HAL, Spiellogik und Tabellen folgen direkt auf das Hauptprogramm (unter $4000);
; der RMT-Player und die Editor-Exporte werden mit .org an feste Adressen gelegt (siehe Speicherkarte oben).
.include "hal_atari.asm"
.include "../../common/jumprun/core.asm"
.include "lib/levels.inc"

maps_lo: .byte <lv1_map, <lv2_map, <lv3_map, <lv4_map      ; Kachelkarte je Bild (240 Byte, aus build/export/lv?.asm)
maps_hi: .byte >lv1_map, >lv2_map, >lv3_map, >lv4_map

.org RMT_PLAYER
.include "lib/rmtplayr.asm"             ; Original-Spieler; seine Tabellen liegen unmittelbar vor $6000 ($5CE0-$5FFF)

.org $4400                              ; Zeichensatz: 128 Zeichen * 8 Byte = 1 KB, auf einer 1-KB-Grenze (Bedingung von ANTIC)
.include "build/export/chars.asm"
.org $7000                              ; Titelbild, Karten und Musikmodul (title_chars, lv1_map..lv4_map, lv1_tiles, tune_rmt)
.include "build/export/title.asm"
.include "build/export/lv1.asm"
.include "build/export/lv2.asm"
.include "build/export/lv3.asm"
.include "build/export/lv4.asm"
.include "build/export/tune.asm"
.run start                              ; Startadresse der XEX-Datei (RUNAD $02E0): das Programm beginnt bei start
