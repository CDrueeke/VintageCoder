# BASIC: Dialekte und Zielsysteme

Jedes System hat sein eigenes BASIC, und VintageCoder baut jedes im Originaldialekt. Das Zielsystem des Projekts bestimmt den **Dialekt**; pro Quellgruppe lässt er sich im Projekt-Editor ändern (`"dialect"` in der `.vcprj`). Die Seite zeigt, was jeder Dialekt kann, wie Text und Steuerzeichen geschrieben werden und welche Besonderheiten es gibt. Sie gehört zum [BASIC-Handbuch](help:basic).

## Überblick

| System | Dialekt (Kennung) | Programmstart | Ergebnis | Variablennamen |
|---|---|---|---|---|
| C64 | BASIC V2 (`cbm_v2`) | `$0801` | `.prg` | 2 Zeichen zählen |
| VC-20 | BASIC V2 (`cbm_v2`) | je nach Speicherausbau (`$1001`, `$0401` oder `$1201`) | `.prg` | 2 Zeichen zählen |
| C128 | BASIC 7.0 (`basic7`) | `$1C01` | `.prg` | 2 Zeichen zählen |
| Plus/4 | BASIC 3.5 (`basic35`) | `$1001` | `.prg` | 2 Zeichen zählen |
| MEGA65 | BASIC 65 (`basic65`) | `$2001` | `.prg` | 2 Zeichen zählen |
| Atari 8-Bit | Atari BASIC (`atari_basic`) | im SAVE-Format | `.bas` | alle Zeichen zählen |
| Atari 8-Bit | Turbo BASIC XL (`turbo_basic_xl`) | im SAVE-Format | `.bas` | alle Zeichen zählen |

- Die **Programmstart-Adresse** ist die Ladeadresse der `.prg`-Datei; sie steht in den ersten zwei Bytes.
- Beim VC-20 hängt sie vom **Speicherausbau** ab (Projekt-Einstellung, auf der Kommandozeile `--memory 8k`).
- Der Dialekt gilt für die **Quellgruppe**. Soll eine Datei ausnahmsweise einen anderen Dialekt haben (z. B. Turbo BASIC XL statt Atari BASIC), wählst du ihn bei dieser Quellgruppe.

Wählst du den falschen Dialekt, bleibt der Bau ohne Meldung: Ein Befehl, den der Dialekt nicht kennt, wird als Folge von Buchstaben, also wie Variablennamen, übersetzt. Das merkst du erst auf dem Gerät.

## Text, Zeichen und Steuerzeichen (Commodore)

Auf dem C64 und seinen Verwandten gilt in Zeichenketten:

- **Kleinbuchstaben** erscheinen als Großbuchstaben auf dem Bildschirm (so zeigt der Rechner im Normalzustand Großschrift), **Großbuchstaben** als Grafikzeichen. Schreibe Text also klein. Außerhalb von Zeichenketten ist die Schreibweise der Befehle und Namen gleichgültig.
- Ziffern, Satzzeichen und `@ [ ] ^ _` wie auf der Tastatur. Umlaute, `ß`, `\`, `{` und `}` (außer als Steuerzeichen) haben keine Entsprechung und ergeben den Fehler **B100** (`Zeichen 'ö' hat keine PETSCII-Entsprechung`).
- **Steuerzeichen** in geschweiften Klammern: `{clr}`, `{home}`, `{wht}` …
- `{name*n}` wiederholt ein Zeichen `n`-mal: `{down*3}` sind drei Cursor-nach-unten.
- `{$xx}` schreibt ein beliebiges Byte hexadezimal.

Die folgenden Zeilen ergeben dasselbe Programm:

```basic liste
print "{clr}hallo"
print "{$93}hallo"
print chr$(147);"hallo"
```

```text
10 print "{clr}hallo"
20 print "{clr}hallo"
30 print chr$(147);"hallo"
```

(Die dritte Zeile ist ein ganz anderes Programm, nur mit derselben Wirkung: `chr$(147)` wird erst beim Ausführen berechnet.)

### Namen der Steuerzeichen

| Name | Code | Name | Code | Name | Code |
|---|---|---|---|---|---|
| `{blk}` | `$90` | `{wht}` | `$05` | `{red}` | `$1C` |
| `{cyn}` | `$9F` | `{pur}` | `$9C` | `{grn}` | `$1E` |
| `{blu}` | `$1F` | `{yel}` | `$9E` | `{orng}` | `$81` |
| `{brn}` | `$95` | `{lred}` | `$96` | `{gry1}` | `$97` |
| `{gry2}` | `$98` | `{lgrn}` | `$99` | `{lblu}` | `$9A` |
| `{gry3}` | `$9B` | `{clr}` | `$93` | `{home}` | `$13` |
| `{down}` | `$11` | `{up}` | `$91` | `{rght}` | `$1D` |
| `{left}` | `$9D` | `{rvs on}` oder `{rvon}` | `$12` | `{rvs off}` oder `{rvof}` | `$92` |
| `{inst}` | `$94` | `{del}` | `$14` | `{return}` | `$0D` |
| `{shift return}` | `$8D` | `{lower}` | `$0E` | `{upper}` | `$8E` |
| `{stop}` | `$03` | `{pound}` | `$5C` | `{pi}` | `$FF` |
| `{f1}` bis `{f8}` | `$85 $89 $86 $8A $87 $8B $88 $8C` | | | | |

`{rvs on}` und `{rvon}`, `{rvs off}` und `{rvof}` sind zwei Schreibweisen für dasselbe Zeichen. Ein unbekannter Name ist ein Fehler:

```basic fehler
print "{foo}"
```

```text
main.bas:1: error B100: unbekannter Steuercode {foo}
```

Die Tabelle gilt für alle Commodore-Dialekte. Ob ein Gerät die Farbe oder Funktion hat, bestimmt das Gerät; der Plus/4 hat zum Beispiel eine eigene Farbtabelle und ordnet die Codes anders zu.

Auch fehlende schließende Klammern meldet der Bau:

```basic fehler
print "{clr"
```

```text
main.bas:1: error B100: '}' fehlt
```

## C64 und VC-20: BASIC V2

BASIC V2 ist das kleinste Commodore-BASIC. Befehle:

`END FOR NEXT DATA INPUT# INPUT DIM READ LET GOTO RUN IF RESTORE GOSUB RETURN REM STOP ON WAIT LOAD SAVE VERIFY DEF POKE PRINT# PRINT CONT LIST CLR CMD SYS OPEN CLOSE GET NEW`

Funktionen und Operatoren: `TAB( SPC( TO FN THEN NOT STEP AND OR SGN INT ABS USR FRE POS SQR RND LOG EXP COS SIN TAN ATN PEEK LEN STR$ VAL ASC CHR$ LEFT$ RIGHT$ MID$`, dazu `GO` (für `GO TO`). Variablen mit besonderer Bedeutung: `ST`, `TI`, `TI$`, `DS`, `DS$` (nicht zuweisbar).

Besonderheiten:

- **Keine Grafik- und Tonbefehle.** Der C64 bedient man mit `POKE` und `PEEK` auf die Chips (VIC-II, SID). Beispiel: Rahmen- und Hintergrundfarbe:

```basic liste
poke 53280, 0
poke 53281, 0
print "{clr}{wht}hallo"
```

```text
10 poke 53280, 0
20 poke 53281, 0
30 print "{clr}{wht}hallo"
```

- **`RESTORE` ohne Zeile.** Es setzt auf das erste `DATA` zurück. `RESTORE @name` (mit Zeile) kennt der Dialekt nicht (siehe [Assets](help:basic-directives)).
- **`DEF FN`**: Funktionen mit einem Parameter, Aufruf mit `fn name(x)`.
- **Zeilenlänge:** Tippt man auf dem Gerät, endet die Zeile nach 80 Zeichen. Geladene Programme dürfen länger sein (bis 250 Byte), aber das Gerät kann sie nicht mehr bearbeiten.
- Die **Auswahl des Schlüsselworts** richtet sich nach der Reihenfolge der Befehlstabelle des Geräts: Es gewinnt der erste Treffer. Darum wird `GOTO` nicht in `GO` und `TO` zerlegt.

Auf dem VC-20 gilt dasselbe BASIC. Die Speicheradressen (`POKE` auf den Chip VIC) sind anders:

```basic liste system=vic20
poke 36879, 8
print "{clr}hallo vc-20"
```

```text
10 poke 36879, 8
20 print "{clr}hallo vc-20"
```

## C128: BASIC 7.0

BASIC 7.0 erweitert V2 um Grafik, Ton, Sprites, Fehlerbehandlung, Schleifen und Plattenbefehle. Zu den Befehlen von V2 kommen unter anderem:

- **Grafik:** `GRAPHIC COLOR BOX CIRCLE DRAW PAINT CHAR LOCATE SCALE GSHAPE SSHAPE SCNCLR WINDOW WIDTH RGR RCLR RDOT`
- **Sprites:** `SPRITE MOVSPR SPRCOLOR SPRSAV SPRDEF COLLISION RSPPOS RSPRITE RSPCOLOR BUMP`
- **Ton:** `SOUND VOL PLAY TEMPO ENVELOPE FILTER`
- **Programmablauf:** `DO LOOP UNTIL WHILE EXIT` (`DO … LOOP`-Schleifen), `ELSE` (zu `IF … THEN`), `BEGIN … BEND` (Befehlsblöcke), `TRAP RESUME ERR$ TRON TROFF`
- **Speicher und Platte:** `BANK FETCH STASH SWAP DIRECTORY CATALOG DOPEN DCLOSE APPEND DSAVE DLOAD BSAVE BLOAD DVERIFY DCLEAR HEADER SCRATCH COLLECT COPY RENAME BACKUP CONCAT BOOT RECORD DELETE`
- **Weiteres:** `KEY MONITOR USING PUDEF HELP AUTO RENUMBER DEC HEX$ INSTR JOY POT PEN SLEEP FAST SLOW QUIT OFF XOR RWINDOW POINTER`

```basic liste system=c128
color 0, 1
color 4, 1
graphic 1, 1
circle 1, 160, 100, 60
box 1, 10, 10, 100, 60
sleep 3
graphic 0, 1
```

```text
10 color 0, 1
20 color 4, 1
30 graphic 1, 1
40 circle 1, 160, 100, 60
50 box 1, 10, 10, 100, 60
60 sleep 3
70 graphic 0, 1
```

- Der Übersetzer **prüft nicht die Parameter** (Syntax). Welche Zahlen ein Befehl braucht, steht im C128-Handbuch.
- `do … loop` mit `exit` und `else` brauchst du nicht mehr zu bauen wie auf dem C64:

```basic liste system=c128
n = 0
do
  n = n + 1
  if n >= 10 then exit
loop
print n
```

```text
10 n = 0
20 do
30 n = n + 1
40 if n >= 10 then exit
50 loop
60 print n
```

- Bei **Fehlerbehandlung** zählt die erzeugte Zeilennummer: `trap @fehler` setzt die Nummer der Marke ein, `resume next` und `err$(er)` arbeiten damit. Die Variablen `ER` und `EL` (Fehlernummer, Fehlerzeile) sind reserviert.

## Plus/4: BASIC 3.5

BASIC 3.5 liegt zwischen V2 und 7.0. Es hat die Grafik-, Ton-, Schleifen- und Fehlerbefehle, aber **keine Sprites**, keine Bank-Umschaltung und kein `PLAY`:

`RGR RCLR RLUM JOY RDOT DEC HEX$ ERR$ INSTR ELSE RESUME TRAP TRON TROFF SOUND VOL AUTO PUDEF GRAPHIC PAINT CHAR BOX CIRCLE GSHAPE SSHAPE DRAW LOCATE COLOR SCNCLR SCALE HELP DO LOOP EXIT DIRECTORY DSAVE DLOAD HEADER SCRATCH COLLECT COPY RENAME BACKUP DELETE RENUMBER KEY MONITOR USING UNTIL WHILE`

```basic liste system=plus4
graphic 1, 1
draw 1, 10, 10 to 100, 80
box 1, 20, 20, 90, 70
for t = 1 to 500 : next t
graphic 0, 1
```

```text
10 graphic 1, 1
20 draw 1, 10, 10 to 100, 80
30 box 1, 20, 20, 90, 70
40 for t = 1 to 500 : next t
50 graphic 0, 1
```

Der Plus/4 hat eine eigene Farbtabelle mit Helligkeitsstufen. `{wht}` & Co. benennen dieselben Codes wie auf dem C64; welche Farbe daraus wird, bestimmt das Gerät.

## MEGA65: BASIC 65

BASIC 65 ist der umfangreichste Dialekt. Es enthält fast alles aus BASIC 7.0 und dazu Befehle für DMA, Palette, Maus, Bildschirmmodi, Dateisystem und Sprachen. Neu gegenüber 7.0 (Auswahl):

`BORDER BACKGROUND FOREGROUND HIGHLIGHT SCREEN PALETTE PIXEL DOT LINE POLYGON ELLIPSE VIEWPORT GCOPY CUT PASTE DMA EDMA DMODE DPAT MOUSE RMOUSE CURSOR RCURSOR FONT VSYNC SPEED FIND CHANGE ERASE DIR MKDIR CHDIR MOUNT LOADIFF SAVEIFF WPEEK WPOKE MOD LOG10`

