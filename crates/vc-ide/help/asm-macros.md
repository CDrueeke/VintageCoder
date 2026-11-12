# Assembler: Labels, Makros, Scopes und Strukturen

Diese Seite erklärt, wie man Stellen im Programm benennt (**Labels**), wiederkehrende Codestücke abkürzt (**Makros**), Namen ordnet (**Scopes**) und Datenlayouts beschreibt (**Strukturen**). Sie gehört zum [Assembler-Handbuch](help:assembler). Zeilen der Form `;= 01 02` zeigen die erzeugten Bytes; alle Beispiele sind mit dem Assembler geprüft.

## Labels

Ein **Label** ist ein Name für eine Adresse. Statt `jmp $c012` schreibst du `jmp start`; der Assembler setzt die Adresse ein, auch wenn du den Code später verschiebst.

```
name:   lda #1          ; Label vor einem Befehl
name:                   ; oder allein in einer Zeile
```

- Ein Label **muss mit einem Doppelpunkt enden**. Ohne `:` hält der Assembler das erste Wort für einen Befehl und meldet **E205** (`unbekannter Befehl`).
- Namen bestehen aus Buchstaben (auch Umlaute), Ziffern und `_` und beginnen nicht mit einer Ziffer. **Groß-/Kleinschreibung zählt** bei Labels (`Start` und `start` sind verschiedene Namen); Befehle und Direktiven sind davon frei.
- Ein Label darf nicht doppelt vergeben werden (**E204**). Eine Ausnahme sind lokale Labels und Scopes.
- Vor dem ersten `.org` darf kein Label stehen (**E300**).
- Ein Label kann auch eine Zahl benennen: `name = ausdruck` (siehe [Konstanten](help:asm-directives)).

### Lokale Labels: `.name`

Hunderte Schleifen heißen `schleife` oder `warten`. Damit sie sich nicht in die Quere kommen, beginnst du den Namen mit einem **Punkt**: `.warten`. Ein lokales Label gehört zum letzten **globalen** Label (einem ohne Punkt) davor; dasselbe `.warten` darf unter jedem globalen Label einmal vorkommen.

```asm
a:      nop
.l:     nop
        jmp .l              ; a.l
b:      nop
.l:     nop
        jmp .l              ; b.l
        jmp a.l             ; von außen: global.lokal
;= ea ea 4c 01 c0 ea ea 4c 06 c0 4c 01 c0
```

Von außerhalb sprichst du ein lokales Label als `global.lokal` an (`a.l`). Lokale Labels gelten je Scope getrennt.

### Anonyme Labels: `-` und `+`

Für kurze Sprünge ohne Namen: ein einzelnes `+` oder `-` am Zeilenanfang ist ein Label. Im Operanden springt `-` zum **vorigen** `-`-Label, `+` zum **nächsten** `+`-Label. `--` und `++` überspringen eines, `---` zwei und so weiter.

```asm
        ldx #3
-       dex
        bne -               ; zurück zum "-" darüber
        beq +               ; vor zum nächsten "+"
        nop
+       rts
;= a2 03 ca d0 fd f0 01 ea 60
```

Anonyme Labels zählen über Scopes hinweg in Quelltextreihenfolge und tauchen nicht in der Symboltabelle auf. Fehlt ein Ziel, meldet der Assembler **E214**.

```asm fehler
 bne -
```

```text
error[E214]: kein passendes '-'-Label davor
```

**Welches nehme ich?** Anonyme Labels für Sprünge über zwei, drei Zeilen; lokale Labels (`.name`) für Schleifen und Verzweigungen innerhalb einer Routine; globale Labels für Einsprungpunkte, die von anderswo angesprungen werden.

## Makros

