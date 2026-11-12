# BASIC: Werkzeuge, Konvertierung und Fehlersuche

Diese Seite zeigt, was der Editor für BASIC tut, wie du Programme tokenisierst und zurücklesen kannst, wie `basic2asm` ein BASIC-Programm in Assembler umwandelt und wie du Fehler findest. Sie gehört zum [BASIC-Handbuch](help:basic).

## Hilfe zu einem Wort: F1

Setze den Cursor auf ein BASIC-Wort (`poke`, `for`, `restore` …) und drücke **F1** (oder wähle im Rechtsklick-Menü „Hilfe zu »Wort«“). Es erscheint eine kurze Erklärung mit einem Beispiel. Die Texte stammen aus der Datenbank `data/help/basic_keywords.json`; sie umfasst die häufigsten Wörter von BASIC V2 und Atari BASIC. Zu Wörtern ohne Eintrag (viele Befehle von BASIC 7.0 oder 65) zeigt die Hilfe die Suche über alle Seiten.

Die Suche im Hilfefenster findet Befehle, Seiten und Fehlercodes: Tippe `B110` oder `longnames` und du landest auf der passenden Seite.

## Der Editor

- **Zeilennummern im Seitenstreifen:** In einer `.bas`- oder `.lst`-Datei steht links neben jeder Zeile **ohne eigene Nummer** die Nummer, die sie beim Bauen bekommt (10, 20, …). Zeilen mit eigener Nummer, Kommentare, Leerzeilen und reine Markenzeilen zeigen nichts. Siehe [Sprungmarken](help:basic-labels).
- **Hover:** Hältst du die Maus über eine Zahl, zeigt ein Hinweis ihren Wert in den üblichen Schreibweisen. Über einem **langen Variablennamen** zeigt er den Kurznamen, den der Bau daraus macht (siehe [Lange Namen](help:basic-names)).
- **Zeile beim Verlassen formatieren:** Eine bearbeitete Zeile wird beim Verlassen aufgeräumt: ein Leerzeichen hinter der Zeilennummer, Mehrfachleerzeichen außerhalb von Zeichenketten werden zu einem. Der Text hinter `REM` bleibt, wie er ist. Abschalten: **Fenster > Zeile beim Verlassen formatieren**.
- **Kommentar an/aus:** „Zeile(n) auskommentieren / Kommentar entfernen“ im Rechtsklick-Menü setzt `;` an den Zeilenanfang bzw. nimmt es weg.
- **`#include`-Zeilen:** Rechtsklick „Datei öffnen“ oder Doppelklick auf den Dateinamen öffnet die Datei bzw. das Asset.
- **Fehler:** Meldungen des Baus erscheinen in der Fehlerliste und als Markierung an der Zeile. Bei eingebundenen Dateien zeigt die Meldung auf die richtige Datei und Zeile (siehe [Direktiven](help:basic-directives)).

## Was beim Bauen entsteht

| Datei in `build/` | Inhalt |
|---|---|
| `<name>.prg` oder `<name>.bas` | das Programm (Commodore: PRG mit Ladeadresse, Atari: SAVE-Format) |
| `<name>.classic.txt` | das fertige Programm als Text mit Zeilennummern, aus dem Programm zurückgelesen |
| `<name>.labels.txt` | Marke ↔ Zeilennummer (nur, wenn es Marken gibt) |
| `<name>.vars.txt` | lange Namen ↔ Kurznamen (nur mit `#longnames`) |
| `export/<name>.data.bas` | Assets als DATA-Zeilen (nur, wenn ein `#include` sie verlangt) |

`<name>.classic.txt` ist das beste Werkzeug zum Prüfen: So sieht das Gerät dein Programm. Die Schlüsselwörter erscheinen dort klein geschrieben (Commodore) bzw. groß (Atari).

## Tokenisieren und Zurücklesen

Das Wort **Tokenisieren** bedeutet: Der Text eines BASIC-Programms wird in das Format übersetzt, in dem das Gerät es speichert. Aus `PRINT` wird ein einzelnes Byte (ein *Token*), aus Zahlen werden ihre Ziffern bzw. das Zahlenformat des Dialekts. **Detokenisieren** ist die Umkehrung.

```bash
vcc basic hallo.bas --target c64 -o hallo.prg
vcc basic --detokenize hallo.prg
vcc basic --tokenize liste.lst --target c64 -o liste.prg
```

