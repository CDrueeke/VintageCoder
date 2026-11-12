; =====================================================================================================================
; prallball.asm - PRALLBALL fuer den Commodore 128 (VIC-II-Fassung, 40 Spalten: 24 Levels, Joystick/Tastatur, Bestenliste)
; =====================================================================================================================
;
; ZWECK
;   Ein Ball-und-Schlaeger-Spiel (Breakout-Art) fuer den C128 im 128-Modus, PAL, auf dem 40-Spalten-Bildschirm des VIC-IIe.  Der Ball
;   prallt von Waenden, Schlaeger und Steinen ab; sind alle zerstoerbaren Steine weg, beginnt das naechste Level (24 Levels, darunter
;   die Bonuslevels 11 und 22 und die Bosslevels 23 und 24; nach dem letzten kommt wieder das erste, die Ballgeschwindigkeit bleibt
;   dabei erhalten).  Faellt der Ball unten heraus, geht ein Ball (Leben) verloren; nach dem letzten endet das Spiel, und wer in die
;   Bestenliste kommt, gibt seinen Namen ein (Datei PRALL.HS auf der Diskette).
;   Die Technik entspricht der C64-Fassung: Mehrfarb-Textmodus mit eigenem Zeichensatz, Hardware-Sprites, Raster-Farbbaender, SID.
;   Der Code ist aber eigenstaendig (er wird nicht mit anderen Systemen geteilt), weil MMU, Programmstart (BASIC 7) und Dateizugriff
;   des C128 anders sind.  Der 80-Spalten-Chip VDC wird NICHT benutzt (dafuer gibt es examples/c128/prallball_vdc).
; 
;
; BEDIENUNG
;   Joystick in Port 2 (links/rechts, Feuer) ODER Tastatur: Z = links, X = rechts, Leertaste = Feuer, RUN/STOP = Abbruch zum Titel.
;   (P = Pause meldet nur die Eingabebibliothek als IN_PAUSE; das Spiel wertet es nicht aus.  Q/A = hoch/runter sowie alle Buchstaben und
;   Ziffern werden nur bei der Namenseingabe gebraucht.)  Feuer startet den Ball vom Schlaeger und schiesst beim Laser-Extra.
;   Im 40-Spalten-Modus starten (Taste 40/80 DISPLAY bzw. x128 -40col), sonst zeigt der Monitor nur den VDC-Schirm.
;   Der Abprallwinkel haengt davon ab, wo der Ball den Schlaeger trifft (8 Zonen: -60, -43, -26, -9, +9, +26, +43, +60 Grad zur
;   Senkrechten, siehe lib/dirtable.inc) und von der Schlaegerbewegung (eine Zone weiter in Bewegungsrichtung).  Alle 8 Schlaegertreffer
;   steigt die Geschwindigkeitsstufe (0..15) um 1.
;   Extras: Aus einem zerstoerten Stein faellt mit etwa 1/12 Wahrscheinlichkeit (Ueberraschungsstein: immer) eine rotierende Muenze mit
;   einem Buchstaben; faengt der Schlaeger sie, wirkt sie: B breit, S schmal, R umgekehrte Steuerung, L Ball langsamer, F schneller,
;   K klebrig (naechster Treffer), + ein Ball mehr (hoechstens 9), P 500 Punkte, M Mehrball (bis 3), G Laser, D Durchschlag (Zeiten: activate).
;
; ABLAUF (Zustandsautomat im Hauptprogramm main)
;   title      Titelbild mit Musik, Top 5, Ticker und Lauftext; wartet auf Feuer (nach 30 s ohne Eingabe: Demo)       ->  new_game
;   new_game   Punkte 0, 3 Baelle, Level 1, Geschwindigkeitsstufe 0                                                   ->  load_level
;   load_level Steinkarte laden und zeichnen, Ball auf den Schlaeger                                                  ->  play
;   play       ein Durchlauf je Bildwechsel (50 Hz PAL, wait_frame): Eingabe, Schlaeger, Kapsel, Ball, Kollisionen, Anzeige
;              Rueckgabe: 0 = Level geschafft -> naechstes Level;  1 = Ball verloren -> Ball - 1 (im Bonuslevel kostenlos);  2 = Abbruch
;   game_over  Bestenliste pruefen, Namen eingeben (name_entry), speichern (hs_save)                                  ->  title
;   Demo (demo = 1): der Rechner spielt die Levels 1-5, die Soundeffekte schweigen, die Musik laeuft; jede Eingabe beendet die Demo.
;
; TESTFASSUNGEN (vcc asm prallball.asm --target c128 --cpu 8502 -D NAME[=wert] -o x.prg; ohne -D entsteht das normale Spiel)
;   AUTOPLAY       Rechner spielt von Anfang an ohne Ende (Demo-Modus, Effekte stumm, Titel wird uebersprungen): zur Pruefung der Physik
;   STARTLEVEL=n   beginnt mit Level n (0-basiert: 0 = erstes Level)
;   TESTMULTI      jeder spawn_cap-Aufruf loest sofort Mehrball aus, die Kapselart ist immer M
;   TESTLASER      Laser ist nach jedem reset_extras (neues Level, neuer Ball) 240 Halbsekunden lang an
;   TESTLOSE       ueberspringt das Warten im Titel; Eingabe ist nur Feuer (der Schlaeger bewegt sich nie), also geht jeder Ball verloren
;   TESTFPS        zeigt alle 16 Bilder den 16-Bit-Bildzaehler ($C200/$C201) als Hexziffern in Zeile 24 (Luecken = ausgelassene Bilder)
;   TESTCAP        jeder zerstoerte Stein laesst eine Kapsel fallen
;   NOTICKER       schaltet den Ticker in der Demo ab
;
; HARDWARE (Namen aus data/sys/c128/hardware.asm und data/sys/c64/hardware.asm; Bibliotheken data/sys/c128/{input,rand,sound,hiscore}.asm)
;   VIC-IIe  Mehrfarb-Textmodus ($D016 Bit 4), 8 Sprites ($D000-$D02E), Rasterinterrupt ($D012/$D019/$D01A), Rand und Hintergrund
;            ($D020/$D021) fuer die Farbbaender;  $D030 Bit 0 = Taktwahl (hier 1 MHz)
;   CIA 1    Joystick Port 2 und Tastatur (input.asm);  CIA 2 ($DD00 Bits 0-1) waehlt die VIC-Bank
;   SID      Effekte auf Stimme 3 (sound.asm: sfx_*), Titelmusik auf allen 3 Stimmen (mus_*)
;   MMU      $FF00 (Spiegel von $D500): Konfiguration, siehe unten;  KERNAL nur fuer die Bestenliste (LOAD/SAVE, hiscore.asm)
;
; MMU, VIC-BANK UND TAKT
;   Nach RUN gilt die MMU-Konfiguration $00 (RAM-Bank 0, Systemroms und E/A sichtbar).  copy_font stellt $3E ein und das Spiel bleibt
;   dabei: Bit 0 = 0 laesst den E/A-Bereich $D000-$DFFF (VIC, SID, CIA, Farb-RAM) sichtbar, die Bits 1-5 = 1 blenden alle Systemroms aus
;   (Low-ROM $4000-$7FFF, Mid-ROM $8000-$BFFF, High-ROM/KERNAL $C000-$FFFF), die Bits 6-7 = 0 waehlen die RAM-Bank 0.  Der Prozessor liest
;   die Vektoren $FFFA-$FFFF daher aus dem RAM: der Interruptvektor $FFFE zeigt auf irq, NMI (und der Vollstaendigkeit halber RESET) auf rti_only.  Die MMU-Register
;   $FF00-$FF04 bleiben in jeder Konfiguration sichtbar.  Es gibt keine KERNAL-Aufrufe ausser in den folgenden Fenstern:
;     - copy_font (Konfiguration $01: ROMs ein, E/A aus, damit das Zeichen-ROM bei $D000 lesbar ist) und
;     - die Dateizugriffe der Bestenliste (hs_io_on/hs_io_off: Konfiguration $00, VIC-Interrupt aus, SETMSG/SETBNK).
;   Der Code, der in diesen Fenstern laeuft, liegt unter $4000 (dort ist bei jeder Konfiguration RAM sichtbar), darum ist hiscore.asm
;   zuerst eingebunden.
;   VIC-Bank: CIA 2 $DD00 Bits 0-1 = %01 (invertiert) = Bank 2 = $8000-$BFFF.  Welche RAM-Bank der VIC liest, bestimmt das MMU-Register
;   RCR ($D506 Bits 6-7); das Spiel aendert es nicht (vermutlich Bank 0 wie nach dem Start).  Das Zeichen-ROM sieht der VIC in Bank 0 und 2
;   an $1000-$1FFF bzw. $9000-$9FFF, darum liegt dort nichts Grafisches (Zeichensatz $8800, Sprites $8000, Bildschirm $8400).
;   Takt: 1 MHz (vic_init loescht $D030 Bit 0); die 2-MHz-Betriebsart wuerde laut Kommentar im Code den VIC-II stoeren und bleibt aus.
;
; SPEICHERPLAN (Stand: Adressen laut Assemblerlauf; die Grenzen im Programmbereich vergibt der Assembler fortlaufend)
;   $02-$0D       Zeropage der Bibliotheken: Zufall $02-$03 (rand.asm), Musik $04-$07 (sound.asm), Bestenliste $08-$0D (hiscore.asm)
;   $10-$8B       Zeropage des Spiels (siehe .const unten);  $8C-$8F Eingabe (input.asm).  Das KERNAL (nur in den Fenstern der Bestenliste
;                 aktiv) benutzt $90 und aufwaerts; laut Messung in VICE (rt_zp.asm) bleibt $02-$8F von KERNAL und Interrupt unberuehrt
;   $1C01-$1C0C   BASIC-Zeile "10 SYS <start>" (derzeit SYS 8067 = $1F83), vom Assembler erzeugt (.basicstub)
;   $1C0D-$1F82   Bestenliste (hiscore.asm) mit hs_io_on/hs_io_off, hs_buf, hs_tmp   (muss unter $4000 liegen)
;   $1F83-$404B   Spielcode und -tabellen: start, Hauptschleife, copy_font (unter $4000!), vic_init, irq, Routinen, Ticker-Texte tk_msgs
;   $404C-$45BF   Bibliotheken input.asm, rand.asm, sound.asm
;   $45C0-$47BF   Richtungstabelle dir_xl/dir_xh/dir_yl/dir_yh (lib/dirtable.inc: 16 Stufen x 8 Zonen)
;   $47C0-$47DF   eigene Zeichen 64-67 (gfx_chars, lib/gfx.inc.chars)
;   $47E0-$7FFF   unbenutzt (die PRG-Datei fuellt die Luecke mit Nullen, sie ist darum ca. 37 KB gross)
;   $8000-$82BF   Sprite-Daten (VIC-Bank 2, Zeiger $00..$0A; lib/gfx.inc.sprites): Ball, Schlaeger 16 px links/rechts, 24 px links/rechts,
;                 16 px, Kapsel und ihre 3 Drehphasen, Laserschuss.  Zeiger = (Adresse - $8000) / 64
;   $8400-$87E7   Bildschirm (Text, 40 x 25);  $87F8-$87FF Sprite-Zeiger (SPTR)
;   $8800-$8FFF   Zeichensatz (aus dem Zeichen-ROM kopiert; eigene Zeichen 64-67 bei $8A00, Ticker-Zeichen 224-255 bei $8F00)
;   $9000-$93FF   unbenutzt (der VIC sieht in $9000-$9FFF nur das Zeichen-ROM)
;   $9400-$ADAD   Levelkarten (24 x 216 Byte, Quelle levels/levels.txt), level_kind, level_tab_lo/hi, danach das Lied (lib/music.inc)
;   $C000-$C0D7   Steinkarte im RAM (216 Byte = 12 Zeilen x 18 Spalten, 0 = leer)
;   $C100-$C17F   Ballspeicher (3 Baelle), Laserschuesse, Titelbaelle, Ticker-Puffer;  $C200/$C201 Bildzaehler (nur TESTFPS)
;   $D800-$DBE7   Farb-RAM (E/A-Bereich, 4 Bit je Zelle)
;
; KOORDINATEN UND EINHEITEN (wichtig fuer das Verstaendnis der Physik!)
;   Bildpunkte (px) zaehlen ab der linken oberen Ecke des TEXTBILDSCHIRMS (Zeichen 0/0 = px 0/0).  Ein Zeichen ist 8 x 8 px.
;   Spielfeld: x = 16..303 (Spalten 2..37), y = 16.. (Zeile 2 abwaerts); Waende in Spalte 1 und 38 sowie Zeile 1.
;   Steinfeld: 18 Steine je 16 px Breite (2 Zeichen) ab x = 16, 12 Zeilen je 8 px Hoehe ab y = 32 (Bildschirmzeile 4).
;   Ball und Schlaeger-Positionen: BALL in "Festkomma 1/64": Wert = Bildpunkte * 64 (16 Bit).  So reichen 16 Bit fuer x bis 1023 px
;   und die Geschwindigkeit kann in 1/64 px je Bild feiner als 1 px sein.  Die obere Haelfte (High-Byte) = Bildpunkte / 4.
;   Die Ballgeschwindigkeit betraegt (1,6 + 0,1 * Stufe) px je Bild (50 Bilder/s), aufgeteilt nach dem Winkel der Zone.
;   Der Schlaeger (padl/padh) und die Kapsel (capxl/capxh) zaehlen in ganzen px; Schlaeger oben bei PADY = 184 px (Bildschirmzeile 23).
;   Sprite-Koordinaten des VIC-II beginnen bei x = 24, y = 50 (linker/oberer Rand des Textfensters): sprite_x = px + 24, sprite_y = py + 50.
;
; BEKANNTE GRENZEN
;   Prototyp: keine Pause-Funktion; NTSC nicht getestet; der 2-MHz-Betrieb wird bewusst nicht benutzt; Tastatur und Joystick sind nur im
;   Emulator gespielt, der Klang ungehoert (siehe README.md).  Steintyp 12 (explosiv) hat im Code keine Sonderwirkung.
; =====================================================================================================================
.target c128
.cpu 8502 { illegals = deny }
.branchlong auto                        ; zu weite bedingte Spruenge verlaengert der Assembler selbst (Warnung W203)

; ---------------------------------------------------------------------------------------------------------------------
; Nullseiten-Variablen des Spiels ($10-$8B, nicht der Reihe nach deklariert; frei sind $3C-$3F; $02-$0D und $8C-$8F gehoeren den Bibliotheken)
; Namen und Bedeutung: Ball (bx.., by.., vx.., vy..) = Zustand des GERADE bearbeiteten Balls (load_ball/save_ball kopieren ihn aus dem
; Ballspeicher); t0..t3 = kurzlebige Hilfswerte, die jede Routine fuer sich benutzt (Vorsicht: Aufrufer duerfen sich nicht darauf verlassen).
; ---------------------------------------------------------------------------------------------------------------------
.const bxl     = $10                    ; Ball-X, Festkomma 1/64 px, Low-Byte
.const bxh     = $11                    ;                           High-Byte
.const byl     = $12                    ; Ball-Y, Festkomma 1/64 px
.const byh     = $13
.const vxl     = $14                    ; Ball-Geschwindigkeit X (vorzeichenbehaftet, 1/64 px je Bild)
.const vxh     = $15
.const vyl     = $16                    ; Ball-Geschwindigkeit Y (negativ = nach oben)
.const vyh     = $17
.const padl    = $18                    ; Schlaeger: linke Kante in px (16 Bit, 16..272)
.const padh    = $19
.const speed   = $1a                    ; Geschwindigkeitsstufe 0..15 (Index in die Richtungstabelle)
.const hits    = $1b                    ; Schlaegertreffer seit dem Start; alle 8 steigt die Stufe um 1
.const lives   = $1c                    ; verbleibende Baelle
.const level   = $1d                    ; aktuelles Level 0.. (Index in die Levelliste)
.const bricks  = $1e                    ; noch zu zerstoerende Steine (unzerstoerbare zaehlen nicht)
.const stuck   = $1f                    ; 1 = Ball haftet noch am Schlaeger (wartet auf Feuer)
.const sc0     = $20                    ; Punktestand BCD, niedrigste zwei Ziffern (Dezimalmodus!)
.const sc1     = $21                    ;                  mittlere zwei Ziffern
.const sc2     = $22                    ;                  hoechste zwei Ziffern  -> 6 Stellen
.const mask    = $23                    ; Eingabemaske dieses Bildes (IN_*)
.const t0      = $24                    ; Hilfsspeicher (kurzlebig, jede Routine beschreibt ihren Gebrauch)
.const t1      = $25
.const t2      = $26
.const t3      = $27
.const bcol    = $28                    ; zuletzt getroffener Stein: Spalte 0..17
.const brow    = $29                    ;                             Zeile 0..11
.const bidx    = $2a                    ;                             Index in die Steinkarte (Zeile * 18 + Spalte)
.const pxl     = $2b                    ; Ball-Position in ganzen px (aus get_px), Low-Byte (linke Ballkante)
.const pxh     = $2c                    ;                                          High-Byte
.const sp      = $2d                    ; Zeiger auf Bildschirm-Ziel (2 Byte, fuer (sp),y)
.const cp      = $2f                    ; Zeiger auf Farb-RAM-Ziel (2 Byte)
.const tp      = $31                    ; Zeiger auf Text (2 Byte)
.const txtcol  = $33                    ; Textfarbe fuer put_cell/puts
.const atgt    = $34                    ; Autoplay: Zielpunkt auf dem Schlaeger (8..23 px von der linken Kante)
.const ptA     = $35                    ; (ungenutzt)
.const ptB     = $36                    ; Hilfsspeicher von probe: Y-Versatz (0 = obere, 1 = untere Ballkante)
.const oxl     = $37                    ; Ball-X vor der Bewegung (zum Zuruecksetzen beim Abprall)
.const oxh     = $38
.const oyl     = $39                    ; Ball-Y vor der Bewegung
.const oyh     = $3a
.const npos    = $40                    ; Namenseingabe: Cursorposition 0..7
.const nprev   = $41                    ;   Eingabemaske des vorigen Bildes (Flankenerkennung)
.const nrep    = $42                    ;   Bilder bis zur naechsten Tastenwiederholung
.const ntl     = $43                    ;   Zaehler bis zur Auto-Bestaetigung (16 Bit, in Bildern), Low-Byte
.const nth     = $44                    ;   High-Byte
.const padw    = $45                    ; aktuelle Schlaegerbreite in px (16, 32 oder 48)
.const zstep   = $46                    ; Breite einer Abprallzone = padw / 8 (2, 4 oder 6 px)
.const pad2o   = $47                    ; Abstand des zweiten Schlaegersprites vom ersten (16 oder 24 px; 0 = nur ein Sprite)
.const capact  = $48                    ; 1 = eine Kapsel faellt
.const capxl   = $49                    ; Kapsel: linke Kante in px (16 Bit)
.const capxh   = $4a
.const capy    = $4b                    ;          obere Kante in px
.const captype = $4c                    ;          Art 0..7 (siehe Tabelle cap_letter)
.const tpad    = $4d                    ; Rest-Halbsekunden: Schlaegerbreite geaendert (breit/schmal)
.const trev    = $4e                    ;                    Steuerung umgekehrt
.const tspd    = $4f                    ;                    Geschwindigkeit geaendert
.const spdmod  = $50                    ; 0 = normal, 1 = schneller (+4 Stufen), 2 = langsamer (-4 Stufen)
.const tick    = $51                    ; Bilder bis zur naechsten halben Sekunde (zaehlt die Timer)
.const sticky  = $52                    ; 1 = naechster Schlaegertreffer faengt den Ball (Klebe-Extra)
.const demo    = $53                    ; 1 = Demo-Modus (Rechner spielt)
.const idl     = $54                    ; Titel: Bilder ohne Eingabe bis zur Demo (16 Bit)
.const idh     = $55
.const lt      = $56                    ; Titellogo: Buchstabe 0..8
.const lr      = $57                    ;             Zeile 0..4
.const lbits   = $58                    ;             Bitmuster der Zeile
.const lcol    = $59                    ;             Spalte des ersten Buchstabenpixels
.const cb      = $5a                    ; aktueller Ball (Schlitz 0..2), dessen Werte in bxl..vyh stehen
.const nact    = $5b                    ; Anzahl aktiver Baelle in diesem Bild
.const sprmsb  = $5c                    ; update_sprites: gesammeltes 9. X-Bit der Sprites
.const sprena  = $5d                    ; update_sprites: gesammelte Sprite-Freigabebits
; Titelbaelle (3 Byte je Eigenschaft, Schlitz 0..2 = Sprites 0, 4, 5), ausserhalb der Nullseite:
.const tbu     = $c130                  ; Titel-Baelle (3): X in Einheiten zu 2 px
.const tby     = $c133                  ;                    Y in px
.const tbdu    = $c136                  ;                    Schritt X (1 oder $FF)
.const tbdy    = $c139                  ;                    Schritt Y (1, 2 oder negativ)
.const bp      = $61                    ; Zeiger auf die Farbtabelle des aktuellen Hintergrund-Verlaufs (2 Byte, im Interrupt benutzt)
.const bflash  = $63                    ; Rest-Bilder des Randblitzens (0 = kein Blitz)
.const glg     = $64                    ; Glanzlicht: Position der laufenden Diagonale (0..30, danach Pause bis 99)
.const scx     = $65                    ; Lauftext: Feinversatz 7..0 (Bildpunkte)
.const stp     = $66                    ; Lauftext: Leseposition im Text
.const lph     = $67                    ; Titellogo: Farbphase
.const lcnt    = $68                    ; Titellogo: Bildzaehler fuer den Farbwechsel
.const nch     = $69                    ; Namenseingabe: getipptes Zeichen dieses Bildes (0 = keins)
.const ntyped  = $6a                    ;   1 = es wurde schon getippt (die Vorgabe ist geloescht)
.const capfr   = $6b                    ; Kapsel: Drehphase (Bildzaehler 0..23, je 4 Bilder eine Phase)
.const lkind   = $6c                    ; Levelart: 0 normal, 1 Bonus, 2 Boss
.const btime   = $6d                    ; Bonuslevel: Restsekunden
.const bsub    = $6e                    ; Halbsekundenzaehler fuer die Sekunden
.const cores   = $6f                    ; Bosslevel: verbliebene Kerne
.const batk    = $70                    ; Bosslevel: Halbsekunden bis zum naechsten Angriff
.const capforce = $71                   ; spawn_cap: erzwungene Kapselart + 1 (0 = zufaellig)
.const stq     = $72                    ; Lauftext: Leseposition als 16-Bit-Zeiger (der Text ist laenger als 256 Zeichen)
.const tpierce = $88                    ; Rest-Halbsekunden: Durchschlag aktiv (Ball zerstoert jeden zerstoerbaren Stein und prallt nicht ab)
.const tkrow   = $74                    ; Ticker: Bildschirmzeile
.const tkst    = $75                    ;         0 = warten, 1 = Effekt laeuft
.const tkw     = $76                    ;         Wartebilder
.const tkeff   = $77                    ;         Effekt 0..6
.const tkstep  = $78                    ;         Schritt
.const tkmsg   = $79                    ;         Index in der Textfolge
.const tkseq   = $7a                    ;         Folge 0 = Titel, 1 = Demo
.const tkph    = $7b                    ;         Farbphase (zaehlt Bilder)
.const tka     = $7c                    ;         Zeiger Bitmuster altes Zeichen (2 Byte)
.const tkb     = $7e                    ;         Zeiger Bitmuster neues Zeichen (2 Byte)
.const tkc     = $80                    ;         laufendes Zeichen 0..31 (Ticker-Zeichen) bei Zeichen- und Effektschleifen
.const tkr     = $81                    ;         laufende Pixelzeile 0..7
.const tkd     = $82                    ;         Versatz d der senkrechten Effekte (+1..+8 nach oben, -1..-8 nach unten)
.const tkt     = $83                    ;         Hilfswert (Zielindex im Puffer, Zeichen oder Schiebe-Zwischenwert)
.const tks     = $84                    ;         Hilfswert (Quellzeile s = r + d, neue Pixel oder Index des Aufloesens)
.const tklf    = $85                    ;         Schieberegister (LFSR) des Aufloesen-Effekts
.const tkdiv   = $86                    ;         Teilschritt (Viertel bzw. Haelfte) innerhalb eines Effektschritts
.const tkd2    = $87                    ;         Hilfswert von tk_hright
.const tkce    = $89                    ; tk_vert: Endzeichen (exklusiv) dieses Durchlaufs
.const tkre    = $8a                    ; tk_hleft/tk_hright: Endzeile (exklusiv) dieses Durchlaufs
.const tkrs    = $8b                    ;                     Startzeile
.const wfc     = $60                    ; wait_frames: Zaehler
.const shi     = $5f                    ; update_shots: Schussnummer (ueberlebt hit_brick)
.const tlas    = $5e                    ; Rest-Halbsekunden: Laser aktiv
.const dxo     = $3b                    ; Abtastversatz X der Ballkante (0 = links, 1 = rechts); in move_ball auch Versatz Y