Ein **Makro** ist eine benannte Abkürzung für mehrere Zeilen. Der Assembler setzt bei jedem Aufruf den Text des Makros ein (wie „Suchen und Ersetzen“) und übersetzt ihn. Ein Makro ist **kein Unterprogramm**: Es gibt weder `jsr` noch `rts`; der Code steht bei jedem Aufruf neu im Programm. Das ist schneller als ein Unterprogramm, kostet aber Speicher.

### Definieren und aufrufen

```
.macro name(parameter, parameter, …)
    …Zeilen…
.endmacro
```

Die Klammern um die Parameter sind freiwillig (`.macro name a, b` geht auch). Aufgerufen wird das Makro wie ein Befehl, mit oder ohne Klammern:

```
name wert1, wert2
name(wert1, wert2)
```

```asm
.macro wait(n)
        ldx #n
@schleife:
        dex
        bne @schleife
.endmacro

        wait 10             ; ldx #10 / dex / bne
        wait(20)            ; gleichwertig mit Klammern
;= a2 0a ca d0 fd a2 14 ca d0 fd
```

Regeln:

- Der Makroname ist nicht von der Groß-/Kleinschreibung abhängig (`WAIT` = `wait`) und darf nicht wie ein Befehl heißen (`nop`, `lda` … → **E209**).
- Ein Makro darf **vor** seiner Definition aufgerufen werden: Der Assembler sammelt alle Makros zuerst.
- Eine Makro-Definition gilt immer, auch wenn sie in einem `.if`-Zweig steht, der nicht gewählt wurde.
- Ein Label vor dem Aufruf (`start: wait 10`) bekommt die Adresse des ersten eingesetzten Befehls.
- Makros dürfen andere Makros aufrufen, höchstens 32 Ebenen tief; Aufruf in sich selbst endet mit **E211** („zu tief verschachtelt“). Ein Makro kann kein Makro definieren.
- Ein Makroaufruf öffnet **keinen** eigenen Scope; so können Makros bewusst globale Labels anlegen.
- `.macro` ohne `.endmacro` ist **E210**.

### Parameter und Aufrufwerte

Die Parameter sind **Textersetzung**: Jeder Name, der genau so heißt wie ein Parameter, wird durch das Argument ersetzt – außer in Zeichenketten (`"…"`) und Kommentaren. Das Argument darf ein beliebiger Text sein, auch ein ganzer Operand:

```asm
.macro lade(wert)
        lda wert
.endmacro
        lade #5             ; lda #5      (unmittelbar)
        lade $1234          ; lda $1234   (absolut)
;= a9 05 ad 34 12
```

**Achtung Vorrang:** Wegen der Textersetzung sieht der Assembler genau das, was du als Argument tippst. `.byte a*2` mit dem Argument `1+2` wird zu `.byte 1+2*2`, also 5 statt 6. Setze deshalb Klammern um die Parameter, wenn du mit ihnen rechnest:

```asm
.macro doppelt(a)
        .byte (a) * 2
.endmacro
        doppelt 1 + 2
;= 06
```

**Achtung Klammern im Aufruf:** Der Assembler schneidet beim Aufruf führende `(` und abschließende `)` vom Argumenttext ab (damit `name(a, b)` geht). Deshalb gelingt es nicht, ein Argument anzugeben, das mit `(` **beginnt** oder mit `)` **endet**, etwa `lade ($fb),y` oder `m 4, (1+2)`. Das Ergebnis ist eine Fehlermeldung wie **E209** oder **E202**. Hilfen: Argument so umschreiben, dass es nicht mit einer Klammer endet (`(1+2)+0`), eine Konstante (`SUMME = (1+2)`) benutzen oder den Zugriffsmodus ins Makro ziehen.

### Standardwerte

Ein Parameter kann einen Standardwert haben (`name=wert`). Parameter mit Standardwert stehen hinter den Pflichtparametern; beim Aufruf dürfen die **rechten** weggelassen werden. Falsche Argumentzahl ist **E209**.

