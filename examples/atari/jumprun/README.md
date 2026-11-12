# Hugo & Gumbo (Atari 8 Bit)

Selbstlaufende Jump-and-Run-Demo in Assembler: Titelbild, vier Bilder mit Autopilot, Finale, von vorn.
Gleiche Spiellogik wie die C64-Version (`../../common/jumprun/core.asm`), eigene Hardware-Schicht `hal_atari.asm`:

- ANTIC-Modus 4 (vier Farben) mit eigenem Zeichensatz, Himmelsbalken per Display-List-Interrupt
- Held aus vier Spielern (Umriss, Haut, Hemd, Hose), Gegner aus den Missiles (Player-Missile-Grafik)
- Musik im RMT-Format (POKEY, Kanal 1-3), Klangeffekte auf Kanal 4

Bauen: `vcc build .` (Ziel atari8, Start in atari800 oder Altirra). Die Daten erzeugt `tools/gen_jumprun_atari.py`.
