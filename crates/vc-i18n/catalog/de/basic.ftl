# SPDX-License-Identifier: GPL-3.0-or-later
# Kontext: Dialektdatei gehört nicht zur Atari-Familie
# $path: Pfad der Datei | Beispiel: data/basic/cbm_v2.dialect.json
basic-dialect-not-atari = { $path }: kein Atari-Dialekt

# Kontext: Feld fehlt in der Dialektdatei
# $key: Name des Feldes | Beispiel: tokens
basic-dialect-key-missing = { $key } fehlt

# Kontext: Eintrag ohne Namen in der Dialektdatei
basic-dialect-name-missing = Name fehlt

# Kontext: Eintrag mit ungültigem Code in der Dialektdatei
basic-dialect-code-invalid = Code ungültig

# Kontext: Feld ops fehlt in der Dialektdatei
basic-dialect-ops-missing = ops fehlt

# Kontext: Pflicht-Schlüsselwort fehlt in der Dialektdatei
# $path: Pfad der Datei | Beispiel: data/basic/x.dialect.json
# $need: das Schlüsselwort | Beispiel: REM
basic-dialect-keyword-required = { $path }: Pflicht-Keyword { $need } fehlt

# Kontext: Datei zu kurz für ein BASIC-Programm
basic-prg-too-short = Datei zu kurz

# Kontext: Programmende fehlt im PRG
basic-prg-end-missing = Programmende ($0000) fehlt

# Kontext: Zeile im PRG abgeschnitten
basic-prg-line-truncated = abgeschnittene Zeile

# Kontext: Zeilenende fehlt im PRG
basic-prg-line-end-missing = Zeilenende (0) fehlt

# Kontext: Link-Zeiger einer PRG-Zeile stimmt nicht; next und expect sind Hexadressen ohne Zeichen $
# $num: Zeilennummer | Beispiel: 10
# $next: Zeiger in der Datei (Hex) | Beispiel: 080D
# $expect: erwarteter Zeiger (Hex) | Beispiel: 0810
basic-prg-link-mismatch = Link-Zeiger in Zeile { $num } stimmt nicht (${ $next }, erwartet ${ $expect })

# Kontext: Kopfzeile des strukturierten Textes aus dem Detokenizer (BASIC-Kommentar)
basic-detok-structured-header = ' Aus PRG zurückgewandelt (vcc basic --detokenize --structured)

# Kontext: SAVE-Datei: unerwarteter VNTP-Offset; offset ist Hex
# $offset: Offset (Hex) | Beispiel: 0100
basic-atari-vntp-offset = unerwarteter VNTP-Offset ${ $offset }

# Kontext: SAVE-Datei: Länge stimmt nicht mit dem Kopf überein
# $head: Länge laut Kopf | Beispiel: 120
# $file: Länge der Datei | Beispiel: 100
basic-atari-length-mismatch = Länge stimmt nicht (Kopf: { $head }, Datei: { $file })

# Kontext: SAVE-Datei: Zeiger im Dateikopf ungültig
basic-atari-pointers-invalid = Zeiger im Dateikopf ungültig

# Kontext: SAVE-Datei: Variablenwertetabelle passt nicht zur Namenstabelle
basic-atari-vvt-mismatch = Variablenwertetabelle passt nicht zur Namenstabelle

# Kontext: SAVE-Datei: ungültige Zeilenlänge; offset ist Hex
# $offset: Offset (Hex) | Beispiel: 0042
basic-atari-line-length = ungültige Zeilenlänge bei ${ $offset }

# Kontext: Rechner: unbekanntes Wort
# $w: das Wort | Beispiel: XNOR
basic-calc-unknown-word = unbekanntes Wort { $w }

# Kontext: Rechner: ungültige Zahl
# $s: die Zahl | Beispiel: $GG
basic-calc-bad-number = ungültige Zahl { $s }

# Kontext: Rechner: unerwartetes Zeichen
# $c: das Zeichen | Beispiel: #
basic-calc-unexpected-char = unerwartetes Zeichen '{ $c }'

# Kontext: Rechner: Klammer zu fehlt
basic-calc-rparen-missing = ')' fehlt

# Kontext: Rechner: Zahl erwartet
basic-calc-number-expected = Zahl erwartet

# Kontext: Rechner: Überlauf
basic-calc-overflow = Überlauf

# Kontext: Rechner: Division durch 0
basic-calc-div-zero = Division durch 0

# Kontext: Rechner: Schiebeweite außerhalb 0–63
basic-calc-shift-range = Schiebeweite 0–63

# Kontext: Rechner: leerer Ausdruck
basic-calc-empty = leer

# Kontext: Rechner: Rest nach dem Ausdruck
basic-calc-trailing = Rest nach dem Ausdruck

# Kontext: Teil der Meldung E301/E302 (was Stufe 2 oder 3 braucht)
# $s: die Zahl | Beispiel: 1.5
basic-b2a-what-number = Zahl { $s }

# Kontext: Teil der Meldung E301/E302 (was Stufe 2 oder 3 braucht)
basic-b2a-what-string-expr = Zeichenkette in einem Ausdruck

# Kontext: Teil der Meldung E301/E302 (was Stufe 2 oder 3 braucht)
# $name: Name | Beispiel: A
basic-b2a-what-string-var-named = Zeichenkettenvariable { $name }$

# Kontext: Teil der Meldung E301/E302 (was Stufe 2 oder 3 braucht)
basic-b2a-what-string-array = Zeichenkettenfeld

