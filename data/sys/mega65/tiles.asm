// SPDX-License-Identifier: GPL-3.0-or-later
// Kachel-Bibliothek für den MEGA65 im VIC-II-Textmodus (Konzept Kachel- und Screen-Editor, Abschnitt 6).
// Einbinden: #include <mega65/tiles.asm>. Die Befehle und Schalter stehen im Kopf von common/tiles.asm.
//
// MEGA65 mit dem VIC-II-Verhalten des VIC-IV (40 x 25 Zeichen, ein Byte je Zelle, Farb-RAM $D800, Bit 3 = Mehrfarbe,
// nur Farben 0-7 bei Mehrfarbe wie im C64): Dieselbe Rechenart wie der C64. Der Bildschirm liegt standardmäßig bei $0800
// (SCREEN_RAM_C65; beim C64 $0400), wie ihn BASIC 65 benutzt; das Programm stellt ihn über $D018 ein und setzt dafür
// VIC-IV frei und löscht H640 ($D031) wie examples/mega65/jumprun/hal_mega65.asm. Soll der Bildschirm woanders liegen
// (jumprun: $4400 in VIC-Bank 1), vor dem Einbinden TILE_SCREEN setzen. Das Low-Byte von Bildschirm- und Farbspeicher
// muss gleich sein. Der VIC-IV-Zeichenmodus mit 16 Bit je Zelle und der 80-Spalten-Modus werden nicht unterstützt.
// Zeropage: Standard TILE_ZP = $F7 (zu prüfen: der 45GS02 nutzt die Basisseite über B; das Beispiel setzt B nicht neu).

// Zyklenbudgets der Makros: der 45GS02 braucht für lda absolut 5 Takte (Tabelle des Assemblers, 40 MHz)
.const TILE_CYC_LDA = 5

#include <mega65/hardware.asm>

.ifndef TILE_SCREEN
.const TILE_SCREEN = SCREEN_RAM_C65
.endif
.ifndef TILE_COLORRAM
.const TILE_COLORRAM = COLOR_RAM
.endif
.const TILE_COLS = 40
.const TILE_ROWS = 25

#include <common/tiles.asm>
