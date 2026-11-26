# Troxx (Plus/4)

Eigenständiges Arcade-Beispiel für VCASM auf dem Commodore Plus/4 (und C16-ähnlichen TED-Rechnern mit 64 KB): Eine Figur fährt die Linien eines Gitters (8 × 5 Rechtecke) ab und färbt sie. Sind alle vier Kanten eines Rechtecks abgefahren, wird es gefüllt. Gegner laufen auf denselben Linien und kosten ein Leben. Die Regeln sind die der Referenzfassung für den VC-20 (`examples/vic20/troxx`); der Plus/4 bringt den **TED mit 121 Farben** (Helligkeitsstufen, Füllungen mit Verlauf und Abschrägung), einen Zeichensatz im RAM, Figuren aus Zeichen mit vier Animationsbildern (der Plus/4 hat keine Sprites), einen Rasterinterrupt, zwei Stimmen und Zusatzfunktionen. Konzept: `assets/docs/Konzept-Beispiel-VC20-Gittermosaik.md` (Arbeitstitel „Gittermosaik“, Abschnitte 15 und 16), Mockup `assets/docs/mockups/gittermosaik/gm-4-plus4.png`.

**Stand:** Welle W2 des Konzepts, nur im Emulator (VICE `xplus4` 3.10, PAL und NTSC) geprüft, nicht an echter Hardware. Handprüfliste für den Inhaber: `HANDTEST.md`. Prüfsummen der Ausgabe: `PRUEFSUMMEN.txt`.

## Bedienung

Starten: `vcc build` im Projektordner, dann **`build/disk.d64`** im Emulator starten (`xplus4 build/disk.d64`, Autostart), damit die Bestenliste auf derselben Diskette landet. Ohne Diskette (nur `build/main.prg`) läuft das Spiel genauso, nur ohne Bestenliste. Joystick im Anschluss 1 oder 2 (Emulator: `-joydev1 2` = Tastensatz 1, `-joydev2 1` = Ziffernblock, Feuer = Ziffernblock 0 bzw. rechte Alt-Taste, je nach Einstellung des Emulators).

**Titelbild:** Joystick hoch/runter wählt die Zeile, links/rechts ändert den Wert, Feuer startet.

| Zeile | Werte |
|---|---|
| PLAYERS | 1 oder 2 (zwei Spieler **abwechselnd**: nach jedem verlorenen Leben ist der andere dran, jeder hat sein eigenes Gitter, seine Punkte und Leben) |
| JOYSTICK | `PORT 2` (ein Joystick am Anschluss 2, Standard), `PORT 1` (ein Joystick am Anschluss 1), `TWO STICKS` (nur mit zwei Spielern: Spieler 1 am Anschluss 1, Spieler 2 am Anschluss 2) |
| SPEED | Tempo 1–9 |

Bei **einem Joystick** und zwei Spielern erscheint vor jedem Zug „pass joystick – fire“: Joystick weiterreichen, Feuer drücken. Die Wahl bleibt bis zum Ausschalten im RAM. Unter den Einstellungen steht die Bestenliste, rechts neben „HIGH SCORES“ das Ergebnis des Plattenzugriffs (`DISK OK`, `NO DRIVE`, `NO FILE`, `DISK ERR`).

**Spiel:** Die Figur läuft nur, solange eine Richtung gehalten wird; an Knoten (Kreuzungen und Ecken) wechselt sie die Richtung, auf einer Linie sind nur vor und zurück möglich. Feuer = Pause, nochmal Feuer = weiter. Anzeigen wie im Mockup: oben SCORE, LEVEL und HI (Zeile 0), darunter Meldungen (Zeile 1); unter dem Feld die Leben als Herzen (Zeile 23, bei zwei Spielern rechts PLAYER n) und in Zeile 24 SPEED, BONUS (Symbol und Text des Gegenstands auf dem Feld) und der Rechnername.

**Hinweis zur Tastatur:** Die Joysticks des Plus/4 hängen an der Tastaturmatrix. Während des Spiels keine Taste drücken (sie würde einen Joystick-Kontakt vortäuschen). Die Tastatur wird nicht abgefragt. Das Feuer wird an beiden Anschlüssen als Bit 7 oder Bit 6 von `$FF08` gelesen (welches Bit der Emulator liefert, ist nicht geklärt; im Handtest funktionierte es so).

