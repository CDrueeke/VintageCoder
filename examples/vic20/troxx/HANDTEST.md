# Troxx (VC-20): Handpruefliste fuer den Inhaber

Stand: Etappe 7. Bisher nur im Emulator (VICE `xvic`, PAL und NTSC, ohne Speichererweiterung) geprueft, nicht an echter Hardware. Alle Eingaben ausser dem Joystick wurden im Emulator durch Schreiben in die VIA-Register nachgestellt; die echten Joystick-Leitungen sind **nicht** getestet.

**Start:** `vcc build` im Ordner `examples/vic20/troxx`, dann `xvic` ohne `-memory` mit `build/main.prg` (Autostart) oder ueber die IDE (Ziel VC-20 unerweitert). Joystick im Anschluss (Emulator: `-joydev1 1` = Ziffernblock 8 4 6 2, Feuer = Ziffernblock 0 oder Alt). Norm NTSC: `xvic -ntsc`.

Bei jedem Punkt: bestanden / nicht bestanden / Bemerkung.

| Nr | Pruefpunkt | Erwartet |
|---|---|---|
| 1 | Start | Titelbild: TROXX (gelb), "TEMPO n", "FEUER STARTET"; schwarzer Rand, Bild mittig |
| 2 | Tempowahl | Joystick links/rechts aendert die Ziffer von 1 bis 9, nicht darueber hinaus; gedrueckt halten wiederholt langsam |
| 3 | Spielstart | Feuer startet; Gitter 5 x 4 in Blau, Figur (weiss, Gesicht) unten links, "BEREIT", nach etwa 1,5 s geht es los; ein Gegner (Stufe 1) oben rechts |
| 4 | Joystick hoch, runter, links, rechts | Figur laeuft nur, solange die Richtung gehalten wird, Schritt fuer Schritt (zwei Bilder wechseln); **rechts** besonders pruefen (VIA 2) |
| 5 | Linien und Knoten | Auf einer Linie nur vor/zurueck; Abbiegen nur an Knoten; ausserhalb des Gitters kein Schritt; bei zwei gehaltenen Richtungen gilt hoch, runter, links, rechts in dieser Reihenfolge |
| 6 | Faerben, Punkte | Befahrene Linien werden gelb; +1 je neues Zeichen, +10 je Segment (Anzeige SCORE oben) |
| 7 | Fuellen | Sind alle vier Kanten eines Rechtecks gelb: Fuellmuster erscheint Zeile fuer Zeile (4 Muster, Farbe nach Zeile), +100 |
| 8 | Kettenreaktion | Schliesst ein Segment zwei Rechtecke zugleich: beide fuellen zugleich, zusammen +400, andere Tonfolge |
| 9 | Gegner | Laeufer (rot) naehert sich der Figur, Streuner (violett, ab Stufe 2) laeuft zufaellig; beide wenden nicht mitten auf einer Linie, entscheiden an Knoten; Bild wechselt |
| 10 | Kollision, Leben | Beruehrung/Zusammenstoss: Treffer-Zeichen, "TREFFER", Rauschen, ein Leben weniger (Herz-Ziffer unten links), Figuren zurueck auf die Startknoten, Farben und Rechtecke bleiben; danach kurzes Blinken (cyan), in dem nichts passiert |
| 11 | Extraleben | Je 10000 Punkte ein Leben mehr, hoechstens 9 |
| 12 | Stufenwechsel | Alle 20 Rechtecke gefuellt: "STUFE OK", Fanfare, +500, LV hochzaehlen, neues leeres Gitter; Gegnerzahl 1, 2, 2, 3, 3, 3, 4 ... je Stufe; alle 2 Stufen Tempo + 1 bis 9 |
| 13 | Tempo | Tempo 1 deutlich langsamer als Tempo 9; Anzeige SPD stimmt mit der Wahl; Spielgefuehl in PAL und NTSC aehnlich |
| 14 | Pause | Feuer im Spiel: Meldung "PAUSE", alles steht, Ton aus; nochmal Feuer: weiter |
| 15 | Spielende | Nach dem letzten Leben "SPIEL ENDE", Ton, nach etwa 3 s Titelbild; BEST (Anzeigezeile unten) zeigt die beste Punktzahl der Sitzung (nur im RAM, nach Neustart weg) |
| 16 | Ton | Schritt: kurzes Klicken; Segment: zwei kurze Toene; Rechteck: zwei Toene; Kette: drei Toene; Treffer: fallendes Rauschen; Stufe: Fanfare; Ende: fallende Toene; nichts bleibt haengen |
| 17 | PAL und NTSC | Beide Normen: Bild vollstaendig sichtbar und mittig (Bildlage PAL $0C/$26, NTSC $05/$19 laut KERNAL; bei Abweichung am Fernseher melden), Tonhoehen gleich |
| 18 | RESTORE-Taste | Tut nichts (kein Absprung ins BASIC) |
| 19 | Gegner zerstoeren Linien | Betritt ein Gegner ein Linienzeichen (kein Knoten) eines Segments, das an **kein gefuelltes Rechteck** grenzt und in dem mindestens ein Zeichen gelb ist: die Zeichen blitzen kurz auf (Stern, zwei Bilder), ein fallender Ton erklingt, das ganze Segment wird wieder blau, Punkte werden abgezogen (1 je gelbes Zeichen, +10 bei ganz abgefahrenem Segment, nie unter 0) |
| 19a | **Knotenberuehrung (neu)** | Betritt ein Gegner einen **Knoten** (Kreuzung, T-Stueck, Ecke), werden alle dort endenden Segmente (bis zu vier, auch das, aus dem er kam) wie unter 19 geprueft: freistehende gelbe Verbindungen (auch teilweise gelbe) werden zurueckgesetzt, geschuetzte (angrenzendes Rechteck gefuellt) bleiben; mehrere Segmente an einem Knoten blitzen nacheinander (je zwei Bilder) mit je einem Ton; die Knotenfarbe selbst aendert sich nicht |
| 19b | **Knotenregel (neu)** | Ein gelber Knoten (Ecke, T-Stueck, Kreuzung) wird bei einer Zerstoerung mit zurueckgesetzt (blau), wenn an ihm kein gelbes Segment mehr anschliesst (keine der vier Nachbarzellen auf den Linien gelb) und kein angrenzendes Rechteck gefuellt ist; ein Knoten mit gelbem Segment oder an einem gefuellten Rechteck bleibt gelb; ein Knoten, auf dem die Figur steht, bleibt; Knoten bringen keine Punkte; die Figur faerbt einen Knoten gelb, wenn sie ihn verlaesst |
| 20 | Geschuetzte Linien | Ein Segment am Rand eines **gefuellten** Rechtecks bleibt gelb, auch wenn ein Gegner darueberlaeuft; ein noch ganz blaues Segment loest nichts aus (kein Ton, kein Blitz) |
| 21 | Abgeschlossenes Segment zerstoert | Das Segment war ganz gelb, die angrenzenden Rechtecke noch nicht gefuellt: nach der Zerstoerung fehlt dem Rechteck diese Kante (es fuellt sich erst wieder, wenn die Kante neu abgefahren ist); Punkte: 13 (waagerecht) bzw. 14 (senkrecht) weniger |
| 22 | Figur auf zerstoertem Segment | Steht die Figur auf dem Segment, laeuft sie weiter; ihre Spur dahinter ist weg. (Bekannte Kleinigkeit: die Zelle unter der Figur wird beim Weiterlaufen gelb, ohne Punkt.) |
| 23 | Schalter | In `main.asm` / per `-D ENEMY_DESTROYS_LINES=0` abschaltbar: dann zerstoeren Gegner nichts (alte Regel) |
| 24 | Selbstspiel (Test) | `vcc asm main.asm --target vic20 -D AUTOPLAY=1` (zusaetzlich `-D AUTOGOD=1`): spielt allein, kein Titelbild |

Notizen: Beim Emulator-Test der Verknuepfung `RUN/STOP` oder Tastatur passiert nichts; die Tastatur wird nicht abgefragt.
