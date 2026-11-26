# Handprüfung E1.2: Sprachdienste und Editor-Intelligenz

Stand: 2026-11-20 · Branch `claude/ui-e1-2` (aus `claude/ui-e1-1`) · App: `~/Development/abnahme-ui/e1-2/VintageCoder.app` (unsigniert: beim ersten Start Rechtsklick ▸ Öffnen) · Beispielprojekte (nur Kopien) in `~/Development/abnahme-ui/e1-2/beispiele/`.

**Zu wissen:** Dieser Stand baut auf E1.1 auf (der Test von E1.1 Teil 2 läuft getrennt davon im Ordner `e1-1-teil2`). Neu: Diagnosen (Wellenlinie, Fehlerkarte, Probleme), Hover (Befehlshilfe, BASIC-Karten, Zeichenangaben), Vervollständigung, Gehe zu Definition, Gliederung, Projektsuche, Quick-Fix, Prettyprint, Kommentar umschalten, Zeilenumbruch, Zeilenende, Kontextmenü im Editor. **Noch nicht:** Bauen und „Probleme anklicken“ aus dem Bauergebnis (E1.3), Signaturhilfe, Snippets, Einrückungslinien und unsichtbare Zeichen, Kodierung ISO-8859-1, Sitzung wiederherstellen (E1.4). Die Mindestliste der Editorfunktionen steht in `docs/editor-mindestliste.md`.

Vorbereitung: Sprache Deutsch, Tastatur QWERTZ, Projekt öffnen: `…/beispiele/c64/jumprun`. Für viele Punkte: im Baum `+` ▸ neue Datei `probe.asm` anlegen und diesen Text einfügen:
```
.target c64
.org $0801
start:   lda #$00
         sta $d020
         ldx #8
loop:    dex
         bne loop
         jmp start
```
Je Punkt: **ok** oder **Fehler** (Beschreibung, Bild).

## Teil 0: Auslieferungsbau
| Nr | Handlung | Erwartet | Ergebnis |
|---|---|---|---|
| 0 | Mit `lsof -nP -iTCP:1420 -sTCP:LISTEN` prüfen, dass **kein** Entwicklungsserver (Vite) lauscht (Ausgabe leer); die App starten; danach `lsof -nP -i -a -p $(pgrep -f 'VintageCoder.app/Contents/MacOS/vintagecoder')` | Das Fenster zeigt die Oberfläche; die Ausgabe von `lsof` zeigt **keine** Verbindung zu Port 1420 (Abnahme-Apps sind Auslieferungsbauten mit eingebettetem Frontend, gebaut mit `tauri build`; der Bau-Nachweis: ein gleich gebautes Programm startet, rendert und läuft vollständig bei freiem Port 1420) | |

## Teil 0b: E1.1 Rest und Wiederholung

Punkte R13 bis R27 sind Punkte 13 bis 27 der Liste von Teil 2 (`e1-1-teil2`), die noch offen waren; danach die Wiederholung der Befunde und die neuen Punkte.

