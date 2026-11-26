# Änderungsprotokoll

Die Versionsnummer hat die Form `MAJOR.MINOR.PATCH`. Solange sie unter 1.0 liegt, steht `0.MINOR` für ein neues Funktionspaket oder
einen Bruch bei Projektdateien oder Einstellungen und `PATCH` für Fehlerkorrekturen. Die Nummer und die Tags (`vMAJOR.MINOR.PATCH`) legt
der Projektinhaber fest. Änderungen für die nächste Version stehen unter „Unveröffentlicht“.

## Unveröffentlicht

## 0.3.0

0.3.0 bringt eine neu gebaute Oberfläche und macht VintageCoder zweisprachig: Oberfläche, Meldungen, Hilfe und Handbuch gibt es auf Deutsch und auf Englisch. Die Oberfläche ist komplett neu aufgebaut und hat jetzt für jeden Bereich einen eigenen Editor. Assembler, BASIC und Kommandozeile bleiben im Kern, werden aber an vielen Stellen erweitert. VintageCoder ist weiter in Entwicklung und enthält mit Sicherheit noch Fehler.

### Neue Oberfläche

- Wieder einmal (dafür aber letztmalig) neue Technik und ein neues Design: Die Oberfläche läuft auf Tauri und Svelte. Das frühere Fenster entfällt; Projekte, Einstellungen und Dateien bleiben unverändert nutzbar.
- Sechs Hauptbereiche in der Kopfzeile: **Projekt**, **Code**, **Grafik**, **Musik**, **Daten** und **Debug**. Jeder Bereich hat eine zweite Reihe für seine Editoren (zum Beispiel Sprites, Zeichensatz, Kacheln, Bildschirm, Karten, Palette unter Grafik; Disk, Hex, Speicher unter Daten).
- Der Bereich **Projekt** (Cmd/Strg+Alt+0) sammelt alles, was zum ganzen Projekt gehört: Übersicht, Allgemein, Ziele, Quellen (mit Baureihenfolge und Startprogramm), Bauen und Starten, Diskette, Weitergeben.
- Aufbau des Fensters: Seitenleiste, Editor, Ablage unter dem Editor (Probleme, Ausgabe, Tests, Suche, Gliederung) und Panel „Maschine“. Bereiche lassen sich ziehen, einklappen und groß machen; zwei Editorbereiche nebeneinander sind möglich. Der Zen-Modus blendet alles außer der Arbeitsfläche aus. In schmalen Fenstern legen sich die Leisten über die Fläche.
- Startseite mit Karten für Neues Projekt, Projekt öffnen und Beispiele, mit zuletzt geöffneten Projekten (anheften, entfernen, Projektbild) und den erkannten Emulatoren. Ein Startbild erscheint beim Start (lässt sich abschalten, Klick oder Taste beendet es).
- Befehlspalette (Cmd/Strg+K) für Befehle, Dateien, Projekte und Orte; sie findet auch Synonyme. Alle Befehle stehen zusätzlich in den Menüs, mit Tastenkürzel. Die Menüleiste ist die des Systems.
- Einstellungen (Cmd/Strg+Komma) mit den Seiten Allgemein, Ziele und Hardware, ROMs, Aussehen, Editor, Toolchain, Tastenkürzel, Sicherung sowie Info und Update. Dazu gehören Farbschema (Dunkel, Hell, Hoher Kontrast), Größe der Oberfläche, Emulatoren suchen und testen, Sicherung (Export, Import, Zurücksetzen) und eine Update-Suche auf Wunsch.
- Einrichtungsassistent mit Schnellweg, „Alles selbst einstellen“ und Teil-Assistenten. Er zeigt vorab eine Liste aller Änderungen, die du einzeln abwählen kannst, sichert den alten Stand und lässt sich zurücknehmen. Beim ersten Start fragt eine Willkommenskarte, statt den Assistenten ungefragt zu öffnen.
- Sitzung wiederherstellen (je Projekt): geöffnete Reiter, Haltepunkte und Lesezeichen bleiben über einen Neustart erhalten. Fensterlage und Layout werden gemerkt.
- Neues Logo: Wortmarke, Kurzform und App-Icon (macOS, Windows, Linux).

