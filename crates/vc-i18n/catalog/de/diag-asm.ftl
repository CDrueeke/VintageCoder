# SPDX-License-Identifier: GPL-3.0-or-later
# Meldungen des Assemblers (vc-asm), i18n I2. Schlüssel diag-asm-<code>[-<variante>]; Meldungen ohne Code (Modulbau, Ausgabe) heißen asm-…

# Fehler E201: Befehl gibt es auf dieser CPU nicht
# $mn: Befehl (Mnemonic), bei Q-Befehlen mit Q | Beispiel: SLO
# $cpu: Kennung der CPU | Beispiel: 6510
diag-asm-e201-cpu = { $mn } existiert auf CPU { $cpu } nicht

# Fehler E201: undokumentierter Opcode bei illegals=deny
# $mn: Befehl (Mnemonic) | Beispiel: SLO
diag-asm-e201-illegal-denied = undokumentierter Opcode { $mn } (illegals=deny)

# Fehler E202: Fehler in einem Ausdruck (mit dem Ausdruck)
# $inner: Fehlermeldung des Ausdrucks | Beispiel: (Meldung)
# $expr: Ausdruck | Beispiel: 1 +
diag-asm-e202-expression = { $inner } in '{ $expr }'

# Fehler E202: Liste, wo eine Zahl erwartet wird
# $expr: Ausdruck | Beispiel: tabelle
diag-asm-e202-list-for-number = Liste statt Zahl in '{ $expr }' (Elemente mit [i] wählen oder .byte/.word für die ganze Liste)

# Fehler E202: .for … in ohne Liste
# $expr: Ausdruck | Beispiel: 5
diag-asm-e202-for-in-not-list = .for … in erwartet eine Liste oder range(…), nicht '{ $expr }'

# Fehler E202: Operator für Liste nicht erlaubt (Teil von E202)
# $op: Operator | Beispiel: +
diag-asm-e202-list-not-allowed = '{ $op }' ist für Listen nicht erlaubt

# Fehler E202: Ergebnis ist keine endliche Zahl (Teil von E202)
diag-asm-e202-not-finite = Ergebnis ist keine endliche Zahl

# Fehler E202: Operator nur für ganze Zahlen (Teil von E202)
# $op: Operator | Beispiel: %
diag-asm-e202-integer-only = '{ $op }' ist nur für ganze Zahlen erlaubt

# Fehler E202: ungültige Hex-Zahl (Teil von E202)
diag-asm-e202-bad-hex = ungültige Hex-Zahl

# Fehler E202: Zahl zu groß (Teil von E202)
diag-asm-e202-number-too-large = Zahl zu groß

# Fehler E202: ungültige Kommazahl (Teil von E202)
diag-asm-e202-bad-float = ungültige Kommazahl

# Fehler E202: ungültiges Zeichenliteral (Teil von E202)
diag-asm-e202-bad-char-literal = ungültiges Zeichenliteral

# Fehler E202: unerwartetes Zeichen (Teil von E202)
# $ch: Zeichen | Beispiel: #
diag-asm-e202-unexpected-char = unerwartetes Zeichen '{ $ch }'

# Fehler E202: Klammer zu fehlt bei Funktionsaufruf (Teil von E202)
# $name: Funktionsname | Beispiel: min
diag-asm-e202-call-paren-missing = ')' fehlt bei { $name }(…)

# Fehler E202: falsche Anzahl Argumente (Teil von E202)
# $name: Funktionsname | Beispiel: atan2
# $expected: erwartete Anzahl der Argumente | Beispiel: 2
# $got: Anzahl der übergebenen Argumente | Beispiel: 1
diag-asm-e202-fn-arity = { $name }() erwartet { $expected } Argument(e), erhalten { $got }

# Fehler E202: zu wenige Argumente (Teil von E202)
# $name: Funktionsname | Beispiel: min
# $expected: Mindestzahl der Argumente | Beispiel: 1
# $got: Anzahl der übergebenen Argumente | Beispiel: 1
diag-asm-e202-fn-arity-min = { $name }() erwartet mindestens { $expected } Argument(e), erhalten { $got }

# Fehler E202: Funktion ruft sich selbst auf (Teil von E202)
# $name: Funktionsname | Beispiel: f
diag-asm-e202-fn-recursion = Funktion '{ $name }' ruft sich selbst auf

# Fehler E202: Funktionen zu tief verschachtelt (Teil von E202)
diag-asm-e202-fn-too-deep = Funktionen zu tief verschachtelt

# Fehler E202: Fehler innerhalb einer Funktion (Teil von E202)
# $inner: eingebettete Meldung | Beispiel: (Meldung)
# $name: Funktionsname | Beispiel: f
diag-asm-e202-in-function = { $inner } (in Funktion { $name })

