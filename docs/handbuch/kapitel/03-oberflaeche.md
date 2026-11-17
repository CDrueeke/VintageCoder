---
id: oberflaeche
titel: Die Oberfläche im Überblick
reihenfolge: 3
stufe: Einsteiger
zeit: 20 Minuten
siehe_auch: [erstes-projekt, projekte, editor]
referenz: [layout, settings, shortcuts, overview]
---

# Die Oberfläche im Überblick

**Ziel:** Du kennst die Bereiche des VintageCoder-Fensters und weißt, wofür sie da sind. Du kannst sie ein- und ausblenden, einen Bereich groß machen, den **Zen-Modus** benutzen, die Befehlspalette aufrufen und Farbschema und Größe der Oberfläche ändern.

## Das brauchst du

- Ein geöffnetes Projekt, zum Beispiel das aus dem Kapitel [Dein erstes Projekt](kapitel:erstes-projekt). Ist keine Datei offen, zeigt VintageCoder im Dokumentenfeld die **Startseite** (zuletzt geöffnete Projekte, Beispiele, Neues Projekt). Du holst sie mit dem Befehl **Startseite** (Menü **Datei**) jederzeit als Reiter zurück.

## Das Gerüst des Fensters

Das Fenster hat **vier Bereiche um ein Dokumentenfeld** und zwei Dauerleisten. Du kannst die Bereiche in der Größe ziehen, einklappen und groß machen, aber nicht frei umbauen. So ist alles immer dort, wo du es erwartest.

| Zone | Wofür sie da ist |
|---|---|
| **Menüleiste** (nur Windows und Linux, ganz oben) | Datei, Bearbeiten, Ansicht, Projekt, Hilfe mit denselben Befehlen und Kürzeln wie die Menüleiste des Macs; Alt+D, Alt+B, Alt+A, Alt+P, Alt+H öffnen ein Menü (nur hier; auf dem Mac gibt es dieses Kürzel nicht, dort gilt die Menüleiste des Systems; der Buchstabe ist im Menünamen unterstrichen; AltGr auf deutschen Tastaturen löst nichts aus); lange Menüs (Bearbeiten, Ansicht, Projekt) haben Untermenüs: Pfeil rechts oder Enter öffnet eines, Pfeil links geht zurück; im Zen-Modus ausgeblendet |
| **Kopfzeile** | der Projektname, das **Ziel** („C64 · VICE x64sc ▾“; Klick wählt das Startziel: Emulator oder eine eingetragene Maschine), die Knöpfe **Starten**, **Debuggen**, **Bauen**, das **Suchfeld** (öffnet die Befehlspalette) und das Zahnrad für die Einstellungen |
| **Linke Spalte** | umschaltbar zwischen **Dateien** (der Projektbaum: Ordner und Dateien mit Art-Marken) und **Suche** im Projekt |
| **Dokumentenfeld** (Mitte) | alle geöffneten Dateien als **Reiter**: Quelltext, Studios, Notizen, Projekt-Einstellungen, Hilfe, Einstellungen |
| **Schublade** (unten) | **Protokoll**, **Probleme**, **Tests**; während einer Debug-Sitzung zusätzlich der Reiter **Debug** (Register, Disassembler, Speicher, Beobachten) |
| **Rechtes Panel** | **Gliederung**, **Hilfe**, **Zeichen**, **Karte** (Speicherkarte); in einer Debug-Sitzung zusätzlich **Bild** (das Emulator-Bild) |
| **Statusleiste** (ganz unten) | Zustand (bereit, baut, Debug), Zeile und Spalte, Dialekt, Kodierung, Sprachdienst und Update-Hinweis; auf Wunsch der **Takt** des Zielsystems |

![Die Menüleiste im Fenster: oben die Titel Datei, Bearbeiten, Ansicht, Projekt und Hilfe, darunter das geöffnete Menü Bearbeiten mit Kürzeln und den Untermenüs (Suchen und Ersetzen, Markdown, Falten …); nicht mögliche Einträge sind grau und nennen ihren Grund.](bild:menueleiste "Die Menüleiste im Fenster (hier das Menü Bearbeiten mit Untermenüs): unter Windows und Linux so, unter macOS liegt sie in der Menüleiste des Systems")

