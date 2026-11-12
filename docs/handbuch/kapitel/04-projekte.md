---
id: projekte
titel: Projekte und Dateien
reihenfolge: 4
stufe: Einsteiger
zeit: 20 Minuten
siehe_auch: [oberflaeche, editor, bauen-starten]
referenz: [overview, editor, wizard, layout]
---

# Projekte und Dateien

**Ziel:** Du verstehst, was ein VintageCoder-Projekt ist und wie der Projektbaum aufgebaut ist. Du legst Dateien und Ordner an, benennst sie um, ziehst fremde Dateien ins Projekt, stellst Startprogramm und Diskette in den Projekt-Einstellungen ein und liest die Projekt-Übersicht.

## Das brauchst du

- Ein geöffnetes Projekt, zum Beispiel das aus [Dein erstes Projekt](kapitel:erstes-projekt). Für Übungen mit mehreren Dateien eignet sich auch ein Beispiel: Auf der Willkommensseite öffnet **Beispiel öffnen** den Ordner `examples/` der Installation; wähle dort etwa `c64/border` oder `c64/tests_cycles` und öffne die Datei mit der Endung `.vcprj`.

## Was ist ein Projekt?

Ein **Projekt** ist ein Ordner auf deiner Festplatte. Der wichtigste Inhalt ist die **Projektdatei** mit der Endung `.vcprj`. In ihr steht, für welches Zielsystem du programmierst, welche Quelltexte zum Projekt gehören, welcher davon das **Startprogramm** ist und ob eine Diskette erzeugt werden soll. Daneben liegen deine Quelltexte, Grafiken und Songs. Beim Bauen entsteht ein Unterordner `build/` mit den fertigen Programmen.

Ein Projekt ist **in sich geschlossen**. Du kannst den ganzen Ordner verschieben, kopieren oder in ein Git-Repository stecken, und alles funktioniert weiter. Alle Pfade in der Projektdatei gelten **relativ** zum Projektordner.

Typisch sieht ein frisches Projekt aus dem Assistenten so aus (hier für den C64 mit der Assembler-Vorlage):

```
hallo/
  hallo.vcprj          die Projektdatei
  src/c64/main.asm     dein Quelltext
  build/               entsteht beim Bauen (Programm, Listing, …)
```

> **Info:** Im Projekt-Assistenten kannst du unter **Projektstruktur** wählen, welche zusätzlichen Ordner angelegt werden (*Einfach*, *Standard*, *Groß, modular*, *Mehrere Systeme*, *Bibliothek*, *Spiel mit Levels*). Mit *Einfach* (Vorgabe) entstehen keine zusätzlichen Ordner.

## Der Projektbaum

Der **Projektbaum** ist die Seitenleiste links. Du siehst ihn, wenn in der Aktivitätsleiste das Symbol **Projekt** gewählt ist. Er zeigt alle Dateien und Ordner des Projekts.

![Der Projektbaum eines C64-Projekts: Ordner src/c64 mit main.asm (Marke ASM, Stern als Startprogramm), Ordner graphics mit einer Sprite-Datei (Marke SPR) und einem Zeichensatz (Marke CHR), die Projektdatei hallo.vcprj](bild:projektbaum "Der Projektbaum mit Dateiart-Marken und dem Stern am Startprogramm")

Jede Datei trägt eine **Marke**, die ihre Art zeigt:

| Marke | Art der Datei |
|---|---|
| ASM | Assembler-Quelltext (`.asm`) |
| BAS | BASIC-Quelltext (`.bas`) |
| SPR | Sprite oder Player/Missile-Grafik |
| CHR | Zeichensatz |
| SCR | Bildschirm |
| MAP | Kachelkarte oder Kachelsatz |
| PAL | Palette |
| TRK | Song für den Tracker |
| PLAY | Song für den PLAY-Tracker (MEGA65) |
| BIN, TXT | Binärdaten und Textdateien, die du ins Projekt aufgenommen hast |

