# SPDX-License-Identifier: GPL-3.0-or-later
# Texte der Oberfläche: media (aus crates/vc-media/src/cpu6502.rs extrahiert, Wortlaut unverändert)

# Kontext: cpu6502.rs, Err
media-unbekannter-befehl-op-bei-at = unbekannter Befehl ${ $op } bei ${ $at }

# Kontext: goat.rs, Err
media-datei-zu-kurz = Datei zu kurz

# Kontext: goat.rs, Err
media-alte-goattracker-version-gts2-gts-bitte = alte GoatTracker-Version (GTS2/GTS!): bitte in GoatTracker 2.x laden und neu speichern

# Kontext: goat.rs, Err
media-keine-goattracker-datei-erwartet-gts5 = keine GoatTracker-Datei (erwartet GTS5)

# Kontext: goat.rs, Err
media-restdaten-nach-den-patterns = Restdaten nach den Patterns

# Kontext: goat.rs, Text
media-stereo-song-2-sids-nur-die = Stereo-Song (2 SIDs): nur die ersten drei Stimmen werden übernommen

# Kontext: goat.rs, Text
media-p1-subtunes-nur-der-erste-wird = { $p1 } Subtunes: nur der erste wird übernommen

# Kontext: goat.rs, ok_or
media-orderliste-fehlt = Orderliste fehlt

# Kontext: goat.rs, Err
media-song-zu-lang-ueber-65536-zeilen = Song zu lang (über 65536 Zeilen)

# Kontext: goat.rs, Text
media-orderliste-verweist-auf-ein-nicht-vorhandenes = Orderliste verweist auf ein nicht vorhandenes Pattern (übersprungen)

# Kontext: goat.rs, Text
media-clamped-noten-lagen-nach-der-transponierung = { $clamped } Noten lagen nach der Transponierung außerhalb des Bereichs und wurden begrenzt

# Kontext: goat.rs, Err
media-song-enthaelt-keine-zeilen = Song enthält keine Zeilen

# Kontext: goat.rs, Text
media-die-stimmen-sind-unterschiedlich-lang-kuerzere = Die Stimmen sind unterschiedlich lang: kürzere werden mit Pausen aufgefüllt (GoatTracker lässt sie unabhängig wiederholen)

# Kontext: goat.rs, Text
media-die-stimmen-haben-verschiedene-wiederholungspunkte-es = Die Stimmen haben verschiedene Wiederholungspunkte: es gilt der der ersten Stimme

# Kontext: goat.rs, Text
media-tempo-speed-liegt-ueber-dem-tracker = Tempo { $speed } liegt über dem Tracker-Maximum 31 und wurde auf 31 gesetzt

# Kontext: goat.rs, Text
media-kanalspezifische-tempi-und-funktempo-rueckruf-fxy = Kanalspezifische Tempi und Funktempo-Rückruf (FXY mit $00/$01/$80+) werden nicht übernommen

# Kontext: goat.rs, Text
media-befehl-c-xy-p2-in-n = Befehl { $c }XY ({ $p2 }) in { $n } Zeilen nicht übernommen

# Kontext: goat.rs, Text
media-wellentabelle-arpeggio-wellenformwechsel-nur-der-erste = Wellentabelle (Arpeggio, Wellenformwechsel): nur der erste Eintrag zählt – Instrument { $p1 }

# Kontext: goat.rs, Text
media-kombinierte-wellenform-auf-eine-einzelne-wellenform = Kombinierte Wellenform auf eine einzelne Wellenform abgebildet – Instrument { $p1 }

# Kontext: goat.rs, Text
media-pulsbreiten-modulation-wird-nicht-uebernommen-instrument = Pulsbreiten-Modulation wird nicht übernommen – Instrument { $p1 }

# Kontext: goat.rs, Text
media-filtertabelle-wird-nicht-uebernommen-instrument-p1 = Filtertabelle wird nicht übernommen – Instrument { $p1 }

# Kontext: goat.rs, Text
media-vibrato-wird-nicht-uebernommen-instrument-p1 = Vibrato wird nicht übernommen – Instrument { $p1 }

# Kontext: goat.rs, Text
media-zeilen-mit-nicht-vorhandenem-instrument-behalten = Zeilen mit nicht vorhandenem Instrument behalten das vorherige Instrument

# Kontext: goat.rs, Text
media-patternlaengen-haben-nur-den-gemeinsamen-teiler = Patternlängen haben nur den gemeinsamen Teiler { $n }: es wird mit 16 Zeilen je Pattern gearbeitet

# Kontext: goat.rs, Text
media-der-wiederholungspunkt-liegt-nicht-auf-einer = Der Wiederholungspunkt liegt nicht auf einer Patterngrenze und wurde nach vorn gerückt

# Kontext: import.rs, Err
media-png-import-fuer-p1-nicht-verfuegbar = PNG-Import: für '{ $p1 }' nicht verfügbar

# Kontext: import.rs, Err
media-png-import-nur-fuer-profile-mit = PNG-Import: nur für Profile mit höchstens 16 Farben (4 Bit je Pixel)

# Kontext: import.rs, ok_or
media-frames-fehlt = frames fehlt

# Kontext: import.rs, Err
media-frame-frame-existiert-nicht = Frame { $frame } existiert nicht

# Kontext: import.rs, ok_or
media-colors-fehlt = colors fehlt

# Kontext: import.rs, Err
media-keine-zielfarben-in-dieser-datei = keine Zielfarben in dieser Datei

# Kontext: lib.rs, Err
media-profil-id-extends-ist-zu-tief = Profil '{ $id }': extends ist zu tief verschachtelt (höchstens 3 Ebenen) oder zyklisch

# Kontext: lib.rs, ok_or_else
media-profil-id-extends-verweist-auf-unbekanntes = Profil '{ $id }': extends verweist auf unbekanntes Profil '{ $parent_id }'

# Kontext: lib.rs, ok_or
media-profil-ist-kein-objekt = Profil ist kein Objekt

# Kontext: lib.rs, ok_or
media-profiles-fehlt = profiles fehlt

# Kontext: lib.rs, Text
media-datei-json-e = Datei (JSON): { $e }

# Kontext: lib.rs, ok_or
media-profile-fehlt = profile fehlt

# Kontext: lib.rs, Err
media-datei-format-1-erwartet = Datei: format 1 erwartet

# Kontext: lib.rs, ok_or
media-kind-fehlt = kind fehlt

# Kontext: lib.rs, ok_or
media-frame-rows-fehlt = frame.rows fehlt

# Kontext: lib.rs, Err
media-frame-hat-p1-statt-h-zeilen = Frame hat { $p1 } statt { $h } Zeilen

# Kontext: lib.rs, ok_or
media-zeile-ist-kein-string = Zeile ist kein String

# Kontext: lib.rs, Err
media-zeile-s-hat-p2-statt-w = Zeile '{ $s }' hat { $p2 } statt { $w } Pixel ({ $digits } Hex-Ziffer(n) je Pixel)

# Kontext: lib.rs, ok_or
media-pixelwert-t-ungueltig-max-maxv = Pixelwert '{ $t }' ungültig (max. { $maxv })

# Kontext: lib.rs, Err
media-p1-hat-keine-bitmap-kodierung-nutze = '{ $p1 }' hat keine Bitmap-Kodierung (nutze encode_screen/encode_tilemap)

# Kontext: lib.rs, Err
media-zeichensatz-braucht-p1-zeichen-datei-hat = Zeichensatz braucht { $p1 } Zeichen, Datei hat { $p2 }

# Kontext: lib.rs, Err
media-breite-bits-pro-pixel-muss-durch = Breite × Bits pro Pixel muss durch 8 teilbar sein

