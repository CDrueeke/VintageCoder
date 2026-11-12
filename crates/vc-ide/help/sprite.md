# Sprite-/PMG-Editor

Pixelcanvas für C64-Sprites (hires 24×21, multicolor 12×21) und Atari Player/Missiles (8 Pixel breit, 4 Frames).

- **Werkzeuge:** Stift, Radierer, Füllen, Linie, Rahmen, Pipette, **Auswahl** (aufziehen, Strg+C/X/V, Entf, Pfeile verschieben).
- **Rechte Maustaste** löscht.
- **Pixelwerte** (rechts): die Tabelle zeigt je Wert die Bitfolge, die Farbe, den Namen und, wo es sie gibt, das Register (zum Beispiel `$D025` für Multicolor 1 oder `$D027+n` für die Sprite-Farbe; beim Atari `COLPM0` bis `COLPM3`). Ein Klick wählt den Wert, den der Stift setzt, die Tasten 1 bis 4 auch. Darunter ordnen die Farbfelder den Werten Palettenfarben zu.
- **Frames:** Liste links, „+ Neu“, „Duplizieren“, „Löschen“; **Animation** spielt sie ab; **Onion-Skin** zeigt den Vorgänger-Frame als Schatten.
- **Bearbeiten:** Spiegeln, Schieben, Invertieren, Löschen, Kopieren/Einfügen von Frames.
- **Export** über den Fuß des Studios, siehe [Export](help:export): 64 Bytes je Sprite-Frame; im Programm den Zeiger auf `adresse / 64` setzen (siehe `examples/c64/sprite_demo/sprite_demo.asm`).

## Hardware-Flags
Für C64 und C128 zeigt die rechte Spalte die **VIC-II Sprite-Flags** als Schalter: **MC** (Multicolor, `$D01C`), **X2** (doppelt breit, `$D01D`), **Y2** (doppelt hoch, `$D017`) und **BG** (hinter dem Hintergrund, `$D01B`). MC folgt dem Modus des gewählten Frames; ein Klick darauf wechselt ihn wie in der Seitenleiste. X2, Y2 und BG gelten für die ganze Datei und stehen als optionales Feld `hw` in der `.vcsprite` (`expandX`, `expandY`, `behind`). Sind alle aus, fehlt das Feld, und alte Dateien bleiben unverändert. Der Export hängt dann die Konstanten `<name>_expand_x`, `<name>_expand_y` und `<name>_behind` (0 oder 1) an.

Bei Atari Player/Missiles stehen stattdessen **Breite** (SIZEP0 bis SIZEP3: einfach, doppelt, vierfach) und **Priorität** (PRIOR `$D01B`: Player vor Feld oder Feld vor Player) in der rechten Spalte; sie werden als `hw` in der `.vcpmg` gespeichert, aber noch nicht in den Export übernommen.

## Hires und Multicolor mischen (C64)
Ein Sprite-Asset gehört zu einem Profil (hires 24×21 oder multicolor 12×21), aber jedes Frame darf einen eigenen **Modus** haben. In der Seitenleiste wählst du für das gewählte Frame **Hires** oder **Multicolor**; die Frame-Liste zeigt den Modus jedes Frames, und der Editor passt Rasterbreite, Pixelwerte und Farbfelder an. Beim Wechsel werden die Pixel umgerechnet (Hires → Multicolor: je zwei Pixel werden eins; Multicolor → Hires: jedes Pixel wird doppelt breit, alle Farben außer „transparent“ werden zur Sprite-Farbe). Strg+Z macht den Wechsel rückgängig. Die Multicolor-Farben 1 und 2 gelten für alle Frames des Assets (wie die Register `$D025`/`$D026`).

Der Export schreibt jedes Frame in seinem Modus und legt die Konstanten `<name>_count` und `<name>_multicolor_mask` (Bit i = Frame i, für `$D01C`) an; bei gemischten Frames außerdem die Tabelle `<name>_modes` (je Frame 0 = hires, 1 = multicolor, siehe unten). „Onion-Skin“ und „PNG laden“ gelten nur für Frames im Modus des Profils.

