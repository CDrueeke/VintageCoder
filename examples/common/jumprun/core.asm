; =====================================================================================================================
; core.asm - Spiellogik des Beispiels "Hugo & Gumbo"  (6502-Assembler, plattformneutral)
; =====================================================================================================================
;
; WAS DIESE DATEI TUT
;   Sie enthaelt alles, was am Spiel unabhaengig von der Hardware ist:
;     * den AUTOPILOTEN, der den Helden Hugo ohne Spieler durch ein Bild fuehrt (laufen, springen, klettern, warten),
;     * die BEWEGUNG des Helden (Gehen, Sprung als Sinuskurve, Klettern),
;     * die KUENSTLICHE INTELLIGENZ des Gegners Gumbo (verfolgt Hugo, klettert Leitern, kann aber NIE springen),
;     * kleine Hilfen: Kachel an einer Position lesen, Muenzen einsammeln, Punkte zaehlen.
;   Was sie NICHT tut: Sie schreibt nie in Hardware-Register. Alles, was die Hardware braucht, steht in der
;   "Hardware-Schicht" (HAL) des jeweiligen Systems (z. B. hal_c64.asm).
;
; SO WIRD SIE EINGEBUNDEN
;   Der Einbinder (jumprun.asm des jeweiligen Systems) legt vorher zwei Konstanten fest und bindet dann diese Datei ein:
;     ZP    = erste Nullseiten-Adresse fuer die Variablen dieser Datei (sie belegt ZP bis ZP_END-1, also 52 Byte)
;     LMAP  = Adresse von 240 Byte RAM fuer die Bildkarte (20 Spalten x 12 Zeilen, ein Byte je Kachel)
;   Alles Uebrige steht ausserhalb (Tabellen, HAL-Routinen); der Assembler loest die Verweise auf, die Reihenfolge der
;   .include-Zeilen ist deshalb gleichgueltig.  Belegung der Nullseite je System (die Bereiche duerfen sich nicht ueberlappen):
;     C64, MEGA65:  Kern $10-$43   HAL $50-$56   Kachel-Bibliothek (nur C64) $58-$5F   Musik (lib/music.inc) $F7-$FA
;     Atari:        Kern $80-$B3   HAL $B4-$BB   RMT-Player $CB-$DD
;     Plus/4:       Kern $80-$B3   HAL $C0-$E3
;
; SCHNITTSTELLE ZWISCHEN KERN, HAUPTPROGRAMM UND HAL  (gilt gleich fuer C64, Atari, Plus/4 und MEGA65; der Kopf jeder HAL-Datei
; wiederholt den Teil, der sie betrifft)
;
;   A) Was der KERN von der HAL erwartet.  Er ruft diese Routinen auf; danach setzt er KEIN Register und KEIN Flag als erhalten voraus.
;      Die Nullseitenzellen ZP .. ZP_END-1 darf die HAL dagegen nie veraendern (sie hat ihre eigenen, siehe oben).
;        hal_sfx          A = Effektnummer 0..7: Klangeffekt starten, ein laufender wird ersetzt.  Nummern: 0 Absprung, 1 Landung,
;                         2 Schritt, 3 Klettern (Hugo), 4 Muenze, 5 Klettern (Gumbo), 6 Bildwechsel-Jingle, 7 Fanfare (Finale).
;                         Der Kern startet selbst 2, 3, 4 und 5; 0 und 1 kommen aus den Skripten (Befehl SFX), 6 und 7 vom Hauptprogramm.
;        hal_put_tile     X = Spalte 0..19, Y = Zeile 0..11, A = Kachelnummer: eine Kachel (2 x 2 Zeichen) auf den Bildschirm legen.
;                         Der Kern ruft es nur mit A = 0 (leer) auf, um eine eingesammelte Muenze zu loeschen.
;        hal_score_draw   keine Eingabe: Punktestand aus sc2:sc1:sc0 (BCD) in die Statuszeile schreiben.
;        sfx_on           Speicherbyte der HAL (kein Unterprogramm): ungleich 0, solange ein Effekt klingt.  Der Kern liest es nur,
;                         damit Schritt- und Klettergeraeusche keinen laufenden Effekt abwuergen.
;   B) Was der KERN an Daten erwartet (Tabellen aus lib/levels.inc; beim C64 teilweise aus dem Export des Kachelsatzes):
;        tile_attr        Eigenschaft je Kachelnummer (0 leer, 1 tragend, 2 Leiter).  Atari, Plus/4, MEGA65: Tabelle in lib/levels.inc.
;                         C64: Tabelle hg_props aus dem Export von graphics/hg.vctiles (jumprun.asm setzt .const tile_attr = hg_props).
;        lvl_hx/hy/hd     Startposition (x als .word, Fuss-y, Blickrichtung) des Helden je Bild;  lvl_ex/ey/ed dasselbe fuer den Gegner
;        script_lo/hi     Zeiger auf das Autopilot-Skript je Bild
;   C) Was der KERN anbietet (Aufrufer: Hauptprogramm bzw. HAL):
;        core_level_init  A = Bildnummer 0..3: Spielzustand fuer ein Bild setzen
;        hero_update, enemy_update       je ein Spielschritt, einmal je Bild
;        core_prepare_sprites            rechnet die Ausgabewerte h_px_l.. / e_px_l.. fuer die HAL aus
;        get_tile         X = Spalte, Y = Zeile -> A = Kachelnummer aus LMAP (benutzt von hal_draw_level der HALs fuer Atari, Plus/4
;                         und MEGA65; der C64 zeichnet mit der Kachel-Bibliothek c64/tiles.asm)
;      Variablen des Kerns, die ausserhalb gelesen oder geschrieben werden:
;        lvl (Bildnummer 0..3; die HAL liest sie in hal_hud), lvl_done (Hauptprogramm liest), sc0/sc1/sc2 (Punkte, BCD; das
;        Hauptprogramm loescht sie, die HAL liest sie), tmr_l/tmr_h (Zaehler des Titelbilds, nur Hauptprogramm), hx../hy/hdir/hmode/
;        hatim und ex../ey/edir/emode/eatim (das Titelbild setzt sie von Hand), h_px_l/h_px_h/h_py/h_fr und e_px_l/e_px_h/e_py/e_fr
;        (Ausgabe des Kerns an hal_update_sprites).
;   D) Was das HAUPTPROGRAMM (nicht der Kern) von der HAL erwartet:
;        hal_init             Chips einrichten; C64, Plus/4, MEGA65: die Interrupts bleiben gesperrt, das Hauptprogramm gibt sie mit
;                             cli frei.  Atari: kein sei/cli noetig, hal_init gibt die NMIs (DLI, Bildruecklauf) selbst frei.
;        hal_wait_frame       auf das naechste Bild warten (Bildtakt 50 Hz PAL; ein Spielschritt je Bild)
;        hal_blank/hal_unblank   Bild aus / an (hal_blank wartet zuerst auf ein Bildende)
;        hal_title            Titelbild in den Bildschirmspeicher bringen
;        hal_load_level       A = Bildnummer: Farben, Himmel, Karte nach LMAP, Kacheln und Statuszeile
;        hal_update_sprites   Held und Gegner aus h_*/e_* auf den Bildschirm bringen
;      Zusaetzlich greift show_title im Hauptprogramm DIREKT in HAL-Interna: Himmelsfarben-Tabelle skyc, Farbregister des Bildchips,
;      Bildschirmspeicher SCREEN/COLOR (blinkender Hinweis) und beim Plus/4 die Hintergrundkopie BGS.  Das Hauptprogramm ist deshalb
;      je System eigen; wirklich gemeinsam ist nur der Kern.
;
; UNTERSCHIEDE ZWISCHEN DEN SYSTEMEN  (Einzelheiten im Kopf der jeweiligen HAL-Datei)
;   Bild:      C64, MEGA65: VIC-II-Mehrfarbtext 40 x 25 mit Farbe je Zelle im Farb-RAM.  Plus/4: TED-Mehrfarbtext, Farb-RAM-Wert mit
;              Helligkeit.  Atari: ANTIC-Modus 4 mit EINER Farbtabelle je Bild; Bit 7 des Zeichencodes schaltet auf COLPF3 (Gras, Busch).
;   Figuren:   C64, MEGA65: je Figur zwei Sprites (Mehrfarb-Flaechen + Hires-Umriss).  Atari: Held = vier Spieler, Gegner = vier
;              Missiles.  Plus/4: KEINE Sprites; die Figuren werden als "Zeichen-Sprites" aus umdefinierten Zeichen zusammengesetzt.
;   Umrechnung der logischen Punkte: C64, MEGA65: 1 Punkt = 1 Sprite-Bildpunkt (x + 24, y + 30 bzw. + 36).  Atari: 1 Farbtakt = 2 Punkte.
;              Plus/4: 1 Mehrfarbpixel = 2 Punkte, Feinlage ueber vorverschobene Bilder und Zeilenversatz.
;   Himmel:    sechs Hintergrundbalken; C64, Plus/4, MEGA65 per Rasterinterrupt, Atari per Display-List-Interrupt.
;   Klang:     C64: SID, Effekte leihen sich Stimme 3 (die Musik setzt dort aus).  MEGA65: Musik auf SID 1, Effekte auf SID 2.
;              Atari: POKEY, Musik (RMT) auf Kanal 1-3, Effekte auf Kanal 4.  Plus/4: TED, Melodie Stimme 1, Bass Stimme 2, Effekte
;              leihen sich Stimme 2.
;   Bildtakt:  Rasterinterrupt (C64, MEGA65 in Zeile 251; Plus/4 in Zeile 0) bzw. Zaehler RTCLOK des Betriebssystems (Atari).
;   Start:     BASIC-Starter bei $0801 (C64), $1001 (Plus/4, danach ROM aus), $2001 (MEGA65, BASIC-65-Umfeld wird abgeraeumt);
;              Atari: XEX-Datei mit Startadresse.
;   Kacheln:   C64 zeichnet mit der Kachel-Bibliothek (tile_put, screen_draw_all); die drei anderen Systeme haben eine eigene Schleife
;              hal_draw_level und die Kachelliste lv1_tiles (4 Zeichen je Kachel).
;
; KOORDINATEN
;   Alle Positionen sind LOGISCH 320 x 200 Punkte gross, unabhaengig vom System (die HAL rechnet auf die Hardware um).
;     x  = 16 Bit (hx_h:hx_l) plus 8 Bit Nachkommaanteil (hx_f), also Festkomma 8.8: 0,5 Punkt = $80 im Nachkommateil.
;          Gemeint ist immer die LINKE KANTE der Figur; die Figur ist 24 Punkte breit, ihre Mitte liegt bei x + 12.
;     y  = 8 Bit, "Fuss-y": die Zeile, auf der die Fuesse stehen = Oberkante der Kachel, auf der die Figur steht.
;          Eine Kachel ist 16 Punkte hoch; Kachelzeile k beginnt bei y = 16 * k. Zeile 0 ist die Statuszeile.
;   Die Kachelkarte hat 20 Spalten (x / 16) und 12 Zeilen (y / 16).
;
; FESTKOMMA 8.8 ("wie rechnet man mit halben Punkten?")
;   Ein Wert wie 1,5 Punkte je Bild wird als 16-Bit-Zahl $0180 gespeichert: hoeherwertiges Byte = ganze Punkte (1),
;   niederwertiges Byte = Bruchteil in 1/256 ($80 = 0,5). Bei jeder Addition entsteht aus dem Bruchteil ein Uebertrag
;   in die ganzen Punkte; so bewegen sich Figuren gleichmaessig mit Bruchteilen von Punkten je Bild.
;
; ABLAUF PRO BILD (ruft das Hauptprogramm auf):
;   hero_update -> enemy_update -> core_prepare_sprites -> hal_update_sprites
;
; =====================================================================================================================