## Regeln (wie die Referenzfassung)

- Gitter 8 × 5 Rechtecke, Innenraum 3 × 3 Zeichen (33 × 21 Zellen); 93 Segmente (48 waagerecht, 45 senkrecht), 40 Rechtecke. Nummerierung H(r, c) = 8 r + c, V(r, c) = 48 + 9 r + c, B(r, c) = 8 r + c.
- Punkte: 1 je neu gefärbtes Zeichen, 10 je abgeschlossenes Segment, 100 je gefülltes Rechteck, **Kettenreaktion**: schließt ein Segment zwei Rechtecke zugleich, kommen 200 dazu (zusammen 400), 500 je Stufe, Bonusstern 50. Extraleben je 10 000 Punkte, höchstens 9 Leben; Start mit 3 Leben.
- Gegner nur auf den Linien, **Entscheidung nur an Knoten** (jede Richtung außer der Umkehr): **Läufer** (rot) wählt mit der Wahrscheinlichkeit PROB (50–90 % je Stufe) die Richtung, die den Abstand zur Figur am stärksten verkleinert, sonst Zufall; **Streuner** (violett) immer Zufall; **Wächter** (cyan, Zusatzfunktion) geradeaus, an Ecken die einzige Abbiegung: fester Rundgang um den Rand. Gegnerzahl 1, 2, 2, 3, 3, 3, 4, … je Stufe. Zufall: 8-Bit-LFSR, fester Startwert je Stufe.
- **Gegner zerstören Linien** (Konzept 16.2, `ENEMY_DESTROYS_LINES`): Betritt ein Gegner ein Innenzeichen eines Segments, das an **kein gefülltes Rechteck** grenzt (auch ein ganz abgefahrenes Segment ist sonst nicht geschützt), wird das ganze Segment zurückgesetzt (Zeichen wieder blau). Dafür werden die gebuchten Punkte abgezogen (1 je gefärbtes Zeichen, bei abgeschlossenem Segment 10 mehr, nie unter 0) und bei abgeschlossenem Segment `BOXCNT` der angrenzenden Rechtecke um 1 verringert; gefüllte Rechtecke und ihre Prämien bleiben. **Knotenregel:** Betritt ein Gegner einen **Knoten**, werden alle dort endenden Segmente (bis zu vier, auch das Herkunftssegment und nicht abgebogene) auf dieselbe Weise geprüft und zerstört (je Segment Blitz, Ton und Punktabzug). Die Figur färbt Knoten **beim Verlassen** gelb (ohne Punkte; so wird auch der Startknoten gelb). Nach jeder Zerstörung werden gelbe Knoten wieder blau, wenn an ihnen kein gelbes Segment mehr anschließt (keine Nachbarzelle gelb) und kein angrenzendes Rechteck gefüllt ist (Füllzeichen in der Nachbarschaft); ein Knoten mit Figur oder Gegner (auch als Ziel des laufenden Schritts) bleibt; kein Zeitablauf (`nodes_free`/`node_free` in `grid.asm`, `paint_node` beim Verlassen in `player.asm`). Eingefrorene Gegner (Eisblock) zerstören nichts, alle Gegnertypen zerstören. Rückmeldung am Plus/4: die gefärbten Zeichen blitzen zwei Bilder lang als weißes Treffer-Zeichen auf, dazu der Effekt „Linie zerstört“ (fallender Ton auf Stimme 1, Knistern aus Rauschen in wechselnder Höhe auf Stimme 2). Steht die Figur auf dem Segment, läuft sie weiter; ihre Spur ist weg. Die Regel steht an wenigen getrennten Stellen: Aufruf in `enemies.one`, Logik in `grid.asm` (`seg_is_closed`, `seg_destroy`, `cell_op`), `hud.bcd_sub`, Effekt `Fx.destroy` in `sound.asm`.
- Kollision: gleiche Zelle oder Platztausch. Ein Leben weniger, Figuren auf die Startknoten, Farben und gefüllte Rechtecke bleiben, danach kurze Unverwundbarkeit (Blinken weiß/cyan).
- Stufe geschafft, wenn alle 40 Rechtecke gefüllt sind; alle zwei Stufen Tempo + 1 bis 9. Tempo in Bildern je Spielschritt (PAL 12 … 3, NTSC ×1,2): gleiche Tabelle wie in der Referenzfassung.

