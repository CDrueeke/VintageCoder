# Handprüfung Bereich „Projekt“ (P1 Navigation, P2 Übersicht/Allgemein/Ziele, P3 Quellen/Bauen und Starten/Weitergeben)

Start: `start-projekt.command` im Paket (öffnet `arbeitskopie/jumprun` direkt auf der Seite Projekt ▸ Übersicht); `start-projekt-quellen.command` öffnet Projekt ▸ Quellen (`--view projekt:quellen`).

| # | Schritte | Erwartet | Ergebnis |
|---|---|---|---|
| 1 | Obere Navigation ansehen | **Projekt** steht als erster Bereich vor Code, Grafik, Musik, Daten, Debug; der Bereich ist auch ohne Projekt sichtbar | |
| 2 | Auf Projekt klicken | Zweite Reihe mit sieben Seiten: Übersicht, Allgemein, Ziele, Quellen, Bauen und Starten, Diskette, Weitergeben; die aktuelle ist hervorgehoben | |
| 3 | Seiten nacheinander öffnen; Pfeiltasten in der zweiten Reihe | Titel wechselt („Projekt: Ziele“ …); die Seiten sind in P1 noch Rahmen mit Hinweis und Knöpfen zu den anderen Seiten; Pfeile wechseln die Seite | |
| 4 | Projekt ▸ Projekt-Einstellungen … (Menü oder Befehlspalette) | Öffnet den Bereich bei der zuletzt benutzten Seite (anfangs Allgemein); der frühere Extra-Reiter entfällt | |
| 5 | Befehlspalette: „Projekt“ tippen | Alle sieben Seiten sind als Orte auffindbar | |
| 6 | Turbo-BASIC-XL-Projekt: Seite **Diskette** | Die Turbo-Karte (früher in den Projekt-Einstellungen) steht hier; sonst Hinweis „kein Turbo-Projekt“ | |
| 7 | Ohne Projekt auf Projekt klicken | Text erklärt, dass zuerst ein Projekt zu öffnen ist | |
| 8 | Sprache Englisch | Project, Overview, General, Targets, Sources, Build and run, Disk, Share | |
| 9 | VoiceOver: Projekt-Reihe | Gruppe „Unterbereiche“, aktuelle Seite als „aktuell“ angesagt | |

| 10 | Projekt ▸ **Übersicht** | Vier Karten (Allgemein, Ziele, Quellen, Diskette) mit Punkt und Zustandswort (in Ordnung, Hinweis, Angabe fehlt, nicht eingeschaltet) und „Ändern ›“, das zur Seite springt; unten ggf. Probleme der Projektdatei | |
| 11 | **Allgemein**: Namen ändern und aus dem Feld gehen | „Gespeichert.“, der Name in der Kopfzeile folgt sofort; die .vcprj-Datei enthält den neuen Namen | |
| 12 | Namen leeren und aus dem Feld gehen | Meldung „Nicht gespeichert: …“, das Feld zeigt wieder den alten Namen, die Datei ist unverändert | |
| 13 | Beschreibung ändern; Start-Programm wählen; wieder „Automatisch“ | Jeweils sofort gespeichert; bei „Automatisch“ steht kein `main` in der Datei | |
| 14 | **Ziele**: Variante PAL/NTSC umschalten | Wird sofort in der Projektdatei gespeichert; Systeme ohne Varianten sagen es | |
| 15 | VoiceOver: Karten der Übersicht | Jede Karte ist eine Gruppe mit Überschrift „Name, Zustand“; „Ändern ›“ ist ein Knopf | |

| 16 | **Quellen** (`start-projekt-quellen.command`) | Programme je Quellgruppe in Baureihenfolge, Start-Programm mit ★, fehlende Dateien rot mit Wort „fehlt“; Klick auf den Namen öffnet die Datei im Code-Editor; ↑/↓ verschieben innerhalb der Gruppe (bei mehreren Dateien), „Als Start-Programm“ setzt `main` | |
| 17 | **Bauen und Starten**: Bauen, Starten | Karte „Letzter Bau“ nennt Uhrzeit und Ergebnis; „Emulator je Ziel“ zeigt Überschreibungen des Projekts oder „aus den Einstellungen“; Karte „Pakete (Registry)“ mit Herkunft | |
| 18 | **Weitergeben**: „Projekt kopieren …“; Liste „Zuletzt geöffnet“ | Kopieren-Dialog öffnet; Verlauf mit Öffnen-Knopf; Hinweis auf den Export in den Editoren | |
| 19 | **Turbo-Projekt** (`start-projekt-diskette-turbo.command`, Arbeitskopie turbo_basic): Projekt ▸ Diskette | Die Turbo-Karte steht hier. Ohne eigene Turbo-Diskette erscheint der Hinweis zum Bereitstellen der Diskette (Assistent „Diskette bereitstellen“) | |
| 20 | Kürzel Cmd/Strg+**Alt+0** | Öffnet den Bereich Projekt (Cmd+0 bleibt „Zoom zurücksetzen“) | |
| 21 | Start-Programm wieder auf „Automatisch“ stellen | In der .vcprj steht kein `main` mehr (behoben: Wert wird aus dem Ereignis gelesen) | |
| 22 | VoiceOver: Karten und Seiten des Bereichs | Überschriften je Karte, „Ändern ›“ und ↑/↓ als Knöpfe mit sprechender Beschriftung | |

