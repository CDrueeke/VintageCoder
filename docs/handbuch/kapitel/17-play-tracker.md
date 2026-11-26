---
id: play-tracker
titel: PLAY-Tracker (MEGA65)
reihenfolge: 17
stufe: Einsteiger
zeit: 20 Minuten
siehe_auch: [tracker, basic]
referenz: [play, envelope, mega65]
---

# PLAY-Tracker (MEGA65)

**Ziel:** Du schreibst ein Musikstück für den MEGA65 in einer Tabelle und lässt VintageCoder daraus ein **BASIC-Programm** mit `PLAY`-Befehlen erzeugen, das du ohne eigene Spielroutine abspielst.

## Das brauchst du

- ein **MEGA65-Projekt** (Emulator Xemu, siehe [Willkommen](kapitel:willkommen)),
- das Kapitel [Der Tracker](kapitel:tracker) (Begriffe wie Zelle und Hüllkurve) ist hilfreich, aber nicht Pflicht.

## Wofür ist der Editor gedacht?

Der PLAY-Tracker ist für **schnelle Musik im BASIC des MEGA65**: Du schreibst ein Stück in einer Tabelle, VintageCoder macht daraus ein fertiges BASIC-Programm. Das passt für Titelmusik in BASIC-Spielen, Klangproben und Lernprojekte, bei denen du keine Maschinensprache schreiben willst.

Typische Einsätze:

- **Titelmusik** für ein BASIC-Spiel auf dem MEGA65.
- **Effekte**, die du mit `PLAY` aus dem eigenen BASIC-Programm auslöst.
- **Ein Klangexperiment**, das du auf dem echten Gerät anhören willst, ohne eine Spielroutine zu schreiben.

Im fertigen Programm ist das ein **Text**: ein BASIC-Programm mit `ENVELOPE`- und `PLAY`-Zeilen.

## Grundbegriffe ohne Vorwissen

- **BASIC 65:** der BASIC-Dialekt des MEGA65; er kennt eingebaute Musikbefehle.
- **PLAY:** BASIC-Befehl, der eine Zeichenkette mit Noten spielt.
- **Schritt:** eine Zeile im Raster; hier ein Sechzehntel.
- **Stimme:** eine Tonspur (sechs Stück).
- **Hüllkurve (ENVELOPE 0 bis 9):** Lautstärkeverlauf und Wellenform eines Klangs.
- **Tempo:** Geschwindigkeit des Stücks (1 bis 255).

## Wie unterscheidet sich der PLAY-Tracker?

Der MEGA65 hat in seinem BASIC-Dialekt (BASIC 65) den Befehl **`PLAY`**. Man übergibt ihm eine Zeichenkette wie `"V1T0O4QCEG"` („Stimme 1, Hüllkurve 0, Oktave 4, Viertelnoten C, E, G“), und der Rechner spielt sie, ohne dass du Interrupts programmierst. Der **PLAY-Tracker** erzeugt diese Zeichenketten aus einem Raster.

Die Unterschiede zum normalen Tracker:

- Es gibt **keine Muster und keine Reihenfolge**. Ein PLAY-Song ist ein einziges Raster aus **Schritten × Stimmen**.
- Ein **Schritt** ist ein **Sechzehntel**. Eine Note klingt, bis eine neue Note oder `===` (Pause) folgt.
- Der Song hat **6 Stimmen** und standardmäßig 32 Schritte.
- Statt Instrumenten gibt es **Hüllkurven 0 bis 9**. Sie entsprechen dem BASIC-Befehl `ENVELOPE n,a,d,s,r,wellenform,pw`.
- Das Ergebnis ist **Text** (ein BASIC-Programm), keine Maschinensprache.

## Mein erster PLAY-Song (am Beispiel `play_demo`)

