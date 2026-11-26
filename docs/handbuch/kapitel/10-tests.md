---
id: tests
titel: Automatisch testen
reihenfolge: 10
stufe: Fortgeschritten
zeit: 30 Minuten
siehe_auch: [debugger, bauen-starten, systeme]
referenz: [tests, debugger]
---

# Automatisch testen

**Ziel:** Du schreibst eine kleine Testdatei, die dein Programm automatisch prüft: Ist es klein genug? Greift es auf verbotene Speicherstellen zu? Liefert ein Unterprogramm das richtige Ergebnis und braucht es nicht zu viele Zyklen? Dafür musst du nicht jedes Mal von Hand starten und nachsehen.

## Das brauchst du

- Ein Projekt, das sich fehlerfrei bauen lässt. Wir nehmen das Übungsprogramm `farbwechsel` aus dem Kapitel [Der Debugger](kapitel:debugger). Du kannst auch das fertige Beispiel `examples/c64/tests_demo` öffnen.
- Für die **statischen Prüfungen** brauchst du keinen Emulator.
- Für **Tests im Emulator** brauchst du VICE (die Tests laufen derzeit nur mit VICE; alle anderen Emulatoren werden nicht unterstützt).

## Zwei Arten von Prüfungen

Eine Testdatei hat die Endung **`.vctest`** und kann beides enthalten:

- **Statische Prüfungen** (`static`) schauen sich das gebaute Programm an, ohne es auszuführen: Größe, Speicherüberlappungen, Zeropage-Nutzung, Zyklen.
- **Tests im Emulator** (`tests`) rufen einzelne Unterprogramme mit bestimmten Startwerten auf und prüfen das Ergebnis. Dafür startet VintageCoder VICE, schreibt dein Programm in den Speicher und ruft das Unterprogramm über eine „Falle“ (ein Stück Code im freien RAM) auf.

Wichtig: Die Tests prüfen das **gebaute** Programm. Baue also zuerst (F7). Die IDE liest `build/farbwechsel.prg` und die `.vcdbg`-Datei daneben (sie enthält die Namen deiner Marken).

## Eine Testdatei anlegen

1. Klicke in der Seitenleiste (Überschrift **Projekt**) auf das Symbol **Neue Datei**. Im Dialog **Neue Datei** gib als Name `farbwechsel.vctest` ein und stelle bei **Art** den Eintrag **Ohne Endung** ein. Weil der Name schon eine Endung hat, soll VintageCoder keine zweite anhängen. Klicke auf **Anlegen**. Eine eigene Vorlage für Testdateien gibt es in der Oberfläche noch nicht.
2. Die neue Datei öffnet sich im Editor. Trage diesen Text ein. Er prüft das Übungsprogramm aus dem Debugger-Kapitel:

```json
{
  "kind": "test", "target": "c64", "program": "build/farbwechsel.prg",
  "static": [
    { "check": "size", "max": 200 },
    { "check": "overlap" },
    { "check": "cycles", "label": "naechste_farbe", "max": 20 }
  ],
  "tests": [
    { "name": "naechste_farbe: 3 wird 4",
      "setup": { "mem": { "$fb": 3 } },
      "call": "naechste_farbe",
      "expect": [ "[$fb] == 4", "a == 4", "([$d020] & 15) == 4" ] },
    { "name": "naechste_farbe: nach 15 kommt wieder Farbe 0",
      "setup": { "mem": { "$fb": 15 } },
      "call": "naechste_farbe",
      "expect": [ "[$fb] == 16", "a == 0", "fz" ],
      "cycles": { "max": 20 } }
  ]
}
```

Was darin steht:

| Feld | Bedeutung |
|---|---|
| `kind`, `target` | immer `"test"`; das Zielsystem des Programms |
| `program` | die gebaute Programmdatei, relativ zur Testdatei |
| `static` | die statischen Prüfungen, jede mit `check` |
| `tests` | die Tests im Emulator |
| `setup` | Startwerte: `mem` (Adresse → Byte), die Register `a`, `x`, `y`, `sp`, `p` und die Flags |
| `call` | das Unterprogramm, als Marke oder Adresse (mit `"return": "rti"` für Interrupt-Routinen) |
| `expect` | Ausdrücke, die nach dem Aufruf **wahr** sein müssen |
| `cycles` | Zyklenbudget: wie lange der Aufruf höchstens dauern darf |

