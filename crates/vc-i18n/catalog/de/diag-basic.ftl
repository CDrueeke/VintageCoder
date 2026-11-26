# SPDX-License-Identifier: GPL-3.0-or-later
# Fehler B183: Exponent einer Zahl ungültig (Atari)
# $lit: das Zahlenliteral | Beispiel: 1E+X
diag-basic-b183-bad-exponent = ungültiger Exponent in '{ $lit }'

# Fehler B183: Zahlenliteral ungültig (Atari)
# $lit: das Zahlenliteral | Beispiel: 1.2.3
diag-basic-b183-bad-number = ungültige Zahl '{ $lit }'

# Fehler B183: Zahl außerhalb des Bereichs der BCD-Darstellung (mit Höchstwert)
# $lit: das Zahlenliteral | Beispiel: 1E99
diag-basic-b183-number-range-max = Zahl '{ $lit }' außerhalb des Bereichs (max. 9,99…E97)

# Fehler B183: Zahl außerhalb des Bereichs der BCD-Darstellung
# $lit: das Zahlenliteral | Beispiel: 1E-99
diag-basic-b183-number-range = Zahl '{ $lit }' außerhalb des Bereichs

# Fehler B100: Schließende geschweifte Klammer eines Steuercodes fehlt
diag-basic-b100-brace-missing = '{ "}" }' fehlt

# Fehler B100: Steuercode {…} in einer Zeichenkette fehlerhaft (Meldung der Zeichentabelle)
# $text: Meldung der Zeichentabelle | Beispiel: unbekannter Steuercode {nix}
diag-basic-b100-char-ref = { $text }

# Fehler B184: Zeichen außerhalb des ATASCII-Bereichs in einer Zeichenkette
# $c: das Zeichen | Beispiel: é
diag-basic-b184-atascii-range = Zeichen '{ $c }' nicht im ATASCII-Bereich (nutze { "{" }{ "$" }xx{ "}" })

# Fehler B185: Zu viele Variablen im Programm (Atari)
# $n: erlaubte Höchstzahl | Beispiel: 128
diag-basic-b185-too-many-vars = mehr als { $n } Variablen

# Fehler B182: Ungültiger Labelname (Turbo BASIC XL)
# $id: der Name | Beispiel: A$
diag-basic-b182-label-name = '{ $id }' ist kein gültiger Labelname

# Fehler B182: Label erwartet (Turbo BASIC XL)
diag-basic-b182-label-expected = Label erwartet

# Fehler B181: Schließende Klammer fehlt
diag-basic-b181-rparen-missing = ')' fehlt

# Fehler B180: Nach dem Vergleichsoperator fehlt eine Zeichenkette (Atari)
diag-basic-b180-string-compare-expected = Zeichenkette nach dem Vergleichsoperator erwartet

# Fehler B181: Ausdruck unvollständig (Atari)
diag-basic-b181-expr-incomplete = Ausdruck unvollständig

# Fehler B100: Hexzahl zu groß (Atari)
diag-basic-b100-hex-too-big = Hexzahl zu groß

# Fehler B184: Zeichenkette länger als 255 Zeichen (Atari)
diag-basic-b184-string-too-long = Zeichenkette länger als 255 Zeichen

# Fehler B100: Unerwartetes Zeichen
# $c: das Zeichen | Beispiel: #
diag-basic-b100-unexpected-char = unerwartetes Zeichen '{ $c }'

# Fehler B185: Eintrag fehlt in der Dialektdatei (Atari)
# $name: Name des Eintrags | Beispiel: RND0
diag-basic-b185-dialect-missing = { $name } fehlt

# Fehler B180: Flag-Anweisung fehlt in der Dialektdatei (Turbo BASIC XL)
diag-basic-b180-flag-statement-missing = Flag-Anweisung fehlt

# Fehler B182: Reserviertes Wort an einer Stelle, an der ein Name stehen muss (Atari)
# $id: das Wort | Beispiel: TO
diag-basic-b182-reserved-word = reserviertes Wort '{ $id }' an dieser Stelle

# Fehler B180: Zuweisung ohne '=' (Atari)
# $id: Name der Variablen | Beispiel: A
diag-basic-b180-assign-expected = '=' nach '{ $id }' erwartet

# Fehler B180: Anweisung erwartet (Turbo BASIC XL)
diag-basic-b180-statement-expected = Anweisung erwartet

# Fehler B180: THEN fehlt (Atari)
diag-basic-b180-then-expected = THEN erwartet

# Fehler B180: Nach THEN fehlt Anweisung oder Zeilennummer (Atari)
diag-basic-b180-then-target-expected = Anweisung oder Zeilennummer nach THEN erwartet

# Fehler B180: FOR ohne Laufvariable (Atari)
diag-basic-b180-for-var-expected = Laufvariable erwartet

# Fehler B180: '=' fehlt (Atari)
diag-basic-b180-equals-expected = '=' erwartet

# Fehler B180: TO fehlt (Atari)
diag-basic-b180-to-expected = TO erwartet

# Fehler B182: Variable erwartet (Atari)
diag-basic-b182-variable-expected = Variable erwartet

# Fehler B180: ON ohne GOTO oder GOSUB (Atari)
diag-basic-b180-goto-gosub-expected = GOTO oder GOSUB erwartet

# Fehler B180: Parameter bei PROC/EXEC (Turbo BASIC XL)
diag-basic-b180-proc-params = Parameter bei PROC/EXEC sind in Turbo BASIC XL nicht vorgesehen (Variablen sind global)

# Fehler B182: Name erwartet (DIM, Atari)
diag-basic-b182-name-expected = Name erwartet

# Fehler B181: '(' fehlt (DIM, Atari)
diag-basic-b181-lparen-expected = '(' erwartet

# Fehler B185: Leere Zeile (Atari)
diag-basic-b185-empty-line = leere Zeile

# Fehler B185: Zeile länger als 255 Bytes (Atari)
diag-basic-b185-line-too-long-atari = Zeile zu lang (mehr als 255 Bytes)

# Fehler B100: Zeile beginnt nicht mit einer Zeilennummer
diag-basic-b100-no-line-number = Zeile beginnt nicht mit einer Zeilennummer

# Fehler B185: Zeilennummer größer als 32767 (Atari)
# $num: die Zeilennummer | Beispiel: 40000
diag-basic-b185-line-number-max-atari = Zeilennummer { $num } größer als 32767

# Fehler B100: Zeilennummer größer als 63999 (CBM)
# $num: die Zeilennummer | Beispiel: 70000
diag-basic-b100-line-number-max-cbm = Zeilennummer { $num } größer als 63999

# Fehler B102: Zeile länger als 250 Bytes (CBM)
diag-basic-b102-line-too-long = Zeile zu lang (mehr als 250 Bytes)

# Fehler B100: Zeichen ohne PETSCII-Entsprechung
# $c: das Zeichen | Beispiel: é
diag-basic-b100-petscii-char = Zeichen '{ $c }' hat keine PETSCII-Entsprechung

# Fehler B100: Zeichen ohne PETSCII-Entsprechung in einer Zeichenkette mit Steuercodes
# $c: das Zeichen | Beispiel: é
diag-basic-b100-petscii-char-hint = Zeichen '{ $c }' hat keine PETSCII-Entsprechung (nutze { "{" }{ "$" }xx{ "}" })

# Fehler B110: Label nicht definiert
# $name: Name des Labels | Beispiel: schleife
diag-basic-b110-label-undefined = Label @{ $name } ist nicht definiert

# Fehler B121: #include zu tief verschachtelt oder zyklisch
# $name: Dateiname | Beispiel: lib/d.bas
diag-basic-b121-include-nesting = #include "{ $name }": zu tief verschachtelt oder zyklisch