# Kontext: lib.rs, Err
media-frame-i-p2-bytes-passen-nicht = Frame { $i }: { $p2 } Bytes passen nicht in Block von { $p3 }

# Kontext: lib.rs, Err
media-rom-import-nur-fuer-zeichensaetze-p1 = ROM-Import: nur für Zeichensätze, '{ $p1 }' ist { $p2 }

# Kontext: lib.rs, ok_or_else
media-offset-offset-liegt-hinter-dem-dateiende = Offset { $offset } liegt hinter dem Dateiende ({ $p2 } Bytes)

# Kontext: lib.rs, ok_or
media-frames-fehlen = frames fehlen

# Kontext: lib.rs, Err
media-png-raster-nur-fuer-zeichensaetze-p1 = PNG-Raster: nur für Zeichensätze, '{ $p1 }' ist { $p2 }

# Kontext: lib.rs, Err
media-png-raster-nur-fuer-zeichensaetze-mit = PNG-Raster: nur für Zeichensätze mit 1 Bit je Pixel (und die MEGA65-Vollfarbe)

# Kontext: lib.rs, Err
media-bild-p1-p2-passt-nicht-zu = Bild { $p1 }×{ $p2 } passt nicht zu { $cols }×{ $rows } Zeichen

# Kontext: lib.rs, Err
media-zeichen-im-bild-sind-cw-ch = Zeichen im Bild sind { $cw }×{ $ch } Pixel – erwartet ein ganzzahliges Vielfaches von { $p3 }×{ $p4 }

# Kontext: lib.rs, Err
media-png-raster-nur-zeichensaetze-mit-1 = PNG-Raster: nur Zeichensätze mit 1 Bit (oder MEGA65-Vollfarbe)

# Kontext: lib.rs, ok_or
media-rows-fehlen = rows fehlen

# Kontext: lib.rs, Err
media-zu-wenige-daten = zu wenige Daten

# Kontext: lib.rs, Err
media-p1-bytes-fuer-n-bloecke-zu = { $p1 } Bytes für { $n } Blöcke zu je { $block } nötig

# Kontext: lib.rs, ok_or
media-key-fehlt = { $key } fehlt

# Kontext: lib.rs, Err
media-key-p2-statt-p3-zeilen = { $key }: { $p2 } statt { $p3 } Zeilen

# Kontext: lib.rs, ok_or
media-zeile-ist-kein-array = Zeile ist kein Array

# Kontext: lib.rs, Err
media-key-zeile-hat-p2-statt-p3 = { $key }: Zeile hat { $p2 } statt { $p3 } Zellen

# Kontext: lib.rs, ok_or
media-key-wert-ausserhalb-0-255 = { $key }: Wert außerhalb 0..255

# Kontext: lib.rs, ok_or
media-tiles-fehlt = tiles fehlt

# Kontext: lib.rs, ok_or
media-kachel-i-chars-fehlt = Kachel { $i }: chars fehlt

# Kontext: lib.rs, ok_or
media-kachel-i-zeichencode-ausserhalb-0-255 = Kachel { $i }: Zeichencode außerhalb 0..255

# Kontext: lib.rs, ok_or
media-map-fehlt = map fehlt

# Kontext: lib.rs, ok_or
media-map-zeile-ist-kein-array = map-Zeile ist kein Array

# Kontext: lib.rs, ok_or
media-map-zahl-erwartet = map: Zahl erwartet

# Kontext: lib.rs, Err
media-map-verweist-auf-kachel-v-es = map verweist auf Kachel { $v }, es gibt nur { $p2 }

# Kontext: lib.rs, Text
media-erzeugt-von-vintagecoder-nicht-von-hand = ; erzeugt von VintageCoder – nicht von Hand ändern

# Kontext: lib.rs, Text
media-rem-erzeugt-von-vintagecoder-p1-bytes = REM erzeugt von VintageCoder: { $p1 } Bytes

# Kontext: lib.rs, Text
media-erzeugt-von-vintagecoder-const-label-len = ; erzeugt von VintageCoder

# Kontext: lib.rs, ok_or
media-entries-fehlt = entries fehlt

# Kontext: lib.rs, Err
media-palette-braucht-256-eintraege-datei-hat = Palette braucht 256 Einträge, Datei hat { $p1 }

# Kontext: lib.rs, Text
media-eintrag-i-ungueltige-farbe-s = Eintrag { $i }: ungültige Farbe '#{ $s }'

# Kontext: naming.rs, ok_or_else
media-r-kind-fehlt = { $r }: kind fehlt

# Kontext: naming.rs, Err
media-umstellung-abgebrochen-t-existiert-bereits-oder = Umstellung abgebrochen: { $t } existiert bereits oder kommt doppelt vor

# Kontext: palette.rs, ok_or_else
media-palettes-index-json-kein-eintrag-fuer = palettes/index.json: kein Eintrag für '{ $target }'

# Kontext: palette.rs, ok_or_else
media-palettes-index-json-target-ohne-palette = palettes/index.json: '{ $target }' ohne palette/builtin

# Kontext: palette.rs, ok_or_else
media-palettes-name-json-key-fehlt = palettes/{ $name }.json: '{ $key }' fehlt

# Kontext: palette.rs, ok_or_else
media-palettes-name-json-ungueltige-farbe = palettes/{ $name }.json: ungültige Farbe

# Kontext: play.rs, ok_or
media-instruments-fehlt = instruments fehlt

# Kontext: play.rs, Err
media-play-kennt-huellkurven-0-9-der = PLAY kennt Hüllkurven 0–9, der Song hat { $p1 } Instrumente

# Kontext: play.rs, ok_or
media-steps-fehlt = steps fehlt

# Kontext: play.rs, ok_or
media-zeile-r-array-erwartet = Zeile { $r }: Array erwartet

# Kontext: play.rs, Err
media-zeile-r-p2-statt-voices-zellen = Zeile { $r }: { $p2 } statt { $voices } Zellen

# Kontext: play.rs, Text
media-zeile-r-stimme-p2-e = Zeile { $r }, Stimme { $p2 }: { $e }

# Kontext: play.rs, Text
media-rem-p1-erzeugt-von-vintagecoder = REM { $p1 } - erzeugt von VintageCoder

# Kontext: play.rs, Err
media-stimme-p1-oktave-oct-play-kennt = Stimme { $p1 }: Oktave { $oct } - PLAY kennt nur 0–6

# Kontext: play.rs, Err
media-stimme-p1-instrument-i-existiert-nicht = Stimme { $p1 }: Instrument { $i } existiert nicht

# Kontext: png.rs, Err
media-png-ungueltiger-huffman-code = PNG: ungültiger Huffman-Code

# Kontext: png.rs, Err
media-png-zlib-strom-zu-kurz = PNG: zlib-Strom zu kurz

# Kontext: png.rs, Err
media-png-ungueltige-laengen = PNG: ungültige Längen

# Kontext: png.rs, Err
media-png-ungueltiges-symbol = PNG: ungültiges Symbol

# Kontext: png.rs, Err
media-png-ungueltige-distanz = PNG: ungültige Distanz

# Kontext: png.rs, Err
media-png-distanz-zu-gross = PNG: Distanz zu groß

# Kontext: png.rs, Err
media-png-ungueltiger-block = PNG: ungültiger Block

# Kontext: png.rs, Err
media-keine-png-datei = keine PNG-Datei

# Kontext: png.rs, Err
media-png-ungueltige-groesse = PNG: ungültige Größe