### Verbesserungen gegenüber dem VC-20

- **TED-Farben**: Farbspeicher-Byte = Helligkeit (0–7) · 16 + Farbton. Die Linien sind blau, abgefahrene Linien hellgelb. Die vier Füllmuster (Schraffur, Punkte, Mauer, Schachbrett) sind Mehrfarben-Zeichen mit **neun Lagen im 3 × 3-Block** (Glanz oben und links, Schatten unten und rechts = Abschrägung, `charset.asm`, erzeugt von `tools/gen_charset.py`); der Farbton wechselt je Rechteckzeile (grün, cyan, violett, rot, gelb), die **Helligkeit steigt je Zeile des Blocks** (Farbverlauf), und ein Rasterinterrupt stellt je Rechteckzeile die beiden Mehrfarben-Hintergrundfarben (Schatten und Glanz) auf den passenden dunklen und hellen Ton ein.
- **Figuren als Zeichen** (`figs.asm`): Spieler, bis zu vier Gegner und der Gegenstand sind Zeichen mit **vier Animationsbildern** (alle acht Bilder ein Wechsel) und Farbwechsel (Unverwundbarkeit weiß/cyan, Eisblock hellblau, Gegenstand blinkt). Das Gitter hat eine **Schattenkopie** (Zeichen und Farben je Zelle, `grid.asm`); eine Figur überdeckt nur den Bildschirm, beim Weiterziehen wird die Zelle aus der Schattenkopie wiederhergestellt. Spiellogik und Schattenkopie kennen keine Figuren.
- **Zweites Feldlayout ab Stufe 5**: sechs Rechtecke sind zu Beginn schon gefüllt (graue Mauerblöcke, Kanten gelb), die Zähler (`BOXCNT`, `BOXES_DONE`) werden daraus neu gezählt, die Stufe endet weiterhin bei 40 Rechtecken. `LAYOUT2_LEVEL` legt die Stufe fest.
- **Rasterinterrupt** (`raster.asm`, über den KERNAL-Vektor `$0314`, KERNAL-ROM bleibt eingeblendet): Bildtakt (Zeile 204), **wandernde Farbbalken** im oberen und unteren Rand (Verlauf Blau, Helligkeit 0–7 und zurück; je Norm eine Tabelle, weil PAL und NTSC verschieden viele Randzeilen zeigen), Farbbänder der Füllungen, im Titelbild Hintergrundbänder und ein **weicher Bildlauf** (Zeile 6, X-Versatz in `$FF07`, 38-Spalten-Modus nur in dieser Zeile); Diskettenzugriffe sperren ihn.
- **Titelbild** mit großem Logo aus abgeschrägten Mehrfarben-Blöcken (5 × 5 je Buchstabe) und Farbverlauf in Farbton und Helligkeit.
- **Füllanimation**: Zeile für Zeile mit Aufblitzen der neuesten Zeile (nur Farbspeicher, die Logik bleibt unberührt).
- **TED-Ton** (zwei Stimmen, `sound.asm`): Titelmusik (Melodie auf Stimme 1, Bass im Wechsel mit dem Akkord auf Stimme 2), Effekte Schritt (Rauschen auf Stimme 2), Segment, Rechteck, Kette (zweistimmig), Treffer (tiefer Ton und fallendes Rauschen), Stufe (Fanfare), Ende, Bonusstern, Eisblock, je mit Prioritäten. Der TED hat keine Hüllkurve und nur eine Lautstärke für beide Stimmen; Töne unter 108 Hz gibt es nicht, darum liegt der Bass eine Oktave höher als beim C64.
- **Zusatzfunktionen** (jeweils abschaltbar, siehe unten): Bonusstern, Eisblock, Wächter, zweites Layout, Bestenliste auf Diskette, zwei Spieler abwechselnd mit Joystickwahl.

## Zusatzfunktionen und Schalter

Am Kopf von `main.asm` stehen die Schalter (`.const`, mit `-D NAME=0` überschreibbar). Mit allen `EXTRA_*` auf 0 gelten genau die Regeln der Referenzfassung (Feldgröße bleibt).