In den Ausdrücken stehen die Register (`a x y`), die Flags (`fz fc` …), der Speicher (`[adresse]`) und die Namen aus deinem Programm. Das Flag-Kürzel allein heißt „gesetzt“, mit Ausrufezeichen davor „nicht gesetzt“ (`!fc`).

> **Tipp:** Jeder Test muss alles selbst vorbereiten, was er braucht. Zwischen den Tests schreibt die IDE nur Programm und Daten neu, nicht aber Zeropage, Bildschirm oder Chip-Register. Das Hauptprogramm läuft nicht: Was es beim Start einstellt, gehört in das `setup`. Standard sind `sp=$FF` und gelöschte Flags (nur I gesetzt).

> **Achtung:** Ein Test ohne `expect` und ohne `cycles` ist ein Fehler („prüft nichts“).

## Tests ausführen

Die Tests laufen in der **Ablage** unter dem Editor im Reiter **Tests**.

![Der Code-Editor mit der Datei tests_demo.asm. Unter dem Editor ist die Ablage mit dem Reiter „Tests“ geöffnet: Knöpfe „Alle ausführen“ und „Fehlgeschlagene“, die Auswahl „Ziel“ mit „VICE x64sc“, ein Filterfeld, das Häkchen „nur Fehler“ und die Angabe „8 von 17 bestanden“. Darunter stehen die statischen Prüfungen mit grünen Punkten und dem Wort „bestanden“.](bild:tests-panel "Der Reiter „Tests“ in der Ablage mit der Liste der Prüfungen und ihren Ergebnissen.")

1. Öffne die Ablage mit **Cmd/Strg+J** und klicke auf den Reiter **Tests**. Oder drücke **Alt+F8** (Menü **Projekt ▸ Tests ausführen**): Das öffnet den Reiter und führt alle Tests aus.
2. Die Leiste oben im Reiter hat diese Bedienelemente: den Knopf **Alle ausführen** (während des Laufs auch **Abbrechen**), den Knopf **Fehlgeschlagene** (wiederholt nur die gescheiterten Tests), die Auswahl **Ziel**, das Feld **Tests filtern …** und das Häkchen **nur Fehler**. Rechts daneben steht die Zusammenfassung, zum Beispiel „3 von 5 bestanden“.
3. Links steht die Liste der Tests, gegliedert nach Testdateien. Rechts steht das Detail des gewählten Tests.

Mit **Ziel** bestimmst du, wo die Tests laufen. Die Auswahl bietet:

- **Nur statische Prüfungen**: nur die statischen Prüfungen, schnell und ohne Emulator.
- **VICE …** (oder ein anderer gefundener Emulator): auch die Tests im Emulator.

> **Info:** Tests auf echter Hardware (Ultimate 64) und die Zyklenmessung über die Ultimate gibt es in der Oberfläche noch nicht. Sie laufen auf der Kommandozeile (siehe unten).

Die erste Prüfung kannst du sofort machen: Baue mit **F7**, stelle **Ziel** auf *Nur statische Prüfungen* und klicke **Alle ausführen**. Es laufen die drei statischen Prüfungen, sie sind alle bestanden:

```
OK  Größe alle Segmente ≤ 200  $20 (32 Bytes)
OK  Keine Segment-Überlappung  1 Segmente
OK  Zyklen naechste_farbe ≤ 20  20  [farbwechsel.asm:18]
```

Die Tests im Emulator werden dabei als „übersprungen“ gemeldet, mit dem Grund „braucht den Emulator“. Stelle **Ziel** auf VICE, um sie auszuführen.

### Ergebnisse lesen

Jeder Test hat einen Zustand:

| Zustand | Bedeutung |
|---|---|
| **bestanden** | alle Erwartungen erfüllt, der Messwert (zum Beispiel die Zyklen) steht dabei |
| **fehlgeschlagen** | eine Erwartung stimmt nicht; die Meldung nennt linke und rechte Seite des Vergleichs |
| **Hinweis** | ein Zyklenbudget ist überschritten, gilt aber nicht als Fehler (`level: hint`) |
| **Zeit überschritten** | die Falle wurde nicht rechtzeitig erreicht (`timeoutMs`) |
| **übersprungen** | der Test lässt sich hier nicht ausführen; der Grund steht dabei |