| Nr | Handlung | Erwartet | Ergebnis |
|---|---|---|---|
| R13 | „Pfad kopieren“ (Reiter oder Baum), danach in ein anderes Programm einfügen | Absoluter Pfad der Datei | |
| R14 | „Im Ordner zeigen“ | Finder öffnet mit der Datei markiert | |
| R15 | Palette ⌘K, Teil eines Dateinamens tippen | Treffer in der Gruppe „Dateien“; Enter öffnet die Datei; Befehle wie Speichern sind jetzt auswählbar, wenn ein Reiter offen ist | |
| R16 | Baumfilter: Lupe über dem Baum, Text eingeben | Nur Treffer (mit ihren Ordnern, aufgeklappt); Esc leert und schließt; „Keine Treffer.“ bei nichts | |
| R17 | Ordner aufklappen, App beenden und neu starten, Projekt wieder öffnen | Aufgeklappte Ordner sind wieder aufgeklappt | |
| R18 | Rechtsklick auf leere Baumfläche ▸ „Alle zuklappen“, „Neu einlesen“ | Alle Ordner zu; Baum liest neu (Datei im Finder anlegen, dann „Neu einlesen“: erscheint) | |
| R19 | Eine offene, **ungeänderte** Datei in einem anderen Programm ändern, zurück in die App | Die App liest sie neu ein (Meldung), Cursor und Marken bleiben | |
| R20 | Dieselbe Datei im Editor ändern (nicht speichern) **und** außerhalb ändern, zurück in die App | Meldung, dass beim Speichern überschrieben wird | |
| R21 | Datei im Finder löschen, zurück in die App | Meldung „außerhalb gelöscht“, Reiter bleibt als ungespeichert stehen | |
| R22 | Nichttext (z. B. aus `graphics/` ein PNG, falls vorhanden) im Baum anklicken | Öffnet im Standardprogramm des Systems, Meldung; kein Reiter | |
| R23 | ⌘O (Datei öffnen) | Systemdialog im Projektordner; Datei im Projekt öffnet als Reiter; Datei außerhalb: Meldung „liegt außerhalb des Projekts“ | |
| R24 | Ungespeicherte Änderung, dann Menü „Startseite“ bzw. „Projekt öffnen“ | Dialog „Ungespeicherte Änderungen“ mit Alle speichern / Verwerfen / Abbrechen; jede Wahl tut, was sie sagt | |
| R25 | Ungespeicherte Änderung, dann ⌘Q bzw. Fensterknopf Schließen | Derselbe Dialog; „Abbrechen“ hält die App offen, „Verwerfen“ beendet | |
| R26 | Datei anklicken, Entf | Nachfrage; Papierkorb (auf macOS immer vorhanden) | |
| R27 | Färbung von Befehlen prüfen: in `.asm` mit 65C02/45GS02-Befehlen (z. B. `bra`, `inz`, `ldq`) | Auch diese sind als Befehle gefärbt (nicht nur die 56 der 6502) | |
| R28–36 | Wiederholung der Befunde aus Block A (Handtest 28 bis 36): Fenster verkleinern auf 900×600 | Zeilen brechen um (Zeilenumbruch ist Standard), keine waagerechte Leiste; Menü Ansicht ▸ „Zeilenumbruch im Editor“ schaltet um | |
| a | Suche (⌘F), nach `;` oder einem Wort suchen, in Hell und Dunkel | Treffer gelb, der aktuelle kräftig gelb mit dunklem Text; gut lesbar, auch über Kommentartext | |
| b | „Gehe zu Zeile“ (⌘L), `3`, Enter; ebenso Befehlspalette schließen, Dialog abbrechen | Danach hat der Editor sofort den Fokus; Tippen geht ohne Klick | |
| c | Falten (Alles falten, Dreiecke im Rand) | Dreieck groß, eingeklappt farbig hervorgehoben, Platzhalter „…“ mit Rahmen | |
| d | Lesezeichen: Umschalt+Klick im Rand; Menü Bearbeiten ▸ „Lesezeichen umschalten“ (⌘⌥K), „Nächstes“ (⌘⌥J), „Voriges“ (⌘⌥H); F2 nur mit Fn (⌘F2 fängt macOS ab) | Blaue Marke in der Zeile; Sprünge laufen rundum | |
| e | Rechtsklick in den Editor | Deutsches Menü: Ausschneiden, Kopieren, Einfügen, Alles auswählen, … Lesezeichen, Haltepunkt, Falten (kein englisches Standardmenü) | |
| f | Linke Spalte einklappen, Maus auf den Streifen | Tooltip erscheint am Mauszeiger, nicht am unteren Rand | |
| g | Datei im Baum umbenennen (F2), die in einer anderen Datei per `.include` eingebunden ist; ebenso einen Ordner | Die Include-Zeile in der anderen Datei wird angepasst (offene Reiter lesen neu, Meldung); Rückgängig im Menü „Baumaktion rückgängig“ stellt wieder her | |
| i | Zen-Modus mit ⌘⌥Z (neues Kürzel; ⌘⇧B kam auf dem Rechner des Inhabers nicht an) und über das Menü Ansicht | Beides schaltet Zen um (Baum und Statuszeile weg) | |
| j | Rechtsklick auf einen Reiter ▸ „Nach rechts“ (auch „Nach links“) | Der Reiter rückt um eine Stelle; es öffnet sich kein weiteres Menü | |
| k | Menü Datei ▸ „Neuer Reiter“, Text tippen, ⌘S | „Speichern unter“ mit Auswahl der Dateiart (Assembler, BASIC, Text, Notiz, **Ohne Endung**); die Endung folgt der Art | |
| l | „Speichern unter“ bzw. „Neue Notiz“, wenn `neu.asm`/`notiz.md` schon im Ordner liegt | Der Vorschlag nimmt einen freien Namen (`neu-2.asm`, `notiz-2.md`) | |
| m | „Neue Datei“: selbst `x.bas` bzw. `x.md` eintippen | Die Art in der Auswahl springt von selbst auf BASIC bzw. Notiz; unbekannte Endung: „Ohne Endung“ | |