Noch nicht: Zielsysteme hinzufügen oder entfernen (Ziele), Auswahl des Emulators je Ziel (kommt mit den Einstellungen E12).

## Projekt für anderes System kopieren (vom Inhaber bestätigt, Variante 1)

Im Dialog „Projekt kopieren“ gibt es die Wahl „Zielsystem der Kopie“. Das gewählte System wird **Normalziel** (erster Eintrag der Projektdatei); vorhandene Ziele bleiben. **Keine Umsetzung des Codes:** Quellen, Medien und Einstellungen werden unverändert kopiert (wie sonst, ohne `build/` und Git), das Original bleibt unberührt, die Kopie wird danach geöffnet. Ein Hinweis nennt, was zielsystemabhängig ist (Zeropage- und Speicheradressen, Sprite- und Zeichensatzformate, Palette). Die Frage nach der Lesart ist beantwortet: Der Inhaber hat Variante 1 bestätigt (Kopie mit neuem Zielsystem als Normalziel, Quellen unverändert, keine Code-Umsetzung, Hinweistexte).

| # | Schritte | Erwartet | Ergebnis |
|---|---|---|---|
| E1 | Projekt ▸ Weitergeben ▸ „Projekt kopieren …“, Zielsystem wählen | Hinweis unter dem Feld; nach dem Kopieren Hinweis-Meldung, Kopie geöffnet | |
| E2 | Projekt ▸ Übersicht der Kopie | Gewähltes System steht als erstes Ziel, die alten Ziele darunter | |

## P4: Quellgruppen, Ziele, Diskettenabbild bearbeiten

Bilder: `~/Development/abnahme-ui/projekt-p4/bilder`. Alles wird sofort in die Projektdatei geschrieben; ein Fehler (fehlende Datei, doppelte Quelle, Pfad außerhalb) bleibt ungespeichert und steht als Meldung da.

| # | Schritte | Erwartet | Ergebnis |
|---|---|---|---|
| Q1 | Projekt ▸ Quellen: „Neue Gruppe: Assembler“, darin „Datei hinzufügen“ (Vorschläge aus dem Projekt), „Hinzufügen“ | Datei steht in der Gruppe; zweite Eingabe derselben Datei wird abgelehnt | |
| Q2 | Eine Datei mit ✕ entfernen; die Start-Datei entfernen | Die Datei verschwindet; war es das Start-Programm, gilt wieder „Automatisch“ | |
| Q3 | Art einer Gruppe auf BASIC stellen, Dialekt eintragen, Ziele an- und abhaken | Änderungen bleiben nach Neuöffnen; „Gruppe entfernen“ entfernt sie | |
| Q4 | Projekt ▸ Ziele: „Undokumentierte Befehle“ und „Startziel (F5)“ je Zielsystem | Eintrag in der Projektdatei; „Standard“ nimmt ihn wieder heraus; F5 startet wie gewählt | |
| Q5 | Projekt ▸ Bauen und Starten ▸ „Diskettenabbild beim Bauen“: D64/D81/ATR wählen, Namen setzen, bauen | Nach dem Bauen liegt das Abbild unter `build/`; „Kein Abbild“ entfernt den Abschnitt | |

## Rückfragen aus dem Kern (Sicherheit): Papierkorb und Programme

Bilder: `~/Development/abnahme-ui/rueckfragen/bilder`. Die Ablauftests prüfen beides ohne Papierkorb und ohne Programmstart; **nur von Hand** lässt sich die echte Wirkung sehen.

| # | Schritte | Erwartet | Ergebnis |
|---|---|---|---|
| S1 | Eine Datei im Projektbaum löschen, wenn der Papierkorb des Systems nicht erreichbar ist (z. B. Linux ohne `gio`/`trash-put`; am Mac nur schwer nachzustellen) | Rückfrage „Papierkorb nicht verfügbar … Endgültig löschen?“ mit **Abbrechen** als Vorgabe; Abbrechen/Esc: Datei bleibt; „Endgültig löschen“: Datei weg | |
| S2 | Im Projektbaum eine `.sh`-, `.command`- oder `.app`-Datei „Im Standardprogramm öffnen“ (oder in der Hex-Ansicht „Im Standardprogramm öffnen“) | Rückfrage „Diese Datei ist ein Programm … Trotzdem öffnen?“ mit **Abbrechen** als Vorgabe; erst nach „Trotzdem öffnen“ startet sie | |
