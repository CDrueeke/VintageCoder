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

**Ziel:** Du verstehst, was ein VintageCoder-Projekt ist und wie der Dateibaum aufgebaut ist. Du legst ein neues Projekt an, öffnest Beispiele, legst Dateien und Ordner an, benennst sie um, ziehst fremde Dateien ins Projekt, stellst im Hauptbereich **Projekt** Startprogramm und Quellen ein und kopierst ein ganzes Projekt.

## Das brauchst du

- Ein geöffnetes Projekt, zum Beispiel das aus [Dein erstes Projekt](kapitel:erstes-projekt). Für Übungen mit mehreren Dateien eignet sich auch ein Beispiel: Auf der Startseite öffnet **Beispiel öffnen** eine Liste aller Beispiele aus dem Ordner `examples/` der Installation.

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

> **Info:** Hat ein Zielsystem mehrere Dateistrukturen zur Auswahl, bietet der Dialog **Neues Projekt** sie an. Auf der Kommandozeile geht das mit `vcc new … --structure <id>` (`vcc new --list` nennt die Strukturen).

## Die Startseite

Ist kein Projekt offen, zeigt der Hauptbereich **Code** die **Startseite**. Du erreichst sie auch jederzeit über **Datei ▸ Startseite** (das schließt das aktuelle Projekt; bei Ungespeichertem fragt VintageCoder vorher).

![Die Startseite ohne offenes Projekt: der Titel VintageCoder, die Karten Neues Projekt, Projekt öffnen und Beispiel öffnen, der Link Projektdatei (.vcprj) wählen, der Abschnitt Zuletzt geöffnete Projekte (noch leer) und die Beispielprojekte mit Filtern je System und Karten mit Öffnen und Kopieren.](bild:willkommen-seite "Die Startseite erscheint, solange kein Projekt offen ist.")

Links stehen:

- die Knöpfe **Neues Projekt**, **Projekt öffnen**, **Projektdatei (.vcprj) wählen …** und **Beispiel öffnen**,
- die Liste **Zuletzt geöffnete Projekte**. Jede Zeile zeigt das System, den Namen, den gekürzten Pfad und wann du das Projekt zuletzt geöffnet hast. Der Stern heftet ein Projekt an (angeheftete stehen oben), das Menü **…** bietet **Öffnen**, **Kopieren …**, **Im Ordner zeigen** und **Aus der Liste entfernen**. Fehlt der Ordner eines Projekts, steht „Ordner fehlt“ dabei,
- die **Beispielprojekte**: eine Auswahl empfohlener Beispiele, jeweils mit **Öffnen** und **Kopieren …**. **Alle Beispiele …** öffnet einen Dialog mit Suchfeld, in dem die Beispiele nach Zielsystem geordnet stehen.

Rechts stehen die erkannten Emulatoren („bereit“ oder „nicht gefunden“) und eine **Schnellreferenz** mit den wichtigsten Tastenkürzeln.

> **Tipp:** Manche Beispiele lassen sich nur kopieren, nicht direkt öffnen (kein Knopf **Öffnen**). Mit **Kopieren …** legst du eine eigene Kopie an (siehe unten) und änderst die Kopie.

## Schritt für Schritt: Ein neues Projekt anlegen

1. Wähle **Datei ▸ Neues Projekt …** (**Cmd/Strg+Alt+N**) oder klicke auf der Startseite auf **Neues Projekt**. Es öffnet sich der Dialog **Neues Projekt erstellen** mit drei Spalten.
2. Wähle links das **Zielsystem**. Darunter steht, ob das Hardware-Profil gültig ist.
3. Wähle in der Mitte die **Sprache** (**Assembler** oder **BASIC**) und eine **Vorlage**. Unter jeder Vorlage stehen die Dateien, die entstehen.
4. Gib rechts bei **Projektname** einen Namen ein (nur Buchstaben, Ziffern, `_` und `-`) und wähle mit **Wählen …** den **Speicherort**. Je nach System kommen **BASIC-Dialekt**, **Videonorm**, **Speicherausbau**, **Standard-Emulator**, **Startziel (F5)** und **Diskette** dazu. Weiter unten legst du fest, welchen **Zeichensatz des Projekts** du möchtest (**Automatisch**, **Nur Standard (eigen)** oder **Keinen**), ob eine **README.md** und ein **Git-Repository** angelegt werden und ob **weitere Zielsysteme** dazukommen. Die **Dateistruktur** zeigt, was entsteht.
5. Klicke unten auf **Projekt erstellen und öffnen**. *Erwartet:* VintageCoder legt das Projekt an und öffnet es im Hauptbereich **Code**.

