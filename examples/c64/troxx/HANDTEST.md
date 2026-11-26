# Troxx (C64): Handprüfliste für den Inhaber

Stand: erste Fassung (Welle W2). Bisher nur im Emulator (VICE `x64sc` 3.10, PAL und NTSC) geprüft, nicht an echter Hardware. Der Joystick wurde im Emulator nur über Testeingaben (Skripte in `tools/`) nachgestellt, die **echten Joystick-Leitungen an beiden Anschlüssen sind nicht getestet** (Punkte 4–8).

**Start:** `vcc build` im Ordner `examples/c64/troxx`, dann `x64sc build/disk.d64` (Autostart; so landet die Bestenliste auf der Diskette `build/disk.d64`) oder über die IDE (Ziel C64). Joysticks: Emulator `-joydev1 1 -joydev2 2` (Anschluss 1 = Ziffernblock, Anschluss 2 = Zifferntasten-Gruppe laut VICE-Einstellung „Joystick“) oder echte Geräte. NTSC: `x64sc -ntsc build/disk.d64`. Während des Spiels **keine Taste** drücken (die Tastaturmatrix teilt sich die Leitungen mit den Joysticks).

Bei jedem Punkt: bestanden / nicht bestanden / Bemerkung.

| Nr | Prüfpunkt | Erwartet |
|---|---|---|
| 1 | Start | Titelbild: gelber Rahmen mit „T R O X X“; darunter PLAYERS 1, JOYSTICK PORT 2, SPEED 5 (Zeiger bei PLAYERS), „FIRE = START“, „HIGH SCORES“ mit zehn Zeilen und rechts davon `NO FILE` (erste Diskette) oder `DISK OK`; Rahmen hellblau, Feld blau; Titelmusik mit drei Stimmen (Bass, Arpeggio, Melodie), wiederholt sich |
| 2 | Menü bedienen | Hoch/runter bewegt den Zeiger zwischen PLAYERS, JOYSTICK, SPEED; links/rechts ändert den Wert (SPEED 1–9, nicht darüber hinaus; gedrückt halten wiederholt langsam). Das Menü reagiert auf **beide** Anschlüsse |
| 3 | Spielerzahl und Joystick | PLAYERS 2 schaltet bei JOYSTICK zusätzlich `TWO STICKS` frei; mit PLAYERS 1 gibt es nur `PORT 2` und `PORT 1`; wird PLAYERS wieder 1, springt `TWO STICKS` auf `PORT 2` |
| 4 | Ein Spieler, Anschluss 2 | Voreinstellung, Feuer startet (Musik endet): Gitter 8 × 5 in Violett, Figur (weißes Gesicht) unten links, „READY“, Seitenleiste rechts (SCORE, HI, LEVEL, LIVES als drei Herzen, SPEED, BONUS); ein Gegner (rotes Krebstier) oben rechts. Joystick **am Anschluss 2**: Figur läuft nur, solange die Richtung gehalten wird, in allen vier Richtungen; **rechts** und **Feuer** besonders prüfen |
| 5 | Ein Spieler, Anschluss 1 | Im Titelbild JOYSTICK `PORT 1` wählen: jetzt steuert der Joystick am **Anschluss 1**, der am Anschluss 2 tut nichts |
| 6 | Zwei Spieler, zwei Joysticks | PLAYERS 2, `TWO STICKS`: Spieler 1 (Anschluss 1) beginnt („PLAYER 1 READY“, Anzeige PLAYER 1); nach dem ersten verlorenen Leben ist Spieler 2 am **Anschluss 2** dran (eigenes, leeres Gitter, eigene Punkte); danach wechseln sie sich nach jedem Treffer ab, jeder behält sein Gitter, seine Punkte und Leben |
| 7 | Zwei Spieler, ein Joystick | PLAYERS 2, `PORT 2` (oder `PORT 1`): vor jedem Zug „PASS JOYSTICK – FIRE“, Feuer startet den Zug, der Joystick bleibt am gewählten Anschluss |
| 8 | Spielende bei zwei Spielern | Hat ein Spieler kein Leben mehr, spielt der andere weiter, ohne Wechsel; nach dem letzten Leben des zweiten „GAME OVER“, danach (wenn die Punkte reichen) die Namenseingabe für Spieler 1, dann für Spieler 2 |
| 9 | Pixelbewegung | Figur und Gegner gleiten **glatt** (acht Punkte je Zeichen) entlang der Linien, ohne Ruckeln oder Zittern, in jedem Tempo; die Sprites sitzen mittig auf den Linien, auch am linken und rechten Rand des Gitters |
| 10 | Linien und Knoten | Auf einer Linie nur vor/zurück; Abbiegen nur an Knoten; außerhalb des Gitters kein Schritt; bei zwei gehaltenen Richtungen gilt hoch, runter, links, rechts in dieser Reihenfolge |
| 11 | Färben, Punkte | Befahrene Linien werden gelb; +1 je neues Zeichen, +10 je Segment (SCORE) |
| 12 | Füllen | Sind alle vier Kanten eines Rechtecks gelb: Füllmuster (vier Mehrfarben-Muster: Schraffur, Punkte, Mauer, Schachbrett, Farbe je Zeile: grün, gelb, violett, rot, weiß), zeilenweise, +100 |
| 13 | Kettenreaktion | Schließt ein Segment zwei Rechtecke zugleich: beide füllen zugleich, zusammen +400, andere (längere) Tonfolge |
| 14 | Gegner | **Läufer** (rot) nähert sich der Figur; **Streuner** (orange, ab Stufe 2) läuft zufällig und setzt manchmal aus; **Wächter** (cyan Tintenfisch, ab Stufe 4) läuft einen festen Rundgang um den Rand; alle wenden nicht mitten auf einer Linie; Bild wechselt je Schritt |
| 15 | Kollision, Leben | Berührung oder Platztausch: Treffer-Bild (Sternchen, rot), „HIT“, Rauschen, ein Herz weniger, Figuren zurück auf die Startknoten, Farben und Rechtecke bleiben; danach kurzes Blinken (weiß/cyan), in dem nichts passiert |
| 16 | Extraleben | Je 10000 Punkte ein Leben mehr, höchstens 9 (die Seitenleiste zeigt höchstens sechs Herzen) |
| 17 | Stufenwechsel | Alle 40 Rechtecke gefüllt: „LEVEL CLEAR“, Fanfare (drei Stimmen), +500, LEVEL hochzählen, neues leeres Gitter; Gegnerzahl 1, 2, 2, 3, 3, 3, 4 … je Stufe; alle zwei Stufen Tempo + 1 bis 9 |
| 18 | Tempo | Tempo 1 deutlich langsamer als Tempo 9; Anzeige SPEED stimmt mit der Wahl; Spielgefühl in PAL und NTSC ähnlich |
| 19 | Pause | Feuer im Spiel: „PAUSE“, alles steht (Gegner, Sprites, Zeitgeber des Gegenstands), Ton aus; nochmal Feuer: weiter |
| 20 | Bonusstern | Ab und zu erscheint ein gelber Stern auf einer Linie (Symbol bei BONUS); nach zehn Sekunden verschwindet er; die Figur nimmt ihn auf: +50, kurzer Klang |
| 21 | Eisblock | Ab und zu ein hellblauer Eisblock; aufgenommen: alle Gegner stehen fünf Sekunden still (hellblau), die Figur läuft weiter, dann laufen sie wieder; eigener Klang |
| 22 | Bestenliste, Namenseingabe | Nach dem Spielende bei genug Punkten (Platz unter den ersten zehn): „NEW HIGH SCORE“, „ENTER NAME“, drei Buchstaben mit dem Joystick (hoch/runter A–Z und Leerzeichen, links/rechts Stelle, Feuer fertig); „SAVING…“; im Titelbild steht der Eintrag an der richtigen Stelle, rechts `DISK OK` |
| 23 | Bestenliste auf Diskette | Nach Neustart (gleiche Diskette `build/disk.d64`, Datei `TROXX.HS`) ist die Liste wieder da; HI in der Seitenleiste = erster Platz |
| 24 | Ohne Laufwerk, Schreibschutz | Ohne Laufwerk 8 (Emulator: Laufwerk aus) läuft alles; das Titelbild zeigt `NO DRIVE`, nach einem Spiel „SAVING…“ ohne Absturz; mit schreibgeschützter Diskette `DISK ERR`, das Spiel läuft weiter |
| 25 | Ton | Schritt: leises Klicken; Segment: zwei Töne; Rechteck: zwei Töne; Kette: drei Töne zweistimmig; Treffer: fallendes Rauschen mit tiefem Ton; Stufe: Fanfare; Ende: fallende Töne; Stern, Eisblock; Titelmusik; nichts bleibt hängen |
| 26 | PAL und NTSC | Beide Normen: Bild vollständig sichtbar, Tonhöhen gleich, Tempo ähnlich |
| 27 | RESTORE-Taste | Tut nichts (kein Absprung ins BASIC) |
| 28 | Zusatzfunktionen aus | Variante mit `-D EXTRA_STAR=0 -D EXTRA_ICE=0 -D EXTRA_GUARD=0 -D EXTRA_HISCORE=0 -D EXTRA_2P=0` (siehe README): keine Gegenstände, nur Läufer und Streuner, keine Diskette, nur ein Spieler; sonst dieselben Regeln |
| 29 | Linie zerstört (Regel 16.2) | Ein Gegner läuft über ein abgefahrenes, noch **nicht** an ein gefülltes Rechteck grenzendes Segment: die Zeichen blitzen kurz weiß, der Rand blitzt rot/weiß, die Punktzahl wird kurz rot und sinkt (1 je Zeichen, bei ganzem Segment 10 mehr, nie unter 0), Knistern mit fallendem Ton (anders als beim Treffer), das Segment ist wieder violett; Segmente an gefüllten Rechtecken bleiben; Knoten und eingefrorene Gegner (Eisblock) zerstören nichts; mit `-D ENEMY_DESTROYS_LINES=0` gilt die alte Regel |
| 30 | Knoten gelb, Knoten zerstört, Knoten zurückgesetzt | Die Figur färbt jeden Knoten (Ecke, Kreuzung, T-Stück), den sie betritt, **gelb**. Erreicht ein Gegner einen Knoten, werden alle dort endenden gelben Segmente ohne angrenzendes gefülltes Rechteck zurückgesetzt (ein Blitz, ein Ton je Segment, Punkte ab). Nach jeder Zerstörung werden gelbe Knoten **violett**, an die kein gelbes Segment mehr anschließt und die an kein gefülltes Rechteck grenzen (auch weit entfernte); ein Knoten, auf dem Figur oder Gegner steht, bleibt gelb; Knoten geben keine Punkte |
| 31 | Selbstspiel (Test) | Variante mit `-D AUTOPLAY=1` (zusätzlich `-D AUTOGOD=1`, `-D AUTO_PLAYERS=2`, `-D TESTDEMO=1`): spielt allein, kein Titelbild; die Zahlen unten rechts zählen weiter |

Notizen: Das Spiel fragt die Tastatur nicht ab. Texte auf dem Bildschirm sind englisch (wie im Mockup `gm-5-c64.png`); sollen sie deutsch sein, steht alles an einer Stelle in `hud.asm`.
