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

**Ziel:** Du verstehst, was ein VintageCoder-Projekt ist und wie der Projektbaum aufgebaut ist. Du legst Dateien und Ordner an, benennst sie um, ziehst fremde Dateien ins Projekt, stellst Startprogramm und Diskette in den Projekt-Einstellungen ein und kopierst ein ganzes Projekt.

## Das brauchst du

- Ein geöffnetes Projekt, zum Beispiel das aus [Dein erstes Projekt](kapitel:erstes-projekt). Für Übungen mit mehreren Dateien eignet sich auch ein Beispiel: Auf der Startseite klappt **Beispiel öffnen** eine Liste der Beispiele aus dem Ordner `examples/` der Installation auf; wähle dort etwa „c64 / border“ oder „c64 / tests_cycles“.

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

> **Info:** Zusätzliche Ordner lässt du auf der Kommandozeile mit `vcc new … --structure <id>` anlegen (`vcc new --list` nennt die Strukturen). Der Projekt-Assistent der Oberfläche bietet diese Auswahl noch nicht; neue Ordner legst du dort im Projektbaum an (siehe unten).

## Der Projektbaum

Der **Projektbaum** ist die linke Spalte. Du siehst ihn, wenn oben in der linken Spalte **Dateien** gewählt ist (daneben steht **Suche**). Er zeigt die Ordner und Dateien des Projekts so, wie sie auf der Platte liegen; der Bauordner `build/` und versteckte Dateien fehlen. Unter dem Baum steht, wie viele Dateien und Ordner es sind.

![Der Dateibaum eines C64-Projekts mit Ordnern, Dateiart-Marken (ASM, SPR, CHR, SCR, MAP, TRK), dem Stern am Startprogramm und dem Kontextmenü der freien Fläche](bild:projektbaum "Der Dateibaum mit Dateiart-Marken und dem Stern am Startprogramm")

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
| NOT | Notiz (Markdown) |
| BIN, TXT | Binärdaten und Textdateien, die du ins Projekt aufgenommen hast |

Ein **★** hinter dem Namen kennzeichnet das **Startprogramm**: das Programm, das F5 startet. Ein **●** hinter einem Namen bedeutet: Die Datei hat **ungespeicherte Änderungen**. Ein ● hinter einem eingeklappten Ordner sagt: Darin liegt eine geöffnete Datei.

Ein Klick auf eine Datei öffnet sie als Reiter, ein Klick auf einen Ordner klappt ihn auf oder zu. Mit den Pfeiltasten, **→**, **←**, **Enter**, **F2** (umbenennen) und **Entf** (löschen) bedienst du den Baum auch ohne Maus. Ziehst du eine Datei oder einen Ordner auf einen anderen Ordner im Baum, wird sie dorthin verschoben. Die Projektdatei (`.vcprj`) öffnet sich als Text; die **Projekt-Einstellungen** erreichst du über **Projekt ▸ Projekt-Einstellungen …** oder die Befehlspalette (siehe unten).

## Schritt für Schritt: Eine neue Datei anlegen

1. Klicke über dem Baum auf den Knopf **＋** („Neue Datei im gewählten Ordner“). Der Ordner, in dem die Datei entsteht, ist der Ordner der zuletzt angeklickten Datei oder der angeklickte Ordner; sonst der Projektordner. Es öffnet sich der Dialog **Neue Datei**. (Dasselbe erreichst du mit Rechtsklick auf die freie Fläche des Baums ▸ **Neue Datei …**, mit dem Befehl **Datei ▸ Neue Datei …** oder über die Befehlspalette.)
2. Wähle bei **Art** den Typ der Datei: **Assembler**, **BASIC**, **Text**, **Notiz**, die Studio-Arten (zum Beispiel Sprites, Zeichensätze, Bildschirme, Kacheln, Paletten, Tracker) oder eine leere **Diskette** (D64, D71, D81, ATR). Bei Studio-Arten wählst du zusätzlich ein **Profil**.
3. Wähle bei **Ordner** den Zielordner (**(Projektordner)** oder zum Beispiel `src/c64/`) und tippe den **Namen**, zum Beispiel `hilfsroutinen`. Ohne Endung hängt VintageCoder die Endung der Art an (`.asm`, `.bas`, `.txt`, `.md`). Unter dem Feld steht, was entsteht („Wird angelegt: …“).
4. Klicke auf **OK**. Existiert die Datei schon, erscheint „… gibt es schon.“ im Dialog.