Ein **★** vor dem Namen kennzeichnet das **Startprogramm**: das Programm, das F5 startet. Ein Punkt hinter einem Namen bedeutet: Die Datei hat **ungespeicherte Änderungen**.

Klicken und Öffnen funktioniert wie im Kapitel [Die Oberfläche im Überblick](kapitel:oberflaeche) beschrieben: Einfachklick öffnet eine Vorschau, Doppelklick einen festen Reiter, Cmd/Strg+Klick den zweiten Editorbereich. Die Projektdatei (`.vcprj`) öffnet sich beim Doppelklick in den **Projekt-Einstellungen**.

## Schritt für Schritt: Eine neue Datei anlegen

1. Klicke im Kopf der Seitenleiste auf das **+**. Es öffnet sich ein Menü mit:
   - **Quelltext …** (eine neue `.asm`- oder `.bas`-Datei),
   - **Ordner …**,
   - **Sprite …**, **Zeichensatz …**, **Bildschirm …**, **Kacheln …**, **Palette …**, **Song …**, **PLAY …** (Dateien für die Studios),
   - **Test …** (eine Testdatei `.vctest`).
2. Wähle **Quelltext …**. Es erscheint der Dialog **Neue Datei**. Unter dem Titel steht der Ordner, in dem die Datei entsteht („Projektordner“ oder zum Beispiel „src/c64/“).
3. Tippe den **Namen**, zum Beispiel `hilfsroutinen`. Ohne Endung hängt VintageCoder `.asm` bzw. `.bas` an.
4. Klicke auf **Anlegen**. Existiert die Datei schon, erscheint „Diese Datei gibt es schon.“ und **Anlegen** bleibt gesperrt.

![Der Dialog „Neue Datei“ mit dem Eingabefeld Name, dem Hinweis „name (ohne Endung: .asm bzw. .bas)“ und dem Knopf „Anlegen“](bild:datei-neu-dialog "Der Dialog „Neue Datei“")

> **Tipp:** **Cmd/Strg+N** (Menü **Datei ▸ Neue Quelldatei …**) öffnet denselben Dialog direkt. Für Sprites, Zeichensätze, Songs und Co. gibt es das Menü **Datei ▸ Neu: Sprite, Zeichensatz, Song …** oder die Studio-Knöpfe auf der Willkommensseite.

### Einen Ordner anlegen

Wähle im **+**-Menü **Ordner …**, oder klicke einen Ordner im Baum mit der rechten Maustaste an: **Neuer Ordner hier …** legt einen Unterordner an, **Neue Datei hier …** eine Datei in diesem Ordner. Im Namen darfst du Unterordner mit `/` angeben, zum Beispiel `res/bin/fonts`; sie werden mit angelegt. Leere Ordner bleiben im Baum sichtbar.

## Schritt für Schritt: Umbenennen, Duplizieren, Löschen

Klicke eine Datei im Projektbaum mit der **rechten Maustaste** an. Das Kontextmenü bietet bei Dateien unter anderem:

- **Öffnen**
- **★ Als Startprogramm festlegen** und **Nur diese Datei bauen** (bei Quelltexten)
- **Umbenennen …**, **Duplizieren**, **Löschen …**
- **Im Finder zeigen** (unter Windows entsprechend der Dateimanager) und **Pfad kopieren**

**Umbenennen:**

1. Wähle **Umbenennen …**. Der Dialog **Datei umbenennen** zeigt den alten Pfad und ein Feld **Neuer Name**.
2. Tippe den neuen Namen. Gibt es ihn schon, meldet der Dialog „Diese Datei gibt es schon.“
3. Wird die Datei in anderen Quelltexten mit `.include` oder `.incbin` eingebunden, erscheint ein Häkchen **Code anpassen**: „n Pfad(e) in m Quelltext(en) auf den neuen Namen ändern“. Lass es **angehakt**, dann passt VintageCoder die Pfade für dich an. Ohne Anpassung „finden diese Quelltexte die Datei nicht mehr“, wie der Dialog warnt.
4. Klicke auf **Umbenennen**. Die Projektdatei wird nachgeführt.

