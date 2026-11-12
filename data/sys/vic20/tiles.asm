// SPDX-License-Identifier: GPL-3.0-or-later
// Kachel-Bibliothek für den VC-20 (Konzept Kachel- und Screen-Editor, Abschnitt 6). Einbinden: #include <vic20/tiles.asm>
// Die Befehle und Schalter stehen im Kopf von common/tiles.asm.
//
// VC-20 (VIC-I): Textbildschirm 22 Spalten x 23 Zeilen, ein Zeichen je Zelle, Farb-RAM mit 4 Bit je Zelle (Bit 3 = Mehrfarbe).
// Die Adressen kommen aus <vic20/hardware.asm> und hängen vom Speicherausbau ab:
//   Grundgerät und +3 KB:  Bildschirm $1E00, Farb-RAM $9600
//   ab +8 KB:              Bildschirm $1000, Farb-RAM $9400
// Das Projekt wählt den Ausbau (vcprj "memory"; der Assembler setzt dann MEM_SCREEN_RAM, MEM_COLOR_RAM). Beides lässt sich
// vor dem Einbinden überschreiben: TILE_SCREEN, TILE_COLORRAM. Das Low-Byte der Differenz muss 0 sein (beide Paare erfüllen das).
// Bildschirmspeicher und Zeichenzeiger des VIC-I (VIC_MEMPTR $9005, Bit 7 von VIC_COLS $9002) stellt das Programm ein; die Bibliothek
// schreibt nur Speicher. Bei Kacheln der Breite 3 passen 7 Kacheln in eine Zeile (21 von 22 Spalten); 23 Zeilen: Kachelhöhe 2 gibt 11.
// Die Zuordnung der Adressen zum Speicherausbau steht in hardware.asm (zu prüfen am Handbuch, hier nicht neu bewertet).

#include <vic20/hardware.asm>

.ifndef TILE_SCREEN
.const TILE_SCREEN = SCREEN_RAM
.endif
.ifndef TILE_COLORRAM
.const TILE_COLORRAM = COLOR_RAM
.endif
.const TILE_COLS = 22
.const TILE_ROWS = 23

#include <common/tiles.asm>