.const MAP     = $c000                  ; Steinkarte im RAM: 216 Byte = 12 Zeilen x 18 Spalten, Index = Zeile * 18 + Spalte; Typ 0 = leer
                                        ; (Typen: 1..8 Farbsteine, 9 Silber, 10 Gold, 11 unzerstoerbar, 12 explosiv, 13 Ueberraschung,
                                        ; 16..23 Boss-Kern mit Typ - 15 Lebenspunkten; siehe brick_color/brick_pts)
; Ballspeicher: bis zu 3 Baelle (Mehrball).  Je Eigenschaft 3 Byte (Schlitz 0..2).  load_ball/save_ball kopieren einen Ball in die
; Nullseite (bxl..vyh), mit der die Bewegungs- und Kollisionsroutinen arbeiten.  Alle Adressen oberhalb von $C000 (RAM der Bank 0).
; Laserschuesse (2 Stueck, Sprites 6 und 7)
.const shact   = $c120                  ; Laserschuesse (2 Stueck): 1 = aktiv
.const shu     = $c122                  ;   X-Lage in Einheiten zu 2 px (px / 2, damit 8 Bit reichen: max. 150)
.const shy     = $c124                  ;   Y-Lage (Oberkante) in px
.const bact    = $c100                  ; 1 = Ball aktiv
.const b_xl    = $c103                  ; X-Position (1/64 px), Low-Byte
.const b_xh    = $c106                  ;                       High-Byte
.const b_yl    = $c109                  ; Y-Position (1/64 px)
.const b_yh    = $c10c
.const b_vxl   = $c10f                  ; Geschwindigkeit X (1/64 px je Bild, 16 Bit mit Vorzeichen)
.const b_vxh   = $c112
.const b_vyl   = $c115                  ; Geschwindigkeit Y (negativ = nach oben)
.const b_vyh   = $c118
.const SCR  = $8400                     ; Bildschirmspeicher (VIC-Bank 2: $8000-$BFFF; $D018 waehlt ihn per Bits 4-7 = 1)
.const SPTR = $87f8                     ; Sprite-Zeiger 0..7 (die letzten 8 Byte des Bildschirmspeichers: SCR + $3F8); Zeiger * 64 + $8000 = Daten
.const COLORRAM = $d800                 ; Farb-RAM (feste Adresse, E/A-Bereich; nur die unteren 4 Bit zaehlen)
.const PADY    = 184                    ; obere Kante des Schlaegers in px (Bildschirmzeile 23)
.const PADW    = 32                     ; Schlaegerbreite in px (Ueberbleibsel des Prototyps, ungenutzt: es gilt die Variable padw)
.const LIVES0  = 3                      ; Anzahl Baelle zu Spielbeginn

; Zeichencodes (Bildschirmcode) der eigenen Zeichen (Bitmuster: lib/gfx.inc.chars, Kopie nach $8A00 in copy_font)
.const CH_BRICK_L = 64                  ; linke Haelfte eines Steins
.const CH_BRICK_R = 65                  ; rechte Haelfte eines Steins
.const CH_WALL    = 66                  ; Wand
.const CH_BLOCK   = 67                  ; glaenzender Block fuer das Titellogo
.const CH_SPACE   = 32                  ; Leerzeichen

; Vorab: Hardware-Konstanten (VIC, SID, CIA, MMU; Quelle data/sys/c128/hardware.asm, darin data/sys/c64/hardware.asm) und KERNAL-Sprungtabelle
; (data/sys/c128/kernal.asm).  Die Bibliotheken Eingabe, Zufall, Klang und Bestenliste folgen weiter unten bzw. vor start.
#include <c128/hardware.asm>
#include <c128/kernal.asm>  ; (enthaelt eine Konstante SCREEN; unsere Bildschirmadresse heisst deshalb SCR)

.org $1c01                              ; Start des BASIC-Speichers des C128 (im C64-Modus waere es $0801)
.basicstub start                        ; erzeugt "10 SYS <start>" ($1C01-$1C0C), damit RUN das Programm startet; der Code beginnt bei $1C0D

; Bestenliste ZUERST einbinden: ihre Umschalt-Routinen (hs_io_on/off) muessen unter $4000 liegen (dort ist immer RAM; die Systemroms beginnen bei
; $4000 und wuerden bei der kurzen Umschaltung auf Konfiguration $00 den eigenen Code verdecken).  Auch copy_font (Zeichen-ROM lesen) steht darum vorn.
; Die Bibliothek liefert hs_init, hs_load, hs_save, hs_check, hs_insert, hs_default sowie hs_buf (127 Byte), hs_points, hs_level, hs_name
; (Zeropage $08-$0D, siehe Kopf von hiscore.asm).  Sie enthaelt auch den ersten ausfuehrbaren Code des Programms ($1C0D).
.include "lib/hiscore.asm"

; ---------------------------------------------------------------------------------------------------------------------
; Programmstart
; ---------------------------------------------------------------------------------------------------------------------
; start: Einsprung von "SYS" (nach RUN).  Der Reihe nach: Interrupts aus, CIA-Interrupts aus, Startgeraet merken, Zeichensatz kopieren
;   (dabei MMU-Konfiguration $3E), VIC/SID/Eingabe/Zufall einrichten, Vektoren im RAM setzen, Bestenliste laden, Interrupt frei.
;   Danach folgt ohne Rueckkehr die Hauptschleife (main).
start:  sei                             ; Interrupts aus: Wir brauchen weder die KERNAL-Uhr noch deren Tastaturabfrage; wir lesen
                                        ; die Tastatur selbst (input.asm) und takten uns ueber die Rasterzeile (wait_frame).
        lda #$7f                        ; Bit 7 = 0: die gesetzten Bits 0..6 LOESCHEN Interruptquellen der CIA
        sta CIA1_ICR                    ; CIA 1: alle Interruptquellen aus (Timer, Tastatur, ...)
        sta $dd0d                       ; CIA 2 (Interrupt-Steuerregister $DD0D, loest sonst den NMI aus) ebenso
        lda CIA1_ICR                    ; Lesen der Statusregister quittiert anstehende Meldungen
        lda $dd0d
        jsr hs_init                     ; Startgeraet fuer die Bestenliste merken ($BA)
        jsr copy_font                   ; Zeichensatz aus dem ROM ins RAM kopieren, eigene Zeichen einsetzen; danach MMU: alles RAM
        jsr vic_init                    ; VIC-II einrichten
        jsr sfx_init                    ; SID: Lautstaerke 15, Stimme 3 fuer Effekte
        jsr input_init                  ; Eingabezustand loeschen
.ifdef AUTOPLAY
        lda #1
        sta sfx_mute                    ; Demo-Modus: keine Soundeffekte (Vorgabe: nur die Musik spielt weiter)
.endif
        lda VIC_RASTER                  ; die aktuelle Rasterzeile ist als Startwert fuer den Zufall gut genug
        jsr rand_seed
        lda #<rti_only                  ; NMI- und Reset-Vektor im RAM (gelesen wird er nur bei Konfiguration $3E) zeigen auf ein blosses rti;
                                        ; ein echter Reset schaltet die MMU auf $00 zurueck und braucht den RAM-Vektor vermutlich nie
        sta $fffa
        sta $fffc
        lda #>rti_only
        sta $fffb
        sta $fffd
        jsr hs_load                     ; Bestenliste von der Diskette (die Systemroms werden kurz eingeschaltet); bei Fehlern bleibt die Standardliste
        cli                             ; jetzt laeuft der Rasterinterrupt (Farbbaender, Bildtakt)
        ; weiter in die Hauptschleife (main_jump/main folgen unmittelbar)

; ---------------------------------------------------------------------------------------------------------------------
; Hauptschleife: Titel -> Spiel -> Spielende -> Titel
; ---------------------------------------------------------------------------------------------------------------------
main_jump:
        jmp main                        ; Sprungbrett: bcs main_jump (Demo ende) erreicht main ueber die Branch-Reichweite
main:   jsr title                       ; Titelbild; kehrt zurueck, sobald Feuer gedrueckt wurde (oder die Demo beginnt, demo = 1)
        jsr new_game                    ; Punkte, Leben, Level, Geschwindigkeit zuruecksetzen
.level: jsr load_level                  ; Steine/Waende/Anzeige aufbauen, Ball auf den Schlaeger legen
.ball:  jsr play                        ; Rueckgabe in A: 0 = Level geschafft, 1 = Ball verloren, 2 = Abbruch (RUN/STOP)
        cmp #2
        beq main                        ; Abbruch: zurueck zum Titel
        cmp #1
        beq .lost
        lda demo
        beq +
        lda level
        cmp #4
        bcs main_jump                   ; Demo zeigt nur die Levels 1-5
+       inc level                       ; Level geschafft: naechstes Level, nach dem letzten wieder das erste
        lda level                       ; (die Geschwindigkeitsstufe bleibt erhalten: das Spiel wird immer schneller)
        cmp #level_count
        bcc .level
        lda #0
        sta level
        beq .level                      ; (immer: A = 0)
.lost:  lda lkind
        cmp #1
        bne .loselife
        jsr place_ball                  ; Bonuslevel: ein verlorener Ball kostet kein Leben, ein neuer haftet am Schlaeger
        jmp .ball
.loselife:
        dec lives
        beq .over                       ; keine Baelle mehr -> Spielende
        jsr reset_extras                ; Extras enden mit dem Ball
        jsr place_ball                  ; neuer Ball haftet auf dem Schlaeger
        jsr draw_hud                    ; Ballanzeige aktualisieren
        jmp .ball
.over:  jsr game_over
        jmp main
rti_only: rti                           ; Vektor fuer NMI/RESET im RAM: tut nichts (RESTORE-Taste und CIA-NMIs bleiben ohne Wirkung)

; ---------------------------------------------------------------------------------------------------------------------
; copy_font: kopiert den Zeichensatz (Zeichen-ROM, 2 KB) nach $8800 und setzt die eigenen Zeichen 64..67 ein.  Wird einmal von start gerufen.
;   Das Zeichen-ROM liegt im Adressraum des Prozessors bei $D000-$DFFF, ist dort aber nur sichtbar, wenn der E/A-Bereich abgeschaltet ist.
;   Darum stellt die Routine die MMU-Konfiguration $01 ein (Register $FF00): Bit 0 = 1 schaltet E/A aus, die Bits 1-5 = 0 lassen die
;   Systemroms (hier: das Zeichen-ROM im High-ROM-Bereich) sichtbar, Bits 6-7 = 0 = RAM-Bank 0.  Der Code dieser Routine liegt
;   unter $4000 und bleibt deshalb sichtbar.  Geschrieben wird immer ins RAM (auch unter ROM-Bereichen), das Ziel $8800 liegt also im
;   RAM, obwohl dort bei eingeschaltetem Mid-ROM ($8000-$BFFF) das BASIC-ROM zu lesen waere.  Danach gilt die Konfiguration $3E.
;   Nur die ersten 2 KB des Zeichen-ROMs (Grossbuchstaben/Grafik, Zeichen 0..255) werden kopiert.  Der VIC sieht das ROM unabhaengig
;   von der MMU nur in Bank 0 und 2 (an $1000-$1FFF bzw. $9000-$9FFF); wir benutzen aber den RAM-Zeichensatz bei $8800.
;   Zeichen n beginnt bei Zeichensatz + n * 8.  Nach der Kopie gilt: 1..26 = Buchstaben, 48..57 = Ziffern, 32 = Leerzeichen.
;   Eigene Zeichen (aus lib/gfx.inc.chars, alle MULTICOLOR): 64 = Stein links, 65 = Stein rechts, 66 = Wand, 67 = Block (Logo).
; Eingabe: keine.  Veraendert A, X, Y, sp, cp und die MMU-Konfiguration (danach $3E).  Interrupts muessen gesperrt sein (start: sei).
; ---------------------------------------------------------------------------------------------------------------------
copy_font:
        lda #$01                        ; MMU-Konfiguration $01: Systemroms sichtbar, E/A AUS -> an $D000 erscheint das Zeichen-ROM (Programm unter $4000 bleibt sichtbar)
        sta MMU_CR_FF00
        lda #<$d000                     ; sp = Quelle (Zeichen-ROM), cp = Ziel ($8800): Nullseitenzeiger fuer (sp),y / (cp),y
        sta sp
        lda #>$d000
        sta sp + 1
        lda #<$8800
        sta cp
        lda #>$8800
        sta cp + 1
        ldx #8                          ; 8 Seiten a 256 Byte = 2048 Byte (256 Zeichen zu 8 Byte)
        ldy #0
.cf:    lda (sp),y
        sta (cp),y
        iny
        bne .cf                         ; eine Seite
        inc sp + 1                      ; naechste Seite von Quelle und Ziel
        inc cp + 1
        dex
        bne .cf
        lda #MMUCFG_RAM0_IO             ; $3E: alles RAM der Bank 0 plus E/A (ab hier laeuft das Spiel ohne Systemroms)
        sta MMU_CR_FF00                 ; (E/A ist wieder sichtbar; die Vektoren $FFFA-$FFFF liegen jetzt im RAM und sind erst nach vic_init/start
                                        ; belegt: bis dahin darf kein Interrupt kommen - IRQ ist gesperrt, CIA-NMIs sind aus)
        ldx #31                         ; 4 Zeichen a 8 Byte (64..67), rueckwaerts kopieren
.cc:    lda gfx_chars,x
        sta $8a00,x                     ; $8800 + 64 * 8 = $8A00: Zeichen 64
        dex
        bpl .cc
        rts

; ---------------------------------------------------------------------------------------------------------------------
; vic_init: Grafikchip einrichten (einmal von start gerufen; Interrupts gesperrt).  Eingabe: keine.  Veraendert A.
;   $D030 Bit 0 = 0: 1 MHz.  Die 2-MHz-Betriebsart (nur C128) bleibt aus.
;   $DD00 Bit 0..1 waehlen die VIC-Bank, INVERTIERT: %11 = Bank 0 ($0000-$3FFF), %01 = Bank 2 ($8000-$BFFF; hier).  Die uebrigen Bits
;          (Seriellbus) bleiben erhalten.
;   $D018: Bits 4..7 = Bildschirmspeicher / 1 KB (innerhalb der Bank, hier 1 -> $8400), Bits 1..3 = Zeichensatz / 2 KB (hier 1 -> $8800):
;          Wert = (1 << 4) | (1 << 1) = $12.
;   Sprites: Die Zeiger stehen am Ende des Bildschirmspeichers ($87F8..$87FF); Zeiger * 64 + Bankanfang ($8000) = Adresse der Sprite-Daten.
;   Danach: Mehrfarb-Textmodus, Farben, Sprite-Einrichtung (Sprites 0-5 mehrfarbig, 6-7 einfarbig) und der Rasterinterrupt (Zeile = bar_line[0]).
; ---------------------------------------------------------------------------------------------------------------------
vic_init:
        lda VIC_CLOCK
        and #$fe
        sta VIC_CLOCK                   ; 1 MHz (die 2-MHz-Betriebsart stoert den VIC-II)
        lda $dd00
        and #$fc
        ora #$01                        ; Bits 0..1 = %01: VIC-Bank 2 ($8000-$BFFF)
        sta $dd00
        lda #$12                        ; Bildschirm $8400 = Bank + $0400 (1 << 4), Zeichensatz $8800 = Bank + $0800 (1 << 1)
        sta VIC_MEMORY
        lda #$1b                        ; Textmodus, Bildschirm an, 25 Zeilen, Y-Scroll 3 (Standardwert)
        sta VIC_CTRL1
        lda #$18                        ; 40 Spalten, MEHRFARB-Textmodus (Bit 4), X-Scroll 0.  Zeichen mit Farb-RAM-Wert >= 8 sind
        sta VIC_CTRL2                   ; Mehrfarbzeichen (4 breite Pixel zu 2 Bit), Werte 0..7 bleiben normale Hires-Zeichen (Text)
        lda #15                         ; Mehrfarbe 1 ($D022): hellgrau = Lichtkante der Steine
        sta VIC_BGCOLOR1
        lda #11                         ; Mehrfarbe 2 ($D023): dunkelgrau = Schattenkante
        sta VIC_BGCOLOR2
        lda #0
        sta VIC_BORDERCOLOR
        sta VIC_BGCOLOR0
        sta VIC_SPR_PRIORITY            ; Sprites vor dem Hintergrund
        sta VIC_SPR_ENABLE              ; erst im Spiel einschalten
        lda #$00                        ; Ball: Block $00 = $8000 (gfx_ball)
        sta SPTR
        lda #$01                        ; Schlaeger links: Block $01 = $8040 (gfx_padl; set_padw tauscht je nach Breite)
        sta SPTR + 1
        lda #$02                        ; Schlaeger rechts: Block $02 = $8080 (gfx_padr)
        sta SPTR + 2
        lda #$06                        ; Kapsel (Sprite 3): Block $06 = $8180 (gfx_cap; spawn_cap und update_sprites setzen die Drehphase)
        sta SPTR + 3
        lda #$07                        ; Sprites 6 und 7: Laserschuesse (einfarbig), Block $07 = $81C0 (gfx_shot)
        sta SPTR + 6
        sta SPTR + 7
        lda #7
        sta VIC_SPR6_COLOR
        sta VIC_SPR7_COLOR
        lda #$00                        ; Sprites 4 und 5: weitere Baelle (Mehrball) mit denselben Daten wie der erste Ball
        sta SPTR + 4
        sta SPTR + 5
        lda #1
        sta VIC_SPR4_COLOR
        sta VIC_SPR5_COLOR
        lda #%00111111                  ; Sprites 0-5 (Baelle, Schlaegerteile, Kapsel) MEHRFARBIG, die Laserschuesse (6, 7) einfarbig
        sta VIC_SPR_MULTICOLOR
        lda #15                         ; Sprite-Mehrfarbe 1 ($D025): hell (Lichtkante)
        sta VIC_SPR_MCOLOR0
        lda #6                          ; Sprite-Mehrfarbe 2 ($D026): dunkelblau (Schatten)
        sta VIC_SPR_MCOLOR1
        lda #1                          ; Ball weiss
        sta VIC_SPR0_COLOR
        lda #14                         ; Schlaeger hellblau
        sta VIC_SPR1_COLOR
        sta VIC_SPR2_COLOR
        ; Rasterinterrupt: Hintergrundfarbe in Baendern wechseln (siehe irq).  Es laeuft kein KERNAL: der Prozessor liest den Vektor $FFFE/$FFFF
        ; bei der MMU-Konfiguration $3E direkt aus dem RAM und springt in irq.  CIA-Interrupts sind aus, nur der Rasterinterrupt des VIC-II ist aktiv.
        lda #<irq
        sta $fffe                       ; Interruptvektor im RAM (alles RAM sichtbar)
        lda #>irq
        sta $ffff
        lda #0
        jsr set_theme                   ; Verlauf 0
        lda bar_line
        sta VIC_RASTER                  ; erste Interruptzeile (Bit 0..7 der Vergleichszeile)
        lda VIC_CTRL1
        and #$7f                        ; Bit 8 der Vergleichszeile ($D011 Bit 7) = 0 (alle Zeilen < 256)
        sta VIC_CTRL1
        lda #1
        sta VIC_IRQ_ENABLE              ; Rasterinterrupt freigeben (Bit 0 = Raster)
        lda #0
        sta band                        ; Bandzaehler auf 0 (naechster Interrupt = bar_line[0])
        sta vblank
        rts