```asm
.macro rahmen(farbe=0, takte=1)
        lda #farbe
        sta $d020
        .repeat takte
        nop
        .endrepeat
.endmacro
        rahmen              ; farbe = 0, takte = 1
        rahmen 5            ; takte = 1
        rahmen 5, 2
;=* a9 00 8d 20 d0 ea
```

```asm fehler
.macro m(a)
        nop
.endmacro
        m 1, 2
```

```text
error[E209]: Makro 'M' erwartet 1 Argument(e), erhalten 2
```

### `@label` – Labels, die bei jedem Aufruf neu sind

Steht ein normales Label im Makrorumpf, entsteht es bei jedem Aufruf wieder → **E204**. Mit einem `@` davor bekommt es bei jedem Aufruf einen eigenen Namen (intern ein lokales Label):

```asm
.macro clear(adr, anzahl)
        ldx #anzahl
        lda #0
@loop:  sta adr - 1,x
        dex
        bne @loop
.endmacro
        clear $0400, 8
        clear $0500, 4          ; kein Konflikt: @loop ist je Aufruf eindeutig
;=* a2 08 a9 00 9d ff 03 ca d0 fa
```

`@name` gilt nur im Makro; außerhalb ist `@x:` ein Fehler. Wiederholbare Labels bekommst du auch, wenn du den Rumpf in `{ … }` setzt (Scope) – dann aber nicht mehr von außen erreichbar.

### `.exitmacro` – vorzeitig aufhören

Beendet das laufende Makro, den Rest des Rumpfs übersetzt der Assembler nicht. Es wirkt auch aus `.if` und `.repeat` heraus, aber nur auf das Makro, in dem es steht.

```asm
.macro w(n, reg=1)
        ldx #n
        .if reg == 0
        .exitmacro         ; hier endet das Makro
        .endif
        nop
.endmacro
        w 1                ; ldx #1 / nop
        w 2, 0             ; nur ldx #2
;= a2 01 ea a2 02
```

Marken mit `@` hinter der `.exitmacro`-Stelle sind dann nicht definiert; springe also nicht dorthin. `.break` und `.continue` wirken nicht über ein Makro hinweg (**E210**, wenn sie im Makro stehen, das in einer Schleife aufgerufen wird).

### Beispiele

16-Bit-Zähler erhöhen:

```asm
.macro inc16(adr)
        inc adr
        bne @fertig
        inc adr + 1
@fertig:
.endmacro
zaehler = $fb
        inc16 zaehler
;= e6 fb d0 02 e6 fc
```

Makro mit einer Schleife darin (Speicher löschen mit Schrittweite):

```asm
.macro schleife(n)
        .repeat n, i
        .byte i
        .endrepeat
.endmacro
        schleife 3
;= 00 01 02
```

**Makro, `.function` oder `.repeat`?** Brauchst du *Code* zum Wiederverwenden → Makro. Brauchst du nur eine *Zahl* aus anderen Zahlen → [.function](help:asm-expr). Willst du denselben Block mehrfach hintereinander → `.repeat` / `.for` ([Direktiven](help:asm-directives)). Ist der Code lang und wird oft gebraucht, ist ein Unterprogramm mit `jsr` platzsparender.

Fertige Makros (16-Bit-Rechnen, Speicher kopieren u. a.) enthält die [Systembibliothek](help:asm-library).

## Scopes

Ein **Scope** (englisch für „Geltungsbereich“) ist ein eigener Namensraum: Namen darin kollidieren nicht mit gleichen Namen außerhalb. Er wird mit geschweiften Klammern geöffnet und geschlossen.

```asm
init: {                     ; `init` ist ein normales Label (Adresse des Blocks) und der Name des Scopes
        ldx #8
loop:   dex                 ; heißt außerhalb des Blocks init.loop
        bne loop
        rts
}
main: {
loop:   jsr init            ; main.loop – kein Konflikt mit init.loop
        jmp init.loop       ; Zugriff von außen: Scope-Name, Punkt, Label
}
;= a2 08 ca d0 fd 60 20 00 c0 4c 02 c0
```