# Kontext: png.rs, Err
media-png-interlace-wird-nicht-unterstuetzt-bitte = PNG: Interlace wird nicht unterstützt (bitte ohne Interlace speichern)

# Kontext: png.rs, Err
media-png-farbtyp-nicht-unterstuetzt = PNG: Farbtyp nicht unterstützt

# Kontext: png.rs, Err
media-png-bittiefe-nicht-unterstuetzt = PNG: Bittiefe nicht unterstützt

# Kontext: png.rs, Err
media-png-zu-wenige-bilddaten = PNG: zu wenige Bilddaten

# Kontext: png.rs, Err
media-png-ungueltiger-filter = PNG: ungültiger Filter

# Kontext: png.rs, Err
media-png-palettenindex-ungueltig = PNG: Palettenindex ungültig

# Kontext: project.rs, Err
media-name-nur-buchstaben-ziffern-und = Name: nur Buchstaben, Ziffern, _ und -

# Kontext: project.rs, Err
media-profil-p1-gehoert-zum-system-p2 = Profil '{ $p1 }' gehört zum System '{ $p2 }', das Projekt hat nur: { $p3 }

# Kontext: project.rs, Err
media-export-fuer-k-nicht-unterstuetzt = Export für '{ $k }' nicht unterstützt

# Kontext: project.rs, Text
media-p1-datei-en = { $p1 } Datei(en)

# Kontext: project.rs, ok_or
media-prg-export-braucht-eine-adresse = PRG-Export braucht eine Adresse

# Kontext: project.rs, Text
media-p1-datei-en-ladeadresse-p2 = { $p1 } Datei(en), Ladeadresse ${ $p2 }

# Kontext: project.rs, Text
media-erzeugt-von-vintagecoder-daten-fuer-strukturiertes = ' erzeugt von VintageCoder – Daten für strukturiertes BASIC (Reihenfolge: { $p1 })

# Kontext: project.rs, Err
media-base64-ungueltiges-zeichen = Base64: ungültiges Zeichen

# Kontext: project.rs, Err
media-pokey-songs-werden-als-rmt-modul = POKEY-Songs werden als RMT-Modul exportiert: Format 'asm' oder 'rmt' (nicht '{ $f }')

# Kontext: project.rs, Err
media-p1-kein-song-profil-p2 = { $p1 }: kein Song (Profil { $p2 })

# Kontext: project.rs, Err
media-der-song-hat-fc-stimmen-das = Der Song hat { $fc } Stimmen, das Ziel nur { $tc }

# Kontext: project.rs, Text
media-gleicher-chip-p1-leere-stimme-n = Gleicher Chip: { $p1 } leere Stimme(n) ergänzt

# Kontext: project.rs, Text
media-gleicher-chip-der-song-wurde-unveraendert = Gleicher Chip: der Song wurde unverändert übernommen

# Kontext: project.rs, Err
media-profil-p1-ist-kein-sprite-profil = Profil '{ $p1 }' ist kein Sprite-Profil

# Kontext: project.rs, Err
media-p1-kein-sprite-art-k-profil = { $p1 }: kein Sprite (Art '{ $k }', Profil { $p3 })

# Kontext: project.rs, Err
media-sprite-import-in-p1-wird-nicht = Sprite-Import in '{ $p1 }' wird nicht unterstützt

# Kontext: project.rs, Err
media-sprite-p1-gibt-es-nicht-die = Sprite { $p1 } gibt es nicht (die Quelle hat { $p2 })

# Kontext: project.rs, Text
media-die-quelle-hat-p1-sprites-uebernommen = Die Quelle hat { $p1 } Sprites; übernommen wird Sprite { $p2 } (ein PMG-Asset hat einen Satz aus 4 Playern)

# Kontext: project.rs, Err
media-import-fuer-dieses-profil-nicht-unterstuetzt = Import für dieses Profil nicht unterstützt

# Kontext: project.rs, ok_or_else
media-eigener-zeichensatz-id-ist-nicht-mitgeliefert = Eigener Zeichensatz „{ $id }“ ist nicht mitgeliefert

# Kontext: project.rs, Text
media-c64-grossbuchstaben-und-grafik-satz-1 = C64: Großbuchstaben und Grafik (Satz 1)

# Kontext: project.rs, Text
media-c64-gross-kleinschreibung-satz-2 = C64: Groß-/Kleinschreibung (Satz 2)

# Kontext: project.rs, Text
media-c128-zeichensatz-satz-1 = C128: Zeichensatz Satz 1

# Kontext: project.rs, Text
media-c128-zeichensatz-satz-2 = C128: Zeichensatz Satz 2

# Kontext: project.rs, Text
media-mega65-vollfarbe-satz-1 = MEGA65 (Vollfarbe): Satz 1

# Kontext: project.rs, Text
media-mega65-vollfarbe-satz-2 = MEGA65 (Vollfarbe): Satz 2

# Kontext: project.rs, Text
media-vc-20-grossbuchstaben-und-grafik-satz = VC-20: Großbuchstaben und Grafik (Satz 1)

# Kontext: project.rs, Text
media-vc-20-gross-kleinschreibung-satz-2 = VC-20: Groß-/Kleinschreibung (Satz 2)

# Kontext: project.rs, Text
media-atari-xl-xe-betriebssystem-zeichensatz-bei = Atari XL/XE-Betriebssystem: Zeichensatz bei $E000

# Kontext: project.rs, Text
media-atari-xl-xe-betriebssystem-internationaler-zeichensatz = Atari XL/XE-Betriebssystem: internationaler Zeichensatz bei $CC00

# Kontext: project.rs, Text
media-plus-4-kernal-rom-zeichensatz-bei = Plus/4 (KERNAL-ROM): Zeichensatz bei $D000

# Kontext: project.rs, Text
media-atari-400-800-os-b-zeichensatz = Atari 400/800 (OS-B): Zeichensatz bei $E000

# Kontext: project.rs, Text
media-atari-zeichensatz-bei-e000-rom-c000 = Atari: Zeichensatz bei $E000 (ROM $C000–$FFFF → Offset $2000)

# Kontext: project.rs, Text
media-c128-zeichen-rom-80-spalten-zeichensatz = C128 (Zeichen-ROM): 80-Spalten-Zeichensatz (VDC 8×16)

# Kontext: project.rs, Err
media-p1-oder-p2-existiert-bereits = { $p1 } oder { $p2 } existiert bereits

# Kontext: project.rs, Text
media-entfernt-p1 = entfernt: { $p1 }

# Kontext: project.rs, Text
media-keine-alten-kachelkarten-gefunden = keine alten Kachelkarten gefunden

