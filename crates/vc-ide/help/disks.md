# Disk-Editor

Der Disk-Editor öffnet Diskettenimages der Formate **D64**, **D71**, **D81** und **ATR** (Atari DOS 2). Du findest ihn im Hauptbereich **Daten**, zweite Reihe **Disk**. Er läuft auch ohne Projekt; nur zum Übernehmen von Dateien ins Projekt muss eines offen sein.

Öffnen:
- Klick auf das Image im Projektbaum,
- Knopf **Diskette öffnen …** im Disk-Editor,
- das Image aus Finder oder Explorer auf das Fenster ziehen,
- **Baum ▸ Diskette öffnen …** (Mac-Menü; sonst die Befehlspalette Cmd/Strg+K).

Ein leeres Image legst du mit **Neue Diskette …** an (Format D64, D71, D81 oder ATR, dazu Name und Kennung; bei ATR ohne). Es ist immer nur ein Image offen.

## Bedienung
Links die Karte **Diskette** (Format, Belegung, Aktionen), in der Mitte **Verzeichnis** und **Belegungskarte**, rechts der **Sektor**.
- **Speichern** schreibt das Image, **Speichern unter …** speichert eine Kopie unter neuem Namen. **Dateien ablegen …** legt Dateien auf die Diskette (nicht bei einer Diskette ohne Dateisystem). Cmd/Strg+S speichert im Disk-Editor nicht. Nicht gespeicherte Änderungen („Image ungespeichert“) gehen beim Schließen verloren; **Diskette schließen** fragt vorher nach.
- **Verzeichnis:** die Dateien mit Namen (PETSCII bei Commodore, ATASCII bei Atari), Typ, Blöcken, Größe und Start. Ein Schloss zeigt eine geschützte Datei, **Autostart** die Startdatei.
- **Prüfen** prüft BAM bzw. VTOC, Verzeichnis und Dateiketten; das Ergebnis steht im **Prüfbericht**.
- **Belegungskarte:** eine Zeile je Spur, ein Kästchen je Sektor. Die Farben bedeuten Gewählte Datei, Belegt, Verzeichnis und Kopf/BAM; Sektoren mit Fehlern sind rot. **Kette zeigen** färbt alle Sektoren der gewählten Datei. Ein Klick oder die Pfeiltasten wählen den Sektor.
- **Sektor** (Karte rechts): zeigt die Bytes des gewählten Sektors in Hex und als Zeichen; Zeigerbytes sind hervorgehoben. **Folgen ▶** springt zum Folgesektor („Kettenende“ am Ende), **Zurück** geht den Weg zurück.
- **Im Emulator einlegen** startet den Standard-Emulator des Systems mit dieser Diskette (Laufwerk 8 bei Commodore, D1: bei Atari). Das Image muss vorher gespeichert sein; sonst ist der Knopf gesperrt.

## Dateien ins Projekt übernehmen
Mit dem Häkchen je Datei (Spalte **Vormerken**) wird sie vorgemerkt; **Alle**, **Keine**, **Programme** und **BASIC** wählen mehrere auf einmal. Danach erscheint links das Feld **Übernahme**. **Ins Projekt übernehmen** schreibt die Dateien in den **Zielordner im Projekt**; er muss innerhalb des Projekts liegen. Optionen:
- **BASIC als Quelltext:** das tokenisierte Programm wird in lesbaren, nummerierten Text zurückübersetzt.
- **Originaldatei behalten:** zusätzlich die unveränderte Datei.
- **Auch als Assembler:** Maschinenprogramme als Quelltext. Das ist rein mechanisch, Daten erscheinen als Befehle und müssen von Hand geprüft werden.

## Formate
| Format | System | Aufbau |
|---|---|---|
| D64 | C64, 1541 | 35 Spuren, 683 Sektoren |
| D71 | C64/C128, 1571 | 70 Spuren (2 Seiten), 1366 Sektoren |
| D81 | C64/C128/MEGA65, 1581 | 80 Spuren zu 40 Sektoren, 3200 Sektoren |
| ATR | Atari, DOS 2 | 720 Sektoren zu 128 Byte |

Ein Projekt kann sein Image beim Bauen schreiben lassen: Abschnitt `disk` der Projektdatei (Format, Name, Ausgabedatei, Programme), im Dialog „Neues Projekt“ unter **Diskette** wählbar.

## Kommandozeile
`vcc disk <format> -o ausgabe --add datei[:NAME]` erzeugt ein Image (`d64`, `d71`, `d81`, `atr [--boot prog.xex]`). `vcc disk list x.d64` zeigt das Verzeichnis und prüft die BAM, `vcc disk extract x.d64 NAME -o datei` holt eine Datei zurück. Die Images sind mit `c1541` gegengeprüft.
