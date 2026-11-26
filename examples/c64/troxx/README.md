# Troxx (C64)

Eigenständiges Arcade-Beispiel für VCASM auf dem Commodore 64: Eine Figur fährt die Linien eines Gitters (8 × 5 Rechtecke) ab und färbt sie. Sind alle vier Kanten eines Rechtecks abgefahren, wird es gefüllt. Gegner laufen auf denselben Linien und kosten ein Leben. Die Regeln sind die der Referenzfassung für den VC-20 (`examples/vic20/troxx`); der C64 bringt mehr Leistung und damit Hardware-Sprites mit Pixelbewegung, einen Mehrfarben-Zeichensatz, SID-Klang und Zusatzfunktionen. Konzept: `assets/docs/Konzept-Beispiel-VC20-Gittermosaik.md` (Arbeitstitel „Gittermosaik“, Abschnitte 15 und 16), Mockup `assets/docs/mockups/gittermosaik/gm-5-c64.png`.

**Stand:** Welle W2 des Konzepts, nur im Emulator (VICE `x64sc`, PAL und NTSC) geprüft, nicht an echter Hardware. Handprüfliste für den Inhaber: `HANDTEST.md`. Prüfsummen der Ausgabe: `PRUEFSUMMEN.txt`.

## Bedienung

Starten: `vcc build` im Projektordner, dann **`build/disk.d64`** im Emulator starten (`x64sc build/disk.d64`, Autostart), damit die Bestenliste auf derselben Diskette landet. Ohne Diskette (nur `build/main.prg`) läuft das Spiel genauso, nur ohne Bestenliste. Joystick im Anschluss 1 oder 2 (Emulator: `-joydev1 1` oder `-joydev2 1` = Ziffernblock, Feuer = Ziffernblock 0 oder rechte Alt-Taste).

**Titelbild:** Joystick hoch/runter wählt die Zeile, links/rechts ändert den Wert, Feuer startet.

| Zeile | Werte |
|---|---|
| PLAYERS | 1 oder 2 (zwei Spieler **abwechselnd**: nach jedem verlorenen Leben ist der andere dran, jeder hat sein eigenes Gitter, seine Punkte und Leben) |
| JOYSTICK | `PORT 2` (ein Joystick am Anschluss 2, Standard), `PORT 1` (ein Joystick am Anschluss 1), `TWO STICKS` (nur mit zwei Spielern: Spieler 1 am Anschluss 1, Spieler 2 am Anschluss 2) |
| SPEED | Tempo 1–9 |

Bei **einem Joystick** und zwei Spielern erscheint vor jedem Zug „pass joystick – fire“: Joystick weiterreichen, Feuer drücken. Die Wahl bleibt bis zum Ausschalten im RAM. Unter den Einstellungen steht die Bestenliste, rechts neben „HIGH SCORES“ das Ergebnis des Plattenzugriffs (`DISK OK`, `NO DRIVE`, `NO FILE`, `DISK ERR`).

**Spiel:** Die Figur läuft nur, solange eine Richtung gehalten wird; an Knoten (Kreuzungen und Ecken) wechselt sie die Richtung, auf einer Linie sind nur vor und zurück möglich. Feuer = Pause, nochmal Feuer = weiter. Die Seitenleiste (Spalten 34–39) zeigt SCORE, HI (beste Punkte der Bestenliste), LEVEL, LIVES (Herzen, höchstens sechs sichtbar), SPEED, BONUS (Gegenstand auf dem Feld) und bei zwei Spielern PLAYER. Meldungen stehen in Zeile 24.

**Hinweis zur Tastatur:** Die Tastaturmatrix liegt auf denselben Leitungen wie die Joysticks. Während des Spiels keine Taste drücken (sie würde einen Joystick-Kontakt vortäuschen). Die Tastatur wird nicht abgefragt.

## Regeln (wie die Referenzfassung)