; ---------------------------------------------------------------------------------------------------------------------
; Rasterinterrupt: Hintergrund-Farbbaender (Sonnenuntergang hinter dem Spielfeld) und Bildtakt.  IRQ-Vektor $FFFE (RAM), siehe vic_init.
;   Der Interrupt feuert nacheinander an den Zeilen bar_line[0..22] (band = Index); jedes Mal wird die Hintergrundfarbe $D021 auf die
;   naechste Bandfarbe gesetzt (sie wirkt bei allen Pixeln mit Bitmuster 00: Luecken zwischen den Steinen und der leere Spielraum).
;   Beim letzten Interrupt (Zeile 251, unterhalb des sichtbaren Bildes) kommt die Farbe wieder auf Schwarz fuer die Kopfzeile des
;   naechsten Bildes, $D016 wird auf den Normalwert $18 gestellt und vblank wird gesetzt: das Hauptprogramm darf das naechste Bild
;   berechnen (wait_frame).  Im letzten Band (Index 21) wird ausserdem der Feinversatz des Lauftextes ($D016 aus scrollreg) eingestellt.
;   Quittieren: Schreiben einer 1 nach $D019 loescht das Interruptflag.  Register A, X, Y werden selbst gerettet und am Ende zurueckgeholt
;   (RTI).  Benutzt bp (Zeiger auf die Farbtabelle des Levels), band, vblank und scrollreg.
; ---------------------------------------------------------------------------------------------------------------------
irq:    pha                             ; (ohne Systemrom ruft kein KERNAL die Routine: Register selbst retten)
        txa
        pha
        tya
        pha
        lda #1
        sta VIC_IRQ_FLAGS               ; Rasterinterrupt quittieren (Bit 0 loeschen)
        ldy band                        ; Y = Nummer des Bandes 0..22 (22 = Bildende)
        cpy #22
        beq .ifin                       ; Band 22 = Bildende (Zeile 251): kein Farbwechsel mehr
        lda bar_line,y                  ; X = Zeile T des Bandes, in der der Interrupt ausgeloest wurde
        tax
.iw:    cpx VIC_RASTER                  ; STABILE FARBKANTE: nicht sofort schreiben (der Interrupt kommt je nach laufendem Befehl um bis zu
        bcs .iw                         ; 7 Takte versetzt), sondern warten, bis die Zeile T+1 beginnt (Schleife: 7 Takte; solange X >= Zeile).
                                        ; Dann liegt der Schreibzugriff in den ersten Takten der Zeile: der linke Rand verdeckt ihn, die Kante
                                        ; zwischen zwei Baendern ist eine saubere Waagerechte.  (Haette der Interrupt sich bis nach T+1
                                        ; verspaetet, endet die Schleife sofort.)  Die Bandzeilen liegen so, dass T+1 keine "Bad Line" ist.
        lda (bp),y                      ; Farbe dieses Bandes aus der Verlaufstabelle des Levels
        sta VIC_BGCOLOR0
        cpy #21
        bne .i4
        lda scrollreg                   ; letztes Band (T = 238): Feinversatz des Lauftextes ab Zeile 239 (die Textzeile 24 beginnt bei 243;
        sta VIC_CTRL2                   ; die Zeilen 239-242 gehoeren noch zur leeren Bildschirmzeile 23)
        jmp .i4
.ifin:  lda #$18                        ; Zeile 251: Versatz wieder 0 (Mehrfarbtext, 40 Spalten), Hintergrund schwarz, Bildtakt
        sta VIC_CTRL2
        lda (bp),y
        sta VIC_BGCOLOR0
        lda #1
        sta vblank
.i4:    iny
        cpy #23
        bcc +
        ldy #0
+       sty band
        lda bar_line,y
        sta VIC_RASTER                  ; naechste Interruptzeile
        pla
        tay
        pla
        tax
        pla
        rti

; Interruptzeilen: 22 Baender alle 8 Zeilen von 70 bis 238 (jeweils 6 mod 8: die folgende Zeile ist keine Bad Line) und 251 (Bildende).
;   Rasterzeilen in der Zaehlung des VIC: das Textfenster (25 Zeilen, Y-Scroll 3) umfasst die Zeilen 51-250; 251 liegt direkt darunter.
bar_line: .byte 70, 78, 86, 94, 102, 110, 118, 126, 134, 142, 150, 158, 166, 174, 182, 190, 198, 206, 214, 222, 230, 238, 251
; Verlaufstabellen (je 23 Farben, Index 21 = letztes Band, Index 22 = Schwarz fuer die Kopfzeile des naechsten Bildes):
;   0 Daemmerung (blau - violett - rot), 1 Wald (grau - gruen - braun), 2 Glut (braun - rot - orange), 3 Nacht (graue Streifen)
bar_themes:
        .byte 0, 6, 6, 6, 6, 6, 6, 6, 6, 6, 4, 4, 4, 4, 4, 2, 2, 2, 2, 2, 2, 0, 0
        .byte 0, 11, 11, 11, 5, 5, 5, 5, 5, 5, 5, 9, 9, 9, 9, 9, 9, 9, 0, 0, 0, 0, 0
        .byte 0, 9, 9, 9, 2, 2, 2, 2, 2, 2, 8, 8, 8, 8, 8, 8, 2, 2, 2, 2, 2, 0, 0
        .byte 0, 11, 11, 0, 0, 11, 11, 0, 0, 11, 11, 0, 0, 11, 11, 0, 0, 11, 11, 0, 0, 0, 0
scrollreg: .byte $18                    ; Wert fuer $D016 ab Zeile 239 (Mehrfarbtext + 40 Spalten + Feinversatz 0..7 des Lauftextes); im Spiel $18

; set_theme: Verlauf A (0..3) waehlen: bp = bar_themes + 23 * A.   Veraendert A, X, bp (2 Byte).  Aufruf: vic_init (A = 0), load_level (Level mod 4).
set_theme:
        tax
        lda #<bar_themes
        sta bp
        lda #>bar_themes
        sta bp + 1
        cpx #0
        beq .st9
.st1:   clc
        lda bp
        adc #23
        sta bp
        bcc +
        inc bp + 1
+       dex
        bne .st1
.st9:   rts

; flash: Rand kurz in Farbe A faerben fuer X Bilder (fx_frame zaehlt herunter und stellt danach Schwarz her).
;   Eingabe: A = Farbe 0..15, X = Anzahl Bilder.  Veraendert nur $D020 und bflash.
flash:  sta VIC_BORDERCOLOR
        stx bflash
        rts

band:     .byte 0               ; Nummer des Bandes (Index in bar_line), den der naechste Interrupt bearbeitet (0..22)
vblank:   .byte 0               ; 1 = der Bildwechsel (Zeile 251) ist erreicht; wait_frame wartet darauf und loescht es


; ---------------------------------------------------------------------------------------------------------------------
; Bildschirm-Hilfen
; ---------------------------------------------------------------------------------------------------------------------
; clear_screen: 1000 Zeichen Bildschirmspeicher = Leerzeichen, Farb-RAM = weiss.   Veraendert A, X.
;   Vier Zugriffe je Schleifendurchlauf (X = 0..255): drei volle Seiten und die letzte ab $86E8 ($8400 + $2E8): $86E8..$87E7 sind genau
;   die letzten 256 Zellen vor den Sprite-Zeigern ($87F8) - die Zeiger bleiben unberuehrt.
clear_screen:
        ldx #0
.cs:    lda #CH_SPACE
        sta SCR,x
        sta SCR + $100,x
        sta SCR + $200,x
        sta SCR + $2e8,x
        lda #1
        sta COLORRAM,x
        sta COLORRAM + $100,x
        sta COLORRAM + $200,x
        sta COLORRAM + $2e8,x
        inx
        bne .cs
        rts

; set_rowptr: sp (Bildschirm) und cp (Farb-RAM) auf Zeile Y, Spalte X setzen.
;   Adresse = $8400 + Zeile * 40 + Spalte (Tabellen rowlo/rowhi); das Farb-RAM liegt $5400 hoeher ($D800 - $8400) bei gleichem Low-Byte.
; Eingabe: X = Spalte 0..39, Y = Zeile 0..24.  Ausgabe: sp, cp (je 2 Byte) zeigen auf die Zelle.  Veraendert A.
set_rowptr:
        txa
        clc
        adc rowlo,y
        sta sp
        sta cp
        lda rowhi,y
        adc #0                          ; Uebertrag aus dem Low-Byte
        sta sp + 1
        clc
        adc #$54                        ; Farb-RAM $D800 liegt $5400 ueber dem Bildschirm $8400
        sta cp + 1
        rts

; put_cell: Zeichen A (Bildschirmcode) mit Farbe txtcol (Farb-RAM-Wert, Bit 3 = Mehrfarbe) an Spalte X, Zeile Y ausgeben.
;   Veraendert A, X, Y, sp, cp.
put_cell:
        pha
        jsr set_rowptr
        ldy #0
        pla
        sta (sp),y
        lda txtcol
        sta (cp),y
        rts

; puts: Text ab (tp), mit 0 abgeschlossen (Bildschirmcodes, siehe .screen), an Spalte X, Zeile Y in Farbe txtcol ausgeben.
;   Veraendert A, X, Y, t1..t3, sp, cp.  Hoechstens 255 Zeichen.
puts:   stx t2                          ; t2 = aktuelle Spalte
        sty t3                          ; t3 = Zeile
        ldy #0
        sty t1                          ; t1 = Index im Text
.pl:    ldy t1
        lda (tp),y
        beq .pd                         ; 0 = Ende
        ldx t2
        ldy t3
        jsr put_cell
        inc t2
        inc t1
        bne .pl                         ; (Text kuerzer als 256 Zeichen)
.pd:    rts

; put_bcd: ein BCD-Byte A als zwei Ziffern an Spalte X (Zehner) und X + 1 (Einer), Zeile Y ausgeben (Ziffernzeichen = $30 + Wert), Farbe txtcol.
;   Veraendert A, X, Y, t0, t2, t3, sp, cp.
put_bcd:
        sta t0
        stx t2
        sty t3
        lsr                             ; obere Ziffer: High-Nibble nach unten
        lsr
        lsr
        lsr
        clc
        adc #$30
        jsr put_cell                    ; (X und Y sind noch gueltig)
        lda t0
        and #$0f                        ; untere Ziffer
        clc
        adc #$30
        ldx t2
        inx                             ; eine Spalte weiter rechts
        ldy t3
        jmp put_cell

; Zeilenadressen des Bildschirms: $8400 + 40 * Zeile (Low- und High-Byte), Zeile 0..24.
rowlo:  .byte $00, $28, $50, $78, $a0, $c8, $f0, $18, $40, $68, $90, $b8, $e0, $08, $30, $58, $80, $a8, $d0, $f8, $20, $48, $70, $98, $c0
rowhi:  .byte $84, $84, $84, $84, $84, $84, $84, $85, $85, $85, $85, $85, $85, $86, $86, $86, $86, $86, $86, $86, $87, $87, $87, $87, $87

; wait_frame: wartet auf den Anfang des Bildwechsels (Rasterzeile 251, unterhalb des sichtbaren Bildes).  So laeuft das Spiel
;   mit 50 Hz (PAL) und Sprites/Bildschirm werden aktualisiert, wenn der Strahl sie nicht gerade zeichnet.  Veraendert A.
;   Der Rasterinterrupt (irq) setzt vblank; kommt der Aufruf zu spaet (Bild schon vorbei), kehrt die Routine sofort zurueck (ein Bild
;   wird dann ausgelassen: das Spiel ruckelt, TESTFPS zeigt es).
wait_frame:
.w1:    lda vblank
        beq .w1                         ; der Rasterinterrupt setzt vblank in Zeile 251
        lda #0
        sta vblank
        rts

; wait_frames: X Bilder warten (X = 1..255); dabei laeuft der Klangeffekt weiter (sfx_update).   Veraendert A, X, wfc.
wait_frames:
        stx wfc                         ; Zaehler im Speicher: sfx_update veraendert X (Zugriff auf sfx_id)
.wf:    jsr wait_frame
        jsr sfx_update
        dec wfc
        bne .wf
        rts

; HINWEIS ZU .screen: Der Assembler wandelt KLEINBUCHSTABEN in die Bildschirmcodes 1..26 (= Grossbuchstaben im Zeichensatz, den wir
; aus dem ROM kopieren); Grossbuchstaben ergaeben 65..90 (Grafikzeichen bzw. unsere eigenen Zeichen).  Deshalb stehen alle
; Anzeigetexte hier in Kleinbuchstaben und erscheinen auf dem Bildschirm in GROSSBUCHSTABEN.
; ---------------------------------------------------------------------------------------------------------------------
; Titelbild
; ---------------------------------------------------------------------------------------------------------------------
; title: Titelbild aufbauen und auf den Spielstart warten.   Eingabe: keine.
;   Rueckkehr: bei Feuer (demo = 0: neues Spiel) oder nach 30 s ohne Eingabe (demo = 1: der Rechner spielt eine Demo).  Mit AUTOPLAY
;   oder TESTLOSE kehrt sie sofort zurueck.  Zeitplan: erst 40 Bilder Schonfrist (Eingaben zaehlen noch nicht), dann je Bild Musik,
;   Titelbaelle, Logo/Lauftext, Ticker und Eingabeabfrage; jede andere Eingabe als Feuer startet die 30 s neu.
;   Bildaufbau: Titelbaelle (Sprites), Logo (Zeilen 3-7), Ticker (Zeile 10), Bestenliste "Top 5" (Zeilen 12-17), Bedienhinweise
;   (Zeilen 20-22), Lauftext (Zeile 24).  Ruft clear_screen, hs_buf wird nur gelesen.  Veraendert alles (A, X, Y, t0..t3, tp, txtcol, ...).
title:  lda #0
        sta VIC_BORDERCOLOR
        jsr title_balls_init            ; drei Baelle huepfen ueber den Titel (Sprites 0, 4, 5)
        jsr clear_screen
        jsr draw_logo                   ; "PRALLBALL" aus glaenzenden Bloecken
        jsr title_scroll_init           ; Lauftext in der letzten Bildschirmzeile
        lda #10                         ; Untertitel: Ticker in Zeile 10 mit zwei wechselnden Texten
        ldx #0
        jsr ticker_init
        lda #7                          ; gelb
        sta txtcol
        lda #<t_best
        sta tp
        lda #>t_best
        sta tp + 1
        ldx #14
        ldy #12
        jsr puts
        jsr draw_top5                   ; die besten fuenf Eintraege der Liste
        lda #3                          ; tuerkis
        sta txtcol
        lda #<t_ctrl
        sta tp
        lda #>t_ctrl
        sta tp + 1
        ldx #4
        ldy #20
        jsr puts
        lda #<t_keys
        sta tp
        lda #>t_keys
        sta tp + 1
        ldx #7
        ldy #21
        jsr puts
        lda #1
        sta txtcol
        lda #<t_start
        sta tp
        lda #>t_start
        sta tp + 1
        ldx #10
        ldy #22
        jsr puts
        lda #0                          ; normaler Titel: kein Demo-Modus, Effekte an
        sta demo
        sta sfx_mute
        ldx #<song                      ; Titelmusik von vorn starten (Liedformat: examples/c128/prallball/lib/sound.asm)
        ldy #>song
        jsr mus_init
.ifdef AUTOPLAY
        lda #1                          ; Testfassung: der Rechner spielt von Anfang an und ohne Ende
        sta demo
        sta sfx_mute
.else
.ifndef TESTLOSE
        jsr input_wait_release          ; ein noch gedrueckter Knopf (vom vorigen Spiel) zaehlt nicht
        lda #40                         ; Schonfrist: in den ersten 0,8 s nach dem Erscheinen des Titels zaehlt keine Eingabe
        sta idl                         ; (Zaehler im Speicher, NICHT in X: mus_tick und title_balls_tick veraendern X!)
.gr:    jsr wait_frame                  ; (ein vom Ende der letzten Runde noch nachwirkender Tastendruck startet kein neues Spiel)
        jsr mus_tick
        jsr title_balls_tick
        jsr title_fx
        jsr ticker_tick
        dec idl
        bne .gr
        jsr input_wait_release
        lda #<1500
        sta idl                         ; 30 s Leerlauf (1500 Bilder bei 50 Hz) -> Demo-Modus
        lda #>1500
        sta idh
.tw:    jsr wait_frame
        jsr mus_tick                    ; Musik einen Bildschritt weiter
        jsr title_balls_tick
        jsr title_fx
        jsr ticker_tick
        jsr input_poll
        sta mask
        and #IN_FIRE
        bne .start                      ; Feuer: Spiel starten
        lda mask
        beq .idle
        lda #<1500                      ; andere Eingabe: Wartezeit neu starten
        sta idl
        lda #>1500
        sta idh
        bne .tw
.idle:  lda idl
        ora idh
        beq .demo                       ; Zeit abgelaufen
        lda idl
        bne +
        dec idh
+       dec idl
        jmp .tw
.demo:  lda #1                          ; Demo-Modus: der Rechner spielt, die Soundeffekte schweigen (die Musik laeuft weiter)
        sta demo
        sta sfx_mute
        jmp .go                         ; die Musik laeuft in der Demo weiter
.start: jsr mus_stop                    ; Spielbeginn: die Musik schweigt (nur auf dem Titel und in der Demo)
.go:
.endif
.endif
        rts

; title_fx: Zierde des Titels, einmal je Bild: der Lauftext (title_scroll_tick) und - alle 4 Bilder ein Schritt - der Farbwechsel im Logo:
;   Die Logozeilen 3..7 bekommen die Farben rainbow[(lph + Zeile) and 7] im Farb-RAM (Spalten 2..37).   Veraendert A, X, Y, t1, txtcol, cp, sp.
title_fx:
        jsr title_scroll_tick
        inc lcnt
        lda lcnt
        and #3
        bne .tf9
        inc lph                         ; Farbphase weiter
        lda #0
        sta t1                          ; Logozeile 0..4
.tf1:   lda lph
        clc
        adc t1
        and #7
        tax
        lda rainbow,x                   ; Farb-RAM-Wert (8 + Farbe) dieser Zeile
        sta txtcol
        ldx #2
        lda t1
        clc
        adc #3
        tay                             ; Bildschirmzeile = 3 + Logozeile
        jsr set_rowptr                  ; cp zeigt auf das Farb-RAM (Zeile, Spalte 2)
        ldy #0
        lda txtcol
.tf2:   sta (cp),y                      ; Spalten 2..37 einfaerben (nur die Bloecke sind sichtbar)
        iny
        cpy #36
        bne .tf2
        inc t1
        lda t1
        cmp #5
        bne .tf1
.tf9:   rts
rainbow: .byte 10, 15, 13, 11, 14, 12, 9, 15            ; rot, gelb, gruen, tuerkis, blau, violett, weiss, gelb (je 8 + Farbe)

; Lauftext in Bildschirmzeile 24: weicher Versatz ueber $D016 (der Rasterinterrupt im letzten Band, Zeile 238, setzt ihn ab Zeile 239, siehe irq);
;   nach 8 Bildpunkten rutscht die Zeile um ein Zeichen nach links und rechts kommt das naechste Zeichen des Textes nach.
; title_scroll_init: Lauftext starten: Feinversatz scx = 7, scrollreg = $1F, die ersten 40 Textzeichen (gelb) in Zeile 24, stq hinter ihnen.
;   Veraendert A, X, scx, scrollreg, stq.
title_scroll_init:
        lda #7
        sta scx
        lda #$1f
        sta scrollreg
        ldx #39
.sc1:   lda scroll_text,x               ; die ersten 40 Zeichen des Textes
        sta SCR + 24 * 40,x
        lda #7                          ; gelb (Farbe < 8: normaler Text, kein Mehrfarbzeichen)
        sta COLORRAM + 24 * 40,x
        dex
        bpl .sc1
        lda #<(scroll_text + 40)        ; Leseposition hinter den ersten 40 Zeichen
        sta stq
        lda #>(scroll_text + 40)
        sta stq + 1
        rts

; title_scroll_tick: einmal je Bild (aus title_fx): Feinversatz scx um 1 verringern (7..0 = 1 Bildpunkt je Bild nach links).  Bei Unterlauf
;   Zeile 24 um ein Zeichen nach links schieben, rechts (Spalte 39) das naechste Zeichen nachschieben (stq, 16 Bit; Textende 0 = von vorn).
;   scrollreg = $18 | scx ist der Wert, den irq ab Zeile 239 in $D016 schreibt.   Veraendert A, X, Y, scx, scrollreg, stq.
title_scroll_tick:
        dec scx
        bpl .sk1                        ; noch im Feinversatz
        lda #7
        sta scx
        ldx #0
.sk2:   lda SCR + 24 * 40 + 1,x         ; Zeile um ein Zeichen nach links schieben
        sta SCR + 24 * 40,x
        inx
        cpx #39
        bne .sk2
        ldy #0
        lda (stq),y                     ; naechstes Zeichen
        bne .sk3
        lda #<scroll_text               ; Textende (0): von vorn
        sta stq
        lda #>scroll_text
        sta stq + 1
        lda scroll_text
.sk3:   sta SCR + 24 * 40 + 39
        inc stq                         ; 16-Bit-Zeiger weiter
        bne .sk1
        inc stq + 1
.sk1:   lda scx
        ora #$18                        ; Mehrfarbtext, 40 Spalten, plus Feinversatz
        sta scrollreg
        rts

