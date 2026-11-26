# Troxx (VIC-20, unexpanded) - Stage 1 (Spike)

Self-contained arcade example for VCASM: a figure travels along the lines of a grid (5 x 4 rectangles); when all four edges of a rectangle have been traveled, it is filled. Enemies walk on the lines. Concept: `assets/docs/Konzept-Beispiel-VC20-Gittermosaik.md` (working title "Gittermosaik").

**Status: stages 1-7** (VIC-20 reference version finished, hand test by the owner pending, see `HANDTEST.md`; checksums in `PRUEFSUMMEN.txt`). Program 3068 bytes (hard limit 3071; tables are placed additionally in the free characters 60..63). Game rule 16.2 (concept): enemies destroy unfinished lines (a segment without a filled neighbor rectangle is reset, points are deducted; this applies when entering a line character and when entering a node, for all segments ending there; free-standing yellow nodes are reset as well (node rule); switchable with `.const ENEMY_DESTROYS_LINES`). Playable: travel the grid (segments, rectangles, filling, chain reaction), points (BCD), lives, levels, Runner and Wanderer as enemies, title screen, game over.

| File | Content |
|---|---|
| `main.asm` | Constants, memory map, start, state machine |
| `vic.asm` | Screen init, standard detection, copy character set, `wait_frame` |
| `grid.asm` | Data model, drawing, segments, rectangles, filling |
| `player.asm` | Joystick, movement, coloring, self-play |
| `enemies.asm` | Random numbers (LFSR), decision, movement, collision |
| `sound.asm` | Sound module with effect lists and priorities |
| `hud.asm` | Displays, messages, BCD points |
| `charset.asm` | 64 characters ($1C00) |
| `data.inc` | Tables (generated at assembly time) |

Build: `vcc build`. Variants via `vcc asm main.asm --target vic20 -D NAME=1`: `AUTOPLAY=1` (the computer plays itself, no title screen), additionally `AUTOGOD=1` (permanently invulnerable, for the endurance run). Test: `vcc test --backend vice` (`troxx.vctest`: static checks and routines in the emulator; `troxx_regel0.vctest`: rule switched off (program `build0/main.prg`, built with `vcc asm main.asm --target vic20 -D ENEMY_DESTROYS_LINES=0 --debug build0/main.vcdbg -o build0/main.prg`); `troxx_kurz.vctest`: very short routines that the test runner occasionally does not confirm). Run: `xvic` without memory expansion.

Controls: title screen: left/right selects the speed 1-9, fire starts; in the game fire = pause; joystick in the port, the figure only moves while a direction is held (turn at nodes).

Checked in the emulator only, not on real hardware.
