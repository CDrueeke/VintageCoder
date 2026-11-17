# Herkunft der Codebelegung (Stand 2026-11-16)

Die Zeichenformen der Sätze „VC-Standard“ sind eigene Gestaltung (CC0-1.0, `STIL.md`). **Welches Zeichen auf welchem Code liegt**, folgt dem
jeweiligen System und wurde gegen öffentliche Code-Tabellen geprüft. Benutzt wurden nur Codes und Zeichennamen, nie Zeichenformen; es wurden
keine ROM-Abbilder, Emulator-Bitmaps oder Nachbauten gelesen. Jede Satzdatei trägt das Ergebnis in `origin.belegung`
(`status`, `datum`, `geprueft_gegen`, `hinweis`); `index.json` führt `belegung` und `belegung_datum`.

## Quellen

- Unicode-Zuordnung PETSCII von Triad: https://www.df.lth.se/~triad/krad/recode/ (`petscii_c64en_uc.txt`, `petscii_c64en_lc.txt`, `petscii_vic20en_uc.txt`)
- Aivosto, Commodore PETSCII character sets (2014): https://www.aivosto.com/articles/petscii.pdf
- Wikipedia, Artikel PETSCII und ATASCII
- COMPUTE! Nr. 26, Juli 1982, S. 130, „All About PET/CBM Character Sets“
- Altirra Hardware Reference Manual (Avery Lee), Kapitel 9 (Tabelle der internationalen Zeichen)
- Forum plus4world.powweb.com (Grundzeichensatz des Plus/4)

## Status je Satz

| Satz | Status | Bemerkung |
|---|---|---|
| `vc-c64-upper`, `vc-c64-lower` | quellengeprüft | Satz 2 wechselt genau die Buchstaben und die vier Grafikcodes 94, 95, 105, 122 |
| `vc-vic20-upper`, `vc-vic20-lower` | quellengeprüft | Zuordnung der Grafikzeichen laut Tabelle gleich der des C64 |
| `vc-pet-upper`, `vc-pet-lower` | quellengeprüft | wie C64/VIC-20, nur Code 28 ist ein Rückwärtsschrägstrich statt £ (PET 8032) |
| `vc-atari-standard` | quellengeprüft | eine Quelle (Wikipedia-Tabelle) für die Grafikzeichen |
| `vc-atari-international` | quellengeprüft | zwei Quellen; ersetzt Codes 64 bis 90, 96 (¡), 123 (Ä) |
| `vc-plus4-standard`, `vc-plus4-lower` | quellengeprüft | Belegung wie C64; das Zeichen-ROM ($D000 bis $D7FF, 2 KB) hat zwei Sätze zu je 128 Zeichen, den Invers-Teil erzeugt der TED (Plus/4 Docs) |
| `vc-c128-upper`, `vc-c128-lower` | quellengeprüft | wie C64 (Quelle nennt C64, C16, C128 zusammen) |
| `vc-c128-vdc-*`, `vc-mega65-*` | **vorläufig** | keine eigene Quelle abgefragt |

## Offene Punkte (nicht belegbar oder nicht entschieden)

1. **Plus/4 und C16 (geklärt: zwei Sätze).** Laut Plus/4 Docs liegt das Zeichen-ROM bei $D000 bis $D7FF (2 KB = 2 × 128 Zeichen; Umkehrbit des TED erzeugt die Invers-Hälfte); angelegt sind jetzt `vc-plus4-standard` (Groß/Grafik) und `vc-plus4-lower` (Klein/Groß). Der ROM-Import (`vc-roms`, `vc-media`) liest sie als `upper` ($1000, 1 KB) und `lower` ($1400, 1 KB) und erzeugt die Invers-Hälfte. Die Reihenfolge der beiden 1-KB-Teile ist am 2026-11-17 an den VICE-3.10-KERNAL-Abbildern 318004-01, 318004-05, 318005-05 und 364 gemessen (nur Vergleich, welcher Teil Großbuchstaben auf Code 65 hat; keine Zeichenformen gespeichert): erster Teil ($1000) Groß/Grafik, zweiter ($1400) Klein/Groß.
2. **C128 VDC und MEGA65:** Codebelegung des VDC-Zeichenspeichers und der MEGA65-Zeichen (Vollfarbmodus) nicht gegen ein Handbuch geprüft; Annahme: wie C64.
3. **C128 40 Spalten:** keine eigene Bildschirmcode-Tabelle des C128 gelesen (nur die PETSCII-Tabelle, die C64, C16 und C128 zusammenfasst); nationale Varianten (Deutsch, Schwedisch …) sind nicht angelegt.
4. **PET:** Das PET 2001 der ersten Serie hat vertauschte Groß-/Kleinbuchstaben (nicht angelegt). Grafikzeichen des PET sind nur über „gleich wie VIC-20“ (Aivosto) belegt, nicht Zeichen für Zeichen. CBM-II wird nicht angelegt.
5. **Atari Standard:** Grafikzeichen 0 bis 31 nur aus einer Quelle; eine zweite Tabelle (Atari-OS-Handbuch) war nur als Bild verfügbar.
6. **Linienleitern des C64:** Die Namen („eine Viertel nach oben/links/…“) legen die Reihenfolge der Linien fest, nicht die genaue Zeile oder Spalte; die gewählten Leitern (Zeilen 1 bis 6, Spalten 1 bis 6) sind eigene Gestaltung.
