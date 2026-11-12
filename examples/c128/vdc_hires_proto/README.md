# C128: VDC-Hires-Prototyp (640 x 200)

Machbarkeitsstudie für den PRALLBALL-Ausbau. Kein Spiel: kein Klang, keine Eingabe.

## Was es zeigt
- VDC-Bitmap 640 x 200 mit Farbattributen je 8 x 8 Pixel (Register 25 = `$C7`, Bildanfang `$0000`, Attribute `$4000`).
- 64-KB-VDC erkannt (Register 28 Bit 4; Spiegelungstest `$0000`/`$8000`); Hintergrundbild im VDC-RAM ab `$5000`.
- Steinwand (6 Reihen x 26 Steine, je Stein eigene Farbe), Rand, 8 Kugeln (8 x 8 Pixel) in 8 Feinlagen. Je Bild: alte Lage per **Blockkopie** aus dem Hintergrund wiederherstellen, neue Lage zeichnen.
- Messung der Bildzeit mit dem CIA-Zeitgeber (Mikrosekunden) und der Rasterzeile; Ergebnis im Hauptspeicher ab `$C200` (Aufbau im Quelltext).

## Messwerte (VICE 3.10 x128, `-80col -VDC64KB`, PAL, 2 MHz)
- 200 Bilder, 8 Kugeln: **etwa 28 ms je Bild** (ein PAL-Bild sind 20 ms), das sind rund **3,5 ms je Kugel** (Löschen mit 8 Blockkopien + Zeichnen von 8 Zeilen). Mit 8 Kugeln reicht es nicht für 50 Bilder/s, mit etwa 4 bis 5 Kugeln schon.
- Der VDC-Zugriff ist der Engpass: jedes Register- und Datenbyte wartet auf das Bereit-Bit. Das ist **Emulatorzeit**; am echten Gerät nicht geprüft.

## Bekannte Mängel
- Neben jeder Kugel steht ein kleiner senkrechter Balken (Rest der Zeichenroutine, Ursache nicht geklärt).
- Kugeln zeichnen ohne Verknüpfung (Überschreiben, kein ODER): sie fliegen nur im freien Feld unter den Steinen. Für das Spiel braucht man die Verknüpfung (Lesen im VDC ist langsam) oder getrennte Bildebenen.
- Ohne 64-KB-VDC (16 KB) zeigt der Prototyp nur die Steine.

## Lesen der Ergebnisse
`bank ram`, dann `m c200 c21f` im Monitor: `$C200` = `$FF` fertig, `$C201` = 1 bei 64 KB, `$C202/3` = größte Rasterzeilenzahl (Umlauf über 312 Zeilen!), `$C204..6` Summe, `$C207/8` Bilder, `$C210..3` letzte Bildzeit in µs.