; Bewegungszustand des Helden (hmode)
.const HM_STAND = 0                     ; steht (auch wartend und nach dem Skriptende)
.const HM_WALK  = 1                     ; geht (hdir bestimmt die Richtung)
.const HM_JUMP  = 2                     ; Sprung (32 Bilder lang)
.const HM_CLIMB = 3                     ; klettert (hclimb bestimmt hoch oder runter)

; ---- Variablen (Nullseite). Die Nullseite ist schneller und kuerzer anzusprechen als normaler Speicher (2 statt 3 Byte,
; ---- 3 statt 4 Takte); deshalb liegen alle oft benutzten Variablen dort.
; ---- Namensregel: h... = Held (Hero), e... = Gegner (Enemy), t0..t3 = Hilfsspeicher fuer Zwischenwerte.
.const st      = ZP + 0      ; reserviert; im Kern und in den Hauptprogrammen wird st nirgends benutzt
.const lvl     = ZP + 1      ; Nummer des laufenden Bildes 0..3 (core_level_init setzt sie; show_title setzt 0 fuer das Titelbild)
.const tmr_l   = ZP + 2      ; 16-Bit-Bildzaehler (tmr_h:tmr_l); der Kern benutzt ihn nicht, das Titelbild des Hauptprogramms
.const tmr_h   = ZP + 3      ; zaehlt damit 400 Bilder herunter
; Hilfsspeicher: Jede Routine darf t0..t3 beliebig ueberschreiben.  Ihr Inhalt gilt nur INNERHALB einer Routine und ueber Aufrufe
; von Hilfsroutinen hinweg, die ihn nachweislich nicht anfassen; ap_fetch gibt Muenzenspalte/-zeile in t0/t1 an do_coin weiter.
.const t0      = ZP + 4
.const t1      = ZP + 5
.const t2      = ZP + 6
.const t3      = ZP + 7
; Autopilot: ap_ptr zeigt auf das naechste Skriptbyte, ap_cnt zaehlt die Bilder, die der laufende Befehl noch dauert.
.const ap_ptr  = ZP + 8      ; 2 Byte
.const ap_cnt  = ZP + 10     ; Bilder, die der laufende Befehl noch dauert; 0 = naechsten Befehl holen; 255 = Skript zu Ende
.const lvl_done= ZP + 11     ; 0 = Bild laeuft, 1 = Autopilot hat END erreicht (das Hauptprogramm wartet darauf)
; Held: Position (Festkomma 8.8 in x), Bewegungszustand und Geschwindigkeiten.
;   hmode: 0 stehen, 1 gehen, 2 springen, 3 klettern.   hdir: 0 = blickt/laeuft nach rechts, 1 = nach links.
;   hvx/hvy: Sprunggeschwindigkeit pro Bild in 8.8 (vorzeichenbehaftet).   jt: Schritt im Sprung (0..32).
.const hx_f    = ZP + 12     ; Held x = hx_h:hx_l . hx_f
.const hx_l    = ZP + 13
.const hx_h    = ZP + 14
.const hy      = ZP + 15     ; Fuss-y (beim Sprung: Grundlinie)
.const hy_f    = ZP + 16     ; Nachkommaanteil von hy (1/256 Punkt); nur beim Sprung von Bedeutung
.const hdir    = ZP + 17     ; 0 rechts, 1 links
.const hmode   = ZP + 18
.const hatim   = ZP + 19     ; Animationszaehler des Helden (je Geh-/Kletterbild + 1; core_prepare_sprites teilt ihn)
.const hvx_l   = ZP + 20     ; Sprunggeschwindigkeit x je Bild, 8.8 mit Vorzeichen (niedriges / hohes Byte)
.const hvx_h   = ZP + 21
.const hvy_l   = ZP + 22     ; Sprunggeschwindigkeit y je Bild, 8.8 mit Vorzeichen
.const hvy_h   = ZP + 23
.const jt      = ZP + 24     ; Sprungschritt 0..32
.const hclimb  = ZP + 25     ; 0 hoch, 1 runter
; Gegner: gleiche Idee wie beim Helden, aber mit eigenem Zustandsautomaten (emode: 0 gehen, 1 klettern, 2 warten).
.const ex_f    = ZP + 26     ; Gegner x
.const ex_l    = ZP + 27
.const ex_h    = ZP + 28
.const ey      = ZP + 29
.const ey_f    = ZP + 30
.const edir    = ZP + 31
.const emode   = ZP + 32     ; 0 gehen, 1 klettern, 2 warten
.const eatim   = ZP + 33     ; Animationszaehler des Gegners
.const eclimb  = ZP + 34     ; Kletterrichtung des Gegners: 0 hoch, 1 runter (wird beim Aufsteigen auf die Leiter aus want kopiert)
.const ewait   = ZP + 35     ; Restbilder im Zustand "warten" (emode 2)
.const ecol    = ZP + 36     ; Spalte der Leiter, auf der der Gegner klettert (von find_ladder gesetzt)
; Punktestand als dreistellige BCD-Zahl (je Byte zwei Dezimalziffern), niederwertigstes Byte zuerst.
.const sc0     = ZP + 37     ; Punkte BCD (niedrig .. hoch)
.const sc1     = ZP + 38
.const sc2     = ZP + 39
.const sfx_t1  = ZP + 40     ; gemeinsamer Bildzaehler fuer die Schritt- und Kletter-Geraeusche von Hugo und Gumbo
; Ausgabe fuer die HAL: Die HAL liest diese Werte und malt damit die Figuren (siehe core_prepare_sprites).
; px = x-Position (16 Bit), py = Fuss-y (8 Bit), fr = Bildnummer der Animation (0..13 Held, 0..5 Gegner).
.const h_px_l  = ZP + 41     ; Ausgabe fuer die HAL
.const h_px_h  = ZP + 42
.const h_py    = ZP + 43
.const h_fr    = ZP + 44
.const e_px_l  = ZP + 45
.const e_px_h  = ZP + 46
.const e_py    = ZP + 47
.const e_fr    = ZP + 48
; Zielposition, die der Gegner ansteuert (z. B. Fuss einer Leiter), want: 0 = er will nach oben, 1 = nach unten.
.const tx_l    = ZP + 49     ; Zielposition des Gegners (x der linken Kante, 16 Bit; von find_ladder gesetzt)
.const tx_h    = ZP + 50
.const want    = ZP + 51     ; gewuenschte Leiterrichtung: 0 = hoch, 1 = runter
.const ZP_END  = ZP + 52     ; erste freie Nullseitenzelle hinter dem Kern

