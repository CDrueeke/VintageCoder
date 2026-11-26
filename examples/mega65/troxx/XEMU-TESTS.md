# Troxx (MEGA65): Xemu-Läufe für die Windows-Sitzung

Ausgangspunkt: Branch `claude/troxx-mega65-265e63` (Projekt `examples/mega65/troxx`). Alles im Projektordner. Nur lesen und melden, nichts ändern, nichts pushen.

## 1. Trap-Tests (wichtigster Lauf)
```
vcc build .
vcc test . --backend xemu
```
Erwartet: 4 statische Prüfungen und **94 Tests** (der erste, „Vorbereitung“, blendet die ROMs aus; er muss laufen, sonst schlagen die übrigen fehl) bestehen. 25 Tests sind neu (Knotenregel): `node_free` (14), `paint_node` (4), `player.step` auf Knoten (1), `seg_destroy` mit Endknoten (3, Zeitgrenze 60 s) und `enemies.touch` (3, bis 120 s). Alle neuen Tests laufen im Rechenmodell (Python-Interpreter über dieselbe `main.prg`, `put_tile`, `wait_frame` und `sound.play` als Rücksprung) bereits grün; echte Xemu-Läufe fehlen. Bei einem Fehler: Name des Tests, `expect`-Zeile und den gelesenen Wert melden. Nach einer Zeitüberschreitung startet der Läufer Xemu neu, dann schlagen die folgenden Tests fehl (der erste Fehler zählt).

## 2. Dauerlauf
```
vcc asm main.asm --target mega65 --cpu 45gs02 --illegals deny -o auto.prg -D AUTOPLAY=1 -D AUTOGOD=1 -D TESTDEMO=1 -D NOSAVE=1 -D AUTO_PLAYERS=2 -D AUTO_SIMUL=1
xmega65 -fastboot -prg auto.prg -prgmode 65
```
PAL (Standard) und NTSC (`-videostd 1`), je mindestens 10 Minuten; einmal mit `-D AUTO_SIMUL=0` (abwechselnd). Zähler ab `$CE84` über den Monitor lesen (Bilder, Spiele, höchste Stufe, Treffer, Rechtecke): sie müssen weiterlaufen, kein Absturz, kein Stehenbleiben. Neu gegenüber dem ersten Lauf: Knoten färben sich und werden zurückgesetzt, Gegner zerstören an Knoten bis zu vier Segmente.

## 3. Bildschirmfotos (für den Handtest-Vergleich)
Ohne `AUTOGOD` (`-D AUTOPLAY=1 -D AUTO_PLAYERS=2 -D NOSAVE=1`, optional `-D AUTO_SIMUL=1`) laufen lassen und festhalten:
1. Gelbe und cyanfarbene **Knoten** auf dem Gitter nach dem Abfahren (Vergleich mit den alten Bildern in `bilder/`).
2. Eine **Zerstörung an einem Knoten**: mehrere Segmente blitzen weiß auf; danach sind die frei stehenden Knoten wieder blau.
3. Ein Knoten **neben einem gefüllten Rechteck** bleibt gefärbt.
Bitte jeweils kurz melden, ob die Knotenkachel (Kreuz, Ecke, T) in der Spielerfarbe richtig aussieht (kein Rest der alten Farbe, keine Lücke in der Linie).

## 4. Varianten bauen (kurz prüfen, ob sie starten)
`-D ENEMY_DESTROYS_LINES=0`, `-D EXTRA_2P=0`, `-D EXTRA_HISCORE=0` (die beiden letzten ließen sich vorher nicht übersetzen, jetzt schon; Start in Xemu ungeprüft). Fertig gebaut: `ohne-extras.prg` im Paket.

## 5. Zerstörungsbild
Die Testfassung `zerstoerung-hold` (Bauanleitung in `testfassungen/LIESMICH.md`; fertige `.prg` liegen im Paket, nicht im Repository) hält jedes Aufblitzen 150 Bilder lang: Zerstörung an Linie und Knoten aufnehmen.

Feste Szenarien für das Zerstörungsbild: `szenario-destroy` und `szenario-node` (Bauanleitung und Beschreibung in `testfassungen/LIESMICH.md`); die Zerstörung tritt etwa 2 bis 3 s nach dem Start ein und steht je Segment 2 s.
