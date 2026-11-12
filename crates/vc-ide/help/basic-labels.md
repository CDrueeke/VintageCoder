# BASIC: Sprungmarken und Zeilennummern

Im klassischen BASIC springt man mit `GOTO 100` zu einer **Zeilennummer**. Das hat zwei Nachteile: Man muss jede Zeile nummerieren, und wer eine Zeile einfügt, muss Nummern ändern oder Platz gelassen haben. Hier benutzt du stattdessen **Sprungmarken** (englisch *labels*): Namen für Stellen im Programm. Beim Bauen vergibt der Übersetzer die Zeilennummern selbst und setzt sie an die Stelle der Marken ein. Das Ergebnis ist ein ganz normales nummeriertes BASIC-Programm. Die Seite gehört zum [BASIC-Handbuch](help:basic).

## Eine Marke festlegen: `@name:`

```
@name:
@name: anweisung
100 @name: anweisung
```

- Die Marke beginnt mit `@` und endet mit einem **Doppelpunkt**. Sie steht am **Anfang der Zeile**, bei einer Zeile mit Nummer direkt hinter der Nummer.
- Der Name besteht aus Buchstaben, Ziffern und `_` und beginnt nicht mit einer Ziffer. **Groß-/Kleinschreibung ist egal:** `@Ende` und `@ende` sind dieselbe Marke.
- Steht hinter der Marke nichts, **gilt sie für die nächste Anweisung** darunter. Das ist die übliche Schreibweise, weil sie die Marke gut sichtbar macht.
- Steht hinter der Marke eine Anweisung, gehört die Marke zu dieser Zeile.
- Mehrere Marken hintereinander (`@a: @b: print 1`) benennen dieselbe Zeile.
- Dieselbe Stelle darf auch eine eigene Nummer haben (`100 @start: print "x"`); dann hat die Marke diese Nummer.

Das Beispiel zeigt alle drei Schreibweisen. Sie ergeben dasselbe Programm:

```basic liste
@a:
print "x"
@b: print "y"
100 @c: print "z"
goto @a : goto @b : goto @c
```

```text
10 print "x"
20 print "y"
100 print "z"
110 goto 10 : goto 20 : goto 100
```

### Eine Marke benutzen: `@name`

Überall, wo im klassischen BASIC eine Zeilennummer als **Sprungziel** stünde, schreibst du `@name`:

| Anweisung | Beispiel |
|---|---|
| `GOTO` | `goto @start` |
| `GOSUB` | `gosub @zeichnen` |
| `THEN` und `ELSE` | `if x = 1 then @gewonnen` oder `if x = 1 then @ja else @nein` |
| `ON … GOTO` und `ON … GOSUB` | `on wahl goto @a, @b, @c` |
| `RESTORE` mit Zeile (C128, Plus/4, MEGA65, Atari; nicht C64 und VC-20) | `restore @daten` |
| `TRAP`, `RUN`, `LIST`, `RESUME` und alles andere mit Zeilennummer | `trap @fehler` |

Der Übersetzer ersetzt **jedes** `@name` im BASIC-Text durch die Zeilennummer, und zwar überall, wo es nicht in einer Zeichenkette (`"…"`), hinter `REM` oder in einer `DATA`-Zeile steht. Dort bleibt der Text unverändert, ein `@` darf also in deinen Texten vorkommen.

Beispiel mit mehreren Sprungarten:

```basic liste
input "zahl"; n
on n goto @eins, @zwei
if n > 2 then @gross else @klein
@eins: print "eins" : end
@zwei: print "zwei" : end
@gross: print "gross" : end
@klein: print "kleiner als eins" : end
```

```text
10 input "zahl"; n
20 on n goto 40, 50
30 if n > 2 then 60 else 70
40 print "eins" : end
50 print "zwei" : end
60 print "gross" : end
70 print "kleiner als eins" : end
```

