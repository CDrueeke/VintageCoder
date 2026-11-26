# Handprüfung E7 Palette-Editor

Start: `start-palette.command` im Paket (öffnet `arbeitskopie/pal_demo`, Datei `graphics/pal65.vcpal`, MEGA65-Palette mit 256 Farben).

| # | Schritte | Erwartet | Ergebnis |
|---|---|---|---|
| 1 | Datei öffnen (Startskript oder Projektbaum) | Grafik ▸ **Palette** öffnet; links das Farbraster (16 × 16), rechts die Karte „Farbe n“ mit Farbfeld, Reglern, Hexwert, Im Einsatz, Ausgabe | |
| 2 | Farbe anklicken; Pfeiltasten, Pos1/Ende, Bild auf/ab | Die Farbe wird gewählt (Rahmen), Nummer und Hexwert stimmen; Pfeil unten = eine Zeile tiefer | |
| 3 | Regler Rot, Grün, Blau ziehen | Farbfeld und Raster folgen sofort, Zahl daneben; ein Zug ist ein Rückgängig-Schritt | |
| 4 | Hexwert eingeben (#rrggbb), Enter oder aus dem Feld gehen; danach Unsinn eingeben | Gültiger Wert wird übernommen; ungültiger zeigt den Hinweis „kein Farbwert“ und wird nicht übernommen | |
| 5 | Rückgängig, Wiederholen (Knöpfe und Cmd/Strg+Z) | Funktioniert wie im Sprite-Editor | |
| 6 | **Standardpalette** | Setzt die Farben zurück (mit Rückgängig zurück) | |
| 7 | **Im Einsatz** bei Farben, die ein Sprite verwendet | Nennt die Dateien (Klick öffnet sie); sonst „in keiner Mediendatei verwendet“ | |
| 8 | Export … | Export-Dialog mit Vorschau (Assembler-Daten, 768 Byte); Speichern schreibt die Datei | |
| 9 | **Tastatur und VoiceOver:** Raster mit Tab fokussieren, Pfeiltasten | Ansage „Farbe n, $hh, #rrggbb“, „n von 256“; alles ohne Maus bedienbar | |
| 10 | Mehrere .vcpal im Projekt | Dateiliste unter dem Raster wechselt die Palette (mit Rückfrage bei Änderungen) | |
