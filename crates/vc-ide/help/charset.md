# Zeichensatz-Editor

Bearbeitet 256 Zeichen (C64) bzw. 128 Zeichen (Atari ANTIC 2), je 8×8 Pixel; multicolor als 4×8 mit doppelt breiten Pixeln. Du findest den Editor im Hauptbereich **Grafik** in der zweiten Reihe unter **Zeichensatz**; ein Doppelklick auf eine `.vcchar`-Datei im Projektbaum öffnet sie ebenfalls dort.

- Zeichen links in der **Zeichenübersicht** auswählen (Pfeiltasten wandern durch das Raster), in der Mitte Pixel setzen.
- Die Reihenfolge ist die der **Bildschirmcodes** (nicht ASCII): C64 `@`=0, `A`=1…; Atari: Leerzeichen=0, Ziffern ab 16, Großbuchstaben ab 33.
- Laden im Programm: `gfx_load_charset` (Assembler) bzw. `gosub @gfx_charset` (BASIC).

## Aufbau
Der Editor hat drei Bereiche nebeneinander; in schmalen Fenstern rutscht der rechte unter die beiden anderen. Er folgt dem [gemeinsamen Aufbau](help:graphics).

| Bereich | Inhalt |
|---|---|
| Links | die **Zeichenübersicht** (16 Zeichen je Zeile), darunter die **Bereiche** (Alle, Buchstaben, Ziffern, Satzzeichen, Frei) und, wenn das Projekt mehrere Zeichensätze hat, die **Zeichensatz-Dateien** |
| Mitte | Werkzeugleiste, Nummer des Zeichens, die Zeichenfläche mit der **Umgebung** (das Zeichen davor und danach) und die Fußzeile |
| Rechts | **Vorschau im Bild**, **Vorlage** mit **Herkunft**, **Zeichen** (Nummer und Position), **Farben** und **Verschieben** |

![Der Zeichensatz-Editor im Bereich „Grafik“ mit der Datei font.vcchar. Links die Zeichenmatrix „FONT“ mit 256 Zeichen und den Bereichen „Alle“, „Buchstaben“, „Ziffern“, „Satzzeichen“ und „Frei“. In der Mitte die große Zeichenfläche mit Werkzeugleiste, rechts die Karten „VORSCHAU IM BILD“, „VORLAGE“, „ZEICHEN“, „FARBEN“ und „VERSCHIEBEN“.](bild:zeichensatz-ueberblick "Der Zeichensatz-Editor mit Zeichenmatrix, Zeichenfläche und den Karten rechts.")

Die Werkzeugleiste enthält **Stift** (D), **Radierer** (E), **Pipette** (I), **Füllen** (F), **Linie** (L), **Rechteck** (R), **Auswahl** (S), Rückgängig, Wiederholen, **Waagerecht spiegeln** (X), **Senkrecht spiegeln** (Y), **Raster** (G), den Zoom und die Dateiknöpfe **Neuer Zeichensatz …**, **PNG laden …**, **Exportieren …** und **Speichern**. **Exportieren …** (Cmd/Strg+Umschalt+E) öffnet den [Export-Dialog](help:export). Die Einzeltasten stehen unter [Tastenkürzel](help:shortcuts).

Rechts stehen die Pixelwerte mit Registern in der Karte **Farben** (beim C64 zum Beispiel Hintergrund `$D021`, im Multicolor-Modus `$D022` und `$D023`, beim Atari ANTIC 2 `COLPF2` und `COLPF1`). Beim Atari stellt die **Palettenansicht** zwischen **Projekt**, **PAL** und **NTSC** um. **Verschieben** schiebt das ganze Zeichen mit vier Pfeilknöpfen; **Zeichen leeren** löscht alle Punkte. Die **Bereiche** springen zum ersten Zeichen einer Gruppe; bei **Frei** gibst du **von** und **bis** als Zeichennummern ein. Die **Vorschau im Bild** zeigt einen Text (Feld **Text für die Vorschau**) in den Zeichen deines Satzes.

## Neuer Zeichensatz und Vorlage
Ein **neuer Zeichensatz** (Knopf **Neuer Zeichensatz …** im Editor, oder `vcc media new`) beginnt nicht leer, sondern mit einem **Verweis auf den Standardzeichensatz des Systems** in deinem ROM, so wie er nach dem Einschalten erscheint: beim C64, C128 (40 Spalten) und MEGA65 „Großbuchstaben und Grafik“, beim Atari der Standardsatz, beim Plus/4 sein Satz. Leer beginnt er mit `vcc media new … --blank`.
Die Datei enthält dann nur `"base": "rom:c64/chargen#upper"` und **deine Änderungen** (`overrides`, Format 2); die Original-Zeichen stehen nicht im Projekt. Hat das System mehrere Zeichensatz-Quellen (C64: je ROM-Revision zwei Sätze, Atari: Standard und international, C128-VDC), wählst du in der Karte **Vorlage** unter **Vorlage zum Laden** eine andere und klickst auf **Vorlage laden**; das ersetzt alle Zeichen (mit Cmd/Strg+Z rückgängig). Ein Eintrag, dessen ROM nicht gefunden wird, ist grau. Fehlt das ROM, zeigt der Editor den **eigenen Standardsatz** von VintageCoder (Eintrag **Standard (eigen)**, Etikett EIGEN, frei unter CC0); deine Änderungen bleiben erhalten und gelten über dem echten ROM, sobald es da ist.

Die Karte **Herkunft** sagt in Worten, woher die Zeichen stammen, mit einem Etikett (**EIGEN**, **EIGEN, GEÄNDERT**, **AUS DEINEM ROM** oder **ROM FEHLT**). Geänderte Zeichen haben in der Übersicht einen kleinen Punkt; **Zeichen auf Vorlage zurücksetzen** holt das Original des gewählten Zeichens zurück.

*[Bild folgt in E13: zeichensatz-standard]*

## Standardzeichensätze
Die Standardzeichensätze für die Zielsysteme des Projekts (mit Verweis auf dein ROM, `VC-Standard` als eigene Kopie) legt die Kommandozeile an: `vcc media std-charset` (Liste der Quellen mit Status) und `vcc media std-charset rom:c64/chargen#upper [--profile ID] [--copy]`. Der Dialog **Zeichensatz hinzufügen** in der Oberfläche steht noch nicht zur Verfügung. Im Dialog **Neues Projekt erstellen** wählst du den **Zeichensatz des Projekts** (*Automatisch*, *Nur Standard (eigen)* oder *Keinen*), siehe [Startseite, Konfigurationsassistent und neues Projekt](help:wizard).

Die Zeichensätze der Systeme liefert VintageCoder nicht mit; Verweise darauf sind frei weitergebbar. Siehe Hilfe [ROM-Dateien](help:roms).