### Editoren

**Code-Editor**
- Neu gebaut auf CodeMirror 6: Reiter, Syntaxfarben, Falten, Vervollständigung, Hover-Karten mit Befehlshilfe, Fehler beim Tippen mit Wellenlinie, Quick-Fix, Gehe zu Definition, Suchen und Ersetzen im Projekt, Lesezeichen, Haltepunkte, Kommentar umschalten, Klammersprung, Zeilen sortieren.
- Prettyprint beim Verlassen der Zeile (Assembler und BASIC), dazu „Zeile formatieren“ und „Datei formatieren“; abschaltbar.
- Zyklenzahl je Zeile im Rand, Speicherstreifen in der Statuszeile.
- Doppelklick auf eine Include-Zeile springt zur Datei. Im Dateibaum fügt „Als Include einfügen“ (Kontextmenü) die passende `.include`-Zeile mit relativem Pfad in den geöffneten Assembler-Reiter ein.
- Zahlenwerkzeuge (Bearbeiten ▸ Zahlen und BASIC): Zahl oder Ausdruck (`$D020`, `$D000+$20`, `1<<5`) in allen Schreibweisen, mit Bit-Editor für 8 und 16 Bit, Kopieren und Einsetzen.
- Kopieren, Ausschneiden und Einfügen über Menü und Befehlspalette auch im Code-Editor.
- Unbenannte Reiter und Notizen; Art des Reiters (Text, Assembler, BASIC) wählbar.

**Sprite-Editor**
- Zeichnen mit Linie, Rahmen, Füllen, Auswahl, Zwischenablage, Spiegeln, Verschieben; Zoom, Raster, Rückgängig.
- Bilderliste, Hires oder Multicolor je Bild, Sprite-Eigenschaften (C64, C128, MEGA65), Palette.
- Atari Player/Missile mit Breite und Priorität.
- Animation: Zeitleiste, Filmstreifen mit Miniaturen, Dauer je Bild, Tempo für alle, große Vorschau mit direkt bearbeitbaren Schritten, Animation aus einer Auswahl erzeugen, Onion Skin.
- PNG laden, Export als Include, in den Quelltext kopieren oder als BASIC-DATA-Zeilen mit Einleseroutine.

**Zeichensatz-Editor**
- Übersicht aller Zeichen (mit Tastatur und Ansage), 8x8-Editor mit den Werkzeugen des Sprite-Editors, Umgebung und Vorschau, Zeichenkarte.
- Bereiche, neuer Zeichensatz, Multicolor, MEGA65-Vollfarbzeichen.
- Eigene Standardzeichensätze für alle Systeme; Export als Include.

**Tracker (SID, POKEY, PLAY)**
- Raster mit Zeilen und Stimmen, Noteneingabe mit den Tasten wie am Klavier (nach Tastenposition), Auswahl, Block, Transponieren, Muster und Reihenfolge mit Wiederholung, Instrumente (SID: Klangform, Hüllkurve, Pulsbreite; POKEY: Verzerrung und Hüllkurve).
- Wiedergabe im Editor mit Mitscrollen, Pegel je Stimme, Stumm und Solo, Änderungen während des Spielens, Vorhören beim Eintragen, Klaviatur zum Anklicken, Stimmen ein- und ausblenden.
- Songs übernehmen: SID-Dateien, GoatTracker-Songs (.sng), Songs anderer Systeme; Atari: RMT-Module importieren.
- Export als Assembler, Include, Binärdatei, PRG oder BASIC-DATA; die Spielroutine liegt den Beispielen bei.

**Kachel-, Bildschirm- und Karten-Editor**
- Bildschirm-Editor: Zeichentafel mit Zeichenfarbe, Werkzeuge (Zeichen, Rahmen, Füllen, Text, Pipette, Löschen), Systemfarben, Zeichensatz, Tastatur.
- Kachel-Editor: Kachelsatz mit Vorschau und Verwendungszähler, große Kachel mit Zellen, Pipette, Farbmodi, Zeichentafel mit Zeichensätzen; Export mit wählbarem Layout.
- Karten-Editor (neu): Screens einer Screen-Bank zeichnen, auswählen, kopieren, einfügen, löschen, umordnen; Linie und Rechteck mit genauer Vorschau; Zoom, Gitter und Kachelgrenzen, PAL/NTSC-Seitenverhältnis. Ein Textbildschirm lässt sich im Bildschirm-Editor mit „In Kacheln zerlegen …“ in Kachelsatz und Screen-Bank verwandeln, die dann im Karten-Editor öffnet.

