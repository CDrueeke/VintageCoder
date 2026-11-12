# Hugo & Gumbo (Plus/4)

Selbstlaufende Jump-and-Run-Demo in Assembler: Titelbild, vier Bilder mit Autopilot, Finale, von vorn.
Gleiche Spiellogik wie die C64- und Atari-Version (`../../common/jumprun/core.asm`), eigene Hardware-Schicht `hal_plus4.asm`:

- TED-Mehrfarb-Text mit eigenem Zeichensatz (256 Zeichen im RAM); Mehrfarbe gilt je Zelle über Bit 3 des Farb-RAM-Werts
- Himmelsbalken per Rasterinterrupt (Hintergrundfarbe `$FF15`)
- **Zeichen-Sprites**: Der TED hat keine Sprites. Held (3 × 4 Zellen) und Gegner (3 × 3) werden in jedem Bild aus vorverschobenen Bildern
  in umdefinierte Zeichen (`$E0`–`$F4`) gerechnet und mit dem Hintergrund gemischt
- Klang: zwei TED-Stimmen (Melodie, Bass); Effekte leihen sich die Bass-Stimme

Bauen: `vcc build .` (Ziel plus4, Start in VICE `xplus4`). Die Daten erzeugt `tools/gen_jumprun_plus4.py`.
Das Betriebssystem wird abgeschaltet (ROM aus), der Interrupt-Vektor liegt im RAM.
