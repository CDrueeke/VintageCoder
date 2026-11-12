// SPDX-License-Identifier: GPL-3.0-or-later
// Kachel-Bibliothek für den Plus/4 und C16 (Konzept Kachel- und Screen-Editor, Abschnitt 6). Einbinden: #include <plus4/tiles.asm>
// Die Befehle und Schalter stehen im Kopf von common/tiles.asm.
//
// Plus/4 (TED): Textbildschirm 40 x 25 Zeichen, Bildschirmspeicher standardmäßig $0C00, Farb-RAM (Attribut-Byte je Zelle) bei
// $0800, also 1024 Byte VOR dem Bildschirm (laut examples/plus4/jumprun/hal_plus4.asm: COLORRAM - SCREEN = -$0400). Die Bibliothek
// rechnet den Zeiger auf die Farbzelle mit einem Versatz von Seiten (High-Byte), der Abstand darf also negativ sein.
// Das Attribut-Byte ist der volle TED-Farbwert: Bits 0-3 Farbton, Bits 4-6 Helligkeit, Bit 7 Blinken; die Spalte "col" des
// Exports gehört deshalb als fertiges Attribut-Byte in die Tabelle (z. B. $71 = Weiß bei Helligkeit 7). Das Mehrfarb-Bit
// (Bit 3 des Farbtons) hat dort keine Sonderrolle in der Bibliothek; sie kopiert das Byte.
// Anderen Bildschirmspeicher einstellen: vor dem Einbinden TILE_SCREEN = $1000 (und TILE_COLORRAM = TILE_SCREEN - $0400).
// Das Low-Byte von Bildschirm- und Farbspeicher muss gleich sein (hier immer 0).
// Zeropage: Standard TILE_ZP = $F7 (zu prüfen: Belegung durch KERNAL und BASIC; das Beispiel läuft bei gesetztem I-Flag ohne KERNAL).

#include <plus4/hardware.asm>

.ifndef TILE_SCREEN
.const TILE_SCREEN = SCREEN_RAM
.endif
.ifndef TILE_COLORRAM
.const TILE_COLORRAM = TILE_SCREEN - $0400
.endif
.const TILE_COLS = 40
.const TILE_ROWS = 25

#include <common/tiles.asm>