; ---------------------------------------------------------------- Tabellen
; Sinustabelle fuer den Sprung. Ein Sprung dauert 32 Bilder (t = 0..32). In jedem Bild liegt die Figur um
;     40 * sin(pi * t / 32)
; Punkte ueber ihrer Grundlinie. Bei t = 0 und t = 32 ist der Sinus 0 (Absprung und Landung auf der Grundlinie),
; bei t = 16 ist er 1 (hoechster Punkt, 40 Punkte hoch = zweieinhalb Kacheln).  Die Zahlen werden beim Uebersetzen
; berechnet (.repeat und die Funktion sin), es muss also keine Tabelle von Hand getippt werden.
; Beispiel: t = 8 ergibt 40 * sin(pi/4) = 28,28, gerundet 28.
sinetab:                     ; Sprungbogen: 40 * sin(pi * t / 32), t = 0..32
    .repeat 33, i
        .byte round(40 * sin(i * pi / 32))
    .endrepeat
; row20[k] = k * 20: Startindex der Kachelzeile k in der Bildkarte (20 Kacheln je Zeile).  Damit ist
; Index = row20[Zeile] + Spalte; mit hoechstens 11 * 20 + 19 = 239 passt der Index in ein Byte.
row20:
    .repeat 12, i
        .byte i * 20
    .endrepeat
; Reihenfolge der Laufbilder (Bildnummern im Spritesatz): 1, 2, 3, 2 - Schritt, Mitte, Schritt, Mitte.
walkseq:
        .byte 1,2,3,2
; Dasselbe fuer den Gegner (Bilder 0, 1, 2, 1).
enemyseq:
        .byte 0,1,2,1