Klicke einen Test an: Rechts im Reiter siehst du den **Zustand**, die Meldung mit **Erwartung und Ist-Wert**, den **gemessenen Wert** („Gemessen“, zum Beispiel die Zyklen) und den Zustand der Register bei der Rückkehr. Drei Knöpfe gehören zum Test: **Zur Quelle** springt in den Quelltext, **Testdatei öffnen** öffnet die Testdatei, und **Diesen Test debuggen** startet den Test im Debugger (siehe unten).

![Der Bereich Code mit der Datei tests_demo.asm und unten der Seite „Tests“. Die Liste zeigt Prüfungen mit grünen Punkten („bestanden“), einen gelben Eintrag „fehlgeschlagen“ und zwei „übersprungen“; oben steht „8 von 17 bestanden“. Rechts daneben stehen Zustand, Meldung und gemessener Wert des gewählten Tests und die Knöpfe „Zur Quelle“, „Testdatei öffnen“ und „Diesen Test debuggen“.](bild:tests-ergebnis "Die Seite „Tests“ mit der Liste der Prüfungen und dem Ergebnis des gewählten Tests.")

Typisch sieht eine Fehlermeldung so aus: *Erwartung „peekw($0200) == 90000“ ist nicht erfüllt: linke Seite = 24464 ($5F90), rechte Seite = 90000 ($15F90).* So erkennst du sofort, dass etwa ein Übertrag fehlt.

### Einen Test im Debugger untersuchen

Schlägt ein Test fehl und du siehst nicht, warum, wähle ihn aus und klicke auf **Diesen Test debuggen**. VintageCoder startet eine Debug-Sitzung mit der „Falle“ des Tests und öffnet den Bereich **Debug**. Dort steht neben dem Status der Hinweis **Test-Sitzung**. Das Programm hält gleich am Anfang des Aufrufs an, mit den Startwerten aus dem `setup`. Von dort gehst du Schritt für Schritt weiter und siehst Register, Flags und Speicher (siehe [Der Debugger](kapitel:debugger)). Dieser Knopf baut nicht neu: Baue vorher mit **F7**, damit das Programm zum Test passt. Nicht für jedes Zielsystem gibt es das Debuggen im Trap-Modus; dann meldet VintageCoder das.

*[Bild folgt in E13: debug-test-sitzung]*

## Zyklen zählen

Wie viele CPU-Zyklen braucht ein Unterprogramm? Das ist bei Rasterprogrammen die entscheidende Frage. VintageCoder beantwortet sie auf drei Wegen.

**1. Statisch** (ohne Emulator): `{ "check": "cycles", "label": "naechste_farbe", "max": 20 }` rechnet die Zyklen aller Wege von der Marke bis `rts` oder `rti` aus. Bei einem Unterprogramm ohne Verzweigung steht ein genauer Wert da (`20`), bei Verzweigungen eine Spanne (`18 bis 19`). Bei Schleifen, `jsr`, `jmp (…)` und unbekannten Befehlen meldet die Prüfung „nicht bestimmbar“ und schätzt nicht. Gezählt werden reine CPU-Zyklen; Verzögerungen durch Bad Lines, Sprites, Interrupts und Wartezustände sind nicht enthalten.

Im Quelltext kannst du auch einen Block markieren, den die Prüfung messen soll (Beispiel aus `examples/c64/tests_demo/tests_demo.asm`):

```asm
irq_handler:
.cycles irq max=24
        pha
        lda COLOR
        sta BORDER
        pla
        rti
.endcycles
```

In der Testdatei prüft `{ "check": "cycles", "block": "irq" }` genau diesen Block. Das Ergebnis dazu aus dem echten Lauf: `OK  Zyklen irq ≤ 24  20`.

**2. Im Emulator**: `"cycles": { "max": 20 }` in einem Test misst die Dauer des Aufrufs (ohne den Aufrufbefehl, aber einschließlich `rts` oder `rti`) und vergleicht sie mit dem Budget. Das Budget `max` ist ein *Hinweis*, außer du gibst `"level": "error"` an.

**3. Auf echter Hardware** (nur auf der Kommandozeile): Mit einer Ultimate 64 misst der Modus `"mode": "measure"` die Zyklen von Adresse A bis Adresse B auf dem echten Gerät, einschließlich Interrupts und Bad Lines (siehe `examples/c64/tests_cycles`). Dafür muss am Gerät unter *Data Streams ▸ Debug Stream Mode* der Wert „6510 Only“ stehen. Die Messung wird nie geschätzt: Gehen Pakete verloren, meldet die IDE „übersprungen“.