# Fehler B120: #include-Datei nicht gefunden
# $name: Dateiname | Beispiel: fehlt.bas
diag-basic-b120-include-not-found = #include "{ $name }": Datei nicht gefunden (bei exportierten Dateien: erst bauen bzw. exportieren)

# Fehler B100: Zeilennummer größer als das Maximum des Dialekts
# $digits: die Zeilennummer | Beispiel: 70000
# $max: das Maximum | Beispiel: 63999
diag-basic-b100-line-number-max = Zeilennummer { $digits } größer als { $max }

# Fehler B100: Automatisch vergebene Zeilennummer größer als das Maximum
# $num: die Zeilennummer | Beispiel: 64000
# $max: das Maximum | Beispiel: 63999
diag-basic-b100-line-number-auto-max = Zeilennummer { $num } größer als { $max } (zu viele Zeilen oder zu großer Abstand)

# Fehler B100: Zeilennummer nicht größer als die vorige
# $num: die Zeilennummer | Beispiel: 10
# $last: die vorige Zeilennummer | Beispiel: 20
diag-basic-b100-line-number-order = Zeilennummer { $num } nicht größer als die vorige ({ $last })

# Fehler B111: Label doppelt definiert
# $name: Name des Labels | Beispiel: schleife
diag-basic-b111-label-duplicate = Label @{ $name } ist doppelt definiert

# Fehler B112: Label am Ende ohne Anweisung dahinter
# $name: Name des Labels | Beispiel: ende
diag-basic-b112-label-at-end = Label @{ $name } steht am Ende ohne Anweisung dahinter

# Fehler B100: Name nach '@' fehlt
diag-basic-b100-label-name-missing = Label-Name nach '@' fehlt

# Fehler B100: Ungültiger Name mit Punkt (Struktursprache)
# $word: der Name | Beispiel: a.b
diag-basic-b100-bad-name = ungültiger Name '{ $word }' (Punkt nur in Konstanten wie asm.label)

# Fehler B100: Binärzahl zu groß
diag-basic-b100-bin-too-big = Binärzahl zu groß

# Warnung B101: Zeile länger als die Zeilenlänge des Editors (nicht mehr bearbeitbar)
# $max: Zeichen, die der Editor des Dialekts zulässt | Beispiel: 80
diag-basic-b101-line-long = Zeile länger als { $max } Zeichen: im Editor nicht mehr bearbeitbar

# Fehler B102: Zeile länger als 250 Bytes (Struktursprache)
diag-basic-b102-line-too-long-split = Zeile zu lang (mehr als 250 Bytes) – Zeile teilen

# Fehler B103: Reserviertes Wort der Struktursprache als Variablenname
# $name: das Wort | Beispiel: loop
diag-basic-b103-reserved-word = '{ $name }' ist ein reserviertes Wort der Struktursprache

# Fehler B104: Label doppelt definiert (Struktursprache)
# $name: Name des Labels | Beispiel: schleife
diag-basic-b104-label-duplicate = Label '{ $name }' doppelt definiert

# Fehler B105: Zeilennummern über 63999
diag-basic-b105-line-numbers-cbm = Zeilennummern über 63999 – step verkleinern oder pack nutzen

# Fehler B105: Zeilennummern über 32767 (Atari)
diag-basic-b105-line-numbers-atari = Zeilennummern über 32767 – step verkleinern

# Fehler B106: else ohne mehrzeiliges if
diag-basic-b106-else-without-if = else ohne passendes mehrzeiliges if

# Fehler B106: endif ohne if
diag-basic-b106-endif-without-if = endif ohne if

# Fehler B106: wend ohne while
diag-basic-b106-wend-without-while = wend ohne while

# Fehler B106: until ohne repeat
diag-basic-b106-until-without-repeat = until ohne repeat

# Fehler B106: elseif ohne mehrzeiliges if oder nach else
diag-basic-b106-elseif-without-if = elseif ohne passendes mehrzeiliges if (oder nach else)

# Fehler B106: do mit unzulässigem Zusatz
diag-basic-b106-do-syntax = do erwartet nichts, 'while Bedingung' oder 'until Bedingung'

# Fehler B106: loop mit unzulässigem Zusatz
diag-basic-b106-loop-syntax = loop erwartet nichts, 'while Bedingung' oder 'until Bedingung'

# Fehler B106: loop ohne do
diag-basic-b106-loop-without-do = loop ohne do

# Fehler B106: exit außerhalb einer Schleife
diag-basic-b106-exit-outside = exit außerhalb von while, repeat oder do

# Fehler B106: endproc ohne proc
diag-basic-b106-endproc-without-proc = endproc ohne proc

# Fehler B106: Block nicht geschlossen
diag-basic-b106-block-not-closed = Block nicht geschlossen

# Fehler B107: Label nicht definiert (Struktursprache)
# $name: Name des Labels | Beispiel: ende
diag-basic-b107-label-undefined = Label '@{ $name }' nicht definiert

# Fehler B108: const ohne NAME = Wert
diag-basic-b108-const-syntax = const erwartet NAME = Wert

# Fehler B108: const mit unzulässigem Wert
diag-basic-b108-const-value = const erwartet eine einzelne Zahl oder Zeichenkette

# Fehler B109: Name der Prozedur fehlt
diag-basic-b109-proc-name-missing = Prozedurname fehlt

# Fehler B109: Prozedurkopf ohne Klammern um die Parameter
diag-basic-b109-proc-head-syntax = erwartet name(parameter, …)

# Fehler B109: Ungültiger Parametername einer Prozedur
# $p: der Name | Beispiel: 1x
diag-basic-b109-proc-param-name = ungültiger Parametername '{ $p }'

# Fehler B109: proc innerhalb einer proc
diag-basic-b109-proc-nested = proc darf nicht in einer anderen proc stehen

# Fehler B109: Prozedur nicht definiert
# $name: Name der Prozedur | Beispiel: zeichne
diag-basic-b109-proc-undefined = Prozedur '{ $name }' ist nicht definiert

# Fehler B109: Falsche Zahl von Argumenten beim Aufruf einer Prozedur
# $name: Name der Prozedur | Beispiel: zeichne
# $expected: erwartete Zahl | Beispiel: 2
# $given: übergebene Zahl | Beispiel: 1
diag-basic-b109-proc-arg-count = Prozedur '{ $name }' erwartet { $expected } Argument(e), übergeben: { $given }

# Fehler B110: Include-Datei nicht lesbar
# $path: Pfad der Datei | Beispiel: lib/d.bas
# $error: Meldung des Systems | Beispiel: permission denied
diag-basic-b110-include-read = { $path }: { $error }

# Fehler B110: include zu tief verschachtelt
diag-basic-b110-include-depth = include zu tief verschachtelt

# Fehler B110: Include-Datei nicht gefunden
# $name: Dateiname | Beispiel: fehlt.bas
diag-basic-b110-include-not-found = Include-Datei '{ $name }' nicht gefunden

# Fehler B111: #elseif, #else oder #endif ohne #if
# $word: die Direktive | Beispiel: #endif
diag-basic-b111-directive-without-if = { $word } ohne #if

# Fehler B111: #elseif oder #else nach #else
diag-basic-b111-else-after-else = #elseif/#else nach #else

# Fehler B111: {system} im include-Pfad, Zielsystem unbekannt
diag-basic-b111-include-system = { "{" }system{ "}" } im include-Pfad, aber das Zielsystem ist unbekannt

# Fehler B111: #if ohne #endif
diag-basic-b111-if-without-endif = #if ohne #endif

