# Tests mit vcc test

`vcc test` prüft ein gebautes Programm anhand einer Testdatei `*.vctest`. Es gibt zwei Stufen: **statische Prüfungen**, die ohne Emulator laufen, und **Tests im Emulator** (Trap-Modus), die einzelne Unterprogramme aufrufen. Beides steht in derselben Datei. Die Oberfläche hat dafür noch keine Ansicht; der Aufruf geschieht auf der Kommandozeile.

    vcc test [ordner|datei.vctest] [--strict] [--backend vice] [--filter TEXT] [--prologue] [--data-dir ordner]

Ohne Pfad wird der aktuelle Ordner durchsucht (alle `*.vctest`, ohne `build`, `target` und versteckte Ordner). **Vorher bauen** (`vcc build`), denn die Tests lesen das gebaute Programm und die `.vcdbg` daneben. Das Beispiel `examples/c64/tests_demo` zeigt beide Stufen.

## Die Testdatei
    {
      "kind": "test", "target": "c64", "program": "build/tests_demo.prg",
      "static": [ { "check": "size", "max": 1000 }, { "check": "overlap" } ],
      "tests":  [ { "name": "…", "call": "next_color", "expect": [ "[$fb] == 0" ] } ]
    }

`program` ist die Programmdatei, relativ zur Testdatei oder zu einem übergeordneten Ordner.

## Statische Prüfungen (`static`)
| `check` | Prüft |
|---|---|
| `size` | Größe in Bytes eines Segments (`segment`) oder aller Segmente: `min`, `max`, und mit `maxEnd` die Adresse hinter dem letzten Byte |
| `zeropage` | `forbid` nennt Adressbereiche (`$02-$1f`), die kein Segment belegen und kein Befehl ansprechen darf |
| `overlap` | Segmente, die sich überlappen |
| `cycles` | die Zyklenspanne ab einer Marke (`label`) oder aus einem Zyklenblock (`block`, im Quelltext `.cycles name` … `.endcycles`) gegen `max`; `level` ist `hint` (Standard) oder `error` |
| `expr` | ein beliebiger Ausdruck, etwa `size("") < $2000 - $0801`, mit optionalem `name` |

Zahlen und Ausdrücke dürfen als Zahl oder als Text (`"$3f"`) stehen; die Sprache ist die der bedingten Haltepunkte (Register, Flags, `[adresse]`, Symbole der `.vcdbg`). Die Zyklenspanne folgt allen Wegen eines Unterprogramms bis `rts` oder `rti`. Bei Schleifen, `jsr`, `jmp (…)` und unbekannten Befehlen meldet sie „nicht bestimmbar“ und rät keine Zahl. Gezählt werden CPU-Zyklen; Abzüge durch DMA (Bad Lines, Sprites), Interrupts und Wartezustände sind nicht enthalten, Seitenübertritte zählen im ungünstigsten Fall.

## Tests im Emulator (`tests`, `--backend vice`)
Ohne `--backend` erscheinen diese Tests als „übersprungen“. Mit `--backend vice` startet der Läufer VICE (nur ein Emulator zugleich), schreibt je Test das Programmabbild und `setup.mem` in den Speicher, setzt die Register, ruft das Unterprogramm über eine Falle im freien RAM auf und wertet `expect` aus.

    { "name": "next_color: 15 wird 0",
      "setup":  { "mem": { "$fb": 15 }, "a": "8", "x": 0 },
      "call":   "next_color",
      "return": "rts",
      "expect": [ "[$fb] == 0", "!fc" ],
      "cycles": { "max": 20 },
      "timeoutMs": 2000 }

- `call`: Marke oder Adresse. `return`: `rts` (Standard) oder `rti` für Interruptroutinen.
- `setup`: `mem` (Adresse auf Byte oder Liste von Bytes), die Register `a x y sp p` und die Flags `fn fv fd fi fz fc` (0 oder 1). Standard: `sp=$FF`, alle Flags gelöscht, I gesetzt.
- `expect`: Ausdrücke wie oben, dazu `cycles` für die gemessene Dauer des Unterprogramms ohne den Aufrufbefehl, einschließlich `rts` oder `rti`. Ein Test ohne `expect` und ohne `cycles` ist ein Fehler („prüft nichts“).
- `cycles.max` ist ein Budget; `level` ist `hint` (Standard) oder `error`, `--strict` macht jeden Hinweis zum Fehler.
- Dateiebene: `trap` (Adresse der Falle, Standard beim C64 `$C000`) und `isolate: true` (VICE startet für jeden Test neu).