![Der Dialog „Neue Datei“ mit den Knöpfen für die Art (Assembler, BASIC, Text, Notiz, Studios, Disketten), der Auswahl „Ordner“, dem Feld „Name“ und den Knöpfen „OK“ und „Abbrechen“.](bild:datei-neu-dialog "Der Dialog „Neue Datei“")

> **Tipp:** **Cmd/Strg+N** (oder das **+** rechts neben den Reitern) öffnet dagegen einen **unbenannten Reiter**, in dem du sofort schreibst; den Namen vergibst du beim Speichern (Cmd/Strg+S). **Cmd/Strg+Umschalt+T** öffnet eine Notiz. Für Sprites, Zeichensätze, Songs und Co. gibt es außerdem die Studio-Knöpfe auf der Startseite.

### Einen Ordner anlegen

Klicke einen Ordner im Baum mit der rechten Maustaste an: **Neuer Ordner …** legt einen Unterordner an, **Neue Datei …** eine Datei in diesem Ordner, **Neue Notiz …** eine Notiz. Ein Rechtsklick auf die freie Fläche bietet dieselben drei Einträge im Projektordner. Im Dialog **Neuer Ordner** wählst du den Ordner und tippst den Namen; im Namen darfst du Unterordner mit `/` angeben, zum Beispiel `res/bin/fonts`; sie werden mit angelegt. Leere Ordner bleiben im Baum sichtbar.

## Schritt für Schritt: Umbenennen und Löschen

Klicke eine Datei im Projektbaum mit der **rechten Maustaste** an. Das Kontextmenü bietet (diese Befehle stehen nur hier und in der Befehlspalette, nicht im Menü **Datei**):

- **Öffnen**
- **Umbenennen …**, **Löschen …**
- **Als Include einfügen** (bindet die Datei an der Cursorposition des offenen Quelltexts ein: `.include` für Quelltexte, `.incbin` für sonstige Dateien), **Pfad kopieren**, **Im Ordner zeigen** (öffnet den Ordner im Dateimanager deines Systems) und **Als Hex ansehen (nur Lesen)**
- **Neue Datei …**, **Neuer Ordner …**, **Neue Notiz …**

**Umbenennen:**

1. Wähle **Umbenennen …** (oder drücke **F2**). Der Dialog **Datei umbenennen** zeigt ein Feld **Name** mit dem alten Namen; der Ordner steht davor und bleibt gleich.
2. Tippe den neuen Namen. Er darf kein `/` enthalten; zum Verschieben ziehst du die Datei im Baum. Gibt es den Namen schon, meldet der Dialog „… gibt es schon.“
3. Klicke auf **OK**. Die Projektdatei wird nachgeführt.

> **Achtung:** Beim Umbenennen führt VintageCoder nur die Projektdatei nach. Steht die Datei in einem Quelltext in einer `.include`- oder `.incbin`-Zeile, musst du diese Zeile selbst ändern. Verschiebst du die Datei dagegen per Ziehen im Baum, passt VintageCoder die Pfade in den Quelltexten an (vorher speichern alle geänderten Dateien; **Verschieben im Baum rückgängig** nimmt den Schritt zurück).

![Der Dialog „Datei umbenennen“ mit dem Feld „Name (der Ordner steht davor)“ und den Knöpfen „OK“ und „Abbrechen“.](bild:datei-umbenennen-dialog "Der Dialog zum Umbenennen einer Datei")

**Löschen:** **Löschen …** (oder **Entf**) fragt nach und legt die Datei in den **Papierkorb**. Auf Systemen ohne Papierkorb-Unterstützung warnt der Dialog, dass die Datei endgültig gelöscht wird.

**Startprogramm ändern:** Das Startprogramm stellst du in den Projekt-Einstellungen ein, Abschnitt **Quellen**: Der Stern ☆/★ neben einer Datei macht sie zum Programm, das F5 und F6 starten.

## Schritt für Schritt: Dateien von außen ins Projekt holen