- Gitter 8 × 5 Rechtecke, Innenraum 3 × 3 Zeichen (33 × 21 Zellen); 93 Segmente (48 waagerecht, 45 senkrecht), 40 Rechtecke. Nummerierung H(r, c) = 8 r + c, V(r, c) = 48 + 9 r + c, B(r, c) = 8 r + c.
- Punkte: 1 je neu gefärbtes Zeichen, 10 je abgeschlossenes Segment, 100 je gefülltes Rechteck, **Kettenreaktion**: schließt ein Segment zwei Rechtecke zugleich, kommen 200 dazu (zusammen 400), 500 je Stufe, Bonusstern 50. Extraleben je 10 000 Punkte, höchstens 9 Leben; Start mit 3 Leben.
- **Spielregel 16.2 (Gegner zerstören Linien):** Betritt ein Gegner ein Innenzeichen eines Segments, das an **kein gefülltes Rechteck** grenzt, wird das ganze Segment zurückgesetzt (alle Zeichen wieder violett, bei abgeschlossenem Segment `BOXCNT` der Nachbarn − 1); die dafür gebuchten Punkte werden abgezogen (1 je gefärbtes Zeichen, 10 mehr bei abgeschlossenem Segment, nie unter 0). Knoten zerstören nichts, eingefrorene Gegner (Eisblock) auch nicht; gilt für alle Gegnertypen. Rückmeldung: die Zeichen blitzen weiß, Rasterfarbblitz im Rand (rot/weiß), Punktzahl kurz rot, eigener Klang (Knistern und fallender Ton). Schalter `ENEMY_DESTROYS_LINES` (Standard 1, 0 = alte Regel). **Erweiterung W-T1:** Erreicht ein Gegner einen **Knoten**, werden alle dort endenden, nicht geschützten Segmente mit mindestens einem gelben Zeichen ebenso zerstört (je Segment wie oben; die Gemeinsame Logik `enemies.touch` stammt aus der Referenzfassung). **Knoten:** Sie färben sich gelb, wenn die Figur sie betritt (`grid.paint_node`; beim VC-20 geschieht das beim Verlassen durch `erase`, gleiches Ergebnis). Nach jeder Zerstörung setzt `grid.nodes_free` freistehende gelbe Knoten zurück (violett), wenn keine der acht Nachbarzellen gelb oder ein Füllmuster ist (kein gelbes Segment schließt mehr an, kein angrenzendes Rechteck gefüllt); ein Knoten mit Figur oder Gegner bleibt, Knoten zählen nicht für Punkte, kein Zeitablauf. Der Läufer behält im C64 die genauere Verfolgung (`greedy`); die Referenz wählt `toward`. Der Streuner setzt nicht mehr jeden vierten Schritt aus (wie in der Referenzfassung).
- Gegner nur auf den Linien, **Entscheidung nur an Knoten** (jede Richtung außer der Umkehr): **Läufer** (rot) wählt mit der Wahrscheinlichkeit PROB (50–90 % je Stufe) die Richtung, die den Abstand zur Figur am stärksten verkleinert, sonst Zufall; **Streuner** (orange) immer Zufall,; **Wächter** (cyan, Zusatzfunktion) geradeaus, an Ecken die einzige Abbiegung: fester Rundgang um den Rand. Gegnerzahl 1, 2, 2, 3, 3, 3, 4, … je Stufe. Zufall: 8-Bit-LFSR, fester Startwert je Stufe.
- Kollision: gleiche Zelle oder Platztausch. Ein Leben weniger, Figuren auf die Startknoten, Farben und gefüllte Rechtecke bleiben, danach kurze Unverwundbarkeit (Blinken).
- Stufe geschafft, wenn alle 40 Rechtecke gefüllt sind; alle zwei Stufen Tempo + 1 bis 9. Tempo in Bildern je Spielschritt (PAL 12 … 3, NTSC ×1,2): gleiche Tabelle wie in der Referenzfassung.

### Verbesserungen gegenüber dem VC-20