- `vcc basic datei.bas` (auch `.lst`) baut ein Programm im nativen Dialekt des Ziels mit allem, was auf diesen Seiten steht (Marken, Direktiven, lange Namen). Es druckt `@name = Zeile n` für jede Marke und, mit `#longnames`, die Zuordnungstabelle.
- `--target c64|c128|plus4|mega65|vic20|atari8` wählt das System, `--dialect ID` den Dialekt (z. B. `turbo_basic_xl`), `--memory 8k` den VC-20-Speicherausbau, `--start` und `--step` Startnummer und Schrittweite.
- `--detokenize` liest ein Programm (PRG oder Atari-SAVE-Datei) und druckt das Listing. Für Atari-Dateien gibst du `--target atari8` an.
- `--tokenize` übersetzt ein **klassisch nummeriertes** Listing, ohne Marken und ohne Direktiven. Zeilen ohne Nummer sind dort Fehler (**B100**, `Zeile beginnt nicht mit einer Zeilennummer`).

Beim Zurücklesen erscheinen die Schlüsselwörter klein (Commodore) bzw. groß (Atari) und Steuerzeichen als `{clr}`. Mit `vcc import` wird so aus einem BASIC-Programm von einer Diskette oder aus einer `.prg`-Datei ein lesbarer `.bas`-Quelltext, den du weiterbearbeiten kannst.

## BASIC in Assembler umwandeln: basic2asm

`basic2asm` übersetzt ein BASIC-Programm in lesbaren **Assembler-Quelltext**. Das Programm läuft danach nicht mehr im BASIC-Interpreter, sondern als Maschinenprogramm: schneller, und du kannst jede Zeile ansehen, anpassen und im Debugger durchlaufen. Jede BASIC-Zeile wird ein Abschnitt mit dem Original als Kommentar und einem eigenen Label `L<Zeile>`.

In der IDE: Rechtsklick auf die `.bas`-Datei im Projektbaum, „Nach Assembler konvertieren …“. Der Dialog fragt nach Ausgabedatei (Vorgabe `<name>_asm.asm`), Zielsystem und einigen Schaltern. Auf der Kommandozeile:

```bash
vcc basic2asm programm.bas --target c64 -o programm.asm
vcc basic2asm programm.bas --check
```

```text
programm.asm: Stufe 1, 5 Zeilen, Module: rt_core, rt_arith, rt_io
programm.bas: in Ordnung (Stufe 1, 5 Zeilen)
```

| Schalter | Wirkung |
|---|---|
| `--target c64`, `c128`, `plus4`, `atari8` | Zielsystem (Standard c64). Fließkomma gibt es für alle vier Ziele; `atari8` kennt alle drei Stufen (siehe unten) |
| `-o datei.asm` | Ausgabe (Vorgabe: gleicher Name mit `.asm`) |
| `--map datei.json` | Zeilenkarte (Vorgabe: gleicher Name mit `.b2a.json`) |
| `--stage 1` bis `3` | höchste erlaubte Stufe; gewählt wird die kleinste, die das Programm braucht |
| `--float` | Fließkomma-Laufzeit erzwingen |
| `--int-div` | `/` als ganzzahlige Division übersetzen (sonst braucht es Stufe 2) |
| `--style compact\|inline` | Ausdrucksstil. `compact` (Standard): jede Operation ruft eine Laufzeitroutine auf. `inline`: einfache Ganzzahlausdrücke (Variable ± Konstante, Variable ± Variable, Konstante − Variable, `=` und `<>`, `X=X±c`) stehen als direkte 6502-Befehle da – länger im Quelltext, kürzer in der Laufzeit. Alles andere bleibt `compact` |
| `--runtime include\|embed` | `include` (Standard): die Laufzeit kommt per `#include` aus `data/sys/`. `embed`: die Quellen der Laufzeit (und der Systemdateien wie `os.asm`) werden in die Ergebnisdatei kopiert; sie lässt sich dann ohne Include-Ordner assemblieren und weitergeben |
| `--no-track-line` | keine Zeilennummer zur Laufzeit mitführen (kleiner, aber Fehlermeldungen ohne Zeile) |
| `--no-rem` | `REM`-Zeilen und `;`-Kommentarzeilen nicht übernehmen |
| `--check` | nur prüfen, keine Dateien schreiben |