Du hast eine Grafik, einen Zeichensatz oder einen fertigen Quelltext aus einem anderen Ordner? Ziehe die Datei einfach auf das VintageCoder-Fenster. Das geht auch mit mehreren Dateien und mit ganzen Ordnern.

1. Ziehe die Datei(en) aus dem Finder oder Explorer auf das Fenster. Alle Dateien kommen in den Ordner, den du zuletzt im Baum gewählt hast (sonst in den Projektordner). Legst du eine Projektdatei (`.vcprj`) oder einen Projektordner ab, öffnet VintageCoder stattdessen dieses Projekt; Dateien, die schon im Projekt liegen, werden einfach geöffnet.
2. Es erscheint der Dialog **Dateien ins Projekt übernehmen**. Er zeigt je Datei eine Zeile mit: Haken zum Übernehmen, Name (änderbar), erkannte **Art**, **Ordner im Projekt**, dem Zielpfad hinter dem Pfeil und dem Zustand („neu“, „Name belegt“ oder „schon vorhanden“).
3. Prüfe die Vorschläge und ändere sie bei Bedarf. Gibt es den Namen schon, wählst du **Umbenennen** (der Name bekommt eine Nummer), **Ersetzen** oder **Überspringen**.
4. Klicke auf **übernehmen** (mit der Zahl der Dateien davor). VintageCoder **kopiert** die Dateien ins Projekt; deine Originale bleiben, wo sie sind. So bleibt das Projekt in sich geschlossen.

![Die Rückfrage „Dateien ins Projekt übernehmen“ beim Ablegen von zwei Dateien: je eine Zeile mit Haken, Name, Art, Ordner und Ziel, unten die Knöpfe „2 übernehmen“ und „Abbrechen“.](bild:drag-drop-rueckfrage "Beim Ablegen von Dateien fragt VintageCoder nach Name, Art und Ordner")

Wie man eine abgelegte Binärdatei im Assembler benutzt, zeigt das Beispiel in der Hilfeseite [Editor](hilfe:editor), Abschnitt „Dateien ins Projekt ziehen“: Eine Zeichensatzdatei `font.bin` legst du in den Ordner `data` ab und bindest sie im Quelltext mit `.incbin "../data/font.bin"` ein. Schneller geht es mit einem Rechtsklick auf die Datei im Projektbaum und **Als Include einfügen**: Für eine Binärdatei entsteht dabei die `.incbin`-Zeile.

> **Tipp:** Eine Datei, die schon im Projekt liegt, bindest du ohne Tippen ein: aus dem Projektbaum in den Quelltext ziehen oder Rechtsklick ▸ **Als Include einfügen** (siehe [Der Code-Editor](kapitel:editor)).

> **Achtung:** Pfade in `.include` und `.incbin` gelten immer **relativ zur Quelldatei**. Verschiebst du eine eingebundene Datei im Baum, passt VintageCoder die Pfade an. Benennst du sie nur um, musst du die Zeilen selbst korrigieren, sonst zeigen die Pfade ins Leere.

## Schritt für Schritt: Ein ganzes Projekt kopieren

Eine Kopie brauchst du, wenn du eine Variante ausprobieren willst, ohne das Original anzufassen, oder wenn du ein Projekt als Ausgangspunkt für ein neues nehmen möchtest.

1. Öffne das Projekt, das du kopieren willst. VintageCoder speichert beim Kopieren alle offenen Dateien, damit die Kopie den aktuellen Stand hat.
2. Wähle **Datei ▸ Projekt kopieren …** (oder öffne die Befehlspalette mit **Strg+K** beziehungsweise **Cmd+K** und tippe „kopieren“). Auf der Startseite hat jedes zuletzt geöffnete Projekt außerdem einen Knopf **Kopieren …**.
3. Gib den **Namen der Kopie** ein. Vorgeschlagen wird „<Name> Kopie“, beim zweiten Mal „<Name> Kopie 2“. Der Name wird zugleich Ordnername und Name der Projektdatei, deshalb sind Zeichen wie `/ \ : * ? " < > |` nicht erlaubt.
4. Wähle mit **Wählen …** den **Speicherort**. Vorgeschlagen wird der Ordner, in dem auch das Original liegt.
5. Entscheide, was mitkopiert wird. Standardmäßig bleiben der Bauordner `build/` (die Bauergebnisse lassen sich neu erzeugen) und der Ordner `.git/` (die Versionsgeschichte; die Kopie beginnt als neues Projekt) zurück. Beides lässt sich einschalten. Unter dem Feld steht, wie viele Dateien und wie viel Platz es sind.
6. Klicke auf **Kopieren**. Mit dem Häkchen **Danach die Kopie öffnen** wechselt VintageCoder gleich in die Kopie.