![Der Dialog „Datei umbenennen“ mit dem Feld „Neuer Name“ und dem Knopf „Umbenennen“](bild:datei-umbenennen-dialog "Beim Umbenennen passt VintageCoder auf Wunsch die Pfade in den Quelltexten an")

**Löschen:** **Löschen …** fragt nach und legt die Datei in den **Papierkorb**. Auf Systemen ohne Papierkorb-Unterstützung warnt der Dialog, dass die Datei endgültig gelöscht wird.

**Startprogramm ändern:** **★ Als Startprogramm festlegen** macht die Datei zum Programm, das F5 und F6 starten.

## Schritt für Schritt: Dateien von außen ins Projekt holen

Du hast eine Grafik, einen Zeichensatz oder einen fertigen Quelltext aus einem anderen Ordner? Ziehe die Datei einfach auf das VintageCoder-Fenster. Das geht auch mit mehreren Dateien und mit ganzen Ordnern.

1. Ziehe die Datei(en) aus dem Finder oder Explorer auf das Fenster.
   - Lässt du sie über dem **Projektbaum** los, ist der Ordner unter dem Mauszeiger das Ziel (auf freier Fläche der Projektordner selbst).
   - Lässt du sie über einem **Assembler-Quelltext** los, fügt VintageCoder nach dem Übernehmen zusätzlich eine `.include`- oder `.incbin`-Zeile an der Cursorposition ein.
2. Es erscheint der Dialog **Dateien ins Projekt übernehmen**. Sie zeigt je Datei eine Zeile mit: Haken zum Übernehmen, Name, erkannter **Art**, **Kategorie** (Quelltext, Text/Doku, Binärdaten, Grafikrohdaten oder Zeichensatz roh), **Ordner** und Ziel.
3. Prüfe die Vorschläge und ändere sie bei Bedarf. Gibt es den Namen schon, wählst du **Umbenennen** (der Name bekommt `_2`, `_3` …), **Ersetzen** oder **Überspringen**.
4. Mit **Für alle übernehmen** gibst du Kategorie, Ordner und Konfliktlösung der ersten Datei an alle weiter.
5. Klicke auf **Übernehmen**. VintageCoder **kopiert** die Dateien ins Projekt; deine Originale bleiben, wo sie sind. So bleibt das Projekt in sich geschlossen.

![Die Rückfrage beim Ablegen von zwei Dateien: je eine Zeile mit Haken, Name, Art, Kategorie, Ordner und Ziel, unten die Auswahl „Umbenennen / Ersetzen / Überspringen“](bild:drag-drop-rueckfrage "Beim Ablegen von Dateien fragt VintageCoder nach Art, Kategorie und Ordner")

Wie man eine abgelegte Binärdatei im Assembler benutzt, zeigt das Beispiel in der Hilfeseite [Editor](hilfe:editor), Abschnitt „Dateien ins Projekt ziehen“: Eine Zeichensatzdatei `font.bin` legst du in den Ordner `data` ab und bindest sie im Quelltext mit `.incbin "../data/font.bin"` ein. Schneller geht es mit einem Rechtsklick auf die Datei im Projektbaum und **Als .incbin einfügen**.

> **Tipp:** Eine Datei, die schon im Projekt liegt, bindest du ohne Tippen ein: aus dem Projektbaum in den Quelltext ziehen oder Rechtsklick ▸ **Als Include einfügen** (siehe [Der Code-Editor](kapitel:editor)).

> **Achtung:** Pfade in `.include` und `.incbin` gelten immer **relativ zur Quelldatei**. Zieht eine eingebundene Datei in einen anderen Ordner oder wird sie umbenannt, fragt VintageCoder, ob die Pfade angepasst werden sollen. Beantworte das mit Ja, sonst zeigen die Pfade ins Leere.

## Die Projekt-Einstellungen