### Kopfzeile

Von links nach rechts: das Logo, der **Projektname** (darunter die Beschriftung „Projekt“; sie ist kein Menü, die Projekt-Einstellungen öffnest du über **Projekt ▸ Projekt-Einstellungen …** oder die Befehlspalette), das **Ziel**, **Starten** (F5, startet das Startprogramm), **Debuggen** (F6) und **Bauen** (F7), das **Suchfeld** (**Cmd/Strg+K**) und das Zahnrad für die Einstellungen. Das Ziel ist ein Knopf mit dem System und dem Startziel („C64 · VICE x64sc ▾“). Er öffnet eine Liste mit dem Emulator des Systems und den eingetragenen Maschinen; deine Wahl gilt für diese Sitzung. Das dauerhafte Startziel stellst du in den Einstellungen ein. Bei schmalem Fenster zeigt die Kopfzeile nur Symbole; was nicht mehr hineinpasst, steckt in einem Überlaufmenü.

Einen eigenen Speichern-Knopf gibt es im Fenster nicht: **Cmd/Strg+S** speichert, ein Punkt am Reiter zeigt Ungespeichertes. Vor jedem Bauen wird ohnehin gespeichert.

### Linke Spalte

- **Dateien:** der Projektbaum. Mehr dazu im Kapitel [Projekte und Dateien](kapitel:projekte). Mit den Pfeiltasten wählst du Einträge, **→** und **←** klappen Ordner auf und zu, **Enter** öffnet, **F2** benennt um, **Entf** löscht (mit Rückfrage). Neues legst du mit dem Knopf **＋** („Neue Datei im gewählten Ordner“) oder mit Rechtsklick auf die freie Fläche an. Daneben stehen ein Filterfeld, **⟳** („Baum neu einlesen“) und **⊟** („Alle Ordner zuklappen“).
- **Suche** (**Cmd/Strg+Umschalt+F**): sucht Text in allen Textdateien des Projekts.

**Cmd/Strg+B** klappt die Spalte ein und aus.

### Dokumentenfeld

Hier öffnen Dateien als Reiter. **Cmd/Strg+W** schließt (bei Ungespeichertem fragt VintageCoder), **Cmd/Strg+N** öffnet einen **unbenannten Reiter**, in dem du sofort schreibst. **Cmd/Strg+Umschalt+T** öffnet eine **Notiz**; unbenannte Reiter und Notizen sichert VintageCoder alle paar Sekunden lokal. Den Editor kannst du **teilen** (**Cmd/Strg+Alt+T**) und Reiter in die andere Hälfte schicken.

Studios (Sprite, Zeichensatz, Bildschirm, Kacheln, Bank, Palette, Tracker, Diskette) sind **Dokumente wie andere Dateien**: Ihre Datei öffnet im Dokumentenfeld mit dem passenden Studio. Alle Studios haben denselben Aufbau mit sechs Zonen (siehe [Grafik und Klang im Überblick](kapitel:grafik-ueberblick)).

### Schublade und rechtes Panel

Die **Schublade** öffnet sich von selbst, wenn es etwas zu zeigen gibt (Baufehler, Testlauf, Debug), und lässt sich mit **Cmd/Strg+J** ein- und ausklappen. **Protokoll** zeigt, was beim Bauen und Starten geschehen ist, **Probleme** zeigt Fehler und Warnungen mit Datei und Zeile (Klick springt zur Stelle), **Tests** die Testergebnisse.

Das **rechte Panel** ist ein schmaler Streifen mit Symbolen, bis du ihn aufklappst (**Cmd/Strg+Alt+B** oder Klick auf ein Symbol): ≡ **Gliederung** (Marken der Datei), ? **Hilfe**, Ω **Zeichen** (PETSCII/ATASCII und Steuerzeichen zum Einfügen) und ▦ **Karte** (Speicherkarte). In einer Debug-Sitzung kommt ◉ **Bild** dazu, das Emulator-Bild.

## Schritt für Schritt: Bereiche ein- und ausblenden