Die Kopie ist ein eigenständiges Projekt: Die Projektdatei heißt wie die Kopie und trägt den neuen Namen, alle übrigen Einstellungen bleiben gleich. Das Original bleibt unverändert. Ist das Ziel schon belegt, oder läge es im Quellordner, kopiert VintageCoder nichts und sagt dir, warum. Symbolische Verknüpfungen im Projektordner werden nicht kopiert; im **Protokoll** steht, was ausgelassen wurde.

## Textdateien: Zeichenkodierung und Zeilenenden

Alle Textdateien im Projekt (Quelltexte, Notizen, Projektdateien) werden als **UTF-8 ohne BOM** mit **LF** als Zeilenende gespeichert, auf jedem System gleich. Öffnest du eine Datei, die anders aufgebaut ist, liest VintageCoder sie trotzdem und zeigt in der Statusleiste (und im Protokoll) einen Hinweis:

- **BOM entfernt**: Eine UTF-8-Markierung am Dateianfang wird nicht mehr geschrieben.
- **Zeilenenden CRLF → LF**: Windows-Zeilenenden werden umgewandelt.
- **Als Windows-1252 gelesen**: Dateien, die kein gültiges UTF-8 sind (zum Beispiel ältere Texte mit Umlauten), werden als Windows-1252 gelesen und beim Speichern als UTF-8 abgelegt.

Die Datei auf der Platte ändert sich erst, wenn du speicherst. Dateien mit Nullbytes gelten als Binärdateien und werden nicht im Editor angezeigt. Assembler und BASIC-Übersetzer bauen Quelltexte mit BOM oder CRLF ebenfalls fehlerfrei.

## Die Projekt-Einstellungen

Die Projekt-Einstellungen gelten nur für **dieses** Projekt (die globalen Einstellungen für alle Projekte hast du im Kapitel [Die Oberfläche im Überblick](kapitel:oberflaeche) kennengelernt). Du öffnest sie über **Projekt ▸ Projekt-Einstellungen …** (Mac-Menü) oder über die Befehlspalette (**Cmd/Strg+K**, Wort „Projekt-Einstellungen“). Die Seite öffnet als Reiter und hat vier **Unterreiter**:

- **Allgemein:** **Name** und **Beschreibung** des Projekts, der Schalter **Beim Öffnen die zuletzt geöffneten Dateien wiederherstellen** und das **Projektbild** (siehe unten).
- **Targets:** für jedes Zielsystem des Projekts die **Videonorm** (Standard, pal, ntsc), beim VC-20 der **Speicherausbau**, der **Standard-Emulator** und das **Startziel (F5)**, jeweils mit „wie in den Einstellungen“ als Vorgabe. Hier legst du auch fest, ob undokumentierte Prozessorbefehle erlaubt sind (**Undokumentierte Befehle**), und kannst Targets hinzufügen oder **Entfernen**.
- **Quellen:** die Liste aller Programme, die beim Bauen übersetzt werden, **in der Baureihenfolge**. Die Programme sind in **Quellgruppen** geordnet. Je Gruppe wählst du die **Art** (Assembler oder BASIC), das **Target** und bei BASIC den **BASIC-Dialekt**. Je Datei setzt der Stern (☆/★) das **Startprogramm**, **Auf** und **Ab** ändern die Reihenfolge, **Entf.** nimmt die Datei aus der Gruppe (sie bleibt auf der Platte erhalten). Unter **Datei hinzufügen:** wählst du eine weitere Datei des Projekts, **Gruppe löschen** entfernt eine ganze Gruppe, **Quellgruppe hinzufügen** legt eine neue an. Ist kein ★ gesetzt, gilt das erste Programm als Startprogramm.
- **Diskette:** der Schalter **Beim Projektbau ein Diskettenabbild erzeugen**, das **Format** (D64 für C64, D71 für 1571, D81 für 1581 und MEGA65, ATR für Atari), der **Diskettenname**, die **Ausgabedatei** und die Auswahl der **Programme auf der Diskette** (nichts angehakt heißt: alle).