; ---------------------------------------------------------------- Bild starten
; Eingang:  A = Bildnummer 0..3
; Wirkung:  setzt alle Held- und Gegner-Variablen auf die Startwerte dieses Bildes (aus lib/levels.inc),
;           loescht den Bewegungszustand und richtet den Autopiloten auf den Anfang des Skripts dieses Bildes.
;           Der Punktestand (sc0..sc2) bleibt unberuehrt: Das Hauptprogramm loescht ihn nur zu Beginn eines ganzen Durchlaufs.
; Hinweis:  Die Bildkarte (LMAP) laedt NICHT dieser Kern, sondern die HAL (hal_load_level), weil nur sie weiss, woher die Karte kommt.
; Aendert:  A, X, Y, Flags; alle Variablen des Helden und des Gegners; ruft nichts auf.
; Beispiel: Bild 0 startet den Helden bei x = 16, y = 160 (steht auf dem Gras), Blick nach rechts.
core_level_init:
        sta lvl                         ; merken, welches Bild laeuft (0..3)
        tax                             ; X = Bildnummer: Index fuer alle Tabellen mit einem Byte je Bild
        lda #0
        sta lvl_done
        sta hmode
        sta hatim
        sta hy_f
        sta hx_f
        sta ex_f
        sta ey_f
        sta emode
        sta eatim
        sta sfx_t1
        sta ap_cnt
        txa
        asl a                           ; Bildnummer * 2: die Tabellen mit 16-Bit-Werten (lvl_hx, lvl_ex) haben zwei Byte je Bild
        tay                             ; Y = Bildnummer * 2 = Index in die .word-Tabellen
        lda lvl_hx,y                    ; x-Start des Helden, niederwertiges Byte
        sta hx_l
        lda lvl_hx+1,y                  ; ... hoeherwertiges Byte (x kann ueber 255 liegen)
        sta hx_h
        lda lvl_ex,y
        sta ex_l
        lda lvl_ex+1,y
        sta ex_h
        lda lvl_hy,x                    ; Fuss-y des Helden (ein Byte je Bild, deshalb Index X)
        sta hy
        lda lvl_hd,x
        sta hdir
        lda lvl_ey,x
        sta ey
        lda lvl_ed,x
        sta edir
        lda script_lo,x                 ; Autopilot auf den Anfang des Skripts dieses Bildes richten (Zeiger niedrig ...
        sta ap_ptr
        lda script_hi,x                 ; ... und hoch)
        sta ap_ptr+1
        rts

; ---------------------------------------------------------------- Kachel lesen
; get_tile: Eingang X = Spalte (0..19), Y = Zeile (0..11).  Ausgang A = Kachelnummer an dieser Stelle (0 = leer/Himmel).
;           Verbraucht X (und A, Carry); Y bleibt erhalten.  Die Karte liegt in LMAP (RAM), weil eingesammelte Muenzen dort
;           geloescht werden.  Index = row20[Zeile] + Spalte.
get_tile:
        txa
        clc
        adc row20,y                     ; A = Spalte + 20 * Zeile = Index in die Bildkarte
        tax
        lda LMAP,x                      ; Kachelnummer aus der Karte im RAM
        rts
; get_attr: Eingang wie get_tile.  Ausgang A = Eigenschaft der Kachel: 0 = leer/Dekoration (man faellt hindurch),
;           1 = tragend (Boden, Plattform, Mauer), 2 = Leiter.  Spalten ausserhalb 0..19 zaehlen als Wand (1), damit
;           der Gegner nie ueber den Bildrand laeuft.  Verbraucht X und Y bleibt erhalten.
get_attr:
        cpx #20                         ; Spalte >= 20? (ausserhalb des Bildes)
        bcc +
        lda #1
        rts
+       jsr get_tile
        tax
        lda tile_attr,x                 ; Eigenschaft dieser Kachelnummer nachschlagen
        rts

; ---------------------------------------------------------------- Autopilot
; Der Autopilot ist ein winziger Interpreter. Das Skript eines Bildes ist eine Folge von Befehlen (Bytes):
;
;     Byte(s)            Befehl   Bedeutung
;     0                  END      Bild geschafft (Held steht an der Tuer); lvl_done wird 1
;     1  d  n            WALK     d = Richtung (0 rechts, 1 links), n Bilder lang gehen (1,5 Punkte je Bild)
;     2  d  dx dy        JUMP     Sprung: d = Richtung, dx = Weite in Punkten (0..255, ohne Vorzeichen), dy = Hoehenunterschied
;                                 (Byte mit Vorzeichen -128..127, negativ = aufwaerts); dauert immer 32 Bilder
;     3  d  n            CLIMB    d = 0 hoch, 1 runter, n Bilder lang klettern (1 Punkt je Bild)
;     4  n               WAIT     n Bilder stehen bleiben
;     5  k               SFX      Klangeffekt k abspielen (kostet keine Zeit)
;     6  col row         COIN     Muenze an Kachel (col,row) einsammeln: Kachel loeschen, 50 Punkte, Klang (kostet keine Zeit)
;
; Grenze: Die Dauer n von WALK, CLIMB und WAIT soll 1..254 sein.  n = 255 gilt sofort als "Skript zu Ende".  n = 0 ist ein Befehl
; ohne Dauer: hero_update ueberspringt ihn und holt gleich den naechsten (sonst wuerde ap_cnt beim Herunterzaehlen auf 255 laufen).
;
; Die Skripte erzeugt tools/jumprun_art.py: Dort wird jeder Weg vorab simuliert und geprueft (landet der Held nach
; jedem Sprung wirklich auf einer tragenden Kachel?), bevor die Bytes in lib/levels.inc geschrieben werden.
;
; ap_byte: liest das naechste Skriptbyte nach A und rueckt ap_ptr um eins weiter (16-Bit-Zeiger; bei Ueberlauf des
;          niederwertigen Bytes wird das hoeherwertige erhoeht).  Verbraucht Y (= 0); Flags nach dem Zeigerinkrement.
ap_byte:                     ; naechstes Skriptbyte holen
        ldy #0
        lda (ap_ptr),y                  ; Skriptbyte (indirekt ueber den Zeiger, Y = 0)
        inc ap_ptr                      ; Zeiger niederwertig + 1; bei Ueberlauf (Null) das hoeherwertige Byte erhoehen
        bne +
        inc ap_ptr+1
+       rts