## Animationen
Unter dem Editor liegt das Feld **Animationen**. „+ Neu“ legt eine Folge an; mit „Frame anhängen“ fügst du das gewählte Frame als Schritt hinzu. Jeder Schritt hat eine **Dauer in Bildern** (1/50 s) und lässt sich verschieben (◀ ▶) oder entfernen (×). Der **Ablauf** ist *Schleife*, *Hin und zurück* (die Endbilder werden nicht doppelt gezeigt) oder *Einmal*. Die Vorschau links läuft in Echtzeit; ein Klick startet sie von vorn. Frames dürfen in einer Animation hires und multicolor mischen. Wird ein Frame eingefügt oder gelöscht, werden die Animationen nachgeführt.

Gibt es Animationen oder gemischte Frames, schreibt der Export neben `<name>.asm` eine zweite Datei **`<name>.anim.asm`** (Daten, die hinter die Sprite-Daten gehören: `.include "build/export/<name>.anim.asm"` direkt nach dem Include der Sprites). Sie enthält je Animation (`<name>_anim_<animation>_…`):
- `_ptr`: die Sprite-Zeiger, also die Blocknummer im VIC-Bank (`(Adresse & $3FFF) / 64 + Frame`), „Hin und zurück“ schon ausgeschrieben;
- `_delay`: die Dauer jedes Schritts in Bildern;
- `_len`: die Zahl der Schritte, `_mode`: 0 = Schleife, 1 = einmal.

Im Raster-Interrupt zählst du die Dauer herunter und schreibst beim Ablauf den nächsten Zeiger in den Sprite-Zeiger (`$07F8` + Sprite-Nummer). Die Sprite-Daten (`<name>`) müssen auf eine 64-Byte-Grenze gelegt werden, z. B. mit `.org $2000`.

## Sprites aus anderen Quellen übernehmen
**Konvertieren von …** (Werkzeugleiste) legt aus einer Datei ein neues Sprite-Asset im Projekt an. Das Ziel ist das Profil des gerade geöffneten Assets (C64 hires, C64 multicolor oder Atari-Player/Missile). Möglich sind:

- **SPD-Format (`.spd`, Versionen 1, 3 und 5):** Sprites, Hires/Multicolor je Sprite (Bit 7 des Attributbytes), Multicolor-Farben aus dem Kopf. Hat jedes Sprite im SPD-Format eine eigene Farbe, gilt die des ersten; Overlays und Animationen werden nicht übernommen.
- **Rohdaten (`.bin`, `.spr`, `.prg`, …):** Blöcke zu 64 Byte (63 Daten + 1 Füllbyte). Du gibst **Offset** (dezimal oder `$hex`), **Anzahl** (0 = alle) und den **Modus** (wie Profil, Hires, Multicolor) an; bei `.prg` werden die 2 Bytes Ladeadresse automatisch übersprungen.
- **Sprite-Datei eines anderen Systems (`*.vcsprite`, `*.vcpmg`):**
  - *C64 → Atari-PMG:* die 24 Pixel eines Hires-Sprites verteilen sich auf die Player 0–2 (je 8 Pixel), Player 3 bleibt leer. Multicolor wird zu Hires gedoppelt, die Mehrfarbigkeit geht verloren. Bei mehreren Sprites wählst du **Sprite Nr.**
  - *Atari-PMG → C64:* die Player 0–2 ergeben ein 24 Pixel breites Hires-Sprite, auf 21 Zeilen gekürzt; Player 3 und die Player-Farben entfallen.
  - *C64 ↔ C64:* Frames mit ihrem Modus.

Hinweise zu Verlusten (Farben, Größen, abgeschnittene Zeilen) erscheinen danach in der Ausgabe. Auf der Kommandozeile: `vcc media import-sprite <datei> --name NEU [--profile ZIELPROFIL] [--offset N] [--count N] [--mode hires|multicolor] [--frame N]`.

