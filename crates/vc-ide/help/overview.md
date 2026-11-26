# Projekt-Übersicht

Den Zustand deines Projekts siehst du an mehreren Stellen: im Bereich **Projekt**, im Projektbaum, im Panel **Maschine**, in der Speicherkarte (**Daten ▸ Speicher**) und in der Ablage unter dem Editor.

## Seite „Übersicht“ im Bereich Projekt
Der Hauptbereich **Projekt** (Cmd/Strg+Alt+0) hat in der zweiten Reihe die Seiten **Übersicht**, **Allgemein**, **Ziele**, **Quellen**, **Bauen und Starten**, **Diskette** und **Weitergeben**. Die Seite **Übersicht** zeigt vier Karten mit je einem Punkt für den Zustand: grün „in Ordnung“, gelb „Hinweis“ (zum Beispiel keine Programme), rot „Angabe fehlt“ (zum Beispiel der Name) oder grau „nicht eingeschaltet“.

- **Allgemein:** der Projektname und das Start-Programm (ohne Auswahl das erste Programm).
- **Ziele:** die Zielsysteme des Projekts.
- **Quellen:** Zahl der Programme und Quellgruppen; fehlende Dateien werden gezählt.
- **Diskette:** aus, oder Format und Zahl der Programme (ohne Auswahl: alle Programme).

**Ändern ›** springt zur passenden Seite. Gibt es Fehler oder Warnungen in der Projektdatei, stehen sie darunter im Kasten **Probleme der Projektdatei**.

## Projektbaum
Die Seitenleiste **Projekt** (Bereich Code) zeigt alle Dateien mit Art-Marke (siehe [Dateibaum](help:filetree)). Ein Knopf mit Lupe filtert die Dateien nach Namen.

## Speicher
Das rechte Panel **Maschine** (Cmd/Strg+Alt+B) zeigt unter **Letzter Bau** nach dem Bauen, wo dein Programm im Speicher liegt: Quelle, Programm, Größe, Start (SYS) und eine Tabelle der Segmente (zur genauen Belegung siehe [Memory Map](help:memmap)). Vorher steht dort „Noch nicht gebaut. Bauen mit F7.“ Während einer Debug-Sitzung zeigt das Panel außerdem Register, Flags und, wo der Emulator es liefert, das Bild der Maschine. Die **grafische Speicherkarte** findest du im Hauptbereich **Daten** unter **Speicher**; sie zeigt den Adressraum als Raster aus 256 Seiten und das Programm aus dem letzten Bau darin (siehe [Memory Map](help:memmap)).

## Letzter Build
Die Ablage unter dem Editor (Cmd/Strg+J) zeigt im Reiter **Ausgabe**, was beim Bauen und Starten geschehen ist, und im Reiter **Probleme** die Fehler und Warnungen mit Datei und Zeile. Die Statusleiste nennt das Ergebnis („gebaut: … Bytes“ oder „Bau: … Fehler“). Siehe [Bauen und Starten](help:build).

## Ziel und Verbindung
- Der Knopf **Startziel** in der Kopfzeile (zum Beispiel „C64 · VICE x64sc“) zeigt, wohin F5 startet. Ein Klick öffnet die Auswahl.
- Auf der Startseite zeigt der Kasten **Toolchain und Hardware**, welche Emulatoren gefunden wurden („bereit“ oder „nicht gefunden“).
- Die allgemeine Wahl von Emulator und Startziel steht in den Einstellungen, siehe [Einstellungen](help:settings). Die Seite **Ziele** im Bereich Projekt zeigt die Zielsysteme des Projekts mit ihrer Variante (zum Beispiel PAL oder NTSC).

## Editoren
Neue Dateien legst du mit dem Knopf **Neue Datei** über dem Projektbaum an (Art Assembler, BASIC, Text oder Notiz). Sprites, Zeichensätze, Kacheln, Bildschirme und Paletten bearbeitest du im Bereich **Grafik**, Songs im Bereich **Musik** (Tracker), Disketten im Bereich **Daten** (Neue Diskette im Disk-Editor). Das Debuggen hat den Bereich **Debug**. Siehe [Sprites, Zeichensätze, Songs](help:graphics) und [Disk-Editor](help:disks).

**Starten** (F5) und **Bauen** (F7) stehen oben in der Kopfzeile, siehe [Bauen und Starten](help:build).