| Schalter | Standard | Wirkung |
|---|---|---|
| `EXTRA_STAR` | 1 | Bonusstern auf einer freien Linienzelle (Zeichen, 50 Punkte), nach zehn Sekunden weg; danach vier bis fünf Sekunden Pause |
| `EXTRA_ICE` | 1 | Eisblock: alle Gegner stehen fünf Sekunden still (hellblau), die Figur läuft weiter; jeder vierte Gegenstand ist ein Eisblock |
| `EXTRA_GUARD` | 1 | dritter Gegnertyp „Wächter“ (ab Stufe 4 als dritter Gegner) |
| `EXTRA_HISCORE` | 1 | Bestenliste (zehn Einträge, Namenseingabe mit dem Joystick) auf Diskette |
| `EXTRA_2P` | 1 | zwei Spieler abwechselnd, Spielerwahl im Titelbild |
| `ENEMY_DESTROYS_LINES` | 1 | Regel 16.2: Gegner zerstören Linien (0 = alte Regel) |
| `EXTRA_LAYOUT` | 1 | zweites Feldlayout ab Stufe `LAYOUT2_LEVEL` (5) |
| `FX_RASTER` | 1 | Randbalken, Farbbänder, Bildlauf (ohne sie bleibt nur der Bildtakt-Interrupt) |
| `FX_TITLE` | 1 | großes Logo, Farbverlauf, Bildlauf im Titelbild (braucht `FX_RASTER`) |
| `FX_FLASH` | 1 | Aufblitzen beim Füllen |
| `FX_SHADE` | 1 | Helligkeitsverlauf der Füllungen und Farbband je Rechteckzeile |

Test- und Dauerlaufschalter: `AUTOPLAY=1` (der Rechner spielt selbst, kein Titelbild), `AUTO_PLAYERS=2` (dabei zwei Spieler), `AUTOGOD=1` (unverwundbar), `TESTDEMO=1` (Zähler im RAM ab `$5030` und hexadezimal in Zeile 1: Bilder, Spiele, höchste Stufe), `LIVES0=n`, `START_SPEED=n`, `LAYOUT2_LEVEL=n`, `TESTSCORE=$nn` (Startpunkte), `SCRIPT=1` oder `2` (Joystickeingaben aus `tools/script*.inc`), `TESTPAD=1` (Hüllen für die kurzen Testroutinen). Bauen einer Variante:

```bash
vcc asm main.asm --target plus4 --cpu 7501 --illegals deny -o troxx_auto.prg -D AUTOPLAY=1 -D AUTOGOD=1 -D TESTDEMO=1
```

## Bestenliste (Diskette)

Datei `TROXX.HS` auf Laufwerk 8, 77 Byte: Kennung „TXHS“ und Version 1, zehn Einträge zu 7 Byte (Name 3 Bildschirmcodes, Punkte BCD niedrig/mittel/hoch, Stufe BCD), Prüfsumme (Summe und Summe der mit `$5A` verknüpften Bytes). Gespeichert wird mit „@0:“ (ersetzt die alte Datei) und danach durch **Zurückladen und Vergleichen** geprüft (Schreibschutz meldet das KERNAL nicht). Fehlt das Laufwerk oder die Datei oder stimmt die Prüfsumme nicht, gilt die Standardliste („TRX“, 10 000 bis 1 000 Punkte); jeder Fehler wird abgefangen und angezeigt, das Spiel läuft weiter. Namenseingabe: hoch/runter Buchstabe (A–Z, Leerzeichen), links/rechts Stelle, Feuer fertig.

Während eines Plattenzugriffs ist der Rasterinterrupt aus und der KERNAL-Interruptvektor wieder gesetzt (`raster.pause`/`resume`). Beobachtung im Emulator: Ein `OPEN` auf den Fehlerkanal 15 nach dem Speichern ließ den Rechner abstürzen (Ursache nicht geklärt); darum wird der Fehlerkanal nicht gelesen.

## Speicherplan

| Bereich | Verwendung |
|---|---|
| `$0002`–`$007F` | Zeropage: Spielzustand (Plan im Kopf von `main.asm`); `$90` und höher bleibt dem KERNAL (Diskette) |
| `$0800`–`$0BE7`, `$0C00`–`$0FE7` | Farbspeicher, Bildschirm 40 × 25 |
| `$1001`–`$3FFF` | Programm (Segment `PROGRAM`, derzeit etwa 8,4 KB) |
| `$4000`–`$47FF` | Zeichensatz-RAM: Codes 0–127 Kopie des Zeichen-ROMs (Text), 128–255 eigene Grafik (69 Zeichen) |
| `$4800`–`$4AB4`, `$4B00`–`$4DB4` | Schattenkopie des Gitters: Zeichen, Farben (33 × 21) |
| `$5000`–`$5FFF` | Spielvariablen, Bestenliste, Spielstand und Gitter des zweiten Spielers |

