# Troxx (MEGA65)

Eigenständiges Arcade-Beispiel für VCASM auf dem MEGA65: Eine Figur fährt die Linien eines Gitters (14 × 7 Rechtecke) ab und färbt sie. Sind alle vier Kanten eines Rechtecks abgefahren, wird es gefüllt. Gegner laufen auf denselben Linien und kosten ein Leben; **Regel 16.2**: Läuft ein Gegner über das Innenzeichen eines Segments, das an kein gefülltes Rechteck grenzt, wird das Segment zerstört. Die Regeln sind die der Referenzfassung für den VC-20 (`examples/vic20/troxx`) und des C64-Beispiels; der MEGA65 bringt mehr Leistung: 80 × 50 Zeichen mit Vollfarb-Kacheln (VIC-IV), eine 256-Farben-Palette, 16-Farben-Sprites mit Pixelbewegung, eine Füllanimation per DMA, 40 MHz, mehrstimmige Musik über vier SID-Chips, Zusatzfunktionen und **zwei Spieler**, abwechselnd oder gleichzeitig. Konzept: Abschnitt 15 des Konzepts „Gittermosaik“ (Projektunterlagen, nicht im Beispielordner).

**Stand:** nur im Emulator Xemu (PAL und NTSC) geprüft, nicht an echter Hardware. Handprüfliste für den Inhaber: `HANDTEST.md`. Bildschirmfotos aus Xemu: `bilder/`.

## Starten

Im Projektordner `vcc build`, dann `build/main.prg` im Emulator starten oder auf den MEGA65 laden:

```
xmega65 -prg build/main.prg -prgmode 65
```

Das Programm ist ein BASIC-65-Start (`SYS 8205`) und läuft ohne Zusatzdateien. Die Bestenliste (`TROXX.HS`) liegt auf dem Laufwerk 8 (echter MEGA65: SD-Karte, Xemu: Verzeichnis „XEMU EXTERNAL“); ohne Laufwerk oder Datei läuft das Spiel mit der Standardliste weiter und meldet das nach dem Eintragen („nicht gespeichert“). Joysticks an den beiden Anschlüssen (Xemu: Joystick-Emulation über die Tastatur, siehe Xemu-Einstellungen; echte Joysticks am MEGA65).

## Bedienung

**Titelbild:** Joystick hoch/runter wählt die Zeile, links/rechts ändert den Wert, Feuer startet. Darunter steht die Bestenliste („rekord“).

| Zeile | Werte |
|---|---|
| spieler | 1 oder 2 |
| ablauf | nur mit zwei Spielern: `abwechselnd` (nach jedem Treffer ist der andere dran, jeder hat sein eigenes Gitter, Punkte und Leben) oder `gleichzeitig` (beide Figuren im selben Gitter, gelb und cyan) |
| joystick | `ein joystick, anschluss 2` (Standard), `ein joystick, anschluss 1`, `zwei joysticks` (Spieler 1 an Anschluss 1, Spieler 2 an Anschluss 2; beim gleichzeitigen Spiel immer so) |
| tempo | 1–9 |

Bei **einem Joystick** und zwei Spielern abwechselnd erscheint vor jedem Zug „joystick weitergeben, feuer“.

**Spiel:** Die Figur läuft nur, solange eine Richtung gehalten wird; an Knoten (Kreuzungen und Ecken) wechselt sie die Richtung, auf einer Linie sind nur vor und zurück möglich. Feuer = Pause, nochmal Feuer = weiter. Unter dem Feld (Zeilen 43–49) zeigen drei Tafeln Spieler 1, Spieler 2 (Punkte, Leben als Herzen) und die Stufe (Rekord, gefüllte Felder, Fortschrittsbalken); Meldungen stehen in der untersten Zeile.

**Gleichzeitig:** Beide Figuren färben im selben Gitter in der Farbe ihres Spielers (gelb: Spieler 1, cyan: Spieler 2). Die Punkte für ein Zeichen, ein Segment und ein Rechteck gehen an den Spieler, dessen Figur das letzte Zeichen färbt. Einen Gegenstand (Stern, Eisblock) nimmt die Figur auf, die seine Zelle betritt; die Punkte gehen an ihren Spieler. Ein Treffer kostet nur ein Leben des getroffenen Spielers; ist das letzte verbraucht, scheidet diese Figur aus, das Spiel endet, wenn beide ausgeschieden sind.

## Regeln

