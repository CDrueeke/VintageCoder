# Troxx (MEGA65): Handprüfliste für den Inhaber

Stand: erste Fassung. Bisher nur im Emulator Xemu (PAL und NTSC) geprüft, nicht an echter Hardware. Der Joystick wurde im Emulator über Testeingaben (Skripte in `tools/`, Schalter `SCRIPT`) nachgestellt, die **echten Joystick-Leitungen an beiden Anschlüssen, die vier SID-Chips, die 40 MHz und der Zugriff auf die SD-Karte sind nicht an Hardware getestet** (Punkte 4–8, 22–26).

**Start:** `vcc build` im Ordner `examples/mega65/troxx`, dann `build/main.prg` im Emulator (`xmega65 -prg build/main.prg -prgmode 65`), über die IDE (Ziel MEGA65) oder auf den MEGA65 laden und mit `RUN` starten. NTSC im Emulator: `-videostd 1`. Bildschirmfotos aus Xemu liegen in `bilder/` (Vergleich mit `MOCKUP-mega65.png`).

Bei jedem Punkt: bestanden / nicht bestanden / Bemerkung.

| Nr | Prüfpunkt | Erwartet |
|---|---|---|
| 1 | Start | Titelbild: großes Logo „TROXX“ in Regenbogenfarben (Farbzyklus der Palette läuft), darunter „ein arcade-spiel fuer mega65“, die vier Zeilen spieler, ablauf, joystick, tempo (Zeiger bei spieler), „hoch/runter waehlt, links/rechts aendert, feuer startet“ und die Bestenliste („rekord“, zehn Zeilen); Titelmusik mit mehreren Stimmen, wiederholt sich |
| 2 | Menü bedienen | Hoch/runter bewegt den Zeiger; links/rechts ändert den Wert (tempo 1–9, nicht darüber hinaus; gedrückt halten wiederholt langsam). Das Menü reagiert auf **beide** Anschlüsse |
| 3 | Spielerzahl, Ablauf, Joystick | spieler 2 schaltet ablauf (`abwechselnd` / `gleichzeitig`) und bei joystick `zwei joysticks` frei; mit spieler 1 zeigt ablauf „-“ und es gibt nur `ein joystick, anschluss 2` und `… anschluss 1`; `gleichzeitig` verlangt zwei Joysticks |
| 4 | Ein Spieler, Anschluss 2 | Voreinstellung, Feuer startet (Musik endet): Gitter 14 × 7 Rechtecke, Figur (Sprite) unten links, „bereit“, Tafeln unter dem Feld (Spieler 1 mit Punkten und drei Herzen, Stufe mit Rekord und Feldern 0 /98); mehrere Gegner an den Startknoten oben und seitlich. Joystick **am Anschluss 2**: Figur läuft nur, solange die Richtung gehalten wird, in allen vier Richtungen; **rechts** und **Feuer** besonders prüfen |
| 5 | Ein Spieler, Anschluss 1 | Im Titelbild joystick `ein joystick, anschluss 1`: jetzt steuert der Joystick am **Anschluss 1**, der am Anschluss 2 tut nichts |
| 6 | Zwei Spieler abwechselnd, zwei Joysticks | spieler 2, ablauf `abwechselnd`, `zwei joysticks`: Spieler 1 (Anschluss 1, gelbe Spur) beginnt („spieler 1 bereit“); nach dem ersten Treffer ist Spieler 2 am **Anschluss 2** dran (eigenes, leeres Gitter, eigene Punkte, cyanfarbene Spur); danach wechseln sie sich nach jedem Treffer ab, jeder behält sein Gitter, seine Punkte und Leben |
| 7 | Zwei Spieler abwechselnd, ein Joystick | spieler 2, `abwechselnd`, ein Joystick: vor jedem Zug „joystick weitergeben, feuer“, Feuer startet den Zug |
| 8 | Zwei Spieler gleichzeitig | spieler 2, `gleichzeitig`: beide Figuren (gelb, cyan) im selben Gitter, Figur 1 unten links, Figur 2 unten rechts, beide steuerbar (Anschluss 1 und 2); jeder färbt in seiner Farbe, Punkte gehen an den, der das Zeichen färbt; **ein Rechteck, das beide zusammen schließen, geht an den Spieler, der das letzte Zeichen färbt**; ein Gegenstand geht an die Figur, die ihn aufnimmt; Treffer kostet nur das Leben des getroffenen Spielers; „spiel ende“, wenn beide ausgeschieden sind |
| 9 | Spielende bei zwei Spielern abwechselnd | Hat ein Spieler kein Leben mehr, spielt der andere weiter, ohne Wechsel; nach dem letzten Leben des zweiten „spiel ende“, danach (wenn die Punkte reichen) die Namenseingabe für Spieler 1, dann für Spieler 2 |
| 10 | Pixelbewegung | Figuren und Gegner gleiten **glatt** (acht Punkte je Zeichen) entlang der Linien, ohne Ruckeln oder Zittern, in jedem Tempo; die Sprites sitzen mittig auf den Linien, auch am linken und rechten Rand des Gitters |
| 11 | Linien und Knoten | Auf einer Linie nur vor/zurück; Abbiegen nur an Knoten; außerhalb des Gitters kein Schritt; bei zwei gehaltenen Richtungen gilt hoch, runter, links, rechts in dieser Reihenfolge |
| 12 | Färben, Punkte | Befahrene Linien werden gelb (Spieler 2: cyan); +1 je neues Zeichen, +10 je Segment (Punkte in der Tafel) |
| 13 | Füllen per DMA | Sind alle vier Kanten eines Rechtecks gefärbt: das Rechteck füllt sich **Zeile für Zeile** mit einem Verlauf (Farbe je Rechteckzeile), eine helle Glanzzeile läuft mit, +100; ohne Flackern der übrigen Zeichen |
| 14 | Kettenreaktion | Schließt ein Segment zwei Rechtecke zugleich: beide füllen zugleich, zusammen +400, andere (längere) Tonfolge |
| 15 | Gegner | **Läufer** nähert sich der nächsten Figur; **Streuner** läuft zufällig und setzt manchmal aus; **Wächter** läuft einen festen Rundgang um den Rand; alle wenden nicht mitten auf einer Linie; Bild wechselt je Schritt |
| 16 | Regel 16.2: Linien zerstören | Läuft ein Gegner über das Innenzeichen eines Segments ohne gefülltes Nachbarrechteck: das Segment blitzt weiß auf (Knistern), wird wieder blau, die gebuchten Punkte werden abgezogen. Segmente an gefüllten Rechtecken bleiben unberührt |
| 16a | Knotenregel | Die Figur färbt Knoten beim Betreten in ihrer Farbe (gelb bzw. cyan), ohne Punkte. Betritt ein Gegner einen **Knoten**, werden alle dort endenden Segmente (bis zu vier) geprüft und zerstört, je Segment Blitz, Knistern und Punktabzug. Nach jeder Zerstörung werden **freistehende** gefärbte Knoten wieder blau (kein abgefahrenes Segment und kein gefülltes Rechteck daneben, keine Figur und kein Gegner darauf); Knoten an gefüllten Rechtecken und an abgefahrenen Segmenten bleiben gefärbt |
| 17 | Kollision, Leben | Berührung: Treffer-Bild (Figuren rot), „treffer“, Rauschen, ein Herz weniger, Figuren zurück auf die Startknoten, Farben und Rechtecke bleiben; die Meldung „treffer“ verschwindet danach; kurze Unverwundbarkeit (Blinken), in der nichts passiert |
| 18 | Extraleben | Je 10000 Punkte ein Leben mehr, höchstens 9 |
| 19 | Stufenwechsel | Alle 98 Rechtecke gefüllt: „stufe ok“, Fanfare, +500, Stufe hochzählen, neues leeres Gitter; Gegnerzahl 2, 2, 3, 3, 4, 4, 5, 5, 5 je Stufe; alle zwei Stufen Tempo + 1 bis 9 |
| 20 | Tempo | tempo 1 deutlich langsamer als tempo 9; Spielgefühl in PAL und NTSC ähnlich |
| 21 | Pause | Feuer im Spiel: „pause“, alles steht (Gegner, Sprites, Zeitgeber des Gegenstands), Ton aus; nochmal Feuer: weiter |
| 22 | Bonusstern | Ab und zu erscheint ein Stern auf einer Linie; nach zehn Sekunden verschwindet er; die Figur nimmt ihn auf: +50, kurzer Klang |
| 23 | Eisblock | Ab und zu ein Eisblock; aufgenommen: alle Gegner stehen fünf Sekunden still, die Figuren laufen weiter, dann laufen sie wieder; eigener Klang |
| 24 | Bestenliste, Namenseingabe | Nach dem Spielende bei genug Punkten: „neuer rekord!“, „name:“, acht Zeichen mit dem Joystick (hoch/runter Zeichen, links/rechts Stelle, Feuer fertig, nach 30 s ohne Eingabe automatisch); „gespeichert“; im Titelbild steht der Eintrag an der richtigen Stelle |
| 25 | Bestenliste auf der SD-Karte | Nach Neustart (gleiche SD-Karte, Datei `TROXX.HS`) ist die Liste wieder da; „rekord“ in der Stufentafel = erster Platz. **Nur an echter Hardware prüfbar:** Karte ohne Schreibrecht oder ohne Datei → „nicht gespeichert“ bzw. Standardliste, das Spiel läuft weiter |
| 26 | Ton | Schritt: leises Klicken; Segment: zwei Töne; Rechteck: zwei Töne; Kette: drei Töne; Treffer: fallendes Rauschen; Stufe: Fanfare; Ende: fallende Töne; Stern, Eisblock; Linie zerstört: Knistern; Titelmusik mit sechs Stimmen; nichts bleibt hängen; **alle vier SID-Chips** hörbar (Musik und Effekte getrennt) |
| 27 | PAL und NTSC | Beide Normen: Bild vollständig sichtbar, 80 × 50 Zeichen, Tonhöhen gleich, Tempo ähnlich |
| 28 | 40 MHz | Spielt flüssig, keine Aussetzer beim Füllen (DMA) und beim Spielerwechsel (Gitter sichern/laden) |
| 29 | Zusatzfunktionen aus | Variante mit `.const EXTRA_STAR = 0`, `EXTRA_ICE = 0`, `EXTRA_GUARD = 0`, `EXTRA_HISCORE = 0`, `EXTRA_2P = 0`, `ENEMY_DESTROYS_LINES = 0` vor dem Programmtext (siehe README): keine Gegenstände, nur Läufer und Streuner, keine Bestenliste, nur ein Spieler, keine zerstörten Linien; sonst dieselben Regeln |
| 30 | Selbstspiel (Test) | Variante mit `AUTOPLAY = 1` (zusätzlich `AUTOGOD = 1`, `AUTO_PLAYERS = 2`, `AUTO_SIMUL = 1`, `TESTDEMO = 1`): spielt allein, kein Titelbild; die Zähler ab `$CE84` laufen weiter |