# Fehler E202: len() ohne Liste (Teil von E202)
diag-asm-e202-len-needs-list = len() erwartet eine Liste

# Fehler E202: Funktion mit Liste statt Zahlen (Teil von E202)
# $name: Funktionsname | Beispiel: sin
diag-asm-e202-fn-needs-numbers = { $name }() erwartet Zahlen, keine Liste

# Fehler E202: range mit Schrittweite 0 (Teil von E202)
diag-asm-e202-range-step-zero = range: Schrittweite darf nicht 0 sein

# Fehler E202: range zu lang (Teil von E202)
diag-asm-e202-range-too-long = range: mehr als 65536 Elemente

# Fehler E202: Liste zu lang (Teil von E202)
diag-asm-e202-list-too-long = Liste zu lang (höchstens 65536 Elemente)

# Fehler E202: Operator für Listen nicht erlaubt (Teil von E202)
# $op: Operator | Beispiel: -
diag-asm-e202-list-op = '{ $op }' ist für Listen nicht erlaubt (nur Liste + Liste und Liste * Zahl)

# Fehler E202: Division durch 0 (Teil von E202)
diag-asm-e202-div-zero = Division durch 0

# Fehler E202: Index auf Nicht-Liste (Teil von E202)
diag-asm-e202-index-not-list = Index nur auf Listen erlaubt

# Fehler E202: eckige Klammer zu fehlt (Teil von E202)
diag-asm-e202-bracket-missing = ']' fehlt

# Fehler E202: runde Klammer zu fehlt (Teil von E202)
diag-asm-e202-paren-missing = ')' fehlt

# Fehler E202: Index fehlt (Teil von E202)
diag-asm-e202-index-missing = Index fehlt

# Fehler E202: Index außerhalb der Liste (Teil von E202)
# $index: Index | Beispiel: 5
# $len: Länge der Liste | Beispiel: 3
diag-asm-e202-index-out-of-range = Index { $index } außerhalb der Liste (Länge { $len })

# Fehler E202: Ausdruck unvollständig (Teil von E202)
diag-asm-e202-incomplete = Ausdruck unvollständig

# Fehler E202: next(n) mit n kleiner 1 (Teil von E202)
diag-asm-e202-next-zero = next(n): n muss größer als 0 sein

# Fehler E202: unbekanntes Symbol (Teil von E202)
# $name: Symbolname | Beispiel: schleife
diag-asm-e202-unknown-symbol = unbekanntes Symbol '{ $name }'

# Fehler E202: unerwarteter Operator (Teil von E202)
# $op: Operator | Beispiel: )
diag-asm-e202-unexpected-operator = unerwarteter Operator '{ $op }'

# Fehler E202: Ausdruck fehlt (Teil von E202)
diag-asm-e202-empty = Ausdruck fehlt

# Fehler E202: Zeichen nach dem Ausdruck (Teil von E202)
diag-asm-e202-trailing = unerwartete Zeichen am Ausdrucksende

# Fehler E203: Wert außerhalb des Bereichs
# $value: Wert | Beispiel: 300
# $lo: untere Grenze | Beispiel: -128
# $hi: obere Grenze | Beispiel: 255
diag-asm-e203-value-range = Wert { $value } außerhalb { $lo }..{ $hi }

# Fehler E203: Wert außerhalb 16 Bit
# $value: Wert | Beispiel: 70000
diag-asm-e203-value-16bit = Wert { $value } außerhalb 16 Bit

# Fehler E203: Wert außerhalb 32 Bit
# $value: Wert | Beispiel: 5000000000
diag-asm-e203-value-32bit = Wert { $value } außerhalb 32 Bit

# Fehler E203: .org außerhalb 0..65535
diag-asm-e203-org-range = .org außerhalb 0..65535

# Fehler E203: .bank außerhalb 0..1023
diag-asm-e203-bank-range = .bank außerhalb 0..1023

# Fehler E203: .segment: Anfang hinter dem Ende
diag-asm-e203-segment-order = .segment: Anfang liegt hinter dem Ende

# Fehler E203: .fill: ungültige Länge
diag-asm-e203-fill-length = .fill: ungültige Länge

# Fehler E203: .lohitable: ungültige Länge
diag-asm-e203-lohifill-length = .lohitable: ungültige Länge

# Fehler E203: .align: Größe kleiner 1
diag-asm-e203-page-size = .align: Größe muss größer als 0 sein

# Fehler E203: .align: Adresse außerhalb
diag-asm-e203-page-address = .align: Adresse außerhalb 0..65535

# Fehler E203: .incbin: Bereich außerhalb der Datei
diag-asm-e203-incbin-range = .incbin: Bereich außerhalb der Datei

