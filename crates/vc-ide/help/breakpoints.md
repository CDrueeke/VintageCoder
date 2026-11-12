# Haltepunkte

Das Panel „Haltepunkte“ listet alle Punkte mit Art, Bereich, Bedingung, Ignorieren und Trefferzähler.

## Ausführung
Im Panel unter „Neu“ Art *Ausführung*, Adresse (`$0810`) oder Symbolname und „Anlegen“. Oder in der Quelle auf die Zeilennummer klicken bzw. im Disassembler auf den Punkt.

## Bedingung
Der Punkt hält nur an, wenn der Ausdruck zutrifft:
```
a == $10
(x & 15) == 5 && fc
[$d020] != 3
peekw($fffe) == $ea31
lin > 100
```
Verfügbar: Register `a x y sp pc p`, Flags `fn fv fb fd fi fz fc`, `lin`/`cyc` (Rasterposition), Speicher `[adr]` / `peek()` / `peekw()`, Symbole der letzten Übersetzung. Operatoren wie in C – Vergleiche binden stärker als `&` und `|`, im Zweifel klammern. Ein Syntaxfehler wird sofort gemeldet; ein Laufzeitfehler hält an.

## Ignorieren
„ignorieren N“ überspringt die ersten N Treffer.

## Datenhaltepunkte
Art *Schreiben*, *Lesen* oder *Zugriff* (Lesen und Schreiben) auf eine Adresse oder einen Bereich (von/bis). Der Emulator hält hinter dem zugreifenden Befehl an. Nur mit VICE; atari800 kennt nur einen Ausführungs-Haltepunkt.

Hinweis: Bedingungen wertet die App nach jedem Halt aus und läuft weiter, wenn sie nicht zutreffen – bei sehr oft getroffenen Stellen kostet das Zeit.
