---
id: oberflaeche
titel: Die Oberfläche im Überblick
reihenfolge: 3
stufe: Einsteiger
zeit: 15 Minuten
siehe_auch: [erstes-projekt, projekte, editor]
referenz: [layout, settings, shortcuts, overview]
---

# Die Oberfläche im Überblick

**Ziel:** Du kennst die Bereiche des VintageCoder-Fensters und weißt, wofür sie da sind. Du kannst sie ein- und ausblenden, zwischen den beiden Arbeitsplätzen **Code** und **Debug** umschalten, die Befehlspalette benutzen und Farbschema und Größe einstellen.

## Das brauchst du

- Ein geöffnetes Projekt, zum Beispiel das aus dem Kapitel [Dein erstes Projekt](kapitel:erstes-projekt). Ohne Projekt zeigt VintageCoder nur die Willkommensseite; Seitenleiste, Inspektor und Fußpanel sind dann ausgeblendet.

## Das Gerüst des Fensters

Das Fenster hat ein **festes Gerüst** aus Zonen. Du kannst ihre Größe ziehen und sie ein- und ausblenden, aber du kannst sie nicht frei umbauen. Das macht die Oberfläche übersichtlich: Alles ist immer dort, wo du es erwartest.

![Das Gesamtfenster mit sechs nummerierten Marken: ① Befehlsleiste, ② Aktivitätsleiste und Seitenleiste mit dem Projektbaum, ③ Editorbereich, ④ Inspektor, ⑤ Fußpanel, ⑥ Statusleiste](bild:ui-gesamt "Das VintageCoder-Fenster mit allen Zonen")

| Nr. | Zone | Wofür sie da ist |
|---|---|---|
| ① | **Befehlsleiste** | oben: Projekt, Ziel („Auf: …“), die Knöpfe Starten, Debuggen und Bauen, Suche und der Umschalter **Code | Debug** |
| ② | **Aktivitätsleiste** und **Seitenleiste** | links: Symbole für Projekt, Suche, Tests, Referenz und Einstellungen; daneben das, was das gewählte Symbol zeigt (zum Beispiel der Projektbaum) |
| ③ | **Editorbereich** | in der Mitte: alle geöffneten Dateien als Reiter |
| ④ | **Inspektor** | rechts: Zusatzinformationen zum aktiven Dokument (Gliederung, Referenz, Hilfe …) |
| ⑤ | **Fußpanel** | unten: Ausgabe, Bauprotokoll, Probleme, Tests |
| ⑥ | **Statusleiste** | ganz unten: nur Anzeige |

Unter Windows sitzt über der Befehlsleiste außerdem die **Menüzeile** (Datei, Bearbeiten, Ansicht, Projekt, Debug, Fenster, Hilfe). Auf dem Mac steht dieselbe Menüleiste wie bei jedem Programm am oberen Bildschirmrand.

### Befehlsleiste

Von links nach rechts:

- **Projekt**: der Name des Projekts mit dem Zielchip. Ein Klick öffnet eine Auswahl der zuletzt geöffneten Projekte mit „Neues Projekt …“, „Projekt öffnen …“ und „Projekt-Einstellungen …“.
- **Auf: …**: das Zielsystem und das Startziel, zum Beispiel „C64 PAL → VICE x64sc“. Hier wählst du aus, ob F5 im Emulator oder auf einer eingetragenen Hardware-Maschine startet.
- **Starten** (F5, grün), **Debuggen** (F6, bernstein) und **Bauen** (F7, blau).
- **Suche** (Cmd/Strg+K): die Befehlspalette, siehe unten.
- **Code | Debug**: der Schalter für den Arbeitsplatz.

Einen Speichern-Knopf gibt es nicht. Du speicherst mit **Cmd/Strg+S**; ein Punkt am Reiter und der Hinweis „ungespeichert“ in der Befehlsleiste zeigen dir, dass noch etwas offen ist. Vor jedem Bauen wird ohnehin gespeichert.

### Aktivitätsleiste und Seitenleiste

Die schmale Leiste ganz links hat fünf Symbole. Das Symbol bestimmt, was die Seitenleiste daneben zeigt:

- **Projekt:** der Projektbaum mit all deinen Dateien (mehr dazu im Kapitel [Projekte und Dateien](kapitel:projekte)).
- **Suche** (Cmd/Strg+Umschalt+F): sucht Text in allen Textdateien des Projekts.
- **Tests:** die Tests des Projekts (siehe [Automatisch testen](kapitel:tests)).
- **Referenz:** öffnet den Reiter „Referenz“ im Inspektor.
- **Einstellungen:** öffnet die Einstellungen als Reiter.

Ein Klick auf das Symbol, dessen Inhalt gerade zu sehen ist, blendet die Seitenleiste aus.

### Inspektor

Der Inspektor richtet sich nach dem **aktiven Dokument**:

- Bei einem **Quelltext**: **Gliederung** (die Labels, Konstanten und Makros der Datei; bei BASIC die angesprungenen Zeilen; ein Klick springt zur Stelle), **Memory Map**, **Referenz** und **Hilfe**.
- Bei einem **Studio** (Sprite, Zeichensatz …): **Studio** (Dateipfad, Exportdateien, Stellen im Quelltext, die die Datei nennen), **Memory Map** und **Hilfe**.
- Im **Debug-Arbeitsplatz**: **Register**, **Beobachten**, **Haltepunkte** und **Hilfe**.

### Fußpanel

Im Arbeitsplatz **Code** hat das Fußpanel vier Reiter:

- **Ausgabe:** Meldungen vom Bauen, Starten und Debuggen.
- **Bauprotokoll:** was beim Bauen jeder Stufe passiert ist (siehe [Bauen und Starten](kapitel:bauen-starten)).
- **Probleme:** Fehler und Warnungen mit Datei und Zeile. Ein Klick springt zur Stelle.
- **Tests:** Ergebnisse der Tests.

Die Reiter **Probleme** und **Tests** tragen Zahlenmarken: Probleme zeigt rot die Zahl der Fehler (sonst gelb die der Warnungen), Tests zeigt „bestanden/alle“.

### Statusleiste

Die Statusleiste unten zeigt nur an: das **Ziel** (zum Beispiel „Ziel: C64 PAL“), die Assembler-Version, die **Cursorposition** („Zeile 5, Spalte 12“), die **Sprache** der aktiven Datei, eine kurze Meldung zum letzten Vorgang und rechts, ob der **Emulator** des Ziels „bereit“ oder „fehlt“ ist. Hast du Tests ausgeführt, zeigt eine Marke „Tests 13/16“; ein Klick darauf öffnet den Reiter „Tests“.

## Schritt für Schritt: Zonen ein- und ausblenden

1. Drücke **Cmd/Strg+B**. Die Seitenleiste verschwindet. Noch einmal drücken blendet sie wieder ein.
2. **Cmd/Strg+Alt+B** blendet den **Inspektor** ein und aus.
3. **Cmd/Strg+J** blendet das **Fußpanel** ein und aus.
4. **Cmd/Strg+Umschalt+Eingabe** blendet alle drei **zeitweise** aus, damit der Editor den ganzen Platz bekommt. Derselbe Befehl bringt sie zurück (auch im Menü als **Ansicht ▸ Maximieren**).
5. Die Größen der Zonen änderst du, indem du den Rand zwischen zwei Zonen mit der Maus ziehst.

Alle diese Befehle stehen auch im Menü **Ansicht**.

![Dasselbe Fenster mit ausgeblendetem Inspektor und Fußpanel: Der Editor nimmt die ganze Breite und Höhe neben der Seitenleiste ein](bild:ui-zonen "Zonen lassen sich ein- und ausblenden, hier ohne Inspektor und Fußpanel")

> **Tipp:** Wenn du dich verklickt hast und nichts mehr stimmt: **Ansicht ▸ Layout zurücksetzen** stellt die Standardgrößen und alle Reiter wieder her.

## Schmale Fenster

Das Fenster darf schmal werden. Ab **1280 Punkten** Breite ist alles sichtbar. Zwischen **1000 und 1279 Punkten** klappt der Inspektor zu einem schmalen Streifen mit senkrechter Beschriftung zusammen. Ein Klick auf den Streifen (oder Cmd/Strg+Alt+B) öffnet ihn als **Schublade** über dem Editor. **Unter 1000 Punkten** wird auch die Seitenleiste zur Schublade. Eine Schublade schließt du mit einem Klick in den Editor, mit Esc oder indem du eine Datei öffnest. Der Editorbereich behält immer mindestens 480 Punkte.