**Palette**
- Farbraster mit Regler und Hexwert, Standardpalette, „Im Einsatz“, Export, Speichern, Rückgängig (MEGA65-Palette, TED-Palette für Plus/4; Atari und Co. wählen Farben im jeweiligen Editor).

**Disk-Editor**
- Diskettenabbilder öffnen und ansehen: Verzeichnis, Belegungskarte mit Kette der gewählten Datei, Sektor-Inspektor (D64, D71, D81, ATR).
- Dateien ablegen, Dateien ins Projekt übernehmen, Prüfen (BAM/VTOC, Verzeichnis, Ketten), Speichern, Speichern unter, neue leere Diskette, Diskette im Emulator einlegen. Cmd/Strg+S speichert die Diskette.
- **Verzeichnis umordnen:** Dateizeilen per Ziehen (mit Einfügemarke), Alt+Pfeil oder Knöpfe; jeder Zug ist ein Rückgängig-Schritt. Gilt für D64, D71, D81 und Atari DOS 2/2.5; die Dateien selbst bleiben unverändert. Bei MyDOS erscheint stattdessen eine Meldung.
- Atari-Abbilder (ATR): Sektorgrößen 128 und 256, Boot-Anordnung, DOS 2, DOS 2.5 und MyDOS werden erkannt; Verzeichnis mit Unterverzeichnissen.

**Hex und Speicher**
- Hex-Ansicht für Binärdateien des Projekts: Suche, Auswahl mit Deutung, Seiten. Nur zum Ansehen.
- Speicherkarte des letzten Baus: Belegung, Filter, Bankumschaltung, Seitendetails, Prüfungen.

**Debug und Maschine**
- Haltepunkte (auf Zeile, Adresse, mit Bedingung, Datenzugriff), Einzelschritt, Register und Chip-Fenster, Disassembler, Speicher, Beobachten, Momentaufnahme, Bild des Emulators; ein Test lässt sich im Debugger ansehen.
- Panel „Maschine“ mit Ziel, Zustand, Bauergebnis, Registern, Flags und Einzelbild; Ausführungszeile im Quelltext, PC in der Statusleiste.
- Emulator und alle von ihm gestarteten Prozesse werden beim Beenden mit beendet; ein Ende des Emulators wird erkannt und gemeldet.
- Bauen und Starten laufen im Hintergrund, die Ausgabe enthält klickbare Fehlerzeilen. Tests (`.vctest`, statisch oder im Emulator) laufen in der Ablage mit Filter und Sprung zur Quelle.

**Markdown**
- Dateien mit `.md` zeigen eine gesetzte Vorschau, der Quelltext ist gefärbt. 16 Schreibhilfen (zum Beispiel Überschriften, Listen, Tabellen, Links), Gliederung.

### BASIC