; Lauftext (Bildschirmcodes: Kleinbuchstaben erscheinen als Grossbuchstaben), 0-terminiert; beginnt mit 40 Leerzeichen (leere Zeile).
;   Er beschreibt die Muenzenfarben, passend zu cap_color.
scroll_text:
        .screen "                                        prallball - zerstoere alle steine mit ball und schlaeger!   fange die muenzen, die aus zerstoerten steinen fallen:  hellblau = breiter schlaeger,  hellrot = schmaler schlaeger,  rot = umgekehrte steuerung,  gruen = ball langsamer,  gelb = ball schneller,  orange = ball klebt,  weiss = extraball,  tuerkis = bonuspunkte,  violett = mehrball,  hellgruen = laser,  grau = durchschlag.   viel spass!                "
        .byte 0

; title_balls_init: Startwerte der drei Titelbaelle (Schlitz 0..2 = Sprites 0, 4, 5) aus tb_u0/tb_y0/tb_du/tb_dy nach tbu/tby/tbdu/tbdy kopieren
;   und die Sprites 0, 4, 5 einschalten (Schlaeger/Kapsel aus).   Veraendert A, X, VIC_SPR_ENABLE.
title_balls_init:
        ldx #2
.tbi:   lda tb_u0,x
        sta tbu,x
        lda tb_y0,x
        sta tby,x
        lda tb_du,x
        sta tbdu,x
        lda tb_dy,x
        sta tbdy,x
        dex
        bpl .tbi
        lda #1 | 16 | 32
        sta VIC_SPR_ENABLE              ; nur die drei Baelle (Schlaeger/Kapsel aus)
        rts

; title_balls_tick: einmal je Bild: die drei Titelbaelle um (tbdu, tbdy) weiterbewegen, an den Raendern umkehren und die Sprite-Register setzen.
;   Position X in Einheiten zu 2 px (u, 8 Bit reicht fuer das ganze Bild), Y in px; Sprite-X = 2 * u + 24 (9 Bit, das 9. Bit sammelt t3 fuer
;   $D010), Sprite-Y = y + 50.  Grenzen: u 6..147 (px 12..294), y 60..204.  Reine Zierde.   Veraendert A, X, Y, t0, t1, t3, VIC_SPR_X_MSB.
title_balls_tick:
        ldx #0
        stx t3                          ; gesammeltes 9. X-Bit
.tb:    lda tbu,x                       ; X bewegen, an den Raendern (u 6 und 148) umkehren
        clc
        adc tbdu,x
        sta tbu,x
        cmp #6
        bcc .tbf1
        cmp #148
        bcc .tbx
.tbf1:  sec                             ; Schritt umkehren: 0 - Schritt
        lda #0
        sbc tbdu,x
        sta tbdu,x
.tbx:   lda tby,x                       ; Y bewegen, Grenzen 60 und 205
        clc
        adc tbdy,x
        sta tby,x
        cmp #60
        bcc .tbf2
        cmp #205
        bcc .tby
.tbf2:  sec
        lda #0
        sbc tbdy,x
        sta tbdy,x
.tby:   lda tbu,x                       ; Sprite-X = 2 * u + 24 (9 Bit)
        asl
        sta t0
        lda #0
        rol
        sta t1
        clc
        lda t0
        adc #24
        sta t0
        bcc +
        inc t1
+       ldy ball_reg,x
        lda t0
        sta VIC_SPR0_X,y
        lda t1
        beq +
        lda ball_bit,x
        ora t3
        sta t3
+       lda tby,x
        clc
        adc #50
        sta VIC_SPR0_Y,y
        inx
        cpx #3
        bne .tb
        lda t3
        sta VIC_SPR_X_MSB
        rts

; Startwerte der drei Titelbaelle: X (Einheiten zu 2 px), Y (px), Schritt X je Bild (1 oder $FF = -1), Schritt Y je Bild (1, 2 oder $FF = -1)
tb_u0: .byte 30, 80, 120
tb_y0: .byte 90, 140, 70
tb_du: .byte 1, $ff, 1
tb_dy: .byte 1, 2, $ff

; Texte des Titelbilds (Bildschirmcodes, 0-terminiert; Kleinbuchstaben erscheinen gross).  t_sub wird nicht benutzt (der Untertitel kommt aus tk_msgs).
t_sub:  .screen "pralle action fuer den c128"
        .byte 0
t_best: .screen "bestenliste"
        .byte 0
t_ctrl: .screen "joystick port 2: links / rechts"
        .byte 0
t_keys: .screen "tasten z x q a, leertaste"
        .byte 0
t_start: .screen "feuer = start"
        .byte 0

; draw_logo: "PRALLBALL" in einer 3 x 5 Pixel grossen Blockschrift; jeder Pixel ist ein Bildschirmzeichen (CH_BLOCK).
;   Die Zeilen 3..7 des Bildschirms bekommen je eine Regenbogenfarbe (Farb-RAM = 8 + Farbe, Mehrfarbzeichen).  Buchstabe i beginnt in
;   Spalte 2 + 4 * i (3 Spalten Buchstabe + 1 Spalte Abstand) - zusammen 36 Spalten, mittig im 40-Spalten-Bild.
;   Veraendert alles (A, X, Y, t0, t2, t3, lt, lr, lbits, lcol, txtcol).
draw_logo:
        lda #0
        sta lt
.dl:    ldx lt
        lda logo_seq,x                  ; Buchstabennummer
        sta t0
        asl
        asl
        clc
        adc t0                          ; * 5 = Index der ersten Zeile in logo_font
        sta t0
        lda lt
        asl
        asl
        clc
        adc #2
        sta lcol                        ; Spalte des Buchstabenanfangs
        lda #0
        sta lr
.dr:    lda lr
        clc
        adc t0
        tax
        lda logo_font,x                 ; 3 Bit: Bit 2 = linker Pixel
        sta lbits
        ldx lr
        lda logo_color,x
        sta txtcol                      ; Farbe dieser Bildschirmzeile
        ldy #0                          ; Y = Pixel 0..2
.dp:    asl lbits                       ; (das 3-Bit-Muster liegt in Bit 2..0; durch Schieben ueber Bit 3 laufen)
        lda lbits
        and #8
        beq .dn
        tya
        clc
        adc lcol
        tax                             ; Spalte
        stx t2
        sty t3
        lda lr
        clc
        adc #3
        tay                             ; Zeile = 3 + lr
        lda #CH_BLOCK
        jsr put_cell
        ldy t3
.dn:    iny
        cpy #3
        bne .dp
        inc lr
        lda lr
        cmp #5
        bne .dr
        inc lt
        lda lt
        cmp #9
        bne .dl
        rts

logo_seq:   .byte 0, 1, 2, 3, 3, 4, 2, 3, 3          ; P R A L L B A L L (Buchstaben 0..4 = P, R, A, L, B)
logo_color: .byte 10, 15, 13, 11, 14                  ; Zeilenfarben: rot, gelb, gruen, tuerkis, blau (je 8 + Farbe)
; Blockschrift 3 x 5: je Buchstabe 5 Byte (eine Zeile je Byte, Bit 2 = links)
logo_font:  .byte %111, %101, %111, %100, %100        ; P
            .byte %111, %101, %110, %101, %101        ; R
            .byte %111, %101, %111, %101, %101        ; A
            .byte %100, %100, %100, %100, %111        ; L
            .byte %110, %101, %110, %101, %110        ; B

; draw_top5: die ersten fuenf Bestenlisten-Eintraege (Platz in Spalte 9, Name ab Spalte 12, Punkte BCD ab Spalte 22) in den Zeilen 13..17.
;   Eintrag k liegt bei hs_buf + HS_FIRST + 12 * k: 8 Zeichen Name (PETSCII-Grossbuchstaben, fuer Buchstaben = ASCII), dann Punkte
;   lo/mid/hi (BCD, je zwei Ziffern), dann Level.   Veraendert alles (A, X, Y, t0..t3, lt, lr, txtcol).
draw_top5:
        lda #0
        sta lt                          ; lt = Rang 0..4
.t5:    lda #1
        sta txtcol
        lda lt
        clc
        adc #13
        sta lr                          ; lr = Bildschirmzeile
        lda lt
        clc
        adc #$31                        ; Platzziffer 1..5 (Bildschirmcode der Ziffer = $30 + n)
        ldx #9
        ldy lr
        jsr put_cell
        lda lt                          ; Offset des Eintrags in hs_buf: 5 + 12 * lt
        asl
        asl
        sta t0                          ; 4 * lt
        asl
        clc
        adc t0                          ; 8 * lt + 4 * lt = 12 * lt
        clc
        adc #HS_FIRST
        sta t1
        ldx #0
.t5n:   stx t2                          ; Name: 8 Zeichen
        lda t1
        clc
        adc t2
        tay
        lda hs_buf,y                    ; PETSCII/ASCII-Zeichen des Namens
        cmp #64
        bcc +
        sec
        sbc #64                         ; Buchstabe -> Bildschirmcode
+       pha
        txa
        clc
        adc #12                         ; Spalte 12..19
        tax
        ldy lr
        pla
        jsr put_cell
        ldx t2
        inx
        cpx #8
        bne .t5n
        lda t1                          ; Punkte: BCD-Bytes hi, mid, lo an den Spalten 22, 24, 26
        clc
        adc #10
        tay
        lda hs_buf,y
        ldx #22
        ldy lr
        jsr put_bcd
        lda t1
        clc
        adc #9
        tay
        lda hs_buf,y
        ldx #24
        ldy lr
        jsr put_bcd
        lda t1
        clc
        adc #8
        tay
        lda hs_buf,y
        ldx #26
        ldy lr
        jsr put_bcd
        inc lt
        lda lt
        cmp #5
        bne .t5
        rts

; ---------------------------------------------------------------------------------------------------------------------
; Spielstart und Level aufbauen
; ---------------------------------------------------------------------------------------------------------------------
; new_game: alles auf Anfang: Punkte 0, Level 0 (mit STARTLEVEL: dieses), Geschwindigkeitsstufe 0, Schlaegertreffer 0, Schlaeger mittig,
;   3 Baelle (LIVES0), Lauftext-Versatz aus, Glanzlicht und Randblitz aus, Autoplay-Zielpunkt 8; setzt am Ende alle Extras zurueck
;   (reset_extras per JMP).   Veraendert A (und was reset_extras/set_padw veraendern).
new_game:
        lda #0
        sta sc0
        sta sc1
        sta sc2
        sta level
        sta speed
        sta hits
        sta padh
        lda #144                        ; Schlaeger mittig: 16 + (288 - 32) / 2
        sta padl
        lda #$18
        sta scrollreg                   ; kein Feinversatz im Spiel
.ifdef TESTFPS
        lda #0
        sta $c200
        sta $c201
.endif
.ifdef STARTLEVEL
        lda #STARTLEVEL                 ; Testfassung: mit diesem Level (0-basiert) beginnen
        sta level
.endif
        lda #0
        sta glg
        sta bflash
        lda #LIVES0
        sta lives
        lda #8
        sta atgt
        jmp reset_extras

; reset_extras: alle Extras aus (neues Spiel, neues Level, Ball verloren): Kapsel weg, Timer 0, Schlaeger 32 px, Laserschuesse weg,
;   Halbsekundenzaehler tick = 25.  (Mit TESTLASER bleibt der Laser 240 Halbsekunden an.)   Faellt in set_padw durch.   Veraendert A.
reset_extras:
        lda #0
        sta capact
        sta tpad
        sta trev
        sta tspd
        sta spdmod
        sta sticky
        sta tlas
        sta tpierce
        sta shact
        sta shact + 1
        lda #25
        sta tick
.ifdef TESTLASER
        lda #240                        ; Testfassung: Laser immer an
        sta tlas
.endif
        lda #32                         ; weiter mit set_padw

; set_padw: Schlaegerbreite A (16, 32 oder 48) setzen und die Sprites passend einrichten.
;   32 px = zwei 16-px-Haelften (Sprite-Zeiger $01, $02 = gfx_padl/gfx_padr); 48 px = zwei 24-px-Haelften ($03, $04 = gfx_pad24l/r);
;   16 px = ein Sprite ($05 = gfx_pad16), das zweite wird abgeschaltet (pad2o = 0).
;   zstep = Breite / 8 ist die Zonenbreite fuer den Abprallwinkel (8 Zonen ueber die ganze Breite).
;   Eingabe: A = Breite.  Ausgabe: padw, zstep, pad2o, SPTR + 1, SPTR + 2.  Veraendert A.
set_padw:
        sta padw
        lsr
        lsr
        lsr
        sta zstep
        lda padw
        cmp #48
        beq .sw48
        cmp #16
        beq .sw16
        lda #$01
        sta SPTR + 1
        lda #$02
        sta SPTR + 2
        lda #16
        sta pad2o
        rts
.sw48:  lda #$03
        sta SPTR + 1
        lda #$04
        sta SPTR + 2
        lda #24
        sta pad2o
        rts
.sw16:  lda #$05
        sta SPTR + 1
        lda #0
        sta pad2o                       ; 0 = zweites Sprite aus (siehe update_sprites)
        rts

; load_level: Steinkarte des Levels `level` (0-basiert) ins RAM (MAP) kopieren, zerstoerbare Steine zaehlen, Bild aufbauen.
;   Die 216 Byte kommen ueber level_tab_lo/hi (lib/levels.inc).  bricks = alle Steine ausser Typ 0 (leer) und 11 (unzerstoerbar);
;   Boss-Kerne (Typ 16..23, Lebenspunkte = Typ - 15) zaehlen zusaetzlich in cores.  Weiter: Hintergrundverlauf (level and 3), Levelart lkind
;   (level_kind: 0 normal, 1 Bonus, 2 Boss), Bonuszeit btime = 45 s, erster Bossangriff batk = 4 s, Waende, Steine, Kopfzeile und
;   Statuszeile zeichnen, Extras zuruecksetzen, Ball auf den Schlaeger.  In der Demo startet der Ticker (Folge 1) in Zeile 0.
;   Schaltet die Sprites 0 und 1 ein.   Eingabe: level.   Veraendert alles (A, X, Y, tp, ...).
load_level:
        ldx level
        lda level_tab_lo,x
        sta tp
        lda level_tab_hi,x
        sta tp + 1
        jsr clear_screen
        lda #0
        sta bricks
        sta cores
        ldy #0
.ll:    lda (tp),y
        sta MAP,y
        beq .lz                         ; 0 = leer: nicht zaehlen
        cmp #11
        beq .lz                         ; 11 = unzerstoerbar: nicht zaehlen
        inc bricks
        cmp #16
        bcc .lz
        inc cores                       ; Typ 16..23 = Boss-Kern (Treffer-Punkte = Typ - 15)
.lz:    iny
        cpy #216                        ; 12 * 18 Steine
        bne .ll
        lda level
        and #3
        jsr set_theme                   ; jedes Level hat einen eigenen Hintergrundverlauf (4 Verlaeufe im Wechsel)
        ldx level
        lda level_kind,x
        sta lkind                       ; Levelart (Bonus/Boss) aus der Tabelle
        lda #45
        sta btime                       ; Bonuslevel: 45 Sekunden
        lda #0
        sta bsub
        lda #8
        sta batk                        ; Boss: erster Angriff nach 4 s
        jsr draw_walls
        jsr draw_map
        jsr draw_hud
        jsr reset_extras
        jsr place_ball
        jsr draw_status                 ; Bonus-/Bosszeile unten
        lda demo
        beq .nodt
        lda #0                          ; Demo: Ticker in der obersten Zeile (Folge 1: Spielname, Untertitel, Demo-Hinweis)
        ldx #1
        jsr ticker_init
.nodt:
        lda #3
        sta VIC_SPR_ENABLE              ; Ball und Schlaeger an (update_sprites setzt die Bits fuer Sprite 2 und die Kapsel)
        rts

; draw_walls: Oberkante (Zeile 1, Spalten 1..38) und beide Seiten (Spalten 1 und 38, Zeilen 2..24) mit dem Wandzeichen CH_WALL.
;   Farb-RAM 14 = 8 + 6: Mehrfarbzeichen mit blauer Flaeche (Farbe 6), die Kanten kommen aus den Mehrfarbregistern $D022/$D023 (grau).
;   Dazu Spalte 0 und 39 (Zeilen 1..24) als volle schwarze Bloecke, damit die Hintergrundbaender nur zwischen den Waenden zu sehen sind.
;   Veraendert A, X, Y, t1, txtcol, sp, cp.
draw_walls:
        lda #14                         ; 8 + 6: Mehrfarbzeichen, Farbe 6 (blau) als Flaechenfarbe
        sta txtcol
        lda #1
        sta t1
.wt:    ldx t1
        ldy #1
        lda #CH_WALL
        jsr put_cell
        inc t1
        lda t1
        cmp #39
        bne .wt                         ; Spalten 1..38
        lda #2
        sta t1
.ws:    ldx #1
        ldy t1
        lda #CH_WALL
        jsr put_cell
        ldx #38
        ldy t1
        lda #CH_WALL
        jsr put_cell
        inc t1
        lda t1
        cmp #25
        bne .ws                         ; Zeilen 2..24
        lda #0                          ; schwarz: Spalte 0 und 39 mit vollen Bloecken (Zeichen 160 = inverses Leerzeichen) fuellen, damit die
        sta txtcol                      ; Hintergrundbaender nur im Spielfeld zwischen den Waenden sichtbar sind
        lda #1
        sta t1
.wb:    ldx #0
        ldy t1
        lda #160
        jsr put_cell
        ldx #39
        ldy t1
        lda #160
        jsr put_cell
        inc t1
        lda t1
        cmp #25
        bne .wb                         ; Zeilen 1..24
        rts

; draw_map: die ganze Steinkarte zeichnen (12 Zeilen x 18 Spalten).   Veraendert A, X, Y, bidx, bcol, brow, t0, txtcol, sp, cp.
draw_map:
        lda #0
        sta bidx
        sta brow
.dr:    lda #0
        sta bcol
.dc:    ldx bidx
        lda MAP,x
        jsr draw_brick
        inc bidx
        inc bcol
        lda bcol
        cmp #18
        bne .dc
        inc brow
        lda brow
        cmp #12
        bne .dr
        rts

; draw_brick: Stein vom Typ A an (bcol, brow) zeichnen (Typ 0 = Leerraum).  Ein Stein sind zwei Zeichen nebeneinander.
;   Bildschirmspalte = 2 + 2 * bcol; Bildschirmzeile = 4 + brow.  Die Farbe kommt aus brick_color[Typ].   Veraendert A, X, Y, t0, txtcol, sp, cp.
draw_brick:
        sta t0
        tay
        lda brick_color,y
        sta txtcol
        lda bcol
        asl
        clc
        adc #2
        tax
        lda brow
        clc
        adc #4
        tay
        jsr set_rowptr
        ldy #0
        lda t0
        beq .db0
        lda #CH_BRICK_L
        sta (sp),y
        lda txtcol
        sta (cp),y
        iny
        lda #CH_BRICK_R
        sta (sp),y
        lda txtcol
        sta (cp),y
        rts
.db0:   lda #CH_SPACE                   ; leerer Platz: beide Zeichen loeschen
        sta (sp),y
        iny
        sta (sp),y
        rts

; draw_hud: Kopfzeile (Zeile 0) "PUNKTE 000000  LEVEL nn  BAELLE n": Text "punkte" ab Spalte 0, Punktestand 6 Ziffern in Spalte 7-12, "level" ab
;   Spalte 16, Levelnummer 1..24 in Spalte 22-23 (Zehnerstelle leer, wenn 0), "baelle" ab Spalte 26, Anzahl in Spalte 33.  Weiss.
;   In der Demo tut die Routine nichts (die Zeile gehoert dem Ticker).   Veraendert alles (A, X, Y, tp, txtcol, t0, t2, t3, sp, cp).
draw_hud:
        lda demo
        beq .dh0
        rts                             ; Demo: kein Punktestand, die Zeile gehoert dem Ticker
.dh0:   lda #1
        sta txtcol
        lda #<h_score
        sta tp
        lda #>h_score
        sta tp + 1
        ldx #0
        ldy #0
        jsr puts
        lda sc2
        ldx #7
        ldy #0
        jsr put_bcd
        lda sc1
        ldx #9
        ldy #0
        jsr put_bcd
        lda sc0
        ldx #11
        ldy #0
        jsr put_bcd
        lda #<h_level
        sta tp
        lda #>h_level
        sta tp + 1
        ldx #16
        ldy #0
        jsr puts
        lda level                       ; Levelnummer 1..24 in zwei Ziffern (Zehner per wiederholtem Abziehen)
        clc
        adc #1
        ldx #0                          ; X = Zehnerziffer
.tens:  cmp #10
        bcc .ones
        sbc #10                         ; (Uebertrag ist gesetzt: Abziehen ohne sec)
        inx
        bne .tens
.ones:  pha                             ; Einerziffer merken
        txa
        beq .notens                     ; Zehner 0: Leerzeichen statt "0"
        clc
        adc #$30
        bne .tput
.notens: lda #CH_SPACE
.tput:  ldx #22
        ldy #0
        jsr put_cell
        pla
        clc
        adc #$30
        ldx #23
        ldy #0
        jsr put_cell
        lda #<h_balls
        sta tp
        lda #>h_balls
        sta tp + 1
        ldx #26
        ldy #0
        jsr puts
        lda lives
        clc
        adc #$30
        ldx #33
        ldy #0
        jmp put_cell

h_score: .screen "punkte"
         .byte 0
h_level: .screen "level"
         .byte 0
h_balls: .screen "baelle"
         .byte 0

