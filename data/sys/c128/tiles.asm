// SPDX-License-Identifier: GPL-3.0-or-later
// Kachel-Bibliothek für den C128, 40-Spalten-Bildschirm des VIC-IIe (Konzept Kachel- und Screen-Editor, Abschnitt 6).
// Einbinden: #include <c128/tiles.asm>. Die Befehle und Schalter stehen im Kopf von common/tiles.asm.
//
// C128 im 128-Modus mit dem VIC-IIe: Textbildschirm 40 x 25 Zeichen, Bildschirmspeicher standardmäßig $0400, Farb-RAM $D800
// (wie C64: eine Farbe je Zelle, Bit 3 = Mehrfarbe). Der VDC (80 Spalten) wird NICHT unterstützt: sein Bildschirm liegt im
// eigenen Speicher und ist nur über Registerzugriffe erreichbar. Voraussetzung: Ein-/Ausgabebereich und RAM-Bank 0 sind
// sichtbar (MMU-Grundzustand nach dem Start, $D500 = $00), sonst erreicht $D800 das falsche Ziel.
// Zeropage: Standard TILE_ZP = $F7 (zu prüfen: Belegung durch den C128-KERNAL; das Beispiel läuft bei gesetztem I-Flag ohne KERNAL).

#include <c128/hardware.asm>

.ifndef TILE_SCREEN
.const TILE_SCREEN = SCREEN_RAM
.endif
.ifndef TILE_COLORRAM
.const TILE_COLORRAM = COLOR_RAM
.endif
.const TILE_COLS = 40
.const TILE_ROWS = 25

#include <common/tiles.asm>