## Reiter: Vorschau oder fest

Alle Dokumente öffnen im **Editorbereich** als Reiter, immer an derselben Stelle:

- **Einfachklick** auf eine Datei im Projektbaum öffnet einen **Vorschau-Reiter** (der Titel steht kursiv). Ein weiterer Einfachklick auf eine andere Datei ersetzt ihn, sodass sich nicht Dutzende Reiter ansammeln.
- **Doppelklick** auf die Datei (oder auf den Reiter) macht den Reiter **fest**. Auch sobald du in der Datei etwas änderst, wird er fest.
- Jede Datei gibt es nur **einmal** als Reiter: Öffnest du sie ein zweites Mal, kommt der vorhandene Reiter nach vorn.
- **Cmd/Strg+Klick** auf eine Datei (oder Rechtsklick ▸ „Im zweiten Editorbereich öffnen“) öffnet sie im **zweiten Editorbereich**. Den Editor kannst du genau einmal teilen (**Cmd/Strg+\\**); Reiter ziehst du mit der Maus zwischen den beiden Hälften.
- **Cmd/Strg+W** schließt die aktive Datei.

![Die Reiterleiste des Editors mit festen Reitern und der Marke der Dateiart](bild:ui-reiter "Feste Reiter und ein kursiver Vorschau-Reiter")

Neben den Quelltexten gibt es besondere Reiter: **Willkommen**, **Übersicht** (die [Projekt-Übersicht](kapitel:projekte)), **Einstellungen**, **Hilfe** und die Studios für Grafik und Klang.

## Schritt für Schritt: Zwischen Code und Debug umschalten

Mit dem Schalter **Code | Debug** rechts in der Befehlsleiste (oder im Menü **Ansicht ▸ Arbeitsplatz Code / Arbeitsplatz Debug**) wechselst du den **Arbeitsplatz**. Er ändert nur, was Inspektor und Fußpanel zeigen:

| | Code | Debug |
|---|---|---|
| Inspektor | Gliederung, Memory Map, Referenz, Hilfe | Register, Beobachten, Haltepunkte, Hilfe |
| Fußpanel | Ausgabe, Bauprotokoll, Probleme, Tests | Disassembler, Speicher, Emulator, Momentaufnahme, Live-Bild, Ausgabe, Probleme |

F6 schaltet beim Debuggen selbst auf den Debug-Arbeitsplatz um, das Ende der Debug-Sitzung führt zurück zu „Code“. Größen und Sichtbarkeit der Zonen merkt sich VintageCoder für jeden Arbeitsplatz getrennt. Die Panels des Debuggers beschreibt das Kapitel [Der Debugger](kapitel:debugger).

![Der Arbeitsplatz Code: im Fußpanel die Reiter Ausgabe, Bauprotokoll, Probleme und Tests, im Inspektor Gliederung, Memory Map, Referenz und Hilfe](bild:ui-modus-code "Arbeitsplatz Code")

![Der Arbeitsplatz Debug: im Inspektor Register, Beobachten und Haltepunkte, im Fußpanel Disassembler, Speicher und weitere Reiter](bild:ui-modus-debug "Arbeitsplatz Debug")

## Schritt für Schritt: Die Befehlspalette

Die **Befehlspalette** ist die schnellste Art, etwas zu finden: ein Befehl, eine Datei oder ein Studio.

1. Drücke **Cmd/Strg+K** (auch **Cmd/Strg+P** geht) oder klicke auf das Suchfeld in der Befehlsleiste. Es öffnet sich ein Fenster mit einem Eingabefeld „Befehl, Datei oder Studio suchen …“.
2. Tippe ein paar Buchstaben, zum Beispiel `bauen`. Die Liste darunter zeigt passende Befehle, Dateien deines Projekts und Studios.
3. Wähle mit den Pfeiltasten aus und drücke Enter, oder klicke auf einen Eintrag. Esc schließt das Fenster.

Befehle, die jetzt nicht möglich sind (zum Beispiel „Speichern“ ohne offene Datei), sind nicht wählbar. Hinter den Befehlen steht ihr Tastenkürzel, so lernst du die Kürzel nebenbei.

![Die Befehlspalette mit dem Suchtext „bau“: Treffer sind „Projekt bauen F7“, „Aktive Datei bauen Umschalt+F7“ und weitere](bild:befehlspalette "Die Befehlspalette (Cmd/Strg+K) findet Befehle, Dateien und Studios")