## Teil 0c: Nachbesserungen aus dem Test von Abschnitt 0b
| Nr | Handlung | Erwartet | Ergebnis |
|---|---|---|---|
| N1 | Datei mit Kommentarblock am Anfang (z. B. `hal_c64.asm`): Falten | Faltmarke deutlich (18 px, kräftiges Winkelsymbol), gefaltet farbig mit Rahmen | |
| N2 | In Zeile 3 des Kommentarblocks am Zeilenanfang `x` tippen, wieder löschen (mehrfach, auch schnell) | Die Faltmarke steht nach kurzer Pause immer am Anfang des Blocks, nie eine Zeile weiter | |
| N3 | Neue Datei `cpu.asm` mit `bra start`, `inz`, `ldq $1000`, `lda #$00`, `nop` (ohne `.target`/`.org`) | Zeilen mit gepunkteter Unterstreichung (Hinweis, kein Fehler: die Datei gilt als Baustein); Statuszeile „n Hinweise“ (nicht „Keine Probleme“); Karte beim Daraufzeigen mit „Hinweis“, Code und Meldung; Probleme-Liste zeigt die Hinweise | |
| N4 | Dunkles Schema: Hover über eine Direktive, über eine rote Wellenlinie, Vervollständigung (`ld`), Korrigieren (⌘.) | Alle Karten, Listen und Menüs mit dunklem Grund und gut lesbarem Text | |
| N5 | Zen-Modus mit ⌘⌥Z (⌘⇧B kam auf dem Rechner des Inhabers nicht an) | Zen schaltet um | |
| N7 | Vervollständigung öffnen (`ld`), **Tab** | Tab übernimmt den ausgewählten Vorschlag (Enter ebenso); ohne offene Liste fügt Tab wie bisher ein Tabulatorzeichen ein | |
| N8 | `.text "ab{cl` tippen, Vorschlag `clr` mit Tab oder Enter übernehmen | Es entsteht `{clr}` mit **einer** schließenden Klammer | |
| N9 | Kommentar umschalten: ⌘⌥C (auf der deutschen Tastatur ist `/` nur mit Umschalt erreichbar, ⌘/ bleibt zusätzlich); auch im Menü Bearbeiten ▸ „Formatieren“ ▸ „Kommentar umschalten“ und im Rechtsklick-Menü | Zeile bzw. Auswahl wird Kommentar und wieder normal | |
| N6 | Beispielprojekt `jumprun` öffnen, `jumprun.asm` ansehen | Keine Fehler (der Ordner `common/` liegt neben `c64/`, `build/export/` ist vorbereitet) | |

## Teil 1: Diagnosen und Probleme
| Nr | Handlung | Erwartet | Ergebnis |
|---|---|---|---|
| 1 | `probe.asm` öffnen | Keine Wellenlinie; Statuszeile unten: „Keine Probleme“ | |
| 2 | Am Ende eine Zeile `lda nirgendwo` anfügen | Nach kurzer Pause rote Wellenlinie unter dem Fehler, roter Punkt im Rand, Statuszeile „1 Fehler“ | |
| 3 | Mit der Maus auf die Wellenlinie zeigen (kurz warten) | Karte mit „Fehler“, Code (E202) und Meldung „unbekanntes Symbol …“ auf Deutsch; darunter die Befehlshilfe zu `lda` | |
| 4 | Auf „1 Fehler“ in der Statuszeile klicken | Ablage unter dem Editor öffnet mit Reiter „Probleme“: oben die Kurzzeile „Projekt: n Meldungen in m Dateien …“, darunter alle Dateien des Projekts gruppiert (Datei, Zahl) mit ihren Zeilen; Klick springt in den Text. Die **Statuszeile** zählt nur die Datei im Reiter (Fehler, Warnungen, bei Hinweisen „n Hinweise“) | |
| 5 | Name zu `start` ändern | Wellenlinie und Zähler verschwinden von selbst | |
| 6 | Sprache auf Englisch stellen, Fehler wieder erzeugen | Karte, Zähler und Liste englisch („Error“, „unknown symbol“) | |
| 7 | Sprache „Pseudo“ | Texte der Karte sind als Platzhalter erkennbar, nichts bleibt in Deutsch hängen | |
| 8 | Tippen, ohne zu warten (schnell mehrere Zeilen) | Kein Flackern, kein Hängen; Meldungen folgen der Pause | |

