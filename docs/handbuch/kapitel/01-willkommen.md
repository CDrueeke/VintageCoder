---
id: willkommen
titel: Willkommen bei VintageCoder
reihenfolge: 1
stufe: Einsteiger
zeit: 10 Minuten
siehe_auch: [installation, erstes-projekt, oberflaeche]
referenz: [start, settings, roms, linux]
---

# Willkommen bei VintageCoder

**Ziel:** Du weißt, was VintageCoder ist und für welche Computer du damit Programme schreiben kannst. Danach weißt du, wie es weitergeht: erst installieren und einrichten, dann das erste Programm bauen.

## Was ist VintageCoder?

VintageCoder ist eine **Entwicklungsumgebung** (englisch *IDE*) für Computer aus den 1980er-Jahren, deren Herz ein Prozessor der **6502-Familie** ist: der Commodore 64, der Atari 800 XL und ihre Verwandten. In einem einzigen Programm bekommst du:

- einen **Editor** mit Farbhervorhebung, Vervollständigung und Hilfe für Quelltexte,
- einen eigenen **Assembler** (`vcasm`) für Maschinenprogramme und einen **BASIC-Übersetzer**, der BASIC ohne Zeilennummern in ein echtes Programm des Zielsystems umsetzt,
- die Anbindung an **Emulatoren** (Programme, die einen alten Computer auf deinem Rechner nachbilden),
- **Editoren** zum Zeichnen von Sprites, Zeichensätzen, Kacheln und Bildschirmen, für Paletten, für Musik (Tracker) und für Disketten-Abbilder (Disk-Editor), dazu eine Hex-Ansicht und die Speicherkarte,
- einen **Debugger**, der dein Programm im Emulator anhält und Schritt für Schritt verfolgt.

> **Info:** Die Editoren findest du in den Hauptbereichen **Grafik**, **Musik** und **Daten**, den Debugger im Bereich **Debug**.

> **Info:** Ein *Quelltext* ist die Textdatei, die du schreibst. *Bauen* heißt, daraus das Programm zu erzeugen, das der alte Computer versteht. *Starten* heißt, dieses Programm in einem Emulator (oder auf echter Hardware) laufen zu lassen. Mehr Begriffe stehen im Glossar (Datei `GLOSSAR.md`).

## Für welche Computer kannst du programmieren?

Die Oberfläche kennt sechs Zielsysteme. Du wählst sie im Dialog **Neues Projekt erstellen**:

| Zielsystem | Prozessor und Chips | Emulator (Sorte) |
|---|---|---|
| Commodore 64 (C64) | 6510, VIC-II (Grafik), SID (Klang) | VICE x64sc (taktgenau) oder x64 (schnell) |
| Commodore 128 (C128) | 8502, VIC-IIe, VDC (80 Spalten) | VICE x128 |
| C16 / Plus/4 | 7501/8501, TED | VICE xplus4 |
| VC-20 | 6502, VIC | VICE xvic |
| Atari 800 XL / XE | 6502C, ANTIC, GTIA, POKEY | Altirra (nur Windows) oder atari800 |
| MEGA65 | 45GS02, VIC-IV, 40 MHz | Xemu (xmega65) |

Wie du ein Programm baust und im Emulator startest, steht in [Bauen und Starten](kapitel:bauen-starten). Echte Hardware (Ultimate 64 / II+, MEGA65) trägst du in den Einstellungen unter **Ziele und Hardware** ein; je nach Paket ist die MEGA65-Hardware dort ausgeblendet.

## PET

Der Commodore PET/CBM ist in dieser Version **nicht enthalten**: kein Eintrag im Dialog „Neues Projekt erstellen“, kein Start mit F5, keine Beispiele in den Paketen. Das Handbuch beschreibt deshalb die Systeme der Tabelle.

## Installieren und einrichten

