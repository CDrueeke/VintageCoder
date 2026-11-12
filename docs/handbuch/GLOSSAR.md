# Glossar für Einsteiger

Kurze Erklärungen der Begriffe, die im Handbuch vorkommen. Alphabetisch, in einfachen Worten.

## A

**Adresse** – Jede der 65536 Speicherzellen eines 8-Bit-Computers hat eine Nummer, die Adresse. Man schreibt sie meist hexadezimal, zum Beispiel `$d020`.

**Akkumulator (A)** – das wichtigste Arbeitsregister des 6502-Prozessors. Fast alle Rechnungen und Kopiervorgänge laufen über A.

**Assembler** – ein Programm, das lesbare Kürzel wie `lda #2` in Maschinencode übersetzt. Auch die Sprache selbst heißt so. Der Assembler von VintageCoder heißt **VCASM** (`vcasm`).

**Asset** – Alles, was nicht Programmtext ist: Sprite, Zeichensatz, Bildschirm, Kachelsatz, Palette, Song. Liegt als kleine Textdatei im Projekt. → [Grafik und Klang im Überblick](kapitel:grafik-ueberblick)

**ATASCII** – der Zeichencode der Atari-8-Bit-Computer, Gegenstück zu PETSCII beim Commodore.

## B

**BAM / VTOC** – Verwaltungstabelle einer Diskette, die merkt, welche Sektoren belegt sind (Commodore: BAM, Atari: VTOC).

**BASIC-Stub** – eine winzige BASIC-Zeile wie `10 SYS 2061` am Anfang eines Maschinenprogramms. Sie ruft das Maschinenprogramm auf, damit man es mit `RUN` starten kann. Im Assembler erzeugt sie die Direktive `.basicstub`.

**Bauen** – aus dem Quelltext das fertige Programm für das Zielsystem erzeugen (Taste F7). Der Fachbegriff „kompilieren“ oder „assemblieren“ meint dasselbe.

**Bildschirmcode** – die Zahlen, die der Bildschirmspeicher von C64 und Verwandten erwartet. Sie unterscheiden sich von PETSCII. Im Assembler wandelt `.screen` Text in Bildschirmcodes um.

**Byte** – ein Speicherplatz für eine Zahl von 0 bis 255 (acht Bit).

## D

**Debugger** – ein Werkzeug, mit dem man ein laufendes Programm anhält, Schritt für Schritt verfolgt und seine Register und seinen Speicher ansieht.

**Dialekt** (BASIC) – die Ausprägung des BASIC, die ein Computer hat: BASIC V2 (C64, VC-20), BASIC 7.0 (C128), BASIC 3.5 (Plus/4), BASIC 65 (MEGA65), Atari BASIC oder Turbo BASIC XL.

**Direktive** – eine Anweisung an den Assembler (oder den BASIC-Übersetzer), keine an den Prozessor. Im Assembler beginnt sie mit einem Punkt (`.org`), im BASIC mit `#` (`#include`).

**Diskettenabbild (D64, D71, D81, ATR)** – Datei, die den Inhalt einer Diskette Sektor für Sektor enthält. → [Disk-Studio, Segmente, ROMs](kapitel:disk-studio)

## E

**Emulator** – ein Programm, das einen alten Computer auf dem heutigen Rechner nachbildet. VintageCoder startet dein Programm in einem Emulator wie VICE (Commodore), atari800 oder Altirra (Atari) und Xemu (MEGA65).

## F

**Frame** – Ein Einzelbild eines Sprites; mehrere Frames hintereinander ergeben eine Animation.

## H

**Hexadezimal** – eine Schreibweise von Zahlen zur Basis 16 mit den Ziffern 0 bis 9 und a bis f. In Quelltexten steht ein `$` davor: `$ff` ist 255. Binär schreibt man mit `%`: `%1010` ist 10.

**Hires / Multicolor** – Zwei Zeichenarten bei Zeichensätzen und Sprites: *Hires* hat feine Punkte mit einer Farbe, *Multicolor* hat doppelt breite Punkte, aber vier Farbwerte je Punkt. → [Zeichensatz-Editor](kapitel:zeichensatz), [Sprite- und PMG-Editor](kapitel:sprites)

## I

**IDE** – *Integrated Development Environment*, Entwicklungsumgebung: Editor, Übersetzer, Debugger und weitere Werkzeuge in einem Programm. VintageCoder ist eine IDE.

**Include** – das Einbinden einer anderen Datei in den Quelltext. Im Assembler mit `.include "datei.asm"` oder `#include <pfad>` für die Systembibliothek.

**Instrument / Hüllkurve (ADSR)** – Klangfarbe eines Tons; die Hüllkurve beschreibt den Lautstärkeverlauf: Attack (Anstieg), Decay (Abfall), Sustain (Haltepegel), Release (Ausklingen).

## K

**Kachel (Tile) / Kachelsatz / Karte** – Eine Kachel ist ein Baustein aus mehreren Zeichen (z. B. 2×2). Ein Kachelsatz sammelt Kacheln, eine Karte (Screen-Bank) sagt, welche Kachel wo steht. → [Bildschirm- und Kachel-Editor](kapitel:bildschirm-kacheln)

**KERNAL** – das Betriebssystem der Commodore-Computer, fest im ROM. Es bietet Routinen wie `CHROUT` (ein Zeichen ausgeben), die man mit `jsr` aufruft.

**Kommentar** – Text im Quelltext, den der Übersetzer überliest. Im Assembler beginnt er mit `;` (oder `//`), im BASIC am Zeilenanfang mit `;`.