# Fehler B111: Unzulässiges Zeichen in der #if-Bedingung
# $c: das Zeichen | Beispiel: +
diag-basic-b111-cond-char = Zeichen '{ $c }' in der Bedingung nicht erlaubt

# Fehler B111: #if-Bedingung unvollständig
diag-basic-b111-cond-incomplete = Ausdruck unvollständig

# Fehler B111: Schließende Klammer in der #if-Bedingung fehlt
diag-basic-b111-cond-rparen = ')' fehlt

# Fehler B111: Unerwartetes Wort in der #if-Bedingung
# $t: das Wort | Beispiel: and
diag-basic-b111-cond-unexpected = unerwartet '{ $t }'

# Fehler B111: Unbekannter Name in der #if-Bedingung
# $t: der Name | Beispiel: system_c65
diag-basic-b111-cond-unknown-name = unbekannter Name '{ $t }' (bekannt: SYSTEM_C64, SYSTEM_C128, SYSTEM_PLUS4, SYSTEM_VIC20, SYSTEM_MEGA65, SYSTEM_ATARI und -D-Konstanten)

# Fehler B112: RESTORE mit Marke im BASIC V2
diag-basic-b112-restore-label = restore mit Marke gibt es im BASIC V2 nicht (erst ab BASIC 3.5): ohne Marke zurücksetzen und der Reihe nach lesen

# Fehler B113: blob mit falscher Schreibweise
diag-basic-b113-blob-syntax = blob erwartet: blob "datei.asm" [with "song.asm"]

# Fehler B130: Trenner _ falsch gesetzt (Hexzahl)
diag-basic-b130-separator = Trenner _ nur einzeln zwischen zwei Ziffern

# Fehler B131: Trenner _ falsch gesetzt (Binärzahl)
diag-basic-b131-separator = Trenner _ nur einzeln zwischen zwei Ziffern

# Fehler B131: Zeichen ist keine Binärziffer
# $c: das Zeichen | Beispiel: 2
diag-basic-b131-not-binary-digit = '{ $c }' ist keine Binärziffer (nur 0 und 1)

# Fehler B130: Zeichen ist keine Hexziffer
# $c: das Zeichen | Beispiel: G
diag-basic-b130-not-hex-digit = '{ $c }' ist keine Hexziffer (0–9, A–F)

# Fehler B132: Hex- oder Binärzahl ohne Ziffern
# $prefix: Zeichen $ oder % | Beispiel: $
diag-basic-b132-no-digits = { $prefix } ohne Ziffern

# Fehler B130: Zeichen hinter einer Hexzahl ist keine Hexziffer
# $c: das Zeichen | Beispiel: G
diag-basic-b130-not-digit-after = '{ $c }' ist keine Hexziffer (vor einem Schlüsselwort ein Leerzeichen setzen)

# Fehler B131: Zeichen hinter einer Binärzahl ist keine Binärziffer
# $c: das Zeichen | Beispiel: x
diag-basic-b131-not-digit-after = '{ $c }' ist keine Binärziffer

# Fehler B133: Hexzahl zu groß für die Bitbreite des Dialekts
# $bits: Bitbreite | Beispiel: 16
# $max: größter Wert | Beispiel: 65535
diag-basic-b133-hex-bits = Hexzahl über { $bits } Bit (größter Wert { $max })

# Fehler B133: Binärzahl zu groß für die Bitbreite des Dialekts
# $bits: Bitbreite | Beispiel: 16
# $max: größter Wert | Beispiel: 65535
diag-basic-b133-bin-bits = Binärzahl über { $bits } Bit (größter Wert { $max })

# Fehler B134: Adresse außerhalb 0–65535 (Warnung, wenn die Prüfung auf Warnung steht)
# $v: der Wert | Beispiel: 70000
diag-basic-b134-address-range = Adresse { $v } außerhalb 0–65535

# Fehler B135: Wert außerhalb 0–255 (Warnung, wenn die Prüfung auf Warnung steht)
# $v: der Wert | Beispiel: 300
# $kw: das Schlüsselwort | Beispiel: POKE
diag-basic-b135-value-range = Wert { $v } außerhalb 0–255 ({ $kw } schreibt ein Byte)

# Warnung B136: Hexzahl grenzt an ein Schlüsselwort
# $kw: das Schlüsselwort | Beispiel: OR
diag-basic-b136-hex-keyword = Hexzahl grenzt an das Schlüsselwort { $kw }: Leerzeichen setzen (Ziffern A–F könnten zum Wort gehören)

# Warnung B137: Wert passt nicht in eine Ganzzahlvariable
# $v: der Wert | Beispiel: 40000
diag-basic-b137-int-range = { $v } passt nicht in eine Ganzzahlvariable (−32768 … 32767)

# Fehler B138: Hex- oder Binärzahl als Sprungziel
diag-basic-b138-jump-target = Hex/Binär als Sprungziel: Zeilennummern entstehen beim Bauen, Marke verwenden (@name)

# Warnung B139: Hexzahl in C-Schreibweise
diag-basic-b139-c-hex = 0x… ist kein BASIC: Hexzahlen schreibt man { "$" }FF

# Warnung B140: Hex- oder Binärzahl wird im klassischen Weg nicht umgewandelt
# $t: das Literal | Beispiel: $FF
# $file: Name der Quelldatei | Beispiel: spiel.bas
diag-basic-b140-classic-literal = { $t } wird im klassischen Weg nicht umgewandelt, der Dialekt kennt keine Hex-/Binärzahlen (als natives BASIC bauen: vcc basic { $file }, oder Dezimalzahl schreiben)

# Fehler B131: Kein freier Kurzname mehr für einen langen Variablennamen
# $key: der lange Name | Beispiel: punktestand
diag-basic-b131-no-short-name = Für den Namen '{ $key }' ist kein freier Kurzname mehr übrig (zu viele Variablen)

# Warnung W313: Lange Namen unterscheiden sich erst nach den signifikanten Zeichen
# $names: die langen Namen, in Anführungszeichen, mit Komma getrennt | Beispiel: 'punktestand', 'punktezahl'
# $sig: Zahl der unterschiedenen Zeichen | Beispiel: 2
# $nat: gemeinsamer Anfang | Beispiel: PU
# $shorts: die vergebenen Kurznamen | Beispiel: PU, PA
diag-basic-w313-names-collide = { $names } unterscheiden sich erst nach den ersten { $sig } Zeichen ('{ $nat }'); sie werden hier getrennt ({ $shorts }). Ein BASIC ohne diese Umsetzung sähe nur eine Variable

# Warnung W312: Zwei Variablennamen sind im BASIC dieselbe Variable (nur 2 Zeichen zählen)
# $a: erster Name | Beispiel: PUNKTE
# $b: zweiter Name | Beispiel: PUNKTZAHL
diag-basic-w312-same-variable = { $a } und { $b } sind im BASIC dieselbe Variable (nur die ersten 2 Zeichen zählen)

# Warnung I110: Fließkommavariable, Stufe 1 rechnet mit Ganzzahlen
# $name: Name der Variablen | Beispiel: X
diag-basic-i110-float-var = Variable { $name } ist im Original eine Fließkommazahl; Stufe 1 rechnet mit 16-Bit-Ganzzahlen

# Fehler E321: Zeichenkettenvariable ohne DIM (Atari)
# $name: Name der Variablen | Beispiel: A$
diag-basic-e321-string-dim = Zeichenkettenvariable { $name } braucht ein DIM mit fester Größe (Atari-BASIC: DIM { $name }(n), sonst Fehler 9)

# Fehler E303: Zahl zu groß für 16 Bit
# $n: die Zahl | Beispiel: 70000
diag-basic-e303-too-big-16 = Zahl { $n } ist zu groß für 16 Bit