## Teil 2: Hover (Befehlshilfe und Karten)
| Nr | Handlung | Erwartet | Ergebnis |
|---|---|---|---|
| 9 | Auf `lda` zeigen | Karte: „LDA – Akkumulator laden“, Flags, Tabelle der Adressierungsarten mit Opcode, Bytes, Zyklen; die gewählte Zeile ist mit ▶ markiert | |
| 10 | Auf `$d020` zeigen | Wert in mehreren Schreibweisen, Registername des Chips (Rand-Farbe) | |
| 11 | Auf eine Marke zeigen (z. B. `loop` im Aufruf `bne loop`) | Wert/Adresse und „Definiert in … Zeile n“ | |
| 12 | Neue Datei `probe.bas` mit `10 poke 53280,0` anlegen; auf `53280` zeigen | Karte mit Hex-Wert und Register „Rahmenfarbe“ (BASIC-Karte) | |
| 13 | In `probe.asm` die Zeile `.text "ab{clr}cd"` anlegen; auf `{clr}` zeigen | Karte: „{clr} – Bildschirm löschen“, Code $93 | |
| 14 | Hell/Dunkel umschalten, Karte öffnen | Karte gut lesbar | |

## Teil 3: Vervollständigung
| Nr | Handlung | Erwartet | Ergebnis |
|---|---|---|---|
| 15 | Am Zeilenanfang (eingerückt) `ld` tippen | Liste: `lda`, `ldx`, `ldy` … mit Kurzbeschreibung; Pfeile wählen, Enter oder Tab übernimmt, Esc schließt | |
| 16 | `jmp lo` tippen | Marke `loop` wird angeboten, Enter übernimmt | |
| 17 | `.` tippen | Direktiven mit Beschreibung; kanonische Namen (z. B. `.align`, `.org`) zuerst | |
| 18 | `.pseudop` tippen | Der alte Name `.pseudopc` erscheint erst jetzt, mit dem Hinweis auf den kanonischen Namen | |
| 19 | `.text "ab{cl` tippen | Zeichennamen (`clr`, …) werden angeboten; Auswahl schließt mit `}` ab | |
| 20 | In einem Kommentar (`; ld`) tippen | Keine Vorschläge | |
| 21 | Strg+Leertaste oder Menü Bearbeiten ▸ „Vervollständigung öffnen“ | Liste öffnet auch ohne Tippen | |

## Teil 4: Definition, Gliederung, Quick-Fix
| Nr | Handlung | Erwartet | Ergebnis |
|---|---|---|---|
| 22 | Cursor auf `start` in `jmp start`; F12 (oder Menü, oder ⌘-Klick auf den Namen) | Cursor springt zur Marke; bei Marke in anderer Datei wird die Datei geöffnet | |
| 23 | Cursor in eine leere Zeile; F12 | Sichtbare Meldung „Keine Definition gefunden.“ | |
| 24 | Ablage ▸ Reiter „Gliederung“ | Marken, Konstanten, Makros der Datei mit Zeilen; Klick springt | |
| 25 | Zeile `.pseudopc $c000` anlegen; Cursor hinein; ⌘. (oder Menü „Korrigieren“) | Kleines Menü „Kanonischen Namen verwenden: .pseudopc → .relocate“; Enter ersetzt; ein ⌘Z nimmt zurück | |
| 26 | Auf den alten Namen zeigen | Karte der Direktive mit der Schaltfläche „Kanonischen Namen verwenden …“ | |
| 27 | ⌘. in einer Zeile ohne Vorschlag | Sichtbare Meldung, kein Absturz | |