; ap_fetch: holt und startet den naechsten Befehl, wenn der vorige abgelaufen ist (ap_cnt = 0).
;   Zeitlose Befehle (SFX, COIN) fuehrt sie sofort aus und holt gleich den naechsten (Sprung nach .again),
;   bis ein Befehl kommt, der Zeit braucht (WALK, JUMP, CLIMB, WAIT) oder das Skript endet (END).
;   Danach steht in ap_cnt, wie viele Bilder der neue Befehl dauert, und hmode sagt, was der Held gerade tut.
;   Aendert A, X, Y, Flags, t0/t1 und die Variablen des Helden; ruft ap_byte, ap_jump, do_coin, hal_sfx.
;   Die Befehle werden in Zahlenreihenfolge abgefragt (cmp #1, #2, ...); ohne Treffer bleibt nur COIN uebrig.
ap_fetch:
.again: jsr ap_byte
        cmp #0
        bne +
        ; END: Bild geschafft
        lda #1
        sta lvl_done
        lda #255                        ; 255 = "fuer immer": hero_update fuehrt nichts mehr aus, bis das Bild gewechselt wird
        sta ap_cnt
        lda #HM_STAND
        sta hmode
        rts
+       cmp #1
        bne +
        ; WALK d n
        jsr ap_byte
        sta hdir
        jsr ap_byte
        sta ap_cnt
        lda #HM_WALK
        sta hmode
        rts
+       cmp #2
        bne +
        jmp ap_jump
+       cmp #3
        bne +
        ; CLIMB d n
        jsr ap_byte
        sta hclimb
        jsr ap_byte
        sta ap_cnt
        lda #HM_CLIMB
        sta hmode
        rts
+       cmp #4
        bne +
        ; WAIT n
        jsr ap_byte
        sta ap_cnt
        lda #HM_STAND
        sta hmode
        rts
+       cmp #5
        bne +
        ; SFX k
        jsr ap_byte
        jsr hal_sfx
        jmp .again
+       ; COIN col row: Spalte nach t0, Zeile nach t1, do_coin erledigt den Rest
        jsr ap_byte
        sta t0
        jsr ap_byte
        sta t1
        jsr do_coin
        jmp .again

; ap_jump: startet einen Sprung.  Parameter aus dem Skript: Richtung d, Weite dx (Punkte, ohne Vorzeichen), Hoehe dy (mit).
;   Der Sprung dauert genau 32 Bilder.  Damit der Held am Ende EXAKT dx Punkte weiter und dy Punkte hoeher/tiefer
;   steht, wird die Geschwindigkeit je Bild so gewaehlt:
;       vx = dx * 256 / 32 = dx * 8     (in 8.8-Festkomma, also dx * 8 / 256 Punkte je Bild)
;       vy = dy * 256 / 32 = dy * 8
;   Beispiel: dx = 70, dy = -32:  vx = 560 = $0230  (2,19 Punkte je Bild),  vy = -256 = $FF00  (-1 Punkt je Bild).
;   Multiplikation mit 8 geschieht durch dreimaliges Linksschieben (asl/rol) der 16-Bit-Zahl.  Bei Blick nach links wird vx
;   negiert (Zweierkomplement: 0 - vx).  vy bekommt fuer negative dy ein $FF als hoeherwertiges Byte (Vorzeichenerweiterung).
;   Die senkrechte Grundlinie hy wandert linear von der Start- zur Zielhoehe; ueber sie legt hero_update den Sinusbogen.
;   Wertebereich: dx 0..255 gibt |vx| bis 2040 = $07F8; dy -128..127 gibt vy von -1024 bis 1016.  Beides passt in 16 Bit.
;   Aendert A, t0, t1, hdir, hvx, hvy, jt, hy_f, ap_cnt, hmode; bei Rueckkehr (rts) beginnt der Sprung mit dem naechsten hero_update.
ap_jump:
        jsr ap_byte
        sta hdir
        jsr ap_byte
        sta t0                   ; dx
        jsr ap_byte
        sta t1                   ; dy (vorzeichenbehaftet)
        ; vx = dx * 8
        lda t0
        sta hvx_l
        lda #0
        sta hvx_h
        asl hvx_l                       ; vx * 2 (16 Bit: asl schiebt in den Uebertrag, rol holt ihn ins hoeherwertige Byte)
        rol hvx_h
        asl hvx_l
        rol hvx_h
        asl hvx_l
        rol hvx_h
        lda hdir                        ; Blick nach links? Dann Geschwindigkeit negieren
        beq +
        sec                      ; links: negieren
        lda #0
        sbc hvx_l                       ; 0 - vx = Zweierkomplement (Subtraktion mit Carry, deshalb vorher sec)
        sta hvx_l
        lda #0
        sbc hvx_h
        sta hvx_h
+       ; vy = dy * 8 mit Vorzeichenerweiterung
        lda t1
        sta hvy_l
        lda #0
        bit t1                          ; dy negativ? (Bit 7 von t1 setzt das N-Flag; A bleibt 0)
        bpl +
        lda #$ff
+       sta hvy_h
        asl hvy_l
        rol hvy_h
        asl hvy_l
        rol hvy_h
        asl hvy_l
        rol hvy_h
        lda #0
        sta jt
        sta hy_f
        lda #32                         ; ein Sprung dauert genau 32 Bilder
        sta ap_cnt
        lda #HM_JUMP
        sta hmode
        rts

; do_coin: sammelt eine Muenze ein.  Eingang t0 = Spalte, t1 = Zeile der Muenzenkachel.  Aendert A, X, Y, Flags.
;   1. Die Kachel in der Bildkarte (LMAP) auf 0 (leer) setzen, damit sie nie wieder auftaucht,
;   2. dieselbe Kachel auf dem Bildschirm loeschen (hal_put_tile),
;   3. 50 Punkte addieren - im Dezimalmodus (sed), damit der Punktestand direkt als Dezimalziffern lesbar ist
;      (BCD: $50 + $50 = $00 mit Uebertrag, das hoeherwertige Byte bekommt +1),
;   4. Punktestand neu zeichnen und den Muenzenklang (Effekt 4) starten (der Aufruf von hal_sfx ist ein Sprung, kein Unterprogramm-
;      aufruf: dessen rts kehrt gleich zum Aufrufer von do_coin zurueck).
;   Grenze: Der Punktestand ist dreistellig BCD (000000..999999); ein Ueberlauf ueber 999999 ist nicht abgefangen (bei 50 Punkten je
;   Muenze und wenigen Muenzen je Bild unerreichbar).  Die NMOS-CPUs (6510, Sally, 7501) loeschen das D-Flag bei einem Interrupt nicht:
;   Faellt ein IRQ in das kurze sed ... cld, liefe seine Routine (Musik, adc) im Dezimalmodus.  Deshalb sperrt do_coin dort die IRQs
;   (php/sei ... plp).  Am Atari (.const NO_SEI_BCD) entfaellt das: dort laufen DLI und VBI als NMI, den sei nicht sperrt; die
;   Interrupt-Routinen stehen im Betriebssystem bzw. im RMT-Player (dass das OS im NMI das D-Flag loescht, ist hier nicht belegt).
do_coin:
        ldx t0
        ldy t1
        txa
        clc
        adc row20,y
        tax
        lda #0
        sta LMAP,x
        ldx t0
        ldy t1
        lda #0
        jsr hal_put_tile
.ifndef NO_SEI_BCD
        php                             ; I-Flag merken (die Hauptschleife laeuft mit freigegebenen Interrupts) ...
        sei                             ; ... und IRQs waehrend des Dezimalmodus sperren
.endif
        sed                             ; Dezimalmodus: adc/sbc rechnen jetzt mit BCD-Ziffern
        clc
        lda sc0
        adc #$50                        ; +50 Punkte (BCD $50)
        sta sc0
        lda sc1
        adc #0
        sta sc1
        lda sc2
        adc #0
        sta sc2
        cld                             ; Dezimalmodus wieder aus (sonst rechnen alle folgenden Additionen falsch!)
.ifndef NO_SEI_BCD
        plp                             ; I-Flag wie vorher (plp stellt auch D = 0 wieder her)
.endif
        jsr hal_score_draw
        lda #4
        jmp hal_sfx

; ---------------------------------------------------------------- Held
; hero_update: wird einmal je Bild aufgerufen.  Der Ablauf:
;   1. Ist der laufende Befehl abgelaufen (ap_cnt = 0), den naechsten holen (ap_fetch).
;   2. Je nach hmode einen Schritt ausfuehren (gehen / klettern / springen / stehen).
;   3. ap_cnt herunterzaehlen; bei 0 holt das naechste Bild den naechsten Befehl.
;   Der Wert 255 in ap_cnt bedeutet "Bild beendet": der Held bleibt stehen, bis das Hauptprogramm das naechste Bild startet.
;
;   Gehen:    x wird je Bild um 1,5 Punkte veraendert (hero_walk), alle 12 Bilder ein Schrittgeraeusch (Effekt 2).
;   Klettern: y aendert sich um 1 Punkt je Bild, alle 10 Bilder ein Klettergeraeusch (Effekt 3).
;   Springen: x und y laufen mit den in ap_jump berechneten Geschwindigkeiten; jt zaehlt 0..32.  Die ANZEIGE-Hoehe ist
;             hy minus sinetab[jt] (siehe core_prepare_sprites), die Grundlinie hy selbst bewegt sich geradlinig.
;   Eingang: keiner (Zustand in den Variablen).  Aendert alle Register und Flags, t0/t1, die Held-Variablen; ruft ap_fetch, hero_walk, hal_sfx.
hero_update:
        lda ap_cnt
        bne +
.fetch: jsr ap_fetch
        lda ap_cnt
        beq .fetch                      ; Befehl ohne Dauer (n = 0): ueberspringen und den naechsten holen
+       lda ap_cnt
        cmp #255                        ; Bild beendet? Dann nichts mehr tun
        beq .done                       ; Ende des Bildes: stehen bleiben
        lda hmode
        cmp #HM_WALK
        beq .walk
        cmp #HM_JUMP
        beq .jump
        cmp #HM_CLIMB
        beq .climb
        dec ap_cnt                      ; stehen / warten
        jmp .done
.walk:  jsr hero_walk
        dec ap_cnt
        inc hatim
        ; Schrittgeraeusch alle 12 Bilder
        inc sfx_t1                      ; Zaehler fuer das Schrittgeraeusch
        lda sfx_t1
        cmp #12                         ; alle 12 Bilder ein Schritt-Klick (Effekt 2), aber nur wenn kein anderer Effekt klingt
        bcc .done
        lda #0
        sta sfx_t1
        lda sfx_on
        bne .done
        lda #2
        jsr hal_sfx
        jmp .done
.climb: lda hclimb
        bne +
        dec hy                          ; hoch (y waechst nach unten)
        jmp ++
+       inc hy                          ; runter
+      dec ap_cnt
        inc hatim                       ; Kletter-Animation weiterschalten
        inc sfx_t1
        lda sfx_t1
        cmp #10                         ; alle 10 Bilder ein Kletter-Blip (Effekt 3), wenn kein anderer Effekt klingt
        bcc .done
        lda #0
        sta sfx_t1
        lda sfx_on
        bne .done
        lda #3
        jsr hal_sfx
        jmp .done
.jump:  clc                             ; x (24 Bit hx_h:hx_l:hx_f) += vx (16 Bit mit Vorzeichen): zuerst Bruchteil und ganze Punkte ...
        lda hx_f
        adc hvx_l
        sta hx_f
        lda hx_l
        adc hvx_h
        sta hx_l
        lda hvx_h
        bmi +                           ; ... dann das hohe Byte: vx negativ -> $FF addieren, sonst 0 (Vorzeichenerweiterung auf 24 Bit)
        lda #0
        beq ++                          ; immer genommen (A = 0); der Uebertrag aus dem adc oben bleibt dabei erhalten
+       lda #$ff                        ; negative Geschwindigkeit: hoeherwertiges Byte = $FF
+      adc hx_h                         ; plus Uebertrag der Addition von hx_l
        sta hx_h
        clc                             ; y (16 Bit hy:hy_f) += vy
        lda hy_f
        adc hvy_l
        sta hy_f
        lda hy
        adc hvy_h                       ; Grundlinie += vy; Zweierkomplement-Addition gilt auch fuer negatives vy
        sta hy
        inc jt                          ; naechster Schritt im Sprung (0..32)
        dec ap_cnt
        bne .done
        lda #HM_STAND
        sta hmode
.done:  rts


; hero_walk: ein Gehschritt von 1,5 Punkten: addiert (links: subtrahiert) $0180 zur 24-Bit-Position hx_h:hx_l:hx_f.
;   Eingang hdir (0 rechts, 1 links).  Aendert A, Flags und die Position.  Kein Randtest: das Skript sorgt dafuer, dass der Held im Bild bleibt.
;   $0180 = 1,5 in 8.8.  Der Bruchteil ($80 = 0,5) laeuft bei jedem zweiten Schritt ueber und ergibt so abwechselnd
;   1 und 2 ganze Punkte: im Mittel 1,5.  Die Uebertraege werden mit adc/sbc ohne zwischenzeitliches clc/sec weitergereicht.
hero_walk:
        lda hdir
        bne .l
        clc
        lda hx_f
        adc #$80                        ; +0,5 Punkt im Bruchteil (Uebertrag moeglich) ...
        sta hx_f
        lda hx_l
        adc #1                          ; ... + 1 ganzer Punkt + eventueller Uebertrag = 1,5 Punkte je Bild
        sta hx_l
        lda hx_h
        adc #0
        sta hx_h
        rts
.l:     sec
        lda hx_f
        sbc #$80                        ; nach links: -0,5 im Bruchteil ...
        sta hx_f
        lda hx_l
        sbc #1                          ; ... und -1 ganzer Punkt
        sta hx_l
        lda hx_h
        sbc #0
        sta hx_h
        rts

; ---------------------------------------------------------------- Gegner "Gumbo": geht, klettert Leitern, springt NIE
;
; Gumbo hat eine EIGENE Physik, die sich von Hugos unterscheidet (das ist der Witz der Demo):
;   * er geht nur auf tragenden Kacheln und bleibt an jeder Kante stehen (er laeuft nie ins Leere),
;   * er stoppt vor Mauern (tragende Kacheln auf Hoehe des Koerpers) - ueber Hindernisse springen kann er nicht,
;   * er klettert Leitern, aber nur, wenn eine Leiter in seiner Reichweite liegt,
;   * er ist langsamer als Hugo (0,6 statt 1,5 Punkte je Bild beim Gehen, 0,5 beim Klettern).
; Der Zustandsautomat (emode):  0 = gehen/entscheiden,  1 = klettern,  2 = warten ("schimpfen") nach einem Fehlschlag.
;
; enemy_step: versucht EINEN Gehschritt in Richtung edir.  Ausgang: Carry = 1 bei Erfolg, 0 wenn blockiert.
;   Pruefung am "Frontpunkt" = Mitte der Figur +/- 9 Punkte (je nach Richtung):
;     1. Spalte = Frontpunkt / 16 (vier Mal rechts schieben; ausserhalb 0..19 gilt als blockiert),
;     2. Boden: Kachel in der Zeile der Fuesse (ey / 16) muss tragend sein (Eigenschaft 1),
;     3. Wand: Kachel eine Zeile hoeher ((ey-1) / 16) darf NICHT tragend sein.
;   Dann Position um 154/256 = 0,6 Punkte aendern.
;   Aendert A, X, Y, t0, t1, t2 und (bei Erfolg) ex_h:ex_l:ex_f; ruft get_attr.
enemy_step:                     ; C = 1 wenn der Schritt gelungen ist
        clc
        lda ex_l
        adc #12                         ; Figurmitte = linke Kante + 12
        sta t0
        lda ex_h
        adc #0
        sta t1
        lda edir
        bne .left
        clc
        lda t0
        adc #9                          ; Frontpunkt rechts: Mitte + 9 Punkte
        sta t0
        lda t1
        adc #0
        sta t1
        jmp .col
.left:  sec
        lda t0
        sbc #9                          ; Frontpunkt links: Mitte - 9 Punkte
        sta t0
        lda t1
        sbc #0
        sta t1
        bmi .blocked                    ; negativ (links vom Bildrand): blockiert
.col:   lda t1                          ; Spalte = x / 16: 16-Bit-Wert t1:t0 viermal nach rechts, das Ergebnis steht in t0
        lsr a
        ror t0
        lsr a
        ror t0
        lsr a
        ror t0
        lsr a
        ror t0
        lda t0
        cmp #20
        bcs .blocked
        sta t2                   ; Spalte merken
        tax
        lda ey                   ; Boden unter den Fuessen: Zeile ey/16
        lsr a
        lsr a
        lsr a
        lsr a
        tay
        jsr get_attr
        cmp #1
        bne .blocked
        ldx t2
        lda ey                   ; Wand im Weg: Zeile (ey-1)/16
        sec
        sbc #1
        lsr a
        lsr a
        lsr a
        lsr a
        tay
        jsr get_attr
        cmp #1
        beq .blocked
        ; Schritt: 154/256 Punkte je Bild
        lda edir
        bne .sl
        clc
        lda ex_f
        adc #154                        ; 154 / 256 = 0,6 Punkte je Bild (der Gegner ist langsamer als der Held)
        sta ex_f
        lda ex_l
        adc #0
        sta ex_l
        lda ex_h
        adc #0
        sta ex_h
        sec
        rts
.sl:    sec
        lda ex_f
        sbc #154
        sta ex_f
        lda ex_l
        sbc #0
        sta ex_l
        lda ex_h
        sbc #0
        sta ex_h
        sec
        rts
.blocked:
        clc
        rts

; chk_ladder: ist in Spalte X eine passende Leiter?  Eingang X = Spalte, want = 0 (nach oben) oder 1 (nach unten).
;   Nach oben muss die Kachel ueber den Fuessen (Zeile (ey-1)/16) eine Leiter sein, nach unten die Kachel auf Fusshoehe (ey/16).
;   Ausgang: Carry = 1 wenn ja.  X bleibt die Spalte (find_ladder braucht sie danach).  Aendert A, Y, t2; ruft get_attr.
chk_ladder:
        stx t2
        lda want
        bne .down
        lda ey
        sec
        sbc #1
        jmp .row
.down:  lda ey
.row:   lsr a
        lsr a
        lsr a
        lsr a
        tay
        ldx t2
        jsr get_attr
        ldx t2
        cmp #2
        beq +
        clc
        rts
+       sec
        rts

; find_ladder: sucht die naechste passende Leiter, vom Gegner aus nach links UND rechts, immer weiter weg (Abstand 0..19).
;   Mittelspalte = (ex + 12) / 16.  Bei Erfolg: Carry = 1, ecol = Leiterspalte und tx = Zielposition fuer die linke Kante
;   der Figur: Spalte * 16 - 4, damit die Figurmitte (+12) auf der Leitermitte (+8) liegt.  Bei gleichem Abstand wird LINKS zuerst
;   geprueft.  Misserfolg: Carry = 0 (ecol und tx bleiben dann unveraendert).  Aendert A, X, Y, t0, t1, t3; ruft chk_ladder.
find_ladder:
        clc
        lda ex_l
        adc #12
        sta t0
        lda ex_h
        adc #0
        lsr a
        ror t0
        lsr a
        ror t0
        lsr a
        ror t0
        lsr a
        ror t0
        lda t0
        sta t3                   ; Mittelspalte
        ldx #0                   ; Abstand
.dist:  stx t1
        lda t3
        sec
        sbc t1                          ; Spalte links vom Gegner (Mittelspalte - Abstand)
        bcc +                    ; links ausserhalb
        tax
        jsr chk_ladder
        bcs .found
+       lda t1
        beq +                           ; Abstand 0: die Mittelspalte selbst wurde schon geprueft
        clc
        adc t3                          ; Spalte rechts vom Gegner (Mittelspalte + Abstand)
        cmp #20
        bcs +
        tax
        jsr chk_ladder
        bcs .found
+       ldx t1
        inx
        cpx #20
        bcc .dist
        clc
        rts
.found: stx ecol
        txa
        asl a
        asl a
        asl a
        asl a                           ; Spalte * 16 (niedriges Byte) ...
        sta tx_l
        lda ecol
        lsr a
        lsr a
        lsr a
        lsr a                           ; ... und das hohe Byte von Spalte * 16 (= Spalte / 16)
        sta tx_h
        sec
        lda tx_l
        sbc #4                          ; minus 4: die Figurmitte (x + 12) soll auf die Leitermitte (Spalte * 16 + 8) fallen
        sta tx_l
        lda tx_h
        sbc #0
        sta tx_h
        sec
        rts


; enemy_update: einmal je Bild.  Entscheidung des Gegners:
;   emode 2 (warten):  ewait herunterzaehlen, dann wieder entscheiden - die Figur steht und "wackelt" (Animation laeuft).
;   emode 1 (klettern): 0,5 Punkt je Bild (der Bruchteil in ey_f laeuft jedes zweite Bild ueber), am Ende der Leiter
;                       (naechste Kachel in Kletterrichtung ist keine Leiter mehr) zurueck in den Gehmodus.
;   emode 0 (gehen):   a) Steht der Held etwa auf gleicher Hoehe (Unterschied < 8 Punkte in y)? Dann auf ihn zu, bis auf
;                         24 Punkte Abstand.  Blockiert der Weg (Kante, Mauer), wartet Gumbo 25 Bilder.
;                      b) Sonst eine Leiter suchen (find_ladder).  Gibt es keine, wartet er 40 Bilder (er kann nicht
;                         springen!).  Gibt es eine, geht er zu ihr; steht er im Abstand von weniger als 2 Punkten
;                         davor, setzt er sich auf die Leiter (x genau auf die Leitermitte) und klettert.
;   Aendert alle Register und Flags, t0..t3 und die Gegner-Variablen; ruft enemy_step, find_ladder, get_attr, hal_sfx.
;   eatim steuert die Animation und zaehlt nur in Bildern, in denen sich der Gegner bewegt oder wartet (emode 2); beim Stehenbleiben
;   neben dem Helden (.idle) und in Bildern, in denen er nur entscheidet (Leiter gewaehlt, aufsteigen), steht auch die Animation.
enemy_update:
        lda emode
        beq .walk
        cmp #1
        beq .climb
        ; warten (frustriert): kurz auf der Stelle, dann neu entscheiden
        dec ewait
        bne +
        lda #0
        sta emode                       ; Wartezeit um: wieder gehen/entscheiden