# Fehler E203: .basicstub: Zeilennummer außerhalb
# $line: Zeilennummer | Beispiel: 70000
diag-asm-e203-basicstub-line = .basicstub: Zeilennummer { $line } außerhalb 0–63999

# Fehler E203: .basicstub: BASIC-Zeile zu lang
# $len: Länge der Zeile in Bytes | Beispiel: 230
diag-asm-e203-basicstub-too-long = .basicstub: BASIC-Zeile zu lang ({ $len } Bytes, höchstens 200)

# Fehler E203: Sprung zu weit
# $off: Sprungweite in Bytes | Beispiel: 200
diag-asm-e203-branch-range = Sprung zu weit ({ $off } Bytes)

# Fehler E203: Adresse außerhalb 0..65535
# $value: Adresse | Beispiel: 70000
diag-asm-e203-address-range = Adresse { $value } außerhalb 0..65535

# Fehler E203: .struct: ungültige Anzahl
# $field: Feldname | Beispiel: x
diag-asm-e203-struct-count = .struct: ungültige Anzahl bei '{ $field }'

# Fehler E203: .repeat: ungültige Anzahl
diag-asm-e203-repeat-count = .repeat: ungültige Anzahl

# Fehler E203: Fehler in einer Textdirektive (mit Direktive)
# $dir: Direktive | Beispiel: .ptext
# $reason: eingebettete Meldung | Beispiel: (Meldung)
diag-asm-e203-text = { $dir }: { $reason }

# Fehler E203: Text länger als 255 Zeichen (Teil von E203)
diag-asm-e203-text-too-long = Text länger als 255 Zeichen

# Fehler E203: leerer Text (Teil von E203)
diag-asm-e203-text-empty = leerer Text

# Fehler E204: Symbol mehrfach definiert
# $name: Name des Symbols | Beispiel: schleife
diag-asm-e204-symbol = Symbol '{ $name }' doppelt definiert

# Fehler E204: .struct/.enum mehrfach definiert
# $name: Name der Struktur | Beispiel: punkt
diag-asm-e204-struct = '{ $name }' doppelt definiert

# Fehler E204: Name einer eingebauten Funktion
# $name: Funktionsname | Beispiel: sin
diag-asm-e204-builtin-function = '{ $name }' ist eine eingebaute Funktion

# Fehler E204: Funktion mehrfach definiert
# $name: Funktionsname | Beispiel: quadrat
diag-asm-e204-function = Funktion '{ $name }' doppelt definiert

# Fehler E205: unbekannter Befehl
# $mn: Befehl (Mnemonic) | Beispiel: LDA
diag-asm-e205 = unbekannter Befehl '{ $mn }'

# Fehler E206: Befehl braucht einen Operanden
# $mn: Befehl (Mnemonic) | Beispiel: LDA
diag-asm-e206-operand-needed = { $mn } braucht einen Operanden

# Fehler E206: Adressierungsart nicht unterstützt
# $mn: Befehl (Mnemonic) | Beispiel: LDA
# $mode: Adressierungsart | Beispiel: Abs
diag-asm-e206-mode-unsupported = { $mn } unterstützt die Adressierungsart { $mode } nicht

# Fehler E206: Adressierungsart beim Q-Befehl nicht unterstützt
# $mn: Befehl (Mnemonic) mit Q | Beispiel: LDAQ
# $mode: Adressierungsart | Beispiel: Abs
diag-asm-e206-quad-mode = { $mn } unterstützt die Adressierungsart { $mode } nicht

# Fehler E206: 32-Bit-Zeiger auf dieser CPU nicht möglich
diag-asm-e206-ptr32-cpu = 32-Bit-Zeiger [..] gibt es auf dieser CPU nicht

# Fehler E206: ungültiger 32-Bit-Zeiger
diag-asm-e206-ptr32 = ungültiger 32-Bit-Zeiger

# Fehler E206: ($nn,sp) ohne ,y
diag-asm-e206-sp-needs-y = ({ "$" }nn,sp) braucht ,y dahinter

# Fehler E206: ungültiger indirekter Operand
diag-asm-e206-indirect = ungültiger indirekter Operand

# Fehler E206: ungültiger Operand
diag-asm-e206-operand = ungültiger Operand

# Fehler E207: Assertion fehlgeschlagen (ohne Text)
diag-asm-e207-assert = Assertion fehlgeschlagen

# Fehler E207: Assertion fehlgeschlagen (mit Text)
# $message: Text der Assertion | Beispiel: x muss kleiner 10 sein
diag-asm-e207-assert-message = Assertion fehlgeschlagen: { $message }

# Fehler E207: Fehler aus .error (Text vom Programmierer)
# $text: Text hinter .error | Beispiel: Speicher voll
diag-asm-e207-user = { $text }

