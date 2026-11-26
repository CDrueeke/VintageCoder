# Troxx (VC-20, unerweitert) - Etappe 1 (Spike)

Eigenstaendiges Arcade-Beispiel fuer VCASM: Eine Figur faehrt die Linien eines Gitters (5 x 4 Rechtecke) ab; sind alle vier Kanten eines Rechtecks abgefahren, wird es gefuellt. Gegner laufen auf den Linien. Konzept: `assets/docs/Konzept-Beispiel-VC20-Gittermosaik.md` (Arbeitstitel "Gittermosaik").

**Stand: Etappen 1-7** (Referenzfassung VC-20 fertig, Handtest durch den Inhaber steht aus, siehe `HANDTEST.md`; Pruefsummen in `PRUEFSUMMEN.txt`). Programm 3068 Byte (hartes Limit 3071; Tabellen liegen zusaetzlich in den freien Zeichen 60..63). Spielregel 16.2 (Konzept): Gegner zerstoeren nicht geschlossene Linien (ein Segment ohne gefuelltes Nachbarrechteck wird zurueckgesetzt, Punkte werden abgezogen; das gilt beim Betreten eines Linienzeichens und beim Betreten eines Knotens fuer alle dort endenden Segmente; frei stehende gelbe Knoten werden dabei mit zurueckgesetzt (Knotenregel); schaltbar mit `.const ENEMY_DESTROYS_LINES`). Spielbar: Gitter abfahren (Segmente, Rechtecke, Fuellen, Kettenreaktion), Punkte (BCD), Leben, Stufen, Laeufer und Streuner als Gegner, Titelbild, Spielende.

| Datei | Inhalt |
|---|---|
| `main.asm` | Konstanten, Speicherplan, Start, Zustandsautomat |
| `vic.asm` | Bildinit, Norm-Erkennung, Zeichensatz kopieren, `wait_frame` |
| `grid.asm` | Datenmodell, Zeichnen, Segmente, Rechtecke, Fuellen |
| `player.asm` | Joystick, Bewegung, Faerben, Selbstspiel |
| `enemies.asm` | Zufall (LFSR), Entscheidung, Bewegung, Kollision |
| `sound.asm` | Klangmodul mit Effektlisten und Prioritaeten |
| `hud.asm` | Anzeigen, Meldungen, BCD-Punkte |
| `charset.asm` | 64 Zeichen ($1C00) |
| `data.inc` | Tabellen (zur Uebersetzungszeit erzeugt) |

Bauen: `vcc build`. Varianten per `vcc asm main.asm --target vic20 -D NAME=1`: `AUTOPLAY=1` (der Rechner spielt selbst, kein Titelbild), zusaetzlich `AUTOGOD=1` (dauernd unverwundbar, fuer den Dauerlauf). Test: `vcc test --backend vice` (`troxx.vctest`: statische Pruefungen und Routinen im Emulator; `troxx_regel0.vctest`: Regel abgeschaltet (Programm `build0/main.prg`, gebaut mit `vcc asm main.asm --target vic20 -D ENEMY_DESTROYS_LINES=0 --debug build0/main.vcdbg -o build0/main.prg`); `troxx_kurz.vctest`: sehr kurze Routinen, die der Testlaeufer gelegentlich nicht bestaetigt). Lauf: `xvic` ohne Speichererweiterung.

Bedienung: Titelbild: links/rechts waehlt das Tempo 1-9, Feuer startet; im Spiel Feuer = Pause; Joystick im Anschluss, die Figur laeuft nur, solange eine Richtung gehalten wird (an Knoten abbiegen).

Nur im Emulator geprueft, nicht an echter Hardware.
