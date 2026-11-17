# Stilblatt „Fasen-Grotesk 8“ (Stand 2, 2026-11-17)

Lizenz: CC0-1.0. Dieses Blatt wurde **vor** dem ersten Zeichen beschlossen und belegt, dass die Formen der Sätze „VC-Standard“ aus Regeln kommen.
Die Sätze entstehen mit `tools/gen_vc_standard.py` (Regelentwurf); die Gestaltung ist vom Projektinhaber freigegeben
(Feld `freigabe` im Block `origin`). Es werden **keine** Original-ROM-Zeichen und **keine** Nachbauten gelesen, kopiert, abgepaust oder als Vorlage angesehen.
Das Blatt ist keine Rechtsaussage; nichts hier ist als „rechtlich sicher“ gemeint.

## 1. Raster

* Zelle 8×8, Spalten 0 bis 7 (links nach rechts), Zeilen 0 bis 7 (oben nach unten).
* **Buchstaben, Ziffern, Satzzeichen: 5 Spalten breit (Spalten 1 bis 5)**, links 1 Pixel, rechts 2 Pixel Rand (eigene Proportion, luftiger als 6 Spalten).
* **Versalhöhe 6** (Zeilen 0 bis 5), Bezugslinie = Zeile 5; **x-Höhe 4** (Zeilen 2 bis 5); Oberlängen reichen bis Zeile 0; **Unterlängen** liegen auf den Zeilen 6 und 7.
* **Satzlage:** Im Satz „Groß/Grafik“ (keine Unterlängen) rutscht alles um eine Zeile nach unten (Versalhöhe auf den Zeilen 1 bis 6), damit das Zeilenbild nicht oben klebt. Im Satz „Klein/Groß“ und beim Atari bleibt die Bezugslinie in Zeile 5. Ausnahme: der Unterstrich liegt fest auf Zeile 7.

## 2. Strich und Fase

* Strichstärke durchgehend **1 Pixel**, Stämme und Querstriche ohne Verdickung.
* **Fasen statt Rundungen:** Runde Formen (C, G, O, Q, S, U, a, c, e, o, 0 …) bekommen ihre Rundung nur durch das Weglassen des Eckpixels (1-Pixel-Fase); es gibt keine Doppelpixel-Kurven.
* Querbalken (A, E, F, H, P, R, e, f, t) liegen auf Zeile 3 (Versalien) beziehungsweise 4 (e), also unter der Mitte.
* Ziffer 0 hat einen Mittelpunkt; I hat Fußstriche (Serifen), l einen Haken – damit I, l, 1 und | verschieden sind.

## 3. Satzzeichen und Besonderes

* Satzzeichen halten die Breite 5 oder weniger; Komma und Semikolon haben eine Schwanzzeile unter der Bezugslinie.
* Pfeile (↑ ← → ↓) und Pfundzeichen folgen dem Raster; Umlaute gibt es in den Standardsätzen **nicht** (Entscheidung 4); der Atari-Satz „International“ enthält 32 Zeichen mit Akzent nach der Codebelegung des Systems (ungeprüft); Akzente liegen auf den Zeilen 0 und 1, Versalien mit Akzent haben 5 statt 6 Zeilen (eine doppelte Zeile entfällt).

## 4. Grafikzeichen (aus Regeln)

* **Linien und Rahmen:** 1 Pixel stark, durch die **Mitte** der Zelle (Spalte 3 und Zeile 3), damit sich Nachbarzellen nahtlos verbinden. Ecken, T-Stücke und Kreuz entstehen aus diesen Linien.
* **Abgerundete Ecken:** wie die Fasenregel; die Arme berühren sich nur diagonal, das Eckpixel in der Mitte fehlt.
* **Blöcke:** Viertel, Hälften, Dreiviertel und Balken aus ganzen Pixelblöcken (4×4-Ecken, 1 bis 4 Zeilen oder Spalten).
* **Dreiecke und Diagonalen:** Dreiecke aus der Bedingung x+y≥7 beziehungsweise x≥y (und gespiegelt); Diagonalen laufen über die Zellecken (ein Pixel je Zeile).
* **Raster:** 25 % (jedes zweite Pixel jeder zweiten Zeile, Zeilen versetzt), 50 % (Schachbrett), 75 % (Umkehrung der 25 %), Schraffur (jedes vierte Pixel auf der Diagonalen).
* **Kartenfarben, Kreise, Pi, Haken:** je ein eigenes kleines Bild, symmetrisch aufgebaut (Herz, Pik, Karo, Kreuz, Voll- und Hohlkreis).
* Grafikzeichen behalten die **Bedeutung** an ihrem Code, nicht die Form eines Originals. Die Belegung (welcher Code welche Bedeutung) folgt den Handbüchern der Systeme nach bestem Wissen und ist **am Gerät oder Handbuch zu prüfen**.