# Fehler E208: .var/.set ohne NAME = Ausdruck
# $dir: Direktive | Beispiel: .var
diag-asm-e208-var-syntax = { $dir } erwartet NAME = Ausdruck

# Fehler E208: ungültiger Variablenname
# $dir: Direktive | Beispiel: .var
# $name: Variablenname | Beispiel: 9x
diag-asm-e208-var-name = { $dir }: ungültiger Variablenname '{ $name }'

# Fehler E208: Zeichenkette erwartet
diag-asm-e208-string-expected = Zeichenkette in Anführungszeichen erwartet

# Fehler E208: Zeichenketten erwartet
# $dir: Direktive | Beispiel: .text
diag-asm-e208-strings-expected = { $dir } erwartet Zeichenketten in Anführungszeichen

# Fehler E208: .encoding ohne Namen
diag-asm-e208-encoding-name = .encoding erwartet einen Namen in Anführungszeichen

# Fehler E208: unbekannte Kodierung
# $name: Name der Kodierung oder Datei | Beispiel: latin1
diag-asm-e208-encoding-unknown = .encoding: unbekannte Kodierung oder Datei '{ $name }'

# Fehler E208: Kodiertabelle hat nicht 256 Byte
# $name: Dateiname der Tabelle | Beispiel: tabelle.bin
diag-asm-e208-encoding-size = .encoding: '{ $name }' muss genau 256 Byte enthalten

# Fehler E208: .segment ohne Namen
diag-asm-e208-segment-name = .segment erwartet einen Namen in Anführungszeichen

# Fehler E208: .cartridge ohne Typ
diag-asm-e208-cartridge-type = .cartridge erwartet den Typ in Anführungszeichen (z. B. "ocean")

# Fehler E208: .hex ohne Zeichenkette
diag-asm-e208-hex-quoted = .hex erwartet Hexziffern in Anführungszeichen

# Fehler E208: .hex mit ungerader Ziffernzahl
diag-asm-e208-hex-even = .hex: gerade Anzahl Hexziffern erwartet

# Fehler E208: .lohitable ohne Ausdruck
diag-asm-e208-lohifill-syntax = .lohitable erwartet: .lohitable anzahl, ausdruck(i)

# Fehler E208: .incbin ohne Dateinamen
diag-asm-e208-incbin-name = .incbin erwartet einen Dateinamen

# Fehler E208: .const ohne NAME = Ausdruck
diag-asm-e208-const-syntax = .const erwartet NAME = Ausdruck

# Fehler E208: .branchlong mit falschem Wert
diag-asm-e208-branchlong = .branchlong erwartet auto, jmp oder off

# Fehler E208: unbekannter Wert für illegals
# $value: Wert | Beispiel: viel
diag-asm-e208-illegals-value = illegals: unbekannter Wert '{ $value }'

# Fehler E208: unbekannte Direktive
# $name: Direktive | Beispiel: .foo
diag-asm-e208-unknown-directive = unbekannte Direktive '{ $name }'

# Fehler E208: Zuweisung erwartet
# $text: Text | Beispiel: i < 8
diag-asm-e208-assignment-expected = Zuweisung erwartet statt '{ $text }'

# Fehler E208: ungültiger Name bei .struct/.enum
# $dir: Direktive | Beispiel: .struct
# $name: Name | Beispiel: 9x
diag-asm-e208-struct-name = { $dir }: ungültiger Name '{ $name }'

# Fehler E208: .enum: ungültiger Name
# $name: Name des Eintrags | Beispiel: 9x
diag-asm-e208-enum-item-name = .enum: ungültiger Name '{ $name }'

# Fehler E208: .struct: Feld ohne Typ
# $text: Text der Zeile | Beispiel: x
diag-asm-e208-struct-field-syntax = .struct: Feld erwartet 'name typ [anzahl]', gefunden '{ $text }'

# Fehler E208: .struct: ungültiger Feldname
# $field: Feldname | Beispiel: 9x
diag-asm-e208-struct-field-name = .struct: ungültiger Feldname '{ $field }'

# Fehler E208: .struct: Feldtyp ist eine Aufzählung
# $ty: Name der Aufzählung | Beispiel: farbe
diag-asm-e208-struct-enum-field = .struct: '{ $ty }' ist eine Aufzählung; für das Feld .byte (oder .word) verwenden

# Fehler E208: .struct: unbekannter Feldtyp
# $ty: Feldtyp | Beispiel: float
diag-asm-e208-struct-field-type = .struct: unbekannter Feldtyp '{ $ty }' (.byte, .word, .dword oder eine früher definierte Struktur)

# Fehler E208: .function falsch aufgebaut
diag-asm-e208-function-syntax = .function erwartet: .function name(param, …) = ausdruck

# Fehler E208: .function: ungültiger Name
# $name: Funktionsname | Beispiel: 9f
diag-asm-e208-function-name = .function: ungültiger Name '{ $name }'