- **Mehrfarben-Sprites** (Umriss schwarz, Glanz weiß, Körperfarbe je Sprite, zwei Bilder je Figur; `tools/gen_sprites.py`) für Figur, bis zu vier Gegner und Gegenstand: Die Spiellogik rechnet weiter in Zeichenzellen (Entscheidungen nur an Knoten, Kollision auf Zeichenebene); das Sprite gleitet während des Schritts in acht Punkten zur nächsten Zelle (`sprites.asm`). Bildtakt per Rasterabfrage (Zeile 251, unter dem Bild); der Rasterinterrupt ändert nur den Rahmen.
- **Mehrfarben-Zeichensatz**: Linien und Text einfarbig, die vier Füllmuster (Schraffur, Punkte, Mauer, Schachbrett) als Mehrfarben-Zeichen; Farbe nach Rechteckzeile.
- **Rasterinterrupt** (`raster.asm`): animierte Farbbalken im oberen und unteren Rand, im Titelbild Hintergrundbänder (oben und unten schwarz) und ein **weicher Bildlauf** (Zeile 6, Feinversatz in `$D016`, 38-Spalten-Modus nur in dieser Zeile); Diskettenzugriffe sperren ihn.
- **Titelbild** mit großem Logo aus abgeschrägten Mehrfarben-Blöcken (5 × 5 je Buchstabe) und Farbzyklus.
- **Füllanimation**: Zeile für Zeile mit Aufblitzen der neuesten Zeile (nur Farbspeicher, die Logik bleibt unberührt).
- **SID**: Titelmusik (drei Stimmen) mit Tiefpassfilter (Resonanz, Grenzfrequenz läuft hin und her) und Effekte Schritt, Segment, Rechteck, Kette, Treffer, Stufe, Ende, Bonusstern, Eisblock, je mit Prioritäten.
- **Zusatzfunktionen** (jeweils abschaltbar, siehe unten): Bonusstern, Eisblock, Wächter, Bestenliste auf Diskette, zwei Spieler abwechselnd mit Joystickwahl.

## Zusatzfunktionen und Schalter

Am Kopf von `main.asm` stehen die Schalter (`.const`, mit `-D NAME=0` überschreibbar). Mit allen auf 0 gelten genau die Regeln der Referenzfassung (Feldgröße und Sprites bleiben).

| Schalter | Standard | Wirkung |
|---|---|---|
| `EXTRA_STAR` | 1 | Bonusstern auf einer freien Linienzelle (Sprite, 50 Punkte), nach zehn Sekunden weg; danach vier bis fünf Sekunden Pause |
| `EXTRA_ICE` | 1 | Eisblock: alle Gegner stehen fünf Sekunden still (hellblau), die Figur läuft weiter; jeder vierte Gegenstand ist ein Eisblock |
| `EXTRA_GUARD` | 1 | dritter Gegnertyp „Wächter“ (ab Stufe 4 als dritter Gegner) |
| `EXTRA_HISCORE` | 1 | Bestenliste (zehn Einträge, Namenseingabe mit dem Joystick) auf Diskette |
| `ENEMY_DESTROYS_LINES` | 1 | Spielregel 16.2: Gegner zerstören nicht geschlossene Linien |
| `EXTRA_2P` | 1 | zwei Spieler abwechselnd, Spielerwahl im Titelbild |
| `FX_RASTER` | 1 | Rasterinterrupt (Randbalken, Hintergrundbänder, Bildlauf) |
| `FX_TITLE` | 1 | großes Logo, Farbzyklus, Bildlauf im Titelbild (braucht `FX_RASTER`) |
| `FX_FLASH` | 1 | Aufblitzen beim Füllen |
| `FX_PWM` | 1 | Pulsbreitenmodulation und Filterlauf der Titelmusik |