1. **Cmd/Strg+B**: linke Spalte ein oder aus. *Erwartet:* Der Projektbaum verschwindet, das Dokumentenfeld wird breiter.
2. **Cmd/Strg+J**: Schublade. *Erwartet:* Protokoll, Probleme und Tests klappen aus oder ein.
3. **Cmd/Strg+Alt+B**: rechtes Panel.
4. **Zen-Modus (Cmd/Strg+Umschalt+B):** blendet Spalte, Schublade und Panel gemeinsam aus; in den Studios verschwinden auch Liste und Seitenspalte. *Erwartet:* Es bleiben Kopfzeile, Werkzeugleiste, Fläche und Fußzeile. Derselbe Befehl bringt alles zurück.
5. Die Größen änderst du, indem du den Rand zwischen zwei Bereichen mit der Maus ziehst.

## Einen Bereich groß machen

Jeder Bereich hat oben eine kleine Kopfzeile mit **Minimieren ▁** und **Maximieren ⤢**. Maximieren klappt alle anderen Bereiche zu Streifen ein; **Wiederherstellen ⤡** (oder ein Doppelklick auf die Bereichskopfzeile) bringt die vorige Aufteilung zurück. Mit der Tastatur: **Cmd/Strg+Umschalt+Enter** schaltet den Bereich mit dem Fokus um. Das Dokumentenfeld wird nur maximiert, nie minimiert: Es bleibt immer ein Bereich mit Inhalt.

> **Tipp:** Wenn du dich verklickt hast und nichts mehr stimmt: **Ansicht ▸ Bereiche ▸ Layout zurücksetzen** stellt die Standardgrößen wieder her.

Alle diese Befehle stehen im Menü **Ansicht** und in der Befehlspalette.

## Schmale Fenster

Das Fenster darf schmal werden. Unter etwa 900 Punkten Breite schwebt die linke Spalte über dem Dokument und das Panel legt sich darüber; ganz schmal bleibt eine Seite sichtbar mit einem Zurück-Knopf. Die Studios stellen sich nach der Breite des Dokumentenfelds ein: weniger Karten, kürzere Leisten, der Rest steckt im Menü **⋯**.

## Die Befehlspalette

Die **Befehlspalette** ist die schnellste Art, etwas zu finden: ein Befehl, eine Datei oder eine Aktion des Studios, das gerade offen ist.

1. Drücke **Cmd/Strg+K** oder klicke auf das Suchfeld. Es öffnet sich ein Fenster mit einem Eingabefeld.
2. Tippe ein paar Buchstaben, zum Beispiel `bauen`. Die Liste zeigt passende Befehle, Dateien deines Projekts und Studio-Aktionen („Studio: Frame duplizieren“ …).
3. Wähle mit den Pfeiltasten und drücke Enter, oder klicke auf einen Eintrag. Esc schließt das Fenster.

Befehle, die jetzt nicht möglich sind, sind ausgegraut und nennen den Grund. Hinter den Befehlen steht ihr Tastenkürzel, so lernst du die Kürzel nebenbei.

## Einstellungen

Das Zahnrad (oder **Cmd/Strg+Komma**, auf dem Mac auch **Ansicht ▸ Einstellungen**) öffnet die Einstellungen als Reiter. Sie gelten für **alle Projekte**. Die Seiten:

- **Allgemein:** Farbschema (**Dunkel**, **Hell**, **System**, **Hoher Kontrast**) und Größe der Oberfläche; **Konfiguration sichern** (exportieren, importieren, zurücksetzen); **Einrichtung** (Knopf für den Konfigurationsassistenten); **Fehlersuche** (Debug-Protokoll); **Hilfe** (ob die Bilder des Handbuchs geladen werden); **Updates** (nur in Paketen mit Update-Prüfung) und der Ort der Einstellungsdatei.
- **Zielsysteme und Hardware:** Emulatoren, Startziel und echte Hardware (siehe [Installation und Ersteinrichtung](kapitel:installation) und [Bauen und Starten](kapitel:bauen-starten)). Eine geführte Alternative ist der **Konfigurationsassistent** im Menü **Hilfe**.
- **ROMs:** wo deine ROM-Dateien liegen und was erkannt wurde.
- **Toolchain:** was eingebaut ist (Assembler, Include-Suche, BASIC-Dialekt je System).
- **Editor:** Schriftgröße, Zeilenumbruch, Zeile beim Verlassen formatieren (Assembler, BASIC), Menüleiste anzeigen (Windows und Linux), **Einzeltasten der Studios**, **Kürzel-Diagnose** (schreibt zu jedem Kürzel eine Zeile ins Protokoll; nur zur Fehlersuche, nichts wird übertragen), Tastaturlayout für die Tracker-Klaviertasten, **Tasten auf der Klaviatur**, **Startbild**, **Takt in der Statusleiste** und **Sicherung** unbenannter Reiter.
- **Tastenkürzel:** die Liste der Kürzel mit Suchfeld.