+       inc eatim
        rts
.climb: inc eatim
        clc
        lda ey_f
        adc #$80                        ; 0,5 Punkte je Bild
        sta ey_f
        bcc .cdone                      ; Uebertrag nur jedes zweite Bild: dann erst bewegt sich ey um einen ganzen Punkt
        lda eclimb
        bne .down
        dec ey
        lda ey
        sec
        sbc #1
        lsr a
        lsr a
        lsr a
        lsr a
        tay
        ldx ecol
        jsr get_attr
        cmp #2
        beq .cdone
        lda #0
        sta emode                       ; oben angekommen (keine Leiter mehr): zurueck in den Gehmodus (ohne Kletter-Geraeusch)
        rts
.down:  inc ey
        lda ey
        lsr a
        lsr a
        lsr a
        lsr a
        tay
        ldx ecol
        jsr get_attr
        cmp #2
        beq .cdone
        lda #0
        sta emode
.cdone: ; Kletter-Geraeusch: alle 20 Bilder Effekt 5, wenn kein anderer Effekt klingt (sfx_t1 teilt sich Gumbo mit Hugo)
        inc sfx_t1
        lda sfx_t1
        cmp #20
        bcc +
        lda #0
        sta sfx_t1
        lda sfx_on
        bne +
        lda #5
        jsr hal_sfx
