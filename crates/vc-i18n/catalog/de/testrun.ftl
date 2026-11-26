# SPDX-License-Identifier: GPL-3.0-or-later
# Testläufer (Ergebnisse von vcc test und vom Test-Panel der IDE)

# ---- Ablaufprotokoll (trace.rs)
testrun-trace-analyzer-crashed = Auswertung abgestürzt
testrun-trace-no-measure = keine Messung
testrun-trace-header = # Zyklus  PC  Zyklen  Quelle (Zyklen zählen je Epoche; nach einem Paketverlust beginnt eine neue)
testrun-trace-epoch = # --- Paketverlust: neue Epoche { $epoch } ---
testrun-trace-writer-crashed = Schreib-Thread abgestürzt

# ---- Lauf und Anzeige (suite.rs)
testrun-static-title = Statische Prüfungen
testrun-suffix-mailbox = · Postfach
testrun-suffix-measure = · Zyklen
testrun-hw-ultimate-c64-only = Die Ultimate-Hardware spielt nur C64-Programme
testrun-hw-ultimate-prg-only = Auf der Ultimate lassen sich nur .prg-Dateien starten
testrun-hw-mega65-only = Die MEGA65-Hardware spielt nur MEGA65-Programme
testrun-launch-vice-target = VICE kennt das Ziel { $target } nicht
testrun-launch-atari800-target = atari800 kennt nur das Ziel atari8, nicht { $target }
testrun-launch-fujisan-target = Fujisan kennt nur das Ziel atari8, nicht { $target }
testrun-launch-xemu = Xemu lädt Programme über den Monitor: der Postfach-Modus ist dafür noch nicht eingerichtet
testrun-launch-unknown-emulator = Emulator „{ $x }“ ist unbekannt
testrun-mark-note = Hinweis{ $m }: { $name }
testrun-mark-error = Fehler: { $name }{ $m }
testrun-mark-timeout = Zeitüberschreitung: { $name }
testrun-mark-skipped = übersprungen: { $name }
testrun-skip-not-built = das Programm ist nicht gebaut oder nicht lesbar (Bauen, dann erneut ausführen)
testrun-skip-trap-on-hardware = Trap-Modus nicht verfügbar: die Hardware liefert keine CPU-Register und keine Haltepunkte. Als Postfach-Test ("mode": "mailbox", test.inc) neu bauen
testrun-skip-measure-needs-u64 = Zyklenmessung (mode measure) braucht die Ultimate 64: „Läuft auf“ auf das Gerät stellen bzw. vcc test --backend u64:HOST
testrun-skip-no-emulator = kein Emulator gewählt: „Läuft auf“ in der Kopfleiste
testrun-skip-emulator-unsupported = Emulator „{ $b }“ wird vom Läufer noch nicht unterstützt
testrun-debug-mailbox = Im Postfach-Modus gibt es keine Falle: das Programm lässt sich normal debuggen (F6)
testrun-debug-trap-target = Debuggen im Trap-Modus gibt es für das Ziel { $target } noch nicht

# ---- Trap-Modus (runner.rs)
testrun-mem-unreadable = Speicher nicht lesbar
testrun-mem-unreadable-here = Speicher ist hier nicht lesbar
testrun-quoted-error = „{ $src }“: { $e }
testrun-byte-range = { $what }: { $v } passt nicht in ein Byte
testrun-image-unreadable = Programmabbild nicht lesbar: { $e }
testrun-address-range = { $what }: Adresse { $v } außerhalb 0..65535
testrun-trap-unknown = für das Ziel „{ $target }“ ist keine Trap-Adresse bekannt: im Test "trap" angeben
testrun-trap-in-program = die Falle bei { $trap } liegt im Programm: im Test "trap" eine freie Adresse angeben
testrun-emulator-start-failed = Emulator nicht startbar: { $e }
testrun-cancelled = abgebrochen
testrun-skip-iflag = der Test verlangt ein gelöschtes I-Flag: im Trap-Modus laufen Tests mit gesetztem I-Flag (sonst erreicht VICE die Falle nicht zuverlässig)
testrun-hang = { $what } nach { $ms } ms nicht erreicht: das Unterprogramm kehrt nicht zurück; der Emulator wird neu gestartet
testrun-hang-what-prologue = der Vorspann der Falle
testrun-hang-what-return = die Rückkehr
testrun-cycles-not-requested = nicht angefordert
testrun-cycles-no-support = dieser Emulator misst keine Zyklen
testrun-cycles-unmeasurable = nicht messbar: { $e }
testrun-cycles-beam-near-frame = nicht messbar: Strahlzeit { $beam } liegt nahe an einem Bild ({ $frame } Zyklen), die Position wiederholt sich
testrun-cycles-less-than-call = nicht messbar: { $n } Zyklen sind weniger als der Aufrufbefehl
testrun-cycles-mismatch = nicht messbar: Zeitgeber ({ $timer } Zyklen) und Strahlposition ({ $beam } mod { $frame }) stimmen nicht überein (Strahlposition)
testrun-cycles-timer-unreadable = nicht messbar: Zeitgeber nicht lesbar
testrun-expect-parse = expect „{ $src }“: { $e }
testrun-expect-failed = Erwartung „{ $src }“ ist nicht erfüllt
testrun-expect-compare = : linke Seite = { $lv } ({ $lh }), rechte Seite = { $rv } ({ $rh }), verlangt wurde { $op }
testrun-expect-unevaluable = Erwartung „{ $src }“ nicht auswertbar: { $e }
testrun-expect-cycles-why = „{ $src }“: Zyklen { $why }
testrun-state-on-return = Zustand bei der Rückkehr: { $regs }
testrun-budget-cycles-why = Zyklenbudget: Zyklen { $why }
testrun-budget-exceeded = { $n } Zyklen gemessen{ $sfx }, erlaubt: höchstens { $limit } – { $over } zu viel
testrun-budget-beam-note = (Strahlzeit: mit Speicherauffrischung und DMA, nie unter den CPU-Zyklen)
testrun-measured-cycles = { $n } Zyklen
testrun-measured-cycles-beam = { $n } Zyklen (Strahlzeit)
testrun-test-not-found = Test „{ $test }“ nicht gefunden