Das KERNAL-ROM bleibt eingeblendet (Diskette, Zeichen-ROM); verfügbar sind damit `$1001`–`$7FFF` (rund 28 KB). Der Speicherausbau auf 60 KB (ROM aus) ist nicht nötig und wird nicht benutzt.

TED: Bildschirm `$0C00`, Farbspeicher `$0800` (`$FF14` = `$08`), Zeichensatz `$4000` aus dem RAM (`$FF12` Bit 2 = 0, `$FF13` = `$40`), `$FF06` = `$1B`, `$FF07` = `$98` (Zeichen 128–255 getrennt, Mehrfarben möglich, 40 Spalten). Zellen (Text, Linien, Figuren) verwenden nur die Farbtöne 0–7: Bit 3 des Farbtons ist im Mehrfarbenmodus das Mehrfarben-Bit der Zelle (Füllungen und Logo-Blöcke setzen es); Hintergrund, Rahmen und die Mehrfarben-Hintergründe 1 und 2 (`$FF16`, `$FF17`) können alle 121 Farben haben. Mehrfarben-Bitpaare: 00 Hintergrund 0, 01 Hintergrund 1 (dunkler Ton der Zeile), 10 Hintergrund 2 (heller Ton), 11 Farbe der Zelle.

## Dateien

| Datei | Inhalt |
|---|---|
| `main.asm` | Schalter, Konstanten, Speicherplan, Start, Zustandsautomat (Titelbild, Spielerwechsel, Spielende), Füllanimation |
| `ted.asm` | Bildinit, PAL/NTSC-Erkennung, Zeichensatz einrichten, `wait_frame` |
| `raster.asm` | Rasterinterrupt: Bildtakt, Randbalken, Farbbänder, Bildlauf |
| `figs.asm` | Figuren als Zeichen: Zeichnen, Wiederherstellen aus der Schattenkopie, Bilder, Farben |
| `grid.asm` | Datenmodell mit Schattenkopie, Zeichnen, Segmente, Rechtecke, Füllen, zweites Layout, Gitter sichern und laden |
| `player.asm`, `enemies.asm` | Bewegung, Selbstspiel, Zufall, Entscheidung, Kollision |
| `items.asm` | Bonusstern und Eisblock |
| `input.asm` | Joystick über die Tastaturmatrix (`$FF08`), Anschlusswahl |
| `sound.asm` | Klangmodul (TED), Effekte, Titelmusik |
| `hud.asm` | Anzeigen, Meldungen, Titelbild, BCD-Punkte |
| `hiscore.asm`, `disk.asm` | Bestenliste und Plattenzugriff über das KERNAL |
| `data.inc`, `charset.asm` | Tabellen, Grafikzeichen (erzeugt) |
| `KOPIEN.txt` | Liste der aus der Referenzfassung kopierten Routinen |
| `troxx.vctest`, `troxx_kurz.vctest`, `troxx_regel0.vctest` | Tests (die letzte Datei mit `ENEMY_DESTROYS_LINES=0`, gebaut von `tools/build_regel0.sh`) |
| `tools/` | `gen_charset.py`, `gen_tests.py`, `build_pad.sh`, `build_regel0.sh`, `testpad.inc`, Eingabeskripte |

Die Texte auf dem Bildschirm sind englisch und kurz (wie im Mockup); sie stehen an einer Stelle in `hud.asm`.

## Gemeinsame Logik und Kopien

Das Projekt ist in sich geschlossen (nur die Systembibliothek des Assemblers liegt außerhalb). Die systemunabhängige Logik (Nummerierung, Zähler, Zufall, BCD-Punkte, KI-Entscheidung, Tabellen) stammt aus der Referenzfassung; jede Datei nennt im Kopf ihre Herkunft („Herkunft: examples/vic20/troxx/…, Stand Commit …“). `KOPIEN.txt` führt auf, welche Routinen als Kopie gelten und welche bewusst angepasst sind (am Plus/4: Schattenkopie, Figuren als Zeichen, Farben, Gittermaße). Abgleich (im Repository-Ordner):

