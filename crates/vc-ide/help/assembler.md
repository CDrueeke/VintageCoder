# Assembler

Ein **Assembler** übersetzt Programme, die du in lesbaren Kürzeln wie `lda #$00` schreibst, in die Zahlen, die der Prozessor wirklich ausführt (den **Maschinencode**). Der Assembler von VintageCoder beherrscht die 6502-Familie: 6502, 6510 (C64), 7501 (Plus/4), 8502 (C128), „Sally“ (Atari) und die 45GS02 des MEGA65 – also alle Zielsysteme der Entwicklungsumgebung. Er rechnet beim Übersetzen mit Ausdrücken, kennt Makros, Schleifen, Namensräume und Module.

## Die Handbuchseiten

| Seite | Inhalt |
|---|---|
| **diese Seite** | Mein erstes Programm, Aufbau einer Zeile, Adressierungsarten, undokumentierte Befehle |
| [Ausdrücke und Funktionen](help:asm-expr) | Zahlenformate, Operatoren und Vorrang, `<` und `>`, eingebaute Funktionen, Listen, `.function` |
| [Direktiven](help:asm-directives) | `.org`, Daten, Konstanten und Variablen, `.if`, `.repeat`, `.for`, `.include`, `.basicstub`, `.cycles` … |
| [Texte und Zeichensätze](help:asm-text) | `.petscii`, `.screen`, `.atascii`, `.text`, `.encoding`, Sonderzeichen `{clr}` |
| [Labels, Makros, Scopes, Strukturen](help:asm-macros) | lokale und anonyme Labels, `.macro`, `{ }`, `.struct`, `.enum` |
| [Segmente, Bänke, Module](help:cartridges) | `.segment`, `.bank`, `.cartridge` |
| [Systembibliothek](help:asm-library) | `#include <c64/hardware.asm>` und fertige Makros |
| [Spickzettel und Fehlercodes](help:asm-index) | alle Direktiven in einer Tabelle, alle Fehlercodes |

Die Zielsysteme beschreiben [VC-20](help:vic20), [C128 und Plus/4](help:c128-plus4) und [MEGA65](help:mega65). Zum Bauen und Starten siehe [Bauen und Starten](help:build).

## Mein erstes Programm

Das kleinste sinnvolle Programm färbt den Bildschirmrahmen rot und schreibt ein Wort auf den Bildschirm. So gehst du vor:

1. **Projekt anlegen:** *Datei > Neues Projekt …*, Zielsystem **C64**, Vorlage **Assembler**. Oder auf der Kommandozeile: `vcc new hallo --target c64 --template asm-hello` (siehe [Willkommen und Projekt-Wizard](help:wizard)). Die Vorlage legt die Datei `src/c64/main.asm` an.
2. **Quelltext ersetzen:** Öffne `main.asm` und schreibe das Programm unten hinein.
3. **Bauen und starten:** **F7** baut, **F5** baut und startet im Emulator (siehe [Bauen und Starten](help:build)). Auf der Kommandozeile: `vcc build` im Projektordner.

```asm roh
; Mein erstes Programm: roter Rahmen und ein Gruß auf dem Bildschirm
.target c64
.basicstub start            ; BASIC-Zeile "10 SYS 2061", damit RUN das Programm startet

start:  lda #2              ; 2 = Farbe Rot
        sta $d020           ; Rahmenfarbe setzen
        ldx #0
loop:   lda text,x          ; ein Zeichen holen ...
        sta $0400,x         ; ... und in den Bildschirmspeicher schreiben
        inx
        cpx #5              ; fünf Zeichen kopiert?
        bne loop            ; nein: weiter
        rts                 ; zurück zu BASIC

text:   .screen "hallo"     ; Text als Bildschirmcodes
```

Zeile für Zeile:

| Zeile | Bedeutung |
|---|---|
| `; …` | Kommentar bis zum Zeilenende (`//` geht auch). Der Assembler ignoriert ihn |
| `.target c64` | merkt das Zielsystem im Quelltext vor, damit die IDE es erkennt (siehe [Direktiven](help:asm-directives)) |
| `.basicstub start` | legt die BASIC-Zeile `10 SYS 2061` an. `RUN` führt sie aus und springt zu `start`. Der Stub beginnt automatisch an der BASIC-Startadresse des C64, `$0801` (ein `.org` ist nicht nötig) |
| `start:` | ein **Label**: ein Name für die Adresse an dieser Stelle |
| `lda #2` | lade die Zahl 2 in den Akkumulator (das Arbeitsregister A). `#` bedeutet „die Zahl selbst“ |
| `sta $d020` | speichere A an Adresse `$d020`, das Register der Rahmenfarbe |
| `ldx #0` | lade 0 in das Indexregister X |
| `lda text,x` | lade das Byte an Adresse `text + X` |
| `sta $0400,x` | schreibe es in den Bildschirmspeicher (der beginnt bei `$0400`) |
| `inx` / `cpx #5` / `bne loop` | X erhöhen, mit 5 vergleichen, solange nicht gleich zurück zu `loop` |
| `rts` | zurück zum Aufrufer (hier BASIC) |
| `.screen "hallo"` | legt die fünf Zeichen als **Bildschirmcodes** ab (siehe [Texte](help:asm-text)) |

Nach dem Bauen entstehen im Ordner `build/` das Programm (`main.prg`), ein Listing (`main.lst`: Adresse, Bytes und Quelltext nebeneinander), die Symboltabelle und die Speicherbelegung. Das Listing zeigt, was der Assembler aus jeder Zeile gemacht hat:

```
080d  a9 02      start:  lda #2
080f  8d 20 d0           sta $d020
0812  a2 00              ldx #0
0814  bd 20 08   loop:   lda text,x
```

Du siehst: das Programm beginnt bei `$0801`, `lda #2` wurde `a9 02`, `sta $d020` wurde `8d 20 d0` (Adresse, Low-Byte zuerst), und `text` wurde durch die Adresse `$0820` ersetzt. Weiter geht es mit den Seiten [Direktiven](help:asm-directives) und [Ausdrücke](help:asm-expr).

## Aufbau einer Zeile

Eine Zeile besteht aus höchstens vier Teilen: **Label**, **Befehl** oder **Direktive**, Operand und Kommentar.

```
start:  lda #$00        ; Label mit Doppelpunkt
        sta vic2.border ; Chip-Register als Symbole
.loop:  dex             ; lokales Label gilt bis zum nächsten globalen
        bne .loop
```

- **Label:** ein Name mit Doppelpunkt am Zeilenanfang. `start:` benennt die Adresse; `.loop:` ist ein *lokales* Label (siehe [Labels](help:asm-macros)). Ohne Doppelpunkt hält der Assembler das Wort für einen Befehl.
- **Befehl:** ein Kürzel des Prozessors (`lda`, `sta`, `jmp` …). Groß-/Kleinschreibung ist egal. Die Befehle der CPU zeigt **F1** auf dem Wort.
- **Direktive:** beginnt mit einem Punkt und ist eine Anweisung an den Assembler (siehe [Direktiven](help:asm-directives)).
- **Zuweisung:** `NAME = ausdruck` definiert eine Konstante, `*= adresse` setzt die Adresse (wie `.org`).
- **Kommentar:** `;` oder `//` bis zum Zeilenende (nicht innerhalb von Anführungszeichen).
- Auf einer Zeile steht nur **ein** Befehl; einen Trenner wie `:` zwischen mehreren Befehlen gibt es nicht.
- Chip-Register stehen als `chip.register` zur Verfügung (`vic2.border`); **F1** auf dem Wort zeigt die Beschreibung.

## Adressierungsarten

Der 6502 kennt verschiedene Arten, den Operanden anzugeben:

| Schreibweise | Name | Bedeutung |
|---|---|---|
| `#$12` | unmittelbar | die Zahl selbst |
| `$12` | Zeropage | Adresse in den ersten 256 Bytes (kürzer und schneller) |
| `$1234` | absolut | beliebige 16-Bit-Adresse |
| `$12,x` `$12,y` `$1234,x` `$1234,y` | indiziert | Adresse plus Register X oder Y |
| `($12,x)` | indirekt, vor-indiziert | Zeiger in der Zeropage bei `$12 + X` |
| `($12),y` | indirekt, nach-indiziert | Zeiger in der Zeropage bei `$12`, dazu Y |
| `($1234)` | indirekt | Sprungzeiger (nur `jmp`) |
| `a` oder nichts | Akkumulator | `asl a` oder `asl` schiebt den Akkumulator |