# Fehler E303: DATA-Zahl zu groß für 16 Bit
# $n: die Zahl | Beispiel: 70000
diag-basic-e303-data-too-big-16 = DATA-Zahl { $n } ist zu groß für 16 Bit

# Fehler E303: Zahl zu groß für das BASIC-Zahlenformat (CBM)
# $v: die Zahl | Beispiel: 1e40
diag-basic-e303-mbf-too-big = Zahl { $v } ist zu groß für das BASIC-Zahlenformat

# Fehler E303: Zahl zu groß für das BASIC-Zahlenformat (Atari)
# $v: die Zahl | Beispiel: 1e100
diag-basic-e303-bcd-too-big = Zahl { $v } ist zu groß für das BASIC-Zahlenformat (höchstens 9,99999999E+97)

# Fehler E303: Zahl ungültig
# $s: der Text | Beispiel: 1.2.3
diag-basic-e303-number-invalid = Zahl { $s } ungültig

# Warnung I111: Zahlen über 32767 gelten als 16-Bit-Muster
diag-basic-i111-big-numbers = Zahlen über 32767 gelten als 16-Bit-Muster (Adressen für PEEK/POKE/SYS); im Rechnen und im PRINT erscheinen sie negativ

# Warnung I112: TI liefert nur 16 Bit
diag-basic-i112-ti-16bit = TI liefert nur die unteren 16 Bit der Systemuhr

# Warnung I113: INT in Stufe 1 ohne Wirkung
diag-basic-i113-int-noop = INT ist in Stufe 1 ohne Wirkung (nur Ganzzahlen)

# Fehler E304: Stufe 1 kennt nur eindimensionale Felder
diag-basic-e304-one-dim-stage1 = Stufe 1 kennt nur eindimensionale Felder

# Fehler E304: Nur eindimensionale Felder umgesetzt
diag-basic-e304-one-dim = Es sind nur eindimensionale Felder umgesetzt

# Fehler E304: DIM einer Zeichenkette braucht genau eine Größe
diag-basic-e304-dim-string-one-size = DIM einer Zeichenkette braucht genau eine Größe: DIM A$(n)

# Fehler E304: Teilzeichenkette falsch geschrieben
diag-basic-e304-substring = Teilzeichenkette: A$(n) oder A$(n,m)

# Fehler E305: Funktion braucht genau ein Argument
# $name: Name der Funktion | Beispiel: ABS
diag-basic-e305-one-arg = { $name } erwartet genau ein Argument

# Fehler E305: OPEN mit falschen Argumenten
diag-basic-e305-open-args = OPEN erwartet #Kanal, Zahl, Zahl, Name

# Fehler E305: Falsche Argumentzahl
# $n: Name der Funktion | Beispiel: MID$
diag-basic-e305-arg-count = { $n }: falsche Argumentzahl

# Fehler E306: Funktion wird nicht übersetzt
# $name: Name der Funktion | Beispiel: FRE
diag-basic-e306-function = Funktion { $name } wird nicht übersetzt

# Fehler E307: Wert kann nicht zugewiesen werden
# $name: Name der Variablen | Beispiel: TI
diag-basic-e307-assign = { $name } kann nicht zugewiesen werden

# Fehler E201: Sprungziel existiert nicht
# $n: die Zeilennummer | Beispiel: 100
diag-basic-e201-jump-target = Sprungziel: Zeile { $n } existiert nicht

# Fehler E320: Typfehler (TYPE MISMATCH) in einem Ausdruck
diag-basic-e320-assign = Typen passen nicht (TYPE MISMATCH): Zeichenkette und Zahl

# Fehler E320: Typfehler (TYPE MISMATCH) in einem Ausdruck
diag-basic-e320-open-name = OPEN: der Dateiname muss eine Zeichenkette sein (TYPE MISMATCH)

# Fehler E320: Typfehler (TYPE MISMATCH) in einem Ausdruck
diag-basic-e320-number-expected = Zahl erwartet, aber eine Zeichenkette (TYPE MISMATCH)

# Fehler E320: Typfehler (TYPE MISMATCH) in einem Ausdruck
diag-basic-e320-string-expected = Zeichenkette, aber eine Zahl erwartet (TYPE MISMATCH)

# Fehler E320: Typfehler (TYPE MISMATCH) in einem Ausdruck
# $name: Name der Variablen | Beispiel: A
diag-basic-e320-string-var-number = Zeichenkettenvariable { $name }$, aber eine Zahl erwartet (TYPE MISMATCH)

# Fehler E320: Typfehler (TYPE MISMATCH) in einem Ausdruck
diag-basic-e320-string-ops = Mit Zeichenketten sind nur + und Vergleiche erlaubt (TYPE MISMATCH)

# Fehler E320: Typfehler (TYPE MISMATCH) in einem Ausdruck
diag-basic-e320-compare = Zeichenkette und Zahl lassen sich nicht vergleichen (TYPE MISMATCH)

# Fehler E320: Typfehler (TYPE MISMATCH) in einem Ausdruck
diag-basic-e320-arith = Mit einer Zeichenkette kann man nicht rechnen (TYPE MISMATCH)

# Fehler E320: Typfehler (TYPE MISMATCH) in einem Ausdruck
# $n: Name der Funktion | Beispiel: LEFT$
diag-basic-e320-expects-string = { $n } erwartet eine Zeichenkette (TYPE MISMATCH)

# Fehler E320: Typfehler (TYPE MISMATCH) in einem Ausdruck
diag-basic-e320-dec-string = DEC erwartet eine Zeichenkette (TYPE MISMATCH)

# Fehler E320: Typfehler (TYPE MISMATCH) in einem Ausdruck
diag-basic-e320-instr = INSTR(a$,b$) oder INSTR(a$,b$,n) mit Zeichenketten (TYPE MISMATCH)

# Fehler E320: Typfehler (TYPE MISMATCH) in einem Ausdruck
# $n: Name der Funktion | Beispiel: CHR$
diag-basic-e320-returns-string = { $n } liefert eine Zeichenkette, hier wird eine Zahl erwartet (TYPE MISMATCH)

# Fehler E320: Typfehler (TYPE MISMATCH) in einem Ausdruck
diag-basic-e320-concat = Zeichenkette und Zahl lassen sich nicht verketten (TYPE MISMATCH)

# Fehler E320: Typfehler (TYPE MISMATCH) in einem Ausdruck
diag-basic-e320-str-number = STR$ erwartet eine Zahl (TYPE MISMATCH)

# Fehler E320: Typfehler (TYPE MISMATCH) in einem Ausdruck
diag-basic-e320-string-expr = Zeichenkettenausdruck erwartet (TYPE MISMATCH)

# Fehler E320: Typfehler (TYPE MISMATCH) in einem Ausdruck
# $name: Name der Funktion | Beispiel: A
diag-basic-e320-fn-number = FN{ $name } erwartet eine Zahl (TYPE MISMATCH)

# Fehler E308: INPUT mit mehreren Variablen in Stufe 1
diag-basic-e308-input-multi = INPUT mit mehreren Variablen wird in Stufe 1 nicht übersetzt (je Variable ein INPUT)

