# BASIC: Lange Variablennamen

Eine **Variable** ist ein benannter Speicherplatz. Auf dem C64, dem VC-20, dem C128, dem Plus/4 und dem MEGA65 unterscheidet das BASIC an einem Variablennamen nur die **ersten zwei Zeichen**. `punkte`, `punkt` und `pu` sind dort **dieselbe** Variable. Das ist eine der größten Fallen des klassischen BASIC. Mit der Zeile `#longnames` schreibst du lange, sprechende Namen, und der Bau macht daraus passende Kurznamen. Die Seite gehört zum [BASIC-Handbuch](help:basic).

## Das Problem ohne `#longnames`

Ohne Hilfe merkt man die Verwechslung erst, wenn das Programm falsch rechnet:

```basic liste
punkte = 10
punkt = 5
print punkte
```

```text
10 punkte = 10
20 punkt = 5
30 print punkte
```

Auf dem C64 sind `punkte` und `punkt` beide `PU`; die Ausgabe wäre 5, nicht 10. Der Übersetzer meldet das ohne `#longnames` nicht, weil es für ihn normales BASIC ist.

Eine zweite Falle sind **Schlüsselwörter in Namen**. Das BASIC erkennt Befehle auch mitten im Wort: In `total` steckt `TO`, in `score` steckt `OR`, in `form` steckt `FOR`. Das Gerät zerlegt den Namen beim Eintippen in Befehle und meldet `?SYNTAX ERROR`. Auch Namen wie `status`, `time` und `error` bereiten Ärger: `ST`, `TI` und `ER` sind System-Variablen.

## Lange Namen einschalten

Schreibe in eine eigene Zeile der Datei:

```
#longnames
```

Die Zeile darf überall stehen (üblich ist ganz oben), Groß-/Kleinschreibung ist egal. Sie gilt für das **ganze Programm**, auch für eingebundene Dateien (siehe [Direktiven](help:basic-directives)). Sie bekommt keine Zeilennummer und kostet nichts.

Danach darf jeder Name aus **Buchstaben, Ziffern und `_`** bestehen, muss mit einem Buchstaben beginnen und kann beliebig lang sein:

```basic ausgabe
#longnames
punkte = 10
punkt = 5
print punkte; punkt
```

```text
punkte                     →  PU         (Zahl)
punkt                      →  PN         (Zahl)
```

Der Bau hat `punkte` zu `PU` gemacht und `punkt` zu `PN`, getrennte Variablen mit Namen, die das Gerät unterscheidet. Das fertige Programm sieht so aus:

```basic liste ausgabe
#longnames
punkte = 10
punkt = 5
print punkte; punkt
```

```text
10 pu = 10
20 pn = 5
30 print pu; pn
```

Dass die Verwechslung ausgeschlossen ist, hat der Bau mit einer **Warnung** (W313) gemeldet; mehr dazu weiter unten.

### Wie der Kurzname entsteht

Der Bau vergibt die Kurznamen der Reihe nach, in der Reihenfolge, in der die langen Namen im Programm zuerst vorkommen:

1. Namen, die du schon kurz geschrieben hast (höchstens zwei Zeichen, ohne `_`), **bleiben unverändert** und sind belegt.
2. Ein langer Name bekommt zuerst seine **ersten zwei Zeichen** (ohne `_`). So entsteht dasselbe, was du im klassischen BASIC von Hand geschrieben hättest: `punkte` wird `PU`, `hero_x` wird `HE`.
3. Ist dieser Kurzname schon vergeben, nimmt der Bau einen anderen mit demselben ersten Buchstaben und einem späteren Buchstaben des Namens (`punkt` wird `PN`), dann irgendeinen mit demselben ersten Buchstaben, zuletzt irgendeinen freien.
4. **Nie** werden Kurznamen vergeben, die ein Schlüsselwort ergeben oder enthalten (`TO`, `OR`, `IF`, `ON`, `GO`, `FN` und alle, die sonst mit einem Befehl beginnen) oder die das System reserviert (`ST`, `TI`, `DS`, `ER`, `EL`). `total` wird deshalb nicht `TO`, sondern `TT`; `status` wird nicht `ST`, sondern `SA`.

