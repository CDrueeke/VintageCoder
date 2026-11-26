# Handprüfung E11 Hex-Ansicht und Speicherkarte

Start: `start-hex.command` (öffnet das Projekt in Daten ▸ Hex mit `x.prg`) und `start-speicher.command` (Daten ▸ Speicher; erst bauen, Cmd+B).

| # | Schritte | Erwartet | Ergebnis |
|---|---|---|---|
| 1 | Daten ▸ **Hex**; links eine Binärdatei (`test.bin`, `x.prg`) wählen | Zeilen mit Adresse, 16 Bytes, Text; „Nur lesen“; Datei wird nie verändert | |
| 2 | Ein Byte anklicken; Umschalt+Klick; mit der Maus aufziehen | Auswahl hell; rechts Bereich, Byte (dezimal · hex · binär), Wort (klein vorn), Text | |
| 3 | Pfeiltasten, Bild auf/ab, Pos1, Ende im Raster | Auswahl wandert, Umschalt erweitert; Fokus bleibt im Raster | |
| 4 | **Suchen**: `$A9 00` (Bytes) und `HELLO` (Text); Enter oder „Suchen“ | Springt zum Treffer (auch auf andere Seite), wählt ihn; wiederholtes Suchen geht weiter und beginnt am Ende von vorn; sonst „Nichts gefunden.“ | |
| 5 | 8 Byte / 16 Byte | Zeilenbreite wechselt | |
| 6 | Seitenwahl (Anfang, vor, zurück, Ende) bei einer Datei über 4 KB | Seitenanzeige „n / m“ | |
| 7 | Eine `.prg`-Datei öffnen; Doppelklick auf eine Binärdatei im Dateibaum; Rechtsklick ▸ Hex-Ansicht | Ladeadresse wird genannt; Datei öffnet in der Hex-Ansicht, nicht im Standardprogramm (Bilder, PDF, Archive bleiben beim System) | |
| 8 | Sehr große Datei (über 4 MiB) | Hinweis „Gezeigt werden die ersten 4 MiB von …“ | |
| 9 | „Im Standardprogramm öffnen“ | Das Betriebssystem öffnet die Datei | |
| 10 | Daten ▸ **Speicher** ohne Bau | Hinweis „Noch nichts gebaut“ | |
| 11 | Bauen, dann Speicher | 16 × 16 Seiten; Programmseiten mit Balken und Rand; links Bereiche (Klick wählt die erste Seite), Programm belegt, freies RAM | |
| 12 | Seite anklicken oder mit Pfeiltasten wählen | Adresse, Art, Symbole, Quellzeilen (Klick springt in den Code), Belegung | |
| 13 | Prozessorport (C64) / Bankumschaltung (Atari) wählen | Seitenplan ändert sich (ROM, E/A, RAM), Hinweis zum Preset | |
| 14 | Filter Alle, Programm, RAM, ROM, E/A | Nicht passende Seiten verblassen | |
| 15 | Prüfungen (Überlappungen, Stack, Zeropage, Seiten unter ROM/E/A) | Häkchen oder Warnungen mit Text | |
| 16 | **VoiceOver:** Hex-Raster und Karte | Hex: „Adresse 000006, Wert A9“; Karte: „Seite 08, free, 255 Byte belegt“; Pfeiltasten gehen ohne Maus | |