Auf den Seiten **Zielsysteme und Hardware** und **Toolchain** (Assembler-Optionen) gelten Änderungen erst, wenn du unten auf **Speichern** klickst; **Verwerfen** nimmt sie zurück. Darüber steht „● Ungespeicherte Änderungen“. Auswahlen mit mehreren Knöpfen (zum Beispiel Farbschema, Debug-Protokoll, Bilder laden, Update-Häufigkeit, Editor-Schalter) wirken dagegen sofort.

## Farbschema und Größe

In den Einstellungen (Seite **Allgemein**) wählst du **Dunkel**, **Hell**, **System** oder **Hoher Kontrast**. Im Menü **Ansicht** und in der Befehlspalette gibt es dafür **Hell/Dunkel umschalten** und **Hoher Kontrast ein/aus**. Die Größe der ganzen Oberfläche änderst du mit **Cmd/Strg+Plus**, **Cmd/Strg+Minus** und **Cmd/Strg+0** (Originalgröße). Nur die Schrift im Quelltext änderst du über die Befehlspalette (**Cmd/Strg+K**): „Editor-Schrift größer“ und „Editor-Schrift kleiner“ (je ein Punkt, auch im Menü **Ansicht**). „Darstellung zurücksetzen“ stellt Oberflächengröße, Editor-Schrift und Layout auf die Vorgabe zurück; Farbschema und andere Einstellungen bleiben.

## Wenn du von der bisherigen Oberfläche kommst

Beim **allerersten Start** übernimmt VintageCoder Farbschema, Größe und Schrift aus den Einstellungen der bisherigen Oberfläche (Datei `view.json` im Konfigurationsordner), soweit sie vorhanden ist. Die alte Datei wird nur gelesen und bleibt unverändert; das Aussehen der neuen Oberfläche steht danach in einer eigenen Datei (`oberflaeche.json`). Die neue und die bisherige Oberfläche teilen sich denselben Konfigurationsordner: Zielsysteme, Emulatoren, Maschinen (`settings.json`) und die Liste der zuletzt geöffneten Projekte liegen in denselben Dateien wie bisher und bleiben erhalten. Die **Anordnung der Bereiche** beginnt neu. Sichere deine Projekte trotzdem wie immer, bevor du sie in einer neuen Version öffnest.

## Dein Layout wird gemerkt

Du musst Größen und Sichtbarkeit der Bereiche nicht speichern: VintageCoder merkt sie sich selbst. Gespeichert werden nur Zahlen und Schalter, nie Inhalte. Ein maximierter Bereich wird nicht gemerkt: Beim nächsten Start ist das gemerkte Normal-Layout da.

## Häufige Stolpersteine

- **Spalte, Schublade und Panel fehlen.** Du hast vielleicht den **Zen-Modus** (Cmd/Strg+Umschalt+B) oder einen maximierten Bereich eingeschaltet; derselbe Befehl bringt sie zurück. Ist die linke Spalte in einem schmalen Fenster nur ein Streifen, klappt ein Klick auf sein Symbol sie auf.
- **Das Panel ist nur ein schmaler Streifen.** Das ist der Normalfall: Ein Klick auf ein Symbol klappt es auf.
- **Das Layout ist verkorkst.** **Ansicht ▸ Bereiche ▸ Layout zurücksetzen**.
- **Cmd/Strg+B baut nicht.** Das Kürzel blendet die linke Spalte ein und aus. Gebaut wird mit F7.

## Wie geht es weiter?

Im nächsten Kapitel geht es um das, was in der linken Spalte steht: [Projekte und Dateien](kapitel:projekte). Danach lernst du den [Code-Editor](kapitel:editor) kennen.