- Gitter 14 × 7 Rechtecke, Innenraum 4 × 5 Zeichen (71 × 43 Zellen, gx 0–70, gy 0–42); 217 Segmente (112 waagerecht, 105 senkrecht), 98 Rechtecke. Nummerierung H(r, c) = 14 r + c, V(r, c) = 112 + 15 r + c, B(r, c) = 14 r + c. Das Feld füllt die Zeilen 0–42 und die Spalten 4–74 des Bildschirms, darunter die Tafeln.
- Punkte: 1 je neu gefärbtes Zeichen, 10 je abgeschlossenes Segment, 100 je gefülltes Rechteck, **Kettenreaktion**: schließt ein Segment zwei Rechtecke zugleich, kommen 200 dazu (zusammen 400), 500 je Stufe, Bonusstern 50. Extraleben je 10 000 Punkte, höchstens 9 Leben; Start mit 3 Leben. Zerstört ein Gegner ein Segment, kostet das die gebuchten Punkte (nie unter 0).
- Gegner nur auf den Linien, **Entscheidung nur an Knoten** (jede Richtung außer der Umkehr): **Läufer** (rot) wählt mit der Wahrscheinlichkeit PROB (50–90 % je Stufe) die Richtung, die den Abstand zur nächsten Figur am stärksten verkleinert, sonst Zufall; **Streuner** (orange) immer Zufall, jeder vierte Schritt fällt aus; **Wächter** (Zusatzfunktion) geradeaus, an Ecken die einzige Abbiegung: fester Rundgang um den Rand. Gegnerzahl 2, 2, 3, 3, 4, 4, 5, 5, 5 je Stufe, Start an fünf Knoten (nie unten, dort starten die Figuren). Zufall: 8-Bit-LFSR, fester Startwert je Stufe.
- **Gegner zerstören Linien** (Regel 16.2, `ENEMY_DESTROYS_LINES`): Betritt ein Gegner ein Innenzeichen eines Segments, das an **kein gefülltes Rechteck** grenzt, wird das ganze Segment zurückgesetzt (blitzt zwei Bilder lang weiß auf, wird wieder blau); die gebuchten Punkte werden abgezogen (1 je gefärbtes Zeichen, bei abgeschlossenem Segment 10 mehr, nie unter 0, jeder Spieler verliert die Punkte seiner Zeichen) und bei abgeschlossenem Segment `BOXCNT` der angrenzenden Rechtecke um 1 verringert. **Knotenregel:** Betritt ein Gegner einen **Knoten**, werden alle dort endenden Segmente (bis zu vier, auch das Herkunftssegment) auf dieselbe Weise geprüft und zerstört (`enemies.touch`). Die Figur färbt Knoten **beim Betreten** in ihrer Farbe (ohne Punkte, `grid.paint_node`). Nach jeder Zerstörung werden gefärbte Knoten wieder blau, wenn an ihnen kein abgefahrenes Segment mehr anschließt (keine der acht Nachbarzellen abgefahren) und kein angrenzendes Rechteck gefüllt ist; ein Knoten mit Figur oder Gegner darauf bleibt (`nodes_free`/`node_free` in `grid.asm`, wie in der Referenzfassung ab Commit 9fa111ad). Eingefrorene Gegner (Eisblock) zerstören nichts.
- Kollision: gleiche Zelle. Ein Leben weniger, Figuren auf die Startknoten (unten links und unten rechts), Gegner auf ihre Startknoten, Farben und gefüllte Rechtecke bleiben, danach kurze Unverwundbarkeit.
- Stufe geschafft, wenn alle 98 Rechtecke gefüllt sind; alle zwei Stufen Tempo + 1 bis 9. Tempo in Bildern je Spielschritt (PAL 12 … 3, NTSC × 1,2).

## Was der MEGA65 hier leistet