# Fehler E208: ungültiger Name hinter as beim Include
# $first: Direktive | Beispiel: .include
# $name: Name hinter as | Beispiel: 9x
diag-asm-e208-include-alias = { $first } ... as: ungültiger Name '{ $name }' (nur Buchstaben, Ziffern und _)

# Fehler E208: Include ohne Dateinamen
# $first: Direktive | Beispiel: .include
diag-asm-e208-include-name = { $first } erwartet einen Dateinamen in Anführungszeichen oder <spitzen Klammern>

# Fehler E208: .for … in ohne Liste
diag-asm-e208-for-in-list = .for … in erwartet eine Liste oder range(…)

# Fehler E208: .for falsch aufgebaut
diag-asm-e208-for-syntax = .for erwartet: .for anfang, bedingung, schritt (z. B. .for i = 0, i < 8, i += 1)

# Fehler E208: Klammer in Zeichenkette nicht geschlossen
diag-asm-e208-token-unclosed = '{ "{" }' ohne schließendes '{ "}" }' (für ein einzelnes '{ "{" }' schreibe '{ "{{" }')

# Fehler E208: Wiederholung im Sonderzeichen ungültig
# $token: Sonderzeichen in Klammern | Beispiel: {down*0}
diag-asm-e208-token-repeat = { $token }: Wiederholung 1 bis 255 erwartet

# Fehler E208: ungültiger Hexwert im Sonderzeichen
# $token: Sonderzeichen in Klammern | Beispiel: {$zz}
diag-asm-e208-token-hex = { $token }: ungültiger Hexwert (00 bis ff)

# Fehler E208: ungültige Zahl im Sonderzeichen
# $token: Sonderzeichen in Klammern | Beispiel: {300}
diag-asm-e208-token-number = { $token }: Zahl 0 bis 255 erwartet

# Fehler E208: Name im Sonderzeichen nicht erlaubt
# $token: Sonderzeichen in Klammern | Beispiel: {down}
diag-asm-e208-token-names = { $token }: Namen gibt es nur mit .petscii/.atascii oder .encoding "petscii"/"atascii" (hier nur { "{" }{ "$" }hh{ "}" } und { "{" }nnn{ "}" })

# Fehler E208: unbekanntes Sonderzeichen
# $token: Sonderzeichen in Klammern | Beispiel: {nix}
diag-asm-e208-token-unknown = unbekanntes Sonderzeichen { $token }

# Fehler E209: Makroname wie ein Befehl
# $name: Makroname | Beispiel: LDA
diag-asm-e209-macro-name = Makroname '{ $name }' kollidiert mit einem Befehl

# Fehler E209: falsche Anzahl Argumente beim Makroaufruf
# $name: Makroname | Beispiel: FARBE
# $expected: erwartete Anzahl der Argumente | Beispiel: 2
# $got: Anzahl der übergebenen Argumente | Beispiel: 1
diag-asm-e209-macro-args = Makro '{ $name }' erwartet { $expected } Argument(e), erhalten { $got }

# Fehler E209: Anzahl Argumente beim Makroaufruf außerhalb des Bereichs
# $name: Makroname | Beispiel: FARBE
# $min: Mindestwert | Beispiel: 1
# $max: größte Anzahl der Argumente | Beispiel: 3
# $got: Anzahl der übergebenen Argumente | Beispiel: 1
diag-asm-e209-macro-args-range = Makro '{ $name }' erwartet { $min } bis { $max } Argument(e), erhalten { $got }

# Fehler E210: .break/.continue außerhalb einer Schleife
# $directive: Direktive | Beispiel: .break
diag-asm-e210-outside-loop = { $directive } außerhalb einer Schleife

# Fehler E210: .endcycles ohne .cycles
diag-asm-e210-endcycles = .endcycles ohne .cycles

# Fehler E210: .exitmacro außerhalb eines Makros
diag-asm-e210-exitmacro = .exitmacro außerhalb eines Makros

# Fehler E210: Makroparameter ohne Standardwert hinter einem mit
# $param: Parametername | Beispiel: farbe
diag-asm-e210-macro-param-default = Parameter '{ $param }' ohne Standardwert steht hinter einem Parameter mit Standardwert

# Fehler E210: .macro ohne .endmacro
diag-asm-e210-macro-unclosed = .macro ohne .endmacro

# Fehler E210: Makroname fehlt
diag-asm-e210-macro-name-missing = Makroname fehlt

# Fehler E210: .if ohne .endif
diag-asm-e210-if-unclosed = .if ohne .endif

# Fehler E210: .repeat ohne .endrepeat
diag-asm-e210-repeat-unclosed = .repeat ohne .endrepeat