; Farb-RAM-Wert je Steintyp (Index = Typ 0..23) = 8 + Farbe (Bit 3 = Mehrfarbzeichen; nur die Farben 0..7 sind moeglich): 1 rot, 2 gelb, 3 gruen, 4 tuerkis,
; 5 blau, 6 violett, 7 weiss, 8 rot; 9 Silber = weiss, 10 Gold = gelb, 11 unzerstoerbar = schwarz (dunkles Metall), 12 explosiv = rot, 13 Ueberraschung = weiss;
; 14 und 15 unbenutzt (schwarz); 16..23 Boss-Kerne (eine Farbe je Lebensstand, 8 Stufen).
brick_color: .byte 8, 10, 15, 13, 11, 14, 12, 9, 10, 9, 15, 8, 10, 9, 8, 8, 10, 12, 14, 11, 13, 15, 9, 9
; Punkte je Treffer (BCD, ein Byte, Index = Typ): 10 * Typ fuer 1..8, Silber 50, Gold 90, unzerstoerbar 0, explosiv/Ueberraschung 10, 14/15 unbenutzt,
; Boss-Kerne (16..23) 20 je Treffer.  hit_brick addiert den Wert mit add_score.
brick_pts:   .byte 0, $10, $20, $30, $40, $50, $60, $70, $80, $50, $90, 0, $10, $10, 0, 0, $20, $20, $20, $20, $20, $20, $20, $20

; ---------------------------------------------------------------------------------------------------------------------
; Spielschleife
; ---------------------------------------------------------------------------------------------------------------------
; play: spielt bis zum Ende eines Levels oder bis alle Baelle des Bildes verloren sind.   Rueckgabe A: 0 = Level geschafft, 1 = Ball verloren, 2 = Abbruch.
;   Ein Durchlauf (= ein Bild, 50 Hz): wait_frame; in der Demo Musik und Ticker; update_sprites (zuerst, solange der Strahl unten steht);
;   sfx_update; get_input (mask); Abbruch bei IN_ABORT; fx_frame; tick_timers; move_paddle; update_cap; update_shots.  Haftet der Ball (stuck):
;   place_ball, bei Feuer launch, naechstes Bild.  Sonst: bei Laser und Feuer laser_fire, dann move_all (alle Baelle bewegen).  Ende nach
;   Levelart (lkind): normal = bricks = 0; Bonus = btime = 0 (oder bricks = 0); Boss = cores = 0.  Levelende: gelber Randblitz, SFX_EXTRA, 60 Bilder
;   Pause; Ball verloren (kein Ball mehr im Spiel): roter Randblitz, SFX_LOSE, 50 Bilder Pause.
;   Zeit: nach wait_frame bleibt etwa ein Bild (312 Rasterzeilen) bis zum naechsten Bildwechsel; der Strahl ist unten (Zeile 251 und folgende).
play:   jsr wait_frame                  ; ab hier ~ 300 Rasterzeilen Zeit bis zum naechsten Bild
.ifdef TESTFPS
        inc $c200                       ; Testfassung: Bildzaehler (16 Bit) - zeigt, ob Bilder ausgelassen werden (ruckelt)
        bne .fp1
        inc $c201
.fp1:   lda $c200
        and #15
        bne .fp2
        lda #1
        sta txtcol                      ; weiss (Farbe < 8: normale Zeichen)
        lda $c201                       ; alle 16 Bilder: Zaehler als Hexziffern in Zeile 24, Spalte 6..9 anzeigen
        ldx #6
        ldy #24
        jsr put_bcd
        lda $c200
        ldx #8
        ldy #24
        jsr put_bcd
.fp2:
.endif
        lda demo
        beq +
        jsr mus_tick                    ; in der Demo spielt die Musik weiter (Effekte sind dort stumm)
.ifndef NOTICKER
        jsr ticker_tick                 ; Ticker oben (Spielname und Untertitel im Wechsel)
.endif
+       jsr update_sprites              ; zuerst die Sprites setzen (Zustand des vorigen Durchlaufs), solange der Strahl unten steht
        jsr sfx_update
        jsr get_input                   ; mask = Eingabe dieses Bildes
        lda mask
        and #IN_ABORT
        beq +
        lda #2
        rts
+       jsr fx_frame                    ; Randblitz und Glanzlicht
        jsr tick_timers                 ; Extra-Zeiten herunterzaehlen
        jsr move_paddle
        jsr update_cap                  ; Kapsel fallen lassen / auffangen
        jsr update_shots                ; Laserschuesse bewegen
        lda stuck
        beq .live
        jsr place_ball                  ; Ball klebt am Schlaeger und folgt ihm
        lda mask
        and #IN_FIRE
        beq play
        jsr launch                      ; Feuer: Ball abschiessen
        jmp play
.live:  lda mask
        and #IN_FIRE
        beq .nofire
        lda tlas
        beq .nofire
        jsr laser_fire                  ; Laser aktiv und Feuer gedrueckt: zwei Schuesse
.nofire:
        jsr move_all                    ; alle aktiven Baelle bewegen; A = 1 wenn keiner mehr im Spiel ist
        cmp #1
        beq .gone
        lda lkind
        beq .nk                         ; normales Level: nur die Steine zaehlen
        cmp #1
        bne .boss
        lda btime                       ; Bonuslevel: Ende, wenn die Zeit um ist (oder alle Steine weg sind)
        beq .cleared
        bne .nk
.boss:  lda cores                       ; Bosslevel: Ende, wenn alle Kerne zerstoert sind (uebrige Steine zaehlen nicht)
        bne .bplay
        beq .cleared
.nk:    lda bricks
        bne .bplay                      ; noch Steine uebrig
.cleared:
        lda #7
        ldx #25
        jsr flash                       ; Level geschafft: gelber Rand
        lda #SFX_EXTRA                  ; Level geschafft
        jsr sfx_play
        ldx #60
        jsr wait_frames
        lda #0
        rts
.gone:  lda #2
        ldx #12
        jsr flash                       ; Ball verloren: roter Randblitz
        lda #SFX_LOSE
        jsr sfx_play
        ldx #50
        jsr wait_frames
        lda #1
        rts
.bplay: jmp play

; get_input: mask aus Joystick/Tastatur; im Demo-Modus vom Rechner.  Dann beendet jede echte Eingabe die Demo (IN_ABORT).
;   Danach wirkt das Extra "umgekehrte Steuerung" (links <-> rechts vertauscht).   Ausgabe: mask (IN_*-Bits).   Veraendert A, X, Y.
;   Mit AUTOPLAY liefert nur auto_input die Eingabe, mit TESTLOSE nur Feuer.
get_input:
.ifdef AUTOPLAY
        jsr auto_input
.else
.ifdef TESTLOSE
        lda #IN_FIRE                    ; Testfassung: nur schiessen, den Schlaeger nie bewegen -> der Ball geht verloren
        sta mask
.else
        lda demo
        beq .real
        jsr auto_input                  ; Demo: der Rechner spielt ...
        jsr input_poll                  ; ... bis der Spieler irgendetwas drueckt
        beq .rev
        lda #IN_ABORT
        sta mask
        rts
.real:  jsr input_poll
        sta mask
.endif
.endif
.rev:   lda trev
        beq .gi
        lda mask
        and #IN_LEFT | IN_RIGHT
        beq .gi
        lda mask
        eor #IN_LEFT | IN_RIGHT         ; beide Richtungsbits kippen: links wird rechts und umgekehrt
        sta mask
.gi:    rts

; auto_input: Der Rechner spielt.  Der Schlaeger soll den Ball mit dem Zielpunkt atgt (8..23 px von der linken Kante) treffen,
;   damit unterschiedliche Winkel vorkommen.  Differenz = (Schlaegerkante + atgt) - (Ballmitte); >= 6 -> links, < -6 -> rechts, dazwischen
;   stehen bleiben (Totzone, siehe unten).  Feuer ist immer gesetzt.   Ausgabe: mask.   Veraendert A, X, t0..t3 und die Ballregister (pick_ball).
auto_input:
        jsr pick_ball                   ; dem tiefsten Ball folgen
        jsr get_px                      ; pxl/pxh = linke Ballkante in px
        lda pxl
        clc
        adc #3                          ; Ballmitte = Kante + 3 (Ball 6 px breit)
        sta t0
        lda pxh
        adc #0
        sta t1
        lda padl
        clc
        adc atgt
        sta t2
        lda padh
        adc #0
        sta t3
        sec
        lda t2
        sbc t0                          ; Differenz Ziel - Ball (16 Bit, vorzeichenbehaftet)
        tax
        lda t3
        sbc t1
        bmi .neg                        ; Ziel liegt links vom Ball -> Schlaeger nach rechts
        bne .left                       ; >= 256: weit rechts
        cpx #6
        bcc .none                       ; |Differenz| unter 6 px: nichts tun (Totzone: der Schlaeger bewegt sich 4 px je Bild, bei einer
                                        ; Schwelle von 2 px schwingt er um den Zielpunkt hin und her und die Demo wirkt ruckelig)
.left:  lda #IN_LEFT | IN_FIRE
        bne .set
.neg:   cmp #$ff
        bne .right                      ; weit links
        cpx #$fa
        bcs .none                       ; -6 .. -1: klein (Totzone, siehe oben)
.right: lda #IN_RIGHT | IN_FIRE
        bne .set
.none:  lda #IN_FIRE
.set:   sta mask
        rts

; move_paddle: Schlaeger nach mask (IN_LEFT/IN_RIGHT) um 4 px je Bild bewegen und begrenzen: links auf 16, rechts auf 304 - Breite
;   (bei 32 px Breite: 272); 16 Bit: padh:padl.   Veraendert A, t0.
move_paddle:
        lda mask
        and #IN_LEFT
        beq .mr
        sec
        lda padl
        sbc #4
        sta padl
        lda padh
        sbc #0
        sta padh
.mr:    lda mask
        and #IN_RIGHT
        beq .clamp
        clc
        lda padl
        adc #4
        sta padl
        lda padh
        adc #0
        sta padh
.clamp: lda padh
        bmi .low                        ; negativ: unter 0 gelaufen
        bne .high                       ; 256 und mehr
        lda padl
        cmp #16
        bcs .done                       ; 16..255: in Ordnung
.low:   lda #16                         ; links anschlagen
        sta padl
        lda #0
        sta padh
        rts
.high:  lda #48                         ; rechter Anschlag: 304 - Breite = 256 + (48 - Breite)
        sec
        sbc padw
        sta t0
        lda padl
        cmp t0
        bcc .done                       ; links davon: in Ordnung
        lda t0
        sta padl
.done:  rts

; place_ball: Ball haftet in der Schlaegermitte auf dem Schlaeger (stuck = 1).
;   X = (padl + 13) * 64 (Ballkante = Schlaegermitte padl + 16 - Ballbreite / 2 = padl + 13), Y = (184 - 6) * 64 = $2C80.
place_ball:
        lda #1
        sta stuck
        lda padw
        lsr                             ; halbe Schlaegerbreite
        sec
        sbc #3                          ; minus halbe Ballbreite -> Abstand der Ballkante von der Schlaegerkante
        clc
        adc padl
        sta bxl
        lda padh
        adc #0
        sta bxh
        ldx #6                          ; mal 64 = 6 mal links schieben
.pb:    asl bxl
        rol bxh
        dex
        bne .pb
        lda #$80
        sta byl
        lda #$2c
        sta byh
        lda #0
        sta cb
        jsr save_ball                   ; Ball 0 aktiv, die anderen aus (ein neuer Ball beginnt immer allein)
        lda #1
        sta bact
        lda #0
        sta bact + 1
        sta bact + 2
        rts

; load_ball / save_ball: Ball cb aus dem Ballspeicher in die Nullseite holen / dorthin zurueckschreiben.   Veraendert A, X.
load_ball:
        ldx cb
        lda b_xl,x
        sta bxl
        lda b_xh,x
        sta bxh
        lda b_yl,x
        sta byl
        lda b_yh,x
        sta byh
        lda b_vxl,x
        sta vxl
        lda b_vxh,x
        sta vxh
        lda b_vyl,x
        sta vyl
        lda b_vyh,x
        sta vyh
        rts
save_ball:
        ldx cb
        lda bxl
        sta b_xl,x
        lda bxh
        sta b_xh,x
        lda byl
        sta b_yl,x
        lda byh
        sta b_yh,x
        lda vxl
        sta b_vxl,x
        lda vxh
        sta b_vxh,x
        lda vyl
        sta b_vyl,x
        lda vyh
        sta b_vyh,x
        rts

; move_all: jeden aktiven Ball bewegen (load, move_ball, save); verlorene Baelle werden deaktiviert.   Veraendert A, X, cb, nact und die Ballregister.
;   Rueckgabe A: 0 = mindestens ein Ball im Spiel, 1 = alle verloren.
move_all:
        lda #0
        sta nact
        sta cb
.ma:    ldx cb
        lda bact,x
        beq .man
        jsr load_ball
        jsr move_ball
        cmp #1
        bne .mok
        ldx cb
        lda #0
        sta bact,x                      ; dieser Ball ist unten herausgefallen
        beq .man
.mok:   jsr save_ball
        inc nact
.man:   inc cb
        lda cb
        cmp #3
        bne .ma
        lda nact
        bne .alive
        lda #1
        rts
.alive: lda #0
        rts

; pick_ball: den tiefsten aktiven Ball (groesstes byh) in die Nullseite laden (der Autoplay/Demo folgt ihm).   Veraendert A, X, cb, t2, t3.
pick_ball:
        lda #0
        sta t2                          ; bester Y-Wert (High-Byte)
        sta t3                          ; bester Schlitz
        sta cb
.pk:    ldx cb
        lda bact,x
        beq .pkn
        lda b_yh,x
        cmp t2
        bcc .pkn
        sta t2
        stx t3
.pkn:   inc cb
        lda cb
        cmp #3
        bne .pk
        lda t3
        sta cb
        jmp load_ball

; multiball: aus dem ersten aktiven Ball zwei weitere Baelle erzeugen (Zonen 1 und 6 = -43 / +43 Grad), soweit Schlitze frei sind.
;   Die neuen Baelle uebernehmen die Position des Quellballs und bekommen nur eine andere Richtung (set_vel).   Veraendert A, X, cb, t1, t2.
multiball:
        ldx #0
.mb1:   lda bact,x
        bne .mbf
        inx
        cpx #3
        bne .mb1
        rts                             ; kein aktiver Ball
.mbf:   stx cb
        jsr load_ball                   ; Quelle in die Nullseite
        lda #1
        sta t1                          ; naechste Zone: 1, dann 6
        ldx #0
.mb2:   lda bact,x
        bne .mb3                        ; Schlitz belegt
        stx t2
        lda t1
        jsr set_vel                     ; Geschwindigkeit fuer die Zone (Position bleibt die der Quelle)
        ldx t2
        stx cb
        jsr save_ball
        ldx t2
        lda #1
        sta bact,x
        lda #6
        sta t1
        ldx t2
.mb3:   inx
        cpx #3
        bne .mb2
        rts

; launch: Ball abschiessen, leicht schraeg (Zone 3 oder 4 = -9 / +9 Grad, zufaellig).   Spielt SFX_START.   Veraendert A, X, cb, t0 und die Ballregister.
launch: lda #0
        sta stuck
        sta cb
        jsr load_ball                   ; Ball 0 (der haftende) in die Nullseite
        lda #SFX_START
        jsr sfx_play
        jsr rand_byte
        and #1
        clc
        adc #3                          ; Zone 3 oder 4
        jsr set_vel
        jmp save_ball                   ; Ball 0 zurueckschreiben

; set_vel: Geschwindigkeit aus der Richtungstabelle: Index = Stufe * 8 + Zone.   Eingabe: A = Zone 0..7.   Ausgabe: vxl..vyh.   Veraendert A, X, t0.
;   Die Stufe ist speed, bei spdmod = 1/2 um 4 hoeher/niedriger (begrenzt auf 0..15).
set_vel:
        sta t0
        lda speed                       ; wirksame Stufe = Stufe +/- 4 bei "schneller"/"langsamer", begrenzt auf 0..15
        ldx spdmod
        beq .sv
        cpx #1
        bne .slow
        clc
        adc #4
        cmp #16
        bcc .sv
        lda #15
        bne .sv
.slow:  sec
        sbc #4
        bcs .sv
        lda #0
.sv:    asl
        asl
        asl
        ora t0
        tax
        lda dir_xl,x
        sta vxl
        lda dir_xh,x
        sta vxh
        lda dir_yl,x
        sta vyl
        lda dir_yh,x
        sta vyh
        rts

; get_px: ganze Bildpunkte der linken Ballkante: (bxh:bxl) / 64 -> pxh:pxl (nur X; fuer Y rechnet update_sprites selbst).   Veraendert A, X.
get_px: lda bxl
        sta pxl
        lda bxh
        sta pxh
        ldx #6
.gp:    lsr pxh
        ror pxl
        dex
        bne .gp
        rts

; neg_vx / neg_vy: Geschwindigkeitskomponente umkehren (16 Bit: 0 - Wert).   Veraendert A (und C).
neg_vx: sec
        lda #0
        sbc vxl
        sta vxl
        lda #0
        sbc vxh
        sta vxh
        rts
neg_vy: sec
        lda #0
        sbc vyl
        sta vyl
        lda #0
        sbc vyh
        sta vyh
        rts

; ---------------------------------------------------------------------------------------------------------------------
; move_ball: Ball um einen Schritt bewegen; Waende, Steine, Schlaeger.  Rueckgabe A: 0 = weiter, 1 = Ball unten heraus.
;   Erst die X-Bewegung (mit Wand- und Steinpruefung), dann die Y-Bewegung.  Bei Treffer wird die Bewegung dieser Achse
;   zurueckgenommen und die Geschwindigkeitskomponente umgekehrt.  Der Ball ist 6 x 6 px; abgetastet werden die beiden Ecken
;   der Seite, in die er sich bewegt (probe).
; ---------------------------------------------------------------------------------------------------------------------
move_ball:
        lda bxl                         ; alte X-Position merken
        sta oxl
        lda bxh
        sta oxh
        clc
        lda bxl
        adc vxl
        sta bxl
        lda bxh
        adc vxh
        sta bxh
        lda vxh
        bmi .mleft
        ; ---- Bewegung nach rechts: rechte Wand bei px >= 296 (Ball 6 px breit + 296 = 302 <= 303)
        lda bxh
        cmp #$4a                        ; 296 * 64 = $4A00
        bcs .xwall
        ldy #1                          ; Abtastversatz: rechte Ballkante
        bne .xprobe
.mleft: lda bxh
        cmp #4                          ; 16 * 64 = $0400
        bcc .xwall
        cmp #$c0
        bcs .xwall                      ; unter 0 gerutscht (Wert wie $FFxx)
        ldy #0                          ; linke Ballkante
.xprobe:
        sty dxo
        ldx #0                          ; obere Ballkante
        jsr probe
        bne .xbrick
        ldy dxo
        ldx #1                          ; untere Ballkante
        jsr probe
        bne .xbrick
        jmp .ymove
.xbrick:
        jsr hit_brick                   ; Stein getroffen: Punkte, ggf. entfernen
        lda tpierce
        beq .xback
        ldx bidx
        lda MAP,x
        bne .xback                      ; Stein steht noch (unzerstoerbar oder Boss-Kern): normal abprallen
        jmp .ymove                      ; Durchschlag: Stein ist weg, der Ball fliegt in X ohne Umkehr weiter
.xwall: lda #SFX_WALL
        jsr sfx_play
.xback: lda oxl                         ; Bewegung zuruecknehmen
        sta bxl
        lda oxh
        sta bxh
        jsr neg_vx
.ymove: lda byl
        sta oyl
        lda byh
        sta oyh
        clc
        lda byl
        adc vyl
        sta byl
        lda byh
        adc vyh
        sta byh
        lda vyh
        bmi .mup
        ; ---- Bewegung nach unten: Schlaeger oder verloren
        lda byh
        cmp #$2c
        bcc .ybrick0                    ; noch ueber der Schlaegerhoehe: nur Steine
        bne .pad                        ; > $2C00: sicher in der Schlaegerzone oder darunter
        lda byl
        cmp #$80
        bcc .ybrick0                    ; < $2C80 = 178 px: Ballunterkante noch nicht auf Schlaegerhoehe
.pad:   lda byh
        cmp #$30
        bcs .lost                       ; >= 192 px: unten heraus
        jsr paddle_hit                  ; trifft er den Schlaeger? (Rueckgabe C=1 = ja)
        bcs .padok
        lda #0                          ; verfehlt: weiter nach unten fallen
        rts
.padok: lda #0
        rts
.lost:  lda #1
        rts
.mup:   lda byh                         ; ---- Bewegung nach oben: Decke bei px < 16
        cmp #4
        bcc .ywall
        cmp #$c0
        bcs .ywall
.ybrick0:
        ldy #0
        lda vyh
        bmi +
        ldy #1                          ; nach unten: untere Ballkante abtasten
+       sty dxo                         ; (dxo dient hier als Versatz Y)
        ldy #0                          ; linke Ballkante
        ldx dxo
        jsr probe
        bne .ybrick
        ldy #1                          ; rechte Ballkante
        ldx dxo
        jsr probe
        bne .ybrick
        lda #0
        rts
.ybrick:
        jsr hit_brick
        lda tpierce
        beq .yback
        ldx bidx
        lda MAP,x
        bne .yback                      ; Stein steht noch: abprallen
        lda #0                          ; Durchschlag: weiterfliegen (Rueckgabe: Ball im Spiel)
        rts
.ywall: lda #SFX_WALL
        jsr sfx_play
.yback: lda oyl
        sta byl
        lda oyh
        sta byh
        jsr neg_vy
        lda #0
        rts

; probe: tastet einen Punkt der Ballkante in der Steinkarte ab.
;   Eingabe: Y = 0/1: X-Versatz (0 = linke Kante, 1 = rechte Kante: +5 px = +320 in 1/64),
;            X = 0/1: Y-Versatz (0 = obere Kante, 1 = untere Kante)
;   Rueckgabe: Z=1 (A=0) = kein Stein; sonst A = Typ, bcol/brow/bidx gesetzt.
;   Rechnung: Spalte = (X - 16 px) / 16 px = (bxh - 4) / 4 (High-Byte = px / 4);  Zeile = (Y - 32 px) / 8 px = (byh - 8) / 2.
probe:  stx ptB
        clc
        lda bxl
        adc offl,y
        lda bxh
        adc offh,y                      ; High-Byte der Abtastposition X
        sec
        sbc #4
        bcc .none                       ; links vom Steinfeld
        lsr
        lsr
        cmp #18
        bcs .none                       ; rechts davon
        sta bcol
        ldy ptB
        clc
        lda byl
        adc offl,y
        lda byh
        adc offh,y                      ; High-Byte der Abtastposition Y
        sec
        sbc #8
        bcc .none                       ; ueber dem Steinfeld
        lsr
        cmp #12
        bcs .none                       ; darunter
        sta brow
        tax
        lda rowmul,x                    ; Zeile * 18
        clc
        adc bcol
        sta bidx
        tax
        lda MAP,x                       ; Typ; setzt Z
        rts