- **Dialekte:** CBM V2, 3.5, 7.0, 65, VIC-20-BASIC, Atari BASIC und Turbo BASIC XL; `#dialect` wählt den Dialekt. Der Dialekt steht in der Statusleiste.
- **Zeilennummern oder Labels:** Du schreibst entweder mit Zeilennummern oder mit Sprungmarken (`@name`). Beides lässt sich jederzeit umwandeln: **In Labels umwandeln**, **In Nummern umwandeln**, **Neu nummerieren** (mit Vorschau); `#numbering` setzt Start und Schrittweite.
- **Syntaxprüfung** ohne Emulator (Fehlercodes ab B150): unbekannte Variablen, falsche Zeilennummern und mehr, mit Erklärtext zu jedem Code im Problem-Panel. Die Prüfung lässt sich abschalten.
- **Vorverarbeitung:** `#define`, `#if`, `#include` auch im BASIC; Zeilen, die `#if` ausschließt, erscheinen grau.
- **Maschinencode im BASIC mit `#asm`:** ein Assembler-Block im BASIC-Programm wird mit gebaut und als DATA-Zeilen abgelegt (frühere Bezeichnung `#blob`).
- **Medien einbinden:** Sprites, Zeichensätze, Bildschirme und Songs lassen sich im BASIC mit `#include "graphics/hero.vcsprite"` einbinden; dazu stehen Konstanten wie `<marke>_line` zur Verfügung. Die Medien werden dabei nie verändert.
- **Kompakt bauen** (Option) und lange Variablennamen (`#longnames`).
- **Turbo BASIC XL starten:** Turbo BASIC XL ist kein Bestandteil von VintageCoder und wird vom Anwender als Diskette beigesteuert. Ein Assistent prüft die Diskette, kopiert sie ins Projekt (auf Wunsch verkleinert) und baut `build/<name>.atr`, die im Emulator startet. Debuggen ist für Turbo BASIC XL noch nicht möglich.
- Der Apostroph gilt in den Commodore-Dialekten nicht als Kommentar; Schlüsselwörter ohne Leerzeichen (`fori=1to10`) werden erkannt.

### Assembler VCASM

- **Einheitliche, sprechende Direktivennamen:** `.relocate` … `.endrelocate`, `.retaddr`, `.lohitable`, `.screencode`, `.textnull`, `.textlen`, `.textshift`, `.align n, skip` und `.align n, $ea`, `.endfor`, `.be16` und `.be32`, `.dword`. Alle alten Namen (zum Beispiel `.pseudopc`, `.rta`, `.screen`, `.null`, `.page`, `.dbyt`, `.long`) bleiben gleichwertig und erzeugen dieselben Bytes. Neue Namen laufen erst ab dieser Version.
- Vervollständigung, Hover und Quick-Fix kennen die neuen Namen („Kanonischen Namen verwenden“); Hilfe und Handbuch enthalten eine Tabelle der alten Namen.
- **Neue Hinweise:** W205 (`.basicstub` auf dem Atari), W206 (`.run` auf einem Ziel ohne Startvektor), W207 (Atari-Programm mit Befehlen, aber ohne `.run`), W208 (alter Pfad `build/export`), E219 (Medium nicht einbindbar).
- **Medien als Include:** `.include "graphics/hero.vcsprite"` bindet Sprites, Zeichensätze, Bildschirme, Kacheln und Songs ein; der Assembler wandelt sie selbst um, ohne Zwischendateien im Projekt. Mit `vcc migrate includes` lassen sich alte Projekte umstellen.
- Eingebundene Dateien werden im Zusammenhang der einbindenden Datei geprüft; Fehler zeigen auf die Quelle.

### Zielsysteme

- In den Paketen: Commodore 64, Atari 8 Bit (400/800/XL/XE), MEGA65 (Emulator Xemu), Commodore Plus/4 und C16/C116, Commodore 128 und VC-20 (englisch: VIC-20).
- **CBM-II (CBM 600/700) entfällt als Zielsystem.** Ein altes Projekt mit diesem Ziel meldet beim Bauen, dass es das Zielsystem nicht mehr gibt.
- **PET/CBM** ist als Zielsystem nicht in den Paketen. Der ROM-Import kennt aber PET-ROMs.
- **ROM-Import:** Du wählst Dateien, ein ZIP oder einen Ordner; VintageCoder erkennt die ROMs an ihrer Prüfsumme (über 60 Einträge: Landesvarianten, Atari-OS, MEGA65, Plus/4-Funktions-ROMs), legt sie an den richtigen Ort und zeigt eine Fundliste. Kombinierte Abbilder (C64C, C64GS, Plus/4 „3-plus-1“, C128) werden in Teil-ROMs geteilt. VintageCoder liefert keine ROMs mit und lädt keine aus dem Netz.
- **Emulatoren:** Parametereditor für Emulatoren und Assembler; Emulator auf Wunsch portabel installieren.
- **Projekt für anderes System kopieren:** legt eine Kopie mit anderem Normalziel an (ohne Umsetzung des Quelltexts, mit Hinweis).

### Zweisprachigkeit