```bash
python3 tools/troxx_kopien.py examples/plus4/troxx
```

Das Werkzeug vergleicht die Kopien nach Entfernen von Kommentaren und Leerzeilen per Prüfsumme mit dem VC-20-Projekt und meldet Abweichungen (Endestatus 1). Änderungen an der gemeinsamen Logik gehören in die Referenzfassung und von dort in alle Projekte.

## Tests

```bash
vcc build                                   # baut build/main.prg und build/disk.d64
python3 tools/gen_tests.py                  # erzeugt troxx.vctest und troxx_kurz.vctest neu (nur nötig nach Änderungen an den Adressen in main.asm)
vcc test --backend vice                     # troxx.vctest: statische Prüfungen (Größe, Überlappung, Zeropage) und Routinen im Emulator (xplus4)
tools/build_pad.sh vcc                      # baut build/pad.prg (mit Hüllen, siehe tools/testpad.inc)
vcc test troxx_kurz.vctest --backend vice   # sehr kurze Routinen
tools/build_regel0.sh vcc                   # baut build0/main.prg mit ENEMY_DESTROYS_LINES=0
vcc test troxx_regel0.vctest --backend vice # dieselben Fälle der Regel 16.2 mit abgeschalteter Regel
```

Die gemeinsame Logik besteht dieselben Testfälle wie in der Referenzfassung (`bcd_add`, `legal_mask`, `greedy`, `neighbours` für alle Segmente, `paint` mit Kettenreaktion, `fill_box`, `player.step`), dazu Tests der Plus/4-Teile (Schattenkopie, Füllungen mit Verlauf, zweites Layout, Titelmenü, Bestenliste, Klangzustand, Gegenstände, Spielerwechsel). Die Joystick-Leitungen lassen sich im Testlauf nicht setzen (Tastaturmatrix); getestet wird nur die Anschlusszuordnung. Ergebnis (`xplus4`, PAL): `troxx.vctest` 59 von 59 (3 statische Prüfungen, 42 Routinentests, 9 Tests der Regel 16.2 und Knotenregel: Segmente teilweise oder ganz gefärbt, geschützt, Rand, zwei Nachbarrechtecke, Punktuntergrenze, ungefärbt, Gegner am Knoten zerstört drei Segmente, eingefrorener Gegner, und 7 Tests `nodes_free`: freistehender Knoten, gelbe Nachbarzelle, gefülltes Nachbarrechteck, Figur, Gegner, Eckknoten, Randknoten), `troxx_kurz.vctest` 161 von 161 (darunter `nb_boxes` für alle 93 Segmente), `troxx_regel0.vctest` 9 von 9 (dieselben Fälle mit abgeschalteter Regel: nichts ändert sich).

Nur im Emulator geprüft, nicht an echter Hardware; die echten Joystick-Leitungen beider Anschlüsse sind nur über die Handprüfliste prüfbar.

**Dauerläufe** (`AUTOPLAY`, `AUTOGOD`, `TESTDEMO`, Tempo 9, Emulator im Schnelllauf), mit der Regel 16.2 (Standard): 1,5 Milliarden Takte (rund 28 Minuten Emulatorzeit) in PAL mit einem Spieler (Stufe 6) und in NTSC mit zwei Spielern (Stufe 5), 800 Millionen Takte NTSC mit einem Spieler; mit `ENEMY_DESTROYS_LINES=0`: je 800 Millionen Takte NTSC mit einem und PAL mit zwei Spielern; alle ohne Absturz (die Stufen kommen mit der Regel langsamer, weil Gegner Linien zerstören). Dabei kommen alle Gegnertypen, Gegenstände, das zweite Layout, Spielerwechsel und Stufenwechsel vor; Bildschirmfoto mit zerstörter Linie siehe Abnahmeordner. Mit Diskette und `LIVES0=1`: Spielende, Namenseingabe (Selbstspiel „CPU“ und Eingabeskript `SCRIPT=2` mit dem Namen „BC “), Speichern und Zurückladen der Bestenliste mehrfach nacheinander (vor der Regel 16.2 geprüft, der Plattenzugriff blieb unverändert); ein Neustart von derselben Diskette zeigt die gespeicherte Liste. Varianten mit allen `EXTRA_*` auf 0 und mit allen `FX_*` auf 0 liefen vor der Regel.