![Der Dialog „Neues Projekt erstellen“ über der Startseite. Links die Liste „ZIELSYSTEM“ mit Commodore 64 (gewählt) bis MEGA65 und dem Hardware-Profil „gültig“, in der Mitte die „VORLAGE“ mit „Assembler-Programm“ (gewählt), „Steckmodul“ und „Leeres Projekt“, rechts die „PROJEKT-DETAILS“ mit Projektname, Speicherort, Videonorm, Standard-Emulator, Startziel, Diskette und Zeichensatz. Unten die Knöpfe „Abbrechen“ und „Projekt erstellen und öffnen“.](bild:neues-projekt-dialog "Der Dialog „Neues Projekt erstellen“ mit Zielsystem, Vorlage und Projekt-Details.")

## Der Dateibaum

Der **Dateibaum** steht in der Seitenleiste des Hauptbereichs **Code** (Überschrift „Projekt“). Er zeigt die Ordner und Dateien des Projekts so, wie sie auf der Platte liegen. Bei manchen Dateien steht hinter dem Namen eine Marke, die ihre Art nennt. Die Seitenleiste blendest du mit **Cmd/Strg+B** ein und aus.

*[Bild folgt in E13: projektbaum]*

Ein Klick auf eine Datei öffnet sie als Reiter im Editor, ein Klick auf einen Ordner klappt ihn auf oder zu. Mit den Pfeiltasten, **→**, **←**, **Enter**, **F2** (umbenennen) und **Entf** (löschen) bedienst du den Baum auch ohne Maus. Ziehst du eine Datei oder einen Ordner auf einen anderen Ordner im Baum, wird sie dorthin verschoben; ziehst du sie auf die freie Fläche, kommt sie in den Projektordner.

Oben in der Seitenleiste stehen vier Knöpfe: **Dateien filtern** (blendet ein Suchfeld ein, das den Baum nach Namen filtert), **Neue Datei**, **Neuer Ordner** und **Projekt schließen**.

Welche Datei zum Projekt gehört und welche das Startprogramm ist, steht nicht im Baum, sondern im Hauptbereich **Projekt** (siehe unten).

## Schritt für Schritt: Eine neue Datei anlegen

1. Klicke in der Seitenleiste auf **Neue Datei**. Der Ordner, in dem die Datei entsteht, ist der angeklickte Ordner oder der Ordner der zuletzt angeklickten Datei; sonst der Projektordner. (Dasselbe erreichst du mit Rechtsklick auf einen Ordner oder die freie Fläche ▸ **Neue Datei**, mit **Datei ▸ Neue Datei …** oder über die Befehlspalette.)
2. Es öffnet sich der Dialog **Neue Datei**. Wähle bei **Art** den Typ: **Assembler (.asm)**, **BASIC (.bas)**, **Text (.txt)**, **Notiz (.md)** oder **Ohne Endung**.
3. Tippe bei **Name (Pfad relativ zum gewählten Ordner)** den Namen, zum Beispiel `hilfsroutinen`. Ohne Endung hängt VintageCoder die Endung der Art an. Mit `/` im Namen legst du Unterordner mit an.
4. Klicke auf **Anlegen**. Existiert die Datei schon, meldet der Dialog das.

> **Tipp:** **Cmd/Strg+N** öffnet dagegen einen **unbenannten Reiter**, in dem du sofort schreibst; den Namen vergibst du beim Speichern (Cmd/Strg+S). **Cmd/Strg+Umschalt+T** öffnet eine Notiz. Grafik-Dateien (Sprites, Zeichensätze, Paletten) legst du in ihren Hauptbereichen an (**Grafik**).

### Einen Ordner und eine Notiz anlegen

Klicke einen Ordner im Baum mit der rechten Maustaste an: **Neuer Ordner** legt einen Unterordner an, **Neue Datei** eine Datei in diesem Ordner, **Neue Notiz** eine Notiz (Markdown). Ein Rechtsklick auf die freie Fläche bietet dieselben Einträge im Projektordner. Im Dialog **Neuer Ordner** tippst du den Namen; mit `/` im Namen entstehen mehrere Ordner auf einmal, zum Beispiel `res/bin/fonts`.

## Schritt für Schritt: Umbenennen und Löschen

Klicke eine Datei im Dateibaum mit der **rechten Maustaste** an. Das Kontextmenü bietet:

- **Öffnen**
- **Neue Datei**, **Neue Notiz**, **Neuer Ordner**
- **Umbenennen**, **Löschen**
- **Pfad kopieren** und **Im Ordner zeigen** (öffnet den Ordner im Dateimanager deines Systems)

Auf der freien Fläche bietet das Kontextmenü zusätzlich **Alle zuklappen**, **Neu einlesen** (wenn du Dateien außerhalb von VintageCoder angelegt hast) und **Verschieben rückgängig**.

**Umbenennen:**

1. Wähle **Umbenennen** (oder drücke **F2**). Der Dialog zeigt den alten Namen.
2. Tippe den neuen Namen. Gibt es ihn schon, meldet der Dialog das.
3. Klicke auf **Umbenennen**. VintageCoder führt die Projektdatei nach und passt `.include`- und `.incbin`-Pfade in den Quelltexten an. Wie viele Pfade geändert wurden, meldet es kurz.

> **Achtung:** Umbenennen und Verschieben geht erst, wenn alle geänderten Dateien gespeichert sind; VintageCoder sagt dir das. Kontrolliere nach dem Umbenennen trotzdem die `.include`- und `.incbin`-Zeilen. **Verschieben rückgängig** nimmt das letzte Verschieben im Baum zurück.

**Löschen:** **Löschen** (oder **Entf**) fragt nach und legt die Datei in den **Papierkorb**. Auf Systemen ohne Papierkorb warnt der Dialog, dass die Datei endgültig gelöscht wird.

## Schritt für Schritt: Dateien von außen ins Projekt holen

Du hast eine Grafik, einen Zeichensatz oder einen fertigen Quelltext aus einem anderen Ordner? Ziehe die Datei einfach auf das VintageCoder-Fenster.

1. Ziehe die Datei(en) aus dem Finder oder Explorer auf das Fenster. Legst du eine Projektdatei (`.vcprj`) oder einen Projektordner ab, öffnet VintageCoder stattdessen dieses Projekt. Dateien, die schon im Projekt liegen, werden einfach geöffnet. Ein Diskettenabbild öffnet im Disk-Editor.
2. Liegt eine Datei außerhalb des Projekts, fragt VintageCoder: **Ins Projekt kopieren und öffnen?** Die Dateien stehen darunter.
3. Klicke auf **Kopieren und öffnen**. VintageCoder **kopiert** die Dateien ins Projekt (nichts wird überschrieben) und öffnet sie; deine Originale bleiben, wo sie sind. So bleibt das Projekt in sich geschlossen.

Ohne offenes Projekt lässt sich eine einzelne Datei nicht ablegen; öffne zuerst ein Projekt.

Wie man eine abgelegte Binärdatei im Assembler benutzt, zeigt das Beispiel in der Hilfeseite [Editor](hilfe:editor), Abschnitt „Dateien ins Projekt ziehen“: Eine Zeichensatzdatei `font.bin` legst du in den Ordner `data` ab und bindest sie im Quelltext mit `.incbin "../data/font.bin"` ein.

> **Achtung:** Pfade in `.include` und `.incbin` gelten immer **relativ zur Quelldatei**. Verschiebst du eine eingebundene Datei im Baum oder benennst du sie um, passt VintageCoder die Pfade an. Änderst du Dateien außerhalb von VintageCoder, musst du die Zeilen selbst korrigieren, sonst zeigen die Pfade ins Leere.

## Schritt für Schritt: Ein ganzes Projekt kopieren

Eine Kopie brauchst du, wenn du eine Variante ausprobieren willst, ohne das Original anzufassen, oder wenn du ein Projekt oder Beispiel als Ausgangspunkt für ein neues nehmen möchtest.

1. Öffne das Projekt, das du kopieren willst, oder wähle auf der Startseite bei einem Projekt oder Beispiel **Kopieren …**. VintageCoder speichert beim Kopieren eines offenen Projekts alle offenen Dateien, damit die Kopie den aktuellen Stand hat.
2. Wähle **Datei ▸ Projekt kopieren …** (oder im Hauptbereich **Projekt** den Unterbereich **Weitergeben** ▸ **Projekt kopieren …**).
3. Gib den **Namen der Kopie** ein. Vorgeschlagen wird „<Name> Kopie“. Der Name wird zugleich Ordnername und Name der Projektdatei, deshalb sind Zeichen wie `/ \ : * ? " < > |` nicht erlaubt.
4. Wähle mit **Wählen …** den **Speicherort**. Vorgeschlagen wird der Ordner, in dem auch das Original liegt.
5. Entscheide, was mitkopiert wird. Standardmäßig bleiben der Bauordner `build/` (die Bauergebnisse lassen sich neu erzeugen) und der Ordner `.git/` (die Versionsgeschichte; die Kopie beginnt als neues Projekt) zurück. Mit **Bauordner (build/) mitkopieren** und **Versionsgeschichte (.git/) mitkopieren** nimmst du sie mit. Im Dialog steht, wie viele Dateien und wie viel Platz es sind.
6. Klicke auf **Kopieren**. Mit dem Häkchen **Danach die Kopie öffnen** wechselt VintageCoder gleich in die Kopie.