- **VIC-IV, 80 × 50 Zeichen** (`$D031 = $C8`, 640 × 400 Bildpunkte): 16-Bit-Zeichennummern, Bildschirm bei `$0001D000`, Farb-RAM bei `$0FF80000`. Linien, Knoten, Füllmuster, Herzen, Tafeln und das Logo sind **Vollfarb-Kacheln** (FCM, 8 × 8 Punkte, ein Byte je Punkt = Palettenindex) aus `assets.inc`; die Schrift kommt aus dem Zeichensatz-ROM (vier Themen für verschiedenfarbige Tafeln).
- **256-Farben-Palette** (`$D100/$D200/$D300`): Grundfarben, Verläufe für die Füllmuster, 64 Regenbogenfarben für das Titelbild (Farbzyklus der Palette ohne Neuzeichnen).
- **16-Farben-Sprites, 16 × 16** (`$D06B`, echte Bildpunkte): zwei Figuren, fünf Gegner, ein Gegenstand, mit Pixelbewegung: die Spiellogik rechnet weiter in Zeichenzellen, das Sprite gleitet während des Schritts in acht Punkten zur nächsten Zelle (`sprites.asm`).
- **DMA** (DMAgic, `vic4.asm`): Kacheln und Sprites kopieren, Bildschirm löschen, Gitter für den Spielerwechsel sichern und zurückholen, **Füllanimation**: ein Rechteck füllt sich Zeile für Zeile, die neueste Zeile blitzt auf.
- **40 MHz** (`$D054` VFAST); Bildtakt per Rasterabfrage (kein Interrupt).
- **Klang** (`sound.asm`): vier SID-Chips, zwölf Stimmen; Titelmusik mit sechs Stimmen (SID 1 und 2) mit Pulsbreitenmodulation, Effekte (Schritt, Segment, Rechteck, Kette, Treffer, Stufe, Ende, Stern, Eis, Zerstörung) auf SID 3 und 4 mit Prioritäten.
- **Bestenliste** (`hiscore.asm`): zehn Einträge auf der SD-Karte über das KERNAL, Namenseingabe mit dem Joystick, Prüfsumme CRC-16; Fehler (kein Laufwerk, keine Datei, falsche Prüfsumme, schreibgeschützt) führen zur Standardliste und zu einer Meldung.

## Zusatzfunktionen und Schalter

Am Kopf von `main.asm` stehen die Schalter (`.const`, mit `-D NAME=0` überschreibbar). Mit allen auf 0 gelten genau die Regeln der Referenzfassung (Feldgröße und Sprites bleiben).

| Schalter | Standard | Wirkung |
|---|---|---|
| `EXTRA_STAR` | 1 | Bonusstern auf einer freien Linienzelle (Sprite, 50 Punkte), nach zehn Sekunden weg; danach vier bis fünf Sekunden Pause |
| `EXTRA_ICE` | 1 | Eisblock: alle Gegner stehen fünf Sekunden still, die Figuren laufen weiter; jeder vierte Gegenstand ist ein Eisblock |
| `EXTRA_GUARD` | 1 | dritter Gegnertyp „Wächter“ |
| `EXTRA_HISCORE` | 1 | Bestenliste auf der SD-Karte |
| `EXTRA_2P` | 1 | zwei Spieler (abwechselnd oder gleichzeitig), Joystickwahl im Titelbild |
| `ENEMY_DESTROYS_LINES` | 1 | Regel 16.2: Gegner zerstören nicht geschlossene Segmente |
| `FX_FILL` | 1 | Füllanimation mit Glanzzeile |
| `FX_TITLE` | 1 | großes Logo, Farbzyklus der Palette, Pulsbreitenmodulation der Titelmusik |

Test- und Dauerlaufschalter: `AUTOPLAY=1` (der Rechner spielt selbst, kein Titelbild), `AUTO_PLAYERS=2` (dabei zwei Spieler), `AUTO_SIMUL=1` (gleichzeitig), `AUTOGOD=1` (unverwundbar), `TESTDEMO=1` (Zähler im RAM ab `$CE84`: Bilder, Spiele, höchste Stufe, Treffer, Rechtecke), `HOLD_DESTROY=n` (nur Test: Aufblitzen zerstörter Segmente n Bilder lang halten, fertige Fassung in `testfassungen/`), `LIVES0=n`, `TESTSCORE=$nn` (Startpunkte), `NOSAVE=1` (die Bestenliste bleibt unberührt), `SCRIPT=1` oder `2` (Joystickeingaben aus `tools/script*.inc`: Titelbild bedienen und spielen ohne Joystick). Eine Variante bauen: die `.const`-Zeilen vor den Programmtext setzen.

## Tests

```
vcc test --backend xemu            # im Projektordner, nach vcc build
```

