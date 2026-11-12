# Hugo & Gumbo (C64)

Selbstlaufende Jump-and-Run-Demo in Assembler: Titelbild, vier Bilder (Wiese, Höhle, Burg, Wolken), Finale, dann wieder von vorn.
Held (Mehrfarb-Sprite + Hires-Overlay) springt in Sinuskurven, der Gegner klettert Leitern und springt nie. SID-Musik und Klangeffekte.

- Bauen: `vcc build .` oder in der IDE F7; Start: `build/jumprun.prg` (z. B. in VICE `x64sc`).
- Assets: `graphics/*` (Zeichensatz, Kachelsatz `hg.vctiles`, Screen-Bank `levels.vcscreen` mit den vier Bildern, Titelbild, Sprites) und `music/tune.vcsong` öffnen die Editoren der IDE.
- Die Dateien erzeugt `python3 tools/gen_jumprun_c64.py` (im Repository-Wurzelverzeichnis) aus den gemeinsamen Kunstdaten `tools/jumprun_art.py`; ebenso `lib/levels.inc` (Farben, Startwerte, Wege des Autopiloten; die Kacheleigenschaften „tragend“ und „Leiter“ stehen im Kachelsatz und kommen aus dem Export `hg_props`).
- Quelltext: `jumprun.asm` (Ablauf), `hal_c64.asm` (VIC-II, SID, Interrupt), `../../common/jumprun/core.asm` (Spiellogik, plattformneutral).