- **Sichtbarkeit:** Innen sind alle Namen der umgebenden Scopes sichtbar (der innerste gewinnt). Von außen erreichst du ein Symbol nur über `scope.name`, bei Verschachtelung `a.b.name`.
- **Anonym:** `{` ohne Label öffnet einen Scope, dessen Namen man von außen nicht ansprechen kann – praktisch für Hilfsroutinen und für Labels in `.repeat`.
- **`.namespace name {`** macht dasselbe, ohne dass `name` selbst ein Label (eine Adresse) wird – zum Beispiel für Konstanten-Gruppen.
- **Lokale Labels** (`.name`) gelten je Scope getrennt.
- **Variablen** folgen den Scopes: eine gleichnamige `.var` im Block verdeckt die äußere.
- Ein Scope-Name darf keinen Punkt enthalten, `}` ohne `{` und ein nicht geschlossener Scope sind **E213**.

```asm
.namespace farbe {
        schwarz = 0
        weiss   = 1
}
        lda #farbe.weiss
;= a9 01
```

### Include mit Namensraum

`.include "lib/music.inc" as music` legt den Inhalt der Datei in den Scope `music`. Von außen heißen die Symbole `music.play`, `music.SPEED`; die Datei selbst sieht alle globalen Namen (Chip-Register, deine Konstanten). Dieselbe Datei lässt sich mehrfach unter verschiedenen Namen einbinden. Makros bleiben global. Ein Beispiel steht bei [.include](help:asm-directives).

## Strukturen und Aufzählungen

Bei Daten mit mehreren Feldern (etwa Sprite-Werten) ist es lästig, die Abstände (Offsets) selbst zu zählen. Eine **Struktur** gibt jedem Feld einen Namen und berechnet die Abstände.

```asm
.struct Punkt
    x      .byte
    y      .byte
.endstruct

.struct Sprite
    x      .byte          ; Offset 0
    y      .byte          ; 1
    ptr    .word          ; 2
    anim   .byte 4        ; 4: vier Bytes (Anzahl darf ein Ausdruck sein)
    pos    Punkt          ; Feld vom Typ einer früher definierten Struktur
    trail  Punkt 3        ; drei Stück davon
.endstruct

spieler: .fill Sprite.size, 0        ; Platz für eine Instanz
        lda spieler + Sprite.y       ; Feld lesen
        ldx #Sprite.size
;= 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 ad 01 c0 a2 10
```

`Sprite.size` ist 16 (1 + 1 + 2 + 4 + 2 + 6 Bytes); `Sprite.y` ist 1.

- Feldtypen: `.byte` (1 Byte), `.word` und `.addr` (2), `.dword` und `.long` (4) oder eine früher definierte Struktur. Eine Anzahl dahinter vervielfacht die Größe.
- Definiert werden `NAME.feld` (Offset), bei verschachtelten Feldern auch `NAME.pos.x`, und `NAME.size` (Gesamtgröße). Die Namen sind normale Konstanten und gelten auch vor der Definition.
- Eine Struktur reserviert keinen Speicher; dafür `.fill NAME.size, 0` verwenden.
- Doppelte Feld- oder Strukturnamen: **E204**; unbekannter Feldtyp: **E208**; fehlendes `.endstruct`: **E210**.

Eine **Aufzählung** (`.enum`) vergibt fortlaufende Nummern für Namen:

```asm
.enum Farbe
    schwarz              ; 0
    weiss                ; 1
    rot = 5
    gruen, blau          ; 6, 7  (auch mehrere in einer Zeile)
.endenum
        lda #Farbe.rot
;= a9 05
```

Weiter mit: [Direktiven](help:asm-directives), [Ausdrücke](help:asm-expr), [Spickzettel](help:asm-index).
