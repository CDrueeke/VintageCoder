# Rundgang 0.3.0 (Vorabpaket) – Handprüfung in rund 130 Minuten

Nur Punkte, bei denen das Gefühl in der Hand zählt (Maus, Ziehen, Tastatur, Ton, Emulator). Alles andere prüfen die Ablauftests und die Bilderstrecke (`bilder/`). Die Reihenfolge folgt dem Risiko: Wichtiges zuerst. Jeder Punkt hat ein Startskript; es startet direkt auf der Prüfseite.

**Sicher prüfen:** Jedes Startskript läuft mit **eigenem** Einstellungs- und Projektordner (`home/` neben diesem Paket). Deine echten Einstellungen unter `~/.config/vintagecoder` werden nicht angefasst; beim ersten Start wird nur eine **Kopie** der Datei `settings.json` in `home/` gelegt (so sieht es aus wie bei dir). Zum Zurücksetzen: Ordner `home/` löschen. Wer die echten Einstellungen trotzdem sichern will: `cp -R ~/.config/vintagecoder ~/vintagecoder-sicherung`.

Ergebnis je Punkt: ✔ gut · ✘ falsch (kurz notieren, was) · ? unklar.

| # | Skript | Was tun | Erwartet | Ergebnis |
|---|---|---|---|---|
| **A. Erster Start und Rahmen** | | | | |
| 1 | `start-assistent` | App startet mit leerer Konfiguration | Startbild (etwa 3 s), dann die Startseite mit der Willkommenskarte „Willkommen bei VintageCoder“ und „Einrichtung starten“; der Assistent öffnet sich **nicht** von selbst | |
| 2 | `start-assistent` | „Einrichtung starten“ auf der Karte; Schnelleinrichtung durchklicken (Sprache, Ziele, Emulatoren „Suchen“, Projekte), „Übernehmen“ | Zusammenfassung stimmt; danach Startseite; „Rückgängig“ stellt Einstellungen, Beispiele und den Erststart-Stand (wizard.json) wieder her | |
| 3 | `start-assistent` | Nach „Rückgängig“ neu starten: die Willkommenskarte kommt wieder (Erststart). „Später“ auf der Karte | Start ohne Projekt zeigt immer die Startseite (nie den zuletzt benutzten Bereich); „Später“ blendet die Karte dauerhaft aus; der Assistent bleibt über Einstellungen, Befehl und Statuszeile erreichbar | |
| 4 | `start-assistent-mit-projekt` | Start mit leerer Konfiguration **und** Projekt | Das Projekt öffnet sich, kein Überlagern-Dialog; in der Statuszeile steht „Einrichtung noch nicht gemacht: jetzt starten“ (Klick öffnet den Assistenten) | |
| 5 | `start-rundgang` | Programmstart beobachten | Startbild etwa 3 s, Klick oder Taste schließt früher; Einstellungen ▸ Allgemein ▸ Startbild aus → beim nächsten Start kein Bild; **danach Startbild wieder einschalten** (der Schalter wird im Speicher des Fensters gemerkt, der im echten Benutzerordner liegt und für alle Start-Skripte gilt, auch wenn deren `home/` leer ist; dasselbe gilt für Farbschema, Zoom und „Später“ auf der Willkommenskarte) | |
| 6 | `start-rundgang` | Fenster schmal ziehen (unter 960 px) | Hauptnavigation zeigt nur Symbole, das aktive Ziel mit Text; nichts läuft über; auf der Startseite ist „Startseite“ (ganz links) hervorgehoben; Klick auf „Startseite“ mit offenem Projekt fragt, ob das Projekt geschlossen werden soll | |
| 7 | `start-rundgang` | Karten „Neues Projekt“, „Projekt öffnen“, ein Beispiel kopieren („Kopieren“ im Beispieldialog) | Dialoge öffnen sich, Beispiel liegt danach als Kopie im Projektordner und ist offen | |
| 8 | `start-rundgang` | Zuletzt-Karte: Rechtsklick oder Ctrl+Klick auf einen Eintrag (Menü wie „…“: Öffnen, Anheften, Kopieren, Im Ordner zeigen, Entfernen) | Menü wirkt; Angeheftetes steht vorn | |
| **B. Editor, Bauen, Starten** | | | | |
| 9 | `start-editor` | Tippen im Editor; Vorschläge, Hover auf einem Befehl | Eingabe ohne Verzögerung, Vorschlagsliste per Tab/Enter, Hover zeigt Erklärung | |
| 10 | `start-editor` | F7 Bauen | Fortschritt, Meldung „fertig“, Dateien im Ordner | |
| 11 | `start-editor` | Absichtlich Fehler einbauen, Bauen, Klick auf die Meldung | Cursor springt zur Zeile, Fehler rot markiert | |
| 12 | `start-editor` | F5 Starten (Emulator) und wieder Stopp | Emulator öffnet sich, Programm läuft, Stopp beendet ihn | |
| 13 | `start-editor` | Haltepunkt per Klick am Rand, Cmd/Strg+S | Punkt bleibt nach Schließen/Öffnen der Datei und nach einem Neustart des Programms (auch wenn der Text sich geändert hat, folgt er seiner Zeile); Speichern ohne Verzögerung | |
| **C. Tracker (Ton)** | | | | |
| 14 | `start-include` | In `sprite_demo.asm`: Doppelklick auf den Pfad `graphics/hero.vcsprite`; Rechtsklick auf einen Pfad (Öffnen, Im Projektbaum zeigen); Doppelklick auf `.include` selbst | Sprite-Editor öffnet; Menü wirkt; auf das Schlüsselwort wählt der Doppelklick nur das Wort | |
| 15 | `start-tracker-c64` | Zelle anklicken, **Leertaste** | Song spielt hörbar; Zeile folgt, Pegelbalken schlagen aus | |
| 16 | `start-tracker-c64` | M und S im Spaltenkopf während des Spielens | Stimme verstummt bzw. solo; kein Knacken | |
| 17 | `start-tracker-c64` | Klaviertasten Z S X D C V … | Ton beim Eintragen kurz hörbar, Cursor rückt weiter | |
| 18 | `start-tracker-c64` | Umschalt+Leertaste, Stopp | Nur das Muster spielt einmal; Stopp sauber | |
| 19 | `start-tracker-c64` | Klaviatur unten rechts anklicken | Klick spielt vor, ändert den Song nicht | |
| 20 | `start-tracker-atari` | Spielen (POKEY), Instrument-Regler ziehen | Klang ändert sich sofort | |
| 21 | `start-tracker-c64` | Einstellungen ▸ Tastenkürzel ▸ „Einzeltasten der Editoren“ aus | Klaviertasten tragen nichts mehr ein; Cmd/Strg-Kürzel gehen weiter | |
| **D. Debuggen** | | | | |
| 22 | `start-debug` | „Debuggen starten“, Haltepunkt auf ausgeführter Zeile | Halt an der Zeile, Register und Disassembler zeigen Werte | |
| 23 | `start-debug` | Knöpfe „Schritt über“, „Schritt hinein“, „Weiter“ (Tooltip nennt die Taste); am Mac Cmd+Alt+O / I / U, sonst F10 / F11 / Umschalt+F11 | Schritte folgen sichtbar, Zeile wandert mit | |
| 24 | `start-debug` | Beobachten `a`, Speicher `D020` | Werte erscheinen und ändern sich | |
| 25 | `start-debug` | Beenden (Umschalt+F5) | Emulator endet, Ansicht kehrt zurück | |
| **E. Grafik** | | | | |
| 26 | `start-sprites` | Mit gedrückter Maus Pixel malen, Rechteck, Füllen | Zug ohne Aussetzer, ein Zug = ein Rückgängig | |
| 27 | `start-sprites` | Bilder ziehen oder mit „Bild nach links/rechts“ umordnen; „Alle Bilder abspielen“ (oder ≥ 2 Bilder markieren ▸ Animation) | Reihenfolge folgt der Maus; Vorschau läuft | |
| 28 | `start-zeichensatz` | Zeichen anklicken, Pixel ziehen | Wirkt sofort, Tooltips lesbar | |
| 29 | `start-palette` | Farbregler ziehen, Hexwert tippen | Farbe folgt der Maus, Unsinn wird abgelehnt | |
| 30 | `start-kacheln` | Zellen ziehen, Alt+Klick (Pipette) | Mehrere Zellen werden gesetzt, Pipette nimmt Kachel auf | |
| 31 | `start-bildschirm` | `title.vcscreen`: Zeichen wählen, über den Bildschirm ziehen (Zellen leuchten schon beim Ziehen auf), Rahmen aufziehen; `level.vcscreen` zeigt die Kachel-Meldung mit „Als Text öffnen“ | Schnelles Ziehen lässt keine Zellen aus | |
| **F. Daten** | | | | |
| 32 | `start-hex` | `build/…/*.prg` aus der Liste (oder „Datei öffnen …“) öffnen; Bytes aufziehen, Umschalt+Klick, Pfeiltasten | Auswahl folgt, das rechte Panel zeigt Adresse und Deutung | |
| 33 | `start-hex` | Suchen `$A9 00` und einen Text, der im Programm steht (zum Beispiel ein Wort aus dem Quelltext) | Treffer wird markiert und angesprungen | |
| 34 | `start-speicher` | Bauen, dann Speicherkarte: Seite anklicken, Filter | Karte aktualisiert, Prüfungen nennen Befund | |
| 35 | `start-disk` | Datei aus dem Finder auf die Dateiliste der offenen Diskette ziehen (ohne offene Diskette erscheint „Öffne zuerst eine Diskette“) | Eintrag erscheint, Speichern erzeugt gültiges Abbild | |
| 35a | `start-disk` | Dateizeilen per Ziehen umordnen (Linie zeigt die Stelle), Knöpfe „Nach oben/Nach unten“, Alt+Pfeil hoch/runter, Cmd/Strg+Z und Umschalt+Z, Speichern, Abbild im Emulator/Zeile für Zeile prüfen (D64 und ATR) | Ein Zug = ein Rückgängig-Schritt; die Reihenfolge steht nach dem Speichern im Abbild; Dateien bleiben lesbar (Hex/Prüfen ohne Fehler) | |
| **G. Projekt, Einstellungen, Hilfe** | | | | |
| 36 | `start-projekt` | Seiten des Bereichs durchgehen; Cmd/Strg+Alt+0 | Seiten wechseln mit Pfeiltasten; Kürzel öffnet Projekt | |
| 37 | `start-projekt` | Weitergeben ▸ „Projekt kopieren …“, Zielsystem der Kopie wählen, Kopieren | Hinweis „Code wird nicht umgesetzt“; Kopie öffnet sich, gewähltes System ist erstes Ziel (Projekt ▸ Übersicht) | |
| 38 | `start-projekt` | Namen ändern, Ziel hinzufügen und entfernen | Änderung steht in der Projektdatei; nichts geht verloren | |
| 39 | `start-einstellungen` | Zoom Cmd/Strg +, −, 0; Farbschema umschalten | Ansicht folgt sofort, bleibt nach Neustart | |
| 40 | `start-einstellungen` | Ziele: Emulator „Suchen“ und „Testen“ | Pfad erscheint, Test meldet Ergebnis | |
| 41 | `start-einstellungen` | Sicherung exportieren, zurücksetzen, importieren | Rückfragen erscheinen; danach alles wie vorher | |
| 42 | `start-hilfe` (öffnet `jumprun.asm` im Editor) | Cursor auf einen Befehl, F1; dann in der Hilfe nach `lda` suchen | Treffer öffnet Thema; F1 springt zur passenden Stelle | |
| **H. Zum Schluss** | | | | |
| 43 | `start-rundgang` | Englisch wählen (Einstellungen ▸ Allgemein), eine Seite ansehen | Texte wechseln; Maschinenübersetzung, nicht abgenommen | |
| 44 | `start-rundgang` | Helles Farbschema, dann Hoher Kontrast | Lesbar, nichts unsichtbar | |
| 45 | `start-rundgang` | Tab-Taste durch Kopfzeile und Startseite | Reihenfolge sinnvoll, Fokusrahmen sichtbar | |
| 46 | `start-rundgang` | Fenster schließen mit ungespeicherter Datei | Rückfrage; „Nicht speichern“ verwirft | |
| **I. BASIC (Einzelheiten: `docs/handpruefung-basic.md`)** | | | | |
| 47 | `start-basic` (Projekt `c64-basic_test`, Datei `test.bas`) | BASIC-Datei mit `#define`, `#if … #else … #endif`: Maus über `STERNE`, `#if`, `#asm`; Schublade ▸ Gliederung | Hover-Karten mit Definition bzw. Kurzerklärung; Gliederung nennt Marken und `#define`, Klick springt | |
| 48 | `start-basic` (und ein Atari-Projekt) | Zeile mit Hochkomma in einem Commodore- und in einem Atari-Projekt | Commodore: kein Kommentar; Atari: Kommentar; Statuszeile nennt den Dialekt | |
| 49 | `start-basic` | `FOR` ohne `NEXT` bauen (F7), Schublade ▸ Probleme; Maus auf die Kennung B151 | Warnung mit Kennung, Erklärtext beim Überfahren, Klick springt zur Zeile | |
| **J. Neu seit der ersten Fassung** | | | | |
| 50 | `start-einstellungen` | Einstellungen ▸ ROMs ▸ „ROM-Datei oder Zip wählen …“ (eigene ROM-Datei), dann „Übernehmen“ | Plan mit Haken, nichts wird ohne Bestätigung kopiert; Datei steht danach im ROM-Ordner | |
| 51 | `start-einstellungen` | Ziele ▸ Emulator ▸ „Portabel installieren …“ nur ansehen (nur prüfbar, wenn der Emulator noch nicht gefunden wurde; sind alle Emulatoren vorhanden, entfällt der Punkt), **nicht** laden (Netz) | Rückfrage nennt Version, Größe, Quelle, Zielordner; „Abbrechen“ lädt nichts | |
| 52 | `start-assistent` | Assistent bis „Fertig“ durchklicken | „Erstes Projekt“: „Neues Projekt anlegen …“ und „Beispiel öffnen“ funktionieren | |
| 53 | `start-projekt` | Quellen: Gruppe anlegen, Datei hinzufügen/entfernen; Ziele: Startziel (F5) wählen | Wird sofort gespeichert; Fehler (fehlende Datei) bleiben ungespeichert mit Meldung | |
| 54 | `start-debug` | Debuggen, links bei „Chips“ VIC-II anhaken | Register mit Werten im Halt; Statusleiste „Debug: angehalten bei $…“ | |
| 55 | `start-markdown` | `README.md` öffnen: Vorschau, „Quelltext“, Schreibhilfen (B, I, Link, Tabelle), Aufgabe abhaken | Vorschau sauber; jede Hilfe ein Rückgängig-Schritt; Web-Link öffnet nur auf Klick | |
| 56 | `start-rundgang` | Einstellungen ▸ Allgemein ▸ Sprache „English“; danach je Bereich einen Dialog oder eine Seite ansehen (Neues Projekt, Sprite-Editor, Einstellungen, Hilfe, Debug); dann zurück auf Deutsch | Texte wechseln sofort ohne Neustart, auch Listen aus dem Kern (Zuletzt, Beispiele, Farbrollen); nichts bleibt in der alten Sprache stehen (Englisch ist maschinell übersetzt, nicht abgenommen) | |
| 57 | `start-karten` | Karten-Editor: Kachel wählen, über die Karte ziehen (Zellen leuchten beim Ziehen), Rechteck, Zellen-Modus mit Zeichen, Auswahl ▸ Stempel, „Neuer Screen“, Screen kopieren/löschen und per Ziehen umordnen, Rechtsklick mit dem Stift löscht, „Alles auswählen“ bei Auswahl, Cmd/Strg+C, X, V, A (Einfügen setzt einmal ein, das Werkzeug bleibt; Stempel nur bewusst wählen), Größe ändern, Speichern | Zug = ein Rückgängig-Schritt; Screen-Leiste und Kachelraster folgen; gespeichert wird die ganze Bank; Tab wechselt den Modus, Pfeile bewegen den Cursor | |
| 58 | `start-kacheln`, `start-bildschirm`, `start-palette`, `start-tracker-c64`, `start-disk`, `start-karten` | In jedem Bereich etwas ändern, dann Cmd/Strg+Z (Rückgängig), Cmd/Strg+Shift+Z (Wiederholen) und Cmd/Strg+S (Speichern) – einmal über die Tastatur, einmal über das Menü Bearbeiten/Datei | Kürzel wirken im jeweiligen Editor (Menüeinträge nicht grau), Speichern schreibt die Datei; im Texteingabefeld gilt weiter das Rückgängig des Feldes | |