## 5. Zeichen mit zwingender Geometrie (Ausnahmeliste)

Leerzeichen, Vollblock, Viertel-/Halb-/Dreiviertelblöcke, Balken (volle Zeilen oder Spalten), die Raster 25/50/75 % (auch in einer Hälfte, auch als Schrägschraffur)
und die **geometrisch erzwungenen** Grafikzeichen zeichnet jede Gestaltung gleich. Die letzte Klasse („geometrisch“, Entscheidung des Projektinhabers 2026-11-17)
umfasst Linien, Rahmen, Ecken, T-Stücke, Kreuze, Dreiecke und Diagonalen (auch als Negativ); Begründung: Nachbarzellen müssen nahtlos anschließen. Sie gilt nur für
**Grafikcodes**, nie für Buchstaben, Ziffern oder Satzzeichen. Die Zeichen stehen mit ihrer Klasse in `ausnahmen.json` (vom Generator aus dem Bild erzeugt) und werden von
der Gleichheitsprüfung nicht beanstandet; ein Test hält Liste und Bilder in Einklang.

Abstandsprüfung (`tools/charset_distance.py`): kleinster Abstand zu lokalen Vergleichsdateien **3** für Buchstaben sowie Symbole und Ziffern, **6** für alle übrigen
nicht ausgenommenen Zeichen.

### Einfache Formen und Übereinstimmungen

Bei einfachen Formen in einem 8×8-Raster (Klammern, Punkt, Komma, Strich, Doppelpunkt, die I-Familie, Linien- und Rahmenzeichen) sind Übereinstimmungen mit anderen Zeichensätzen funktional bedingt: Die Form folgt der Funktion, und das Raster lässt kaum Spielraum. Die Formen entstanden aus den Regeln dieses Stilblatts, ohne ein Vorbild anzusehen. Eine lokale Abstandsprüfung durch den Projektinhaber ergab bei wenigen solcher Zeichen kleine Abstände; der Projektinhaber hat diese Sätze trotzdem freigegeben (`origin.freigabe`, `origin.freigabe_vermerk`; der Prüfstempel `origin.check` bleibt „offen“). Das ist keine Rechtsaussage.

## 6. Ableitungen

* Invertierte Hälfte (Codes 128 bis 255) = Bitnegation der Codes 0 bis 127 (C64, C128, VC-20, Plus/4, MEGA65).
* VC-20, Plus/4 und C16, C128 (40 Spalten) übernehmen die C64-Sätze unverändert; C128 VDC 8×16 durch Zeilenverdopplung; MEGA65 (Vollfarbmodus) durch Umrechnung 1 Bit → 8 Bit (Palettenfarbe 1).
* Atari: 128 Zeichen in ANTIC-Reihenfolge (Code 0 bis 63 = ASCII 32 bis 95, 64 bis 95 Grafik, 96 bis 127 ASCII 96 bis 127 mit Karo und Pik); die Inversdarstellung macht die Hardware.

## 7. Verboten

Original- oder Nachbau-Zeichen auslesen, kopieren, abpausen oder als Vorlage ansehen; Nachzeichnen aus der Erinnerung; Hash-Listen einzelner Originalzeichen einchecken.

## 8. Änderungen Stand 2 (2026-11-17)

Zur Abstandsprüfung bekamen einige Zeichen mehr Eigenart, ohne die Regeln zu ändern (Strich 1 Pixel, Fasen statt Rundungen, Breite 5): Plattenserifen bei **I** und **1**,
Kopfserife und Aufstrich bei **L**, Haken an den Balkenenden und Fußserife bei **T**, Haken am oberen Arm des **K**, Stamm links mit Fuß nach rechts bei **i** und **l**,
offene **4**, Kopfbalken bei **!**, kleines Plus, unterbrochener Strich, zweipunktige Punkte, Winkel mit Haken bei **<** und **>**, Plattenklammern **[ ]**, größere Pfeile.
Es wurde dabei **kein** Fremdzeichensatz angesehen.