## Dauerlauf (im Emulator durchgeführt)

`AUTOPLAY = 1`, `AUTOGOD = 1`, `TESTDEMO = 1`, `NOSAVE = 1`, `AUTO_PLAYERS = 2`; Xemu mit `-fastboot -prg …`, Zähler ab `$CE84` über den Monitor gelesen (Bilder, Spiele, höchste Stufe, Treffer, Rechtecke).

| Lauf | Norm | Einstellung | Dauer | Ergebnis |
|---|---|---|---|---|
| 1 | PAL (`-videostd` Standard) | zwei Spieler **gleichzeitig** (`AUTO_SIMUL = 1`) | rund 12,5 Minuten (37 373 Bilder) | ohne Absturz; Stufe 3 erreicht; Zähler laufen weiter |
| 2 | NTSC (`-videostd 1`, Fensterzeile „NTSC“) | zwei Spieler **abwechselnd** (`AUTO_SIMUL = 0`), mit `AUTOGOD` (kein Spielerwechsel, jeder Treffer entfällt) | rund 13,6 Minuten (49 132 Bilder) | ohne Absturz; Zähler laufen weiter |
| 3 | PAL | zwei Spieler **abwechselnd**, **ohne** `AUTOGOD`, `AUTO_SIMUL = 0` | gut 3 Minuten, Bildfolge alle 25 s | ohne Absturz; nach jedem Treffer wechselt der Spieler, jeder behält sein Gitter, die Spuren (gelb und cyan) und gefüllten Rechtecke bleiben je Gitter erhalten |

Zusätzlich ohne `AUTOGOD`, zwei Spieler gleichzeitig (Bilder in `bilder/`): Treffer, Lebenabzug, Zerstören von Linien (Regel 16.2), Spielende und die Namenseingabe der Bestenliste wurden durchlaufen; auch hier ohne Absturz. Die Bestenliste wurde dabei **nicht** gespeichert (`NOSAVE`).

## Hinweise

- Das Spiel fragt die Tastatur nicht ab. Während des Spiels keine Taste drücken (die Tastaturmatrix teilt sich die Leitungen mit den Joysticks).
- Alle Texte stehen in `hud.asm` (klein geschrieben, ohne Umlaute, wie im Konzept).
- Der erste Test in `troxx.vctest` („Vorbereitung“) blendet die ROMs aus; ohne ihn laufen die Routinentests in Xemu nicht (README, Abschnitt Tests).
