# Disk-Studio

Das Disk-Studio öffnet Diskettenimages der Formate **D64**, **D71**, **D81** und **ATR** (Atari DOS 2) als Fenster. Es braucht ein geöffnetes Projekt, weil Dateien aus dem Image dorthin übernommen werden.

Öffnen:
- im Projektbaum Rechtsklick auf ein Image > **Diskettenimage öffnen …**,
- in der Werkzeugleiste des Projektbaums **Image …** (beliebige Datei),
- auf der [Projekt-Übersicht](help:overview) die Schaltfläche **Disk-Image**.

## Bedienung
- **Neu …** legt ein leeres Image an (**Anlegen**), **Öffnen …** lädt eines. **Speichern** schreibt das Image, **Exportieren …** speichert eine Kopie unter neuem Namen.
- **Verzeichnis:** die Dateien mit Namen (PETSCII bei Commodore, ATASCII bei Atari), Art und Größe. Dateien, die man auf die Fläche zieht, werden auf die Diskette geschrieben. Nicht gespeicherte Änderungen gehen beim Schließen verloren.
- **Validieren** prüft BAM bzw. VTOC, Verzeichnis und Dateiketten und meldet Abweichungen.
- **Belegung:** eine Karte aus Spuren und Sektoren. Die Farben unterscheiden Boot oder VTOC, Kopf oder BAM, Verzeichnis, Sektoren einer Datei, belegte Sektoren ohne Datei und Sektoren, die mehrere Ketten erreichen. Ein Klick wählt den Sektor.
- **Sektor-Inspektor:** zeigt die Bytes des gewählten Sektors und, was der Kettenzeiger sagt (bei Atari Dateinummer, Folgesektor und Datenbytes, bei Commodore Zeiger oder „Kettenende“). **Folgen ▶** springt zum Folgesektor, **◀ Zurück** geht den Weg zurück. Beim ATR stehen die Boot-Sektoren 1 bis 3 für sich.
- **Im Emulator einlegen** startet den Standard-Emulator des Systems mit dieser Diskette (Laufwerk 8 bei Commodore, D1: bei Atari). Das Image muss vorher gespeichert sein; ein früher gestarteter Emulator wird beendet.

## Dateien ins Projekt übernehmen
Mit dem Häkchen je Datei wird sie vorgemerkt; **Alle**, **Keine**, **Nur Programme** und **Nur BASIC** wählen mehrere auf einmal. **In das Projekt übernehmen** schreibt die Dateien in den **Zielordner**, der innerhalb des Projekts liegen muss. Optionen:
- **BASIC als Quelltext:** das tokenisierte Programm wird in lesbaren, nummerierten Text zurückübersetzt.
- **Original behalten:** zusätzlich die unveränderte Datei.
- **zusätzlich als Assembler:** Maschinenprogramme als Quelltext. Das ist rein mechanisch, Daten erscheinen als Befehle und müssen von Hand geprüft werden.

## Formate
| Format | System | Aufbau |
|---|---|---|
| D64 | C64, 1541 | 35 Spuren, 683 Sektoren |
| D71 | C64/C128, 1571 | 70 Spuren (2 Seiten), 1366 Sektoren |
| D81 | C64/C128/MEGA65, 1581 | 80 Spuren zu 40 Sektoren, 3200 Sektoren |
| ATR | Atari, DOS 2 | 720 Sektoren zu 128 Byte |

Ein Projekt kann sein Image beim Bauen schreiben lassen: Abschnitt `disk` der Projektdatei (Format, Name, Ausgabedatei, Programme), im Wizard als „Diskette“ wählbar.

## Kommandozeile
`vcc disk <format> -o ausgabe --add datei[:NAME]` erzeugt ein Image (`d64`, `d71`, `d81`, `atr [--boot prog.xex]`). `vcc disk list x.d64` zeigt das Verzeichnis und prüft die BAM, `vcc disk extract x.d64 NAME -o datei` holt eine Datei zurück. Die Images sind mit `c1541` gegengeprüft.