Zwischen den Tests werden Programm und Daten neu geschrieben, andere Speicherstellen (Zeropage, Bildschirm, Chips) nicht. Ein Test muss alles, was er braucht, im `setup` setzen. Das Hauptprogramm läuft nicht; was es beim Start einstellt, gehört ins `setup`. Die Zyklenmessung verwendet die Zeitgeber der CIA2 und schreibt dabei in deren Register.

## Ergebnis
| Zustand | Bedeutung |
|---|---|
| OK | alle Erwartungen erfüllt, der Messwert steht dabei |
| FEHLER | Erwartung verletzt (mit linker und rechter Seite) oder `level: error` überschritten |
| HINWEIS | Zyklenbudget überschritten, ohne dass es als Fehler gilt |
| Zeitüberschreitung | die Falle wurde nach `timeoutMs` nicht erreicht |
| übersprungen | der Test lässt sich hier nicht ausführen; der Grund steht dabei (zum Beispiel gelöschtes I-Flag verlangt) |

Der Exit-Code ist 1 bei Fehlern oder Zeitüberschreitungen; Hinweise und übersprungene Tests ändern ihn nicht.

## Postfach-Modus: das Programm prüft sich selbst
Mit `"mode": "mailbox"` in der Testdatei (statt der Aufrufe im Trap-Modus) prüft das Testprogramm sich selbst. Es bindet die Testbibliothek des Ziels ein und schreibt das Ergebnis in ein Postfach im RAM; der Läufer startet das Programm, wartet auf das Ende und liest nur Speicher.

```
.include "c64/test.inc"        ; oder c128/test.inc, atari8bit/test.inc
        test_begin             ; zuerst (verändert A)
        jsr mein_unterprogramm
        assert_a_eq 42         ; A, X, Y und Speicher prüfen; die Register bleiben erhalten
        assert_x_eq 8
        assert_y_eq 0
        assert_mem_eq $fb, $7f
        test_end               ; Ergebnis eintragen, danach Endlosschleife
```

```json
{ "kind": "test", "target": "c64", "program": "build/tests_mailbox.prg", "mode": "mailbox",
  "tests": [ { "name": "Selbsttest", "timeoutMs": 20000, "expect": [ "[$fb] == $7f" ] } ] }
```

- Jeder Eintrag in `tests` ist ein Lauf: ein frisch gestarteter Emulator mit dem Programm (Standard-Zeitlimit 15 s, enthält das Hochfahren). `expect` sind zusätzliche Erwartungen über Speicher und Register nach dem Ende.
- Das Postfach (8 Byte) liegt bei `TEST_MAILBOX`; die Adresse steht in den Zieldaten (`testMailbox`): C64 `$033C` (Kassettenpuffer), C128 `$0B00` (Kassettenpuffer), Atari 8 Bit `$0600` (Seite 6). Plus/4, VC-20 und MEGA65 haben keines; dort wird der Test übersprungen. Das Programm darf das Postfach nicht überschreiben.
- Ergebnis: bestanden (mit der Zahl der Prüfungen), fehlgeschlagen (Nummer der ersten fehlgeschlagenen Prüfung, Art, Ist-Wert und Sollwert), Zeitüberschreitung (Status und bisherige Prüfungen stehen dabei) oder fehlgeschlagen „keine Prüfung ausgeführt“. Zyklen werden in diesem Modus nicht gemessen.
- Warten: Steht im Programm die Endlosschleife von `test_end`, hält der Läufer dort an (auch bei atari800, das sich nicht anhalten lässt). Sonst, oder mit `vcc test --poll`, liest er das Postfach alle 100 ms.
- Geprüft mit VICE (C64) und atari800 (Atari 8 Bit). C128 und Xemu sind noch nicht eingerichtet (Xemu lädt Programme über den Monitor). Der Reiter „Tests“ kennzeichnet die Gruppe mit „· Postfach“; „Diesen Test debuggen“ gibt es dort nicht (es gibt keine Falle, das Programm lässt sich normal debuggen).

## Echte Hardware, JUnit
Postfach-Tests laufen auch auf einer **Ultimate 64 / Ultimate II+** (REST, nur C64-Programme): `vcc test --backend u64:HOST[:PORT] [--password PW] --yes`. Das Gerät lädt das Programm (Reset, Laden, RUN) und der Läufer liest das Postfach bei laufendem Rechner in Abständen von 100 ms bis zum Zeitlimit; danach läuft das Gerät weiter. Trap-Tests werden dort übersprungen, weil die Hardware weder CPU-Register noch Haltepunkte liefert. Das Netzwerkpasswort kommt aus `--password`, sonst aus der Maschine mit derselben Adresse in den Einstellungen. Ohne Adresse (`--backend u64`) wird nichts angesprochen.