Wie du VintageCoder installierst, mit dem **Konfigurationsassistenten** einrichtest, die Emulator-Erkennung prüfst und mit F5 testest, steht im Kapitel [Installation und Ersteinrichtung](kapitel:installation). Zwei Dinge vorab:

- VintageCoder startet dein Programm in einem **Emulator**, den du selbst installierst (für den Einstieg reicht VICE für den C64).
- **ROM-Dateien** (Betriebssystem, BASIC, Zeichensatz der alten Computer) liefert VintageCoder nicht mit. Für dein erstes Programm mit VICE brauchst du keine.

Die **Startseite** erscheint im Bereich **Code**, solange kein Projekt offen ist. Sie hat die Karten **Neues Projekt**, **Projekt öffnen** und **Beispiel öffnen**, den Link **Projektdatei (.vcprj) wählen …**, die Liste **Zuletzt geöffnete Projekte**, die **Beispielprojekte** und unten den Kasten **Toolchain und Hardware** mit den **erkannten Emulatoren** (bereit oder nicht gefunden) sowie die **Schnellreferenz** der wichtigsten Tastenkürzel. Beim ersten Start steht an der Stelle des Kastens die Karte **Willkommen bei VintageCoder** mit dem Knopf **Einrichtung starten**; der **Konfigurationsassistent** öffnet sich beim allerersten Start außerdem von selbst.

![Die Startseite ohne offenes Projekt: der Titel VintageCoder, die Karten Neues Projekt, Projekt öffnen und Beispiel öffnen, der Link Projektdatei (.vcprj) wählen, der Abschnitt Zuletzt geöffnete Projekte (noch leer) und die Beispielprojekte mit Filtern je System und Karten mit Öffnen und Kopieren.](bild:willkommen-seite "Die Startseite erscheint, solange kein Projekt offen ist.")

## Die wichtigsten Begriffe

| Begriff | Bedeutung |
|---|---|
| Zielsystem | der alte Computer, für den du programmierst (C64, Atari, …) |
| Projekt | ein Ordner mit einer Projektdatei (`.vcprj`), deinen Quelltexten und den gebauten Programmen |
| Quelltext | die Textdatei, die du schreibst (`.asm` für Assembler, `.bas` für BASIC) |
| Bauen | aus Quelltext ein Programm für das Zielsystem erzeugen (Taste F7) |
| Starten | bauen und im Emulator oder auf Hardware ausführen (Taste F5) |
| Emulator | ein Programm, das einen alten Computer nachbildet |
| ROM | Betriebssystem, BASIC oder Zeichensatz des alten Computers als Datei |

Mehr Begriffe erklärt das Glossar (Datei `GLOSSAR.md`).

## Tastenkürzel in diesem Handbuch

Die meisten Kürzel gelten auf allen Systemen, nur eine Taste heißt anders: **Cmd** auf dem Mac entspricht **Strg** unter Windows. Im Handbuch steht deshalb zum Beispiel **Cmd/Strg+Umschalt+O** für „Projekt öffnen“. Die Funktionstasten F5 und F7 sind überall gleich.

## Häufige Stolpersteine

- **Du suchst die Einrichtung von Emulator, ROMs oder Hardware.** Sie steht im Kapitel [Installation und Ersteinrichtung](kapitel:installation). Emulator, ROMs und Hardware stellst du in den **Einstellungen** ein (Zahnrad ⚙ in der Kopfzeile oder **Cmd/Strg+Komma**) oder mit dem **Konfigurationsassistenten** (Menü **Hilfe**). Ob VintageCoder deinen Emulator gefunden hat, siehst du auf der Startseite im Kasten **Toolchain und Hardware**.

## Wie geht es weiter?

Im nächsten Kapitel legst du dein [erstes Projekt](kapitel:erstes-projekt) an, baust es und startest es im Emulator. Wenn du dich zuerst in der Oberfläche umsehen möchtest, lies [Die Oberfläche im Überblick](kapitel:oberflaeche).