# Fehler E210: .struct/.enum ohne Ende
# $directive: Direktive | Beispiel: .struct
# $end: erwartetes Ende | Beispiel: .endstruct
diag-asm-e210-block-unclosed = { $directive } ohne { $end }

# Fehler E210: .for ohne .endfor
diag-asm-e210-for-unclosed = .for ohne .endfor

# Fehler E210: Schlussdirektive ohne Beginn
# $directive: Direktive | Beispiel: .endif
diag-asm-e210-unexpected = unerwartetes { $directive }

# Fehler E210: .cycles ohne .endcycles
# $name: Name des Blocks | Beispiel: irq
diag-asm-e210-cycles-unclosed = .cycles '{ $name }' ohne .endcycles

# Fehler E211: Include zu tief verschachtelt
diag-asm-e211-include-depth = Include zu tief verschachtelt (Zyklus?)

# Fehler E211: Makro zu tief verschachtelt
# $name: Makroname | Beispiel: FARBE
diag-asm-e211-macro-depth = Makro '{ $name }' zu tief verschachtelt

# Fehler E212: Include-Datei nicht gefunden
# $name: Dateiname | Beispiel: lib.inc
diag-asm-e212-include-missing = Include-Datei '{ $name }' nicht gefunden

# Fehler E212: .incbin-Datei nicht gefunden
# $name: Dateiname | Beispiel: bild.bin
diag-asm-e212-incbin-missing = Datei '{ $name }' nicht gefunden

# Fehler E212: Datei nicht lesbar (Systemfehler)
# $path: Dateipfad | Beispiel: daten/bild.bin
# $error: Fehlertext des Betriebssystems | Beispiel: (Systemtext)
diag-asm-e212-file-error = { $path }: { $error }

# Fehler E213: Scope-Name mit Punkt
# $name: Name des Scopes | Beispiel: a.b
diag-asm-e213-scope-name-dot = Scope-Name '{ $name }' darf keinen Punkt enthalten

# Fehler E213: } ohne {
diag-asm-e213-unmatched-close = '{ "}" }' ohne öffnendes '{ "{" }'

# Fehler E213: Text hinter {
diag-asm-e213-after-open = nach '{ "{" }' darf nichts in derselben Zeile stehen

# Fehler E213: .namespace falsch aufgebaut
diag-asm-e213-namespace-syntax = .namespace erwartet: .namespace NAME { "{" }

# Fehler E213: Scope nicht geschlossen
# $name: Pfad des Scopes | Beispiel: init.schleife
diag-asm-e213-scope-unclosed = Scope '{ $name }' wird nicht mit '{ "}" }' geschlossen

# Fehler E214: kein passendes anonymes Label davor
# $label: Label-Schreibweise | Beispiel: --
diag-asm-e214-before = kein passendes '{ $label }'-Label davor

# Fehler E214: kein passendes anonymes Label dahinter
# $label: Label-Schreibweise | Beispiel: --
diag-asm-e214-after = kein passendes '{ $label }'-Label dahinter

# Fehler E215: Suffix .z/.a an falscher Operandenform
# $suffix: Suffix | Beispiel: z
diag-asm-e215-suffix = Suffix .{ $suffix } gilt nur für Zeropage- und absolute Adressierung

# Fehler E215: Suffix an Sprungbefehl
# $mn: Befehl (Mnemonic) | Beispiel: BNE
# $suffix: Suffix | Beispiel: z
diag-asm-e215-form-branch = { $mn }.{ $suffix }: Sprungbefehle haben nur eine Form

# Fehler E215: Befehl hat keine Zeropage-Form
# $mn: Befehl (Mnemonic) | Beispiel: LDA
# $suffix: Suffix | Beispiel: z
diag-asm-e215-form-zeropage = { $mn }.{ $suffix }: keine Zeropage-Form

# Fehler E215: Befehl hat keine absolute Form
# $mn: Befehl (Mnemonic) | Beispiel: LDA
# $suffix: Suffix | Beispiel: a
diag-asm-e215-form-absolute = { $mn }.{ $suffix }: keine absolute Form

# Fehler E216: Variable nicht mit .var angelegt
# $name: Variablenname | Beispiel: i
diag-asm-e216 = Variable '{ $name }' ist nicht mit .var angelegt

# Fehler E217: zu viele Schleifendurchläufe
# $limit: Grenze | Beispiel: 65536
diag-asm-e217 = .for: mehr als { $limit } Durchläufe

# Fehler E218: .basicstub nicht am Programmanfang
diag-asm-e218 = .basicstub muss am Anfang des Programms stehen (davor wurden schon Bytes ausgegeben)

# Fehler E300: Code vor .org
diag-asm-e300-code-before-org = Code vor .org

# Fehler E300: Label vor .org
diag-asm-e300-label-before-org = Label vor .org

