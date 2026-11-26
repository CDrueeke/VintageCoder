# Sprite-/PMG-Editor

Zeichenfläche für C64-Sprites (hires 24×21, multicolor 12×21) und Atari Player/Missiles (8 Pixel breit, 4 Frames). Du findest den Editor im Hauptbereich **Grafik** in der zweiten Reihe unter **Sprites**; ein Doppelklick auf eine `.vcsprite`- oder `.vcpmg`-Datei im Projektbaum öffnet sie ebenfalls dort.

## Aufbau
| Bereich | Was er zeigt |
|---|---|
| Links | die **Sprite-Dateien** des Projekts, **Sprite-Datei öffnen …**, **Neue Sprite-Datei …** und die Reiter **Bilder** und **Animationen** |
| Mitte | die Werkzeugleiste, die **Zeichenfläche** mit dem gewählten Bild, darunter die Zeitleiste der Animation und die Fußzeile |
| Rechts | die Karten **Farben**, **Palette**, bei Atari **Player/Missile**, **Sprite**, **Vorschau im Spiel** und **Verschieben** |

![Der Reiter Grafik mit der Seite „Sprites“ und der Datei hero.vcsprite: links die Sprite-Dateien und zwei Bilder, in der Mitte das 12x21 Pixel große Multicolor-Sprite einer Figur in Gelb, Rotbraun und Blau, rechts die Karten „Farben“, „Sprite“ (Mehrfarbig an), „Vorschau im Spiel“ und „Verschieben“. Unter der Zeichenfläche steht der Hinweis „Keine Animation“.](bild:sprite-c64 "Der Sprite-Editor mit einem Multicolor-Sprite.")

- **Werkzeuge:** **Stift** (D), **Radierer** (E), **Pipette** (I), **Füllen** (F), **Linie** (L), **Rechteck** (R) und **Auswahl** (S: aufziehen, Cmd/Strg+C/X/V, Entf). Dazu **Waagerecht spiegeln** (X), **Senkrecht spiegeln** (Y), Rückgängig, Wiederholen, **Onion Skin** (O), **Raster** (G) und der Zoom (Verkleinern, Einpassen, Vergrößern).
- **Rechte Maustaste** löscht.
- **Farben:** Die Karte zeigt je Wert (zum Beispiel durchsichtig, Sprite-Farbe, Multicolor 1 und 2; beim Atari je Spieler die Farbe) die Farbe und, wo es sie gibt, das Register (zum Beispiel `$D025` für Multicolor 1 oder `$D027+n` für die Sprite-Farbe; beim Atari `COLPM0` bis `COLPM3`). Ein Klick wählt den Wert, den der Stift setzt, die Tasten 1 bis 4 auch. Darunter ordnet das Farbraster dem Wert eine Palettenfarbe zu. Beim Atari stellst du die **Palettenansicht** zwischen **Projekt**, **PAL** und **NTSC** um.
- **Frames (Bilder):** Reiter **Bilder** links mit **Neu**, **Duplizieren** und **Löschen**; **Onion Skin** zeigt das vorige Bild als Schatten.
- **Verschieben:** vier Pfeilknöpfe schieben das Bild um einen Punkt; **Bild leeren** löscht alle Punkte.
- **Export:** **Exportieren …** (Werkzeugleiste, Cmd/Strg+Umschalt+E), siehe [Export](help:export): 64 Bytes je Sprite-Frame; im Programm den Zeiger auf `adresse / 64` setzen (siehe `examples/c64/sprite_demo/sprite_demo.asm`).
- Alle Werkzeuge, Zoom, Raster und Onion Skin erreichst du auch über die Befehlspalette (Cmd/Strg+K). Die Einzeltasten stehen unter [Tastenkürzel](help:shortcuts).

## Neue Sprite-Datei
**Neue Sprite-Datei …** (links unter der Dateiliste, auch auf der leeren Seite des Editors) öffnet einen Dialog: Unter **Art und System** wählst du das Profil, gibst den **Namen** ein (Buchstaben, Ziffern, `_` und `-`), prüfst den **Ordner im Projekt** (Vorgabe `graphics`) und klickst auf **Anlegen**. Die Profile gibt es nur für die Zielsysteme des Projekts.

## Hardware-Flags
Für C64 und C128 zeigt die Karte **Sprite** drei Schalter: **Doppelt breit** (X2, `$D01D`), **Doppelt hoch** (Y2, `$D017`) und **Hinter dem Hintergrund** (BG, `$D01B`). Multicolor (MC, `$D01C`) folgt dem Modus des gewählten Frames und hat keinen eigenen Schalter; den Modus stellst du mit dem Schalter **Mehrfarbig** in derselben Karte ein. Die drei Schalter gelten für die ganze Datei und stehen als optionales Feld `hw` in der `.vcsprite` (`expandX`, `expandY`, `behind`). Sind alle aus, fehlt das Feld, und alte Dateien bleiben unverändert. Der Export hängt dann die Konstanten `<name>_expand_x`, `<name>_expand_y` und `<name>_behind` (0 oder 1) an. Die Karte **Vorschau im Spiel** zeigt das gewählte Bild vor einem festen Hintergrund, mit den gesetzten Schaltern, und nennt, wie viele Bytes ein Bild und die Datei belegen.