+       rts
.walk:  ; gleiche Ebene?  Betrag von hy - ey kleiner als 8 Punkte (hy ist beim Sprung die Grundlinie, nicht die Anzeigehoehe)
        lda hy
        sec
        sbc ey
        bcs +
        eor #$ff                        ; Betrag des Hoehenunterschieds: negative Zahl negieren (Zweierkomplement: invertieren, +1)
        adc #1
+       cmp #8                          ; Unterschied kleiner als 8 Punkte: gleiche Ebene
        bcs .other
        ; gleiche Ebene: auf den Helden zu, bis auf 24 Punkte
        lda hx_l
        sec
        sbc ex_l
        sta t0
        lda hx_h
        sbc ex_h
        sta t1
        bmi .toleft
        lda #0
        sta edir                        ; Held rechts von Gumbo: nach rechts
        lda t1
        bne .go
        lda t0
        cmp #24                         ; naeher als 24 Punkte: Gumbo bleibt stehen (er ist "dran")
        bcc .idle
        jmp .go
.toleft:
        lda #1
        sta edir                        ; Held links von Gumbo: nach links
        sec                             ; Abstand = Betrag von (t1:t0): 16-Bit-Zweierkomplement
        lda #0
        sbc t0
        sta t0
        lda #0
        sbc t1
        sta t1
        bne .go
        lda t0
        cmp #24
        bcc .idle