Die Projekt-Einstellungen gelten nur für **dieses** Projekt (die globalen Einstellungen für alle Projekte hast du im Kapitel [Die Oberfläche im Überblick](kapitel:oberflaeche) kennengelernt). Du öffnest sie mit einem Doppelklick auf die `.vcprj`-Datei, über das Menü **Projekt ▸ Projekt-Einstellungen …** oder über **Projekt-Einstellungen …** im Projektmenü der Befehlsleiste. Die Seite hat Abschnitte, die sich aufklappen lassen:

- **Allgemein:** **Name** und **Beschreibung** des Projekts, der Schalter **Beim Öffnen die zuletzt geöffneten Dateien wiederherstellen** und das **Projektbild** (siehe unten).
- **Targets:** für jedes Zielsystem des Projekts die **Videonorm** (PAL/NTSC), beim VC-20 der **Speicherausbau**, der **Standard-Emulator** und das **Startziel (F5)**, jeweils mit „wie in den Einstellungen“ als Vorgabe. Hier legst du auch fest, ob undokumentierte Prozessorbefehle erlaubt sind.
- **Quellen und Startprogramm:** die Liste aller Programme, die beim Bauen übersetzt werden, **in der Baureihenfolge**. Ein ★ markiert das Startprogramm. Die Programme sind in **Quellgruppen** geordnet. Je Gruppe wählst du die **Art** (Assembler oder BASIC), das **Target** und bei BASIC den **BASIC-Dialekt**. Je Datei setzt der Stern (☆/★) das Startprogramm, **Auf** und **Ab** ändern die Reihenfolge, **Entf.** nimmt die Datei aus dem Projekt (sie bleibt auf der Platte erhalten). Über **Datei hinzufügen …** nimmst du eine weitere Datei des Projekts in die Gruppe auf, **Gruppe löschen** entfernt eine ganze Gruppe.
- **Projektstruktur:** wohin abgelegte Dateien je Kategorie kommen.
- **Diskette / Datenträger:** der Schalter **Beim Projektbau ein Diskettenabbild erzeugen**, das **Format** (D64 für C64, D71 für 1571, D81 für 1581 und MEGA65, ATR für Atari) und der **Diskettenname**.

Oben stehen **Speichern** und **Speichern & Projekt bauen**; **Zurücksetzen** verwirft deine Änderungen. Solange etwas ungespeichert ist, steht „● ungespeichert“ da.

![Der Reiter „Projekt-Einstellungen“ mit den Abschnitten Allgemein, Targets, Quellen und Startprogramm, Projektstruktur und Diskette; im Abschnitt Quellen eine Zeile mit Stern für das Startprogramm](bild:projekt-einstellungen "Die Projekt-Einstellungen: Name, Targets, Quellen, Struktur und Diskette")

### Schritt für Schritt: Eine Diskette für das Projekt erzeugen lassen

Manche Programme laden Daten nach und brauchen dafür eine Diskette. So lässt du sie beim Bauen mitbauen:

1. Öffne die **Projekt-Einstellungen** und klappe **Diskette / Datenträger** auf.
2. Hake **Beim Projektbau ein Diskettenabbild erzeugen** an.
3. Wähle ein **Format**, das zum System passt, und gib bei Bedarf einen **Diskettennamen** an.
4. Klicke **Speichern & Projekt bauen**. Das Abbild entsteht neben den gebauten Programmen.

Mehr zu Disketten steht im Kapitel [Disk-Studio, Segmente, ROMs](kapitel:disk-studio).

## Die Projekt-Übersicht

Der Reiter **Übersicht** öffnet sich mit jedem Projekt neben deinen Quelltexten. Er zeigt auf einen Blick den Zustand deines Projekts, nur mit Werten, die das Projekt wirklich hat. Gibt es noch keinen Build, steht dort „noch nicht gebaut“.

![Der Reiter „Übersicht“: oben vier Kacheln (Speicher, Diskette, Grafik und Klang, Letzter Build), darunter die Liste der Quellen mit Stern am Startprogramm und die Dateitabelle](bild:projekt-uebersicht "Die Projekt-Übersicht zeigt Speicher, Diskette, Quellen, Dateien und den letzten Build")