.none:  lda #0
        rts

offl:   .byte $00, $40                  ; 5 px * 64 = 320 = $0140: Low-Byte
offh:   .byte $00, $01                  ;                          High-Byte
rowmul: .byte 0, 18, 36, 54, 72, 90, 108, 126, 144, 162, 180, 198       ; Zeile * 18

; hit_brick: Stein vom Typ A getroffen (bidx/bcol/brow gesetzt).   Immer zuerst ein kurzer weisser Randblitz.
;   Typ 11 (unzerstoerbar): nur Klang SFX_HARD, keine Punkte.  Gold (10) wird zu Silber (9), Silber zu Typ 1; alle anderen verschwinden.
;   Boss-Kern (Typ 16..23): jeder Treffer nimmt einen Lebenspunkt (Typ - 1); bei Typ 15 ist er zerstoert (cores und bricks sinken, gelber Blitz).
;   Typ 12 (explosiv) hat keine Sonderwirkung: er verschwindet wie ein normaler Stein.
;   Fuer jeden Treffer gibt es Punkte nach brick_pts (add_score); im Bonuslevel zusaetzlich 100 je zerstoertem Stein.  Mit Durchschlag (tpierce)
;   wird jeder zerstoerbare Stein sofort entfernt.  Ein zerstoerter Stein laesst mit etwa 1/12 Wahrscheinlichkeit eine Kapsel fallen
;   (Typ 13 immer; spawn_cap).  Zum Schluss wird der Stein neu gezeichnet (draw_brick) und der Punktestand angezeigt (draw_score).
;   Eingabe: A = Typ (aus MAP[bidx]).   Veraendert A, X, Y, t0, t3 und alles, was die gerufenen Routinen veraendern.
hit_brick:
        pha
        lda #1
        ldx #2
        jsr flash                       ; kurzer weisser Randblitz
        pla
        cmp #11
        bne +
        lda #SFX_HARD
        jmp sfx_play
+       cmp #16
        bcc .nocore
        sta t3                          ; Boss-Kern: jeder Treffer nimmt 1 Lebenspunkt, bei 0 ist der Kern zerstoert
        tay
        lda brick_pts,y
        jsr add_score
        ldx bidx
        lda MAP,x
        sec
        sbc #1
        cmp #15
        bne .corehit                    ; noch Lebenspunkte: Typ - 1
        lda #0                          ; zerstoert
        sta MAP,x
        dec bricks
        dec cores
        lda #SFX_BRICK
        jsr sfx_play
        lda #7
        ldx #8
        jsr flash                       ; gelber Randblitz fuer den Kern
        lda #0
        jmp .redraw
.corehit:
        sta MAP,x
        pha
        lda #SFX_HARD
        jsr sfx_play
        pla
        jmp .redraw
.nocore: pha
        sta t3                          ; Typ merken (fuer die Kapsel-Entscheidung)
        tay
        lda brick_pts,y
        jsr add_score
        pla
        ldx tpierce
        bne .rem                        ; Durchschlag: jeder Stein wird sofort zerstoert (auch Silber und Gold)
        cmp #10
        beq .gold
        cmp #9
        beq .silver
.rem:   lda #0                          ; normaler Stein: weg
        ldx bidx
        sta MAP,x
        dec bricks
        lda #SFX_BRICK
        jsr sfx_play
        lda lkind
        cmp #1
        bne +
        sed                             ; Bonuslevel: 100 Punkte je Stein zusaetzlich
        clc
        lda sc1
        adc #1
        sta sc1
        lda sc2
        adc #0
        sta sc2
        cld
+       lda t3                          ; ein Ueberraschungsstein (Typ 13) laesst immer eine Kapsel fallen, andere in etwa 1 von 12
        cmp #13
        beq .drop
.ifdef TESTCAP
        bne .drop                       ; Testfassung: jeder Stein laesst eine Kapsel fallen
.endif
        jsr rand_byte
        cmp #21
        bcs .nodrop
.drop:  jsr spawn_cap
.nodrop:
        lda #0
        jmp .redraw
.gold:  lda #9
        bne .down
.silver: lda #1
.down:  ldx bidx
        sta MAP,x
        pha
        lda #SFX_HARD
        jsr sfx_play
        pla
.redraw:
        jsr draw_brick                  ; (bcol/brow stehen noch)
        jmp draw_score

; draw_score: nur die Punktzahl neu zeichnen.
draw_score:
        lda demo
        beq .dsc0
        rts
.dsc0:  lda #1
        sta txtcol
        lda sc2
        ldx #7
        ldy #0
        jsr put_bcd
        lda sc1
        ldx #9
        ldy #0
        jsr put_bcd
        lda sc0
        ldx #11
        ldy #0
        jmp put_bcd

; add_score: A = BCD-Punkte (00..99) zum Punktestand addieren.  Dezimalmodus: Die CPU rechnet die Bytes als zwei Dezimalziffern.
add_score:
        sed
        clc
        adc sc0
        sta sc0
        lda sc1
        adc #0
        sta sc1
        lda sc2
        adc #0
        sta sc2
        cld
        rts

; paddle_hit: Ball ist auf Schlaegerhoehe (178 <= Y < 192).  Prueft die Ueberlappung in X und prallt ab.
;   Von oben kommender Ball: neue Richtung nach Zone (set_vel), Ball genau auf den Schlaeger gesetzt (Y = 178 px), Klebe-Extra, hits und
;   Geschwindigkeitsstufe, SFX_PADDLE, neuer Autoplay-Zielpunkt.  Von der Seite kommender Ball: prallt seitlich ab (SFX_WALL), kein Treffer.
;   Eingabe: Ballregister bxl.., oyl/oyh (alte Y-Lage), padl/padw/zstep, mask.   Veraendert A, X, t0..t3 und die Ballregister.
;   Ueberlappung: d = Ballkante - Schlaegerkante muss in -5 .. Breite-1 liegen (Ball 6 px breit).
;   Zone = (d + 3) / zstep (Zonenbreite = Breite / 8), hoechstens 7;  negative d -> Zone 0.   Rueckgabe: C=1 = abgeprallt.
paddle_hit:
        jsr get_px
        sec
        lda pxl
        sbc padl
        sta t0
        lda pxh
        sbc padh
        sta t1                          ; t1:t0 = d
        bmi .neg
        bne .miss                       ; d >= 256
        lda t0
        cmp padw
        bcs .miss                       ; d >= Breite
        clc
        adc #3                          ; Ballmitte = Kante + 3
        cmp padw
        bcc +
        lda padw                        ; hoechstens Breite - 1 (letzte Zone)
        sbc #1
+       ldx #0                          ; Zone = (Ballmitte - Schlaegerkante) / zstep, hoechstens 7
.zl:    cmp zstep
        bcc .zd
        sbc zstep
        inx
        bne .zl
.zd:    cpx #8
        bcc +
        ldx #7
+       txa
        jmp .bounce
.neg:   lda t0
        cmp #251                        ; d >= -5 ?
        bcc .miss
        lda #0
.bounce:
        sta t2                          ; Zone merken
        lda oyh                         ; kam der Ball von oben?  (alte Y-Lage + 6 px < Schlaegeroberkante: Y < 178 px = $2C80)
        cmp #$2c
        bcc .top
        bne .side
        lda oyl
        cmp #$80
        bcs .side
.top:   lda t2                          ; Einfluss der Schlaegerbewegung: nach links bewegt -> eine Zone weiter links, nach rechts -> rechts
        tax
        lda mask
        and #IN_LEFT
        beq .nl
        cpx #0
        beq .nl
        dex
.nl:    lda mask
        and #IN_RIGHT
        beq .nr
        cpx #7
        bcs .nr
        inx
.nr:    txa
        pha
        lda #$80                        ; Ball genau auf den Schlaeger setzen
        sta byl
        lda #$2c
        sta byh
        pla
        jsr set_vel
        lda sticky                      ; Klebe-Extra: Ball bleibt am Schlaeger haengen (nur bei einem einzelnen Ball in Schlitz 0)
        beq +
        lda bact + 1
        ora bact + 2
        bne +
        lda cb
        bne +
        lda #0
        sta sticky
        lda #1
        sta stuck
+       inc hits
        lda hits
        and #7
        bne +
        lda speed                       ; alle 8 Treffer eine Stufe schneller, bis 15
        cmp #15
        bcs +
        inc speed
+       lda #SFX_PADDLE
        jsr sfx_play
        jsr rand_byte                   ; neuer Zielpunkt fuer den Autoplay (8..23)
        and #15
        clc
        adc #8
        sta atgt
        sec
        rts
        ; ---- Seitenaufprall: der Ball war schon auf Schlaegerhoehe und wurde von der Seite getroffen (oder lief hinein)
.side:  lda padw                        ; links oder rechts der Schlaegermitte? Ballmitte = Kante + 3, Schlaegermitte = Kante + Breite / 2
        lsr
        sta t3
        ldx t1                          ; t1:t0 = d = Ballkante - Schlaegerkante (aus dem Anfang der Routine)
        bmi .sl                         ; d negativ: der Ball liegt links der Schlaegerkante
        lda t0
        clc
        adc #3                          ; Ballmitte relativ zur Schlaegerkante
        cmp t3
        bcs .sr                         ; rechts der Mitte
.sl:    lda vxh                         ; links: nach links wegprallen (vx negativ), Ball links neben den Schlaeger setzen
        bpl .sl1
        jmp .sl2
.sl1:   jsr neg_vx
.sl2:   sec
        lda padl
        sbc #6
        sta bxl
        lda padh
        sbc #0
        sta bxh
        jmp .sx
.sr:    lda vxh                         ; rechts: nach rechts wegprallen, Ball rechts neben den Schlaeger setzen
        bmi .sr1
        jmp .sr2
.sr1:   jsr neg_vx
.sr2:   clc
        lda padl
        adc padw
        sta bxl
        lda padh
        adc #0
        sta bxh
.sx:    ldx #6                          ; Pixel -> 1/64 px
.sxl:   asl bxl
        rol bxh
        dex
        bne .sxl
        lda #SFX_WALL
        jsr sfx_play
        clc                             ; kein Abprall vom Schlaeger: der Ball faellt weiter und geht meist verloren
        rts
.miss:  clc
        rts

; ---------------------------------------------------------------------------------------------------------------------
; boss_attack: der Boss laesst eine boese Kapsel fallen (S = schmaler Schlaeger oder R = umgekehrte Steuerung) aus einer zufaelligen Spalte.
;   Nur wenn gerade keine Kapsel faellt.  Setzt bcol (1..16), brow = 4 und capforce (2 = S, 3 = R) und ruft spawn_cap.   Veraendert A, bcol, brow, capforce.
boss_attack:
        lda capact
        bne .ba9                        ; es faellt schon eine Kapsel
        jsr rand_byte
        and #15
        clc
        adc #1                          ; Spalte 1..16
        sta bcol
        lda #4
        sta brow                        ; Zeile 4 (mitten im Steinfeld)
        jsr rand_byte
        and #1
        clc
        adc #2                          ; erzwungene Art + 1: 2 = S, 3 = R
        sta capforce
        jmp spawn_cap
.ba9:   rts

; draw_status: Statuszeile (Bildschirmzeile 24): Bonus = verbleibende Sekunden ("bonus  nn", Spalte 17), Boss = verbliebene Kerne ("boss: kerne nn",
;   Spalte 13), sonst nichts (die Zeile bleibt leer).   Zahlen in Spalte 25/26, gelb.   Veraendert A, X, Y, tp, txtcol, sp, cp.
draw_status:
        lda #7
        sta txtcol
        lda lkind
        beq .ds0
        cmp #1
        bne .dsb
        lda #<s_bonus
        sta tp
        lda #>s_bonus
        sta tp + 1
        ldx #17
        ldy #24
        jsr puts
        lda btime
        jmp .dsnum
.dsb:   lda #<s_boss
        sta tp
        lda #>s_boss
        sta tp + 1
        ldx #13
        ldy #24
        jsr puts
        lda cores
.dsnum: ldx #0                          ; Zahl 0..99 in zwei Ziffern (Zehner per Abziehen)
.dst:   cmp #10
        bcc .dso
        sbc #10
        inx
        bne .dst
.dso:   pha
        txa
        clc
        adc #$30
        pha
        ldx #25
        ldy #24
        jsr set_rowptr
        pla
        ldy #0
        sta (sp),y
        pla
        clc
        adc #$30
        iny
        sta (sp),y
        rts
.ds0:   rts
s_bonus: .screen "bonus   "
        .byte 0
s_boss: .screen "boss: kerne "
        .byte 0

; fx_frame: Randblitz herunterzaehlen und das Glanzlicht ueber die Steine laufen lassen (je zweites Bild ein Schritt).
fx_frame:
        lda bflash
        beq .fx1
        dec bflash
        bne .fx1
        lda #0
        sta VIC_BORDERCOLOR             ; Blitz zu Ende
.fx1:   lda tick
        and #1
        bne .fx9                        ; nur jedes zweite Bild
        lda glg
        cmp #31
        bcs .fx2                        ; Pause
        jsr glint_diag_white            ; neue Diagonale weiss
.fx2:   lda glg
        beq .fx3
        cmp #32
        bcs .fx3
        sec
        sbc #1
        jsr glint_diag_restore          ; vorige Diagonale wieder in Steinfarbe
.fx3:   inc glg
        lda glg
        cmp #100
        bcc .fx9
        lda #0
        sta glg
.fx9:   rts

; glint_diag_white / glint_diag_restore: A = Nummer der Diagonale g (Spalte + Zeile = g).  Alle vorhandenen Steine auf ihr werden weiss
;   gefaerbt bzw. in der richtigen Farbe neu gezeichnet.
glint_diag_white:
        ldx #1
        bne glint_run
glint_diag_restore:
        ldx #0
glint_run:
        stx t3                          ; 1 = weiss, 0 = zuruecksetzen
        sta t2                          ; g
        lda #0
        sta brow
.gr1:   lda t2
        sec
        sbc brow                        ; Spalte = g - Zeile
        bcc .gr9                        ; negativ: weiter unten gibt es nichts mehr auf dieser Diagonale
        cmp #18
        bcs .gr8
        sta bcol
        ldy brow
        lda rowmul,y
        clc
        adc bcol
        tax
        lda MAP,x
        beq .gr8                        ; kein Stein
        ldx t3
        bne .grw
        jsr draw_brick                  ; zuruecksetzen: normal zeichnen (A = Typ)
        jmp .gr8
.grw:   lda bcol                        ; weiss faerben: Farb-RAM beider Zeichen = 8 + 1
        asl
        clc
        adc #2
        tax
        lda brow
        clc
        adc #4
        tay
        jsr set_rowptr
        ldy #0
        lda #9
        sta (cp),y
        iny
        sta (cp),y
.gr8:   inc brow
        lda brow
        cmp #12
        bne .gr1
.gr9:   rts

; ---------------------------------------------------------------------------------------------------------------------
; update_sprites: Spritepositionen und -freigaben aus dem Spielzustand setzen (einmal je Bild, am Anfang von play).
;   Baelle (Schlitz 0..2 -> Sprites 0, 4, 5): x = px + 24, y = py + 50 (py = Y / 64); weiss, bei Durchschlag gelb.
;   Schlaeger: Sprite 1 bei padl, Sprite 2 bei padl + pad2o (16 oder 24 px; 0 = nur ein Sprite), x + 24, y = PADY + 50.
;   Kapsel (Sprite 3): x = capx + 20 (die Muenze sitzt 4 px eingerueckt im Sprite), y = capy + 50, Zeiger aus cap_frames[capfr / 4].
;   Laserschuesse (Sprites 6, 7): x = 2 * u + 24, y = shy + 50.
;   Das 9. Bit der X-Koordinaten steht gesammelt in $D010 (Bit n = Sprite n), die Freigabebits gesammelt in $D015: beides wird erst am Ende
;   geschrieben, damit nie ein halb aktualisiertes Bild entsteht.   Veraendert A, X, Y, cb, sprmsb, sprena, t0..t3 und die Ballregister (load_ball).
; ---------------------------------------------------------------------------------------------------------------------
update_sprites:
        lda #0
        sta sprmsb
        sta sprena
        sta cb
.ub:    ldx cb                          ; Baelle 0..2 -> Sprites 0, 4, 5
        lda bact,x
        beq .ubn                        ; Schlitz leer: Sprite bleibt aus
        jsr load_ball
        jsr get_px                      ; pxl/pxh (bis 9 Bit)
        ldx cb
        ldy ball_reg,x                  ; Y = Registerversatz des Sprites (0, 8, 10)
        clc
        lda pxl
        adc #24
        sta VIC_SPR0_X,y
        lda pxh
        adc #0                          ; Bit 8 der Sprite-X-Koordinate
        beq +
        lda ball_bit,x
        ora sprmsb
        sta sprmsb
+       lda byl                         ; py = (Y * 4) / 256 = Y / 64
        sta t0
        lda byh
        sta t1
        asl t0
        rol t1
        asl t0
        rol t1
        lda t1
        clc
        adc #50
        sta VIC_SPR0_Y,y
        lda ball_bit,x
        ora sprena
        sta sprena
.ubn:   inc cb
        lda cb
        cmp #3
        bne .ub
        lda sprmsb
        sta t2
        clc
        lda padl
        adc #24
        sta VIC_SPR1_X
        lda padh
        adc #0
        asl                             ; Bit 1 = Sprite 1
        ora t2
        sta t2
        clc
        lda padl
        adc pad2o                       ; zweite Haelfte rechts daneben (16 oder 24 px) ...
        sta t0
        lda padh
        adc #0
        sta t1
        clc
        lda t0
        adc #24                         ; ... plus Sprite-Versatz 24 (16-Bit-Addition, damit Bit 8 stimmt)
        sta VIC_SPR2_X
        lda t1
        adc #0
        asl
        asl                             ; Bit 2 = Sprite 2
        ora t2
        sta t2
        lda #PADY + 50
        sta VIC_SPR1_Y
        sta VIC_SPR2_Y
        lda #1                          ; Ballfarbe: weiss, bei Durchschlag gelb
        ldx tpierce
        beq .bc
        lda #7
.bc:    sta VIC_SPR0_COLOR
        sta VIC_SPR4_COLOR
        sta VIC_SPR5_COLOR
        lda sprena                      ; Sprite-Freigabe: Baelle (aus der Schleife oben) + Schlaeger links
        ora #2
        ldx pad2o
        beq +
        ora #4                          ; zweite Schlaegerhaelfte (nicht beim schmalen Schlaeger)
+       ldx capact
        beq +
        ora #8                          ; Kapsel
        pha
        sec                             ; die Muenze liegt im Sprite 4 px (2 Multicolor-Pixel) eingerueckt: x = capx + 24 - 4 = capx + 20
        lda capxl
        adc #19                         ; (Uebertrag gesetzt: +20)
        sta VIC_SPR3_X
        lda capxh
        adc #0
        asl
        asl
        asl                             ; Bit 3 = Sprite 3
        ora t2
        sta t2
        lda capfr                       ; Drehphase 0..5 = capfr / 4 -> Zeiger aus der Folge Vorn, 3/4, halb, Kante, halb, 3/4
        lsr
        lsr
        tax
        lda cap_frames,x
        sta SPTR + 3
        lda capy
        clc
        adc #50
        sta VIC_SPR3_Y
        pla
+       sta t3                          ; Freigabebits bisher
        ldx #0                          ; Laserschuesse: Sprites 6 und 7
.ss:    lda shact,x
        beq .ssn
        lda shu,x                       ; x = 2 * u + 24
        asl
        sta t0
        lda #0
        rol
        sta t1                          ; (t1:t0) = 2 * u
        clc
        lda t0
        adc #24
        sta t0
        bcc +
        inc t1
+       lda shot_reg,x
        tay
        lda t0
        sta VIC_SPR0_X,y
        lda t1
        beq +
        lda shot_bit,x
        ora t2
        sta t2
+       lda shy,x
        clc
        adc #50
        sta VIC_SPR0_Y,y
        lda shot_bit,x
        ora t3
        sta t3
.ssn:   inx
        cpx #2
        bne .ss
        lda t3
        sta VIC_SPR_ENABLE
        lda t2
        sta VIC_SPR_X_MSB
        rts
shot_reg: .byte 12, 14                          ; Registerversatz Sprite 6, 7 (2 * Nummer)
shot_bit: .byte 64, 128

ball_reg: .byte 0, 8, 10                      ; Registerversatz der Ballsprites (Sprite 0, 4, 5: $D000 + 2 * Nummer)
ball_bit: .byte 1, 16, 32                     ; Bitmaske der Ballsprites

; tick_timers: einmal je Bild.  Alle 25 Bilder (0,5 s) zaehlen die Extra-Timer um 1 herunter; bei 0 endet das Extra.
tick_timers:
        dec tick
        bne .tt
        lda #25
        sta tick
        lda lkind
        beq .tk0
        cmp #1
        bne .tkb
        lda bsub                        ; Bonus: alle zwei Halbsekunden eine Sekunde herunter
        eor #1
        sta bsub
        bne .tk1
        lda btime
        beq .tk1
        dec btime
.tk1:   jsr draw_status
        jmp .tk0
.tkb:   dec batk                        ; Boss: alle 4 s ein Angriff (eine boese Kapsel)
        bne .tk2
        lda #8
        sta batk
        jsr boss_attack
