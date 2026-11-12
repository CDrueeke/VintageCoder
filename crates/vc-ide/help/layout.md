# Oberfläche anpassen

Das Fenster hat ein **festes Gerüst** aus sechs Zonen. Nur ihre Größen lassen sich ziehen, und sie lassen sich ein- und ausblenden; Reiter bleiben in ihrer Zone.

| Zone | Inhalt | Ein-/Ausblenden |
|---|---|---|
| Titelzeile | macOS: Fenstertitel; sonst die **Menüzeile** (Datei, Bearbeiten, Ansicht, Projekt, Debug, Fenster, Hilfe) | – |
| Befehlsleiste | Projekt, Ziel, Starten, Debuggen, Bauen, Suche, Arbeitsplatz | – |
| Aktivitätsleiste | Projekt, Suche, Tests, Referenz, Einstellungen | – |
| Seitenleiste (260 px, 180 bis 420) | je nach Symbol der Aktivitätsleiste: Projektbaum, Projektsuche oder Tests | Strg/Cmd+B |
| Editorbereich | alle Dokumente als Reiter | immer sichtbar |
| Inspektor (300 px, 240 bis 480) | je nach aktivem Dokument, siehe unten (Quelltext: Gliederung, Memory Map, Referenz, Hilfe; Studio: Studio, Memory Map, Hilfe; Debug: Register, Beobachten, Haltepunkte, Hilfe) | Strg/Cmd+Alt+B |
| Fußpanel (220 px, 120 px bis 60 % der Höhe) | Ausgabe, Bauprotokoll, Probleme, Tests (Debug: Disassembler, Speicher, Momentaufnahme, Live-Bild, Ausgabe, Probleme) | Strg/Cmd+J |
| Statusleiste | nur Anzeige: Ziel, Assembler, Cursor, Sprache, Emulator, Tests | – |

Die Reiter von Inspektor und Fußpanel lassen sich innerhalb ihrer Leiste umsortieren (ziehen oder Rechtsklick) und schließen; **Ansicht ▸ Layout zurücksetzen** stellt Standardgrößen und alle Reiter wieder her. Ohne Projekt sind Seitenleiste, Inspektor und Fußpanel ausgeblendet.

## Inspektor und Fußpanel im Einzelnen
Der Inspektor richtet sich nach dem **aktiven Dokument** im Editorbereich:
- **Quelltext:** *Gliederung* (Assembler: Labels, Konstanten und Makros; BASIC: Zeilen, die GOTO/GOSUB/THEN/ON…GOTO anspringen, und REM-Überschriften; Klick springt zur Zeile, das Label am Cursor ist hervorgehoben), *Memory Map*, *Referenz*, *Hilfe*.
- **Studio** (Sprite, Bildschirm, …): *Studio* mit Dateipfad und Größe, den Exportdateien unter `build/export/` und den Stellen im Quelltext, die die Datei oder ihren Namen nennen (Klick springt dorthin); dazu *Memory Map* und *Hilfe*.
- **Debug-Arbeitsplatz:** *Register* (mit Flags und den weiteren Registern des Adapters), *Beobachten*, *Haltepunkte*, *Hilfe*.

Jeder dieser drei Kontexte merkt sich seinen Reiter. Ein Wechsel zwischen zwei Quelltexten ändert nichts; erst der Wechsel zwischen Quelltext und Studio zeigt die Reiter des anderen Kontexts. Ist Willkommen, Übersicht oder Einstellungen aktiv, bleibt der bisherige Kontext.

Im Fußpanel tragen die Reiter **Zahlenmarken**: *Probleme* zeigt die Zahl der Fehler (rot), sonst der Warnungen (gelb); *Tests* zeigt „bestanden/alle“, grün, bei Fehlschlägen rot. Ein Klick auf „Tests …“ in der Statusleiste öffnet den Reiter. Jeder Reiter nennt einen Hinweis, solange es nichts zu zeigen gibt („Noch nichts gebaut“). Die Regeln beim Bauen stehen unter [Bauen und Starten](help:build).

## Aktivitätsleiste und Seitenleiste
Das Symbol bestimmt, was die Seitenleiste zeigt; ein Klick auf das Symbol des gezeigten Inhalts blendet sie aus. Die Wahl wird je Arbeitsplatz gemerkt.
- **Projekt:** der Projektbaum (Kürzel der Dateiart, ★ Startprogramm, Punkt bei ungespeicherten Dateien). Im Kopf legt **+** neue Dateien aller Arten an (Quelltext, Sprite, Zeichensatz, Bildschirm, Kacheln, Palette, Song, PLAY, Test); **…** enthält Projekt-Einstellungen, Diskettenimage, Neu einlesen.
- **Suche** (Strg/Cmd+Umschalt+F): Textsuche in den Textdateien des Projekts (Quelltext, BASIC, `.vctest`, Markdown, JSON …) im Hintergrund. **Aa** beachtet Groß-/Kleinschreibung. Die Treffer stehen nach Datei gruppiert (ein Treffer je Zeile, höchstens 500; danach „Suchbegriff eingrenzen“); ein Klick springt im Editor zur Zeile. „Abbrechen“ oder Esc beendet eine laufende Suche. Ungespeicherte Änderungen offener Dateien werden mitgesucht. Die Befehlssuche (Strg/Cmd+K) bleibt in der Befehlsleiste.
- **Tests:** die Tests des Projekts mit letztem Ergebnis, **Alle ausführen**, ▶ je Testdatei und je Test; ein Klick auf einen Test wählt ihn im Fußpanel „Tests“, das die ausführlichen Ergebnisse zeigt.
- **Referenz** öffnet den Reiter im Inspektor; die Seitenleiste bleibt dabei auf ihrem Inhalt.

