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
- die Anbindung an **Emulatoren** (Programme, die einen alten Computer auf deinem Rechner nachbilden) und an **echte Hardware**,
- einen **Debugger**, mit dem du ein laufendes Programm anhalten und Schritt für Schritt verfolgen kannst,
- **Studios** zum Zeichnen von Sprites und Zeichensätzen, für Bildschirme, Kacheln, Paletten und für Musik (Tracker).

> **Info:** Ein *Quelltext* ist die Textdatei, die du schreibst. *Bauen* heißt, daraus das Programm zu erzeugen, das der alte Computer versteht. *Starten* heißt, dieses Programm in einem Emulator (oder auf echter Hardware) laufen zu lassen. Mehr Begriffe stehen im Glossar (Datei `GLOSSAR.md`).

## Für welche Computer kannst du programmieren?

Die Oberfläche kennt sechs Zielsysteme. Sie stehen im Projekt-Assistenten und in den Einstellungen:

| Zielsystem | Prozessor und Chips | Emulator (Sorte) |
|---|---|---|
| Commodore 64 (C64) | 6510, VIC-II (Grafik), SID (Klang) | VICE x64sc (taktgenau) oder x64 (schnell) |
| Commodore 128 (C128) | 8502, VIC-IIe, VDC (80 Spalten) | VICE x128 |
| C16 / Plus/4 | 7501/8501, TED | VICE xplus4 |
| VC-20 | 6502, VIC | VICE xvic |
| Atari 800 XL / XE | 6502C, ANTIC, GTIA, POKEY | Altirra (nur Windows) oder atari800 |
| MEGA65 | 45GS02, VIC-IV, 40 MHz | Xemu (xmega65) |

Auf echter Hardware kannst du ein Programm außerdem an eine **Ultimate 64 / II+** (für den C64) oder an einen **MEGA65** schicken. Wie das geht, steht in [Bauen und Starten](kapitel:bauen-starten).


## PET und CBM-II

Für den Commodore PET/CBM und den CBM-II gibt es erste Daten, Vorlagen und Beispiele. Diese beiden Systeme sind aber noch **keine** Zielsysteme der Oberfläche: Kein Eintrag im Projekt-Assistenten, kein Start mit F5. Du kannst dafür mit dem Kommandozeilenwerkzeug `vcc` bauen und das Ergebnis von Hand in VICE starten. Das Handbuch beschreibt deshalb die sechs Systeme der Tabelle.

## Installieren und einrichten

Wie du VintageCoder auf macOS und Windows installierst, beim ersten Start mit dem **Konfigurationsassistenten** einrichtest (Systeme, Emulator-Erkennung, Projektordner) und mit F5 testest, steht im Kapitel [Installation und Ersteinrichtung](kapitel:installation). Zwei Dinge vorab:

- VintageCoder startet dein Programm in einem **Emulator**, den du selbst installierst (für den Einstieg reicht VICE für den C64).
- **ROM-Dateien** (Betriebssystem, BASIC, Zeichensatz der alten Computer) liefert VintageCoder nicht mit. Für dein erstes Programm mit VICE brauchst du keine.

![Die Willkommensseite mit den Knöpfen „Neues Projekt“, „Projekt öffnen“ und „Beispiel öffnen“, der Liste der zuletzt geöffneten Projekte und dem Kasten „Toolchain und Hardware“](bild:willkommen-seite "Die Willkommensseite ist der Startpunkt, solange kein Projekt geöffnet ist")

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

Die meisten Kürzel gelten auf allen Systemen, nur eine Taste heißt anders: **Cmd** auf dem Mac entspricht **Strg** unter Windows. Im Handbuch steht deshalb zum Beispiel **Cmd/Strg+O** für „Projekt öffnen“. Die Funktionstasten F5, F6, F7 und andere sind überall gleich.

## Häufige Stolpersteine

- **Du suchst die Einrichtung von Emulator, ROMs oder Hardware.** Sie steht im Kapitel [Installation und Ersteinrichtung](kapitel:installation). Den Konfigurationsassistenten rufst du jederzeit über **Hilfe ▸ Konfigurationsassistent** wieder auf.

## Wie geht es weiter?

Im nächsten Kapitel legst du dein [erstes Projekt](kapitel:erstes-projekt) an, baust es und startest es im Emulator. Wenn du dich zuerst in der Oberfläche umsehen möchtest, lies [Die Oberfläche im Überblick](kapitel:oberflaeche).
