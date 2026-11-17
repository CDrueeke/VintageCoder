# Haltepunkte

Die Spalte „Haltepunkte“ im Reiter **Debug** der Schublade listet alle Punkte der Sitzung mit Art (`exec` für Ausführung, bei Datenhaltepunkten die Zugriffsart), Ort, Bedingung (`wenn …`) und Trefferzähler. Ein ✕ entfernt einen Punkt.

## Ausführung
In der Quelle in den Rand neben der Zeilennummer klicken, oder den Cursor in die Zeile setzen und **F9** drücken (Rechtsklickmenü: „Haltepunkt umschalten“). Im Disassembler setzt ein Klick auf den Rand vor einer Zeile einen Punkt. Haltepunkte im Editor gelten schon vor dem Start; VintageCoder ordnet sie über die Zeilenzuordnung des letzten Baus einer Adresse zu. **Debuggen: Bis hierher ausführen** (Menü Projekt, Rechtsklickmenü im Editor) setzt einen einmaligen Punkt auf die Zeile am Cursor und läuft weiter.

## Bedingung, Ignorieren und Datenhaltepunkte
Bedingte Haltepunkte (hält nur an, wenn ein Ausdruck stimmt), „ignorieren N“ (überspringt die ersten N Treffer) und Datenhaltepunkte (Schreiben, Lesen, Zugriff auf eine Adresse oder einen Bereich, nur mit VICE) kann der Debugger im Kern. Ein Formular, sie in der Oberfläche anzulegen, gibt es **noch nicht**. Bedingungen in der Liste werden angezeigt.

## Ausdrücke
Dieselbe Ausdruckssprache gilt bei **Beobachten** und in den Tests:
```
a == $10
(x & 15) == 5 && fc
[$d020] != 3
peekw($fffe) == $ea31
lin > 100
```
Verfügbar: Register `a x y sp pc p`, Flags `fn fv fb fd fi fz fc`, `lin`/`cyc` (Rasterposition), Speicher `[adr]` / `peek()` / `peekw()`, Symbole der letzten Übersetzung. Operatoren wie in C – Vergleiche binden stärker als `&` und `|`, im Zweifel klammern. Ein Syntaxfehler wird sofort gemeldet; ein Laufzeitfehler hält an.

Hinweis: Bedingungen wertet die App nach jedem Halt aus und läuft weiter, wenn sie nicht zutreffen – bei sehr oft getroffenen Stellen kostet das Zeit. atari800 kennt nur einen Ausführungs-Haltepunkt.