.go:    jsr enemy_step
        bcs +
        lda #25                         ; blockiert (Kante oder Mauer): 25 Bilder warten und schimpfen
        sta ewait
        lda #2
        sta emode
        rts
+       inc eatim
        rts
.idle:  rts
.other: ; andere Ebene: Leiter suchen
        lda hy
        cmp ey
        bcc +
        lda #1                          ; Held tiefer (hy >= ey, y waechst nach unten) -> Leiter nach unten
        jmp ++
+       lda #0                          ; Held hoeher -> Leiter nach oben
+      sta want
        jsr find_ladder
        bcs +
        lda #40                  ; keine Leiter: stehen und schimpfen
        sta ewait
        lda #2
        sta emode
        rts
+       lda ex_l                        ; Abstand zur Leiter: ex - tx (16 Bit mit Vorzeichen)
        sec
        sbc tx_l
        sta t0
        lda ex_h
        sbc tx_h
        sta t1
        bmi .tright
        lda t1
        bne .tl
        lda t0
        cmp #2
        bcc .atladder
.tl:    lda #1
        sta edir
        jmp .go
.tright:
        lda t1
        cmp #$ff
        bne .tr
        lda t0
        cmp #$fe
        bcs .atladder
.tr:    lda #0
        sta edir
        jmp .go
.atladder:                              ; Abstand zur Leiter kleiner als 2 Punkte:
        lda tx_l                        ; auf die Leiter setzen (x genau auf die Leitermitte, Bruchteil 0)
        sta ex_l
        lda tx_h
        sta ex_h
        lda #0
        sta ex_f
        lda want
        sta eclimb
        lda #1
        sta emode
        rts

; ---------------------------------------------------------------- Ausgabe fuer die HAL aufbereiten
; core_prepare_sprites: rechnet den Spielzustand in die Werte um, die die HAL zum Zeichnen braucht
;   h_px_l/h (x), h_py (y), h_fr (Bildnummer) fuer den Helden und e_px_l/h, e_py, e_fr fuer den Gegner.
;   * Beim Sprung wird die Anzeige-Hoehe berechnet: y = Grundlinie hy - sinetab[jt].
;   * Die Bildnummer richtet sich nach dem Zustand: Held stehen 0, gehen 1/2/3/2 (alle 4 Bilder weiter), springen 4,
;     klettern 5/6 (alle 8 Bilder wechseln); bei Blick nach links wird 7 addiert (zweite Haelfte des Spritesatzes =
;     gespiegelte Bilder).  Gegner: gehen 0/1/2/1 (alle 8 Bilder weiter), klettern 0/2 (alle 8 Bilder wechseln); bei Blick nach
;     links + 3.
;   Eingang: der Spielzustand.  Ausgang: h_px_l/h/h_py/h_fr und e_px_l/h/e_py/e_fr (siehe oben, "Ausgabe fuer die HAL").
;   Aendert A, X, Flags.  Wird auch vom Titelbild des Hauptprogramms aufgerufen, nachdem es Positionen und Zustand von Hand gesetzt hat.
core_prepare_sprites:
        lda hx_l
        sta h_px_l
        lda hx_h
        sta h_px_h
        lda hy
        ldx hmode
        cpx #HM_JUMP
        bne +
        ldx jt
        sec
        sbc sinetab,x                   ; Anzeigehoehe = Grundlinie - Sinuswert (y waechst nach unten, also steigt die Figur)
+       sta h_py
        ; Held-Bild: Stehen 0, Gehen (1,2,3,2), Sprung 4, Klettern 5/6; plus 7 bei Blick nach links
        lda hmode
        bne +
        lda #0
        jmp .hdir
+       cmp #HM_WALK
        bne +
        lda hatim
        lsr a
        lsr a
        and #3
        tax
        lda walkseq,x                   ; Laufphase -> Bildnummer (1, 2, 3, 2)
        jmp .hdir
+       cmp #HM_JUMP
        bne +
        lda #4
        jmp .hdir
+       lda hatim                ; Klettern
        lsr a
        lsr a
        lsr a
        and #1
        clc
        adc #5
.hdir:  ldx hdir
        beq +
        clc
        adc #7
+       sta h_fr
        ; Gegner
        lda ex_l
        sta e_px_l
        lda ex_h
        sta e_px_h
        lda ey
        sta e_py
        lda emode
        cmp #1
        bne +
        lda eatim                       ; Klettern: Bilder 0 und 2
        lsr a
        lsr a
        lsr a
        and #1
        asl a
        jmp .edir
+       lda eatim
        lsr a
        lsr a
        lsr a
        and #3
        tax
        lda enemyseq,x                  ; Gegnerphase -> Bildnummer (0, 1, 2, 1)
.edir:  ldx edir
        beq +
        clc
        adc #3
+       sta e_fr
        rts