- **Deutsch und Englisch** in der gesamten Oberfläche (Menüs, Befehle, Einstellungen, Assistent, Editoren, Dialoge, Statuszeile), in den Meldungen von Assembler, BASIC, Disketten, Emulator-Anbindung, Kommandozeile und Sprachserver, in der Hilfe (59 Themen), im Handbuch (22 Kapitel und Glossar) sowie in README und Kommandozeilen-Referenz.
- Die Sprache wählst du im Menü „Sprache“ der Kopfzeile, unter Einstellungen ▸ Allgemein oder im Einrichtungsassistenten; „Automatisch“ folgt dem System. Sie wirkt sofort.
- `vcc --lang de|en` und `VC_LANG` schalten die Kommandozeile um; `vcc --json` und der Sprachserver liefern jede Meldung zusätzlich mit Code, Schlüssel und Parametern.
- Zahlen mit Nachkommastellen erscheinen landesüblich. Die englische Oberfläche ist vom Projektinhaber geprüft; die englische Hilfe und das englische Handbuch sind maschinell übersetzt und nur stichprobenartig geprüft. Handbuchbilder zeigen die deutsche Oberfläche.
- `vcc new --structure` kennt englische Kennungen (`simple`, `standard`, `large`, `multisystem`, `library`, `game`); die deutschen Namen gelten weiter.

### Beispiele

- Jedes Beispielprojekt ist in sich geschlossen: Es bindet keine Dateien außerhalb seines Ordners ein. Die gemeinsamen Dateien (`examples/common`) entfallen, die vier JUMPRUN-Projekte haben ihre eigene Spiellogik.
- Beispiele binden ihre Medien jetzt per Include der Mediendatei ein und nutzen die neuen Direktivennamen.
- Im Paket sind HELLO, DEMOS, JUMPRUN (C64, Atari, MEGA65, Plus/4) und PRALLBALL (C64, Atari, MEGA65, Plus/4, VIC-20, C128); alle Beispiele gibt es auch mit englischen Kommentaren, Bildschirmtexten und README.
- Beispiele werden nach Zielsystem gruppiert angeboten, mit Suche, Kopieren und Angabe der Herkunft.

### Sicherheit und Stabilität

- **Atomares Speichern:** Projektdatei, Mediendateien, Sitzung und Texte werden erst in eine Zwischendatei geschrieben und dann ersetzt; ein Absturz mitten im Schreiben lässt die alte Datei heil.
- **Papierkorb statt Löschen:** Löschen im Projektbaum legt Dateien in den Papierkorb des Systems. Ist das nicht möglich, fragt VintageCoder, statt still endgültig zu löschen.
- Pfade aus der Projektdatei bleiben im Projektordner; Verknüpfungen (Symlinks) nach außen werden nicht verfolgt. Beim Öffnen ausführbarer Dateien fragt VintageCoder zuerst.
- Der PNG-Leser begrenzt die Größe entpackter Bilder; manipulierte Diskettenabbilder führen nicht mehr zu Zählfehlern in der Belegung (D64, D71, D81).
- Ablegen, Beenden und schnelles Doppelklicken laufen geordnet: Ein Doppelklick öffnet keine zwei Reiter mit demselben Pfad, mehrere übergebene Pfade werden nacheinander verarbeitet.

### Behobene Fehler

Die neue Oberfläche ersetzt das frühere Fenster; Fehler, die nur dieses betrafen (zum Beispiel Tippen in Suchen und Dialoge änderte den Code, Kürzel mit Enter und Pfeilen), entfallen mit ihm. Darüber hinaus:

- Eine Zeile mit Umlaut hinter `#numbering`, `#asm` oder `REM` im BASIC brachte die Prüfung zum Absturz; behoben.
- Ziehen auf leerer Fläche scrollte die Ansicht (Auswahl-Autoscroll der Webview); behoben, Mausrad und Ziehen aus dem Finder unverändert.
- Der Debug-Lauf „Tests ausführen“ beendet eine laufende Sitzung nicht mehr.

### Bekannte Einschränkungen

