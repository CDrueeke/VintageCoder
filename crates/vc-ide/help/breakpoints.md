# Haltepunkte

Die Liste **Haltepunkte** links im Bereich **Debug** zeigt alle Punkte der Sitzung mit Art (Ausführen, Lesen, Schreiben, Zugriff), Adresse oder Bereich, Bedingung (`if …`), zu ignorierenden Treffern (`×N`) und Trefferzähler. Ein ✕ entfernt einen Punkt. Haltepunkte aus dem Editor stehen blass mit der Marke **Editor** und `datei:zeile` dabei.

## Ausführung
In der Quelle in den Rand neben der Zeilennummer klicken, oder den Cursor in die Zeile setzen und **F9** drücken (Menü **Bearbeiten ▸ Haltepunkt umschalten**). Im Disassembler setzt ein Klick auf den runden Knopf vor einer Zeile einen Punkt. Haltepunkte im Editor gelten schon vor dem Start; VintageCoder ordnet sie über die Zeilenzuordnung des letzten Baus einer Adresse zu. **Debuggen: Bis hierher ausführen** (Menü **Projekt ▸ Debuggen**) setzt einen einmaligen Punkt auf die Zeile am Cursor und läuft weiter.

## Bedingung, Ignorieren und Datenhaltepunkte
Unter der Liste legt ein Formular Punkte an:

- **Art des Punkts:** **Ausführen**, **Lesen**, **Schreiben** oder **Zugriff**. Die drei letzten sind Datenhaltepunkte (nur mit VICE) und sonst nicht wählbar.
- **Adresse:** `$D020`, `0x0400` oder `2049` (kein Symbolname).
- **Ende (bei Bereich):** nur bei Datenhaltepunkten; leer heißt eine einzelne Adresse.
- **Bedingung:** ein Ausdruck, zum Beispiel `a == 5`; der Punkt hält nur an, wenn er stimmt.
- **Ignorieren:** überspringt die ersten N Treffer.
- **Hinzufügen** legt den Punkt an. Ein Fehler in der Bedingung erscheint als rote Meldung unter der Liste.

## Ausdrücke
Dieselbe Ausdruckssprache gilt bei **Beobachten**, bei Bedingungen und in den Tests:
```
a == $10
(x & 15) == 5 && fc
[$d020] != 3
peekw($fffe) == $ea31
lin > 100
```
Verfügbar: Register `a x y sp pc p`, Flags `fn fv fb fd fi fz fc`, `lin`/`cyc` (Rasterposition), Speicher `[adr]` / `peek()` / `peekw()`, Symbole der letzten Übersetzung. Operatoren wie in C – Vergleiche binden stärker als `&` und `|`, im Zweifel klammern. Ein Syntaxfehler wird sofort gemeldet; ein Laufzeitfehler hält an.

Hinweis: Bedingungen wertet die App nach jedem Halt aus und läuft weiter, wenn sie nicht zutreffen – bei sehr oft getroffenen Stellen kostet das Zeit. atari800 kennt nur einen Ausführungs-Haltepunkt.
