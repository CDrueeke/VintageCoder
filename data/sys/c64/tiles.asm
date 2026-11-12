// SPDX-License-Identifier: GPL-3.0-or-later
// Kachel-Bibliothek für den C64 (Konzept Kachel- und Screen-Editor, Abschnitt 6). Einbinden: #include <c64/tiles.asm>
// Die Befehle und Schalter stehen im Kopf von common/tiles.asm.
//
// C64: Textbildschirm 40 x 25 Zeichen, Bildschirmspeicher standardmäßig $0400, Farb-RAM $D800 (eine Farbe je Zelle,
// Bit 3 = Mehrfarbe). Anderen Bildschirmspeicher einstellen: vor dem Einbinden
//   TILE_SCREEN = $0800    (oder vcc asm -D TILE_SCREEN=$0800)
// Das Low-Byte von Bildschirm- und Farbspeicher muss gleich sein (hier immer 0).

#include <c64/hardware.asm>

.ifndef TILE_SCREEN
.const TILE_SCREEN = SCREEN_RAM
.endif
.ifndef TILE_COLORRAM
.const TILE_COLORRAM = COLOR_RAM
.endif
.const TILE_COLS = 40
.const TILE_ROWS = 25

#include <common/tiles.asm>