> **Info:** Im Beispiel `tests_demo` siehst du den Unterschied schön: Für `next_color` meldet die statische Prüfung „18 bis 19“ Zyklen, weil der Code eine Verzweigung hat. Das Budget `max: 18` ergibt deshalb einen **Hinweis** („im ungünstigsten Fall 19 Zyklen, erlaubt: höchstens 18“), und `max: 20` ist bestanden.

## Weitere statische Prüfungen

Alle Prüfungen stehen im Abschnitt `static`:

| `check` | Prüft |
|---|---|
| `size` | Größe in Bytes (`min`, `max`; mit `maxEnd` die Adresse hinter dem letzten Byte; mit `segment` nur ein Segment) |
| `zeropage` | `forbid`: Adressbereiche wie `"$02-$1f"`, die kein Segment belegen und kein Befehl ansprechen darf |
| `overlap` | Segmente, die sich überlappen |
| `cycles` | Zyklen ab einer Marke (`label`) oder aus einem Zyklenblock (`block`) gegen `max` |
| `expr` | ein beliebiger Ausdruck, etwa `size("") < $2000 - $0801`, mit optionalem `name` |

Mit `"level": "error"` wird aus einem Hinweis ein Fehler; auf der Kommandozeile macht `--strict` jeden Hinweis zum Fehler.

## Das Programm prüft sich selbst: der Postfach-Modus

> **Info:** Der Postfach-Modus ist in der Oberfläche noch nicht ausgebaut: Die Hardware-Abfrage und die Läufe auf einer Ultimate gibt es nur auf der Kommandozeile (siehe unten). Auch ob und wie die Oberfläche Postfach-Tests im Emulator ausführt, ist noch nicht geprüft. Verlasse dich für diesen Modus auf die Kommandozeile. Im Postfach-Modus gibt es keine Falle: Das Programm lässt sich wie jedes andere mit **F6** debuggen.

Manchmal ist es einfacher, die Prüfungen *im* Programm zu schreiben. Dafür gibt es den **Postfach-Modus**: Dein Testprogramm bindet die Testbibliothek des Zielsystems ein, führt Prüfbefehle aus und schreibt das Ergebnis in ein kleines Postfach im RAM. Die IDE startet das Programm, wartet auf das Ende und liest nur das Postfach. Das Beispiel `examples/c64/tests_demo/tests_mailbox.asm`:

```asm
.target c64
.cpu 6510 { illegals = deny }
.include "c64/test.inc"

.org $0801
.basicstub start

start:  sei
        test_begin
        lda #21
        jsr double
        assert_a_eq 42              ; 21 * 2
        ldx #5
        jsr plus3
        assert_x_eq 8               ; 5 + 3
        assert_a_eq 42              ; A blieb erhalten
        lda #$7f
        sta $fb
        assert_mem_eq $fb, $7f
        test_end                    ; Ergebnis ins Postfach, danach Endlosschleife

double: asl
        rts

plus3:  inx
        inx
        inx
        rts
```

Die zugehörige Testdatei hat nur den Namen des Programms und `"mode": "mailbox"`:

```json
{
  "kind": "test", "target": "c64", "program": "build/tests_mailbox.prg", "mode": "mailbox",
  "static": [
    { "check": "size", "max": "$0100" },
    { "check": "overlap" }
  ],
  "tests": [
    { "name": "double, plus3 und Speicher (Selbsttest)", "timeoutMs": 20000 }
  ]
}
```

Die Prüfbefehle: `test_begin` am Anfang, dann `assert_a_eq`, `assert_x_eq`, `assert_y_eq` und `assert_mem_eq adresse, wert`, am Ende `test_end`. Die Register bleiben bei den Prüfbefehlen erhalten.

Das Postfach liegt beim C64 bei `$033C`, beim C128 bei `$0B00` und beim Atari 8-Bit bei `$0600` (die Bibliotheken heißen `c64/test.inc`, `c128/test.inc`, `atari8bit/test.inc`). Das Programm darf diese Stelle nicht überschreiben. Plus/4, VC-20 und MEGA65 haben kein Postfach; dort wird der Test übersprungen. Geprüft ist der Modus mit VICE (C64) und atari800 (Atari 8-Bit).

