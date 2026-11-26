# Hugo & Gumbo (MEGA65)

Self-playing jump-and-run demo in assembler, here in the **VIC-II mode of the MEGA65**: same game logic (`lib/core.asm`),
same pictures and sprites as the C64 version, hardware layer `hal_mega65.asm`.

- Adaptations: 45GS02 code, switch off the BASIC 65 environment (MAP/EOM, `$01 = $35`, unlock VIC-IV, 40 columns), VIC bank 1 (`$4000`-`$7FFF`),
  interrupt vector in RAM, sound effects on the **second SID** (`$D420`), so the music never pauses.
- **Status:** checked in Xemu (Windows, original ROM, `-prgmode 65`, start with RUN): title, four levels, sprites, sky bars run; not yet tried on the real device. Full-color characters (FCM) and full-color sprites are the next step.
- Build: `vcc build .`; start in Xemu. The data comes from `tools/gen_jumprun_c64.py` (profiles switched to `mega65.*`).