- **Kacheln:** **Speicher** (belegte Bytes laut letzter Übersetzung), **Diskette** (Dateiname, freie Blöcke, Zahl der Dateien), **Grafik und Klang** (Zahl der Sprites, Zeichensätze, Bildschirme, Kachelkarten, Lieder, Paletten) und **Letzter Build** („Sauber“ oder „Fehlgeschlagen“, wie lange er her ist, Zahl der Fehler und Warnungen).
- **Quellen:** die Programme aus der Projektdatei in Baureihenfolge, mit Art, Target und Status (nicht gebaut, gebaut, Warnungen, Fehler oder „Datei fehlt“). Ein Klick auf den Dateinamen öffnet die Datei.
- **Dateien:** eine Tabelle mit **allen** Dateien des Projekts mit Art-Marke, Pfad, Größe und Status. Zuerst steht der Code mit dem Startprogramm ganz oben, danach folgen die übrigen Dateien nach Art gruppiert. **Doppelklick** öffnet eine Datei.
- **Studios:** Knöpfe, die eine neue Datei anlegen (Sprites, Zeichensatz, Bildschirm, Kacheln, Tracker, Palette) oder ein Diskettenimage öffnen.
- **Ziel und Verbindung:** der Emulator je Zielsystem, ob er gefunden wurde, und die Maschinen, die du in den Einstellungen eingetragen hast. Mit **Verbindung testen** prüfst du, ob eine Ultimate oder ein MEGA65 antwortet.
- **Ereignisse:** die letzten Zeilen des Ausgabeprotokolls dieser Sitzung.

Oben rechts stehen außerdem die Knöpfe **Starten (F5)** und **Bauen (F7)**.

## Das Projektbild

Jedes Projekt kann ein **Projektbild** haben: die Datei `projektbild.png` im Projektordner. Es erscheint als kleines Vorschaubild in der Liste **Zuletzt geöffnete Projekte** auf der Willkommensseite, schon bevor du das Projekt öffnest.

So kommst du an ein Projektbild:

- **Automatisch:** Beim ersten Halt im Debugger legt VintageCoder das aktuelle Emulatorbild als Projektbild ab, falls es noch keines gibt (solange in den Projekt-Einstellungen der Schalter **Projektbild verwenden (projektbild.png)** an ist).
- **Von Hand aus dem Emulator:** Im Debug-Arbeitsplatz zeigt der Reiter **Emulator** das Bild. Der Knopf **Als Projektbild setzen** speichert es zugeschnitten.
- **Aus einer Datei:** In den Projekt-Einstellungen, Abschnitt **Allgemein**, ersetzt **Bild wählen …** das Projektbild durch ein PNG oder JPEG. **Löschen** entfernt es.

Das Projektbild wird mit dem Projekt eingecheckt, wenn du Git benutzt.

## Häufige Stolpersteine

- **Die Datei erscheint nicht im Projektbaum.** Hast du sie außerhalb von VintageCoder angelegt, wähle im Menü **…** der Seitenleiste **Neu einlesen**.
- **F5 baut die falsche Datei.** Das Startprogramm steht in den Projekt-Einstellungen (Abschnitt Quellen) und im Projektbaum mit ★. Mit Rechtsklick ▸ **★ Als Startprogramm festlegen** änderst du es.
- **Nach dem Umbenennen findet der Assembler eine Datei nicht mehr.** Du hast **Code anpassen** abgewählt. Korrigiere die `.include`- oder `.incbin`-Zeile von Hand.
- **Eine Datei aus der Projektliste verschwunden.** Haben Quellen der Projektdatei keine Datei mehr auf der Platte, steht in der Übersicht „Datei fehlt“.
- **Im Baum ist ein Eintrag kursiv.** Das ist ein Vorschau-Reiter im Editor, nicht die Datei selbst. Doppelklick macht ihn fest.

## Wie geht es weiter?

Jetzt weißt du, wo alles liegt. Im nächsten Kapitel geht es darum, wie du Text im [Code-Editor](kapitel:editor) bequem bearbeitest. Danach lernst du [Assembler](kapitel:assembler) und [BASIC](kapitel:basic) kennen.