# ---- Postfach-Modus (mailbox.rs)
testrun-kind-mem = assert_mem_eq (Speicher)
testrun-kind-unknown = unbekannte Art
testrun-mailbox-none-for-target = für das Ziel „{ $target }“ ist in den Zieldaten kein Postfach (testMailbox) festgelegt
testrun-mailbox-unreadable = Postfach nicht lesbar: { $e }
testrun-mailbox-state = Status { $status }, { $count } Prüfung(en) bisher
testrun-mailbox-state-unknown = unbekannt
testrun-mailbox-hint-status0 = (Status 0: test_begin wurde nie ausgeführt, das Programm startet nicht oder hängt vorher)
testrun-mailbox-timeout = das Testprogramm endete nach { $ms } ms nicht: { $st }{ $hint }; der Emulator wird beendet
testrun-mailbox-incomplete = Postfach unvollständig gelesen
testrun-mailbox-fail = Prüfung Nr. { $number } von { $of } fehlgeschlagen ({ $kind }): Ist-Wert { $actual } ({ $actualhex }), erwartet { $expected } ({ $expectedhex })
testrun-mailbox-no-checks = das Programm lief durch, hat aber keine Prüfung ausgeführt (assert_… fehlen)
testrun-mailbox-unfinished = Postfach zeigt nach dem Ende Status { $status }: das Programm hat das Postfach überschrieben oder test_end nicht erreicht
testrun-mailbox-expect-unmet = Erwartung „{ $src }“ ist nach dem Ende nicht erfüllt
testrun-mailbox-expect-after = alle { $checks } Prüfungen des Programms bestanden, aber { $b }
testrun-mailbox-checks =
    { $n ->
        [one] { $n } Prüfung
       *[other] { $n } Prüfungen
    }

# ---- Zyklenmessung auf der Hardware (cycles.rs)
testrun-measure-mem-unreadable = Speicher ist bei einer Hardware-Messung nicht lesbar
testrun-measure-addr-range = „{ $src }“ liegt mit { $v } außerhalb von $0000..$FFFF
testrun-measure-text-range = { $a } bis { $b } Zyklen ({ $n } Messungen)
testrun-measure-text-one = { $a } Zyklen
testrun-measure-span = { $from } bis { $to }
testrun-measure-lost = , { $n } Paket(e) verloren
testrun-measure-invalid-no-packets = ungültig: keine Pakete des Debug-Stroms empfangen (Firewall für eingehendes UDP, Modus „6510 Only“ am Gerät, Netzwerk?); nichts geschätzt
testrun-measure-invalid-loss = ungültig: Paketverlust während der Messung { $span } ({ $n } Messung(en) verworfen{ $lost }); nichts geschätzt, Lauf wiederholen
testrun-measure-not-reached-after = { $span } wurde in { $ms } ms nicht durchlaufen: { $from } erreicht, { $to } danach nicht
testrun-measure-never-reached = { $span } wurde in { $ms } ms nicht durchlaufen: { $from } nie erreicht
testrun-measure-over = { $hi } Zyklen von { $span } überschreiten das Budget von { $mx }
testrun-measure-under = { $lo } Zyklen von { $span } unterschreiten die Untergrenze von { $mn }
testrun-measure-extra-invalid = ({ $n } weitere Messung(en) wegen Paketverlust ungültig und verworfen)
testrun-measure-addr-unevaluable = Adresse nicht auswertbar: { $e }
testrun-measure-limit-unevaluable = Grenzwert nicht auswertbar: { $e }
testrun-measure-impossible = Messung nicht möglich: { $e }
testrun-measure-u64-only = Der Debug-Strom (Zyklenmessung) ist nur an der Ultimate 64 verfügbar

# Name einer Hardware in Anführungszeichen (Ziel-Anzeige)
testrun-hw-quoted-name = „{ $name }“