- Die Pakete sind in frischen Systemen geprüft, aber nicht in allen Umgebungen.
- Die Anbindung an einen echten MEGA65 ist am Gerät nicht geprüft und noch nicht enthalten.
- Ton unter Linux ist nicht geprüft.
- Noch nicht in der Oberfläche: Zeichen-Panel (Steuerzeichen und PETSCII/ATASCII-Tabelle), Kodierung (UTF-8 oder ISO-8859-1) umschalten, BASIC in Assembler umwandeln als Menübefehl, Hex-Vorschläge für BASIC-Adressen, Import von Zeichensätzen (Standardsatz, ROM, PNG-Raster), Live-Bild des Ultimate 64.
- Debuggen: atari800 und MEGA65 (Xemu) halten nur einen Haltepunkt gleichzeitig; für Turbo BASIC XL gibt es kein Debuggen.

## 0.2.1 (Alpha)

0.2.1 stellt die Lizenz klar und behebt Fehler aus den Prüfungen von 0.2.0.

### Lizenz
- **Frei nutzbar:** Alle mit VintageCoder erzeugten Dateien (Programme, Quelltexte, Abbilder, Projekt- und Assetdateien) sind frei nutzbar; die Syntax des VCASM und die Dateiformate sind frei; `vcc` und `vc-lsp` dürfen von beliebiger Software aufgerufen werden. Die Erklärung steht in `LICENSING.md`.
- **MIT-0** für die Systemdateien (`data/sys`), die Vorlagen und die Beispiele (ohne Pflicht zum Urheberhinweis). Der RMT-Player in den Atari-Beispielen bleibt unter GPL v3 (Radek Štěrba / Raster / C.P.U.).
- **Lizenzhinweis beim Übersetzen** (L001, kein Fehler), `vcc build --license-report` (Bericht `build/LICENSES.txt`, auch `--json`), `--deny-license` und `licenses.deny`/`allow` in der Projektdatei; `vcc --license`; `LICENSING.md` und `LICENSES/` liegen in den Paketen.

### Fehlerkorrekturen
- Emulator beendet sich sofort: Meldung mit den letzten Ausgabezeilen und Hinweis auf Einstellungen ▸ ROMs.
- Esc schließt Einrichtungsassistent und Befehlsleiste.
- Linux: Anwendungskennung `vintagecoder` (Dock-Symbol).
- `VC_WIN` gilt wieder; Tracker: Kanalköpfe stehen fest; Editor: keine halbe letzte Zeile über der Schublade.

### Geändert
- „Onion Skin“ bleibt englisch (Programm, Hilfe, Handbuch, Glossar).
- Handbuch „Installation“ aktualisiert; Debian-/Ubuntu-Pakete: `vice` und `atari800` nur noch „Suggests“.

### Bekannte Einschränkungen
- Weiterhin Alpha. Unter Wayland entscheidet der Fenstermanager über die Anfangsgröße. Ton unter Linux nicht geprüft.

## 0.2.0 (Alpha)

Die größte Änderung ist die neue Oberfläche: Sie ist neu aufgebaut, zeigt sich in sechs festen Bereichen und lässt sich vollständig mit Tastatur und Befehlspalette bedienen. Assembler, BASIC und Kommandozeile bleiben im Kern, wie sie sind. VintageCoder ist weiter in Entwicklung und enthält mit Sicherheit noch Fehler; Stände und Einschränkungen stehen in STATUS.md.

### Zielsysteme

- Commodore 64, Atari 8 Bit (400/800/XL/XE), MEGA65, Commodore Plus/4 und C16/C116, Commodore 128 und VC-20 sind in den Paketen enthalten.
- PET/CBM und CBM-II sind nicht in den Paketen.

### Neue Oberfläche