## Einstellungen

**Datei ▸ Einstellungen …** (auf dem Mac im Menü „VintageCoder“; Kürzel **Cmd/Strg+Komma**) öffnet die Einstellungen als Reiter. Sie gelten für **alle Projekte**. Die Seiten:

- **Allgemein:** Farbschema und Größe der Oberfläche.
- **Zielsysteme und Hardware:** Emulatoren, Startziel und echte Hardware (siehe [Installation und Ersteinrichtung](kapitel:installation) und [Bauen und Starten](kapitel:bauen-starten)). Eine geführte Alternative ist der **Konfigurationsassistent** (Menü **Hilfe ▸ Konfigurationsassistent**), der dieselben Einstellungen Schritt für Schritt abfragt und vor dem Schreiben eine Änderungsliste zeigt.
- **Toolchain:** zeigt, was eingebaut ist (Assembler, Include-Suche, BASIC-Dialekt je System).
- **Editor:** Schriftgröße, Adressspalte und Einzeltasten in den Grafik-Editoren.
- **Tastenkürzel:** die Liste ist fest; Anpassen ist noch nicht möglich.

Außerdem gibt es die Seite **ROMs**. Änderungen an Zielsystemen und Hardware gelten erst, wenn du unten auf **Änderungen anwenden** klickst; **Zurücksetzen** verwirft sie.

![Der Reiter Einstellungen mit der Seitenleiste der Seiten (Allgemein, Zielsysteme und Hardware, Toolchain, Editor, Tastenkürzel) und der Seite Allgemein mit Farbschema und Größe](bild:einstellungen "Der Reiter „Einstellungen“")

## Farbschema und Größe

Du hast die Wahl zwischen **Dunkel**, **Hell** und **Wie das System**. Du findest sie im Menü **Ansicht ▸ Farbschema** und in den Einstellungen unter **Allgemein**. Die Größe der ganzen Oberfläche änderst du mit **Cmd/Strg+Plus** (größer), **Cmd/Strg+Minus** (kleiner) und **Cmd/Strg+0** (Originalgröße). Nur die Schrift des Editors ändern die Einträge **Ansicht ▸ Editor-Schrift größer / kleiner**. **Darstellung zurücksetzen** stellt Schema und Größe wieder her.

![Links das Farbschema Dunkel, rechts Hell: derselbe Ausschnitt des Fensters in beiden Schemata](bild:theme-hell-dunkel "Das Farbschema Dunkel und Hell im Vergleich")

> **Info:** Alle Bilder in diesem Handbuch zeigen das dunkle Schema.

## Dein Layout wird gemerkt

Du musst Größen und Sichtbarkeit der Zonen nicht speichern: VintageCoder merkt sie sich selbst, getrennt für **Code** und **Debug**. Gespeichert werden nur Zahlen und Schalter, in der Datei `layout.json` im Konfigurationsordner. Mit **Ansicht ▸ Layout zurücksetzen** kehrst du zu den Standardwerten zurück.

## Häufige Stolpersteine

- **Seitenleiste, Inspektor und Fußpanel fehlen.** Ohne geöffnetes Projekt sind sie ausgeblendet. Öffne ein Projekt. Sind sie trotz Projekt weg, hast du sie vielleicht mit Cmd/Strg+Umschalt+Eingabe ausgeblendet; derselbe Befehl bringt sie zurück.
- **Der Inspektor ist nur ein schmaler Streifen.** Das Fenster ist schmaler als 1280 Punkte; klicke auf den Streifen, um ihn als Schublade zu öffnen, oder vergrößere das Fenster.
- **Ein Reiter ersetzt sich ständig.** Das ist ein Vorschau-Reiter (Titel kursiv). Doppelklick macht ihn fest.
- **Das Layout ist verkorkst.** **Ansicht ▸ Layout zurücksetzen**.
- **Cmd/Strg+B baut nicht.** Das Kürzel blendet die Seitenleiste ein und aus. Gebaut wird mit F7.

## Wie geht es weiter?

Im nächsten Kapitel geht es um das, was in der Seitenleiste steht: [Projekte und Dateien](kapitel:projekte). Danach lernst du den [Code-Editor](kapitel:editor) kennen.
