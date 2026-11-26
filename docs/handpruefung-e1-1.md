# Handprüfung E1.1 (Teil 1): Editor-Kern

Stand: 2026-11-20 · Branch `claude/ui-e1-1` · App: `~/Development/abnahme-ui/e1-1/VintageCoder.app` (unsigniert: beim ersten Start Rechtsklick ▸ Öffnen) · Beispielprojekte (nur Kopien) in `~/Development/abnahme-ui/e1-1/beispiele/`.

**Zu wissen:** Dieser Stand enthält den Kern des Code-Editors: Dateien aus dem Projektbaum öffnen, Reiter, Bearbeiten mit Färbung (Assembler, BASIC), Rückgängig, Suchen und Ersetzen, Gehe zu Zeile, Faltung, Haltepunkte und Lesezeichen am Rand, Speichern, Dateiverwaltung im Baum (Neu, Umbenennen, Löschen, Ziehen). **Noch nicht** enthalten (siehe Paritätsmatrix, Zeilen „offen“): Art des Reiters umschalten, neue Notiz, Zen-Modus, Schriftgröße, Baumfilter, Pfad kopieren, „Im Ordner zeigen“, Umbenennen mit Include-Anpassung, Änderung von außen erkennen, nicht-textliche Dateien in anderen Programmen öffnen, Sitzung wiederherstellen (E1.4), Sprachdienste (E1.2).

Vorbereitung: Prüfsumme der Kopien vor dem Test merken:
```bash
cd ~/Development/abnahme-ui/e1-1/beispiele && find . -type f -exec shasum {} + | sort -k2 | shasum
```
Sprache zu Beginn: Deutsch. Tastatur: deutsch (QWERTZ). Projekt öffnen: `…/beispiele/c64/jumprun`.

Je Punkt: **ok** oder **Fehler** (Beschreibung, Bild).

## Teil 1: Nachbesserungen aus dem Test E0

| Nr | Handlung | Erwartet | Ergebnis |
|---|---|---|---|
| 1 | Am leeren Bereich der Titelleiste doppelklicken (mehrmals) | Fenster wird einmal groß bzw. wieder klein, **kein** Flackern, kein sofortiges Zurückspringen | |
| 2 | Startansicht (kein Projekt) mit mehr als 5 früheren Projekten | Die Zuletzt-Liste zeigt höchstens 5 Einträge | |
| 3 | Projekt öffnen, linke Spalte mit ⌘B einklappen | Am linken Rand bleibt ein schmaler Streifen (14 px); Hover zeigt den Tooltip mit ⌘B; Klick klappt die Spalte aus | |
| 4 | Prüfsumme (siehe oben) zweimal bilden | Beide Male derselbe Wert | |

## Teil 2: Öffnen, Reiter, Färbung

| Nr | Handlung | Erwartet | Ergebnis |
|---|---|---|---|
| 5 | Datei `hal_c64.asm` im Baum anklicken | Reiter „hal_c64.asm“ öffnet; Text mit Zeilennummern; Kommentare grau kursiv, Befehle (lda, sta …) kräftig, Zahlen orange, Marken blau, Direktiven rosa | |
| 6 | Weitere Dateien öffnen (`jumprun.asm`, `README.md`, in `lib/` eine `.inc`) | Je ein Reiter; wieder anklicken wechselt nur; `README.md` ist ungefärbt | |
| 7 | Reiter mit Pfeil links/rechts wechseln (Fokus auf Reiter); Reiter ziehen und neu ordnen | Wechsel klappt, Reihenfolge ändert sich, Auswahl bleibt | |
| 8 | Im Editor an eine Stelle klicken und tippen; zwischen Reitern wechseln und zurück | Cursor, Verlauf (Rückgängig) und Faltung bleiben je Reiter erhalten | |
| 9 | Hell/Dunkel umschalten | Färbung bleibt gut lesbar, Auswahl und aktive Zeile sichtbar | |
| 10 | Sprache auf Englisch und auf „Pseudo“ umstellen | Menü, Reiter-Tooltips, Dialoge und die Suchleiste wechseln mit | |

## Teil 3: Bearbeiten (Text, Tastatur)

| Nr | Handlung | Erwartet | Ergebnis |
|---|---|---|---|
| 11 | Am Dateiende tippen: `; Prüfung äöüß € 😀` | Alle Zeichen erscheinen richtig (auch tote Tasten: `´` + `e` = é, `^` + `a` = â); Reiter zeigt einen Punkt (ungespeichert) | |
| 12 | Enter nach einer eingerückten Zeile | Neue Zeile übernimmt die Einrückung | |
| 13 | Tab, Umschalt+Tab (auch mit markierten Zeilen) | Tab fügt ein Tabulatorzeichen ein bzw. rückt Auswahl ein; Umschalt+Tab rückt aus | |
| 14 | Markieren (Maus, Umschalt+Pfeile, Doppelklick = Wort, ⌘A), Ausschneiden, Kopieren, Einfügen (⌘X, ⌘C, ⌘V und Menü Bearbeiten) | Wie erwartet; Zwischenablage klappt auch in andere Programme | |
| 15 | ⌘Z und ⌘⇧Z mehrfach (Tastatur und Menü Bearbeiten) | Schrittweise zurück und wieder vor; nach Rückgängig bis zum Anfang verschwindet der Punkt am Reiter | |
| 16 | Pos1/Ende, ⌘←/⌘→, Seite auf/ab, ⌘↑/⌘↓ | Cursor springt (Pos1 erst auf das erste Zeichen, dann an den Zeilenanfang) | |
| 17 | Klammern und Anführungszeichen tippen: `(`, `"` | Schließendes Zeichen wird ergänzt; passende Klammer wird beim Cursor hervorgehoben | |
| 18 | Statuszeile unten beachten | „Zeile n, Spalte m“ (und „k markiert“) folgt dem Cursor | |