Oben stehen **Zurücksetzen** (verwirft deine Änderungen), **Speichern** und **Speichern & Projekt bauen**. Solange etwas ungespeichert ist, steht „● ungespeichert“ da. Ungültige Angaben meldet die Seite oben („Achtung: …“); dann lässt sich nicht speichern.

Rechts neben den Einstellungen steht eine Karte **Übersicht**: Sie zeigt je Unterreiter den Zustand auf einen Blick (Name, Targets, Zahl der Programme und Startprogramm, Diskette an oder aus) und springt mit **Ändern ›** zum passenden Unterreiter.

![Der Reiter „Projekt-Einstellungen“ mit den Unterreitern Allgemein, Targets, Quellen und Diskette, den Feldern Name und Beschreibung und rechts der Übersicht über den Zustand des Projekts.](bild:projekt-einstellungen "Die Projekt-Einstellungen: Name, Targets, Quellen und Diskette")

### Schritt für Schritt: Eine Diskette für das Projekt erzeugen lassen

Manche Programme laden Daten nach und brauchen dafür eine Diskette. So lässt du sie beim Bauen mitbauen:

1. Öffne die **Projekt-Einstellungen** und wähle den Unterreiter **Diskette**.
2. Hake **Beim Projektbau ein Diskettenabbild erzeugen** an.
3. Wähle ein **Format**, das zum System passt, und gib bei Bedarf einen **Diskettennamen** an.
4. Klicke **Speichern & Projekt bauen**. Das Abbild entsteht neben den gebauten Programmen.

Mehr zu Disketten steht im Kapitel [Disk-Studio, Segmente, ROMs](kapitel:disk-studio).

## Das Projektbild

Jedes Projekt kann ein **Projektbild** haben: die Datei `projektbild.png` im Projektordner. Es erscheint als kleines Vorschaubild in der Liste **Zuletzt geöffnete Projekte** auf der Startseite, schon bevor du das Projekt öffnest.

So kommst du an ein Projektbild: In den Projekt-Einstellungen, Unterreiter **Allgemein**, ersetzt **Bild wählen …** das Projektbild durch ein PNG oder JPEG (auch GIF oder BMP). **Löschen** entfernt es. Der Schalter **Projektbild verwenden (projektbild.png)** schaltet die Vorschau ein und aus. Ein automatisches Projektbild aus dem Emulator-Bild gibt es in der Oberfläche noch nicht.

Das Projektbild wird mit dem Projekt eingecheckt, wenn du Git benutzt.

## Häufige Stolpersteine

- **Die Datei erscheint nicht im Projektbaum.** Hast du sie außerhalb von VintageCoder angelegt, klicke über dem Baum auf **⟳** („Baum neu einlesen“).
- **F5 baut die falsche Datei.** Das Startprogramm steht in den Projekt-Einstellungen (Unterreiter Quellen) und im Projektbaum mit ★. Mit dem Stern ☆/★ im Unterreiter **Quellen** der Projekt-Einstellungen änderst du es.
- **Nach dem Umbenennen findet der Assembler eine Datei nicht mehr.** Beim Umbenennen werden die Pfade in den Quelltexten nicht angepasst. Korrigiere die `.include`- oder `.incbin`-Zeile von Hand.
- **Eine Datei aus der Projektliste verschwunden.** Hat eine Quelle der Projektdatei keine Datei mehr auf der Platte, steht ihr Name in den Projekt-Einstellungen (Unterreiter **Quellen**) in Rot.

## Wie geht es weiter?

Jetzt weißt du, wo alles liegt. Im nächsten Kapitel geht es darum, wie du Text im [Code-Editor](kapitel:editor) bequem bearbeitest. Danach lernst du [Assembler](kapitel:assembler) und [BASIC](kapitel:basic) kennen.