Der Syntaxbaum lässt sich prüfen und ausgeben: `vcc basic ast datei.bas` zeigt ihn, `--json` schreibt ihn als JSON (Zeilen, Anweisungen, Ausdrücke; Zeichenketten als Bytelisten des Zielzeichensatzes), `--roundtrip` schreibt ihn als kanonischen Text zurück (Klammern nur, wo der Rang sie braucht), liest diesen Text noch einmal und meldet **E191**, wenn nicht derselbe Baum herauskommt. Anweisungen, die der Konverter nicht übersetzt (`OPEN`, `LOAD` …), lassen sich nicht zurückschreiben (**E190**). Beide Einstellungen (`inline`, `embed`) gibt es auch im Dialog „BASIC → Assembler“ und in der Projektdatei (`convert`).

### Atari (`--target atari8`)

- **Umfang:** Stufe 1 bis 3 – Ganzzahlen, Fließkomma und Zeichenketten, Schleifen, Unterprogramme, `DATA/READ`, `DIM`, `POKE/PEEK`, `INPUT`, `GET`, `PRINT` mit `;`, `,`, `TAB`. Das Ergebnis ist eine `.xex`-Datei (`.org $2000`, `.run`); Ein- und Ausgabe laufen über CIO (Editor `E:`), die Schicht dafür steht in `data/sys/atari8bit/b2a/rt_os.asm`.
- **Zahlen:** Braucht das Programm nur Ganzzahlen, rechnet es mit 16-Bit-Zellen (Stufe 1, Hinweis I110/I150). Sobald Brüche, Dezimalzahlen, `^`, `SQR`, `RND`, `LOG`, `EXP`, `SIN`, `COS`, `ATN` oder große Zahlen vorkommen, wählt der Konverter Stufe 2: Alle Zahlen sind dann **6-Byte-BCD** wie in Atari-BASIC (10 Stellen, Bereich 1E-98 bis 9,99999999E+97), gerechnet mit dem **Fließkommapaket des Betriebssystems** (`$D800`–`$DFFF`, ohne das BASIC-Modul). Rundungen, `0.1+0.2 = 0.3` und die Ausgabe (`PRINT` zeigt höchstens 9 Stellen, wenn das erste Ziffernpaar ab 10 liegt; Exponentenschreibweise wie `1E+10`) stimmen deshalb mit dem Gerät überein.
- **Unterschiede zu Atari-BASIC:** `PRINT` einer Zahl gibt wie Atari-BASIC keine Vorzeichenstelle aus; Vergleiche liefern 1, `AND`, `OR` und `NOT` sind logisch (1 oder 0) wie im Original. Fehlermeldungen heißen als Text wie bei CBM (`ILLEGAL QUANTITY`, `DIVISION BY ZERO`, `OVERFLOW`, `STRING LENGTH`, `DIMENSION` …), nicht als Nummer.
- **Genauigkeit:** `SQR` rechnet mit `LOG10`/`EXP10` des Pakets und verfeinert einmal nach Newton (Quadratzahlen werden genau). Ganzzahlige Potenzen bis 255 (`2^10`, `(-2)^3`, `2^-2`) rechnet der Konverter durch wiederholtes Multiplizieren, also genau; andere Potenzen laufen über `EXP10(b*LOG10(a))` und sind auf etwa 8 Stellen genau. Das ist die Genauigkeit des OS-Pakets, nicht die des Programms. `RND` liest `RANDOM` (POKEY); das Argument spielt keine Rolle. `SIN`, `COS` und `ATN` sind die Näherungen des BASIC-Moduls (mit `PLYEVL` des OS-Pakets und denselben Koeffiziententafeln nachgebaut), die Ziffern stimmen mit dem Original überein. `DEG` und `RAD` setzen wie dort das Byte `$FB` (6 = Grad); der Konverter schreibt sie zu `POKE 251,6` bzw. `POKE 251,0` um.
- **Nicht übersetzbar:** `TAN` gibt es in Atari-BASIC nicht (Fehler E334); `XIO`, `TRAP`, `POP`, `NOTE`/`POINT`, `PUT`, `PADDLE`/`PTRIG`, `OPEN`/`CLOSE`/`INPUT #` u. a. werden als „nicht übersetzbar“ gemeldet (E311), nicht still ersetzt. Lange Namen werden immer auf Kurznamen abgebildet (Zuordnung in der Zeilenkarte).