Die Kopie ist ein eigenständiges Projekt mit eigenem Namen; alle übrigen Einstellungen bleiben gleich. Das Original bleibt unverändert. Ist das Ziel schon belegt, oder läge es im Quellordner, kopiert VintageCoder nichts und sagt dir, warum. Symbolische Verknüpfungen im Projektordner werden nicht kopiert.

## Textdateien: Zeichenkodierung und Zeilenenden

Alle Textdateien im Projekt (Quelltexte, Notizen, Projektdateien) werden als **UTF-8 ohne BOM** mit **LF** als Zeilenende gespeichert, auf jedem System gleich. Öffnest du eine Datei, die anders aufgebaut ist, liest VintageCoder sie trotzdem und zeigt einen Hinweis:

- **BOM entfernt**: Eine UTF-8-Markierung am Dateianfang wird nicht mehr geschrieben.
- **Zeilenenden CRLF → LF**: Windows-Zeilenenden werden umgewandelt.
- **Als Windows-1252 gelesen**: Dateien, die kein gültiges UTF-8 sind (zum Beispiel ältere Texte mit Umlauten), werden als Windows-1252 gelesen und beim Speichern als UTF-8 abgelegt.

Die Datei auf der Platte ändert sich erst, wenn du speicherst. Dateien mit Nullbytes gelten als Binärdateien und werden nicht im Editor angezeigt. Assembler und BASIC-Übersetzer bauen Quelltexte mit BOM oder CRLF ebenfalls fehlerfrei.

## Der Hauptbereich Projekt

Alles, was für **dieses** Projekt gilt, findest du im Hauptbereich **Projekt**. Du erreichst ihn mit dem Knopf **Projekt** in der Kopfzeile, mit **Cmd/Strg+Alt+0** oder über **Ansicht ▸ Bereiche ▸ Bereich Projekt**. In der zweiten Reihe darunter stehen sieben Unterbereiche. Ohne offenes Projekt bittet die Seite dich, zuerst eines zu öffnen.

![Der Bereich Projekt mit der Unterseite Übersicht. Vier Karten mit grünem oder grauem Punkt: Allgemein, Ziele (C64, PAL), Quellen (Programme: 1, Quellgruppen: 1) und Diskette (nicht eingeschaltet), jeweils mit dem Knopf Ändern.](bild:projekt-uebersicht "Die Übersicht des Projekts zeigt Allgemein, Ziele, Quellen und Diskette als Karten.")

Änderungen werden **sofort geprüft und gespeichert**; einen Speichern-Knopf gibt es dafür nicht. Wird ein Wert abgelehnt, meldet die Seite es.

### Übersicht

Für jeden Unterbereich steht eine Karte da (**Allgemein**, **Ziele**, **Quellen**, **Diskette**) mit einem Zustand in Worten und Farbe (in Ordnung, Hinweis, Angabe fehlt, nicht eingeschaltet) und einer Kurzangabe: Name und Startprogramm, die Zielsysteme, die Zahl der Programme und Quellgruppen, ob eine Diskette eingerichtet ist. **Ändern ›** springt zum passenden Unterbereich. Probleme der Projektdatei stehen darunter.

### Allgemein

- **Name** und **Beschreibung** des Projekts. Der Name darf nicht leer sein.
- **Start-Programm:** das Programm, das **F5** startet und beim Bauen zuerst gebaut wird. „Automatisch“ heißt: das erste Programm der Quellen.
- **Beim Öffnen die zuletzt geöffneten Dateien wiederherstellen:** Schalter; **Übernehmen** speichert ihn.

### Ziele

Die Zielsysteme des Projekts, je mit ihrer **Variante** (zum Beispiel PAL oder NTSC), die Takt und Zeilenzahl für Emulator, Vorschau und Zeitmessung bestimmt. Hat das System keine Varianten, steht das dabei. Zielsysteme hinzufügen oder entfernen geht an dieser Stelle noch nicht.