# Fehler E300: .relocate vor .org
diag-asm-e300-pseudopc-before-org = .relocate vor .org

# Fehler E300: Symbole konvergieren nicht
diag-asm-e300-no-convergence = Symbole konvergieren nicht (zu viele Durchläufe)

# Fehler E301: Segment verlässt seinen Bereich
# $name: Name des Segments | Beispiel: CODE
# $lo: Anfang des Bereichs (hex ohne $) | Beispiel: 8000
# $hi: Ende des Bereichs (hex ohne $) | Beispiel: 8003
# $from: Anfangsadresse (hex ohne $) | Beispiel: 8000
# $to: Endadresse (hex ohne $) | Beispiel: 8004
diag-asm-e301-segment-range = Segment '{ $name }' verlässt seinen Bereich ${ $lo }-${ $hi } (${ $from }..${ $to })

# Fehler E301: Segmente überlappen
# $a: Anfang des ersten Segments (hex ohne $) | Beispiel: 8000
# $b: Anfang des zweiten Segments (hex ohne $) | Beispiel: 8001
diag-asm-e301-overlap = Segmente überlappen: ${ $a } und ${ $b }

# Fehler E301: Segmente überlappen in einer Bank
# $bank: Banknummer | Beispiel: 2
# $a: Anfang des ersten Segments (hex ohne $) | Beispiel: 8000
# $b: Anfang des zweiten Segments (hex ohne $) | Beispiel: 8001
diag-asm-e301-overlap-bank = Segmente überlappen (Bank { $bank }): ${ $a } und ${ $b }

# Fehler E301: Segment überschreitet $FFFF
# $start: Anfang des Segments (hex ohne $) | Beispiel: FFF0
diag-asm-e301-beyond-ffff = Segment bei ${ $start } überschreitet { "$" }FFFF

# Warnung W104: undokumentierter Opcode (illegals=warn)
# $mn: Befehl (Mnemonic) | Beispiel: LDA
diag-asm-w104 = { $mn } ist ein undokumentierter Opcode (illegals=warn)

# Warnung W105: instabiler undokumentierter Opcode
# $mn: Befehl (Mnemonic) | Beispiel: LDA
diag-asm-w105 = { $mn } ist ein instabiler undokumentierter Opcode

# Warnung W200: Warnung aus .warning (Text vom Programmierer)
# $text: Text hinter .warning | Beispiel: Tabelle fast voll
diag-asm-w200 = { $text }

# Warnung W201: Zyklenbudget überschritten
# $name: Name des Blocks | Beispiel: irq
# $max: Zyklen im ungünstigsten Fall | Beispiel: 120
# $limit: Budget in Zyklen | Beispiel: 100
diag-asm-w201 = Zyklenbudget '{ $name }' überschritten: worst-case { $max } > { $limit } Zyklen

# Warnung W202: Kommazahl abgeschnitten
# $value: Kommazahl | Beispiel: 3.5
# $to: ganze Zahl nach dem Abschneiden | Beispiel: 3
# $expr: Ausdruck | Beispiel: 7 / 2
diag-asm-w202 = Kommazahl { $value } auf { $to } abgeschnitten (int(), round() oder floor() verwenden) in '{ $expr }'

# Warnung W203: Sprung zu weit, lange Form verwendet
# $mn: Befehl (Mnemonic) | Beispiel: LDA
# $len: Länge der langen Form in Bytes | Beispiel: 5
diag-asm-w203 = Sprung zu weit für { $mn }: lange Form ({ $len } Bytes) verwendet

# Warnung W204: .basicstub nicht an der BASIC-Startadresse
# $pc: Adresse (hex ohne $) | Beispiel: 1000
# $start: BASIC-Startadresse (hex ohne $) | Beispiel: 0801
diag-asm-w204 = .basicstub steht bei ${ $pc }, die BASIC-Startadresse des Zielsystems ist ${ $start } (BASIC findet die Zeile dort nicht)

# Warnung W205: .basicstub auf einem Ziel ohne BASIC-Zeile
diag-asm-w205 = .basicstub hat auf diesem Ziel keine Wirkung (kein BASIC mit SYS); für den Start .run label verwenden

# Warnung W206: .run auf einem Ziel ohne Startvektor
diag-asm-w206 = .run hat auf diesem Ziel keine Wirkung (PRG hat keinen Startvektor); für den Start .basicstub label verwenden

# Warnung W207: Programm ohne Startvektor
diag-asm-w207 = Kein Startvektor festgelegt; mit .run label ergänzen

# --- Meldungen ohne Code: Bau der Modulformate und Ausgabe (to_prg, to_crt, to_car, …); sie sind keine Diagnostic

# Ausgabe leer
asm-out-empty = keine Ausgabe