# Kontext: Teil der Meldung E301/E302 (was Stufe 2 oder 3 braucht)
basic-b2a-what-division = Division '/' (liefert Bruchzahlen; mit --int-div ganzzahlig)

# Kontext: Teil der Meldung E301/E302 (was Stufe 2 oder 3 braucht)
basic-b2a-what-power = Potenz '^'

# Kontext: Teil der Meldung E301/E302 (was Stufe 2 oder 3 braucht)
# $name: Name der Funktion | Beispiel: SQR
basic-b2a-what-name = { $name }

# Kontext: Teil der Meldung E301/E302 (was Stufe 2 oder 3 braucht)
basic-b2a-what-string-var = Zeichenkettenvariable

# Kontext: Teil der Meldung E301/E302 (was Stufe 2 oder 3 braucht)
basic-b2a-what-string-var-hyphen = Zeichenketten-Variable

# Kontext: Teil der Meldung E301/E302 (was Stufe 2 oder 3 braucht)
basic-b2a-what-string-assign = Zeichenkettenzuweisung

# Kontext: Teil der Meldung E301/E302 (was Stufe 2 oder 3 braucht)
basic-b2a-what-for-string = FOR mit Zeichenkettenvariable

# Kontext: Teil der Meldung E301/E302 (was Stufe 2 oder 3 braucht)
basic-b2a-what-data-text = DATA mit Text

# Kontext: Teil der Meldung E301/E302 (was Stufe 2 oder 3 braucht)
# $s: die Zahl | Beispiel: 1.5
basic-b2a-what-data-number = DATA-Zahl { $s }

# Kontext: Kommentar im erzeugten Assembler
basic-b2a-asm-facexp = Exponent des FAC: 0 = Zahl ist 0

# Kontext: Kommentar im erzeugten Assembler vor jeder BASIC-Zeile; labels = Marken in Klammern oder leer
# $n: Zeilennummer | Beispiel: 10
# $labels: Marken | Beispiel: (@start)
# $text: Text der Zeile | Beispiel: PRINT 1
basic-b2a-asm-line = Zeile { $n }{ $labels }: { $text }

# Kontext: zweite Kommentarzeile des erzeugten Assemblers
# $t: Ziel | Beispiel: c64
# $stage: Stufe | Beispiel: 1
# $kind: Art der Stufe (Text) | Beispiel: Ganzzahl
basic-b2a-asm-target = Ziel: { $t }, Stufe { $stage } ({ $kind }), Stil compact. Jede BASIC-Zeile steht als Kommentar vor ihrem Block (Label L<Zeile>).

# Kontext: Art der Stufe
basic-b2a-asm-kind-str-float = Zeichenketten und Fließkomma

# Kontext: Art der Stufe
basic-b2a-asm-kind-str-int = Zeichenketten und Ganzzahl

# Kontext: Art der Stufe
basic-b2a-asm-kind-float-os = Fließkomma über das OS-Paket

# Kontext: Art der Stufe
basic-b2a-asm-kind-float-rom = Fließkomma über das BASIC-ROM

# Kontext: Art der Stufe
basic-b2a-asm-kind-int = Ganzzahl

# Kontext: Kommentar im erzeugten Assembler
basic-b2a-asm-degflg = DEGFLG: Bogenmaß (DEG/RAD setzen ihn über POKE 251)

# Kontext: Kommentar im erzeugten Assembler
basic-b2a-asm-end-program = Programmende

# Kontext: Kommentar im erzeugten Assembler
basic-b2a-asm-clr = CLR: alle Variablen auf 0, Stapel leeren

# Kontext: Kommentar im erzeugten Assembler
# $shown: Name der Variablen | Beispiel: A$
# $cap: Kapazität | Beispiel: 40
basic-b2a-asm-str-cap = { $shown } (Länge, Kapazität { $cap })

# Kontext: Kommentar im erzeugten Assembler
# $shown: Name der Variablen | Beispiel: A$
basic-b2a-asm-str-addr = { $shown } (Länge, Adresse)

# Kontext: Kommentar im erzeugten Assembler
basic-b2a-asm-end = hier endet das Programm

# Kontext: Kommentar im erzeugten Assembler
basic-b2a-asm-end-heap = darüber liegt der Zeichenketten-Heap (bis RT_HEAPTOP)

# Kontext: Text einer .assert-Zeile im erzeugten Assembler (steht in Anführungszeichen)
# $t: Ziel | Beispiel: c64
basic-b2a-asm-assert = Programm zu groß für { $t }: es muss unterhalb von RT_HEAPTOP enden

# Kontext: Kommentar im erzeugten Assembler vor einer DEF-FN-Funktion
# $n: Zeilennummer | Beispiel: 10
# $name: Name der Funktion | Beispiel: A
# $shown: Name des Parameters | Beispiel: X
basic-b2a-asm-def-fn = Zeile { $n }: DEF FN{ $name }({ $shown }) – Parameter { $shown } gilt nur während des Aufrufs


# I3b
# $kind: Art des Namens | Beispiel: Zahl
basic-names-kind-array = { $kind }, Feld
basic-names-kind-function = Funktion
basic-names-kind-text = Text
basic-names-kind-integer = Ganzzahl
basic-names-kind-number = Zahl
# Ende-Marke der eingebetteten Systemdateien (basic2asm --embed)
basic-embed-end = Ende

# Kontext: Dialektdatei erbt (über `base`) von sich selbst oder die Kette ist zu lang
# $id: Kennung des Dialekts | Beispiel: vic_basic_2
basic-dialect-base-cycle = Dialekt { $id }: base-Kette zu tief oder zyklisch