### Quellen

Die Programme des Projekts **in der Baureihenfolge**, geordnet in **Quellgruppen** (je mit Art, Zielsystem und bei BASIC dem Dialekt). Ein Klick auf einen Dateinamen öffnet die Datei im Code-Editor. Das Startprogramm trägt einen Stern (★); **Als Start-Programm** macht eine andere Datei dazu. Mit den Pfeilen nach oben und unten änderst du die Reihenfolge innerhalb einer Gruppe. Fehlt eine Datei auf der Platte, steht sie rot mit dem Wort „fehlt“. Enthält das Projekt BASIC-Quellen, gibt es hier außerdem die Karte **BASIC** mit **Start** und **Schrittweite** der Zeilennummern, **Kompakt bauen** und **Syntaxprüfung beim Bauen**.

### Bauen und Starten

Die Karte **Letzter Bau** zeigt das Ergebnis des letzten Baus in dieser Sitzung (Uhrzeit, ohne Fehler oder Zahl der Fehler) mit den Knöpfen **Bauen** und **Starten**. Die Karte **Emulator je Ziel** nennt Emulator und Startziel je Zielsystem, soweit dieses Projekt sie festlegt. Die Karte **Pakete (Registry)** listet die geladenen Pakete mit ihrer Herkunft. Mehr zum Bauen steht im Kapitel [Bauen und Starten](kapitel:bauen-starten).

### Diskette

Gehört zum Projekt eine Turbo-BASIC-XL-Diskette (Atari, BASIC-Dialekt „Turbo BASIC XL“), erscheint hier die Karte **Turbo BASIC XL** mit dem Zustand der Diskette und den Knöpfen **Bereitstellen …**, **Prüfen**, **Verkleinern …** und **Im Disk-Editor öffnen**. Bei anderen Projekten steht hier ein Hinweis, dass es keine besonderen Einstellungen gibt. Eine Diskette beim Bauen mit erzeugen zu lassen, stellst du in dieser Fassung nicht in dieser Oberfläche ein; die **Übersicht** zeigt, ob eine eingerichtet ist. Mehr zu Disketten steht im Kapitel [Disk-Editor, Segmente, ROMs](kapitel:disk-studio).

### Weitergeben

Hier kopierst du das Projekt (**Projekt kopieren …**, Ablauf siehe oben), zum Beispiel für die Weitergabe oder als Sicherung. Darunter steht ein Hinweis, dass der Export von Grafik, Zeichensätzen und Musik im jeweiligen Editor liegt (Knopf **Export …**; die erzeugten Dateien dürfen frei verwendet werden). Die Liste **Zuletzt geöffnet** zeigt die letzten Projekte mit einem Knopf **Öffnen**.

### Schritt für Schritt: Das Startprogramm ändern

1. Öffne den Hauptbereich **Projekt** und wähle **Quellen**.
2. Klicke bei der Datei, die F5 starten soll, auf **Als Start-Programm**. *Erwartet:* Der Stern ★ wandert zu dieser Datei.
3. Alternativ wählst du im Unterbereich **Allgemein** beim Feld **Start-Programm** die Datei aus der Liste.

## Häufige Stolpersteine

- **Die Datei erscheint nicht im Dateibaum.** Hast du sie außerhalb von VintageCoder angelegt, wähle Rechtsklick auf die freie Fläche ▸ **Neu einlesen**.
- **F5 startet die falsche Datei.** Das Startprogramm steht im Hauptbereich **Projekt**, Unterbereich **Quellen** (Stern ★) und im Unterbereich **Allgemein**.
- **Nach dem Umbenennen findet der Assembler eine Datei nicht mehr.** Prüfe die `.include`- und `.incbin`-Zeilen; haben sich Dateien außerhalb von VintageCoder geändert, musst du sie von Hand korrigieren.
- **Eine Datei steht in den Quellen rot mit „fehlt“.** Die Projektdatei verweist auf eine Datei, die nicht mehr auf der Platte liegt. Lege sie wieder an oder stelle das Startprogramm um.
- **Umbenennen oder Verschieben geht nicht.** Speichere zuerst alle geänderten Dateien (**Cmd/Strg+Alt+S**).

## Wie geht es weiter?

Jetzt weißt du, wo alles liegt. Im nächsten Kapitel geht es darum, wie du Text im [Code-Editor](kapitel:editor) bequem bearbeitest. Danach lernst du [Assembler](kapitel:assembler) und [BASIC](kapitel:basic) kennen.