# Fehler E311: GET in eine Zeichenkette (Atari)
diag-basic-e311-get-string = GET in eine Zeichenkette gibt es in Atari-BASIC nicht (GET #n,A liest ein Byte in eine Zahl)

# Fehler E311: Anweisung wird nicht übersetzt
# $k: die Anweisung | Beispiel: LIST
diag-basic-e311-unsupported = { $k } wird nicht übersetzt (Dateien, LIST/NEW/CONT und DEF FN sind nicht Teil der Umsetzung)

# Fehler E311: + für Zeichenketten (Atari)
diag-basic-e311-plus-strings = Atari-BASIC kennt kein + für Zeichenketten (A$(LEN(A$)+1) = B$ hängt an)

# Fehler E311: Funktion gibt es in Atari-BASIC nicht
# $n: Name der Funktion | Beispiel: LEFT$
diag-basic-e311-not-in-atari = { $n } gibt es in Atari-BASIC nicht (Teilzeichenkette A$(n,m) benutzen)

# Fehler E311: Atari-Wort wird nicht übersetzt
# $w: das Wort | Beispiel: XIO
diag-basic-e311-atari-only = '{ $w }' ist Atari-BASIC und wird nicht übersetzt (Kanal- und Dateibefehle, XIO, TRAP, NOTE/POINT, PADDLE u. a.)

# Fehler E314: FOR-Variable ganzzahlig
# $name: Name der Variablen | Beispiel: I
diag-basic-e314-for-int = FOR-Variable { $name }% darf nicht ganzzahlig sein (im BASIC V2 ein SYNTAX ERROR)

# Fehler E309: POKE-Wert außerhalb 0..255
diag-basic-e309-poke-range = POKE-Wert muss 0..255 sein

# Fehler E310: RUN mit Zeilennummer
diag-basic-e310-run-line = RUN mit Zeilennummer wird nicht übersetzt

# Fehler E312: DIM A$(n): Größe außerhalb des Bereichs (Atari)
diag-basic-e312-dim-size-range = DIM A$(n): n muss 1 bis 32767 sein (Atari-BASIC: sonst Fehler 9)

# Fehler E312: DIM A$(n): Größe keine Zahl
diag-basic-e312-dim-size-number = DIM A$(n) braucht in basic2asm eine Zahl als Größe

# Fehler E312: DIM: Größe keine Zahl (Stufe 1)
diag-basic-e312-dim-number = DIM braucht in Stufe 1 eine Zahl als Größe

# Fehler E313: Feld zweimal dimensioniert
# $name: Name des Feldes | Beispiel: A
diag-basic-e313-redim = Feld { $name } wird zweimal dimensioniert (REDIM'D ARRAY)

# Fehler E315: FN mit Zeichenketten-Name oder -Parameter
# $name: Name der Funktion | Beispiel: A$
diag-basic-e315-fn-string = FN{ $name }: Zeichenkettenfunktionen und -parameter werden nicht übersetzt

# Fehler E315: Aufruf einer Zeichenkettenfunktion
diag-basic-e315-fn-string-plain = Zeichenkettenfunktionen werden nicht übersetzt

# Fehler E316: FN zweimal definiert
# $name: Name der Funktion | Beispiel: A
diag-basic-e316-fn-twice = FN{ $name } wird zweimal definiert

# Fehler E317: FN nirgends definiert
# $name: Name der Funktion | Beispiel: A
diag-basic-e317-fn-undefined = FN{ $name } ist nirgends mit DEF FN definiert (UNDEF'D FUNCTION)

# Fehler E322: DATA-Eintrag keine Zahl (Atari)
# $entry: der Eintrag | Beispiel: abc
diag-basic-e322-data-entry = DATA-Eintrag '{ $entry }' ist keine Zahl (Atari-BASIC: Fehler 8 beim Lesen)

# Fehler E340: Prozedur zweimal definiert (Turbo BASIC XL)
# $n: Name der Prozedur | Beispiel: zeichne
diag-basic-e340-proc-twice = Prozedur { $n } wird zweimal definiert

# Fehler E341: Turbo-BASIC-Blöcke am Ende nicht abgeschlossen
# $n: Zahl der offenen Blöcke | Beispiel: 2
diag-basic-e341-open-blocks = { $n } Block(s) von Turbo BASIC XL am Programmende nicht abgeschlossen (ENDIF, WEND, UNTIL, LOOP oder ENDPROC fehlt)

# Fehler E334: TAN gibt es in Atari-BASIC nicht
diag-basic-e334-tan = TAN gibt es in Atari-BASIC nicht (SIN(x)/COS(x) benutzen)

# Fehler E342: Turbo-BASIC-Block passt nicht
diag-basic-e342-else = ELSE ohne passendes IF (oder zweites ELSE)

# Fehler E342: Turbo-BASIC-Block passt nicht
diag-basic-e342-endif = ENDIF ohne passendes IF

# Fehler E342: Turbo-BASIC-Block passt nicht
diag-basic-e342-wend = WEND ohne passendes WHILE

# Fehler E342: Turbo-BASIC-Block passt nicht
diag-basic-e342-until = UNTIL ohne passendes REPEAT

# Fehler E342: Turbo-BASIC-Block passt nicht
diag-basic-e342-loop = LOOP ohne passendes DO

# Fehler E342: Turbo-BASIC-Block passt nicht
diag-basic-e342-exit = EXIT außerhalb von WHILE, REPEAT und DO

# Fehler E342: Turbo-BASIC-Block passt nicht
diag-basic-e342-proc-nested = PROC innerhalb einer Prozedur

# Fehler E342: Turbo-BASIC-Block passt nicht
diag-basic-e342-endproc = ENDPROC ohne passendes PROC

# Fehler E343: Prozedur in ON … EXEC nicht definiert
# $n: Name der Prozedur | Beispiel: zeichne
diag-basic-e343-on-exec-undefined = ON … EXEC: Prozedur { $n } ist nicht definiert

# Fehler E343: Prozedur in EXEC nicht definiert
# $n: Name der Prozedur | Beispiel: zeichne
diag-basic-e343-exec-undefined = EXEC: Prozedur { $n } ist nicht definiert

# Fehler E330: Ziel nicht umgesetzt
# $target: das Ziel | Beispiel: vic20
# $available: die verfügbaren Ziele | Beispiel: c64, c128, plus4, mega65, atari8
diag-basic-e330-target = Ziel '{ $target }' ist nicht umgesetzt (verfügbar: { $available })

# Fehler E331: Fließkomma für das Ziel nicht umgesetzt
# $t: das Ziel | Beispiel: mega65
# $available: Ziele mit Fließkomma | Beispiel: c64,c128,plus4,atari8
diag-basic-e331-float-target = Fließkomma (Stufe 2) ist für das Ziel { $t } noch nicht umgesetzt (verfügbar: { $available }); das Programm braucht es

# Fehler E300: Stufe nicht umgesetzt
# $stage: die Stufe | Beispiel: 4
diag-basic-e300-stage = Stufe { $stage } ist nicht umgesetzt; verfügbar sind Stufe 1 (Ganzzahl), 2 (Fließkomma) und 3 (Zeichenketten)

# Fehler E301: Konstrukt braucht Stufe 2 (Fließkomma), die Einstellung erlaubt weniger
# $what: das Konstrukt (Meldung) | Beispiel: Potenz '^'
# $max: höchste erlaubte Stufe | Beispiel: 1
diag-basic-e301-needs-stage = { $what } braucht Stufe 2 (Fließkomma); erlaubt ist höchstens Stufe { $max }

# Fehler E302: Konstrukt braucht Stufe 3 (Zeichenketten), die Einstellung erlaubt weniger
# $what: das Konstrukt (Meldung) | Beispiel: Zeichenkettenfeld
# $max: höchste erlaubte Stufe | Beispiel: 2
diag-basic-e302-needs-stage = { $what } braucht Stufe 3 (Zeichenketten); erlaubt ist höchstens Stufe { $max }

# Fehler E344: Turbo BASIC XL nur für atari8
diag-basic-e344-turbo-target = Turbo BASIC XL gibt es nur für das Ziel atari8

# Fehler E192: embed: Laufzeitdatei nicht lesbar
# $path: Pfad | Beispiel: data/sys/c64/b2a/rt_core.asm
# $error: Meldung des Systems | Beispiel: not found
diag-basic-e192-embed-read = embed: { $path }: { $error }

# Fehler E192: embed: Include-Schleife
# $name: Name der Datei | Beispiel: rt_core.asm
diag-basic-e192-embed-loop = embed: Include-Schleife bei { $name }

# Fehler E192: embed ohne Ordner data
diag-basic-e192-embed-data-dir = embed braucht den Ordner data (B2aOptions::data_dir)

# Fehler E191: Rundreise ergibt anderen Syntaxbaum
# $at: erste abweichende Zeilennummer | Beispiel: 20
diag-basic-e191-roundtrip = Rundreise ergibt einen anderen Syntaxbaum (erste Abweichung in Zeile { $at })

# Warnung I150: Hinweis zu Zahlen beim Atari-Ziel (Fließkomma)
diag-basic-i150-atari-bcd = Atari: Zahlen sind 6-Byte-BCD des OS-Fließkommapakets (wie Atari-BASIC); TAN gibt es nicht; PRINT gibt Zahlen wie Atari-BASIC ohne Vorzeichenstelle aus; Fehlermeldungen heißen wie bei CBM (z. B. ILLEGAL QUANTITY)

# Warnung I150: Hinweis zu Zahlen beim Atari-Ziel (Ganzzahl)
diag-basic-i150-atari-int = Atari: Zahlen sind 16-Bit-Ganzzahlen statt BCD mit 10 Stellen; PRINT gibt Zahlen wie Atari-BASIC ohne Vorzeichenstelle aus; Fehlermeldungen heißen wie bei CBM (z. B. ILLEGAL QUANTITY)

# Fehler E190: Anweisung lässt sich nicht zurückschreiben
# $k: die Anweisung | Beispiel: LOAD
diag-basic-e190-unsupported = { $k } lässt sich nicht zurückschreiben (nicht übersetzte Anweisung)

# Fehler E190: Rundreise: Fehler in einer Zeile
# $n: Zeilennummer | Beispiel: 10
# $e: innere Meldung | Beispiel: (Meldung)
diag-basic-e190-line = Zeile { $n }: { $e }

# Fehler E100: Zeile beginnt nicht mit einer Zeilennummer
diag-basic-e100-no-line-number = Zeile beginnt nicht mit einer Zeilennummer

# Fehler E101: Unbekanntes Token
# $b: Byte (Hex) | Beispiel: FF
diag-basic-e101-unknown-token = unbekanntes Token ${ $b }

# Fehler E101: Unerwartetes Zeichen
# $b: Byte (Hex) | Beispiel: 7C
diag-basic-e101-unexpected-byte = unerwartetes Zeichen ${ $b }

# Fehler E102: Zeilennummer doppelt
# $num: die Zeilennummer | Beispiel: 10
diag-basic-e102-line-twice = Zeilennummer { $num } doppelt

# Fehler E103: Syntaxfehler in einer Zeile
# $num: Zeilennummer | Beispiel: 10
# $e: innere Meldung | Beispiel: (Meldung)
diag-basic-e103-line = Zeile { $num }: { $e }

# Fehler E103: Syntaxfehler im BASIC-Text (basic2asm)
# $c: das Zeichen | Beispiel: (
diag-basic-e103-sym-expected = '{ $c }' erwartet

# Fehler E103: Syntaxfehler im BASIC-Text (basic2asm)
# $s: der Text | Beispiel: 70000
diag-basic-e103-line-no-invalid = Zeilennummer { $s } ungültig

# Fehler E103: Syntaxfehler im BASIC-Text (basic2asm)
diag-basic-e103-line-no-expected = Zeilennummer erwartet

# Fehler E103: Syntaxfehler im BASIC-Text (basic2asm)
diag-basic-e103-not-sign = NOT mit einem Vorzeichen davor (NOT -1) ist in Atari-BASIC Fehler 11; Klammern setzen: NOT (-1)

# Fehler E103: Syntaxfehler im BASIC-Text (basic2asm)
diag-basic-e103-fn-name-expected = Name nach FN erwartet

# Fehler E103: Syntaxfehler im BASIC-Text (basic2asm)
diag-basic-e103-fn-one-arg = FN erwartet genau ein Argument

# Fehler E103: Syntaxfehler im BASIC-Text (basic2asm)
diag-basic-e103-rand-one-arg = RAND erwartet genau ein Argument

# Fehler E103: Syntaxfehler im BASIC-Text (basic2asm)
# $k: das Schlüsselwort | Beispiel: GOTO
diag-basic-e103-kw-not-allowed = { $k } ist hier nicht erlaubt

# Fehler E103: Syntaxfehler im BASIC-Text (basic2asm)
# $found: das gefundene Element | Beispiel: Sym(')')
diag-basic-e103-expr-expected-found = Ausdruck erwartet, gefunden { $found }

# Fehler E103: Syntaxfehler im BASIC-Text (basic2asm)
diag-basic-e103-expr-expected = Ausdruck erwartet

# Fehler E103: Syntaxfehler im BASIC-Text (basic2asm)
diag-basic-e103-variable-expected = Variable erwartet

# Fehler E103: Syntaxfehler im BASIC-Text (basic2asm)
# $found: das gefundene Element | Beispiel: Sym(')')
diag-basic-e103-unexpected = unerwartet: { $found }

# Fehler E103: Syntaxfehler im BASIC-Text (basic2asm)
diag-basic-e103-equals-expected = '=' erwartet

# Fehler E103: Syntaxfehler im BASIC-Text (basic2asm)
# $found: das gefundene Element | Beispiel: Num("5")
diag-basic-e103-stmt-expected-found = Anweisung erwartet, gefunden { $found }

# Fehler E103: Syntaxfehler im BASIC-Text (basic2asm)
diag-basic-e103-stmt-expected = Anweisung erwartet

# Fehler E103: Syntaxfehler im BASIC-Text (basic2asm)
# $k: das Schlüsselwort | Beispiel: NEXT
diag-basic-e103-name-after-expected = Name nach { $k } erwartet

# Fehler E103: Syntaxfehler im BASIC-Text (basic2asm)
diag-basic-e103-move-three = -MOVE erwartet 3 Zahlen

# Fehler E103: Syntaxfehler im BASIC-Text (basic2asm)
# $k: das Schlüsselwort | Beispiel: POSITION
# $want: erwartete Zahl | Beispiel: 2
diag-basic-e103-numbers-expected = { $k } erwartet { $want } Zahlen

# Fehler E103: Syntaxfehler im BASIC-Text (basic2asm)
diag-basic-e103-semicolon-channel = ';' nach der Kanalnummer erwartet

# Fehler E103: Syntaxfehler im BASIC-Text (basic2asm)
diag-basic-e103-semicolon-prompt = ';' nach dem Eingabetext erwartet

# Fehler E103: Syntaxfehler im BASIC-Text (basic2asm)
diag-basic-e103-to-after-go = TO nach GO erwartet

# Fehler E103: Syntaxfehler im BASIC-Text (basic2asm)
diag-basic-e103-proc-name-on-exec = Prozedurname nach ON … EXEC erwartet

# Fehler E103: Syntaxfehler im BASIC-Text (basic2asm)
diag-basic-e103-to-expected = TO erwartet

# Fehler E103: Syntaxfehler im BASIC-Text (basic2asm)
diag-basic-e103-goto-gosub-after-on = GOTO oder GOSUB nach ON erwartet

# Fehler E103: Syntaxfehler im BASIC-Text (basic2asm)
diag-basic-e103-then-or-goto = THEN oder GOTO nach IF erwartet

# Fehler E103: Syntaxfehler im BASIC-Text (basic2asm)
diag-basic-e103-loop-var-expected = Schleifenvariable erwartet

# Fehler E103: Syntaxfehler im BASIC-Text (basic2asm)
diag-basic-e103-array-name-expected = Feldname erwartet

# Fehler E103: Syntaxfehler im BASIC-Text (basic2asm)
diag-basic-e103-fn-after-def = FN nach DEF erwartet

# Fehler E103: Syntaxfehler im BASIC-Text (basic2asm)
diag-basic-e103-fn-name-after-fn = Funktionsname nach FN erwartet

# Fehler E103: Syntaxfehler im BASIC-Text (basic2asm)
diag-basic-e103-param-name-expected = Parametername erwartet

# Fehler E103: Syntaxfehler im BASIC-Text (basic2asm)
# $k: das Schlüsselwort | Beispiel: TO
diag-basic-e103-not-a-statement = { $k } ist hier keine Anweisung


# Warnung B170: Der gewählte BASIC-Dialekt passt nicht zum Zielsystem
# $id: Kennung des Dialekts | Beispiel: vic_basic_2
# $system: Kennung des Zielsystems | Beispiel: c64
diag-basic-b170-dialect-system = Dialekt '{ $id }' ist nicht für das Zielsystem '{ $system }' vorgesehen

# Fehler B178: Unbekannter BASIC-Dialekt (Projekteinstellung, Parameter oder `#dialect`)
# $id: angegebene Kennung | Beispiel: basic99
# $known: bekannte Kennungen, durch Komma getrennt | Beispiel: atari_basic, basic35, basic65
diag-basic-b178-dialect-unknown = unbekannter BASIC-Dialekt '{ $id }' (bekannt: { $known })

# Warnung B160: Die Datei mischt Zeilennummern und Labels (Schnellkorrektur: in Labels oder in Zeilennummern umwandeln)
# $numbers: Zahl der Zeilen mit Nummer | Beispiel: 12
# $labels: Zahl der Zeilen mit Label oder Verweis | Beispiel: 3
diag-basic-b160-mixed-mode =
    Zeilennummern und Labels sind gemischt ({ $numbers ->
        [one] 1 Zeile
       *[other] { $numbers } Zeilen
    } mit Nummer, { $labels } mit Label)

# Fehler B161: Umwandlung nicht möglich, weil Sprungziele berechnet werden (Zusammenfassung; die Stellen stehen als B162)
# $count: Zahl der Stellen | Beispiel: 2
diag-basic-b161-convert-failed = Umwandlung in Labels nicht möglich: { $count } berechnete(s) Sprungziel(e)

# Fehler B161: Neu-Nummerieren eines Bereichs nicht möglich, weil Zeilen ohne Nummer vorkommen
diag-basic-b161-range-labels = Bereich neu nummerieren geht nur im Nummernmodus (es gibt Zeilen ohne Nummer)

# Fehler B161: Neu-Nummerieren eines Bereichs würde die Reihenfolge der Zeilen ändern
# $num: neue Nummer | Beispiel: 120
# $last: davor stehende Nummer | Beispiel: 150
diag-basic-b161-range-order = Bereich neu nummerieren würde die Reihenfolge ändern (Zeile { $num } nach { $last })

# Fehler B162: Sprungziel wird berechnet und lässt sich nicht in ein Label umwandeln (Warnung beim Neu-Nummerieren: das Ziel kann danach falsch zeigen)
# $word: Sprungwort | Beispiel: GOTO
# $text: Anweisung | Beispiel: GOTO A*10
diag-basic-b162-computed-target = berechnetes Sprungziel hinter { $word }: { $text }

# Warnung B163: Das Programm wertet die Zeilennummer zur Laufzeit aus (z. B. EL); Umwandlung oder Neu-Nummerieren ändert den Wert
# $word: das Wort | Beispiel: EL
diag-basic-b163-runtime-line = { $word } liefert die Zeilennummer zur Laufzeit; nach Umwandlung oder Neu-Nummerieren stimmen die Nummern nicht mehr

# Warnung B153: Sprungziel (Nummer) nicht vorhanden
# $num: die Nummer | Beispiel: 500
diag-basic-b153-target-missing = Sprungziel { $num } ist keine Programmzeile

# Warnung B165: Markerzeile ohne folgende Anweisung (verwaist; bleibt erhalten)
# $name: Name | Beispiel: schleife
diag-basic-b165-marker-orphan = Marker ;@{ $name } steht ohne folgende Anweisung

# Fehler B166: Dateien einer Einbindung haben verschiedene Modi
# $file: die Datei | Beispiel: lib/daten.bas
# $mode: numbers oder labels | Beispiel: labels
diag-basic-b166-include-mode = { $file }: Modus { $mode } passt nicht zu den übrigen Dateien der Einbindung

# Warnung B168: Markerzeile ungültig oder Name doppelt
# $text: die Zeile | Beispiel: ;@1x
diag-basic-b168-marker-invalid = ungültige oder doppelte Markerzeile: { $text }

# Warnung B150: Ein Schlüsselwort steckt mitten in einem Namen; der Interpreter zerlegt den Namen beim Tokenisieren
# $name: der Name | Beispiel: SCORE
# $keyword: das darin enthaltene Schlüsselwort | Beispiel: OR
diag-basic-b150-keyword-in-name = '{ $name }' enthält das Schlüsselwort { $keyword } und wird zerlegt

# Warnung B151: FOR ohne passendes NEXT
# $var: Schleifenvariable | Beispiel: I
diag-basic-b151-for-without-next = FOR { $var } hat kein passendes NEXT

# Warnung B151: NEXT ohne passendes FOR
# $var: Schleifenvariable (leer ohne Angabe) | Beispiel: I
diag-basic-b151-next-without-for = NEXT { $var } hat kein passendes FOR

# Warnung B152: GOSUB, aber nirgends ein RETURN
diag-basic-b152-gosub-without-return = GOSUB ohne irgendein RETURN im Programm

# Warnung B152: RETURN, aber nirgends ein GOSUB
diag-basic-b152-return-without-gosub = RETURN ohne irgendein GOSUB im Programm

# Warnung B154: Zwei Variablennamen unterscheiden sich erst hinter den Zeichen, die der Interpreter beachtet
# $name: der zweite Name | Beispiel: COUNTER
# $other: der erste Name | Beispiel: COUNT
# $chars: Zahl der beachteten Zeichen | Beispiel: 2
diag-basic-b154-variable-clash = { $name } und { $other } sind für den Interpreter dieselbe Variable (nur die ersten { $chars } Zeichen zählen)

# Hinweis B155: Zeichenkette am Zeilenende nicht geschlossen (im BASIC erlaubt)
diag-basic-b155-string-open = Zeichenkette am Zeilenende nicht geschlossen

# Warnung B156: Anweisung oder Funktion mit falscher Zahl von Argumenten
# $word: das Wort | Beispiel: POKE
# $count: gefundene Zahl | Beispiel: 1
# $expected: erlaubte Zahl oder Bereich | Beispiel: 2
diag-basic-b156-arg-count = { $word } mit { $count } Argument(en); erwartet: { $expected }

# Warnung B157: Anweisung gehört zu einem anderen BASIC-Dialekt
# $word: das Wort | Beispiel: GRAPHIC
# $dialect: gewählter Dialekt | Beispiel: cbm_v2
diag-basic-b157-keyword-unknown = { $word } gehört nicht zum Dialekt { $dialect }

# Warnung B159: Funktion ohne Klammer
# $word: die Funktion | Beispiel: RND
diag-basic-b159-function-paren = { $word } braucht ein Argument in Klammern

# Warnung B167: Das Zeichen ' ist in diesem Dialekt kein Kommentar
diag-basic-b167-quote-not-comment = ' ist in diesem BASIC kein Kommentarzeichen (Kommentare: REM)

# Fehler B172: Der Name in #define ist ein Schlüsselwort des Dialekts
# $name: der Name | Beispiel: PRINT
diag-basic-b172-define-keyword = #define: '{ $name }' ist ein Schlüsselwort und darf nicht als Name dienen

# Fehler B174: #define hat keinen gültigen Namen
# $text: die Zeile | Beispiel: #define 1X 5
diag-basic-b174-define-syntax = #define erwartet einen Namen aus Buchstaben, Ziffern und _ : { $text }

# Fehler B174: Name zweimal mit #define festgelegt
# $name: der Name | Beispiel: SCR
diag-basic-b174-define-twice = #define: '{ $name }' ist schon festgelegt

# Fehler B175: Bedingung von #if nicht lesbar
# $text: die Bedingung | Beispiel: SYSTEM-C64
diag-basic-b175-if-condition = #if: Bedingung '{ $text }' nicht lesbar (erlaubt: SYSTEM_C64, ein definierter Name, vorangestelltes !)

# Fehler B175: #else ohne #if
diag-basic-b175-else-without-if = #else ohne #if

# Fehler B175: zweites #else im selben #if
diag-basic-b175-else-twice = zweites #else im selben #if

# Fehler B175: #endif ohne #if
diag-basic-b175-endif-without-if = #endif ohne #if

# Fehler B175: #if ohne #endif
diag-basic-b175-if-not-closed = #if ohne #endif

# Fehler B176: Ersetzung eines #define-Namens ist zyklisch oder zu tief verschachtelt
# $name: der Name | Beispiel: A
diag-basic-b176-define-cycle = #define: '{ $name }' verweist auf sich selbst (zyklisch oder mehr als acht Stufen)

# Fehler B177: #asm hat keine gültige Form
diag-basic-b177-blob-syntax = #asm erwartet: #asm "datei.asm" [with "song.asm"]

# Fehler B177: Der Programmtext braucht eine Marke, die der Maschinencode nicht festlegt
# $name: die Marke (klein) | Beispiel: start
diag-basic-b177-symbol-missing = asm.{ $name }: der Maschinencode legt keine Marke '{ $name }' fest

# Fehler B177: Code und BASIC-Text passen nicht in den Speicher
# $end: Endadresse (hexadezimal) | Beispiel: 10A00
diag-basic-b177-too-large = Programm samt Maschinencode endet bei ${ $end }: zu groß

# Fehler B177: Die Adresse des Maschinencodes wird nicht stabil
diag-basic-b177-address-unstable = Adresse des Maschinencodes wird nicht stabil (BASIC-Text und Zahlen beeinflussen sich)

# Fehler B177: Der Maschinencode ist zu groß für den Speicher unter dem BASIC-ROM
# $bytes: Größe des Codes | Beispiel: 30000
diag-basic-b177-code-too-large = Maschinencode ({ $bytes } Bytes) passt nicht unter { "$" }A000

# Fehler B177: #asm ist hier nicht möglich (Übersetzen ohne Anbindung des Assemblers)
diag-basic-b177-blob-here = #asm braucht den Bau mit Maschinencode (vcc basic bzw. Bauen in der IDE mit Assembler-Anbindung)

# Fehler B173: Die DATA-Zeilen des Maschinencodes passen nicht mehr hinter die letzte Zeilennummer
# $last: höchste Zeilennummer | Beispiel: 63000
# $lines: Zahl der DATA-Zeilen | Beispiel: 200
# $max: größte erlaubte Nummer | Beispiel: 63999
diag-basic-b173-blob-range = Zeilennummern reichen nicht für die DATA-Zeilen des Maschinencodes ({ $lines } Zeilen nach { $last }, höchstens { $max })

# Fehler B190: Die Zeile #numbering ist ungültig
# $text: die Zeile | Beispiel: #numbering step=x
diag-basic-b190-numbering-syntax = ungültige Zeile '{ $text }' (erwartet: #numbering start=100 step=5)

# Fehler B191: Start oder Schrittweite der Zeilennummern außerhalb des erlaubten Bereichs
# $start: Start | Beispiel: 100
# $step: Schrittweite | Beispiel: 0
# $max: größte Zeilennummer des Dialekts | Beispiel: 63999
diag-basic-b191-numbering-range = Zeilennummern: Start { $start } und Schrittweite { $step } ungültig (Schrittweite mindestens 1, Start höchstens { $max })

# Fehler B192: Die Zeilennummern reichen bei dieser Schrittweite nicht für alle Zeilen
# $count: Zahl der Zeilen | Beispiel: 900
# $last: letzte Nummer bei dieser Schrittweite | Beispiel: 90000
# $max: größte Zeilennummer des Dialekts | Beispiel: 63999
# $suggestion: Vorschlag für die Schrittweite | Beispiel: 7
diag-basic-b192-numbering-overflow = { $count } Zeilen brauchen bei dieser Schrittweite Nummern bis { $last } (größte erlaubte: { $max }); kleinere Schrittweite, z. B. { $suggestion }

# Warnung B150: derselbe Name kommt in weiteren Zeilen vor (eine Meldung je Name und Datei)
# $name: der Name | Beispiel: SCORE
# $keyword: das darin enthaltene Schlüsselwort | Beispiel: OR
# $lines: weitere Zeilen | Beispiel: 5, 7
diag-basic-b150-keyword-in-name-more = '{ $name }' enthält das Schlüsselwort { $keyword } und wird zerlegt (auch in Zeile { $lines })

# Fehler B177: die frühere Schreibweise #blob gibt es nicht mehr
diag-basic-b177-blob-renamed = #blob heißt jetzt #asm: #asm "datei.asm" [with "song.asm"]

# Fehler B193: Medium beim Einbinden nicht gefunden oder nicht als BASIC-DATA umwandelbar
# $name: Pfad im #include | Beispiel: graphics/hero.vcsprite
# $reason: Grund des Auflösers | Beispiel: Medium nicht gefunden
diag-basic-b193-media-failed = #include "{ $name }": { $reason }

# Warnung B194: Altpfad build/export im #include
# $name: Pfad im #include | Beispiel: build/export/hero.data.bas
# $source: Medium, das den Text erzeugt | Beispiel: graphics/hero.vcsprite
diag-basic-b194-legacy-path = #include "{ $name }": veralteter Pfad; besser direkt das Medium einbinden: #include "{ $source }"

# Warnung B195: Programm größer als der freie BASIC-Speicher
# $size: Größe des übersetzten Programms in Byte | Beispiel: 41200
# $free: freier BASIC-Speicher in Byte | Beispiel: 38911
# $system: Zielsystem | Beispiel: c64
diag-basic-b195-program-too-big = Das Programm belegt { $size } Byte, frei sind { $free } Byte ({ $system }); es passt nicht in den Speicher. Maschinencode mit #asm oder weniger DATA-Zeilen verwenden

# Warnung B196: RESTORE @marke bei mehreren Medien in einem BASIC ohne RESTORE-Ziel
# $count: Zahl der eingebundenen Medien | Beispiel: 2
# $dialect: Dialekt | Beispiel: cbm_v2
diag-basic-b196-media-restore-order = { $count } Medien eingebunden, aber { $dialect } kennt kein RESTORE auf eine Zeile: READ liest streng der Reihe nach; die Medien in Lesereihenfolge einbinden