`troxx.vctest` (erzeugt von `tools/gen_tests.py`) enthält statische Prüfungen (Größe, Überlappung, Speicherplan) und 93 Routinentests der gemeinsamen Logik (`rnd`, `level_idx`, `set_row`, `cell_type`, `nb_boxes`, `legal_mask`, `box_filled`, `seg_is_closed`, `bcd_add`/`bcd_sub`, `greedy`, `place_all`, `find`, `player.step`, Knotenregel: `node_free`, `paint_node`, `seg_destroy`, `touch`, Bestenliste), im Emulator Xemu. **Hinweis zu Xemu:** Der Testläufer startet Xemu mit dem Original-ROM, dessen BASIC über `$2000–$7FFF` liegt; der erste Test („Vorbereitung“) blendet die ROMs aus, danach laufen die übrigen. Er muss darum vor den anderen stehen, und nach einer Zeitüberschreitung (Neustart des Emulators) schlagen die folgenden Tests fehl, bis er wieder läuft.

Gemeinsame Logik: `KOPIEN.txt` listet, welche Routinen Kopien der Referenzfassung sind und welche angepasst wurden; `python3 tools/troxx_kopien.py examples/mega65/troxx` (im Repository-Ordner) prüft das.

**Dauerlauf:** `AUTOPLAY=1 AUTOGOD=1 TESTDEMO=1` in Xemu, PAL und NTSC, mehrere Minuten, einmal gleichzeitig mit zwei Spielern, einmal abwechselnd (Ergebnisse in `HANDTEST.md`, Abschnitt „Dauerlauf“).

## Bilder

Bildschirmfotos aus Xemu (`bilder/`): `titelbild.png`, `spielstand.png` (ein Spieler), `zweispieler-gleichzeitig.png`, `zweispieler-treffer.png`, `zerstoerte-linie.png` (ein Segment blitzt auf, weil ein Gegner es zerstört), `spielende-rekord.png` (Namenseingabe). Abweichungen vom Mockup des Konzepts: Das Spiel zeigt die Tafeln ohne die Systemzeile („VIC-IV 80x50 …“) und füllt die Rechtecke mit einem Verlauf je Zeile (Farbe je Rechteckzeile); die Figuren sind 16 × 16-Sprites mit Gesicht, die Gegner Läufer (rot), Streuner (violett) und Wächter.

## Dateien

| Datei | Inhalt |
|---|---|
| `main.asm` | Konstanten, Speicherplan, Start, Zustandsautomat (Titel, Stufenanfang, Spiel, Füllen, Treffer, Stufenende, Spielende), Titelbild |
| `vic4.asm` | VIC-IV: Einrichten, Kacheln, Palette, Schrift, DMA, Bildtakt, Zeichen setzen |
| `grid.asm` | Schattengitter, Zeichnen, Segmente, Rechtecke, Färben (`paint`), Zerstören (`seg_destroy`), Füllen |
| `player.asm`, `enemies.asm`, `items.asm` | Figuren (ein oder zwei), Gegner, Stern und Eisblock |
| `sprites.asm` | Sprites: Platz, Pixelbewegung, Treffer-Bild |
| `hud.asm` | Tafeln, Texte, Punkte (BCD), Herzen, Fortschritt |
| `input.asm`, `sound.asm`, `hiscore.asm` | Joysticks, vier SID, Bestenliste |
| `data.inc`, `assets.inc` | Tabellen; Palette, Kacheln, Sprites (erzeugt von `tools/gen_assets.py`) |
| `KOPIEN.txt`, `HANDTEST.md`, `XEMU-TESTS.md` | Kopienliste, Handprüfliste, Anleitung für die Xemu-Läufe |
| `testfassungen/LIESMICH.md` | Testfassungen für Bildschirmaufnahmen der Zerstörung (`zerstoerung-hold`, `szenario-destroy`, `szenario-node`): Bauanleitung, nicht im Standardbau |

## Speicherplan

`$0002–$00BF` und `$00E0–$00F7` Zeropage (Plan im Kopf von `main.asm`), `$2001–$B7FF` Programm, `$B800–$BFFF` Sprite-Daten und -Zeiger (VIC-Bank 2), `$C000–$CD6F` Schattengitter (43 Zeilen zu 80 Byte), `$CE00–$CFFF` Spielvariablen, `$E000–$FFEF` gesicherte Gitter der beiden Spieler; im Chip-RAM `$0001D000` Bildschirm, `$00012000` Kacheln, `$00014000` Schrift, `$0FF80000` Farb-RAM. Alle ROMs sind im Spiel ausgeblendet; die Bestenliste blendet das KERNAL nur für die Dauer eines Zugriffs wieder ein.
