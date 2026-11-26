# Testfassungen (nicht Teil des Standardbaus)

Die fertigen `.prg` liegen nicht im Repository (Binärdateien); sie werden mit den Befehlen unten im Projektordner gebaut (Ausgabe z. B. `-o szenario-node.prg`) und liegen im Handtest-Paket.

`zerstoerung-hold.prg`: Der Rechner spielt allein (`AUTOPLAY=1`, zwei Spieler abwechselnd, `NOSAVE=1`, ohne `AUTOGOD`); jedes Aufblitzen zerstörter Segmente (Linien- und Knotenzerstörung, Regel 16.2) wird **150 Bilder** (3 s PAL) lang gehalten, damit man es aufnehmen kann. Start: `xmega65 -fastboot -prg zerstoerung-hold.prg -prgmode 65`. Neu bauen (andere Haltezeit n = 1..255):

```
vcc asm main.asm --target mega65 --cpu 45gs02 --illegals deny -o zerstoerung-hold.prg -D AUTOPLAY=1 -D AUTO_PLAYERS=2 -D NOSAVE=1 -D HOLD_DESTROY=150
```

## Feste Szenarien (Bildschirmaufnahmen)

Beide Fassungen: `AUTOPLAY=1 AUTOGOD=1 NOSAVE=1 HOLD_DESTROY=100` (Aufblitzen je Segment 100 Bilder = 2 s PAL). Start: `xmega65 -fastboot -prg szenario-destroy.prg -prgmode 65`. Nach dem Start von Level 1 ist das Gitter vorbelegt, ein einziger Gegner steht bei (26, 18) und betritt im ersten Schritt den Knoten (25, 18); sichtbar nach etwa 2 bis 3 s (nach „bereit“).

- `szenario-destroy.prg` (`-D SCENARIO_DESTROY=1`): drei abgefahrene Segmente enden am Knoten (25, 18): waagerecht links (gelb), senkrecht oben (gelb), senkrecht unten (cyan). Der Gegner zerstört sie nacheinander (Reihenfolge rechts, links, runter, hoch; je Segment Blitz, Knistern, Punktabzug); danach sind alle Knoten blau.
- `szenario-node.prg` (`-D SCENARIO_NODE=1`): ein waagerechtes Segment (20..25, 18) wird zerstört. Vorher gefärbt: die frei stehenden Knoten (10, 12) gelb und (40, 24) cyan sowie ein Segment (50..55, 12) mit Endknoten. Nach der Zerstörung sind die frei stehenden Knoten und die Endknoten (20, 18), (25, 18) wieder blau; das Segment (50..55, 12) bleibt gelb.

Im Rechenmodell (Interpreter über dieselbe `main.asm`) liefen beide Szenarien wie beschrieben; in Xemu noch nicht. Neu bauen: `vcc asm main.asm --target mega65 --cpu 45gs02 --illegals deny -o szenario-node.prg -D AUTOPLAY=1 -D AUTOGOD=1 -D NOSAVE=1 -D SCENARIO_NODE=1 -D HOLD_DESTROY=100`.