Die **Zeropage** (Seite 0, Adressen `$00`–`$FF`) ist ein besonders schnell erreichbarer Speicherbereich. Der Assembler wählt bei `$12` oder einem Label mit kleinem Wert automatisch die kürzere Zeropage-Form. Die 45GS02 des MEGA65 kennt weitere Arten, siehe [MEGA65](help:mega65).

### Adressmodus erzwingen

Bei Vorwärtsreferenzen (das Label steht weiter unten) steht die Größe des Wertes erst im zweiten Durchlauf fest, und der Assembler plant erst einmal die lange Form ein. Das kann Adressen verschieben. Mit einem Suffix legst du die Form fest, etwa für exakte Rasterzeit oder selbstmodifizierenden Code:

```asm
lda.z zeiger     ; Zeropage (2 Byte); auch .zp
lda.a $10        ; absolut (3 Byte), obwohl der Wert < 256 ist; auch .abs
sta.a $d020,x
zeiger = $fb
;= a5 fb ad 10 00 9d 20 d0
```

Ein Suffix geht nur bei direkter Adressierung (mit `,x` / `,y`). Sprungbefehle, `#wert` und indirekte Formen melden **E215**, ebenso eine Form, die der Befehl nicht hat (`jmp.z`).

```asm fehler
 jmp.z $10
```

```text
error[E215]: JMP.z: keine Zeropage-Form
```

### Weitere Schreibweisen

| Schreibweise | Hinweis |
|---|---|
| `blt ziel` / `bge ziel` | Alias für `bcc` / `bcs` („kleiner“, „größer oder gleich“ nach `cmp`) |
| `asl` ohne Operand | gleichbedeutend mit `asl a` |
| `nop` auf dem MEGA65 | wird zu `eom` (`$EA`) |

## Undokumentierte Befehle (6502)

Die 6502-Familie hat Befehle, die nie offiziell beschrieben wurden, aber funktionieren (`lax`, `dcp`, `slo` …). Ob du sie benutzen darfst, steuert `illegals = deny | warn | allow` im `.cpu`-Kopf: `.cpu 6510 { illegals = warn }`. Mit `deny` ist ein solcher Befehl der Fehler **E201**, mit `warn` die Warnung **W104**. Besonders unzuverlässige (`xaa`) warnen immer mit **W105**. Gebräuchliche Zweitnamen werden erkannt: `aso`=`slo`, `lse`=`sre`, `isb`=`ins`=`isc`, `dcm`=`dcp`, `sbx`=`axs`, `asr`=`alr`, `ane`=`xaa`, `axa`=`sha`, `sxa`=`shx`, `sya`=`shy`, `xas`=`tas`, `kil`=`hlt`=`jam`.

## So arbeitet der Assembler

- **Mehrere Durchläufe:** Der Assembler übersetzt den Quelltext mehrmals, bis alle Adressen feststehen. Deshalb darfst du Labels vor ihrer Definition benutzen. Die Anzahl der Durchläufe steht im [Bauprotokoll](help:build).
- **Ausgabe:** Die Bytes liegen in **Segmenten** (je `.org` eines). Je nach Zielsystem entsteht ein PRG (Ladeadresse + Daten), ein Atari-XEX, ein Modul (`.crt`, `.car`) oder eine Diskette.
- **Ausgabedateien:** Listing (`.lst`), Symboltabelle (`.sym.txt`), VICE-Labels (`.vs`), Debug-Info (`.vcdbg`).
- **Kommandozeile:** `vcc asm datei.asm -o ausgabe --target c64 --format prg` übersetzt eine einzelne Datei; `-I ordner` ergänzt die Include-Suche, `-D NAME=wert` definiert eine Konstante (für `.ifdef`/`.if`), `--illegals deny|warn|allow` setzt die Voreinstellung. `vcc build` baut ein ganzes Projekt.
- **Fehlermeldungen:** `error[E…]` und `warning[W…]` mit Datei und Zeile; die Codes erklärt der [Spickzettel](help:asm-index).

Mit **F1** auf einer Direktive, einem Befehl oder einem Chip-Register zeigt die Hilfe die passende Beschreibung.