# Kontext: romref.rs, ok_or_else
media-text-ist-kein-rom-verweis-rom = '{ $text }' ist kein ROM-Verweis (rom:system/rolle#satz)

# Kontext: romref.rs, ok_or_else
media-text-satz-fehlt-rom-system-rolle = '{ $text }': Satz fehlt (rom:system/rolle#satz)

# Kontext: romref.rs, Err
media-text-unvollstaendiger-rom-verweis = '{ $text }': unvollständiger ROM-Verweis

# Kontext: romref.rs, Text
media-eingebaute-ersatzschrift-eigenbau-nicht-die-originalschrift = Eingebaute Ersatzschrift (Eigenbau), nicht die Originalschrift des Systems – ROMs einrichten für das Original

# Kontext: romref.rs, Text
media-ersatzschrift-freie-rom-nicht-die-originalschrift = Ersatzschrift (freie ROM), nicht die Originalschrift des Systems

# Kontext: romref.rs, Text
media-unbekannte-revision-des-zeichensatz-roms = unbekannte Revision des Zeichensatz-ROMs

# Kontext: romref.rs, Text
media-s-byte = { $s } Byte

# Kontext: romref.rs, Text
media-in-der-revision-v = in der Revision { $v }

# Kontext: romref.rs, Text
media-zeichensatz-rom-p1-p2-what-fehlt = Zeichensatz-ROM { $p1 }/{ $p2 }{ $what } fehlt (erwartet { $size }); Datei nach { $dir } legen oder ROMs einrichten

# Kontext: romref.rs, Text
media-p1-role-enthaelt-keinen-zeichensatz-p3 = { $p1 }/{ $role } enthält keinen Zeichensatz '{ $p3 }'

# Kontext: romref.rs, Text
media-p1-zeichensatz-p2-liegt-hinter-dem = { $p1 }: Zeichensatz '{ $p2 }' liegt hinter dem Dateiende

# Kontext: romref.rs, ok_or
media-format-2-base-fehlt = Format 2: base fehlt

# Kontext: romref.rs, Err
media-zeichensatz-rom-base-profil-p2-passt = Zeichensatz-ROM '{ $base }' (Profil { $p2 }) passt nicht zum Profil { $profile }

# Kontext: romref.rs, Text
media-overrides-k-ist-keine-zeichennummer = overrides: '{ $k }' ist keine Zeichennummer

# Kontext: romref.rs, ok_or_else
media-overrides-zeichen-i-gibt-es-nicht = overrides: Zeichen { $i } gibt es nicht (Satz hat { $n })

# Kontext: romref.rs, ok_or
media-datei-ist-kein-objekt = Datei ist kein Objekt

# Kontext: romref.rs, Text
media-ersatzschrift = (Ersatzschrift)

# Kontext: romref.rs, Text
media-ersatz-eingebaut = Ersatz (eingebaut)

# Kontext: sid.rs, Err
media-keine-sid-datei-psid-rsid-erwartet = keine SID-Datei (PSID/RSID erwartet)

# Kontext: sid.rs, Err
media-sid-kopf-ungueltig-version-version-datenbeginn = SID-Kopf ungültig (Version { $version }, Datenbeginn ${ $off })

# Kontext: sid.rs, Err
media-zeitueberschreitung-bei-addr-routine-kehrt-nicht = Zeitüberschreitung bei ${ $addr } (Routine kehrt nicht zurück)

# Kontext: sid.rs, Err
media-sid-datei-ohne-daten = SID-Datei ohne Daten

# Kontext: sid.rs, Err
media-daten-ab-load-passen-nicht-in = Daten ab ${ $load } passen nicht in den Speicher

# Kontext: sid.rs, Text
media-mehrfachtempo-die-spielroutine-laeuft-calls-pro = Mehrfachtempo: die Spielroutine läuft { $calls }× pro Bild; aufgenommen wird der Stand am Bildende

# Kontext: sid.rs, Text
media-p1-sid-chips-nur-der-erste = { $p1 } SID-Chips: nur der erste wird aufgenommen

# Kontext: sid.rs, Text
media-rsid-datei-sie-erwartet-einen-echten = RSID-Datei: sie erwartet einen echten C64 (Zeitgeber, KERNAL); die Aufnahme kann unvollständig sein

# Kontext: sid.rs, Err
media-subtune-song-gibt-es-nicht-1 = Subtune { $song } gibt es nicht (1..{ $p2 })

# Kontext: sid.rs, Text
media-ntsc-stueck-60-bilder-s-im = NTSC-Stück (60 Bilder/s): im Tracker die Bildrate auf 60 Hz stellen

# Kontext: sid.rs, Text
media-filter-ringmodulation-und-synchronisation-werden-nicht = Filter, Ringmodulation und Synchronisation werden nicht übernommen

# Kontext: sid.rs, Err
media-in-der-aufnahme-sind-keine-toene = In der Aufnahme sind keine Töne zu hören (Gate nie gesetzt). Bei RSID-Dateien oder besonderen Abspielroutinen kann die Aufnahme scheitern.

# Kontext: sid.rs, Text
media-kombinierte-wellenformen-wurden-auf-eine-einzelne = Kombinierte Wellenformen wurden auf eine einzelne Wellenform abgebildet

# Kontext: sid.rs, Text
media-mehr-als-255-verschiedene-instrumente-weitere = Mehr als 255 verschiedene Instrumente: weitere Klänge nutzen das letzte Instrument

# Kontext: sid.rs, Text
media-das-stueck-laeuft-ueber-die-aufgenommenen = Das Stück läuft über die aufgenommenen { $seconds } s hinaus (Länge im Dialog erhöhen, falls mehr gewünscht)

# Kontext: song.rs, Text
media-oktave-in-n-ungueltig = Oktave in '{ $n }' ungültig

# Kontext: song.rs, Err
media-oktave-in-n-ausserhalb-0-7 = Oktave in '{ $n }' außerhalb 0..7

# Kontext: song.rs, Text
media-instrument-h-ungueltig-hex = Instrument '{ $h }' ungültig (Hex)

# Kontext: song.rs, ok_or
media-patterns-fehlt = patterns fehlt

# Kontext: song.rs, ok_or
media-pattern-pi-rows-fehlt = Pattern { $pi }: rows fehlt

# Kontext: song.rs, Err
media-pattern-pi-p2-statt-rows-zeilen = Pattern { $pi }: { $p2 } statt { $rows } Zeilen

# Kontext: song.rs, ok_or
media-pattern-pi-zeile-ri-liste-erwartet = Pattern { $pi } Zeile { $ri }: Liste erwartet

# Kontext: song.rs, Err
media-pattern-pi-zeile-ri-p3-statt = Pattern { $pi } Zeile { $ri }: { $p3 } statt { $ch } Stimmen

# Kontext: song.rs, Text
media-pattern-pi-zeile-ri-stimme-ci = Pattern { $pi } Zeile { $ri } Stimme { $ci }: { $e }

# Kontext: song.rs, Err
media-pattern-pi-zeile-ri-stimme-ci-2 = Pattern { $pi } Zeile { $ri } Stimme { $ci }: Instrument { $i } existiert nicht

# Kontext: song.rs, ok_or
media-order-fehlt = order fehlt

# Kontext: song.rs, ok_or
media-order-zahl-erwartet = order: Zahl erwartet

# Kontext: song.rs, Err
media-order-verweist-auf-pattern-v-es = order verweist auf Pattern { $v }, es gibt nur { $p2 }

# Kontext: song.rs, Err
media-order-ist-leer = order ist leer

# Kontext: song.rs, Err
media-loop-ausserhalb-der-reihenfolge = loop außerhalb der Reihenfolge

# Kontext: song.rs, Err
media-instrument-i-k-v-ausserhalb-0 = Instrument { $i }: { $k } = { $v } außerhalb 0..{ $max }

# Kontext: song.rs, ok_or
media-instrument-i-env-fehlt = Instrument { $i }: env fehlt

# Kontext: song.rs, Err
media-who-zeile-r-note-p3-liegt = { $who } Zeile { $r }: Note { $p3 } liegt unter dem POKEY-Bereich (ab C-3)

# Kontext: song.rs, Err
media-who-zeile-r-note-p3-zu = { $who } Zeile { $r }: Note { $p3 } zu hoch für RMT

# Kontext: song.rs, Err
media-rmt-export-nur-fuer-4-stimmen = RMT-Export nur für 4 Stimmen (POKEY)

# Kontext: song.rs, Err
media-rmt-1-255-zeilen-je-pattern = RMT: 1..255 Zeilen je Pattern

# Kontext: song.rs, Err
media-order-ungueltig = order ungültig

# Kontext: song.rs, ok_or
media-pattern-pi-zeile-ri-stimme-ch = Pattern { $pi } Zeile { $ri }: Stimme { $ch } fehlt

# Kontext: song.rs, Text
media-pattern-pi-zeile-ri-stimme-ch-2 = Pattern { $pi } Zeile { $ri } Stimme { $ch }: { $e }

# Kontext: song.rs, Err
media-pattern-pi-zeile-ri-stimme-ch-3 = Pattern { $pi } Zeile { $ri } Stimme { $ch }: Instrument { $i } existiert nicht

# Kontext: song.rs, Err
media-rmt-hoechstens-254-verschiedene-tracks = RMT: höchstens 254 verschiedene Tracks

# Kontext: song.rs, Text
media-erzeugt-von-vintagecoder-rmt-modul-raster = ; erzeugt von VintageCoder – RMT-Modul (Raster Music Tracker, RMT4); Start: { $label }

# Kontext: song.rs, Text
media-instrument-i-huellkurve-auf-16-bilder = Instrument { $i }: Hüllkurve auf 16 Bilder gekürzt (Attack/Decay länger als { $p2 } ms)

# Kontext: song.rs, Err
media-konvertierung-nur-fuer-songs = Konvertierung nur für Songs

# Kontext: song.rs, Err
media-quell-und-zielprofil-haben-denselben-chip = Quell- und Zielprofil haben denselben Chip

# Kontext: song.rs, Text
media-stimme-p1-entfaellt-p2-noten-ereignisse = Stimme { $p1 } entfällt ({ $p2 } Noten/Ereignisse)

# Kontext: song.rs, Text
media-shifted-noten-lagen-unter-c-3 = { $shifted } Noten lagen unter C-3 (POKEY-Bereich) und wurden oktaviert

# Kontext: song.rs, Text
media-klang-wellenform-und-adsr-sind-aus = Klang: Wellenform und ADSR sind aus der Verzerrung und Lautstärkehüllkurve geschätzt – bitte nachhören

# Kontext: song.rs, Err
media-keine-atari-binaerdatei-kopf-ffff-fehlt = keine Atari-Binärdatei (Kopf $FFFF fehlt) – das Modul braucht seine Ladeadresse

# Kontext: song.rs, Err
media-binaerdatei-beschaedigt = Binärdatei beschädigt

# Kontext: song.rs, ok_or
media-kein-rmt-modul-kennung-rmt4-am = kein RMT-Modul (Kennung 'RMT4' am Blockanfang nicht gefunden)

# Kontext: song.rs, Err
media-rmt8-stereo-8-spuren-wird-nicht = RMT8 (Stereo, 8 Spuren) wird nicht unterstützt

# Kontext: song.rs, Err
media-unbekannte-modulkennung-erwartet-rmt4 = unbekannte Modulkennung (erwartet RMT4)

# Kontext: song.rs, Err
media-tracklaenge-maxlen-wird-nicht-unterstuetzt-1 = Tracklänge { $maxlen } wird nicht unterstützt (1..128)

# Kontext: song.rs, Err
media-zeigertabellen-im-kopf-ungueltig = Zeigertabellen im Kopf ungültig

# Kontext: song.rs, Text
media-instrumentgeschwindigkeit-instrspeed-huellkurven-laufen-instrspeed-pro = Instrumentgeschwindigkeit { $instrspeed } (Hüllkurven laufen { $instrspeed }× pro Bild) wird als 1 behandelt

# Kontext: song.rs, Text
media-instrument-i-steps-huellkurvenschritte-nur-die = Instrument { $i }: { $steps } Hüllkurvenschritte, nur die ersten 16 übernommen

# Kontext: song.rs, Text
media-instrument-i-wechselnde-verzerrung-nur-die = Instrument { $i }: wechselnde Verzerrung, nur die des ersten Schritts übernommen

# Kontext: song.rs, Text
media-instrument-i-verzerrung-p2-16-bit = Instrument { $i }: Verzerrung { $p2 } (16-Bit-Bass) als „bass“ angenähert

# Kontext: song.rs, Text
media-effekte-tabellen-portamento-filter-oder-audctl = Effekte, Tabellen, Portamento, Filter oder AUDCTL in Instrumenten werden nicht übernommen

# Kontext: song.rs, Err
media-modul-enthaelt-keine-instrumente = Modul enthält keine Instrumente

# Kontext: song.rs, Err
media-track-t-instrument-ins-existiert-nicht = Track { $t }: Instrument { $ins } existiert nicht

# Kontext: song.rs, Text
media-track-t-tempowechsel-im-track-wird = Track { $t }: Tempowechsel im Track wird ignoriert

# Kontext: song.rs, Text
media-lautstaerken-unter-15-in-den-tracks = Lautstärken unter 15 in den Tracks werden nicht übernommen (der Tracker kennt keine Lautstärkespalte)

# Kontext: song.rs, Err
media-songende-sprungzeile-nicht-gefunden = Songende (Sprungzeile) nicht gefunden

# Kontext: song.rs, Err
media-songzeile-verweist-auf-einen-fehlenden-track = Songzeile verweist auf einen fehlenden Track

# Kontext: song.rs, Err
media-song-hat-keine-zeilen = Song hat keine Zeilen

# Kontext: sprite.rs, Text
media-schleife = Schleife

# Kontext: sprite.rs, Text
media-hin-und-zurueck = Hin und zurück

# Kontext: sprite.rs, Text
media-einmal = Einmal

# Kontext: sprite.rs, Err
media-animation-p1-name-name-ungueltig-buchstaben = Animation { $p1 }: Name '{ $name }' ungültig (Buchstaben, Ziffern, _; nicht mit Ziffer beginnen)

# Kontext: sprite.rs, ok_or
media-animation-name-schritt-p2-frame-fehlt = Animation '{ $name }', Schritt { $p2 }: frame fehlt

# Kontext: sprite.rs, Err
media-animation-name-schritt-p2-frame-f = Animation '{ $name }', Schritt { $p2 }: Frame { $f } gibt es nicht ({ $n } Frames)

# Kontext: sprite.rs, Err
media-animation-name-schritt-p2-dauer-d = Animation '{ $name }', Schritt { $p2 }: Dauer { $d } außerhalb 1..255 Bilder

# Kontext: sprite.rs, Text
media-bit-i-frame-i-ist-multicolor = ' Bit i = Frame i ist Multicolor (für $D01C, erste 8 Frames)

# Kontext: sprite.rs, Text
media-bit-i-frame-i-ist-multicolor-2 = ; Bit i = Frame i ist Multicolor (für $D01C, erste 8 Frames)

# Kontext: sprite.rs, Err
media-animation-p1-hat-keine-schritte = Animation '{ $p1 }' hat keine Schritte

# Kontext: spriteimport.rs, Err
media-keine-spd-datei-spd = keine SPD-Datei (SPD)

# Kontext: spriteimport.rs, Err
media-spd-kopf-unvollstaendig = SPD-Kopf unvollständig

# Kontext: spriteimport.rs, Err
media-spd-version-v-wird-nicht-unterstuetzt = SPD-Version { $v } wird nicht unterstützt (1, 3 und 5)

# Kontext: spriteimport.rs, Err
media-die-spd-datei-enthaelt-keine-sprites = die SPD-Datei enthält keine Sprites

# Kontext: spriteimport.rs, Err
media-spd-datei-zu-kurz-sprites-sprites = SPD-Datei zu kurz: { $sprites } Sprites brauchen { $p2 } Bytes ab Offset { $start }, Datei hat { $p4 }

# Kontext: spriteimport.rs, Text
media-die-sprites-haben-im-spd-format = Die Sprites haben im SPD-Format verschiedene Farben; übernommen wird die Farbe des ersten (ein Sprite-Asset hat eine Sprite-Farbe)

# Kontext: spriteimport.rs, Text
media-overlays-sprite-s-sind-im-spd = { $overlays } Sprite(s) sind im SPD-Format als Overlay markiert; die Overlay-Zuordnung wird nicht übernommen

# Kontext: spriteimport.rs, ok_or_else
media-offset-p1-liegt-hinter-dem-dateiende = Offset { $p1 } liegt hinter dem Dateiende ({ $p2 } Bytes)

# Kontext: spriteimport.rs, Err
media-ab-offset-p1-sind-weniger-als = ab Offset { $p1 } sind weniger als 64 Bytes übrig

# Kontext: spriteimport.rs, Err
media-p1-sprites-verlangt-ab-offset-p2 = { $p1 } Sprites verlangt, ab Offset { $p2 } gibt es nur { $avail } vollständige Blöcke

# Kontext: spriteimport.rs, ok_or
media-frame-i-rows-fehlt = Frame { $i }: rows fehlt

# Kontext: spriteimport.rs, Err
media-pmg-datei-hat-weniger-als-3 = PMG-Datei hat weniger als 3 Player

# Kontext: spriteimport.rs, Text
media-die-pmg-grafik-hat-h-zeilen = Die PMG-Grafik hat { $h } Zeilen; die Zeilen ab 22 fehlen im C64-Sprite

# Kontext: spriteimport.rs, Text
media-player-3-hat-pixel-ein-c64 = Player 3 hat Pixel; ein C64-Sprite ist nur 24 Pixel breit, Player 3 wird nicht übernommen

# Kontext: spriteimport.rs, Text
media-die-farben-der-player-werden-nicht = Die Farben der Player werden nicht übernommen (C64-Farben stehen im Sprite-Asset)

# Kontext: spriteimport.rs, Text
media-multicolor-sprite-die-pixel-werden-doppelt = Multicolor-Sprite: die Pixel werden doppelt breit als Hires übernommen, die Mehrfarbigkeit geht verloren (der Atari hat je Player eine Farbe)

# Kontext: spriteimport.rs, Text
media-das-pmg-profil-hat-nur-h = Das PMG-Profil hat nur { $h } Zeilen; das Sprite wird unten abgeschnitten

# Kontext: spriteimport.rs, Text
media-die-farben-werden-nicht-uebernommen-standardfarben = Die Farben werden nicht übernommen (Standardfarben der Player); die drei Player müssen im Programm nebeneinander positioniert werden

# Kontext: tiles.rs, ok_or_else
media-zeichencode-ausserhalb-0-255 = Zeichencode außerhalb 0..255

# Kontext: tiles.rs, Err
media-zelle-zahl-liste-oder-objekt-erwartet = Zelle: Zahl, Liste oder Objekt erwartet

# Kontext: tiles.rs, Err
media-kachelsatz-erwartet-kind-tileset-gefunden-p1 = Kachelsatz erwartet (kind 'tileset'), gefunden '{ $p1 }'

# Kontext: tiles.rs, Err
media-kachelgroesse-tw-th-breite-und-hoehe = Kachelgröße { $tw }×{ $th }: Breite und Höhe müssen 1..{ $max_side } sein

# Kontext: tiles.rs, Err
media-eigenschaft-n-bit-muss-0-7 = Eigenschaft '{ $n }': Bit muss 0..7 sein

# Kontext: tiles.rs, Err
media-p1-kacheln-mit-p2-bit-index = { $p1 } Kacheln: mit { $p2 }-Bit-Index sind höchstens { $max } möglich

# Kontext: tiles.rs, ok_or
media-kachel-i-cells-fehlt = Kachel { $i }: cells fehlt

# Kontext: tiles.rs, Text
media-kachel-i-e = Kachel { $i }: { $e }

# Kontext: tiles.rs, Err
media-kachel-i-zeichensatz-p2-gibt-es = Kachel { $i }: Zeichensatz { $p2 } gibt es nicht (charsets hat { $p3 })

# Kontext: tiles.rs, ok_or
media-kachel-i-props-enthaelt-keinen-namen = Kachel { $i }: props enthält keinen Namen

# Kontext: tiles.rs, ok_or
media-kachel-i-eigenschaft-n-ist-in = Kachel { $i }: Eigenschaft '{ $n }' ist in propDefs nicht definiert

# Kontext: tiles.rs, Err
media-kachelsatz-hat-keinen-zeichensatz-charsets-ist = Kachelsatz hat keinen Zeichensatz (charsets ist leer)

# Kontext: tiles.rs, Err
media-p-ist-kein-zeichensatz = { $p } ist kein Zeichensatz

# Kontext: tiles.rs, Err
media-zeichensaetze-mit-verschiedenen-profilen-prof-und = Zeichensätze mit verschiedenen Profilen ('{ $prof }' und '{ $id }') lassen sich nicht zusammenführen

# Kontext: tiles.rs, ok_or
media-p-frames-fehlt = { $p }: frames fehlt

# Kontext: tiles.rs, ok_or
media-zeichensatz-cs-gibt-es-nicht = Zeichensatz { $cs } gibt es nicht

# Kontext: tiles.rs, ok_or
media-zeichensatz-cs-hat-kein-zeichen-code = Zeichensatz { $cs } hat kein Zeichen { $code }

# Kontext: tiles.rs, Err
media-pin-code-ziel-hat-nur-max = Pin { $code }: Ziel hat nur { $max } Zeichen

# Kontext: tiles.rs, Err
media-pin-konflikt-bei-code-code-zwei = Pin-Konflikt bei Code { $code }: zwei verschiedene Glyphen

# Kontext: tiles.rs, Err
media-zeichensatz-ueberlauf-mehr-als-max-verschiedene = Zeichensatz-Überlauf: mehr als { $max } verschiedene Glyphen (Ziel fasst { $max })

# Kontext: tiles.rs, ok_or
media-zeichen-p1-p2-fehlt-in-der = Zeichen { $p1 }:{ $p2 } fehlt in der Zusammenführung

# Kontext: tiles.rs, Err
media-farbwahl-bit-sel-braucht-einen-zielzeichensatz = Farbwahl-Bit (sel) braucht einen Zielzeichensatz mit höchstens 128 Zeichen (Atari)

# Kontext: tiles.rs, Err
media-screen-bank-erwartet-kind-screen-mit = Screen-Bank erwartet (kind 'screen' mit 'screens')

# Kontext: tiles.rs, Err
media-screen-i-name-groesse-w-h = Screen { $i } '{ $name }': Größe { $w }×{ $h } ungültig (1..255)

# Kontext: tiles.rs, Err
media-screen-i-name-kartenzeilen-verschieden-lang = Screen { $i } '{ $name }': Kartenzeilen verschieden lang ({ $p3 } und { $p4 })

# Kontext: tiles.rs, ok_or
media-screen-i-patch-ist-keine-liste = Screen { $i }: Patch ist keine Liste [x, y, zelle]

# Kontext: tiles.rs, Err
media-screen-i-name-patch-x-y = Screen { $i } '{ $name }': Patch ({ $x }, { $y }) liegt außerhalb { $w }×{ $h }

# Kontext: tiles.rs, Err
media-screen-i-p2-kachelraster-cols-rows = Screen { $i } '{ $p2 }': Kachelraster { $cols }×{ $rows } ab ({ $p5 }, { $p6 }) mit { $p7 }×{ $p8 }-Kacheln passt nicht in { $p9 }×{ $p10 } Zellen

# Kontext: tiles.rs, Err
media-screen-i-p2-karte-x-y = Screen { $i } '{ $p2 }': Karte ({ $x }, { $y }) verweist auf Kachel { $t }, der Satz hat nur { $p6 }

# Kontext: tiles.rs, Err
media-rle-daten-paar-unvollstaendig-oder-anzahl = RLE-Daten: Paar unvollständig oder Anzahl 0

# Kontext: tiles.rs, Text
media-p1-kacheln-p2-p3-zeichensatz-fuellstand = { $p1 } Kacheln { $p2 }×{ $p3 }, Zeichensatz-Füllstand { $p4 } von { $p5 } Glyphen, { $p6 } Doppelte zusammengelegt

# Kontext: tiles.rs, Err
media-screen-bank-braucht-mindestens-einen-kachelsatz = Screen-Bank braucht mindestens einen Kachelsatz in 'tilesets'

# Kontext: tiles.rs, Err
media-kachelsatz-rel-zeichensatzprofil-id-passt-nicht = Kachelsatz { $rel }: Zeichensatzprofil '{ $id }' passt nicht zu '{ $cpid }' der übrigen Sätze

# Kontext: tiles.rs, Err
media-screen-p1-kachelsatz-p2-gibt-es = Screen '{ $p1 }': Kachelsatz { $p2 } gibt es nicht (die Bank hat { $p3 })

# Kontext: tiles.rs, Err
media-screen-p1-kachel-t-passt-nicht = Screen '{ $p1 }': Kachel { $t } passt nicht in einen 8-Bit-Index (höchstens 256 Kacheln); 16-Bit-Index im Layout wählen

# Kontext: tiles.rs, Err
media-screen-p1-mehr-als-255-zeilen = Screen '{ $p1 }': mehr als 255 Zeilen mit eigenen Zellen

# Kontext: tiles.rs, Text
media-p1-screens-zeichensatz-fuellstand-p2-von = { $p1 } Screens, Zeichensatz-Füllstand { $p2 } von { $p3 } Glyphen{ $p4 }

# Kontext: wizard.rs, ok_or
media-systems-fehlt = systems fehlt

# Kontext: wizard.rs, Err
media-projektname-nur-buchstaben-ziffern-und = Projektname: nur Buchstaben, Ziffern, _ und -

# Kontext: wizard.rs, Err
media-mindestens-ein-zielsystem-waehlen = mindestens ein Zielsystem wählen

# Kontext: wizard.rs, ok_or_else
media-vorlage-template-hat-keine-variante-v = Vorlage '{ $template }' hat keine Variante '{ $v }'

# Kontext: wizard.rs, Err
media-vorlage-label-gibt-es-nicht-fuer = Vorlage '{ $label }' gibt es nicht für '{ $s }' (möglich: { $p3 })

# Kontext: wizard.rs, Err
media-p1-enthaelt-schon-ein-projekt = { $p1 } enthält schon ein Projekt

# Kontext: wizard.rs, ok_or
media-from-fehlt = from fehlt

# Kontext: wizard.rs, ok_or
media-to-fehlt = to fehlt

# Kontext: wizard.rs, Err
media-vorlage-unzulaessiger-zielpfad-to = Vorlage: unzulässiger Zielpfad '{ $to }'

# Kontext: project.rs, rom_candidates
media-rom-c64-zeichensatz = C64: Zeichensatz

# Kontext: project.rs, rom_candidates
media-rom-mega65-vollfarbe = MEGA65 (Vollfarbe)

# Kontext: project.rs, rom_candidates
media-rom-c128-zeichen-rom-40-spalten-zeichensatz-satz-1 = C128 (Zeichen-ROM): 40-Spalten-Zeichensatz Satz 1

# Kontext: project.rs, rom_candidates
media-rom-c128-zeichen-rom-40-spalten-zeichensatz-satz-2 = C128 (Zeichen-ROM): 40-Spalten-Zeichensatz Satz 2

# Kontext: project.rs, rom_candidates
media-rom-atari-zeichensatz = Atari: Zeichensatz

# Kontext: goat.rs, Befehlsname 1
media-goat-befehl-1 = Portamento aufwärts

# Kontext: goat.rs, Befehlsname 2
media-goat-befehl-2 = Portamento abwärts

# Kontext: goat.rs, Befehlsname 3
media-goat-befehl-3 = Tonportamento

# Kontext: goat.rs, Befehlsname 4
media-goat-befehl-4 = Vibrato

# Kontext: goat.rs, Befehlsname 5
media-goat-befehl-5 = AD setzen

# Kontext: goat.rs, Befehlsname 6
media-goat-befehl-6 = SR setzen

# Kontext: goat.rs, Befehlsname 7
media-goat-befehl-7 = Wellenform setzen

# Kontext: goat.rs, Befehlsname 8
media-goat-befehl-8 = Wellentabellen-Zeiger

# Kontext: goat.rs, Befehlsname 9
media-goat-befehl-9 = Pulstabellen-Zeiger

# Kontext: goat.rs, Befehlsname 10
media-goat-befehl-10 = Filtertabellen-Zeiger

# Kontext: goat.rs, Befehlsname 11
media-goat-befehl-11 = Filtersteuerung

# Kontext: goat.rs, Befehlsname 12
media-goat-befehl-12 = Filter-Grenzfrequenz

# Kontext: goat.rs, Befehlsname 13
media-goat-befehl-13 = Hauptlautstärke

# Kontext: goat.rs, Befehlsname 14
media-goat-befehl-14 = Funktempo

# Kontext: lib.rs, Name der ersten (leeren) Kachel in neuen Dateien
media-leer = leer

# Kontext: project.rs, Warnung beim Import
# $mode: Multicolor oder Hires
media-rohdaten-ohne-modusangabe-gelesen-als = Rohdaten ohne Modusangabe: gelesen als { $mode } (Modus des Zielprofils)

# Kontext: romref.rs, Trenner in Aufzählungen von Größen
media-oder = oder

# Kontext: romref.rs, Quelle einer fehlenden ROM-Datei
media-fehlt = fehlt

# Kontext: song.rs, Name eines unbenutzten Instruments
# $i: Nummer (hex) | Beispiel: 0A
media-leer-i = Leer { $i }

# Kontext: sid.rs, Warnung beim SID-Import
# $title: Titel | Beispiel: Commando
# $author: Autor | Beispiel: Rob Hubbard
# $song: Subtune | Beispiel: 1
# $seconds: Sekunden | Beispiel: 30
# $onsets: Anzahl Noteneinsätze | Beispiel: 420
# $speed: Bilder je Zeile | Beispiel: 6
media-aus-aufgenommen-subtune-song-seconds = Aus „{ $title }“ ({ $author }) aufgenommen: Subtune { $song }, { $seconds } s, { $onsets } Noteneinsätze, Takt { $speed } Bilder/Zeile. Das ist eine Annäherung an den Klang, nicht der Originalsong.

# Kontext: sprite.rs, Kopfkommentar der erzeugten Tabellendatei (zwei Zeilen, Zeilenende folgt im Code)
media-kopf-sprite-tabellen =
    ; erzeugt von VintageCoder – nicht von Hand ändern
    ; Tabellen zu den Sprite-Daten; diese Datei gehört hinter die Sprite-Daten (sie belegt eigenen Speicher)

# Kontext: wizard.rs, Inhalt der HERKUNFT.txt (mehrzeilig; Zeilenende nach der letzten Zeile folgt im Code)
media-herkunft-text =
    Herkunft der Zeichensätze in diesem Ordner
    ===========================================

    Diese Dateien sind die eigenen Standardzeichensätze „VC-Standard“ von VintageCoder.
    Urheber: das VintageCoder-Projekt. Lizenz: CC0-1.0 (gemeinfrei, frei weitergebbar, auch in eigenen Programmen).
    Es wurden keine Original-ROMs und keine Nachbauten von Original-Zeichen benutzt.
    Die Gestaltung entstand aus den Regeln des Stilblatts und ist vom Projektinhaber freigegeben. Bei einfachen Formen in einem 8x8-Raster (Klammern, Punkt, Komma, Strich, Doppelpunkt, I-Familie, Linien- und Rahmenzeichen) sind Übereinstimmungen mit anderen Zeichensätzen funktional bedingt; eine lokale Abstandsprüfung ergab bei wenigen solcher Zeichen kleine Abstände, der Projektinhaber hat die Sätze trotzdem freigegeben. Das ist keine Rechtsaussage.

    Sätze:

# Kontext: Export-Vorschau: gekürzte Mitte eines langen Textes (Kommentarzeile)
export-vorschau-gekuerzt = … { $n } Zeilen ausgelassen (die Datei enthält sie)
# I3b
# $subtunes: Anzahl Subtunes | Beispiel: 40
media-goat-subtune-zahl-unplausibel = Unplausible Subtune-Zahl { $subtunes }
# $n_ins: Anzahl Instrumente | Beispiel: 70
media-goat-instrumentzahl-unplausibel = Unplausible Instrumentzahl { $n_ins }
# $i: Nummer der Kachel | Beispiel: 3
# $p1: gefundene Zeichen | Beispiel: 3
# $p2: erwartete Zeichen | Beispiel: 4
media-kachel-i-zeichen-anzahl = Kachel { $i }: { $p1 } statt { $p2 } Zeichen
# $i: Nummer des Eintrags | Beispiel: 17
media-palette-eintrag-rrggbb-erwartet = Eintrag { $i }: '#rrggbb' erwartet
# $o: angegebene Wellenform | Beispiel: sinus
media-wellenform-unbekannt = Wellenform '{ $o }' unbekannt (triangle, saw, pulse, noise)
# $rel: Pfad der Datei | Beispiel: graphics/c64_upper.vcchar
media-datei-existiert-bereits = { $rel } existiert bereits
# $text: der Verweis | Beispiel: rom:c64
media-text-system-rolle-erwartet = '{ $text }': system/rolle erwartet
# $system: Name des Systems | Beispiel: C64
# $role: Name der Rolle | Beispiel: Zeichensatz
media-rom-unbekannte-revision = { $system } { $role } (unbekannte Revision)
# $n: angegebene Note | Beispiel: H-4
media-note-unbekannt = Note '{ $n }' unbekannt
# $o: angegebene Verzerrung | Beispiel: fuzz
media-verzerrung-unbekannt = Verzerrung '{ $o }' unbekannt (pure, noise, buzz, gritty, bass)
# $max_ins: größte Anzahl | Beispiel: 32
media-instrumente-anzahl-spielroutine = 1..{ $max_ins } Instrumente erwartet (Spielroutine)
# $i: Nummer des Instruments | Beispiel: 2
# $d: angegebene Verzerrung | Beispiel: fuzz
media-instrument-i-verzerrung-unbekannt = Instrument { $i }: Verzerrung '{ $d }' unbekannt
# $i: Nummer des Frames | Beispiel: 0
# $p1: gefundene Breite | Beispiel: 23
# $p2: gefundene Höhe | Beispiel: 21
# $w: erwartete Breite | Beispiel: 24
media-frame-i-pixelgroesse = Frame { $i }: { $p1 }×{ $p2 } statt { $w }×21 Pixel
# $m: angegebener Wert | Beispiel: row
media-colormode-unbekannt = colorMode '{ $m }' unbekannt (tile, cell, none)
# $i: Nummer der Kachel | Beispiel: 3
# $p1: gefundene Zellen | Beispiel: 3
# $p2: erwartete Zellen | Beispiel: 4
media-kachel-i-zellen-anzahl = Kachel { $i }: { $p1 } statt { $p2 } Zellen
# $p: Pfad des Zeichensatzes | Beispiel: graphics/a.vcchar
# $e: Fehlertext | Beispiel: No such file
media-zeichensatz-p-e = Zeichensatz { $p }: { $e }
# $p1: ergibt | Beispiel: 10
# $len: erwartet | Beispiel: 12
media-rle-daten-ergeben-anzahl = RLE-Daten ergeben { $p1 } statt { $len } Kacheln
media-alte-kachelkarte-erwartet = alte Kachelkarte (kind 'tilemap') erwartet
# $template: Name der Vorlage | Beispiel: asm-hello
media-wizard-vorlage-unbekannt = Vorlage '{ $template }' unbekannt
# $s: Kennung des Zielsystems | Beispiel: c65
media-wizard-zielsystem-unbekannt = Zielsystem '{ $s }' unbekannt
# $name: Name der Vorlage | Beispiel: Hallo-Welt
# $systems: Zielsysteme, durch Komma getrennt | Beispiel: c64, vic20
media-wizard-projektbeschreibung = Vorlage '{ $name }' ({ $systems })

# Kontext: include.rs, Fehlermeldung beim Einbinden eines Mediums (Assembler/BASIC)
# $p1: Pfad der Mediendatei | Beispiel: graphics/hero.vcsprite
media-include-medium-nicht-gefunden = Medium nicht gefunden: { $p1 }

# Kontext: include.rs, Fehlermeldung beim Einbinden eines Mediums
# $p1: Pfad der Mediendatei | Beispiel: graphics/hero.vcsprite
# $p2: Meldung des Lesers | Beispiel: Profil fehlt
media-include-medium-nicht-lesbar = Medium { $p1 } nicht lesbar: { $p2 }

# Kontext: include.rs, Fehlermeldung beim Einbinden eines Mediums
# $p1: Pfad der Mediendatei | Beispiel: graphics/hero.vcsprite
# $p2: Zielsystem des Profils | Beispiel: c64
# $p3: Ziele des Baus, durch Komma getrennt | Beispiel: atari8
media-include-profil-passt-nicht-zum-ziel = Medium { $p1 } ist für { $p2 }, das Ziel des Baus ist { $p3 }

# Kontext: include.rs, Fehlermeldung beim Einbinden eines Mediums
# $p1: Pfad der Mediendatei | Beispiel: graphics/hero.vcsprite
# $p2: Meldung des Exporters | Beispiel: PLAY-Song: Exportformat 'bas' verwenden
media-include-medium-nicht-exportierbar = Medium { $p1 } lässt sich nicht einbinden: { $p2 }

# Kontext: vcc migrate includes, Hinweis zu einer Zeile, die nicht umgestellt wird
media-include-migrate-anim-allein = Animationstabellen allein eingebunden: bleibt unverändert (der Include der Mediendatei liefert Daten und Tabellen zusammen)

# Kontext: vcc migrate includes, Hinweis zu einer Zeile, die nicht umgestellt wird
# $p1: Name des Mediums aus dem alten Pfad | Beispiel: hero
media-include-migrate-nicht-eindeutig = Kein eindeutiges Medium mit dem Namen '{ $p1 }' im Projekt: bleibt unverändert
media-png-daten-zu-gross = PNG: Entpackte Daten sind zu groß (Dekompressionsbombe?).
media-png-abgeschnitten = PNG-Datei ist abgeschnitten.
media-png-pruefsumme = PNG: Prüfsummenfehler im Block { $block }.
media-png-ihdr-zu-kurz = PNG: Der Block IHDR ist zu kurz.
