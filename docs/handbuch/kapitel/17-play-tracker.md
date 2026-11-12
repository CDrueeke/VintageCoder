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

## Wie unterscheidet sich der PLAY-Tracker?

Der MEGA65 hat in seinem BASIC-Dialekt (BASIC 65) den Befehl **`PLAY`**. Man übergibt ihm eine Zeichenkette wie `"V1T0O4QCEG"` („Stimme 1, Hüllkurve 0, Oktave 4, Viertelnoten C, E, G“), und der Rechner spielt sie, ohne dass du Interrupts programmierst. Der **PLAY-Tracker** erzeugt diese Zeichenketten aus einem Raster.

Die Unterschiede zum normalen Tracker:

- Es gibt **keine Muster und keine Reihenfolge**. Ein PLAY-Song ist ein einziges Raster aus **Schritten × Stimmen**.
- Ein **Schritt** ist ein **Sechzehntel**. Eine Note klingt, bis eine neue Note oder `===` (Pause) folgt.
- Der Song hat **6 Stimmen** und standardmäßig 32 Schritte.
- Statt Instrumenten gibt es **Hüllkurven 0 bis 9**. Sie entsprechen dem BASIC-Befehl `ENVELOPE n,a,d,s,r,wellenform,pw`.
- Das Ergebnis ist **Text** (ein BASIC-Programm), keine Maschinensprache.

## Schritt für Schritt: einen PLAY-Song anlegen

1. **+** im Projektbaum ▸ **PLAY …**, Namen eingeben, Profil **MEGA65 PLAY-Tracker (BASIC-Befehl PLAY, 6 Stimmen)**, **Anlegen**.
2. Der Editor sieht aus wie der Tracker: In der Mitte das Raster mit den Spalten **Stimme 1** bis **Stimme 6**, rechts die **Hüllkurven**, unten eine Klaviertastatur und der Export-Streifen.
3. Oben stellst du **Tempo** (1 bis 255) und **Lautstärke** (0 bis 15) ein. **▶ Abspielen** startet die Hörprobe, dasselbe macht die Leertaste.
4. Wähle eine Zelle und tippe Noten mit dem Computer-Keyboard wie beim Tracker: untere Reihe `Z S X D C V G B H N J M` für eine Oktave, obere Reihe `Q 2 W 3 E R 5 T 6 Y 7 U I` für die nächste. **F9** und **F10** ändern die Oktave.
5. Mit **Punkt**, **Entf** oder **Rücktaste** leerst du eine Zelle, mit **1** setzt du eine Pause (`===`).
6. Mit **[** und **]** wählst du die Hüllkurve für die nächste Note; du kannst sie auch rechts in der Liste anklicken.
7. Der Cursor springt nach jeder Eingabe eine Zeile weiter. Strg/Cmd+S speichert.

![PLAY-Tracker mit sechs Stimmen, Hüllkurven und Klaviertastatur](bild:play-tracker "Der PLAY-Tracker: sechs Stimmen als Spalten, ein Schritt (Sechzehntel) pro Zeile, rechts die Hüllkurven 0 bis 9")

### Hüllkurven bearbeiten

Rechts in der Liste **Hüllkurven** stehen die Einträge 0 bis 9. **+ Hüllkurve** legt eine weitere an (höchstens zehn). Für jede stellst du Wellenform (Dreieck, Sägezahn, Puls oder Rauschen), Pulsbreite und die vier Werte **A**ttack, **D**ecay, **S**ustain und **R**elease ein, genau wie beim SID im Tracker-Kapitel. **Hörprobe (C-4)** spielt sie einmal.

> **Info:** Eine **Hüllkurve** beschreibt den Lautstärkeverlauf eines Tons: wie schnell er anschwillt (Attack), wie schnell er auf die Haltelautstärke fällt (Decay, Sustain) und wie lange er nach dem Ende ausklingt (Release).

## Exportieren

Im Fuß steht als einziges Format **bas** („PLAY-Befehle als BASIC-Programm“). Ein Klick auf **Export …** schreibt die Datei `build/export/<name>.bas`. Das Ergebnis für das Beispielstück sieht so aus:

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
3. Klicke im Fuß auf **Export …**. Es entsteht `build/export/tune_play.bas`.
4. Damit das Programm auch im Emulator läuft, trägst du es als BASIC-Quelle ins Projekt ein. In der Projektdatei `play_demo.vcprj` ersetzt du `"sources": []` durch:

```json
"sources": [
  { "targets": ["mega65"], "kind": "basic", "files": ["build/export/tune_play.bas"] }
],
"main": "build/export/tune_play.bas"
```

5. Baue mit **F7** und starte mit **F5**. Xemu startet und spielt das Stück.

> **Achtung:** Die BASIC-Datei muss **vor** dem Bauen existieren (Schritt 3); sonst meldet der Bau „Datei fehlt“. Nach jeder Änderung am Song exportierst du neu. Auf der Kommandozeile geht es mit `vcc media export music/tune_play.vcplay --format bas`.

## Häufige Stolpersteine

- **Die Hörprobe klingt anders als auf dem MEGA65:** Sie ist eine Näherung. Der erzeugte BASIC-Text wurde nicht auf echter Hardware geprüft.
- **Eine Stimme bleibt stumm oder spielt falsch:** Prüfe, ob die Zelle wirklich eine Note enthält und ob die Hüllkurve (`T`) existiert.
- **Das Stück ist zu schnell oder zu langsam:** Ändere oben **Tempo**. Es wird als `TEMPO` ins Programm geschrieben.
- **Der Bau meldet „Datei fehlt“:** Der Export fehlt noch (siehe Achtung oben).

## Wie geht es weiter?

Sollen Musik und Programm in Maschinensprache zusammenlaufen, nimmst du den normalen [Tracker](kapitel:tracker). Das Einbinden von BASIC-Programmen erklärt [BASIC in der IDE](kapitel:basic). Nachschlagen: [PLAY-Tracker](hilfe:play), [Hüllkurve](hilfe:envelope), [MEGA65](hilfe:mega65).
