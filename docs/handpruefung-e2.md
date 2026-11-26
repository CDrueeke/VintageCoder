# Handprüfung E2: Disk-Editor

Spalten: Nr | Schritt | Erwartung | Ergebnis. Echte Abbilder nur in **Kopien** prüfen. Der Disk-Editor steht unter **Daten ▸ Disk**.

**Zu wissen:** Haltepunkt auf Sektor (E6) und die MEGA65-Übertragung (Hardwareprüfung) fehlen absichtlich. Neu: **Im Emulator einlegen** braucht eingerichtetes VICE (oder atari800/Altirra für ATR).

| Nr | Schritt | Erwartung | Ergebnis |
|---|---|---|---|
| 1 | Daten ▸ Disk ohne Diskette | Hinweis „Keine Diskette geöffnet“ mit „Diskette öffnen …“ und „Neue Diskette …“ | |
| 2 | „Diskette öffnen …“, ein D64 aus der Kopie wählen (auch per Befehlspalette ⌘K „Diskette öffnen“) | Diskette öffnet; Format D64 hervorgehoben; Verzeichnis, Belegungskarte und Sektor 18/0 (Kopf/BAM) sichtbar | |
| 3 | Im Projektbaum eine `.d64`, `.d71`, `.d81` oder `.atr` anklicken | Öffnet im Disk-Editor (kein Textreiter, keine Fehlermeldung) | |
| 4 | Ein D71, ein D81 und eine ATR öffnen | Jeweils richtige Spur-/Sektorzahl; ab 41 Spuren steht die Karte in zwei Spurblöcken; ATR zeigt Sektornummern statt Spur/Sektor | |
| 5 | Verzeichnisnamen prüfen (PETSCII, bei ATR ATASCII) | Namen lesbar; Grafikzeichen als „▒“; Typ, Blöcke, Größe, Start stimmen mit dem Verzeichnis überein (zum Vergleich in VICE `LOAD"$",8`) | |
| 6 | Datei im Verzeichnis anklicken | Ihre Sektorkette leuchtet in der Karte; Inspektor springt auf den ersten Sektor, „Block 1 von n“ | |
| 7 | „Folgen ▶“ mehrmals, dann „Zurück“ | Folgesektor wird gewählt, am Kettenende steht „Kettenende“; Zurück geht Schritt für Schritt | |
| 8 | In die Belegungskarte klicken; danach Pfeiltasten | Sektor wird gewählt (Rahmen), Inspektor zeigt Hex und Zeichen; Pfeile bewegen; VoiceOver sagt „Spur …, Sektor …: …“ an (VoiceOver später in E13) | |
| 9 | „Kette zeigen“ aus- und einschalten | Kette der gewählten Datei verschwindet bzw. erscheint | |
| 10 | Pfeil auf/ab im Verzeichnis | Wählt die nächste/vorige Datei | |
| 11 | „Prüfen“ bei einer heilen Diskette, dann bei einer Kopie mit bewusst beschädigter BAM | Heil: „Keine Probleme gefunden“; beschädigt: Liste der Probleme | |
| 12 | Dateien vormerken: Alle, Keine, Programme, BASIC; einzelne Schalter | Anzahl in „Ins Projekt übernehmen (n)“ stimmt; BASIC wählt nur erkannte BASIC-Programme | |
| 13 | Zielordner `../raus` eintragen | Hinweis „muss innerhalb des Projekts liegen“, Übernehmen gesperrt | |
| 14 | Zielordner `aus_disk`, „Ins Projekt übernehmen“ (BASIC als Quelltext an, Original aus) | Dateien erscheinen im Projektbaum (BASIC als `.bas`, sonst `.prg`); Ergebnisliste unter „Übernahme“ | |
| 15 | Wie 14 mit „Auch als Assembler“ und mit „Originaldatei behalten“ | Zusätzlich `.asm` bzw. die Originaldatei | |
| 16 | „Dateien ablegen …“ auf einer D64-Kopie (eine `.prg` wählen) | Datei erscheint im Verzeichnis; „Image ungespeichert“ und ● vor dem Namen | |
| 17 | „Speichern“ | Hinweis weg; Datei auf der Platte geändert (in VICE nachprüfen) | |
| 18 | „Speichern unter …“ | Kopie wird geschrieben, der Editor zeigt jetzt die Kopie | |
| 19 | „Neue Diskette …“ (D64, Name, Kennung) | Leere Diskette, ungespeichert; Speichern fragt nach dem Ort | |
| 20 | Bei ungespeicherter Diskette „Diskette schließen“; ebenso ⌘Q, roter Knopf, Projekt schließen | Rückfrage (Speichern/Verwerfen/Abbrechen) bzw. die Diskette steht in der Beenden-Liste; „Alle speichern“ speichert auch sie | |
| 21 | Eine andere Diskette öffnen, solange die erste ungespeichert ist | Rückfrage wie in 20 | |
| 22 | „Im Emulator einlegen“ bei gespeicherter D64 | VICE startet mit der Diskette in Laufwerk 8 (`LOAD"$",8` zeigt das Verzeichnis); bei ATR bootet atari800/Altirra sie; bei ungespeicherter Diskette ist der Knopf gesperrt („Erst speichern, dann einlegen“) | |
| 23 | Boot-Diskette ohne Dateisystem (ATR) | „Dateien ablegen“ gesperrt mit Hinweis | |
| 24 | Große Abbilder (D81 voll) | Öffnet ohne spürbare Verzögerung; Karte, Verzeichnis, Inspektor bedienbar | |
| 25 | Hell, Dunkel, Englisch | Farben und Kontrast gut lesbar; Texte englisch; Legende und Beschriftungen vollständig | |
| 26 | Fenster schmal ziehen (unter etwa 1100 und 700 Punkte) | Spalten stapeln sich; nichts ist abgeschnitten; kein waagerechtes Scrollen der Seite | |
| 27 | Beschädigtes Abbild (z. B. Datei mit Text statt Daten, Endung `.d64`) im Baum anklicken | Verständliche Fehlermeldung, die geöffnete Diskette bleibt | |
| 28 | VoiceOver durch die Ansicht | gesammelt in E13 (nicht jetzt) | |