Bei Atari Player/Missiles stellt die Karte **Player/Missile** die **Breite** je Spieler (SIZEP0 bis SIZEP3: Normal, Doppelt, Vierfach) und die **Priorität** (PRIOR) ein; der Export schreibt beides als Konstanten. Ein Player zeigt Zeile für Zeile den Speicher von oben nach unten; die ersten etwa 32 Zeilen liegen über dem sichtbaren Bild.

## Hires und Multicolor mischen (C64)
Ein Sprite-Asset gehört zu einem Profil (hires 24×21 oder multicolor 12×21), aber jedes Frame darf einen eigenen **Modus** haben. In der Karte **Sprite** schaltest du **Mehrfarbig** für das gewählte Bild ein oder aus; der Editor passt Rasterbreite, Pixelwerte und Farben an. Beim Wechsel werden die Pixel umgerechnet (Hires → Multicolor: je zwei Pixel werden eins; Multicolor → Hires: jedes Pixel wird doppelt breit, alle Farben außer „transparent“ werden zur Sprite-Farbe). Cmd/Strg+Z macht den Wechsel rückgängig. Die Multicolor-Farben 1 und 2 gelten für alle Frames des Assets (wie die Register `$D025`/`$D026`).

Der Export schreibt jedes Frame in seinem Modus und legt die Konstanten `<name>_count` und `<name>_multicolor_mask` (Bit i = Frame i, für `$D01C`) an; bei gemischten Frames außerdem die Tabelle `<name>_modes` (je Frame 0 = hires, 1 = multicolor, siehe unten). „Onion Skin“ und „PNG laden“ gelten nur für Frames im Modus des Profils.

## Animationen
Im Reiter **Animationen** (links) legt **Neu** eine Folge mit dem gewählten Bild an; mehrere Animationen wählst du über ihre Knöpfe, im Feld **Name der Animation** benennst du sie. Die Schritte stehen als Filmstreifen: Jeder zeigt Miniatur, Nummer und **Dauer in Bildern** (1/50 s) und lässt sich verschieben oder entfernen; **Gewähltes Bild anhängen** fügt das gewählte Bild als Schritt hinzu. Der **Ablauf** ist *Schleife*, *Hin und zurück* (die Endbilder werden nicht doppelt gezeigt) oder *Einmal*. Die **Zeitleiste** unter der Zeichenfläche spielt die Animation in Echtzeit ab. Frames dürfen in einer Animation hires und multicolor mischen. Wird ein Frame eingefügt oder gelöscht, werden die Animationen nachgeführt.

Du kannst auch in der Bilderliste mit Umschalt- oder Strg-Klick mehrere Bilder markieren (Cmd/Strg+A markiert alle) und **Animation aus Auswahl** wählen.

Gibt es Animationen oder gemischte Frames, schreibt der Export neben `<name>.asm` eine zweite Datei **`<name>.anim.asm`** (Daten, die hinter die Sprite-Daten gehören: beim Einbinden der Mediendatei, `.include "graphics/<name>.vcsprite"`, kommen sie von selbst direkt hinter die Sprite-Daten). Sie enthält je Animation (`<name>_anim_<animation>_…`):
- `_ptr`: die Sprite-Zeiger, also die Blocknummer im VIC-Bank (`(Adresse & $3FFF) / 64 + Frame`), „Hin und zurück“ schon ausgeschrieben;
- `_delay`: die Dauer jedes Schritts in Bildern;
- `_len`: die Zahl der Schritte, `_mode`: 0 = Schleife, 1 = einmal.

Im Raster-Interrupt zählst du die Dauer herunter und schreibst beim Ablauf den nächsten Zeiger in den Sprite-Zeiger (`$07F8` + Sprite-Nummer). Die Sprite-Daten (`<name>`) müssen auf eine 64-Byte-Grenze gelegt werden, z. B. mit `.org $2000`.

## Sprites aus anderen Quellen übernehmen
**PNG laden …** (Werkzeugleiste) wandelt ein Bild in Pixelwerte des gewählten Bildes um; dabei speichert der Editor die Datei. Das Übernehmen aus SPD-Dateien, Rohdaten und Sprites anderer Systeme steht in dieser Version noch nicht zur Verfügung.