- Sechs feste Bereiche um das Dokumentenfeld: Kopfzeile, Dateien und Suche links, Schublade unten (Probleme, Protokoll, Tests), Panel rechts (Gliederung, Hilfe, Zeichen, Speicherkarte, Emulator) und Statusleiste. Bereiche lassen sich ziehen, einklappen und groß machen; „Layout zurücksetzen“ stellt alles wieder her.
- Zen-Modus (Cmd/Strg+Umschalt+B) blendet alles außer der Arbeitsfläche aus.
- Befehlspalette (Cmd/Strg+K) findet Befehle, Dateien und Aktionen des offenen Studios; ausgegraute Befehle nennen den Grund. Alle Befehle stehen auch in den Menüs, mit Tastenkürzel.
- Kontextmenü auf der rechten Maustaste im Dateibaum, im Editor, auf Reitern und in den Studios.
- Farbschemata Dunkel, Hell und Hoher Kontrast; Größe der Oberfläche (Cmd/Strg+Plus, Minus, 0); Befehle „Editor-Schrift größer/kleiner“ und „Darstellung zurücksetzen“.
- Dateibaum mit Tastatur (Pfeile, Enter, F2, Entf), nach Dateiarten gruppiert; für Bildschirmleser sind Zeilen mit Namen, Art und Ebene gebaut.
- Startseite mit zuletzt geöffneten Projekten und Beispielen, Startbild, geführte Ersteinrichtung.
- Takt des Zielsystems in der Statusleiste (auf Wunsch).
- Konfiguration sichern: Einstellungen exportieren, importieren, zurücksetzen.
- Hilfe und Handbuch erscheinen als gesetzter Text (Überschriften, Tabellen, Listen, Links) im Panel oder in einem Reiter.

### Editor

- Prettyprint: Beim Verlassen einer bearbeiteten Zeile wird sie ausgerichtet (Assembler und BASIC); dazu „Zeile formatieren“ und „Datei formatieren“. Einstellung zum Abschalten.
- Dateien einbinden per Ziehen: Datei aus dem Projektbaum in den Quelltext ziehen setzt die passende `.include`-, `.incbin`- oder `#include`-Zeile; auch per Rechtsklick „Als Include einfügen“.
- Markdown: Dateien mit `.md` zeigen eine gerenderte Vorschau, dazu 16 Schreibhilfen und eine Gliederung.
- Zahlenwerkzeuge: Zahlen und Ausdrücke in allen Schreibweisen; Hex-Vorschläge für BASIC-Adressen.
- BASIC in Assembler umwandeln (experimentell).
- Aktive Datei bauen, starten, debuggen mit eigenen Kürzeln; Art des Reiters (Text, Assembler, BASIC) wählbar.
- Eingebundene Assembler-Dateien werden im Zusammenhang der einbindenden Datei geprüft.

### Studios

- Alle Studios haben denselben Aufbau; Aktionen stehen in der Befehlspalette und im Kontextmenü.
- Disk-Studio: Verzeichnis als Tabelle über die volle Breite, darunter eine Belegungskarte mit Kette der gewählten Datei.
- Bank-Studio: Miniaturen der Screens. Kachel-Export mit wählbarem Layout (Tabellen planar oder verschränkt, Karte zeilen- oder spaltenweise, 8- oder 16-Bit-Index, Lauflängenkodierung); mehrere Kachelsätze je Bank lassen sich exportieren.
- Palette (MEGA65) als Studio; TED-Palette für den Plus/4.
- Pixel-Studio für Sprites, Player/Missile und Zeichensätze mit Auswahl, PNG-Import und Vorlagen.

### Geändert

- Die gesamte Oberfläche wurde neu überarbeitet; Menüs und Kürzel sind in einer Befehlstabelle gesammelt.
- Die Befehlspalette findet Synonyme („petscii“, „kompilieren“ …).
- Handbuch und Hilfe beschreiben die neue Oberfläche, mit Zweck, Grundbegriffen und „Mein erstes …“ je Studio.

### Fehlerkorrekturen

- Tippen in Suchen, Ersetzen, Gehe zu, Palette und Dialogen änderte auch den Code.
- Kürzel mit Enter und Pfeilen kamen im Editor nicht an; dasselbe Kürzel als Menü- und Tastenereignis (macOS) wirkte doppelt.
- „Tests ausführen“ beendete eine laufende Debug-Sitzung.
- Verbindungsabbrüche zum Emulator werden verständlich gemeldet.
- Das Beobachten-Feld im Debug-Panel zeigt ein einzelnes Byte an einer Adresse.
- Eingebundene Dateien meldeten Fehler, die im Zusammenhang der einbindenden Datei keine waren.
- Die Gliederung zeigt Labels im BASIC ohne Zeilennummern.
- Dazu viele kleinere Korrekturen und Verbesserungen.

### Bekannte Einschränkungen

