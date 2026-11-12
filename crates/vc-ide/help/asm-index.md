# Assembler: Spickzettel und Fehlercodes

Kurzfassung zum Nachschlagen. Die ausführliche Erklärung steht auf den Seiten [Assembler](help:assembler), [Ausdrücke und Funktionen](help:asm-expr), [Direktiven](help:asm-directives), [Texte und Zeichensätze](help:asm-text), [Labels, Makros, Scopes](help:asm-macros) und [Systembibliothek](help:asm-library).

## Aufbau einer Zeile

```
label:  befehl operand      ; Kommentar
        .direktive argumente
NAME =  ausdruck            ; Konstante (oder Variable, wenn mit .var angelegt)
*=      $c000               ; Programmzähler setzen (wie .org)
```

## Zahlen

| Schreibweise | Bedeutung |
|---|---|
| `255` `$ff` `0xff` `%11111111` | dezimal, hexadezimal, hexadezimal, binär |
| `'a'` | ASCII-Wert eines Zeichens |
| `2.5` `1e3` | Kommazahl (nur zum Rechnen, siehe [Ausdrücke](help:asm-expr)) |
| `<x` `>x` | Low-Byte, High-Byte von x |
| `*` | aktueller Programmzähler |

## Adressierungsarten (Kurzform)

| Schreibweise | Art | Beispiel |
|---|---|---|
| `#wert` | unmittelbar | `lda #$00` |
| `$12` / `$1234` | Zeropage / absolut (der Assembler wählt die kürzeste Form) | `lda $d020` |
| `$12,x` `$12,y` `$1234,x` `$1234,y` | indiziert | `sta $0400,x` |
| `($12,x)` | indirekt, vor-indiziert | `lda ($fb,x)` |
| `($12),y` | indirekt, nach-indiziert | `lda ($fb),y` |
| `($1234)` | indirekt (nur `jmp`) | `jmp ($0314)` |
| `a` | Akkumulator | `asl a` |
| `lda.z` `lda.a` | Form erzwingen (Zeropage / absolut) | `lda.a $10` |

## Direktiven von A bis Z