Test- und Dauerlaufschalter: `AUTOPLAY=1` (der Rechner spielt selbst, kein Titelbild), `AUTO_PLAYERS=2` (dabei zwei Spieler), `AUTOGOD=1` (unverwundbar), `TESTDEMO=1` (Zähler im RAM ab `$C020` und hexadezimal in Zeile 24: Bilder, Spiele, höchste Stufe), `LIVES0=n`, `TESTSCORE=$nn` (Startpunkte), `SCRIPT=1` oder `2` (Joystickeingaben aus `tools/script*.inc`, siehe unten), `TESTPAD=1` (Hüllen für die kurzen Testroutinen). Bauen einer Variante:

```bash
vcc asm main.asm --target c64 --cpu 6510 --illegals deny -o /tmp/troxx_auto.prg -D AUTOPLAY=1 -D AUTOGOD=1 -D TESTDEMO=1
```

## Bestenliste (Diskette)

Datei `TROXX.HS` auf Laufwerk 8, 77 Byte: Kennung „TXHS“ und Version 1, zehn Einträge zu 7 Byte (Name 3 Bildschirmcodes, Punkte BCD niedrig/mittel/hoch, Stufe BCD), Prüfsumme (Summe und Summe der mit `$5A` verknüpften Bytes). Gespeichert wird mit „@0:“ (ersetzt die alte Datei). Fehlt das Laufwerk oder die Datei oder stimmt die Prüfsumme nicht, gilt die Standardliste („TRX“, 10 000 bis 1 000 Punkte); jeder Fehler wird abgefangen und angezeigt, das Spiel läuft weiter. Namenseingabe: hoch/runter Buchstabe (A–Z, Leerzeichen), links/rechts Stelle, Feuer fertig.

## Speicherplan

| Bereich | Verwendung |
|---|---|
| `$0002`–`$007B` | Zeropage: Spielzustand (Plan im Kopf von `main.asm`); `$90` und höher bleibt dem KERNAL (Diskette) |
| `$0400`–`$07E7`, `$07F8` | Bildschirm 40 × 25, Sprite-Zeiger |
| `$0801`–`$2FFF` | Programm (Segment `PROGRAM`, derzeit etwa 7,4 KB) |
| `$3000`–`$37FF` | Zeichensatz-RAM: Codes 0–127 Kopie des Zeichen-ROMs (Text), 128–255 eigene Grafik |
| `$3800`–`$3AFF` | Sprite-Daten (11 Sprites zu 64 Byte, Blocknummer `$E0` ff.) |
| `$C000`–`$CFFF` | Spielvariablen, Bestenliste, Spielstand und Gitter des zweiten Spielers |

VIC-II: Bank 0, `$D018` = `$1C`, `$D016` = `$18` (Mehrfarben-Zeichen möglich), Hintergrund 0 blau, 1 schwarz, 2 dunkelgrau, Rahmen hellblau. Linien und Text haben Farbspeicher-Bit 3 = 0 (einfarbig, Farben 0–7), Füllungen Bit 3 = 1. Das Gitter liegt ab Bildschirmspalte 1 und Zeile 2 (Spalte 0 bleibt frei, damit ein Sprite am Rand nicht unter dem Rahmen verschwindet).

## Dateien

| Datei | Inhalt |
|---|---|
| `main.asm` | Schalter, Konstanten, Speicherplan, Start, Zustandsautomat (Titelbild, Spielerwechsel, Spielende) |
| `vic.asm` | Bildinit, PAL/NTSC-Erkennung, Zeichensatz und Sprites einrichten, `wait_frame` |
| `raster.asm` | Rasterinterrupt, Randbalken |
| `sprites.asm` | Sprites: Position mit Pixelbewegung, Bilder, Farben; Sprite-Daten (`sprites.inc`, erzeugt von `tools/gen_sprites.py`) |
| `grid.asm` | Datenmodell, Zeichnen, Segmente, Rechtecke, Füllen, Gitter sichern und laden |
| `player.asm`, `enemies.asm` | Bewegung, Selbstspiel, Zufall, Entscheidung, Kollision |
| `items.asm` | Bonusstern und Eisblock |
| `input.asm` | Joystick über CIA 1, Anschlusswahl |
| `sound.asm` | Klangmodul (SID), Effekte, Titelmusik |
| `hud.asm` | Seitenleiste, Meldungen, Titelbild, BCD-Punkte |
| `hiscore.asm`, `disk.asm` | Bestenliste und Plattenzugriff über das KERNAL |
| `data.inc`, `charset.asm` | Tabellen, Grafikzeichen |
| `KOPIEN.txt` | Liste der aus der Referenzfassung kopierten Routinen |
| `troxx.vctest`, `troxx_kurz.vctest` | Tests |
| `tools/` | `gen_sprites.py`, `gen_tests.py`, `build_pad.sh`, `testpad.inc`, Eingabeskripte |