```basic liste system=mega65
border 0
background 0
foreground 1
print "{clr}hallo mega65"
vsync 100
```

```text
10 border 0
20 background 0
30 foreground 1
40 print "{clr}hallo mega65"
50 vsync 100
```

Besonderheit für den Übersetzer: Bei BASIC 65 gewinnt beim Schlüsselwort der **längste Treffer** (`FOREGROUND` vor `FOR`). Bei Namen mit `#longnames` spielt das keine Rolle (siehe [Lange Namen](help:basic-names)), ohne `#longnames` darfst du Variablen wie `foreground` nicht benutzen. Fremde BASIC-65-Programme, die du importierst, laufen unverändert.

## Atari BASIC

Atari BASIC unterscheidet sich stark von den Commodore-Dialekten: Es wird beim Eingeben geprüft, speichert Zahlen im BCD-Format und kennt lange Variablennamen. VintageCoder erzeugt die Datei im **SAVE-Format** (`.bas`), die das Atari-BASIC mit `LOAD` oder `RUN` laden kann.

Befehle: `REM DATA INPUT COLOR LIST ENTER LET IF FOR NEXT GOTO GO TO GOSUB TRAP BYE CONT COM CLOSE CLR DEG DIM END NEW OPEN LOAD SAVE STATUS NOTE POINT XIO ON POKE PRINT RAD READ RESTORE RETURN RUN STOP POP ? GET PUT GRAPHICS PLOT POSITION DOS DRAWTO SETCOLOR LOCATE SOUND LPRINT CSAVE CLOAD`

Funktionen: `STR$ CHR$ USR ASC VAL LEN ADR ATN COS PEEK SIN RND FRE EXP LOG CLOG SQR SGN ABS INT PADDLE STICK PTRIG STRIG`

```basic liste system=atari8
graphics 0
poke 710, 0 : poke 712, 0
@neu:
position 8, 10 : ? "atari basic mit labels"
goto @neu
```

```text
10 GRAPHICS 0
20 POKE 710,0:POKE 712,0
30 POSITION 8,10:? "atari basic mit labels"
40 GOTO 30
```

Unterschiede zum Commodore-Weg:

- **Kleine Schreibweise im Text bleibt klein.** Das Atari-BASIC kennt in Zeichenketten ASCII/ATASCII von Leerzeichen bis `~`. Zeichen darüber, wie Umlaute, ergeben einen Fehler (siehe unten).
- **Steuerzeichen** in `{…}`: `{clr}` (Bildschirm löschen, `$7D`), `{up}`, `{down}`, `{left}`, `{right}`, `{esc}`, `{tab}`, `{del}`, `{eol}`, `{bell}` und jedes Byte als `{$xx}`.
- **Der Übersetzer prüft die Syntax der Anweisungen** (anders als bei Commodore): Er liest jede Anweisung nach der Grammatik des Atari-BASIC und meldet Fehler wie `'=' nach 'X' erwartet`, `Ausdruck unvollständig` oder `unerwartetes Zeichen '_'` als **B100**. Auch das `'` als Kommentar gibt es dort nicht.
- Die **Zeilennummern** reichen bis 32767; eine Zeile darf höchstens 255 Byte lang werden.
- Namen können beliebig lang sein, aber höchstens 128 verschiedene Variablen. `_` ist in Namen nicht erlaubt (siehe [Lange Namen](help:basic-names)).
- **Marken** funktionieren wie überall: `goto @neu`, `gosub @x`, `trap @fehler`, `on a goto @a, @b`.
- Die ausführliche Tokenisierung wurde gegen das echte Atari-BASIC (im Emulator atari800) abgeglichen.

```basic fehler system=atari8
x =
```

```text
main.bas:1: error B100: Ausdruck unvollständig
```

Zeichen außerhalb des ATASCII-Bereichs melden sich ebenso:

```basic fehler system=atari8
? "grüß"
```

```text
main.bas:1: error B100: Zeichen 'ü' nicht im ATASCII-Bereich (nutze {$xx})
```

## Turbo BASIC XL (Atari)

Turbo BASIC XL ist ein schnelleres, erweitertes Atari-BASIC. Es wird als Dialekt der Quellgruppe gewählt (Projekt-Editor „BASIC-Dialekt“, in der `.vcprj` `"dialect": "turbo_basic_xl"`; auf der Kommandozeile `--dialect turbo_basic_xl`). Zusätzlich zum Atari BASIC kennt der Übersetzer:

- **Strukturen:** `PROC name … ENDPROC`, `EXEC name`, `WHILE … WEND`, `REPEAT … UNTIL`, `DO … LOOP`, `EXIT`, mehrzeiliges `IF … ELSE … ENDIF`.
- **Sprungmarken:** `GO# name` und `# name` (eigene Marke in Turbo BASIC XL); `ON x EXEC a, b` und `ON x GO# a, b`. Daneben funktioniert `@name` wie immer.
- **Befehle:** `DPOKE MOVE -MOVE BPUT BGET FILLTO CIRCLE PAINT FCOLOR PAUSE TEXT CLS DSOUND TRACE DUMP DEL RENUM BLOAD BRUN DIR LOCK UNLOCK RENAME DELETE TIME$= *F *B *L %PUT %GET`.
- **Funktionen und Operatoren:** `DIV MOD EXOR & !`, Hexzahlen `$FF`, `%0`–`%3`, `DPEEK INSTR UINSTR INKEY$ HEX$ DEC FRAC TRUNC RAND TIME TIME$ ERR ERL` und `RND` ohne Klammern.
- **Kommentare:** `'` und `.` am Zeilenanfang (beide ergeben eine `REM`-Zeile), außerdem `--`.
- **Variablen:** bis zu 256, die Schreibweise des ersten Vorkommens bleibt erhalten.
- **Abkürzungen** wie `G.`, `PR.`, `POK.`.

Nicht übernommen: Parameter bei `PROC`/`EXEC` und `$define`-Makros (das sind Erweiterungen eines fremden Vorverarbeiters, kein Teil von Turbo BASIC XL).

```basic liste system=atari8 dialect=turbo_basic_xl
graphics 0
exec titel
spalte = 0
while spalte < 4
  exec balken
  spalte = spalte + 1
wend
repeat
  position 4, 20 : ? "taste druecken";
until peek(764) <> 255
end

proc titel
  position 8, 2 : ? "turbo basic xl"
endproc

proc balken
  if spalte mod 2
    setcolor 2, spalte, 4
  else
    setcolor 2, spalte, 8
  endif
  pause 3
endproc
```

```text
10 GRAPHICS 0
20 EXEC titel
30 spalte=0
40 WHILE spalte<4
50 EXEC balken
60 spalte=spalte+1
70 WEND
80 REPEAT
90 POSITION 4,20:? "taste druecken";
100 UNTIL PEEK(764)<>255
110 END
120 PROC titel
130 POSITION 8,2:? "turbo basic xl"
140 ENDPROC
150 PROC balken
160 IF spalte MOD 2
170 SETCOLOR 2,spalte,4
180 ELSE
190 SETCOLOR 2,spalte,8
200 ENDIF
210 PAUSE 3
220 ENDPROC
```

Eigene Marken von Turbo BASIC XL (`#name`) und `@name` gehen nebeneinander:

```basic liste system=atari8 dialect=turbo_basic_xl
exec rahmen
go# ende
proc rahmen
  ? "rahmen"
endproc
# ende
? "fertig"
```

```text
10 EXEC rahmen
20 GO# ende
30 PROC rahmen
40 ? "rahmen"
50 ENDPROC
60 # ende
70 ? "fertig"
```

**Zum Starten** braucht man Turbo BASIC XL selbst (nicht im Lieferumfang). Die gebaute `.bas`-Datei kann man damit laden. Geprüft ist die Ausgabe Byte für Byte gegen einen unabhängigen Tokenizer, nicht gegen das Originalprogramm in einem Emulator. Zu lange Zeilen (über 255 Byte) lehnt der Bau ab.

## Gleiche Aufgabe auf jedem System

Rahmen und Hintergrund schwarz, Bildschirm löschen:

| System | Zeilen |
|---|---|
| C64 | `poke 53280,0 : poke 53281,0 : print "{clr}"` |
| VC-20 | `poke 36879,8 : print "{clr}"` (Hintergrund und Rahmen teilen sich ein Register) |
| C128 | `color 4,1 : color 0,1 : print "{clr}"` |
| Plus/4 | `color 4,1 : color 0,1 : print "{clr}"` |
| MEGA65 | `border 0 : background 0 : print "{clr}"` |
| Atari | `graphics 0 : poke 710,0 : poke 712,0` |

Da die Befehle abweichen, braucht ein Programm für mehrere Systeme je System eine eigene Datei oder einen eigenen Teil (siehe [Direktiven](help:basic-directives)).