- Die Pakete wurden in frischen Systemen geprüft, aber nicht in allen Umgebungen: echter Desktop-Betrieb unter Linux, Installation auf fremden Rechnern, andere Windows- und Linux-Versionen.
- Der Extraktor (Zeichensätze und Bildschirme aus Programmen holen) und die Anbindung an ein echtes MEGA65 sind noch nicht enthalten.
- Die Umwandlung von BASIC nach Assembler ist experimentell.

## 0.1.0 (Alpha)

Erste öffentliche Version. VintageCoder ist in Entwicklung und enthält mit Sicherheit noch Fehler; Stände und Einschränkungen stehen in
[STATUS.md](STATUS.md).

### Zielsysteme

- Commodore 64, Atari 8 Bit (400/800/XL/XE, PAL und NTSC), Commodore Plus/4 (auch C16) und MEGA65 sind in den Paketen enthalten.
- Commodore 128, VC-20, C16/C116 als eigenes Ziel sowie PET/CBM und CBM-II (die beiden letzten als Vorschau, nur BASIC) sind im Quelltext
  vorhanden und kommen mit einem späteren Paketstand.

### Entwicklungsumgebung

- Quelltext-Editor mit Syntaxfarben, Code-Faltung, Vervollständigung, Hilfe zum Wort am Cursor und Fehlern beim Tippen.
- Projektverwaltung mit Projektbaum, Übersicht, Einstellungen und geführter Ersteinrichtung mit Wiederaufruf; Arbeitsmodi „Code“ und „Debug“.
- Handbuch und Hilfe in der Anwendung (F1), Befehlspalette, Feedback per E-Mail aus der Anwendung (Hilfe ▸ Feedback senden …).
- Studios: Sprites (auch Atari Player/Missile), Zeichensätze, Bildschirme, Kachelkarten, Paletten, Tracker für SID und POKEY,
  PLAY-Tracker für den MEGA65, Disk-Studio (D64, D71, D81, ATR) und Memory Map.
- Debugger mit Haltepunkten (auch bedingt), Datenhaltepunkten (VICE), Registern, Speicher und Disassembler.
- Emulator-Anbindungen: VICE, atari800, Altirra, Fujisan und Xemu; Ultimate 64 / II+ über das Netzwerk.

### Assembler und BASIC

- Eigener 6502-Assembler mit Makros, Scopes, bedingter Übersetzung, Zyklenzählung und Zyklenbudgets, Segmenten und Bänken; Ausgabe als
  PRG, BIN, XEX und Steckmodule.
- BASIC im Originaldialekt des Zielsystems, auf Wunsch ohne Zeilennummern mit Sprungmarken und langen Variablennamen.
- Übersetzung von BASIC nach Assembler (in Arbeit).

### Kommandozeile und Werkzeuge

- Kommandozeile `vcc` für Bauen, Tests, Disk-Images und Medien, mit Hilfe im Terminal, man-Seiten und Vervollständigung.
- Automatische Tests für Assembler-Projekte: statische Prüfungen und Unterprogramm-Tests im Emulator.
- Sprachserver `vc-lsp` für andere Editoren ([Anleitung](docs/sprachserver.md)).
- Auswahl, was ein Paket enthält, über `FEATURES.md` und `tools/release.py` ([Anleitung](docs/release-auswahl.md)).

### Pakete

- macOS (Apple-Chip, `.dmg`), Windows (x86_64, Installer), Linux (x86_64: `.deb` für Debian 13 und Ubuntu 24.04, `.rpm` für Fedora 45
  und openSUSE Tumbleweed, `.tar.gz`). Intel-Macs sowie ARM unter Windows und Linux werden nicht unterstützt.
- Die Pakete sind nicht signiert; Hinweise stehen im README und in den Release-Notizen.

### Bekannte Einschränkungen

- Die Pakete wurden in frischen Systemen geprüft, aber nicht in allen Umgebungen: echter Desktop-Betrieb unter Linux, Installation auf
  fremden Rechnern, andere Windows- und Linux-Versionen.
- Der Extraktor (Zeichensätze und Bildschirme aus Programmen holen) und die Anbindung an ein echtes MEGA65 sind noch nicht enthalten.