**Konfigurationsassistent** – der geführte Dialog zur Einrichtung von VintageCoder (Systeme, Emulatoren, ROMs, Hardware, Design, Projektordner). Er öffnet sich beim ersten Start von selbst, zeigt vor dem Schreiben eine Änderungsliste und lässt sich über das Menü **Hilfe** wieder aufrufen. → [Installation und Ersteinrichtung](kapitel:installation)

## L

**Label** – ein Name für eine Adresse im Assembler, mit Doppelpunkt am Ende (`start:`). Lokale Labels beginnen mit einem Punkt (`.loop:`).

**Listing** – die Datei `.lst`, die zu jeder Zeile des Quelltexts die Adresse und die erzeugten Bytes zeigt.

## M

**Makro** – eine benannte Abkürzung für mehrere Zeilen. Der Assembler setzt bei jedem Aufruf den Text des Makros ein. Ein Makro ist kein Unterprogramm.

**Marke** (BASIC) – ein Name für eine Zeile, den man statt einer Zeilennummer benutzt: `@schleife:` und `goto @schleife`.

**Maschinencode** – die Zahlen, die der Prozessor ausführt. Ein Assembler erzeugt sie aus deinem Quelltext.

**Modul (Cartridge) / Bank** – Steckmodul mit ROM, das beim Einschalten startet; Bänke sind umschaltbare Teile des Moduls, die an derselben Adresse erscheinen.

**Muster (Pattern) / Reihenfolge (Order)** – Ein Muster ist eine Tabelle mit fester Zeilenzahl; die Reihenfolge bestimmt, in welcher Abfolge die Muster gespielt werden.

## O

**Onion-Skin** – Anzeige des vorigen Frames als blasser Schatten unter dem aktuellen, damit man Bewegungen sauber zeichnet.

**Opcode** – die Zahl, die einen Prozessorbefehl bezeichnet. Das Listing zeigt sie vor dem Operanden: bei `a9 02` ist `a9` der Opcode von `lda #`.

## P

**Palette** – Tabelle, die Farbnummern Farben zuordnet. Der MEGA65 hat 256 frei änderbare Einträge. → [Paletten](kapitel:paletten)

**PETSCII** – der Zeichencode der Commodore-Computer (Buchstaben, Grafikzeichen, Steuerzeichen wie „Bildschirm löschen“).

**PLAY** – BASIC-65-Befehl des MEGA65, der eine Zeichenkette aus Noten abspielt. → [PLAY-Tracker](kapitel:play-tracker)

**PMG (Player/Missile-Grafik)** – Die Sprites des Atari: vier Player (8 Punkte breit) und vier Missiles. → [Sprite- und PMG-Editor](kapitel:sprites)

**Projekt** – ein Ordner mit einer Projektdatei (`.vcprj`), deinen Quelltexten und den gebauten Programmen.

## Q

**Quelltext** – die Textdatei, die du schreibst (`.asm` für Assembler, `.bas` für BASIC).

## R

**Raster-Interrupt** – Unterbrechung, die der Grafikchip in einer bestimmten Bildschirmzeile auslöst; ideal, um eine Spielroutine genau einmal je Bild aufzurufen.

**Register** – eine winzige Speicherzelle im Prozessor (A, X, Y) oder in einem Chip (zum Beispiel das Register der Rahmenfarbe bei `$d020`).

**ROM** – fest eingebauter Speicher eines Computers mit Betriebssystem, BASIC und Zeichensatz. VintageCoder liefert keine Original-ROMs mit; du legst deine eigenen Abbilder bereit.

## S

**Segment** – Zusammenhängender Block erzeugter Bytes an fester Adresse; `.segment` gibt ihm Namen und erlaubten Bereich (Fehler E301 bei Verletzung).

**SID / POKEY** – Klangchips des C64 (3 Stimmen) bzw. Atari (4 Stimmen).

**Sprite-Zeiger** – Ein Byte (beim C64 `$07F8`+n), das dem Grafikchip sagt, in welchem 64-Byte-Block die Daten eines Sprites liegen (Adresse geteilt durch 64).

**Startprogramm** – das Programm eines Projekts, das F5 startet. Im Projektbaum trägt es einen Stern (★).

**Steuerzeichen** – ein Zeichen, das etwas bewirkt statt etwas anzuzeigen, zum Beispiel „Bildschirm löschen“. In Quelltexten schreibt man sie in geschweiften Klammern: `{clr}`.

**Studio** – ein Editor in VintageCoder für Grafik oder Klang: Sprites, Zeichensätze, Bildschirme, Kacheln, Paletten und Tracker.

## T

**Target** – siehe *Zielsystem*. In der Projektdatei und im Quelltext (`.target c64`) steht die Kennung des Zielsystems.

**Tracker** – Musikprogramm, in dem Noten in Tabellen stehen: Spalten sind Stimmen, Zeilen sind Zeitschritte. → [Der Tracker](kapitel:tracker)

## V

**Videonorm (PAL/NTSC)** – Fernsehnorm mit 50 bzw. 60 Bildern pro Sekunde; bestimmt Farbaussehen (Atari) und Tempo von Musik.

## Z

**Zeropage** – die ersten 256 Bytes des Speichers (Adressen `$00` bis `$ff`). Der Prozessor erreicht sie mit kürzeren und schnelleren Befehlen.

**Zielsystem** – der alte Computer, für den du programmierst: C64, C128, C16/Plus/4, VC-20, Atari 800 XL/XE oder MEGA65.

**Zielzeichensatz** – Der Zeichensatz, der beim Export aus allen benutzten Zeichen aller Kacheln entsteht.