Postfach-Tests laufen auch auf einer echten **Ultimate 64**, aber nur auf der Kommandozeile (siehe unten). Weil das Gerät dabei zurückgesetzt und sein Speicher überschrieben wird, fragt `vcc test` bei **jedem** Lauf auf Hardware nach. Diese Abfrage lässt sich nur mit `--yes` überspringen.

## Auf der Kommandozeile

Dieselben Tests laufen ohne Oberfläche, etwa in einer automatischen Prüfung (Pipeline):

```bash
vcc build examples/c64/tests_demo
vcc test examples/c64/tests_demo
vcc test examples/c64/tests_demo --backend vice
```

Der erste Aufruf baut, der zweite führt nur die statischen Prüfungen aus (Tests im Emulator werden „übersprungen“), der dritte nimmt VICE dazu. Der Exit-Code ist 1 bei Fehlern oder Zeitüberschreitungen, Hinweise und übersprungene Tests ändern ihn nicht. Mit `--junit datei.xml` schreibt `vcc test` zusätzlich JUnit-XML. Weitere Schalter: `--filter TEXT`, `--strict`, `--poll`.

Im Beispiel `tests_demo` sieht die Ausgabe ohne Emulator so aus (gekürzt):

```
tests_demo/tests_demo.vctest  (c64, build/tests_demo.prg)
  OK       Größe (ohne Namen) ≤ $0100  $37 (55 Bytes)
  OK       Zeropage frei $02-$1f  kein Zugriff
  OK       Keine Segment-Überlappung  1 Segmente
  HINWEIS  Zyklen next_color ≤ 18  18 bis 19  [tests_demo.asm:27]
  ÜBERSPR. Zyklen clear_text ≤ 100  [tests_demo.asm:38]
           nicht bestimmbar: Schleife bei $082F (tests_demo.asm:39)
  ÜBERSPR. next_color: 15 wird 0
           braucht den Emulator (Trap-Modus): `vcc test --backend vice`
10 bestanden, 0 fehlgeschlagen, 0 Zeitüberschreitung, 1 Hinweise, 7 übersprungen
```

## Häufige Stolpersteine

| Problem | Ursache und Lösung |
|---|---|
| Alle Tests „übersprungen“ | In **Ziel** ist „Nur statische Prüfungen“ gewählt, oder VICE ist nicht gefunden. Wähle VICE im Feld **Ziel**; steht VICE dort nicht, hat VintageCoder ihn nicht gefunden (die Startseite zeigt die erkannten Emulatoren). |
| Die Prüfungen finden das Programm nicht | Du hast nicht gebaut, oder `program` zeigt auf die falsche Datei. Baue mit F7 und prüfe den Pfad (relativ zur Testdatei oder zu einem übergeordneten Ordner). |
| Namen aus dem Programm werden nicht erkannt | Die `.vcdbg`-Datei neben dem Programm ist veraltet: neu bauen. |
| Test schlägt fehl, obwohl das Programm richtig scheint | Der Test hat etwas im `setup` vergessen. Das Hauptprogramm läuft nicht, und Zeropage und Chips behalten ihre Werte aus dem vorigen Test. |
| „Zyklen … nicht bestimmbar“ | Schleife, `jsr` oder `jmp (…)` im Weg. Teile das Unterprogramm in Stücke ohne Schleife oder messe im Emulator. |
| Zeitüberschreitung | Die Falle wurde nicht erreicht, zum Beispiel weil das Unterprogramm in einer Endlosschleife endet. Prüfe `call` und `timeoutMs`. |
| Ein Test mit `cycles` wird bei Plus/4, VC-20 oder C128 übersprungen | Die Zyklenmessung im Emulator und die Standardadresse der Falle gibt es bisher nur für den C64 (PAL, x64sc). Bei den anderen VICE-Systemen muss `trap` in der Testdatei stehen, und Tests mit `cycles` werden übersprungen. |
| Messung auf der Ultimate „ungültig“ | Pakete des Debug-Stroms gingen verloren (zum Beispiel Firewall für eingehendes UDP). Wiederholen. |

## Wie geht es weiter?

- Das Übungsprogramm `farbwechsel` stammt aus dem Kapitel [Der Debugger](kapitel:debugger).
- Die Besonderheiten der Zielsysteme (Postfach, Emulatoren): [Zielsysteme im Vergleich](kapitel:systeme).
- Zum Nachschlagen aller Felder: [Tests mit vcc test](hilfe:tests).