## Teil 5: Prettyprint, Kommentar, Anzeige
| Nr | Handlung | Erwartet | Ergebnis |
|---|---|---|---|
| 28 | Zeile `lda   #1      ; wert` tippen, dann mit Pfeil nach oben die Zeile verlassen | Zeile wird gesetzt: Befehl in Spalte 9, Kommentar ab Spalte 41; ein ⌘Z nimmt es zurück | |
| 29 | Menü Bearbeiten ▸ Formatieren ▸ „Datei formatieren“ | Ganze Datei formatiert, Meldung „Formatiert.“; ein einziges ⌘Z nimmt alles zurück; erneut: „Schon formatiert“ | |
| 30 | „Zeile formatieren“ mit Auswahl über mehrere Zeilen | Nur die Zeilen der Auswahl | |
| 31 | In einer BASIC-Datei (`.sbas` oder `.bas`) formatieren | `.bas`: Leerzeichen zusammengefasst; strukturiertes BASIC: Einrückung nach `for`/`next` | |
| 32 | ⌘/ | Zeile wird Kommentar (`;`), erneut hebt es auf; mit Auswahl mehrere Zeilen | |
| 33 | Alt+↑/↓, Umschalt+Alt+↓, ⌘D | Zeile verschieben, duplizieren, nächstes Vorkommen markieren | |
| 34 | Befehl „Zeilenumbruch im Editor“ (Palette ⌘K) | Lange Zeilen brechen um; bleibt nach Neustart erhalten | |
| 35 | Befehl „Zeilenende umschalten“; speichern | Statuszeile zeigt CRLF; die Datei hat Windows-Zeilenenden | |

## Teil 6: Suche im Projekt, Kontextmenü
| Nr | Handlung | Erwartet | Ergebnis |
|---|---|---|---|
| 36 | ⌘⇧F (oder Palette „In Projekt suchen“); `VIC` eingeben | Ablage „Suche“, Treffer nach Datei gruppiert, Zähler; Klick springt zur Zeile | |
| 37 | Während einer langen Suche (großes Projekt) „Abbrechen“ oder Esc | Suche stoppt, Meldung „Abgebrochen nach n Treffern“ | |
| 38 | Text in einer offenen Datei ändern (nicht speichern), nach der Änderung suchen | Der ungespeicherte Text zählt mit | |
| 39 | Rechtsklick in den Editor | Menü: Ausschneiden, Kopieren, Einfügen, Alles auswählen, Gehe zu Definition, Korrigieren, Alles falten/entfalten, Haltepunkt; Pfeile, Enter, Esc | |
| 40 | Menü Ausschneiden/Kopieren/Einfügen | Wirkt wie die Tastenkürzel; schlägt Einfügen fehl, kommt eine sichtbare Meldung | |

## Teil 6b: Ergänzungen aus der Mindestliste
| Nr | Handlung | Erwartet | Ergebnis |
|---|---|---|---|
| 44a | Cursor neben eine Klammer, Ctrl+M (auch in Menü Bearbeiten ▸ Suchen und Ersetzen ▸ „Zur passenden Klammer springen“) | Cursor springt zur Gegenklammer | |
| 44b | Mehrere Zeilen markieren, Palette ⌘K ▸ „Zeilen sortieren“ | Zeilen sortiert (ohne Rücksicht auf Einrückung), ein ⌘Z nimmt zurück; ohne Auswahl Meldung | |
| 44c | Zeilen mit Leerzeichen am Ende; „Leerzeichen am Zeilenende entfernen“ (Auswahl oder ganze Datei) | Entfernt, ein ⌘Z nimmt zurück; Speichern verändert sonst nichts (keine Automatik) | |
| 44d | Mit Fehlern im Text F8 und Umschalt+F8; auf dem Mac ggf. Fn+F8, sonst Palette „Nächstes/Voriges Problem“ | Cursor springt zum nächsten/vorigen Fehler | |

## Teil 7: Robustheit
| Nr | Handlung | Erwartet | Ergebnis |
|---|---|---|---|
| 41 | Datei umbenennen, in anderen Ordner ziehen | Diagnosen, Hover und Gliederung funktionieren im neuen Namen weiter | |
| 42 | Datei von außen ändern (anderes Programm), zurück ins Fenster | Neu gelesen; Diagnosen folgen | |
| 43 | Datei mit 500 Zeilen (`jumprun.asm`) öffnen und tippen | Tippen bleibt flüssig; Meldungen kommen mit Verzögerung | |
| 44 | Projekt wechseln | Alte Meldungen, Suchtreffer, Ablage sind weg | |