.tk2:   jsr draw_status
.tk0:   lda tpad
        beq .t1
        dec tpad
        bne .t1
        lda #32                         ; Breite zurueck auf normal
        jsr set_padw
.t1:    lda trev
        beq .t2
        dec trev
.t2:    lda tlas
        beq .t3
        dec tlas
.t3:    lda tpierce                     ; Durchschlag zaehlt herunter
        beq .t4
        dec tpierce
.t4:    lda tspd
        beq .tt
        dec tspd
        bne .tt
        lda #0
        sta spdmod                      ; Geschwindigkeit wieder normal (wirkt ab dem naechsten Schlaegertreffer)
.tt:    rts

; spawn_cap: laesst an der Position des zuletzt zerstoerten Steins (bcol/brow) eine Kapsel fallen (hoechstens eine zugleich).
;   Start: capx = 16 + 16 * bcol, capy = 32 + 8 * brow (px), capact = 1.  Die Kapselart wird mit cap_pick zufaellig gewaehlt (oder per capforce
;   erzwungen); ihr Buchstabe wird aus dem RAM-Zeichensatz ($8800 + Bildschirmcode * 8) in den Sprite-Block gfx_cap (Zeiger $06 = $8180)
;   kopiert: Zeichenzeile i -> Sprite-Zeile 2 + i, Byte 1 (= Byte 3 * (2 + i) + 1); je zwei Bildpunkte der Zeichenzeile werden zu einem
;   Multicolor-Pixel (mindestens einer gesetzt: Wert 3 = dunkel, sonst 2 = Spritefarbe der Muenze).   Veraendert A, X, Y, t0..t2, sp.
spawn_cap:
.ifdef TESTMULTI
        jsr multiball                   ; Testfassung: sofort Mehrball (prueft Bewegung/Anzeige mehrerer Baelle)
.endif
        lda capact
        bne .sc9
        lda bcol
        asl
        asl
        asl
        asl                             ; Spalte * 16 (Low-Byte; Spalte 16 und 17 laufen ueber 255)
        sta capxl
        lda bcol
        lsr
        lsr
        lsr
        lsr                             ; Spalte / 16 = Uebertrag in das High-Byte
        sta capxh
        clc
        lda capxl
        adc #16                         ; + 16 px (linker Rand des Steinfelds)
        sta capxl
        bcc +
        inc capxh
+       lda brow
        asl
        asl
        asl                             ; Zeile * 8
        clc
        adc #32                         ; + 32 px (Oberkante des Steinfelds)
        sta capy
        jsr rand_byte
        and #15
        tax
        lda cap_pick,x
        sta captype
        lda capforce
        beq .nf
        sec
        sbc #1
        sta captype                     ; erzwungene Art (Bossangriff)
        lda #0
        sta capforce
.nf:
.ifdef TESTMULTI
        lda #8                          ; Testfassung: immer Mehrball
        sta captype
.endif
        ldx captype                     ; Art als Index fuer Farbe und Buchstabe (A enthaelt hier nur 0 bzw. 8, nicht die Art)
        lda cap_color,x
        sta VIC_SPR3_COLOR
        lda cap_letter,x                ; Bildschirmcode des Buchstabens
        sta t0
        lda #0
        sta t1
        asl t0                          ; Code * 8 = Adresse im Zeichensatz ($8800 + Code * 8)
        rol t1
        asl t0
        rol t1
        asl t0
        rol t1
        lda t1
        clc
        adc #$88
        sta sp + 1
        lda t0
        sta sp
        ldy #0                          ; Zeile 0..7 des Buchstabens -> Sprite-Zeile 2 + Zeile, Byte 1 (Multicolor-Pixel 4..7 = Mitte der Muenze)
        ldx #7                          ; Byteindex: (2 * 3) + 1 (Muenze: Zeilen 2..9, Multicolor-Pixel 4..7 = Byte 1)
.sg:    lda (sp),y                      ; Zeichenzeile: 8 Hires-Bildpunkte
        sta t0
        lda #0
        sta t1                          ; Ergebnisbyte: 4 Multicolor-Pixel
        lda #4
        sta t2
.sp:    lda t0                          ; die linken zwei Bildpunkte: mindestens einer gesetzt -> dunkel (3), sonst Muenzenfarbe (2)
        and #$c0
        beq .sb
        lda #3
        bne .sv
.sb:    lda #2
.sv:    asl t1
        asl t1
        ora t1
        sta t1
        asl t0
        asl t0
        dec t2
        bne .sp
        lda t1
        sta gfx_cap,x
        inx
        inx
        inx
        iny
        cpy #8
        bne .sg
        lda #0
        sta capfr
        lda #1
        sta capact
.sc9:   rts

; Kapselarten: 0 B breit, 1 S schmal, 2 R umgekehrt, 3 L langsam, 4 F schnell, 5 K klebrig, 6 + Leben, 7 P Punkte, 8 M Mehrball, 9 G Laser,
; 10 D Durchschlag
cap_frames: .byte $06, $08, $09, $0a, $09, $08           ; Sprite-Zeiger der Drehphasen (vorn, 3/4, halb, Kante, halb, 3/4) = gfx_cap, _cap1, _cap2, _cap3
cap_letter: .byte 2, 19, 18, 12, 6, 11, $2b, 16, 13, 7, 4        ; Bildschirmcodes: B=2, S=19, R=18, L=12, F=6, K=11, + = $2B, P=16, M=13, G=7, D=4
; Farbe der Muenze je Art (Sprite-Farbe): B hellblau, S hellrot, R rot, L gruen, F gelb, K orange, + weiss, P tuerkis, M violett, G hellgruen,
; D (Durchschlag) grau
cap_color:  .byte 14, 10, 2, 5, 7, 8, 1, 3, 4, 13, 12
; Gewichtung der Kapselarten bei der Auswahl: 16 Eintraege (Zufallszahl and 15 -> Art); M kommt oefter als Leben (+)
cap_pick:   .byte 0, 0, 1, 2, 3, 4, 5, 6, 7, 7, 8, 8, 8, 9, 9, 10

; laser_fire: zwei Schuesse an den Enden des Schlaegers starten (nur wenn beide Schlitze frei sind).
;   Schussposition in Einheiten zu 2 px: u = (px) / 2 (16-Bit-Wert padh:padl nach rechts schieben); links = Schlaegerkante + 2 px,
;   rechts = Schlaegerkante + Breite - 4 px; Y = PADY - 8.  Spielt SFX_LASER.   Veraendert A, t0, t1.
laser_fire:
        lda shact
        ora shact + 1
        bne .lf9                        ; es fliegt noch ein Schuss
        lda padl                        ; linker Schuss: Schlaegerkante + 2 px
        clc
        adc #2
        sta t0
        lda padh
        adc #0
        lsr
        ror t0
        lda t0
        sta shu
        lda padl                        ; rechter Schuss: Schlaegerkante + Breite - 4 px
        clc
        adc padw
        sta t0
        lda padh
        adc #0
        sta t1
        sec
        lda t0
        sbc #4
        sta t0
        lda t1
        sbc #0
        lsr
        ror t0
        lda t0
        sta shu + 1
        lda #PADY - 8
        sta shy
        sta shy + 1
        lda #1
        sta shact
        sta shact + 1
        lda #SFX_LASER
        jsr sfx_play
.lf9:   rts

; update_shots: Schuesse um 4 px nach oben; trifft einer einen Stein (Karte an Schussspitze), wirkt hit_brick und der Schuss endet.
;   Spalte = (u - 8) / 8 (u = px / 2, Stein 16 px = 8 Einheiten breit, Feld beginnt bei px 16 = u 8), Zeile = (y - 32) / 8.
update_shots:
        ldx #0
.us:    stx shi                          ; Schuss 0..1 merken: hit_brick veraendert X, shi (Nullseite) bleibt erhalten und wird danach neu geladen
        lda shact,x
        beq .usn
        lda shy,x
        sec
        sbc #4
        sta shy,x
        cmp #20
        bcs .us1                        ; noch im Spielfeld
        lda #0                          ; ueber der Decke: Schuss weg
        sta shact,x
        beq .usn
.us1:   cmp #32
        bcc .usn                        ; ueber dem Steinfeld
        sec
        sbc #32
        lsr
        lsr
        lsr
        cmp #12
        bcs .usn                        ; unter dem Steinfeld (kommt nicht vor, Schuesse fliegen nur aufwaerts)
        sta brow
        lda shu,x
        sec
        sbc #8
        bcc .usn                        ; links vom Steinfeld
        lsr
        lsr
        lsr
        cmp #18
        bcs .usn                        ; rechts davon
        sta bcol
        ldy brow
        lda rowmul,y
        clc
        adc bcol
        sta bidx
        tay
        lda MAP,y
        beq .usn                        ; kein Stein
        ldx shi
        pha
        lda #0
        sta shact,x                     ; Schuss endet
        pla
        jsr hit_brick
.usn:   ldx shi
        inx
        cpx #2
        bne .us
        rts

; update_cap: Kapsel um 1 px fallen lassen und die Drehphase capfr weiterzaehlen (0..23); faengt der Schlaeger sie (Ueberlappung in X und Y,
;   capy zwischen PADY - 12 und PADY + 5), wirkt sie (activate).  Faellt sie unter capy = 200, ist sie weg.   Veraendert A, t0, t1 (und activate).
update_cap:
        lda capact
        beq uc_end
        inc capfr                       ; Drehung: 24 Bilder = ein voller Umlauf
        lda capfr
        cmp #24
        bcc +
        lda #0
        sta capfr
+       inc capy
        lda capy
        cmp #200
        bcc +
        lda #0                          ; unten heraus: weg
        sta capact
        rts
+       cmp #PADY - 12                  ; Unterkante der Kapsel (12 px hoch) erreicht die Schlaegeroberkante?
        bcc uc_end
        cmp #PADY + 6
        bcs uc_end                        ; schon unterhalb des Schlaegers
        sec
        lda capxl
        sbc padl
        sta t0
        lda capxh
        sbc padh
        bmi .un                         ; d < 0: Kapsel liegt links der Schlaegerkante
        bne uc_end                        ; d >= 256
        lda padw
        sec
        sbc #2
        sta t1                          ; die Muenze ist 12 px breit, 2 px eingerueckt: sie beruehrt den Schlaeger, wenn d < Breite - 2
        lda t0
        cmp t1
        bcs uc_end                      ; d zu gross: rechts vorbei
        jmp activate
.un:    lda t0
        cmp #243                        ; d >= -13: noch mindestens 1 px ueberlappt
        bcc uc_end
        ; weiter mit activate

; activate: Kapsel wirkt.  Art in captype (0 B, 1 S, 2 R, 3 L, 4 F, 5 K, 6 +, 7 P, 8 M, 9 G, 10 D; siehe cap_letter).   Spielt SFX_EXTRA.
;   Dauer in Halbsekunden (tick_timers): B 30 (15 s), S 30, R 20 (10 s), L 30, F 30, G 40 (20 s), D 10 (5 s); K gilt bis zum naechsten
;   Schlaegertreffer; + erhoeht lives (hoechstens 9); P addiert 500 Punkte; M ruft multiball.
activate:
        lda #0
        sta capact
        lda #SFX_EXTRA
        jsr sfx_play
        lda captype
        beq .a0
        cmp #1
        beq .a1
        cmp #2
        beq .a2
        cmp #3
        beq .a3
        cmp #4
        beq .a4
        cmp #5
        beq .a5
        cmp #6
        beq .a6
        cmp #8
        beq .a8
        cmp #9
        beq .a9
        cmp #10
        beq .a10
        sed                             ; 7 = P: 500 Punkte (Dezimalmodus, auf die mittleren Ziffern)
        clc
        lda sc1
        adc #5
        sta sc1
        lda sc2
        adc #0
        sta sc2
        cld
        jmp draw_score
.a0:    lda #30                         ; B: breit, 15 s
        sta tpad
        lda #48
        jmp set_padw
.a1:    lda #30                         ; S: schmal, 15 s
        sta tpad
        lda #16
        jmp set_padw
.a2:    lda #20                         ; R: umgekehrte Steuerung, 10 s
        sta trev
        rts
.a3:    lda #30                         ; L: Ball langsamer, 15 s
        sta tspd
        lda #2
        sta spdmod
        rts
.a4:    lda #30                         ; F: Ball schneller, 15 s
        sta tspd
        lda #1
        sta spdmod
        rts
.a5:    lda #1                          ; K: Ball klebt beim naechsten Treffer
        sta sticky
        rts
.a10: lda #10                           ; D: Durchschlag, 5 s
        sta tpierce
        rts
.a9:    lda #40                         ; G: Laser fuer 20 s (Feuer schiesst, solange der Ball nicht am Schlaeger klebt)
        sta tlas
        rts
.a8:    jmp multiball                   ; M: aus dem Ball werden bis zu drei
.a6:    lda lives                       ; +: ein Ball mehr (hoechstens 9)
        cmp #9
        bcs uc_end
        inc lives
        jmp draw_hud
uc_end: rts

; ---------------------------------------------------------------------------------------------------------------------
; Spielende
; ---------------------------------------------------------------------------------------------------------------------
; game_over: Spielende.  Blendet die Sprites aus, zeigt "SPIEL ZU ENDE", uebergibt Punkte und Level an die Bestenlisten-Bibliothek
;   (hs_points = sc0..sc2, hs_level = Level 1..24) und prueft (hs_check), ob ein Platz erreicht wurde (nicht in der Demo).  Wenn ja:
;   Namen eingeben (name_entry), eintragen (hs_insert), speichern (hs_save, mit gesperrtem Interrupt) und "gespeichert" bzw. "nicht
;   gespeichert!" melden.  Zum Schluss 3 Sekunden (150 Bilder) anzeigen.   Veraendert alles.
game_over:
        lda #0
        sta VIC_SPR_ENABLE              ; Ball und Schlaeger ausblenden
        sta VIC_BORDERCOLOR             ; ein laufender Randblitz endet
        lda #1
        sta txtcol
        lda #<g_over
        sta tp
        lda #>g_over
        sta tp + 1
        ldx #14
        ldy #18
        jsr puts
        lda sc0                         ; Punkte (BCD, niedrig bis hoch) in die Puffer der Bestenlisten-Bibliothek
        sta hs_points
        lda sc1
        sta hs_points + 1
        lda sc2
        sta hs_points + 2
        lda level                       ; erreichtes Level (1..)
        clc
        adc #1
        sta hs_level
        lda demo
        bne .nohs                       ; Demo-Modus: keine Bestenliste
        jsr hs_check                    ; A = Platz 0..9 oder $FF (nicht in der Liste)
        cmp #$ff
        beq .nohs
        pha                             ; Platz merken
        lda #7                          ; gelb
        sta txtcol
        lda #<g_new
        sta tp
        lda #>g_new
        sta tp + 1
        ldx #13
        ldy #20
        jsr puts
        jsr name_entry                  ; Name eingeben (Vorgabe SPIELER, Auto-Bestaetigung nach 30 s)
        pla
        jsr hs_insert                   ; Eintrag an den Platz setzen, Rest rutscht nach unten
        sei                             ; die Diskettenroutinen sind zeitkritisch: ohne Rasterinterrupt arbeiten
        jsr hs_save                     ; auf Diskette schreiben; A = HS_OK oder Fehlercode
        cli
        ldx #1
        stx txtcol
        cmp #HS_OK
        bne .bad
        lda #<g_saved
        sta tp
        lda #>g_saved
        sta tp + 1
        bne .msg                        ; (immer: High-Byte der Adresse ist nicht 0)
.bad:   lda #<g_nosave                  ; Speichern ging schief (keine Diskette, schreibgeschuetzt, ...): nur melden
        sta tp
        lda #>g_nosave
        sta tp + 1
.msg:   ldx #11
        ldy #23
        jsr puts
.nohs:  ldx #150                        ; drei Sekunden anzeigen
        jmp wait_frames

g_over:  .screen "spiel zu ende"
         .byte 0
g_new:   .screen "neuer rekord!"
         .byte 0
g_saved: .screen "gespeichert"
         .byte 0
g_nosave: .screen "nicht gespeichert!"
         .byte 0
g_name:  .screen "name:"
         .byte 0
g_help:  .screen "tippen + return  oder  joystick"
         .byte 0

; ---------------------------------------------------------------------------------------------------------------------
; Namenseingabe: 8 Zeichen, Vorgabe "SPIELER".
;   Bedienung: hoch/runter (Joystick oder Q/A) wechselt das Zeichen an der Cursorstelle (Leerzeichen, A-Z, 0-9, im Kreis),
;   rechts/links (Joystick oder X/Z) bewegt den Cursor, Feuer bestaetigt.  Haelt man eine Richtung, wiederholt sie sich
;   (nach 25 Bildern, dann alle 6).  Auf der Tastatur tippt man die Zeichen direkt (A-Z, 0-9, Leerzeichen, -, .; das erste getippte
;   Zeichen loescht die Vorgabe), DEL loescht, RETURN bestaetigt.  Ohne jede Eingabe wird nach 30 s (1500 Bilder bei 50 Hz) automatisch
;   bestaetigt; jede Eingabe stellt den Zaehler zurueck.  Der Name steht als PETSCII-Grossbuchstaben (fuer A-Z, 0-9 und Leerzeichen
;   gleich ASCII) in hs_name (8 Byte).   Eingabe: keine.   Veraendert alles; wartet in einer Schleife (wait_frame je Durchlauf).
; ---------------------------------------------------------------------------------------------------------------------
name_entry:
        ldx #7
.nd:    lda hs_defname,x                ; Vorgabe "SPIELER " in den Namenspuffer
        sta hs_name,x
        dex
        bpl .nd
        lda #0
        sta npos
        sta nprev
        sta nrep
        sta ntyped
        lda #<1500
        sta ntl
        lda #>1500
        sta nth
        lda #1
        sta txtcol
        lda #<g_name
        sta tp
        lda #>g_name
        sta tp + 1
        ldx #10
        ldy #22
        jsr puts
        lda #<g_help
        sta tp
        lda #>g_help
        sta tp + 1
        ldx #5
        ldy #24
        jsr puts
        jsr input_wait_release          ; Feuer vom Spiel noch gedrueckt? -> loslassen lassen
        jsr name_draw
.ne:    jsr wait_frame
        jsr input_char                  ; Tastatur: getipptes Zeichen (A-Z, 0-9, Leerzeichen, -, .), RETURN ($0D), DEL ($14)
        sta nch
        jsr input_joy                   ; Joystick: nur Port 2 (Z/X/Q/A sind hier Buchstaben und keine Richtungen)
        sta mask
        ora nch
        bne .press
        lda ntl                         ; keine Eingabe: Zaehler herunterzaehlen (16 Bit)
        ora nth
        beq .done                       ; 0 erreicht: automatisch bestaetigen
        lda ntl
        bne +
        dec nth
+       dec ntl
        lda #0
        sta nprev                       ; Taste losgelassen: naechster Druck zaehlt wieder als neu
        jmp .ne
.press: lda #<1500                      ; irgendeine Eingabe: Auto-Bestaetigung neu starten
        sta ntl
        lda #>1500
        sta nth
        lda nch
        beq .joy                        ; kein Zeichen getippt: Joystick
        cmp #$0d
        beq .done                       ; RETURN = fertig
        cmp #$14
        beq .del                        ; DEL = zeichen loeschen
        pha                             ; ein Zeichen: bei der ERSTEN Eingabe die Vorgabe "SPIELER" ganz loeschen
        lda ntyped
        bne .typ
        lda #1
        sta ntyped
        lda #0
        sta npos
        ldx #7
        lda #CH_SPACE
.tc:    sta hs_name,x                   ; (Leerzeichen als ASCII = $20 = auch Bildschirmcode 32)
        dex
        bpl .tc
.typ:   pla
        ldy npos
        sta hs_name,y                   ; Zeichen an die Cursorstelle
        cpy #7
        bcs .tdraw                      ; achte Stelle erreicht: bleibt dort (weitere Zeichen ersetzen sie)
        inc npos
.tdraw: jsr name_draw
        jmp .ne
.del:   lda #1                          ; DEL zaehlt auch als "es wird etwas Eigenes eingegeben": Vorgabe loeschen
        ldx ntyped
        bne .dl
        sta ntyped
        lda #0
        sta npos
        ldx #7
        lda #CH_SPACE
.dc:    sta hs_name,x
        dex
        bpl .dc
        jmp .tdraw
.dl:    ldy npos
        lda hs_name,y
        cmp #CH_SPACE
        beq .dprev
        lda #CH_SPACE                   ; Zeichen unter dem Cursor (achte Stelle) loeschen
        sta hs_name,y
        jmp .tdraw
.dprev: tya
        beq .tdraw                      ; schon ganz links
        dec npos
        ldy npos
        lda #CH_SPACE
        sta hs_name,y
        jmp .tdraw
.joy:   lda mask
        cmp nprev
        bne .new                        ; andere Taste als im vorigen Bild: neuer Druck
        dec nrep                        ; gleiche Taste gehalten: erst nach der Verzoegerung wiederholen
        bne .ne
        lda #6
        sta nrep
        jmp .do
.new:   sta nprev
        lda #25
        sta nrep
.do:    lda mask
        and #IN_FIRE
        bne .done                       ; Feuer = fertig
        lda mask
        and #IN_UP
        beq +
        jsr ch_up
+       lda mask
        and #IN_DOWN
        beq +
        jsr ch_down
+       lda mask
        and #IN_RIGHT
        beq +
        lda npos
        cmp #7
        bcs +                           ; ganz rechts: bleibt
        inc npos
+       lda mask
        and #IN_LEFT
        beq +
        lda npos
        beq +                           ; ganz links: bleibt
        dec npos
+       jsr name_draw
        jmp .ne
.done:  rts

; cur_idx: Index des Zeichens an der Cursorstelle in der Zeichenfolge ctab (nicht gefunden: 0).  Rueckgabe X = Index, Y = npos.   Veraendert A.
cur_idx:
        ldy npos
        lda hs_name,y
        ldx #0
