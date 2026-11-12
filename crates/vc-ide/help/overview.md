# Projekt-Übersicht

Der Tab **Übersicht** liegt neben den Quelltexten und öffnet sich mit jedem Projekt. Er zeigt nur Werte, die das Projekt tatsächlich hat. Gibt es noch keinen Build, steht dort „noch nicht gebaut“.

## Kennzahlen
- **Speicher:** die belegten Bytes aus den Segmenten der letzten Übersetzung, mit Adressbereich und Zahl der Segmente (zur genauen Belegung siehe [Memory Map](help:memmap)).
- **Diskette:** Dateiname, freie Blöcke (bei Atari Sektoren) und Zahl der Dateien des Images, das die Projektdatei unter `disk` beschreibt. Steht das Image noch nicht auf der Platte, heißt es „noch nicht gebaut“; hat das Projekt keine Diskette, ist ein Link zu den Projekt-Einstellungen da.
- **Grafik und Klang:** Zahl der Sprites, Zeichensätze, Bildschirme, Kachelkarten, Lieder und Paletten im Projekt.
- **Letzter Build:** „Sauber“ oder „Fehlgeschlagen“, wie lange er her ist, und die Zahl der Programme, Fehler und Warnungen.

## Quellen
Die Liste folgt dem Abschnitt `sources` der `.vcprj` in Baureihenfolge. Je Datei stehen Art (`vcasm`, BASIC mit gewähltem Dialekt), Target und Status: nicht gebaut, gebaut, Warnungen, Fehler oder „Datei fehlt“. Das Startprogramm (`main`, sonst die erste Quelle) trägt ★ und „Startprogramm“. Ein Klick auf den Dateinamen öffnet die Datei, „Projekt-Einstellungen“ bearbeitet die Liste.

## Dateien
Eine Tabelle mit **allen** Dateien des Projekts: Art-Marke wie in der Seitenleiste (ASM, BAS, TRK, SPR, CHR, SCR, MAP, PAL, sonst die Endung), Pfad, Größe und Status. Zuerst steht der Code, das Startprogramm (★) ganz oben, danach alphabetisch; darunter folgen die übrigen Dateien nach Art gruppiert, je Art alphabetisch. Quellen aus der `.vcprj`, die auf der Platte fehlen, stehen mit „Datei fehlt“ dabei. **Doppelklick** öffnet die Datei, wie in der Seitenleiste. Die Tabelle hat die volle Breite; ist der Editorbereich schmal, lässt sie sich seitlich verschieben.

## Layout bei schmalem Platz
Ab etwa 900 px Breite stehen vier Kacheln nebeneinander, Quellen und Studios links, Ziel und Ereignisse rechts. Darunter bilden die Kacheln zwei Reihen, und die Abschnitte stapeln sich untereinander; die Studio-Knöpfe brechen um.

## Studios
Die Schaltflächen legen eine neue Datei an (Sprites, Zeichensatz, Bildschirm, Kacheln, Tracker, Palette) oder öffnen ein Diskettenimage, siehe [Sprites, Zeichensätze, Songs](help:graphics) und [Disk-Studio](help:disks).

## Ziel und Verbindung
- Je Zielsystem des Projekts der Emulator, der dafür eingestellt ist, und ob er gefunden wurde.
- Die Maschinen dieser Systeme. Sie gelten als **ungeprüft**, bis du **Verbindung testen** klickst; der Test läuft im Hintergrund und endet mit „online“ oder „nicht erreichbar“.
- Das Verbindungsdiagramm `vc-app → vc-ide → vc-emu` zeigt die vier Wege zum Ziel: VICE (Binärmonitor), atari800 (Textmonitor), Ultimate (REST) und MEGA65 (UART). Grün heißt: bei den Emulatoren ist einer dieser Sorte für ein Projektsystem eingestellt und gefunden, bei der Hardware war der Test erfolgreich. Rot heißt: der Test schlug fehl. Grau heißt: nicht eingestellt oder noch ungeprüft.
- „F5 startet auf“ nennt das Ziel nach [Einstellungen](help:settings) und „Läuft auf“.

## Ereignisse
Die letzten Zeilen des Ausgabeprotokolls dieser Sitzung mit der Zeit seit ihrem Auftreten.

**Starten (F5)** und **Bauen (F7)** stehen oben rechts, siehe [Bauen und Starten](help:build).