```basic ausgabe
#longnames
score = 1 : total = 2 : form = 3 : status = 4 : tim = 5
print score; total; form; status; tim
```

```text
score                      →  SC         (Zahl)
total                      →  TT         (Zahl)
form                       →  FO         (Zahl)
status                     →  SA         (Zahl)
tim                        →  TM         (Zahl)
```

Das Ergebnis ist Byte für Byte das Programm, das du mit den Kurznamen von Hand geschrieben hättest. Es gibt keine Laufzeitkosten, keine Tabelle im Speicher des Geräts.

### Wörter sind immer ganze Namen

Mit `#longnames` ist ein Wort **entweder genau ein Schlüsselwort oder ein ganzer Name**. Das klassische BASIC erkennt dagegen Befehle auch dann, wenn sie an Namen oder Zahlen kleben. Das hat eine Folge: **Schlüsselwörter brauchen Trenner** (Leerzeichen oder Satzzeichen):

```basic liste
#longnames
for i = 1 to 3
next i
```

```text
10 for i = 1 to 3
20 next i
```

Geklebt, wie man es von alten Listings kennt (`FORI=1TO3`), wäre `fori` ein Name. Der Bau macht dann `fo = 1 to 3` daraus und es fehlt das `FOR`:

```basic liste
#longnames
fori = 1 to 3
```

```text
10 fo = 1 to 3
```

Schreibe also `for i = 1 to 3`, `goto 100`, `if a then`. Jedes Leerzeichen belegt im Programm ein Byte, der Übersetzer entfernt keine. Wer jedes Byte braucht, lässt `#longnames` weg und klebt von Hand.

## Typen: `name`, `name%`, `name$`, `name(`

Zahl, Ganzzahl, Text und Feld sind im BASIC verschiedene Variablen, auch wenn der Name gleich ist. Das bleibt so: Sie teilen den Kurznamen, bleiben aber getrennt.

```basic ausgabe
#longnames
dim feld(3)
zaehler% = 0
name$ = "hero"
name = 2
feld(1) = 5
print name$; name; zaehler%; feld(1)
```

```text
feld(…)                    →  FE(…)      (Zahl, Feld)
zaehler%                   →  ZA%        (Ganzzahl)
name$                      →  NA$        (Text)
name                       →  NA         (Zahl)
```

`name` und `name$` heißen auf dem Gerät `NA` und `NA$`: zwei Variablen. Ein Feld `feld(…)` ist wieder eine eigene Variable, also ist `feld` (ohne Klammer) eine andere als `feld(…)`.

## Funktionen: `def fn`

Namen von Funktionen (`def fn name(x) = …`) haben einen **eigenen Namensraum**. Ihre Kurznamen konkurrieren nicht mit denen der Variablen.

```basic liste
#longnames
def fn quad(wert) = wert * wert
print fn quad(7)
```

```text
10 def fn qu(we) = we * we
20 print fn qu(7)
```

Ein Variablenname, der mit `fn` beginnt (`fnx`), wird als Funktionsaufruf gelesen, wie es auch das BASIC selbst tut. Nenne Variablen nicht so.

## Die Zuordnung sehen

Wo der Bau Namen umsetzt, listet er die Zuordnung auf:

- im **Bauprotokoll** (Zeile „Lange Namen: n umgesetzt“ mit der Tabelle darunter),
- in der Datei **`build/<name>.vars.txt`**,
- bei `vcc basic` auf der Kommandozeile (Abschnitt `Lange Namen → Kurznamen:`),
- als `names` in der Zeilenkarte von [basic2asm](help:basic-tools),
- im **Editor**: Fährst du mit der Maus über einen langen Namen, zeigt ein Hinweis seinen Kurznamen.

Eine Zeile der Tabelle hat die Form `lang  →  KURZ  (Art)`; die Art ist `Zahl`, `Ganzzahl`, `Text`, `Funktion` oder `Zahl, Feld`. Namen, die du selbst kurz geschrieben hast, stehen nicht in der Tabelle. Auch wenn ein langer Name der Anfang eines kurzen ist, bleibt der kurze unverändert und der lange bekommt einen anderen:

```basic ausgabe
#longnames
pu = 1
punkte = 2
print pu; punkte
```

```text
punkte                     →  PN         (Zahl)
```

## Kollisionen und Warnungen