| **K. Neu seit zwischen9** | | | | |
| 59 | `start-rundgang` | Startbild beim Start ansehen; Einstellungen ▸ Info (Über-Karte) | Version 0.3.0 ohne „Alpha“, neues Logo im Startbild und im Info-Bild | |
| 60 | `start-editor` | Bearbeiten ▸ Zahlen (und BASIC): Wert eingeben (dezimal, `$`-hex, `%`-binär), Umrechnung ablesen, übernehmen | Dialog zeigt alle Darstellungen; Übernehmen fügt den Wert an der Cursorstelle ein | |
| 61 | `start-editor` (öffnet `jumprun.asm`) | Im Projektbaum Rechtsklick auf `README.md` ▸ „Als Include einfügen“, Cmd/Strg+Z; Rechtsklick auf `lib/core.asm` (ist schon eingebunden); dann `README.md` als Reiter öffnen und erneut Rechtsklick auf eine Datei im Baum | `.include "README.md"` steht vor der Zeile mit der Schreibmarke, ein Rückgängig-Schritt; `lib/core.asm` meldet „schon eingebunden“, nichts doppelt; mit `README.md` als aktivem Reiter ist der Menüpunkt ausgegraut, der Hinweis (Maus darüber) nennt den Assembler-Quelltext | |
| 62 | `start-editor` | Im Code-Editor Text markieren: Kopieren, Ausschneiden, Einfügen über Menü Bearbeiten und über Cmd/Strg+C, X, V | Zwischenablage wirkt in beide Richtungen; Ausschneiden/Einfügen je ein Rückgängig-Schritt | |
| 63 | `start-bildschirm` | „In Kacheln zerlegen …“ im Bildschirm-Editor | Dialog erklärt das Ergebnis; Kachelsatz und Karte entstehen, nichts geht ohne Bestätigung verloren | |
| 64 | `start-rundgang` | Menüs und Befehlspalette (Cmd/Strg+Shift+P) durchsehen | Keine Zeile ohne Funktion: jede sichtbare Menüzeile tut etwas (11 noch nicht gebaute Befehle sind ausgeblendet, `docs/befehle-ohne-funktion.md`) | |
| 65 | `start-englisch` (Sprache en, Stichprobe) | Ein paar Seiten und Dialoge ansehen, besonders neue Texte (Zahlen, Als Include, Zerlegen) | Englisch durchgehend, nichts Deutsches stehen geblieben | |

Auffälligkeiten ohne Nummer bitte unten notieren.