Marken und `#longnames` funktionieren auch hier: Die Umwandlung beginnt wie der Bau mit der Nummerierung und der Auflösung der Marken. Die Zeilenkarte enthält zu jeder BASIC-Zeile die Zeile im Quelltext und das Label im Assembler, dazu `names` mit der Zuordnung der langen Namen.

- **Grafik, Ton und Joystick:** `GRAPHICS n` (auch `+16` und `+32`), `COLOR`, `PLOT`, `DRAWTO`, `POSITION`, `LOCATE x,y,v`, `SETCOLOR`, `SOUND`, `STICK(n)`, `STRIG(n)` und `PRINT #6;…` laufen wie im Original über den Betriebssystem-Kanal 6 (`S:`) und die Hardwareregister (Laufzeit `data/sys/atari8bit/b2a/rt_gfx.asm`). Die Anweisungen sind für den Konverter eigene Schlüsselwörter, so dass `COLOR` (enthält `OR`) oder `POSITION` (beginnt mit `POS`) keine Variablen sind. `PRINT #n` schreibt über Kanal n; geöffnet wird nur Kanal 6 (durch `GRAPHICS`). Bildschirmspeicher, Farbschatten und Cursor stimmen im Vergleich mit dem Original überein (Modus 0, 1, 2, 3, 5 und 8); `SOUND` ist im Modell geprüft (Registerwerte), nicht gegen das Gerät.
- **Turbo BASIC XL (Endung `.tur` wie auf dem Atari, `--turbo`, Schalter im Dialog und `turbo` in der Projektdatei):** Der Konverter liest zusätzlich die Konstrukte von Turbo BASIC XL und übersetzt sie in einfache Sprünge (Quelle ohne Zeilennummern wie `examples/atari/turbo_basic/balken.tur`): `IF` ohne `THEN` mit `ELSE` und `ENDIF`, `WHILE`/`WEND`, `REPEAT`/`UNTIL`, `DO`/`LOOP` mit `EXIT`, `PROC name`/`ENDPROC`/`EXEC name` (Parameter gibt es auch im Original nicht; läuft der Ablauf in eine Prozedur hinein, meldet Turbo BASIC XL Fehler 27, das übersetzte Programm überspringt sie einfach – Prozeduren gehören hinter `END`), die Operatoren `DIV` und `MOD`, `DPOKE`, `DPEEK`, `PAUSE n` (n Bildwechsel), `MOVE von,nach,n`, `-MOVE von,nach,n` (absteigend kopieren), `CLS`, `HEX$(n)` (zwei Stellen unter 256, sonst vier), `DEC(a$)` (Hexzahl am Anfang, 16 Bit mit Überlauf wie im Original), `INSTR(a$,b$[,n])` (n = Zahl der zu überspringenden Zeichen), `RAND(n)` (= `INT(RND(0)*n)`), `INKEY$` (ohne Warten), `TIME$` (`hhmmss` aus der Uhr), `ON n EXEC a,b`, `FCOLOR n` und `FILLTO x,y` (Füllen wie XIO 18, nach `PLOT`), `BPUT #n,adresse,länge` und `BGET #n,adresse,länge` (Block über CIO). Blöcke dürfen sich verschachteln und über Zeilen gehen; ein nicht abgeschlossener oder falsch geschlossener Block meldet **E341**/**E342**, `EXEC` einer fehlenden Prozedur **E343**, eine doppelte Prozedur **E340**. Ohne `--turbo` sind `WHILE`, `PROC` usw. gewöhnliche Namen wie in Atari-BASIC. Nicht übersetzt: `TRACE`, `DUMP`, `RENUM` und andere Befehle der Programmierumgebung. Kanäle (für Atari-Programme, mit und ohne `--turbo`): `OPEN #n,aux1,aux2,"name"`, `CLOSE #n`, `PUT #n,byte`, `GET #n,v` und `PRINT #n` laufen über CIO. Geprüft sind die Geräte E: und S: (Kanal 6 nach `GRAPHICS`) sowie Dateien auf `D:` (Schreiben und Lesen mit `PUT`, `GET #`, `PRINT #`, `BPUT`, `BGET`, im Vergleich mit dem echten Turbo BASIC XL). `D:` gibt es nur, wenn das Programm von einer DOS-Diskette läuft (als `AUTORUN.SYS`); ein Programm mit `OPEN` bindet immer die Zeichenkettenlaufzeit ein. Ein Fehler des Betriebssystems meldet sich als `?DEVICE ERROR <Nummer> IN <Zeile>` mit der Atari-Fehlernummer (170 = Datei nicht gefunden, 136 = Dateiende) und beendet das Programm; `TRAP` gibt es nicht. Geprüft ist alles gegen das echte Turbo BASIC XL (Diskettenabbild `assets/disks/atari8/Programmiersprachen/Turbo_Basic_XL.atr`, nicht in Git; die Tests tippen die Programme in atari800 ein): Blockanweisungen, `DIV`/`MOD` auch mit negativen und gebrochenen Zahlen, Zahlenausgabe (Turbo BASIC XL druckt bis zu zehn Stellen, Atari-BASIC neun) und die Wörter oben. `RAND(n)` für n ≤ 0 liefert im Original n, hier `INT(RND(0)*n)`; negative Auslassungszahl bei `INSTR` ist nicht geprüft.
- **Zeichenketten (Stufe 3):** nach den Regeln von Atari-BASIC, am echten BASIC-Modul gemessen. `DIM A$(n)` mit einer Zahl legt einen festen Puffer an (ohne `DIM` meldet der Konverter E321, zweimal `DIM` ist zur Laufzeit Fehler 9); Zuweisungen schneiden still auf die Größe ab. Teilzeichenketten `A$(n)` und `A$(n,m)` zählen ab 1, auch als Ziel (`A$(3,4)="XY"` ersetzt höchstens zwei Zeichen, `A$(LEN(A$)+1)="Z"` hängt an); Lesen außerhalb von 1…LEN meldet `STRING LENGTH`. Es gibt `LEN`, `ASC`, `VAL`, `STR$`, `CHR$`, Vergleiche (wahr = 1, nach Bytewert), `INPUT A$` (ganze Zeile) und `READ A$`. `DATA` ist roher Text bis zum Komma, mit Anführungszeichen und Leerzeichen, und reicht bis zum Zeilenende (auch über `:`); jeder Eintrag wird erst beim Lesen zur Zahl (sonst Fehler 8) oder zur Zeichenkette. Das Programm rechnet dann immer mit Fließkomma (`VAL` und `STR$` brauchen das OS-Paket). Nicht übersetzt (und im Original auch nicht vorhanden): `+` für Zeichenketten, `LEFT$`, `RIGHT$`, `MID$` (E311), `GET` in eine Zeichenkette. Texte in Anführungszeichen, `DATA` und `REM` bleiben ASCII (Zeichen wie `|` oder `_` gehen); die Größe von `DIM` muss eine Zahl sein.
- **Parser-Regeln von Atari-BASIC:** Der Konverter liest Atari-Quelltext mit eigenen Regeln: unäres `-`, `+` und `NOT` binden stärker als `^` (`-2^2` ist 4, `NOT 0=1` ist `(NOT 0)=1`), `DATA` reicht bis zum Zeilenende, Texte bleiben ASCII.
- **Abgleich mit dem Original:** `crates/vc-ide/tests/atari_basic_oracle.rs` lässt dieselben Programme im echten Atari-BASIC (`ATARIBAS.ROM` in atari800) und als übersetztes Programm laufen und vergleicht den Bildschirm: Zahlen, Funktionen, `^`, Operatorrang, Schleifen, Felder, Zeichenketten, Logik, Komma-Tabulator, Joystick und Grafikspeicher stimmen überein (nur mit `VC_EMU_TESTS=1`).

### Die drei Stufen

Der Konverter wählt die **Stufe** selbst:

1. **Ganzzahl.** Alle Zahlen sind 16-Bit-Ganzzahlen. Variablen wie `A` werden dann zu 16-Bit-Zellen. Der Konverter weist darauf hin (**I110**: `Variable A ist im Original eine Fließkommazahl; Stufe 1 rechnet mit 16-Bit-Ganzzahlen`). Das ist ein Hinweis, kein Fehler.
2. **Fließkomma** (C64, C128, Plus/4, Atari) über die Rechenroutinen des BASIC-ROMs bzw. (Atari) des OS-Fließkommapakets.
3. **Zeichenketten.**

**MEGA65 (`--target mega65`, Dialekt `basic65`, Programm bei `$2001`):** nur Stufe 1 und 3 (Ganzzahl und Zeichenketten). Fließkomma meldet **E331**, denn das ROM des MEGA65 hat keine verwendbaren Fließkommaroutinen. Das Ergebnis bleibt 6502-kompatibel (lange Sprünge als umgekehrter Sprung plus `JMP`, `.branchlong jmp`). Im Modell läuft es mit denselben Programmen wie C64, C128 und Plus/4. **In Xemu unter Windows geprüft** (Live-Tests in `crates/vc-ide/tests/b2a_mega65_live.rs`): KERNAL und Interrupt lassen den Nullseitenblock `$40`–`$53` unberührt (`data/sys/mega65/b2a/rt_zp.asm`), Ganzzahl- und Zeichenkettenprogramme laufen und schreiben den erwarteten Text. **Am echten MEGA65-Gerät noch nicht geprüft.**

```basic nocheck
a = 1
print a
b = a + 2
print b
@x:
goto @x
```

Dieses Programm ergibt:

```text
warning[I110]: Variable A ist im Original eine Fließkommazahl; Stufe 1 rechnet mit 16-Bit-Ganzzahlen
```

### Was der Konverter (noch) nicht übersetzt

Die Fehler beginnen mit **E**. Die wichtigsten:

| Code | Bedeutung |
|---|---|
| E100, E101, E102, E103 | Zeile oder Zeichen nicht lesbar, Zeilennummer doppelt, Anweisung nicht lesbar (`Zeile 10: '=' erwartet`) |
| E303 | Zahl zu groß für 16 Bit |
| E304 | nur eindimensionale Felder |
| E306 | Funktion wird nicht übersetzt |
| E311 | Befehl wird nicht übersetzt (Dateien, `LIST`, `NEW`, `CONT`, `DEF FN` in Stufe 1) |
| E320 | Typen passen nicht (`TYPE MISMATCH`: Zeichenkette und Zahl vermischt) |
| E330, E331 | Ziel oder Fließkomma für das Ziel nicht umgesetzt |

Befehle, die ein Dialekt zusätzlich hat (Grafik, Sprites), übersetzt der Konverter nicht. Er meldet dann E103 oder E311. MEGA65 ist noch kein Ziel.

```basic fehler nocheck
sprite 1, 1
```

```text
error[E103]: Zeile 10: '=' erwartet
```

### Debuggen mit der Zeilenkarte

Ist das Ergebnis als Assembler-Quelle in die Projektdatei eingetragen (Schalter im Dialog „Ergebnis als Assembler-Quelle in die Projektdatei eintragen“), liest die IDE im Debugger die Zeilenkarte: Haltepunkte in der `.bas`-Datei gelten für den Code der BASIC-Zeile, und die Ausführungsmarke und die Statuszeile zeigen die BASIC-Zeile an (`angehalten bei $… – BASIC-Zeile 30`). Siehe [Debugger](help:debugger).

## Fehler finden

1. **Meldungen des Baus lesen.** Sie beginnen mit `main.bas:ZEILE:` und einem Code. Die Tabellen der Seiten [Sprungmarken](help:basic-labels), [Namen](help:basic-names), [Direktiven](help:basic-directives) und der [Spickzettel](help:basic-index) erklären jeden Code.
2. **Das Programm auf dem Gerät testen.** Der Bau prüft keine Syntax (außer beim Atari). Meldet das Gerät `?SYNTAX ERROR IN 40`, ist 40 die **erzeugte** Zeilennummer. In `build/<name>.classic.txt` steht Zeile 40 so, wie das Gerät sie sieht; im Seitenstreifen des Editors steht dieselbe Nummer neben der Quellzeile.
3. **Den richtigen Dialekt prüfen.** Ein Befehl, den der Dialekt nicht kennt, wird zu Buchstaben und löst auf dem Gerät `?SYNTAX ERROR` aus. Die Liste in `.classic.txt` zeigt es nicht. Prüfe daher die Befehlsliste auf der Seite [Dialekte](help:basic-dialects).
4. **Verdächtige Namen.** Seltsame Fehler bei Namen wie `total` oder `score` kommen von Schlüsselwörtern im Namen. Schalte `#longnames` ein.
5. **Verfolgen, was passiert.** Auf C128, Plus/4 und MEGA65 schaltet `tron` die Zeilen-Verfolgung ein (`troff` aus); `trap @marke` fängt Fehler ab, `print er, el, err$(er)` meldet sie. Auf dem C64 gibt es nur `print`-Ausgaben und `stop`/`cont`. Beim Atari fängt `trap @marke` Fehler, `? err` meldet den Code.
6. **Ein Assembler-Teil läuft nicht?** Siehe [Debugger](help:debugger) und die Seite [Assembler](help:assembler).