Die Texte auf dem Bildschirm sind englisch und kurz (wie im Mockup); sie stehen an einer Stelle in `hud.asm`.

## Gemeinsame Logik und Kopien

Das Projekt ist in sich geschlossen (nur die Systembibliothek des Assemblers liegt außerhalb). Die systemunabhängige Logik (Nummerierung, Zähler, Zufall, BCD-Punkte, KI-Entscheidung, Tabellen) stammt aus der Referenzfassung; jede Datei nennt im Kopf ihre Herkunft („Herkunft: examples/vic20/troxx/…, Stand Commit …“). `KOPIEN.txt` führt auf, welche Routinen als Kopie gelten und welche bewusst angepasst sind. Abgleich (im Repository-Ordner):

```bash
python3 tools/troxx_kopien.py examples/c64/troxx
```

Das Werkzeug vergleicht die Kopien nach Entfernen von Kommentaren und Leerzeilen per Prüfsumme mit dem VC-20-Projekt und meldet Abweichungen (Endestatus 1). Änderungen an der gemeinsamen Logik gehören in die Referenzfassung und von dort in alle Projekte.

## Tests

```bash
vcc build                                   # baut build/main.prg und build/disk.d64
python3 tools/gen_tests.py                  # erzeugt troxx.vctest und troxx_kurz.vctest neu (nur nötig nach Änderungen an den Zeropage-Adressen)
vcc test --backend vice                     # troxx.vctest: statische Prüfungen (Größe, Überlappung, Zeropage) und Routinen im Emulator
tools/build_pad.sh vcc                      # baut build/pad.prg (mit Hüllen, siehe tools/testpad.inc)
vcc test troxx_kurz.vctest --backend vice   # sehr kurze Routinen
tools/build_regel0.sh vcc                   # baut build/regel0.prg (alte Regel)
vcc test troxx_regel0.vctest --backend vice # Test mit ENEMY_DESTROYS_LINES = 0
```

Die gemeinsame Logik besteht dieselben Testfälle wie in der Referenzfassung (`bcd_add`, `legal_mask`, `greedy`, `neighbours` für alle Segmente, `paint` mit Kettenreaktion, `fill_box`, `player.step`), dazu Tests der C64-Teile (Sprite-Lage, Eingabe, Titelmenü, Bestenliste, Klangzustand, Gegenstände, Spielerwechsel). Ergebnis: `troxx.vctest` 87 von 87 (3 statische Prüfungen, dazu drei Tests zum Extraleben bei 10000 Punkten), `troxx_kurz.vctest` 166 von 166, `troxx_regel0.vctest` 1 von 1 (alte Regel). Dauerlauf (`AUTOPLAY`, `TESTDEMO`) im Emulator ohne Absturz: PAL und NTSC je 10 Minuten (ein Spieler und zwei Spieler, mit und ohne `AUTOGOD`), bis 20 Minuten mit Diskette (17 Spiele nacheinander, Bestenliste jedes Mal gespeichert), auch mit allen Zusatzfunktionen aus.

Nur im Emulator geprüft, nicht an echter Hardware; die echten Joystick-Leitungen sind nur über die Handprüfliste prüfbar.
