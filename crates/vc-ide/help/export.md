# Export

Alle Studios (Sprites, Zeichensätze, Bildschirme, Kacheln) haben unten den **Export-Streifen**: die Formate, die Ausgabedatei unter `build/export/` mit dem Chip „aktuell“ oder „veraltet“ (gemessen an der Änderungszeit der Studio-Datei) und die Schaltfläche **Export …**, die den Export-Dialog öffnet. Esc schließt ihn.

## Der Dialog
- **Ausgabeformat** (Segmentschalter):

| Format | Datei in `build/export/` | Inhalt |
|---|---|---|
| Assembler | `<name>.asm` | Daten als `.byte`-Zeilen mit Marken, dazu Konstanten (Farben, Maße, bei Sprites Anzahl und Modi) |
| Konstanten | `<name>.inc` | `<name>_len` je Teil und, wenn eine Adresse angegeben ist, `<name>_addr` |
| Binär | `<name>.bin` | die Rohdaten, bei mehreren Teilen eine Datei je Teil |
| PRG | `<name>.prg` | die Rohdaten mit Ladeadresse |
| BASIC DATA | `<name>.data.bas` | `DATA`-Zeilen mit Marke `@<name>`, einzubinden mit `#include` (siehe [BASIC](help:basic-directives)) |

- **Name:** die Marke, die als Präfix der Namen im Export dient.
- **Adresse:** bei *Konstanten* optional, bei *PRG* die Ladeadresse (`$2000`, `0x2000` oder dezimal). Eine ungültige Eingabe wird rot, bei PRG sperrt sie die Aktionen.
- **Umfang:** die Dateien, die dieses Format erzeugt, mit ihren Größen.
- **Vorschau:** rechnet den aktuellen Stand der Studio-Datei, auch ungespeicherte Änderungen, in einem temporären Ordner aus; dort entsteht nichts im Projekt. Reiter: *Quelltext*, *Hex* (die ersten Bytes der ersten Datei) und *BASIC-DATA* (die Daten als DATA-Zeilen). Fehler stehen rot unter der Vorschau.
- Es gibt keine Auswahl eines Assembler-Dialekts (es gibt nur `vcasm`) und keine Komprimierung.

## Aktionen
- **Im Emulator testen:** speichert, exportiert als Assembler, baut das Projekt und startet es wie F5.
- **In Projekt speichern:** speichert die Studio-Datei und schreibt die Dateien des gewählten Formats nach `build/export/`. Der Dialog bleibt offen.
- **In aktiven Code einfügen:** speichert, exportiert als Assembler und fügt eine `.include`-Zeile in den zuletzt benutzten Assembler-Quelltext ein (das geht auch durch Ziehen der Datei aus dem Projektbaum).

## Automatischer Export
Beim Bauen werden veraltete Studio-Dateien ohnehin nach `build/export/<name>.asm` geschrieben (Einstellung je Projekt, siehe [Einstellungen](help:settings), Toolchain). Der Dialog ist der manuelle Weg mit freier Wahl von Format, Name und Adresse. Auf der Kommandozeile: `vcc media export datei --format asm|inc|bin|prg|data`.