In `DATA`, in Zeichenketten und hinter `REM` wird nichts ersetzt:

```basic liste
print "mail@host"
rem goto @nirgends
data @nein, 1
@wirklich:
goto @wirklich
```

```text
10 print "mail@host"
20 rem goto @nirgends
30 data @nein, 1
40 goto 40
```

### Zeilen, die eine Marke nicht haben dürfen

Eine Marke muss **am Anfang der Zeile** stehen. In der Mitte einer Zeile ist `@name:` keine Festlegung. Die Meldung ist dann irreführend, weil der Übersetzer das Wort als Sprungziel liest, das es nicht gibt:

```basic fehler
a = 1 : @mitte: print a
goto @mitte
```

```text
main.bas:1: error B110: Label @mitte ist nicht definiert
```

Schreibe die Marke in eine eigene Zeile oder an den Anfang der Zeile.

## Wie die Zeilennummern entstehen

Beim Bauen geht der Übersetzer die Datei von oben nach unten durch:

- Eine Zeile **ohne Nummer** bekommt die nächste Nummer. Es beginnt bei **10**, die Schrittweite ist **10** (10, 20, 30, …).
- Eine Zeile **mit eigener Nummer** behält sie. Danach geht es mit dieser Nummer plus der Schrittweite weiter. Du kannst nummerierte und unnummerierte Zeilen mischen.
- Leere Zeilen, Kommentarzeilen (`;` am Anfang), Zeilen mit nur einer Marke und die Zeilen `#include` und `#longnames` bekommen **keine** Nummer und kosten nichts. `REM`-Zeilen sind dagegen echte Zeilen und werden nummeriert.
- Die Nummern müssen **aufsteigen**. Eine eigene Nummer, die nicht größer ist als die vorige, meldet der Bau als Fehler **B100**.
- Die höchste Zeilennummer ist 63999 (Commodore) bzw. 32767 (Atari). Wird sie überschritten, meldet der Bau B100.

```basic liste
print "a"
print "b"
200 print "c"
print "d"
```

```text
10 print "a"
20 print "b"
200 print "c"
210 print "d"
```

Fehler bei falscher Reihenfolge:

```basic fehler
100 print "a"
50 print "b"
```

```text
main.bas:2: error B100: Zeilennummer 50 nicht größer als die vorige (100)
```

Auch eine Nummer, die sich mit einer automatisch vergebenen deckt, ist ein Fehler: Wenn nach `print "a"` (Nummer 10) die Zeile `10 print "b"` folgt, meldet der Bau B100.

### Schrittweite und Startnummer

Der Standard 10/10 gilt im Editor und beim Bauen im Projekt immer. Nur der Befehl `vcc basic` kennt die Schalter `--start` und `--step`:

```bash
vcc basic hallo.bas --start 100 --step 5 -o hallo.prg
```

```text
@a = Zeile 100
hallo.prg: 24 Bytes
```

Nutze eine eigene Nummer, um im Projekt einen anderen Start zu bekommen: `1000 print "start"` setzt die Zählung ab 1000 fort. Das ist auch nützlich, wenn eine Routine eine **feste Zeilennummer** haben muss, etwa weil Maschinencode oder ein anderes Programm sie kennt:

```basic liste
print "haupt"
gosub 9000
end
9000 print "unterprogramm"
return
```

```text
10 print "haupt"
20 gosub 9000
30 end
9000 print "unterprogramm"
9010 return
```

Wie diese Zeilennummer-Sprünge mit `@name` zusammenspielen: Ein klassisches `GOTO 9000` bleibt, wie es ist. Der Übersetzer ändert nur Marken, nie vorhandene Zahlen.

## Zuordnung Marke ↔ Zeilennummer sehen

Nach dem Bauen gibt es mehrere Stellen, an denen du nachsehen kannst, welche Zeile eine Marke bekommen hat:

- **Bauprotokoll:** Die Stufe „Zeilennummern und Labels“ nennt die Zahl der Zeilen und Labels.
- **`build/<name>.labels.txt`:** je Marke eine Zeile, z. B. `ende                     Zeile 70`.
- **`build/<name>.classic.txt`:** das fertige Programm als Text mit Zeilennummern, aus dem gebauten Programm zurückgelesen. So siehst du genau, was das Gerät bekommt.
- **Editor:** Bei Quelltexten ohne Zeilennummer zeigt der Seitenstreifen links neben jeder Zeile die Nummer, die sie beim Bauen bekommt. Zeilen mit eigener Nummer, Kommentare, Leerzeilen und reine Markenzeilen zeigen nichts.
- **Kommandozeile:** `vcc basic datei.bas` druckt `@name = Zeile n` für jede Marke.

Diese Zuordnung brauchst du vor allem bei **Fehlermeldungen des Geräts**. Meldet der C64 `?SYNTAX ERROR IN 40`, ist 40 die erzeugte Nummer; ein Blick in `.classic.txt` (oder in den Seitenstreifen) zeigt die zugehörige Quellzeile.

## Fehler und Hinweise

| Code | Meldung | Ursache |
|---|---|---|
| B110 | `Label @x ist nicht definiert` | Es gibt kein `@x:` am Zeilenanfang. Tippfehler im Namen? Die Marke steht mitten in einer Zeile? |
| B111 | `Label @x ist doppelt definiert` | Zwei Zeilen tragen dieselbe Marke (Groß-/Kleinschreibung zählt nicht; die Meldung nennt den Namen klein geschrieben) |
| B112 | `Label @x steht am Ende ohne Anweisung dahinter` | Hinter der letzten Marke kommt nichts mehr, die Marke hätte keine Zeile |
| B100 | `Zeilennummer N nicht größer als die vorige (M)` | Eigene Nummern sind nicht aufsteigend, oder sie kollidieren mit der automatischen Nummer |
| B100 | `Zeilennummer N größer als 63999` | Zu viele Zeilen oder zu große eigene Nummer (Atari: 32767) |

Beispiele:

```basic fehler
print "a"
goto @weg
```

```text
main.bas:2: error B110: Label @weg ist nicht definiert
```

```basic fehler
@x:
print 1
@X:
print 2
```

```text
main.bas:3: error B111: Label @x ist doppelt definiert
```

```basic fehler
print 1
@ende:
```

```text
main.bas:2: error B112: Label @ende steht am Ende ohne Anweisung dahinter
```

Der Übersetzer sammelt mehrere Meldungen und zeigt sie alle in der Fehlerliste.

## Fallen

- **Berechnete Sprünge:** `goto 100 + x * 10` oder `on x goto 100, 200` mit festen Zahlen funktionieren nur, wenn du die Zeilen selbst nummerierst. Mit automatischen Nummern kennst du die Zahlen nicht im Voraus. Nimm `on x goto @a, @b, @c`.
- **`RESTORE @name` auf C64 und VC-20:** Das BASIC V2 kennt kein `RESTORE` mit Zeilennummer. Der Übersetzer prüft das nicht; das fertige Programm bricht auf dem Gerät mit `?SYNTAX ERROR` ab. Lies dort die `DATA`-Zeilen der Reihe nach (siehe [Assets einbinden](help:basic-directives)).
- **Zeilen der Fehlerbehandlung:** `ER`, `EL`, `ERL`, `RESUME` und `TRAP` arbeiten mit den **erzeugten** Nummern. Das Programm funktioniert, die Zahlen in `EL` musst du über `.classic.txt` deuten.
- **Atari und Turbo BASIC XL:** `GOTO @name` geht für beide. Turbo BASIC XL hat zusätzlich eigene Namen für Sprungziele (`#name`, `GO#`), siehe [Dialekte](help:basic-dialects). Beides lässt sich mischen.