1. **Beispiel öffnen:** Öffne auf der Startseite aus den Beispielprojekten `examples/mega65/play_demo` (zum Ändern nimmst du **Kopieren**). *Erwartet:* Im Baum liegt `music/tune_play.vcplay` (Gruppe „PLAY-Tracker“).
2. **Song öffnen:** Wähle oben den Hauptbereich **Musik** und klicke auf `music/tune_play.vcplay`; ein Doppelklick im Projektbaum geht genauso. *Erwartet:* Der Editor sieht aus wie der normale Tracker, mit **sechs Stimmen** und ohne Reihenfolge der Muster.
3. **Anhören:** **Song abspielen** oder die **Leertaste** (der Cursor steht im Raster). *Erwartet:* Das Stück spielt, die Zeile läuft mit.
4. **Eine Note setzen:** Zelle anklicken, die Taste links unten drücken (deutsche Tastatur: **Y**). *Erwartet:* Note, Cursor eine Zeile weiter.
5. **Speichern:** **Cmd/Strg+S**; danach **Exportieren …**, siehe unten.

Einen neuen PLAY-Song legt der Dialog **Neue Datei** in dieser Version noch nicht an. Beginne mit dem Beispielprojekt oder einem vorhandenen Song.

## Aufbau der Oberfläche

Wie im [Tracker](kapitel:tracker): links der Song mit den Hüllkurven, in der Mitte das Raster mit der Werkzeugleiste, rechts die Einstellungen der gewählten Hüllkurve mit Klaviatur und Lautstärke. Statt der Reihenfolge steht links der Hinweis „PLAY-Dateien haben ein Muster und keine Reihenfolge.“; ein Muster wird einmal gespielt. In der Werkzeugleiste stellst du **Tempo** (1 bis 255) und **Zeilen** (die Zahl der Schritte) ein, die **Lautstärke** (0 bis 15) rechts unter der Klaviatur. Noten tippst du wie beim Tracker nach der Tastenposition: untere Reihe `Y S X D C V G B H N J M` für eine Oktave, obere Reihe `Q 2 W 3 E R 5 T 6 Z 7 U I` (QWERTZ) für die nächste. **Entf oder Rücktaste** leeren, **1** setzt eine Pause (`===`), **[** und **]** wählen die Hüllkurve, **,** und **.** die Oktave, **Leertaste** spielt ab.

*[Bild folgt in E13: play-tracker]*

### Hüllkurven bearbeiten

Links in der Liste **Instrumente** stehen die Hüllkurven 0 bis 9. **Neues Instrument** (bei PLAY höchstens zehn) legt eine weitere an, **Duplizieren** kopiert die gewählte. Für jede stellst du rechts die Klangform (Dreieck, Säge, Rechteck oder Rauschen), die Pulsbreite und die vier Werte **Anschlag** (Attack), **Abfall** (Decay), **Halten** (Sustain) und **Ausklang** (Release) ein, genau wie beim SID im Tracker-Kapitel. Ein Klick auf eine Taste der **Klaviatur** spielt den Ton mit der gewählten Hüllkurve vor.

> **Info:** Eine **Hüllkurve** beschreibt den Lautstärkeverlauf eines Tons: wie schnell er anschwillt (Attack), wie schnell er auf die Haltelautstärke fällt (Decay, Sustain) und wie lange er nach dem Ende ausklingt (Release).

## Exportieren

In der Werkzeugleiste steht **Exportieren …**; der Dialog bietet zwei Formate: **BASIC-Programm (PLAY)** und **BASIC DATA**. Mit **In Projekt speichern** schreibst du die Datei `build/export/<name>.bas`. Das Ergebnis für das Beispielstück sieht so aus:

```text
10 REM TUNE_PLAY - erzeugt von VintageCoder
20 VOL 8
30 TEMPO 8
40 ENVELOPE 0,0,9,8,4,2,2048
50 ENVELOPE 1,0,6,10,2,1,2048
60 ENVELOPE 2,0,5,0,3,3,2048
70 ENVELOPE 3,1,8,9,5,0,2048
80 PLAY "V1T0O4QCEGEO5CO4GEGDF#AF#O5DO4AF#A"
90 PLAY "V2T1O2QCCCCCCCCGGGGGGGG"
100 PLAY "V3T3O3HEEEEF#F#F#F#"
110 PLAY "V4T3O3HGGGGAAAA"
120 PLAY "V5T3O4HCCCCDDDD"
130 PLAY "V6QRT2O4HCCCCCCCQC"
```

So liest du die `PLAY`-Zeichenketten:

| Zeichen | Bedeutung |
|---|---|
| `V1` … `V6` | Stimme |
| `T0` … `T9` | Hüllkurve |
| `O4` | Oktave 4 |
| `W`, `H`, `Q`, `I`, `S` | Notenlänge: ganze, halbe, Viertel-, Achtel-, Sechzehntelnote |
| `.` | Punktierung (anderthalbfache Länge) |
| `R` | Pause |
| `C`, `E`, `F#` … | Noten |

