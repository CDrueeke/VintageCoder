# Projekt-Übersicht

Einen eigenen Reiter „Übersicht“ mit Kacheln gibt es in der Oberfläche nicht. Den Zustand deines Projekts siehst du an mehreren Stellen: in den Projekt-Einstellungen, im Projektbaum, im rechten Panel und in der Schublade.

## Karte „Übersicht“ in den Projekt-Einstellungen
**Projekt ▸ Projekt-Einstellungen …** öffnet einen Reiter mit den Unterreitern **Allgemein**, **Targets**, **Quellen** und **Diskette**. Rechts daneben steht die Karte **Übersicht** mit einer Zeile je Unterreiter. Der Punkt davor zeigt den Zustand: grün (in Ordnung), gelb (Hinweis, zum Beispiel keine Programme), rot (Angabe fehlt, zum Beispiel der Name) oder grau (nicht eingeschaltet).

- **Allgemein:** der Projektname.
- **Targets:** die Zielsysteme des Projekts.
- **Quellen:** Zahl der Programme und Quellgruppen und das Startprogramm (`main`, sonst das erste Programm).
- **Diskette:** aus, oder Format und Zahl der Programme (ohne Auswahl: alle Programme).

**Ändern ›** springt zum passenden Unterreiter. Die Quellen stehen dort in Baureihenfolge; Dateien, die auf der Platte fehlen, erscheinen in Rot.

## Projektbaum
Die linke Spalte zeigt alle Dateien mit Art-Marke (siehe [Dateibaum](help:filetree)), das Startprogramm mit ★ und Ungespeichertes mit ●. Unter dem Baum steht die Zahl der Dateien und Ordner.

## Speicher
Das rechte Panel hat den Reiter **Karte**: Nach dem Bauen zeigt er, wo dein Programm im Speicher liegt (zur genauen Belegung siehe [Memory Map](help:memmap)). Vorher steht dort „Noch nichts gebaut“.

## Letzter Build
Die Schublade zeigt im Reiter **Protokoll**, was beim Bauen und Starten geschehen ist, und im Reiter **Probleme** die Fehler und Warnungen mit Datei und Zeile; die Zahl steht am Reiter. Siehe [Bauen und Starten](help:build).

## Ziel und Verbindung
- Der Knopf **Ziel** in der Kopfzeile („C64 · VICE x64sc ▾“) zeigt, wohin F5 startet: Emulator oder eingetragene Maschine. Ein Klick öffnet die Auswahl für diese Sitzung.
- Auf der Startseite zeigt der Kasten **Toolchain und Hardware**, welche Emulatoren gefunden wurden und welche Maschinen eingetragen sind. Maschinen gelten als **ungeprüft**, bis du **Testen** klickst; der Test läuft im Hintergrund und endet mit „online“ oder „nicht erreichbar“.
- In den Einstellungen (**Zielsysteme und Hardware**) stellst du Emulator, Startziel und Hardware ein; siehe [Einstellungen](help:settings).

## Studios
Neue Dateien für die Studios (Sprites, Zeichensatz, Bildschirm, Kacheln, Tracker, Palette) legst du mit dem Knopf **＋** über dem Projektbaum oder mit den Studio-Knöpfen auf der Startseite an; ein Diskettenimage gibt es dort als Art **Diskette**. Siehe [Sprites, Zeichensätze, Songs](help:graphics) und [Disk-Studio](help:disks).

**Starten** (F5), **Debuggen** (F6) und **Bauen** (F7) stehen oben in der Kopfzeile, siehe [Bauen und Starten](help:build).