## Schmale Fenster
Ab 1280 px Breite ist alles sichtbar. Bei **1000 bis 1279 px** klappt der Inspektor zu einem schmalen Streifen mit senkrechter Beschriftung ein und öffnet als **Schublade** über dem Editor (Klick auf den Streifen oder Strg/Cmd+Alt+B); die Befehlsleiste zeigt nur Symbole. **Unter 1000 px** wird auch die Seitenleiste zur Schublade (Strg/Cmd+B oder Symbol der Aktivitätsleiste) und das Suchfeld der Befehlsleiste zum Symbol. Die Schublade schließt ein Klick in den Editor, Esc oder das Öffnen einer Datei. Der Editorbereich behält mindestens 480 px. Die Schwellen 1280 und 1000 gelten in **Punkten** (Fensterbreite geteilt durch den Zoom), nicht in Bildschirmpixeln: bei gespeichertem Zoom unter 100 % ist das Fenster in Punkten breiter, und es erscheinen entsprechend keine Schubladen – das ist gewollt.

## Befehlsleiste
Links das Projekt (Auswahl: zuletzt geöffnete, neu, öffnen) mit dem Zielchip. In der Mitte **Auf: C64 PAL → VICE x64sc**: ein Feld für das Zielsystem und das Startziel (Emulator oder eine Maschine aus den [Einstellungen](help:settings)), unmittelbar daneben ohne Trenner **Starten** (F5, grün), **Debuggen** (F6, bernstein) und **Bauen** (F7, blau). Rechts die Suche (Strg/Cmd+K: Befehle, Dateien, Studios), der Hinweis „ungespeichert“ und das Segment **Code | Debug**. Einen Speichern-Knopf gibt es nicht: Strg/Cmd+S, der Punkt am Reiter und der Hinweis genügen, vor dem Bauen wird ohnehin gespeichert.

Beim Bauen zeigt der Knopf „Baue …“; das Kreuz daneben bricht ab (ein schon laufender Übersetzungsschritt endet im Hintergrund, sein Ergebnis wird verworfen). Während einer Debug-Sitzung weichen Starten und Debuggen den Knöpfen Pause/Weiter, Schritt, Über, Raus und Stop; das Zielfeld bleibt, ist aber gesperrt. Bei wenig Platz verschwinden zuerst die Knopftexte (unter 1280 px), dann wird die Suche zum Symbol (unter 1000 px) und zuletzt das Projekt zum Namen. Nichts bricht in eine zweite Zeile um.

## Arbeitsplätze
**Code** und **Debug** (Segment rechts in der Befehlsleiste; F6 schaltet beim Debuggen selbst um, das Ende der Sitzung führt zurück) ändern nur, was in Inspektor und Fußpanel steht. Größen und Sichtbarkeit sind je Arbeitsplatz getrennt. Gespeichert werden **nur Zahlen und Schalter** (`layout.json` im Konfigurationsordner), nie ein Reiterbaum; ältere `layout_code.json` und `layout_debug.json` werden nicht mehr gelesen.

## Dateien öffnen
Alles öffnet im **Editorbereich**, immer an derselben Stelle:
- **Einfachklick** im Projektbaum: Vorschau-Reiter (kursiv); ein weiterer Einfachklick auf eine andere Datei ersetzt ihn. Bearbeiten oder Doppelklick auf den Reiter macht ihn fest.
- **Doppelklick**: fester Reiter oder Sprung zum vorhandenen. Jedes Objekt (Datei, Studio, Einstellungen, Hilfe, Willkommen …) gibt es nur einmal als Reiter; ein zweites Öffnen holt den vorhandenen nach vorn. Die Projektdatei (`.vcprj`) öffnet in den **Projekt-Einstellungen**; das JSON zeigt *Als Text öffnen* im Kontextmenü.
- **Strg/Cmd+Klick** (oder Rechtsklick ▸ *Im zweiten Editorbereich öffnen*): im zweiten Bereich; er wird angelegt. Der Editor lässt sich nur einmal teilen (**Strg/Cmd+\**, Reiter-Menü, Ansicht-Menü); Reiter ziehst du zwischen den beiden Hälften. Auf Inspektor und Fußpanel lässt sich nichts ziehen.
- **Strg/Cmd+Umschalt+Eingabe** blendet Seitenleiste, Inspektor und Fußpanel zeitweise aus; derselbe Befehl bringt sie zurück (auch *Ansicht ▸ Maximieren*).

## Studios
Sprites, Zeichensätze, Bildschirme, Kacheln, Paletten, Tracker und PLAY-Tracker öffnen als **Reiter im Editorbereich**, siehe [Sprites, Zeichensätze, Songs](help:graphics). *In eigenem Fenster lösen* (Reiter-Menü, Knopf im Studio, Menü *Fenster*) bleibt als Zusatz, etwa für den zweiten Bildschirm; *Fenster ▸ Gelöste Studios zurückholen* holt sie wieder. Die Wahl wird je Art gemerkt, ebenso Position und Größe des gelösten Fensters (`detached_studios` und `studio_rects` in `layout.json`). Die frühere automatische Fensteranordnung gibt es nicht mehr.

**Farbschema und Größe** (Dunkel, Hell, System; Zoom mit Strg/Cmd+Plus, Minus und 0) stehen im Menü *Ansicht* und in den [Einstellungen](help:settings).

**Eigene Fensterknöpfe:** Unter Windows und Linux liefert das System die Titelleiste. Hat dein Fenstermanager keine (z. B. manche Tiling-Setups), schalte unter *Fenster → Eigene Fensterknöpfe* Minimieren, Maximieren, Schließen und das Ändern der Größe am Fensterrand ein. Unter macOS bleibt die Ampel des Systems.