| Direktive | Kurzbeschreibung | Seite |
|---|---|---|
| `.align n` | mit Nullbytes auf ein Vielfaches von n auffüllen | [Direktiven](help:asm-directives) |
| `.assert bed[, "text"]` | Bedingung prüfen, sonst E207 | [Direktiven](help:asm-directives) |
| `.atascii "…"` | Text als ATASCII | [Texte](help:asm-text) |
| `.bank n` | ROM-Bank wählen (0–1023) | [Module](help:cartridges) |
| `.basicstub ziel[, zeile][, "text"]` | BASIC-Zeile `SYS` voranstellen | [Direktiven](help:asm-directives) |
| `.be16` `.dbyt` | 16 Bit, High-Byte zuerst | [Direktiven](help:asm-directives) |
| `.branchlong auto/off` | zu weite Sprünge automatisch verlängern | [Direktiven](help:asm-directives) |
| `.break` | Schleife verlassen | [Direktiven](help:asm-directives) |
| `.byte` `.db` | Bytes und Zeichenketten | [Direktiven](help:asm-directives) |
| `.cartridge "typ", "NAME"` | Modul statt Programm erzeugen | [Module](help:cartridges) |
| `.const` `.equ` | Konstante: `.const NAME = ausdruck` | [Direktiven](help:asm-directives) |
| `.continue` | nächster Schleifendurchlauf | [Direktiven](help:asm-directives) |
| `.cpu 6510 { illegals = … }` | Umgang mit undokumentierten Befehlen | [Direktiven](help:asm-directives) |
| `.cycles [name] [max=n]` … `.endcycles` | Zyklen eines Blocks zählen | [Direktiven](help:asm-directives) |
| `.dword` `.long` | 32 Bit, Low-Byte zuerst | [Direktiven](help:asm-directives) |
| `.encoding "name"` | Textkodierung wählen (auch als Block `{ … }`) | [Texte](help:asm-text) |
| `.enum` … `.endenum` | Aufzählung | [Makros](help:asm-macros) |
| `.error "text"` | Fehler E207 erzeugen | [Direktiven](help:asm-directives) |
| `.exitmacro` | laufendes Makro vorzeitig beenden | [Makros](help:asm-macros) |
| `.fill n[, ausdr[, var]]` | n Bytes füllen / Tabelle | [Direktiven](help:asm-directives) |
| `.for` … `.next` | Schleife | [Direktiven](help:asm-directives) |
| `.function name(p) = ausdr` | eigene Funktion für Ausdrücke | [Ausdrücke](help:asm-expr) |
| `.hex "aa bb"` | Bytes aus Hexziffern | [Direktiven](help:asm-directives) |
| `.if` `.elseif` `.else` `.endif` | bedingte Übersetzung | [Direktiven](help:asm-directives) |
| `.ifdef` `.ifndef name` | Name definiert / nicht definiert | [Direktiven](help:asm-directives) |
| `.include "datei"` `#include <pfad>` | Datei einbinden (`as name`: eigener Namensraum) | [Direktiven](help:asm-directives) |
| `.incbin "datei"[, off[, len]]` | Binärdatei einbinden | [Direktiven](help:asm-directives) |
| `.lohifill n, ausdr[, var]` | Wortetabelle: erst Low-, dann High-Bytes | [Direktiven](help:asm-directives) |
| `.macro` … `.endmacro` | Makro definieren | [Makros](help:asm-macros) |
| `.namespace name {` … `}` | benannter Namensraum | [Makros](help:asm-macros) |
| `.null "…"` | Text mit Nullbyte am Ende | [Texte](help:asm-text) |
| `.org adr` / `*= adr` | Adresse setzen, neues Segment | [Direktiven](help:asm-directives) |
| `.page [n]` | an die nächste Seitengrenze, ohne Füllbytes | [Direktiven](help:asm-directives) |
| `.petscii "…"` | Text als PETSCII | [Texte](help:asm-text) |
| `.ptext "…"` | Text mit Längenbyte davor | [Texte](help:asm-text) |
| `.pseudopc adr` … `.endpseudopc` (`.realpc`) | Code für eine andere Laufzeitadresse | [Direktiven](help:asm-directives) |
| `.repeat n[, var]` … `.endrepeat` | Block n-mal | [Direktiven](help:asm-directives) |
| `.rta ziel` | Adresse minus 1 (für `rts`-Sprungtabellen) | [Direktiven](help:asm-directives) |
| `.run adr` | Startadresse (Atari) | [Direktiven](help:asm-directives) |
| `.screen "…"` | Text als Bildschirmcode | [Texte](help:asm-text) |
| `.segment "NAME"[, von, bis]` | Segment benennen und Bereich prüfen | [Module](help:cartridges) |
| `.set name = ausdr` | Variable ändern | [Direktiven](help:asm-directives) |
| `.shift "…"` | Text, letztes Zeichen mit Bit 7 | [Texte](help:asm-text) |
| `.struct` … `.endstruct` | Struktur mit Feldnamen | [Makros](help:asm-macros) |
| `.target name` | Zielsystem im Quelltext vermerken | [Direktiven](help:asm-directives) |
| `.text "…"` | Text in der Kodierung von `.encoding` | [Texte](help:asm-text) |
| `.var name = ausdr` | Variable anlegen | [Direktiven](help:asm-directives) |
| `.warning "text"` | Warnung W200 erzeugen | [Direktiven](help:asm-directives) |
| `.word` `.dw` | 16-Bit-Werte, Low-Byte zuerst | [Direktiven](help:asm-directives) |
| `{` … `}` / `name: {` … `}` | Scope (Namensraum) | [Makros](help:asm-macros) |

## Operatoren

Vom stärksten zum schwächsten Vorrang: Vorzeichen `- + ! ~ < >` vor einem Wert · `* / %` · `+ -` · `<< >>` · `&` · `^` · bitweises Oder · `< <= > >=` · `== !=` · `&&` · `||`. Vergleiche liefern 1 oder 0. Die Details stehen unter [Ausdrücke](help:asm-expr).

## Funktionen

`sin cos tan asin acos atan atan2 sqrt pow exp log log2 log10 int floor ceil round abs sign min max len range next align` und `pi`. Eigene mit `.function`. Tabelle unter [Ausdrücke](help:asm-expr).

