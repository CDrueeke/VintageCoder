# Hugo & Gumbo (MEGA65)

Selbstlaufende Jump-and-Run-Demo in Assembler, hier im **VIC-II-Modus des MEGA65**: gleiche Spiellogik (`../../common/jumprun/core.asm`),
gleiche Bilder und Sprites wie die C64-Version, Hardware-Schicht `hal_mega65.asm`.

- Anpassungen: 45GS02-Code, BASIC-65-Umfeld abschalten (MAP/EOM, `$01 = $35`, VIC-IV freischalten, 40 Spalten), VIC-Bank 1 (`$4000`–`$7FFF`),
  Interrupt-Vektor im RAM, Klangeffekte auf dem **zweiten SID** (`$D420`), sodass die Musik nie aussetzt.
- **Stand:** in Xemu (Windows, Original-ROM, `-prgmode 65`, Start mit RUN) geprüft: Titel, vier Bilder, Sprites, Himmelsbalken laufen; auf dem Gerät noch nicht probiert. Vollfarb-Zeichen (FCM) und Vollfarb-Sprites sind der nächste Ausbauschritt.
- Bauen: `vcc build .`; Start in Xemu. Die Daten stammen aus `tools/gen_jumprun_c64.py` (Profile auf `mega65.*` umgestellt).