## Teil 4: Suchen, Gehe zu Zeile, Falten, Rand

| Nr | Handlung | Erwartet | Ergebnis |
|---|---|---|---|
| 19 | ⌘F (oder Menü Bearbeiten ▸ Suchen) | Suchleiste oben im Editor, deutsch beschriftet; Eingabe markiert alle Treffer; Weiter/Zurück/Alle; Groß/Klein, Regulärer Ausdruck, Ganzes Wort | |
| 20 | ⌘⌥F oder Ersetzen-Feld: ein Wort ersetzen, dann „Alle ersetzen“; ⌘Z | Ersetzt wie erwartet; Rückgängig stellt zurück | |
| 21 | ⌘L (Gehe zu Zeile): `0`, dann `3` | Bei `0` Fehlermeldung im Dialog; bei `3` springt der Cursor in Zeile 3 | |
| 22 | Dreiecke links neben den Zeilennummern anklicken (z. B. bei Marken und Kommentarblöcken); Menü Bearbeiten ▸ „Alles falten/entfalten“ | Bereiche falten und entfalten; gefaltete Stelle zeigt „…“ | |
| 23 | Klick in den Rand links der Zeilennummern; Umschalt+Klick; F9 | Roter Punkt (Haltepunkt) bzw. blaue Marke (Lesezeichen) in der Zeile; erneuter Klick entfernt sie; F9 schaltet den Haltepunkt der Cursorzeile; ⌘F2 setzt ein Lesezeichen, F2/Umschalt+F2 springen zum nächsten/vorigen. Marken wandern mit, wenn Zeilen davor eingefügt werden | |

## Teil 5: Speichern und Dateiverwaltung

| Nr | Handlung | Erwartet | Ergebnis |
|---|---|---|---|
| 24 | Text ändern, ⌘S (und Menü Datei ▸ Speichern); danach alles per ⌘Z zurücknehmen und erneut speichern; Prüfsumme (Teil 1, Nr. 4) bilden | Punkt verschwindet nach dem Speichern; nach dem Zurücknehmen und Speichern ist die Prüfsumme wieder **gleich dem Anfangswert** (Datei byte-gleich, Zeilenenden und Endzeile unverändert) | |
| 25 | Zwei Reiter ändern, ⌘⌥S (Alle speichern) | Beide Punkte verschwinden | |
| 26 | Reiter mit Änderung schließen (⌘W oder ×) | Nachfrage Speichern / Verwerfen / Abbrechen; jede Wahl tut, was sie sagt | |
| 27 | Knopf „+“ über dem Baum; Name `lib/neu.asm` eingeben | Datei wird angelegt (Ordner `lib` dabei, klappt im Baum auf), neuer Reiter ist offen; leerer Name und Zeichen wie `:` werden im Dialog gemeldet | |
| 28 | Datei im Baum wählen, F2 (oder Rechtsklick ▸ Umbenennen); Ordner ebenso | Name ändert sich im Baum, der offene Reiter heißt jetzt neu, Inhalt bleibt; vorhandener Name wird gemeldet | |
| 29 | Datei aus dem Baum in einen anderen Ordner ziehen, dann in die leere Fläche unter dem Baum (= Wurzel) | Datei zieht um (Reiter und Baum folgen); ein Ordner lässt sich nicht in sich selbst ziehen | |
| 30 | Datei im Baum wählen, Entf (oder Rechtsklick ▸ Löschen) | Nachfrage; nach „Löschen“ liegt die Datei im Papierkorb, der Reiter schließt, Meldung erscheint | |
| 31 | Rechtsklick im Baum auf Datei, Ordner und leere Fläche; Menütaste/Umschalt+F10 | Passende Einträge (Öffnen, Neue Datei, Neuer Ordner, Umbenennen, Löschen); Pfeiltasten, Esc | |
| 32 | Projektdatei: Datei, die in `jumprun.vcprj` steht, umbenennen; `jumprun.vcprj` ansehen | Der Verweis in der Projektdatei wurde mitgezogen | |
| 33 | Projekt wechseln (Startseite, anderes Projekt öffnen) | Reiter des alten Projekts sind weg; (ungespeicherte Änderungen fragen noch **nicht** nach: bekannte Lücke) | |

## Teil 6: Rand- und Fehlerfälle

| Nr | Handlung | Erwartet | Ergebnis |
|---|---|---|---|
| 34 | Binärdatei (z. B. aus `graphics/`) im Baum anklicken | Verständliche Meldung („keine UTF-8-Textdatei …“), kein Absturz | |
| 35 | Datei mit Windows-Zeilenenden (CRLF) und ohne Endzeile öffnen und speichern (selbst anlegen) | Unverändert nach dem Speichern (byte-gleich) | |
| 36 | Große Datei (`jumprun.asm`, rund 500 Zeilen und mehr) scrollen, tippen | Flüssig; Färbung folgt dem Tippen mit kurzer Verzögerung | |
| 37 | Fenster verkleinern (900 × 600) | Kein Überlauf, Reiterleiste scrollt waagerecht | |

Bekannte Lücken dieses Teilschritts stehen in `docs/abnahmepaket-e1-1.md`.