# mehrere Bänke im PRG/BIN/XEX
asm-out-banks = mehrere Bänke (.bank) gibt es nur in den Modulformaten crt/car/rom

# Segment liegt nicht im Fenster ($bank, $from, $to, $lo, $hi)
asm-cart-segment-window = Bank { $bank }: Segment ${ $from }..${ $to } liegt nicht im Fenster ${ $lo }-${ $hi }

# Standardmodul: mehrere Bänke
asm-cart-normal-bank = Standardmodul: nur Bank 0 (für mehrere Bänke ocean oder magicdesk wählen)

# Standardmodul: kein Code im Fenster
asm-cart-normal-window = Standardmodul: Code muss in $8000-$BFFF liegen

# Standardmodul: Code außerhalb
asm-cart-normal-outside = Standardmodul: Code außerhalb $8000-$BFFF

# Ocean/Magic Desk: zu viele Bänke ($kind, $max)
asm-cart-max-banks = { $kind }: höchstens { $max } Bänke (8 KB)

# EasyFlash: zu viele Bänke
asm-cart-easyflash-max-banks = easyflash: höchstens 64 Bänke (je 8 KB ROML + 8 KB ROMH)

# EasyFlash: ROMH doppelt belegt ($bank)
asm-cart-easyflash-romh-twice = easyflash: Bank { $bank } belegt ROMH sowohl bei $A000 als auch bei $E000

# EasyFlash: Bank 0 ohne ROMH
asm-cart-easyflash-bank0-romh = easyflash: Bank 0 braucht ROMH-Code (Startvektoren bei $FFFC im Fenster $E000-$FFFF)

# Action Replay: zu viele Bänke
asm-cart-actionreplay-banks = actionreplay: genau 4 Bänke (je 8 KB bei $8000)

# Ultimax: mehrere Bänke
asm-cart-ultimax-bank = ultimax: nur Bank 0

# Ultimax: ROMH fehlt
asm-cart-ultimax-romh = ultimax: ROMH-Code im Fenster $E000-$FFFF mit Reset-Vektor bei $FFFC fehlt

# Ultimax: Code außerhalb
asm-cart-ultimax-outside = ultimax: Code nur in $8000-$9FFF oder $E000-$FFFF

# C64-Modultyp unbekannt ($kind)
asm-cart-unknown-type-c64 = unbekannter Modultyp '{ $kind }' (normal, 16k, ocean, magicdesk, easyflash, actionreplay, ultimax)

# Atari-Modul: Startadresse fehlt
asm-cart-atari-run = Atari-Modul: Startadresse fehlt (.run oder eigene Vektoren bei $BFFA)

# Atari-Modul: mehrere Bänke
asm-cart-atari-banks = mehrere Bänke: Modultyp xegs wählen

# Atari-Modul: Code außerhalb ($lo)
asm-cart-atari-outside = Atari-Modul: Code außerhalb ${ $lo }-$BFFF

# Atari-Modultyp unbekannt ($kind)
asm-cart-unknown-type-atari = unbekannter Modultyp '{ $kind }' (8k, 16k, xegs)

# XEGS: zu viele Bänke
asm-cart-xegs-banks = XEGS: höchstens 16 Bänke (128 KB)

# VC-20-Modultyp unbekannt ($kind)
asm-cart-unknown-type-vic20 = unbekannter VC-20-Modultyp '{ $kind }' (a0, 20, 40, 60)

# VC-20-Modul: Bänke
asm-cart-vic20-banks = VC-20-Modul: keine Bänke (.bank) möglich

# VC-20-Modul: kein Code im Fenster ($from, $to)
asm-cart-vic20-window = VC-20-Modul: Code muss in ${ $from }-${ $to } liegen

# VC-20-Modul: Code außerhalb ($from, $to)
asm-cart-vic20-outside = VC-20-Modul: Code außerhalb ${ $from }-${ $to }

# VC-20-Modul bei $A000 ohne Kennung
asm-cart-vic20-id = VC-20-Modul bei $A000: Kennung A0CBM bei $A004 fehlt (.byte $41,$30,$c3,$c2,$cd; davor die Start-Vektoren .word kalt, warm)

# Fehler E219: Medium (Sprite, Zeichensatz, Song …) lässt sich nicht einbinden
# $error: Meldung des Exporters | Beispiel: Medium graphics/hero.vcsprite ist für c64, das Ziel des Baus ist atari8
diag-asm-e219-media = { $error }

# Warnung W208: alter Include-Pfad auf eine Exportdatei im Build-Ordner
# $path: Pfad im Include | Beispiel: build/export/hero.asm
# $media: Mediendatei im Projekt | Beispiel: graphics/hero.vcsprite
diag-asm-w208-legacy-export = Veralteter Pfad '{ $path }': das Medium direkt einbinden, z. B. .include "{ $media }"
