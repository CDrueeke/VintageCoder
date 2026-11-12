// SPDX-License-Identifier: GPL-3.0-or-later
// Kachel-Bibliothek für den Atari 8 Bit (Konzept Kachel- und Screen-Editor, Abschnitt 6). Einbinden: #include <atari8bit/tiles.asm>
// Die Befehle und Schalter stehen im Kopf von common/tiles.asm.
//
// Atari: Textbildschirm 40 x 24 Zeichen (ANTIC-Modus 2 oder Mehrfarbmodus 4), ein Byte je Zelle, KEIN Farbspeicher.
// Deshalb gilt TILE_NOCOLOR: die Makros und Unterprogramme schreiben nur Zeichen; die Spalte "col" des Exports und
// tile_draw_color entfallen (tile_draw_color zeichnet dann nur das Zeichen). Im Modus 4 ist die Farbwahl Teil des Zeichencodes
// (Bit 7 = COLPF3 statt COLPF2); der Kachel-Editor schreibt es über das Zellflag "sel" bereits in die Zeichentabellen.
//
// Bildschirmspeicher: Standard $4000 wie examples/atari/jumprun. Einstellen vor dem Einbinden:
//   TILE_SCREEN = $4000    (oder vcc asm -D TILE_SCREEN=$4000)
// ANTIC zählt die Bildschirmadresse nur innerhalb eines 4-KB-Blocks weiter (ein Sprung über $xFFF zurück nach $x000 ist
// nur mit einem zweiten LMS-Befehl in der Display-Liste möglich), darum muss der Bildschirm (960 Byte) in einem 4-KB-Block liegen;
// das prüft diese Datei. Die Display-Liste liefert das Programm (siehe examples/atari/tiles_demo).
// Zeropage: Standard TILE_ZP = $F7..$FE; die Gleitkommaroutinen des OS ($D4-$FF) werden hier nicht benutzt (zu prüfen, falls ein
// Programm sie aufruft: dann TILE_ZP ändern, z. B. $80, wenn BASIC aus ist).

.ifndef TILE_SCREEN
.const TILE_SCREEN = $4000
.endif
.const TILE_NOCOLOR = 1
.const TILE_COLS = 40
.const TILE_ROWS = 24
.assert (TILE_SCREEN & $0fff) + TILE_COLS * TILE_ROWS <= $1000, "Atari: Bildschirmspeicher liegt nicht in einem 4-KB-Block (ANTIC zählt nur im Block weiter)"

#include <common/tiles.asm>
