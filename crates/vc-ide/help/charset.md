# Zeichensatz-Editor

Bearbeitet 256 Zeichen (C64) bzw. 128 Zeichen (Atari ANTIC 2), je 8×8 Pixel; multicolor als 4×8 mit doppelt breiten Pixeln.

- Zeichen links auswählen (Pfeiltasten wandern durch das Raster), rechts Pixel setzen.
- **Testtext** zeigt eigenen Text im Zeichensatz.
- Die Reihenfolge ist die der **Bildschirmcodes** (nicht ASCII): C64 `@`=0, `A`=1…; Atari: Leerzeichen=0, Ziffern ab 16, Großbuchstaben ab 33.
- Der Zeichensatz kann in den Bildschirm- und Kacheleditoren als Glyphenquelle dienen (Feld „Zeichensatz“).
- Laden im Programm: `gfx_load_charset` (Assembler) bzw. `gosub @gfx_charset` (BASIC).

## Neuer Zeichensatz und Vorlage
Ein **neuer Zeichensatz** (Datei > Neues Asset, `vcc media new`) beginnt nicht leer, sondern mit einem **Verweis auf den Standardzeichensatz des Systems** in deinem ROM, so wie er nach dem Einschalten erscheint: beim C64, C128 (40 Spalten) und MEGA65 „Großbuchstaben und Grafik“, beim Atari der Standardsatz, beim Plus/4 sein Satz. Leer beginnt er mit `vcc media new … --blank`.
Die Datei enthält dann nur `"base": "rom:c64/chargen#upper"` und **deine Änderungen** (`overrides`, Format 2); die Original-Zeichen stehen nicht im Projekt. Hat das System mehrere Zeichensatz-Quellen (C64: je ROM-Revision zwei Sätze, Atari: Standard und international, C128-VDC), wählt man oben im Editor unter **Vorlage** eine andere; das ersetzt alle Zeichen (mit Strg+Z rückgängig). Fehlt das ROM, zeigt der Editor eine **eingebaute Ersatzschrift** (Eigenbau, ohne Grafikzeichen) mit Hinweis; deine Änderungen bleiben erhalten und gelten über dem echten ROM, sobald es da ist.

## Standardzeichensätze
**Datei > Standardzeichensatz hinzufügen …** legt für die Zielsysteme des Projekts Zeichensätze an, die auf dein ROM **verweisen** (nach `graphics/`): C64, C128 (40 Spalten und VDC 8×16), Plus/4 und C16, VC-20, Atari (Standard und international) sowie MEGA65 (Vollfarbe, aus dem C64-Satz). Danach lassen sie sich wie jeder andere Zeichensatz bearbeiten. Wahlweise bettet „Zeichen stattdessen einbetten“ die Glyphen ein (Kopie nach `graphics/standard/`, nicht einchecken). Auf der Kommandozeile: `vcc media std-charset` (Liste der Quellen mit Status) und `vcc media std-charset rom:c64/chargen#upper [--profile ID] [--copy]`.

Die Zeichensätze der Systeme sind urheberrechtlich geschützt (die Rechte liegen bei den Herstellern der Systeme). Deshalb liefert VintageCoder sie nicht mit; Verweise sind frei weitergebbar, eingebettete Kopien nicht ohne Weiteres. Siehe Hilfe „ROM-Dateien“.

## Aufbau
Das Studio folgt dem [gemeinsamen Aufbau](help:graphics): links die Matrix der Zeichen, in der Mitte das 8×8-Canvas, rechts die Pixelwerte mit Registern (beim C64 zum Beispiel Hintergrund `$D021`, im Multicolor-Modus `$D022` und `$D023`, beim Atari ANTIC 2 `COLPF2` und `COLPF1`), die Farben und die Vorschau. Beim Atari wählt **Videonorm** PAL oder NTSC. Der Export-Streifen unten öffnet den [Export-Dialog](help:export).