Unterscheiden sich zwei lange Namen **erst nach den ersten zwei Zeichen**, bekommen sie getrennte Kurznamen, und der Bau warnt mit **W313**. Die Warnung hilft dir, nicht auf ein BASIC ohne diese Umsetzung hereinzufallen:

```basic warnung
#longnames
punkte = 1
punkt = 2
print punkte; punkt
```

```text
main.bas:3: warning W313: 'punkte', 'punkt' unterscheiden sich erst nach den ersten 2 Zeichen ('PU'); sie werden hier getrennt (PU, PN). Ein BASIC ohne diese Umsetzung sähe nur eine Variable
```

Die Warnung ist ein Hinweis, kein Fehler. Das Programm baut und läuft richtig. Willst du sie loswerden, gib den Namen verschiedene Anfänge (`punktzahl`, `rest`).

Einen Fehler gibt es nur, wenn alle Kurznamen vergeben sind (**B131**, `Für den Namen 'x' ist kein freier Kurzname mehr übrig`). Das passiert erst bei Hunderten verschiedener Variablen, die das Gerät sowieso nicht mehr fasst.

Zeichen außer Buchstaben, Ziffern und `_` sind in Namen nicht erlaubt; ein Name beginnt nicht mit einer Ziffer.

## Was unberührt bleibt

Der Bau setzt Namen nur dort um, wo sie Namen sind. **Nicht verändert** werden Zeichenketten (`"total"`), `REM`-Zeilen, `DATA`-Zeilen, `{…}`-Steuerzeichen, Zahlen (auch `$ff` und `%101` auf Systemen, die sie kennen) und Marken (`@name`).

```basic liste
#longnames
total = 3
print "total:"; total
rem total ist die summe
data total, 5
```

```text
10 tt = 3
20 print "total:"; tt
30 rem total ist die summe
40 data total, 5
```

## Atari BASIC und Turbo BASIC XL

Atari BASIC kennt lange Namen schon selbst. Dort bleiben alle Namen **unverändert**. Nur zwei Dinge macht `#longnames` zusätzlich:

- `_` ist in Atari-Namen nicht erlaubt und **fällt weg**: `hero_x` wird `herox`.
- Kollidieren zwei Namen dadurch, hängt der Bau eine Zahl an: `hero_x` und `herox` ergeben `herox2` und `herox`.
- Namen, die so heißen wie ein Befehl oder eine Funktion, werden vermieden.

Ohne `#longnames` ist `_` im Atari-Namen ein Fehler:

```basic fehler system=atari8
hero_x = 1
? hero_x
```

```text
main.bas:1: error B100: '=' nach 'HERO' erwartet
main.bas:2: error B100: unerwartetes Zeichen '_'
```

Mit `#longnames` geht es:

```basic liste system=atari8
#longnames
hero_x = 1
herox = 2
? hero_x; herox
```

```text
10 HEROX2=1
20 HEROX=2
30 ? HEROX2;HEROX
```

Die Grenzen: höchstens 128 verschiedene Variablen in Atari BASIC, 256 in Turbo BASIC XL (Fehler `mehr als 128 Variablen`).

## Typische Fehler im Überblick

| Symptom | Ursache | Abhilfe |
|---|---|---|
| Zwei Variablen verhalten sich wie eine | Namen unterscheiden sich erst nach Zeichen 2, `#longnames` fehlt | `#longnames` an den Anfang schreiben |
| `?SYNTAX ERROR` bei einem harmlosen Namen | Der Name enthält ein Schlüsselwort (`total`, `score`) | `#longnames` einschalten |
| `FOR` oder `GOTO` fehlt im Ergebnis | Schlüsselwort klebt an Namen oder Zahl (`fori`, `goto100`) | Leerzeichen setzen |
| W313 beim Bau | Zwei lange Namen haben denselben Anfang | Hinweis lesen; nur umbenennen, wenn du es willst |
| B131 beim Bau | Keine freien Kurznamen mehr | Variablen sparen, Felder statt Einzelvariablen |
| Hover zeigt keinen Kurznamen | `#longnames` fehlt, oder der Name ist schon kurz | Zeile `#longnames` prüfen |

*Diese Funktion ist Birko vom N3rdroom gewidmet.*
