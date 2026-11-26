# Glossar Deutsch ↔ Englisch (Oberfläche)

Stand: 2026-11-24 (Begriffe der neuen Oberfläche nachgeführt). Maßgeblich für `crates/vc-i18n/catalog/en/*.ftl`; Entscheidungen zu Zweifelsfällen stehen unten und sind dem Inhaber gemeldet. Korrekturen aus dem Prüfblock fließen hierher zurück.

**Stil (Englisch):** einfache Sätze, „you“, Imperativ in Anleitungen, keine Redewendungen. Knöpfe und Menüeinträge im Imperativ („Save“, „Open file …“), Titel in Satzschreibweise („Color scheme“). Eigennamen und feste Begriffe bleiben unübersetzt (Liste: `tools/i18n_names.txt`, unten).

## Feste Regeln

| Deutsch | Englisch | Hinweis |
|---|---|---|
| Zeropage | zero page | Deutsch nie „Nullseite“; Englisch „zero page“, in Zusammensetzungen „zero-page addressing“ (E4) |
| Onion Skin | Onion Skin | **nie übersetzen**, in beiden Sprachen so geschrieben |
| Zusatzfunktion | additional feature | das Wort mit „Kil…ler“ nie verwenden |
| VC-20 | VIC-20 | Entscheidung des Inhabers 2026-11-21: Deutsch „VC-20“, Englisch „VIC-20“ (Chip „VIC“ und „VIC-II“ sind etwas anderes); `tools/check_schreibweise.py` meldet die falsche Schreibweise je Sprache |

## Begriffe

| Deutsch | Englisch | Hinweis |
|---|---|---|
| Befehlspalette | command palette | |
| Ablage unter dem Editor (früher Schublade) | tray below the editor (short: tray) | Bereich unten: Probleme, Ausgabe, Tests, Suche, Gliederung; das Wort „drawer“ entfällt |
| Seitenleiste (früher Linke Spalte) | sidebar | Projektbaum; „left column“ entfällt |
| Panel Maschine (früher Rechtes Panel) | machine panel (title „Machine“) | Startziel, Zustand, Ergebnis des Baus; „right panel“ entfällt |
| Hauptbereiche | main sections | Project, Code, Graphics, Music, Data, Debug (Projekt, Code, Grafik, Musik, Daten, Debug) |
| Startseite (früher Startbild) | start page | |
| Dokumentenfeld | document area | |
| ~~Studio~~ (entfällt) | ~~studio~~ | Entscheidung 2026-11-20: Wort kollidiert mit „C64 Studio“. Deutsch „Sprite-Editor“, „Musik-Editor“, „Zeichensatz-Editor“, „Kachel-Editor“ usw.; in der Navigation nur „Sprites“, „Tracker“, „Zeichensatz“, „Disk“, „Speicher“; Code-Bereich „Code“. Englisch „Sprite editor“, „Music editor“ usw. Quelltextnamen (`bankstudio.rs`) bleiben vorerst. Die Katalogtexte sind noch nicht umgestellt (siehe Bericht). |
| Zeichensatz | character set | kurz „charset“ nur in Bezeichnern |
| Kachel / Kachelkarte | tile / tile map | |
| Haltepunkt | breakpoint | |
| Neuplatzierung (`.relocate`) | relocation | Code für eine andere Laufzeitadresse übersetzen; die Direktivennamen selbst werden nie übersetzt |
| Rücksprungadresse (`.retaddr`) | return address | Adresse minus 1 für RTS-Sprungtabellen |
| Adresstabelle aus Low- und High-Bytes (`.lohitable`) | table of low and high bytes | erst alle Low-Bytes, dann alle High-Bytes |
| Füllbyte | fill byte | `.align n, $ea` |
| Startvektor | start vector | Atari: RUNAD; W205–W207 |
| Bauen / Projekt bauen | build / Build project | F7 |
| Starten (Startprogramm starten) | run (Run start program) | „run“, nicht „start“, damit Knopf und Befehl gleich heißen |
| Debuggen | debug | |
| Schritt über / hinein / hinaus | step over / into / out | |
| Anhalten | pause | im Debugger |
| Ausgabe (früher Protokoll) | output | Reiter der Ablage |
| Probleme | problems | Reiter der Ablage |
| Gliederung | outline | Panel-Reiter |
| Karte (Speicherkarte) | memory map | Panel-Reiter, zeigt die Speicherbelegung |
| Dateibaum / Projektbaum | file tree | in beiden Fällen „file tree“ |
| Zielsystem | target system | |
| Einrichtung / Konfigurationsassistent | setup / setup assistant | |
| Ersteinrichtung | first-run setup | |
| Startseite | start page | |
| Steckmodul | cartridge | |
| Diskettenabbild | disk image | |
| Rasterzeile | raster line | |
| Bank | bank | |
| Oberfläche (Größe der Oberfläche) | UI size | Einstellung für die Größe: „UI size“; im Fließtext „user interface“ |
| Farbschema (Dunkel, Hell, System, Hoher Kontrast) | color scheme (Dark, Light, System, High contrast) | amerikanische Schreibung „color“ |
| Zeilenumbruch (Editor) | word wrap | |
| Reiter | tab | |
| Papierkorb | trash | |
| Zen-Modus | Zen mode | |
| Zahlenwerkzeuge | number tools | |
| Schreibweise „Strg“ | Ctrl | Tastennamen in Kürzeln: Strg → Ctrl, Umschalt → Shift, Links/Rechts → Left/Right |

## Zweifelsfälle und Entscheidungen (vom Inhaber bestätigt, 2026-11-19)

1. **„Starten“ → „Run“** statt „Start“: Knopf „▶ Run“, Befehl „Run start program“ (das „Startprogramm“ der Projektdatei heißt „start program“).
2. **„Karte“ → „Memory map“** (Entscheidung des Inhabers; Breite des Panel-Reiters mit xx prüfen): die Seite zeigt, wo das Programm im Speicher liegt („Nothing built yet: after building, the map shows …“).
3. **„Oberfläche“ → „UI size“** (Entscheidung des Inhabers) (Einstellung für die Größe; im Plural „user interface“ nur im Fließtext).
4. **„Gliederung“ → „Outline“** (wie in anderen Editoren).
5. **„Hinein / Hinaus“ → „Step into / Step out“**: die deutschen Knöpfe sind verkürzt, englisch steht das Verb davor.
6. **„Anhalten“ → „Pause“** (Debuggen), „Beenden“ → „Stop“.
7. **Kapitälchen-Überschriften (bestätigt)** („REGISTER“, „SPEICHER“, „BEOBACHTEN“ …) bleiben Großbuchstaben: REGISTERS, MEMORY, WATCH.

## Eigennamen und Begriffe, die in beiden Sprachen gleich bleiben

VintageCoder, VCASM, Onion Skin, BASIC, Assembler (bleibt „Assembler“), Markdown, Tracker, Sprite, MEGA65, C64, Atari, VICE, Xemu, PAL/NTSC, PRG/D64/CRT (Dateiformate).

## Schreibweise (2026-11-24)

Amerikanisches Englisch: „color“, nicht „colour“ (Katalog, Handbuch, Datenüberlagerungen angeglichen).