.ci:    cmp ctab,x
        beq .ck
        inx
        cpx #37
        bne .ci
        ldx #0
.ck:    rts
; ch_up / ch_down: naechstes / vorheriges Zeichen der Folge ctab (im Kreis, 37 Zeichen) an der Cursorstelle einsetzen.   Veraendert A, X, Y.
ch_up:  jsr cur_idx
        inx
        cpx #37
        bcc +
        ldx #0
+       lda ctab,x
        sta hs_name,y
        rts
ch_down:
        jsr cur_idx
        dex
        bpl +
        ldx #36
+       lda ctab,x
        sta hs_name,y
        rts
ctab:   .text " ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"      ; Zeichenfolge der Joystick-Eingabe (37 Zeichen, ASCII = PETSCII fuer diese Zeichen)

; name_draw: den Namen in Zeile 22 ab Spalte 16 anzeigen; das Zeichen unter dem Cursor in Umkehrschrift (Bit 7 des Bildschirmcodes).
;   ASCII/PETSCII -> Bildschirmcode: Buchstaben (65..90) minus 64, Ziffern, Leerzeichen, '-' und '.' unveraendert.   Veraendert A, X, Y, t1, txtcol.
name_draw:
        lda #1
        sta txtcol
        ldx #0
.nr:    stx t1
        lda hs_name,x
        cmp #64
        bcc +
        sec
        sbc #64
+       cpx npos
        bne +
        ora #$80                        ; Cursorstelle invers
+       pha
        txa
        clc
        adc #16
        tax
        pla
        ldy #22
        jsr put_cell
        ldx t1
        inx
        cpx #8
        bne .nr
        rts

; =====================================================================================================================
; Ticker: eine Textzeile (32 Zeichen breit) mit wechselnden Texten und Uebergangseffekten.
;   Der Titel zeigt darin den Untertitel (Zeile 10), die Demo die Kopfzeile (Zeile 0, statt des Punktestands).
;   TECHNIK: Die 32 Bildschirmzeichen der Zeile (Spalte 4..35) zeigen die Zeichen 224..255 des Zeichensatzes; deren Bitmuster (256 Byte bei
;   $8F00, je Zeichen 8 Byte, ein Byte je Pixelzeile) werden in jedem Schritt neu berechnet.  So sind Bewegungen um einzelne BILDPUNKTE moeglich,
;   obwohl nur Zeichen im Raster sichtbar sind.  Die Bitmuster der Buchstaben holen wir aus dem ins RAM kopierten ROM-Zeichensatz ($8800 + Code * 8).
;   EFFEKTE (zufaellig gewaehlt): 0 nach oben wegscrollen (neu von unten), 1 nach unten, 2 buchstabenweise (erst rechts nach links loeschen,
;   dann links nach rechts schreiben), 3 nach links wegscrollen (neu von rechts), 4 nach rechts, 5 Wischer (Kante laeuft pixelweise von links
;   nach rechts), 6 Aufloesen (zufaellige Pixelzeilen werden ausgetauscht).  Dazu eine laufende Regenbogenfarbe ueber die Zeichen.
;   ABLAUF: warten (tkw Bilder), naechsten Text aus der Folge holen, Effekt starten, Schritte ausfuehren, fertig -> warten.
; Zustand (Nullseite): tkrow Zeile, tkst 0 = warten / 1 = Effekt, tkw Wartebilder, tkeff Effekt, tkstep Schritt, tkmsg Folgenindex, tkseq 0 Titel / 1 Demo
; Texte: tk_msgs (3 Texte zu 32 Bildschirmcodes); Folgen: tk_seqs/tk_seq/tk_seqlen.  Anzeige/Ziel: tk_cur / tk_new (je 32 Byte, $C140/$C160).
; Hinweis: Die Ticker-Zeichen liegen im RAM-Zeichensatz bei $8F00; der Rest des Zeichensatzes ($8800-$8EFF) bleibt unveraendert.
; =====================================================================================================================
.const TK_BUF = $8f00                   ; Bitmuster der Zeichen 224..255 ($8800 + 224 * 8)
.const TK_CHR = 224                     ; erstes Ticker-Zeichen
.const tk_cur = $c140                   ; aktuell angezeigter Text (32 Bildschirmcodes)
.const tk_new = $c160                   ; Zieltext

; ticker_init: A = Bildschirmzeile, X = Folge (0 Titel: Texte 0, 1;  1 Demo: Texte 2, 0, 1).  Zeigt den ersten Text sofort.
;   Setzt auch die Bildschirmzeichen 224..255 in Spalte 4..35 der Zeile (gelb) und den ersten Wechsel in 4 s.   Veraendert A, X, Y, tk*, sp, cp.
ticker_init:
        sta tkrow
        stx tkseq
        lda #0
        sta tkmsg
        sta tkst
        sta tkph
        lda #200
        sta tkw                         ; erster Wechsel nach 4 s
        lda #$41                        ; Zufallsstartwert fuer das Aufloesen (nicht 0)
        sta tklf
        ldx tkseq
        lda tk_seq,x                    ; erster Text der Folge
        jsr tk_loadnew
        ldx #31
.ti1:   lda tk_new,x                    ; Zieltext wird der angezeigte Text
        sta tk_cur,x
        dex
        bpl .ti1
        jsr tk_renderall
        ldx #4
        ldy tkrow
        jsr set_rowptr                  ; sp/cp auf Spalte 4 der Zeile
        ldy #0
.ti2:   tya
        clc
        adc #TK_CHR
        sta (sp),y                      ; Zeichen 224 + Spalte
        lda #7
        sta (cp),y
        iny
        cpy #32
        bne .ti2
        rts

; tk_loadnew: A = Textnummer (0..2): 32 Zeichen aus tk_msgs nach tk_new kopieren.
tk_loadnew:
        asl
        asl
        asl
        asl
        asl                             ; * 32
        tax
        ldy #0
.tl:    lda tk_msgs,x
        sta tk_new,y
        inx
        iny
        cpy #32
        bne .tl
        rts

; tk_ga / tk_gb: A = Bildschirmcode -> tka / tkb = Adresse des Bitmusters im Zeichensatz ($8800 + Code * 8).
tk_ga:  sta tka
        lda #0
        sta tka + 1
        asl tka
        rol tka + 1
        asl tka
        rol tka + 1
        asl tka
        rol tka + 1
        lda tka + 1
        clc
        adc #$88
        sta tka + 1
        rts
tk_gb:  sta tkb
        lda #0
        sta tkb + 1
        asl tkb
        rol tkb + 1
        asl tkb
        rol tkb + 1
        asl tkb
        rol tkb + 1
        lda tkb + 1
        clc
        adc #$88
        sta tkb + 1
        rts

; tk_renderchar: Zeichen tkc (0..31) des angezeigten Textes tk_cur in den Zeichensatz kopieren (8 Pixelzeilen).
tk_renderchar:
        ldx tkc
        lda tk_cur,x
        jsr tk_ga
        lda tkc
        asl
        asl
        asl
        tax                             ; X = tkc * 8 = Anfang im Puffer
        ldy #0
.rc:    lda (tka),y
        sta TK_BUF,x
        inx
        iny
        cpy #8
        bne .rc
        rts

; tk_renderall: alle 32 Zeichen neu aus tk_cur zeichnen.
tk_renderall:
        lda #0
        sta tkc
.ra:    jsr tk_renderchar
        inc tkc
        lda tkc
        cmp #32
        bne .ra
        rts

; tk_sync: der Zieltext wird der angezeigte Text (Effekt zu Ende), sauber neu gezeichnet.
tk_sync:
        ldx #31
.sy:    lda tk_new,x
        sta tk_cur,x
        dex
        bpl .sy
        jmp tk_renderall

; ---- Effekt 0/1: senkrecht pixelweise.  tkd = Versatz d (+1..+8 nach oben, -1..-8 nach unten): Pixelzeile r zeigt Zeile r + d des alten Textes,
;   liegt r + d ausserhalb 0..7, die Zeile (r + d) mod 8 des neuen Textes.
tk_vert:                                ; verarbeitet die Zeichen tkc .. tkce - 1 (ein Viertel je Bild, damit ein Bild nicht zu lang dauert)
.vc:    ldx tkc
        lda tk_cur,x
        jsr tk_ga
        ldx tkc
        lda tk_new,x
        jsr tk_gb
        lda tkc
        asl
        asl
        asl
        sta tkt                         ; Zielindex im Puffer
        lda #0
        sta tkr
.vr:    lda tkr
        clc
        adc tkd
        sta tks                         ; s = r + d
        and #$f8
        bne .vn                         ; ausserhalb 0..7: neuer Text
        ldy tks
        lda (tka),y
        jmp .vs
.vn:    lda tks
        and #7
        tay
        lda (tkb),y
.vs:    ldy tkt
        sta TK_BUF,y
        inc tkt
        inc tkr
        lda tkr
        cmp #8
        bne .vr
        inc tkc
        lda tkc
        cmp tkce
        bne .vc
        rts

; ---- Effekt 2: buchstabenweise.  Schritt 0..31: Zeichen 31 - n loeschen (Leerzeichen), Schritt 32..63: Zeichen n - 32 aus dem neuen Text setzen.
tk_type:
        lda tkstep
        cmp #32
        bcs .ty2
        eor #$ff                        ; 31 - n  (n < 32)
        clc
        adc #32
        sta tkc
        tax
        lda #32
        sta tk_cur,x
        jmp tk_renderchar
.ty2:   sec
        sbc #32
        sta tkc
        tax
        lda tk_new,x
        sta tk_cur,x
        jmp tk_renderchar

; ---- Effekt 3: nach links wegscrollen, neuer Text kommt von rechts (je Schritt 4 Pixel, 64 Schritte).
;   Neue Pixel rechts: Spaltengruppe j = Schritt des NEUEN Textes (Zeichen j / 2, rechte oder linke Haelfte).
tk_hleft:
        lda tkstep
        lsr
        tax
        stx tkt                         ; Zeichen des neuen Textes
        lda tk_new,x
        jsr tk_gb
        lda tkrs
        sta tkr                         ; nur die Pixelzeilen tkrs .. tkre - 1 (halbe Zeichenhoehe je Bild)
.hl1:   ldy tkr
        lda (tkb),y                     ; Pixelzeile des Zeichens
        ldx tkstep
        txa
        and #1
        bne .hl2
        lda (tkb),y                     ; gerader Schritt: linke Haelfte (obere 4 Bit) nach unten
        lsr
        lsr
        lsr
        lsr
        jmp .hl3
.hl2:   lda (tkb),y
        and #$0f                        ; ungerader Schritt: rechte Haelfte
.hl3:   sta tks                         ; neue 4 Pixel
        ldx #0                          ; Zeichen 0..30
        lda tkr
        tay                             ; Y = Index (Zeichen * 8 + Zeile), laeuft in Schritten von 8
.hl4:   lda TK_BUF,y
        asl
        asl
        asl
        asl
        sta tkt
        lda TK_BUF + 8,y
        lsr
        lsr
        lsr
        lsr
        ora tkt
        sta TK_BUF,y
        tya
        clc
        adc #8
        tay
        inx
        cpx #31
        bne .hl4
        lda TK_BUF,y                    ; Zeichen 31: von rechts neue Pixel
        asl
        asl
        asl
        asl
        ora tks
        sta TK_BUF,y
        inc tkr
        lda tkr
        cmp tkre
        bne .hl1
        rts

; ---- Effekt 4: nach rechts wegscrollen, neuer Text kommt von links (die rechten Spaltengruppen des neuen Textes zuerst).
tk_hright:
        lda #63
        sec
        sbc tkstep
        sta tkt                         ; j = 63 - Schritt
        lsr
        tax
        lda tk_new,x
        jsr tk_gb
        lda tkrs
        sta tkr
.hr1:   ldy tkr
        lda tkt
        and #1
        bne .hr2
        lda (tkb),y                     ; gerades j: obere Haelfte (linke 4 Pixel des Zeichens)
        lsr
        lsr
        lsr
        lsr
        jmp .hr3
.hr2:   lda (tkb),y
        and #$0f
.hr3:   asl
        asl
        asl
        asl
        sta tks                         ; neue Pixel stehen in den oberen 4 Bit (ganz links)
        ldx #31                         ; Zeichen 31 .. 1: b[c] = (b[c] >> 4) | (b[c-1] << 4)
        lda tkr
        clc
        adc #248
        tay                             ; Y = 31 * 8 + Zeile
.hr4:   lda TK_BUF,y
        lsr
        lsr
        lsr
        lsr
        sta tkd2
        lda TK_BUF - 8,y
        asl
        asl
        asl
        asl
        ora tkd2
        sta TK_BUF,y
        tya
        sec
        sbc #8
        tay
        dex
        bne .hr4
        lda TK_BUF,y                    ; Zeichen 0: b[0] = (b[0] >> 4) | neue Pixel links
        lsr
        lsr
        lsr
        lsr
        ora tks
        sta TK_BUF,y
        inc tkr
        lda tkr
        cmp tkre
        bne .hr1
        rts

; ---- Effekt 5: Wischer.  Schritt n (0..63): die Kante liegt bei Pixel 4 * (n + 1); komplette Zeichen davor sind neu, das Zeichen an der Kante
;   ist bei ungerader Kante nur zur Haelfte neu (obere 4 Bit von links).
tk_wipe:
        lda tkstep
        clc
        adc #1                          ; k = n + 1
        lsr                             ; k / 2 = Zeichenindex der Kante, Uebertrag = Haelfte
        bcc .wf
        sta tkc                         ; ungerade: Zeichen k/2 halb neu, halb alt
        tax
        lda tk_cur,x
        jsr tk_ga
        ldx tkc
        lda tk_new,x
        jsr tk_gb
        lda tkc
        asl
        asl
        asl
        tax
        ldy #0
.wh:    lda (tkb),y
        and #$f0
        sta tks
        lda (tka),y
        and #$0f
        ora tks
        sta TK_BUF,x
        inx
        iny
        cpy #8
        bne .wh
        rts
.wf:    sec                             ; gerade: Zeichen k/2 - 1 ist jetzt ganz neu
        sbc #1
        sta tkc
        tax
        lda tk_new,x
        sta tk_cur,x
        jmp tk_renderchar

; ---- Effekt 6: Aufloesen.  Je Schritt 8 Pixelzeilen in der Reihenfolge eines 8-Bit-Schieberegisters (alle 255 Werte ausser 0) austauschen.
tk_dissolve:
        lda #8
        sta tkt
.dd:    lda tklf                        ; Galois-LFSR, Polynom $B8
        lsr
        bcc .d1
        eor #$b8
.d1:    sta tklf
        sta tks                         ; Index 1..255 = Zeichen * 8 + Zeile
        lsr
        lsr
        lsr
        tax                             ; Zeichen
        lda tk_new,x
        jsr tk_gb
        lda tks
        and #7
        tay
        lda (tkb),y
        ldy tks
        sta TK_BUF,y
        dec tkt
        bne .dd
        rts

; tk_half: A = 0 / 1 -> tkrs/tkre = Pixelzeilen 0-3 bzw. 4-7.
tk_half:
        asl
        asl
        sta tkrs
        clc
        adc #4
        sta tkre
        rts

; ---- ticker_tick: einmal je Bild (Titel: title; Demo: play).  Zwei Aufgaben:
;   (1) alle 2 Bilder die Regenbogenfarbe um einen Schritt ueber die 32 Zeichen weiterschieben (Farb-RAM der Tickerzeile, Spalte 4..35, tk_pal);
;   (2) den Ablauf: tkst = 0: tkw Bilder warten; dann den naechsten Text der Folge holen (tkmsg), einen Effekt 0..6 zufaellig waehlen
;   (die Zufallszahl 7 wird zu 0) und tkst = 1 setzen.  tkst = 1: je Bild einen Teilschritt des Effekts (die teuren Effekte sind ueber 2 bis 4
;   Bilder verteilt, tkdiv), nach dem letzten Schritt tk_sync und 4 s warten.   Veraendert A, X, Y und die tk*-Zellen.
ticker_tick:
        inc tkph
        lda tkph
        and #1
        bne .tkc9
        lda tkph                        ; Regenbogenfarbe ueber die 32 Zeichen, wandert alle 2 Bilder weiter (Farben < 8: normale Zeichen)
        lsr
        sta tkt
        ldx #4
        ldy tkrow
        jsr set_rowptr
        ldy #0
.tkc1:  tya
        clc
        adc tkt
        and #7
        tax
        lda tk_pal,x
        sta (cp),y
        iny
        cpy #32
        bne .tkc1
.tkc9:  lda tkst
        bne .tkrun
        dec tkw
        bne .tkret
        lda #1                          ; Wartezeit um: naechsten Text und einen Effekt waehlen
        sta tkst
        lda #0
        sta tkstep
        sta tkdiv
        inc tkmsg
        ldx tkseq
        lda tk_seqlen,x
        cmp tkmsg
        bne .tkn1
        lda #0
        sta tkmsg
.tkn1:  lda tkseq
        asl
        asl                             ; Folge * 4 (jede Folge hat Platz fuer 4 Eintraege)
        clc
        adc tkmsg
        tax
        lda tk_seqs,x
        jsr tk_loadnew
        jsr rand_byte
        and #7
        cmp #7
        bcc .tkn2
        lda #0
.tkn2:  sta tkeff
        lda #0
        sta tkd                         ; senkrechte Effekte starten bei d = 0 (ticker_tick rechnet d ab dem ersten Schritt neu aus)
.tkret: rts
.tkrun: lda tkeff                       ; die teuren Effekte werden auf mehrere Bilder verteilt (Rechenzeit je Bild: sonst ruckelt das Spiel)
        cmp #2
        bcs .tkfast
        ; ---- senkrecht (0, 1): ein Viertel der Zeichen je Bild, nach dem vierten Viertel zaehlt der Schritt weiter
        lda tkdiv
        asl
        asl
        asl
        sta tkc                         ; erstes Zeichen dieses Viertels (0, 8, 16, 24)
        clc
        adc #8
        sta tkce
        lda tkeff
        bne .vdown
        lda tkstep
        clc
        adc #1                          ; nach oben: d = Schritt + 1 (1..8)
        sta tkd
        jmp .vgo
.vdown: lda #$ff
        sec
        sbc tkstep                      ; nach unten: d = -(Schritt + 1)
        sta tkd
.vgo:   jsr tk_vert
        inc tkdiv
        lda tkdiv
        cmp #4
        bcc .tkret                      ; naechstes Viertel im naechsten Bild
        lda #0
        sta tkdiv
        inc tkstep
        lda tkstep
        cmp #8
        bcs .tkdone
        rts
.tkfast:
        lda tkeff
        cmp #2
        beq .e2
        cmp #3
        beq .e3
        cmp #4
        beq .e4
        cmp #5
        beq .e5
        jmp .e6
.e2:    jsr tk_type
        jmp .tkadv64
.e3:    lda tkdiv                       ; links: je Bild eine Haelfte der Pixelzeilen (0-3, 4-7), nach beiden Haelften ein Schritt
        jsr tk_half
        jsr tk_hleft
        jmp .hnext
.e4:    lda tkdiv
        jsr tk_half
        jsr tk_hright
.hnext: inc tkdiv
        lda tkdiv
        cmp #2
        bcc .tkret
        lda #0
        sta tkdiv
        jmp .tkadv64
.e5:    jsr tk_wipe
.tkadv64:
        inc tkstep
        lda tkstep
        cmp #64
        bcs .tkdone
        rts
.e6:    jsr tk_dissolve
        inc tkstep
        lda tkstep
        cmp #33                         ; 33 * 8 = 264 >= 255 Zeilen
        bcs .tkdone
        rts
.tkdone:
        jsr tk_sync                     ; Text endgueltig sauber setzen
        lda #0
        sta tkst
        lda #200                        ; 4 s Wartezeit bis zum naechsten Wechsel
        sta tkw
        rts

tk_pal: .byte 1, 7, 5, 3, 6, 4, 2, 7                     ; Regenbogen aus Farben < 8
tk_seqs: .byte 0, 1, 0, 0                                 ; Folge Titel (Texte 0, 1)
         .byte 2, 0, 1, 0                                 ; Folge Demo (Texte 2, 0, 1)
tk_seq:  .byte 0, 2                                       ; erster Text je Folge
tk_seqlen: .byte 2, 3                                     ; Laenge je Folge
; Texte, je 32 Zeichen, mittig aufgefuellt (Kleinbuchstaben = Bildschirmcodes 1..26, siehe Hinweis zu .screen)
tk_msgs:
        .screen "  pralle action fuer den c128   "
        .screen " ein demospiel von vintagecoder "
        .screen "       *** prallball ***        "

; ---------------------------------------------------------------------------------------------------------------------
; Bibliotheken und Daten
; ---------------------------------------------------------------------------------------------------------------------
.include "lib/input.asm"
.include "lib/rand.asm"
.include "lib/sound.asm"
.include "lib/dirtable.inc"             ; dir_xl, dir_xh, dir_yl, dir_yh (erzeugt von tools/gen.py): Geschwindigkeit je Stufe * 8 + Zone, 4 x 128 Byte
.include "lib/gfx.inc.chars"            ; gfx_chars: Bitmuster der Zeichen 64..67 (32 Byte), von copy_font nach $8A00 kopiert

; ---- Daten oberhalb des Programms.  Die PRG-Datei ist zusammenhaengend: der Assembler fuellt die Luecken zwischen den .org-Adressen mit Nullen.
.org $8000                              ; Sprite-Daten (VIC-Bank 2, Zeiger $00..$0A = (Adresse - $8000) / 64): Ball, Schlaegerteile, Kapsel, Schuss, Kapselphasen
.include "lib/gfx.inc.sprites"
.org $9400                              ; (Levels und Lied: nur der Prozessor liest sie; der VIC sieht bei $9000-$9FFF ohnehin nur das Zeichen-ROM)
.include "lib/levels.inc"               ; level_count, levelN, level_tab_lo/hi
.include "lib/music.inc"                ; song: Titelmusik (erzeugt von tools/compose.py)
