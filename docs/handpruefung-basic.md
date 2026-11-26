# Handprüfung BASIC im Editor (Sprachdienste, Apostroph, Erklärtexte)

Gilt für Branch `claude/basic-ui-3`. Beispielprojekt: `examples/c64/basic_native` (Arbeitskopie). Bilder aus den Ablauftests: `~/Development/abnahme-ui/basic-ui-sprachdienste/` und `~/Development/abnahme-ui/basic-ui-erklaertexte/`.
Die Ablauftests (`e2e-basicinfo`, `e2e-basic`) prüfen schon die Funktion; hier geht es um den Eindruck am echten Bedienen. Höchstens sechs Punkte:

| # | Schritte | Erwartet | Ergebnis |
|---|---|---|---|
| 1 | Datei mit `#define STERNE 5`, `#if SYSTEM_C128 … #else … #endif` und `#asm "code.asm"` öffnen; Statuszeile ansehen | Unten steht der Dialekt („BASIC · BASIC V2 · LF“); der nicht geltende Zweig (hier der C128-Zweig) ist grau, die Zeilen `#if`/`#else`/`#endif` bleiben normal | |
| 2 | Maus über `STERNE` in der Programmzeile, dann über `#if` und über `#asm` | Karten: bei `STERNE` die Definition mit Wert und Zeile; bei den Direktiven eine kurze Erklärung. In einer Zeichenkette erscheint nichts | |
| 3 | Schublade ▸ Gliederung | Marken (`@start`) und `#define`-Namen stehen in der Liste; Klick springt zur Zeile | |
| 4 | Zeile mit einem Hochkomma schreiben (`PRINT "A"  ' Kommentar`) in einem Commodore-Projekt, dann in einem Atari-Projekt | Commodore: das Hochkomma ist kein Kommentar (nicht grau gefärbt, Warnung B167 beim Bauen); Atari: Verhalten wie bisher. Falten und Aufräumen („Format“) lassen die Zeile heil | |
| 5 | Programm mit `FOR` ohne `NEXT` bauen (F7), Schublade ▸ Probleme | Die Warnung trägt ihre Kennung (B151); Überfahren der Kennung zeigt den Erklärtext, Klick klappt ihn unter der Zeile auf (zweiter Klick zu); die Erklärung sagt, ob sich die Meldung abschalten lässt. Rechtsklick auf die Zeile bietet „Erklärung zu B151 anzeigen“ | |
| 6 | Projekt ▸ BASIC: Meldung B151 unter „Abgeschaltete Meldungen“ eintragen, speichern, erneut bauen | Die Warnung B151 erscheint nicht mehr; ein Fehler (z. B. B162) lässt sich nicht abschalten, sein Erklärtext sagt das | |

Hinweise zum Eindruck: Sind die Erklärungen verständlich und kurz genug? Stört das gestrichelte Unterstreichen der Kennung? Die englischen Texte sind maschinell übersetzt und nicht abgenommen; für 0.3.0 zählt das Deutsche.
