# Sprite-/PMG-Editor

Pixelcanvas für C64-Sprites (hires 24×21, multicolor 12×21) und Atari Player/Missiles (8 Pixel breit, 4 Frames).

- **Werkzeuge:** Stift, Radierer, Füllen, Linie, Rahmen, Pipette, **Auswahl** (aufziehen, Strg+C/X/V, Entf).
- **Rechte Maustaste** löscht.
- **Pixelwerte** (rechts): die Tabelle zeigt je Wert die Bitfolge, die Farbe, den Namen und, wo es sie gibt, das Register (zum Beispiel `$D025` für Multicolor 1 oder `$D027+n` für die Sprite-Farbe; beim Atari `COLPM0` bis `COLPM3`). Ein Klick wählt den Wert, den der Stift setzt, die Tasten 1 bis 4 auch. Darunter ordnen die Farbfelder den Werten Palettenfarben zu.
- **Frames:** Liste links, „+“, „Dupl.“, „Löschen“; der Abschnitt **Animationen** (Karte *Sprite*) spielt sie ab; **Onion Skin** in der Werkzeugleiste zeigt das Vorgänger-Frame als Schatten.
- **Transformieren ▾:** Spiegeln, Verschieben, Leeren.
- **Export** über **Exportieren …** in der Kopfzeile, siehe [Export](help:export): 64 Bytes je Sprite-Frame; im Programm den Zeiger auf `adresse / 64` setzen (siehe `examples/c64/sprite_demo/sprite_demo.asm`).

## Hardware-Flags
Für C64 und C128 zeigt die Karte *Sprite* drei Schalter: **Doppelt breit** (X2, `$D01D`), **Doppelt hoch** (Y2, `$D017`) und **Hinter dem Hintergrund** (BG, `$D01B`). Multicolor (MC, `$D01C`) folgt dem Modus des gewählten Frames und hat keinen eigenen Schalter; den Modus stellst du in der Karte *Dokument* ein. Die drei Schalter gelten für die ganze Datei und stehen als optionales Feld `hw` in der `.vcsprite` (`expandX`, `expandY`, `behind`). Sind alle aus, fehlt das Feld, und alte Dateien bleiben unverändert. Der Export hängt dann die Konstanten `<name>_expand_x`, `<name>_expand_y` und `<name>_behind` (0 oder 1) an.

Bei Atari Player/Missiles gibt es keine Schalter. **Breite** (SIZEP0 bis SIZEP3) und **Priorität** (PRIOR) stellst du im Programm ein; dafür gibt es in der Oberfläche noch keine Schalter, und der Export schreibt sie nicht.

## Hires und Multicolor mischen (C64)
Ein Sprite-Asset gehört zu einem Profil (hires 24×21 oder multicolor 12×21), aber jedes Frame darf einen eigenen **Modus** haben. In der Karte *Dokument* wählst du unter **Modus dieses Frames** (oder mit der Taste **C**) **Hires** oder **Multicolor**; die Frame-Liste zeigt den Modus jedes Frames, und der Editor passt Rasterbreite, Pixelwerte und Farbfelder an. Beim Wechsel werden die Pixel umgerechnet (Hires → Multicolor: je zwei Pixel werden eins; Multicolor → Hires: jedes Pixel wird doppelt breit, alle Farben außer „transparent“ werden zur Sprite-Farbe). Strg+Z macht den Wechsel rückgängig. Die Multicolor-Farben 1 und 2 gelten für alle Frames des Assets (wie die Register `$D025`/`$D026`).

Der Export schreibt jedes Frame in seinem Modus und legt die Konstanten `<name>_count` und `<name>_multicolor_mask` (Bit i = Frame i, für `$D01C`) an; bei gemischten Frames außerdem die Tabelle `<name>_modes` (je Frame 0 = hires, 1 = multicolor, siehe unten). „Onion Skin“ und „PNG laden“ gelten nur für Frames im Modus des Profils.

## Animationen
In der Karte *Sprite* liegt der Abschnitt **Animationen**. „+ Neu“ legt eine Folge an; mit „Frame anhängen“ fügst du das gewählte Frame als Schritt hinzu. Jeder Schritt hat eine **Dauer in Bildern** (1/50 s) und lässt sich verschieben (◀ ▶) oder entfernen (×). Der **Ablauf** ist *Schleife*, *Hin und zurück* (die Endbilder werden nicht doppelt gezeigt) oder *Einmal*. Die Vorschau dort läuft in Echtzeit; ein Klick auf den Namen der Animation startet sie von vorn. Frames dürfen in einer Animation hires und multicolor mischen. Wird ein Frame eingefügt oder gelöscht, werden die Animationen nachgeführt.

Gibt es Animationen oder gemischte Frames, schreibt der Export neben `<name>.asm` eine zweite Datei **`<name>.anim.asm`** (Daten, die hinter die Sprite-Daten gehören: `.include "build/export/<name>.anim.asm"` direkt nach dem Include der Sprites). Sie enthält je Animation (`<name>_anim_<animation>_…`):
- `_ptr`: die Sprite-Zeiger, also die Blocknummer im VIC-Bank (`(Adresse & $3FFF) / 64 + Frame`), „Hin und zurück“ schon ausgeschrieben;
- `_delay`: die Dauer jedes Schritts in Bildern;
- `_len`: die Zahl der Schritte, `_mode`: 0 = Schleife, 1 = einmal.

Im Raster-Interrupt zählst du die Dauer herunter und schreibst beim Ablauf den nächsten Zeiger in den Sprite-Zeiger (`$07F8` + Sprite-Nummer). Die Sprite-Daten (`<name>`) müssen auf eine 64-Byte-Grenze gelegt werden, z. B. mit `.org $2000`.

## Sprites aus anderen Quellen übernehmen
**PNG laden …** (Kopfzeile) wandelt ein Bild in Pixelwerte um. Das Übernehmen aus SPD-Dateien, Rohdaten und Sprites anderer Systeme ist in dieser Version noch nicht in der Oberfläche enthalten.