## Fehlercodes (E) und Warnungen (W)

Fehler brechen den Bau ab, Warnungen nicht. Die Meldung nennt Datei, Zeile und Code.

| Code | Bedeutung | Typische Ursache |
|---|---|---|
| **E201** | undokumentierter Opcode nicht erlaubt, oder Befehl existiert auf dieser CPU nicht | `lax` bei `illegals = deny`; Befehl einer anderen CPU |
| **E202** | Fehler in einem Ausdruck | unbekanntes Symbol, fehlende Klammer, Division durch 0, falsche Argumentzahl einer Funktion, Liste statt Zahl |
| **E203** | Wert außerhalb des Bereichs | `lda #256`, `.org` außerhalb 0–65535, Sprung zu weit, `.incbin` über das Dateiende |
| **E204** | Name doppelt definiert | zwei gleiche Labels, Konstante mit neuem Wert, Label in `.repeat` ohne Scope |
| **E205** | unbekannter Befehl | Tippfehler, Label ohne Doppelpunkt, Makro nicht definiert |
| **E206** | Operand passt nicht zum Befehl | `lda` ohne Operand, `sta #1`, Adressierungsart nicht vorhanden |
| **E207** | `.assert` fehlgeschlagen oder `.error` erreicht | eigene Prüfungen |
| **E208** | falsche Schreibweise einer Direktive | fehlende Anführungszeichen, ungültiger Name, unbekannte Direktive, unbekanntes Sonderzeichen `{…}` |
| **E209** | Makro: falsche Argumentzahl oder Name wie ein Befehl | `m 1, 2` bei einem Parameter |
| **E210** | Blockstruktur fehlerhaft | `.if` ohne `.endif`, `.macro` ohne `.endmacro`, `.endcycles` ohne `.cycles`, `.break` außerhalb einer Schleife |
| **E211** | zu tief verschachtelt | Makro ruft sich selbst auf, Include-Zyklus |
| **E212** | Datei nicht gefunden | falscher Pfad bei `.include`, `.incbin`, `.encoding` |
| **E213** | Scope-Fehler | `}` ohne `{`, Scope nicht geschlossen, Punkt im Scope-Namen |
| **E214** | anonymes Label fehlt | `bne -` ohne `-` davor, `beq +` ohne `+` dahinter |
| **E215** | Adressmodus-Suffix passt nicht | `jmp.z`, `lda.z #1` |
| **E216** | `.set` auf etwas, das keine Variable ist | `.set` auf eine Konstante |
| **E217** | Schleife läuft zu lange | `.for` mit mehr als 65536 Durchläufen |
| **E218** | `.basicstub` nicht am Programmanfang | Bytes vor `.basicstub` |
| **E300** | Code oder Label vor `.org`; Symbole konvergieren nicht | `nop` in der ersten Zeile; Bedingung hängt von einer eigenen Adresse ab |
| **E301** | Segment überlappt oder verlässt seinen Bereich | zwei `.org` mit überlappendem Inhalt, `.segment` mit zu kleinem Bereich |
| **W104** | undokumentierter Opcode (illegals = warn) | `lax $d020` |
| **W105** | instabiler undokumentierter Opcode | `xaa` |
| **W200** | `.warning` erreicht | eigene Meldung |
| **W201** | Zyklenbudget überschritten | `.cycles … max=n` |
| **W202** | Kommazahl wurde abgeschnitten | `.byte 7 / 2.0` |
| **W203** | zu weiter Sprung automatisch verlängert | `.branchlong auto` |
| **W204** | `.basicstub` nicht an der BASIC-Startadresse des Zielsystems | `.org $0801` auf dem C128 |

Die BASIC-Übersetzung ([BASIC](help:basic)) vergibt für ihre eigenen Fehler ebenfalls Nummern ab E301 (zum Beispiel `E320 TYPE MISMATCH`); dort gelten die Texte der BASIC-Seite. Fehler beim Erzeugen eines Moduls (`crt`, `car`, `rom`, `a0`) erscheinen ohne Code als Klartext, zum Beispiel „Standardmodul: Code muss in $8000-$BFFF liegen“; siehe [Module](help:cartridges).