Jeder Lauf auf Hardware verlangt eine Bestätigung: `vcc test` fragt im Terminal nach (`--yes` überspringt die Frage, ohne Terminal ist `--yes` Pflicht), der Reiter „Tests“ zeigt bei **jedem** Lauf auf Hardware den Dialog „Test auf echter Hardware starten?“ (Gerät, Hinweis auf Zurücksetzen und Überschreiben, Abbruch nach dem Zeitlimit); die Abfrage lässt sich nicht abschalten. „Läuft auf“ nennt die Geräte aus den Einstellungen. MEGA65-Hardware (`--backend m65:SCHNITTSTELLE[@BAUD]`) ist vorbereitet, aber ungeprüft (Gerät war offline), und hat noch kein Postfach in den Zieldaten.

`vcc test --junit datei.xml` schreibt zusätzlich JUnit-XML für Pipelines: fehlgeschlagen → `failure`, Zeitüberschreitung → `error`, übersprungen → `skipped`; Hinweise und Messwerte stehen als `system-out`.

## Zyklenmessung auf der Ultimate 64 (`"mode": "measure"`)
Mit dem Debug-Strom der Ultimate 64 (Modus „6510 Only“) misst `vcc test` die **Zyklen von Adresse A bis Adresse B** auf echter Hardware und wertet sie gegen ein Budget. Dazu muss am Gerät unter *Data Streams → Debug Stream Mode* „6510 Only“ stehen (die IDE ändert das nicht).

```json
{ "kind": "test", "target": "c64", "program": "build/tests_cycles.prg", "mode": "measure",
  "tests": [ { "name": "Warteschleife", "from": "messen", "to": "ende", "max": 1284, "timeoutMs": 10000 } ] }
```

    vcc test examples/c64/tests_cycles --backend u64:192.168.0.10 --yes

- `from`, `to`: Marke oder Adresse (Ausdruck wie sonst). `max`: Budget (Höchstwert), `min`: Untergrenze; mindestens eines ist Pflicht. `level`: `error` (Standard, ein Urteil) oder `hint`. `timeoutMs`: Zeitlimit nach dem Start des Programms (Standard 20 000).
- Ablauf je Test: Debug-Strom starten (der Bildstrom wird dabei gestoppt, beide gleichzeitig gehen nicht), Gerät zurücksetzen, Programm laden und starten, ab dem Reset Zyklen zählen, Strom stoppen. Es werden weder Konfigurationswerte geändert noch Flash-Speicher beschrieben.
- Gemessen wird vom ersten **Lesezugriff** (Abruf oder Lesen) auf `from` bis zum ersten danach auf `to`, auf den Zyklus genau. Enthalten sind alle Wartezyklen (Bad Lines, Sprite-DMA) **und jeder Interrupt**, der dazwischen läuft. Ein Programm mit Raster-IRQ misst deshalb je nach Zeitpunkt verschieden; mit mehreren Durchläufen steht „a bis b Zyklen“ da und das Budget gilt für den größten Wert. Für genaue Messungen: IRQ sperren (`sei`), Bildschirm ausschalten.
- Eine genommene Verzweigung liest als Blindlesen schon die Adresse hinter dem Befehl (PC+2). Steht `from` oder `to` genau dort, zählt das als Zugriff und die Messung beginnt oder endet zu früh. Marken lieber auf ein Sprungziel oder einen Aufruf legen.
- Ergebnis: **OK** mit gemessenen Zyklen, **FEHLER** (oder **HINWEIS** bei `level: hint`) mit „N Zyklen … überschreiten das Budget von M“, **Zeitüberschreitung**, wenn `to` nach `from` nicht erreicht wurde. **Ungültig** (als „übersprungen“ gemeldet, mit `--strict` ein Fehler) heißt: Pakete des Debug-Stroms gingen verloren oder kamen gar nicht an (Firewall für eingehendes UDP); die Messung wird dann **nie geschätzt**, der Lauf lässt sich wiederholen.
- Nur an der Ultimate 64 (MEGA65 und Ultimate II+ haben keinen Debug-Strom), nur 6502/6510. Die Sicherheitsabfrage vor dem Hardwarelauf bleibt. Im Reiter „Tests“ trägt die Gruppe „· Zyklen“; der Messwert steht beim Test.

## Grenzen
Tests im Emulator laufen nur mit `--backend vice`. Die Standardadresse der Falle und die Zyklenmessung gibt es bisher nur für den C64 (x64sc, PAL); bei den anderen VICE-Systemen muss `trap` in der Testdatei stehen, und Tests mit `cycles` werden übersprungen. atari800, Xemu und Hardware werden nicht unterstützt (`--backend` nimmt nur `vice`).

Alle Befehle von `vcc` im Überblick: [Kommandozeile](help:cli).
