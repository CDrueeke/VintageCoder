# Export

Die Editoren für Sprites, Zeichensätze, Paletten, Kacheln, Bildschirme und Songs haben den Knopf **Exportieren …** (Menü Datei ▸ Exportieren …, **Cmd/Strg+Umschalt+E**). Er öffnet den Export-Dialog für die Datei, die im Editor offen ist. Einen Streifen mit Ausgabedatei und dem Chip „aktuell“ oder „veraltet“ gibt es in der Oberfläche noch nicht.

![Der Dialog „Exportieren“ über dem Sprite-Editor für graphics/hero.vcsprite. Oben die Formate Assembler, Konstanten, Binär, PRG und BASIC DATA (Assembler gewählt), darunter das Feld Name mit „hero“ und der erzeugte Assembler-Text. Unten stehen die bereiten Dateien, ein gelber Hinweis unter „Ziel im Quelltext“ und die Knöpfe „Abbrechen“, „Im Emulator testen“, „In Projekt speichern“, „Als Include einfügen“ und „In Quelltext kopieren“.](bild:grafik-exportieren "Der Export-Dialog mit Format, Name, erzeugtem Text und Zielwahl.")

Die exportierten Dateien und die Formate sind frei nutzbar (siehe [Lizenz und Danksagungen](help:license)).

## Der Dialog
- **Format** (Segmentschalter):

| Format | Datei in `build/export/` | Inhalt |
|---|---|---|
| Assembler | `<name>.asm` | Daten als `.byte`-Zeilen mit Marken, dazu Konstanten (Farben, Maße, bei Sprites Anzahl und Modi) |
| Konstanten | `<name>.inc` | `<name>_len` je Teil und, wenn eine Adresse angegeben ist, `<name>_addr` |
| Binär | `<name>.bin` | die Rohdaten, bei mehreren Teilen eine Datei je Teil |
| PRG | `<name>.prg` | die Rohdaten mit Ladeadresse |
| BASIC DATA | `<name>.data.bas` | `DATA`-Zeilen mit Marke `@<name>`, einzubinden mit `#include` (siehe [BASIC](help:basic-directives)) |

- **Name:** die Marke, die als Präfix der Namen im Export dient.
- **Adresse:** bei *Konstanten* optional, bei *PRG* die Ladeadresse (`$2000`, `0x2000` oder dezimal). Eine ungültige Eingabe wird rot, bei PRG sperrt sie die Aktionen.
- **Dateiliste:** die Dateien, die dieses Format erzeugt, mit ihren Größen.
- **Einleseroutine mitliefern:** nur bei BASIC DATA; die Routine liest die DATA-Zeilen mit POKE ab der Adresse in den Speicher (Aufruf mit GOSUB).
- **Ziel im Quelltext:** der geöffnete Quelltext, in den eingefügt wird (Assembler bei Format Assembler, BASIC bei BASIC DATA).
- **Vorschau:** rechnet den aktuellen Stand der Editor-Datei, auch ungespeicherte Änderungen, in einem temporären Ordner aus; dort entsteht nichts im Projekt. Reiter: *Text* und, außer bei BASIC DATA, *Hex* (die ersten Bytes der ersten Datei). Darunter steht „Bereit: n Dateien, n Byte“ oder, rot, der Fehler.
- Es gibt keine Auswahl eines Assembler-Dialekts (es gibt nur `vcasm`) und keine Komprimierung.

## Aktionen
- **Im Emulator testen:** speichert, exportiert als Assembler, baut das Projekt und startet es wie F5.
- **In Projekt speichern:** speichert die Editor-Datei und schreibt die Dateien des gewählten Formats nach `build/export/`. Der Dialog bleibt offen.
- **Als Include einfügen** (bei Assembler und BASIC DATA): speichert, exportiert und fügt eine `.include`-Zeile in den unter **Ziel im Quelltext** gewählten Quelltext ein. Kein passender Quelltext offen: Öffne zuerst die Datei, in die eingefügt werden soll.
- **In Quelltext kopieren** (bei Assembler und BASIC DATA): schreibt den Text selbst an die Cursorzeile dieses Quelltexts, ohne Include.

## Automatischer Export
Zum Einbinden in ein Programm brauchst du den Export nicht: `.include "graphics/hero.vcsprite"` wandelt die Datei beim Bauen selbst um. Der Dialog ist der manuelle Weg mit freier Wahl von Format, Name und Adresse. Auf der Kommandozeile: `vcc media export datei --format asm|inc|bin|prg|data`.