Längen, die sich nicht als eine Note schreiben lassen (zum Beispiel fünf Schritte), gibt der Export als **wiederholten Anschlag** aus.

## Beispiel zum Nachmachen

Das Projekt `examples/mega65/play_demo` enthält `music/tune_play.vcplay`: dasselbe Stück wie das Tracker-Beispiel mit vier SIDs, umgesetzt auf sechs `PLAY`-Stimmen (Melodie, Bass, drei Arpeggio-Stimmen und Snare, 64 Schritte, vier Hüllkurven).

1. Öffne das Projekt und die Datei `music/tune_play.vcplay` (sie liegt in der Gruppe „PLAY-Tracker“ des Projektbaums).
2. Drücke die **Leertaste**, um sie anzuhören.
3. Klicke in der Werkzeugleiste auf **Exportieren …** und im Dialog auf **In Projekt speichern**. Es entsteht `build/export/tune_play.bas`.
4. Damit das Programm auch im Emulator läuft, trägst du es als BASIC-Quelle ins Projekt ein. In der Projektdatei `play_demo.vcprj` ersetzt du `"sources": []` durch:

```json
"sources": [
  { "targets": ["mega65"], "kind": "basic", "files": ["build/export/tune_play.bas"] }
],
"main": "build/export/tune_play.bas"
```

5. Baue mit **F7** und starte mit **F5**. Xemu startet und spielt das Stück.

> **Achtung:** Die BASIC-Datei muss **vor** dem Bauen existieren (Schritt 3); sonst meldet der Bau „Datei fehlt“. Nach jeder Änderung am Song exportierst du neu. Auf der Kommandozeile geht es mit `vcc media export music/tune_play.vcplay --format bas`.

## Hardware-Besonderheiten und Grenzen

- **Sechs Stimmen** (MEGA65), keine Muster und keine Reihenfolge.
- Höchstens **zehn Hüllkurven** (0 bis 9).
- Das Ergebnis ist BASIC-**Text**; Taktgenauigkeit und Klang bestimmt der BASIC-Interpreter des MEGA65.
- Das Vorhören ist eine Näherung; der erzeugte BASIC-Text wurde nicht auf echter Hardware geprüft.

## Vom Editor ins Programm

1. **Exportieren …** (Werkzeugleiste oder **Cmd/Strg+Umschalt+E**) erzeugt `build/export/<name>.bas`.
2. Trage die Datei als BASIC-Quelle ins Projekt ein (siehe das Beispiel oben) oder füge ihre Zeilen in ein eigenes BASIC-Programm ein.
3. Mit **F7** bauen, mit **F5** im Emulator **Xemu** starten.

## Nachbar-Editoren und Glossar

- Mehr Klangfarben und Maschinensprache bietet der [Tracker](kapitel:tracker).
- BASIC-Grundlagen: [BASIC in der IDE](kapitel:basic).
- Begriffe: Glossar (Datei `GLOSSAR.md`): PLAY, Hüllkurve, BASIC 65.

## Häufige Stolpersteine

- **Der Klang im Editor weicht vom MEGA65 ab:** Er ist eine Näherung. Der erzeugte BASIC-Text wurde nicht auf echter Hardware geprüft.
- **Eine Stimme bleibt stumm oder spielt falsch:** Prüfe, ob die Zelle wirklich eine Note enthält und ob die Hüllkurve (`T`) existiert.
- **Das Stück ist zu schnell oder zu langsam:** Ändere in der Werkzeugleiste das **Tempo**. Es wird als `TEMPO` ins Programm geschrieben.
- **Der Bau meldet „Datei fehlt“:** Der Export fehlt noch (siehe Achtung oben).

## Wie geht es weiter?

Sollen Musik und Programm in Maschinensprache zusammenlaufen, nimmst du den normalen [Tracker](kapitel:tracker). Das Einbinden von BASIC-Programmen erklärt [BASIC in der IDE](kapitel:basic). Nachschlagen: [PLAY-Tracker](hilfe:play), [Hüllkurve](hilfe:envelope), [MEGA65](hilfe:mega65).
