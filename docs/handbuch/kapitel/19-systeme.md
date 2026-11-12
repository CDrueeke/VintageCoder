---
id: systeme
titel: Zielsysteme im Vergleich
reihenfolge: 19
stufe: Einsteiger
zeit: 20 Minuten
siehe_auch: [willkommen, erstes-projekt, debugger, tests, grafik-ueberblick]
referenz: [overview, c128-plus4, vic20, mega65, mega65-hardware, roms, settings]
---

# Zielsysteme im Vergleich

**Ziel:** Du erfährst, was die einzelnen Rechner voneinander unterscheidet, und kannst dir das System aussuchen, für das du programmieren willst. Zu jedem System findest du ein Beispielprojekt zum Ausprobieren.

Ein **Zielsystem** ist der Rechner, für den dein Programm gebaut wird. Du wählst es im [Projekt-Wizard](kapitel:erstes-projekt) aus, und es steht in der Projektdatei. Die Befehlsleiste zeigt es im Feld **Auf:**. Mit `.target` im Quelltext legst du es für eine Datei fest (zum Beispiel `.target c64`).

## Das brauchst du

- VintageCoder mit eingerichteten Emulatoren (siehe [Willkommen](kapitel:willkommen)). Für manche Systeme brauchst du außerdem ROM-Dateien (siehe unten und die Hilfe „ROM-Dateien“).
- Etwas Neugier: Alle Systeme hier haben einen 6502-Prozessor oder einen nahen Verwandten, daher ähnelt sich vieles.

## Die Systeme auf einen Blick

| System | Kennung | Prozessor | Grafik | Klang | Emulator in der IDE | Debugger |
|---|---|---|---|---|---|---|
| Commodore 64 | `c64` | 6510 | VIC-II | SID | VICE (x64sc, x64) | ja, mehrere Haltepunkte |
| Commodore 128 | `c128` | 8502 (C128-Modus) | VIC-IIe, VDC | SID | VICE (x128) | ja |
| Plus/4 | `plus4` | 7501/8501 | TED | TED | VICE (xplus4) | ja |
| C16 / C116 | `c16` | 7501 | TED | TED | VICE (xplus4, Modell c16) | ja |
| VC-20 (VIC-20) | `vic20` | 6502 | VIC-I | VIC-I | VICE (xvic) | ja |
| Atari 800 XL / XE | `atari8` | 6502C (Sally) | ANTIC, GTIA | POKEY | atari800, Altirra | atari800: ein Haltepunkt |
| MEGA65 | `mega65` | 45GS02 | VIC-IV | vier SIDs | Xemu | ein Haltepunkt |

Für alle gilt: Baue mit **F7**, starte mit **F5**, debugge mit **F6**. Welcher Emulator startet, steht in den Einstellungen unter **Emulatoren**; das Feld **Auf:** wählt statt des Standards eine andere Maschine (zum Beispiel eine Ultimate 64).

### Takt, Zeilen, Zyklen

Wer Rasterprogramme schreibt, braucht diese Werte (je Norm, aus den Zieldaten):

| System | Norm | Takt | Zeilen je Bild | Zyklen je Zeile |
|---|---|---|---|---|
| C64 / C128 | PAL | 985 248 Hz | 312 | 63 |
| C64 / C128 | NTSC | 1 022 727 Hz | 263 | 65 |
| Plus/4 | PAL | 886 724 Hz | 312 | 57 |
| VC-20 | PAL | 1 108 405 Hz | 312 | 71 |
| Atari 8-Bit | PAL | 1 773 447 Hz | 312 | 114 |
| Atari 8-Bit | NTSC | 1 789 773 Hz | 262 | 114 |
| MEGA65 | PAL | 40 500 000 Hz | 625 | 810 |

## Commodore 64 (`c64`)

Der Klassiker. Programme starten mit einem kleinen BASIC-Vorspann (`.basicstub`) bei `$0801`.

- **Speicher:** 64 KB RAM. RAM von `$0000` bis `$9FFF`, BASIC-ROM bei `$A000`, freier Bereich `$C000` bis `$CFFF`, E/A bei `$D000` bis `$DFFF`, KERNAL-ROM bei `$E000`. Der Bildschirm liegt standardmäßig bei `$0400`. Die Speicherumschaltung geschieht im Prozessorport `$0001`; die Memory Map zeigt dazu Presets (zum Beispiel „Alles RAM“).
- **Grafik:** VIC-II mit 8 Sprites, Textmodus und Bitmap. In der IDE gibt es dafür den Sprite-, Zeichensatz-, Bildschirm- und Kachel-Editor.
- **Klang:** SID, im Tracker mit drei Stimmen.
- **Emulator:** VICE, wahlweise **x64sc** (taktgenau) oder **x64** (schnell).
- **Debugger:** voller Umfang (mehrere Haltepunkte, Datenhaltepunkte, Emulator-Bild, Momentaufnahme).
- **Tests:** Zyklenmessung und Standardadresse der Falle gibt es bisher nur hier; Postfach `$033C`.
- **Echte Hardware:** Die **Ultimate 64 / II+** lässt sich über das Netzwerk anbinden (Starten, Speicher lesen, Live-Bild (in Erprobung, am echten Gerät noch nicht geprüft), Postfach-Tests, Zyklenmessung auf der Ultimate 64).
- **Beispiel:** `examples/c64/border` (Rahmenfarbe), `examples/c64/sprite_demo` (Sprites), `examples/c64/rasterbars` (Rasterbalken), `examples/c64/prallball`.

![Das Beispielprojekt border im Editor des C64](bild:system-vergleich-c64 "Ein C64-Projekt mit Quelltext, Projektbaum, Emulator-Bild und Debugger. ①: Zielsystem in der Befehlsleiste. ②: das Beispielprogramm. ③: das Bild des Emulators.")

## Commodore 128 (`c128`)

Der C128 startet in dieser IDE im **C128-Modus** mit CPU 8502 (derselbe Befehlssatz wie der 6510). Willst du den C64-Modus, nimmst du das Zielsystem `c64`. Den Z80 gibt es nicht.

- **BASIC:** BASIC 7.0 (Dialekt `basic7`); Programme beginnen bei `$1C01`. `.basicstub` erzeugt bei `.org $1c01` den `SYS 7181`.
- **Chips:** VIC-IIe (mit `$D02F` Tastatur und `$D030` 2-MHz-Modus), SID, zwei CIA, die Speichersteuerung MMU (`$D500`, `$FF00`) und der **VDC** für den 80-Spalten-Bildschirm (`$D600`, `$D601`).
- **Grafik und Klang:** Die Profile des C64 (Sprites, Zeichensätze, Bildschirme, Kacheln, Tracker) gibt es auch im C128-Projekt. VDC-Grafik und eine MMU-Hilfe im Assembler gibt es noch nicht.
- **Emulator:** VICE **x128**; der Debugger spricht dessen Binär-Monitor. Im Emulator-Reiter wählst du zwischen **VIC-II** (40 Spalten) und **VDC** (80 Spalten).
- **Tests:** Postfach bei `$0B00`.
- **Disketten:** D64, D71 und D81.
- **Beispiel:** `examples/c128/border`, `examples/c128/prallball`, `examples/c128/prallball_vdc`.

## C16 und Plus/4 (`c16`, `plus4`)

- **BASIC:** BASIC 3.5 (Dialekt `basic35`); Programme beginnen bei `$1001`. `.basicstub` erzeugt bei `.org $1001` den `SYS 4109`.
- **Chip:** Der **TED** macht Bild und Ton in einem. Er hat **keine Sprites**, und es gibt keinen SID.
- **Grafik und Klang:** Es gibt noch keine Editor-Profile für Sprites, Zeichensätze oder Songs dieses Systems; der „Neu …“-Dialog meldet das. Dein Programm schreibst du also direkt gegen die TED-Register.
- **Speicher:** Farb-RAM bei `$0800`, Bildschirm bei `$0C00`. Der **C16/C116** (Target `c16`) hat dieselbe Hardware, aber nur 16 KB RAM (Anwenderspeicher bis `$3FFF`). Wähle beim Neuen Projekt „Commodore 16 / 116“, dann startet F5 den Emulator als C16 (`-model c16`). Ein C16-Programm läuft auch auf dem Plus/4.
- **Emulator:** VICE **xplus4**.
- **Beispiel:** `examples/plus4/border`, `examples/plus4/prallball`, für den C16 `examples/c16/border`.

## VC-20 (`vic20`)

In Deutschland hieß der VIC-20 „VC-20“.

- **Chips:** VIC-I (Bild und Ton, Rahmen und Hintergrund in `$900F`), zwei VIA.
- **BASIC:** BASIC V2.
- **Speicherausbau:** Das ist die Besonderheit dieses Systems. Der Ausbau steht in der Projektdatei; im Wizard und in den Projekt-Einstellungen wählst du ihn. Davon hängen BASIC-Anfang, Bildschirm und Farb-RAM ab:

| Ausbau | BASIC ab | Bildschirm | Farb-RAM |
|---|---|---|---|
| Grundgerät (5 KB) | `$1001` | `$1E00` | `$9600` |
| mit 3 KB | `$0401` | `$1E00` | `$9600` |
| 8, 16, 24 oder 35 KB | `$1201` | `$1000` | `$9400` |

  Schreibe dein Programm deshalb mit `.org BASIC_START` statt mit einer festen Adresse, damit es sich dem Ausbau anpasst. Die Systemdatei `#include <vic20/hardware.asm>` liefert `SCREEN_RAM`, `COLOR_RAM` und `BASIC_START`.
- **Grafik und Klang:** Editoren für Zeichensätze (Hires, Multicolor), den Bildschirm (22×23 Zeichen) und Kacheln (11×11). Der Tracker hat vier Stimmen. Der Chip hat nur **eine gemeinsame Lautstärke**; Hüllkurven schalten die Stimmen deshalb nur ein oder aus.
- **Steckmodule:** Mit `.cartridge "a0", "NAME"` entsteht ein rohes 8-KB-Abbild (nur ohne Bänke).
- **Emulator:** VICE **xvic**, mit dem passenden `-memory`-Argument.
- **Beispiel:** `examples/vic20/prallball`, `examples/vic20/tiles_demo`, `examples/vic20/music_demo`.

## Atari 800 XL / XE (`atari8`)

Der Atari ist **kein Commodore**: Chips, Speicher und Programmformat sind anders, nur der Prozessor ist verwandt.

- **Chips:** **ANTIC** (Anzeigelisten) und **GTIA** (Farben, Player/Missiles) für das Bild, **POKEY** für den Ton, **PIA** für die Speicherumschaltung.
- **Grafik:** Der ANTIC arbeitet mit einer **Anzeigeliste**; die Momentaufnahme im Debugger liest sie und die Farb-Schattenregister aus (nur ANTIC-Modus 2 und kein Bitmap). In der IDE gibt es einen Editor für Player/Missiles (PMG), für Zeichensätze, Bildschirme und Kacheln. Bei den Atari-Editoren wählst du rechts die Videonorm PAL oder NTSC.
- **Klang:** POKEY, im Tracker.
- **Programmformat:** Ausgabe als **XEX** (Standard), ATR (Diskettenabbild) oder BIN. Anders als beim C64 gibt es keinen BASIC-Vorspann: Du legst den Startpunkt mit `.run start` fest, zum Beispiel `.org $2000` … `.run start`.
- **BASIC:** Atari BASIC oder Turbo BASIC XL. Die Dateien entstehen im SAVE-Format, das du mit `LOAD` oder `RUN` lädst.
- **Modelle:** 800 XL (64 KB), 130 XE (128 KB mit Bankumschaltung über PORTB `$D301`) und 800 (48 KB, anderes Betriebssystem, kein BASIC-ROM).
- **Speicher:** Seite 6 (`$0600` bis `$06FF`) ist für Anwender frei; im Betriebssystem belegen `$00` bis `$7F` die Zeropage (BASIC und OS), `$80` bis `$FF` sind ohne BASIC frei. Beim Atari 800 XL/XE gibt es Bank-Presets in der Memory Map.
- **Betriebssystem:** Für Zeichensätze aus dem ROM brauchst du das Betriebssystem-ROM (siehe Hilfe „ROM-Dateien“).
- **Emulator und Debugger:** **atari800** oder **Altirra**. Zum Debuggen dient atari800; er kennt **nur einen** Haltepunkt zugleich, keine Datenhaltepunkte und lässt sich nicht per Knopf anhalten (**F8 im Emulatorfenster**). Der Emulator-Reiter liefert für den Atari kein Bild.
- **Tests:** Postfach bei `$0600`.
- **Beispiel:** `examples/atari/colorbars` (Rasterbalken mit `WSYNC`), `examples/atari/pmg_ship`, `examples/atari/prallball`, `examples/atari/music_demo`.

## MEGA65 (`mega65`)

Ein moderner Nachbau im C65-/C64-Stil mit 40 MHz.

- **CPU:** 45GS02, eine Erweiterung des 65CE02 um 32-Bit-Zeiger und das Q-Register. Neue Befehle sind `ldz`, `phw`, `lbra`, `ldq` und mehr.
- **BASIC:** BASIC 65; Programme beginnen bei `$2001` (`.basicstub` bei `.org $2001`).
- **Grafik:** **VIC-IV** mit bis zu 640 Pixeln breitem Text, Vollfarb-Zeichen mit 256 Farben und breiten Sprites. **Wichtig:** Die erweiterten Register sind nach dem Einschalten **gesperrt**. Du schaltest sie am Programmanfang mit dem Makro `mega65_enable_vic4` frei (schreibt `$47` und `$53` nach `$D02F`).
- **Klang:** **Vier SIDs** (`$D400`, `$D420`, `$D440`, `$D460`), im Tracker 12 Stimmen. Dazu kommt der **PLAY-Tracker** für BASIC-Programme mit `PLAY`-Strings (siehe [PLAY-Tracker](kapitel:play-tracker)).
- **Paletten:** 256 Farben mit je 8 Bit je Kanal (siehe [Paletten](kapitel:paletten)).
- **Format:** PRG, oder als Diskettenabbild **D81**.
- **Emulator:** **Xemu** (`xmega65`). Dafür brauchst du eine MEGA65-ROM-Datei im ROM-Ordner (Einstellungen ▸ **ROMs**); der Name ist egal. Der Start dauert etwa 15 Sekunden.
- **Debugger:** Register, Speicher, Pause/Weiter und Einzelschritt sind möglich, aber nur **ein** Haltepunkt je Sitzung. Xemu hält erst *nach* dem Befehl am Haltepunkt an; der Programmzähler steht dann auf dem folgenden Befehl. Datenhaltepunkte gibt es nur als langsame Beobachtung per Einzelschritt. Der Emulator-Reiter zeigt für den MEGA65 kein Bild.
- **Echte Hardware:** Die Anbindung per USB oder Ethernet ist eingebaut, ist aber noch nicht am echten Gerät geprüft.
- **Beispiel:** `examples/mega65/hello`, `examples/mega65/pal_demo`, `examples/mega65/play_demo`, `examples/mega65/music_demo`.

> **Achtung:** Der MEGA65 hat in diesem Kapitel die meisten Stolpersteine: ROM-Datei nötig, VIC-IV muss freigeschaltet werden, nur ein Haltepunkt. Beginne am besten mit `examples/mega65/hello`.

## PET/CBM-II

Der **Commodore PET / CBM** (Target `pet`) hat Daten und Beispiele, aber keinen Platz in der Oberfläche: kein Eintrag im Wizard, keine Emulator-Einstellung, kein Start mit F5 und kein Debugger. Bauen kannst du mit `vcc`, den Emulator startest du von Hand. Der PET hat einen reinen Textbildschirm bei `$8000` (ohne Farbe und Sprites), BASIC 2 oder 4 und Programme ab `$0401`. Die Angaben sind als Datenstand ohne vollständige Prüfung gekennzeichnet; Der CBM-II (6x0/7x0) lässt sich nur mit BASIC bauen; für ihn gibt es noch keinen Assembler.

## Ein Beispiel für jedes System ausprobieren

1. Öffne ein Beispielprojekt aus dem Ordner `examples/` über die Willkommensseite oder **Datei ▸ Projekt öffnen …** (**Strg/Cmd+O**), zum Beispiel `c64/border`.
2. Drücke **F7** (Bauen). Im Bauprotokoll siehst du das Ergebnis, zum Beispiel `border.asm -> build/border.prg (36 Bytes)`.
3. Drücke **F5**, um das Programm im Emulator zu starten, oder **F6**, um es zu debuggen.
4. Wiederhole das mit `c128/border`, `plus4/border`, `atari/colorbars`, `mega65/hello` und `vic20/tiles_demo`. Diese Beispiele bauen fehlerfrei.

![Das Atari-Beispielprojekt colorbars im Editor](bild:system-vergleich-atari "Das Atari-Beispiel colorbars: ①: Zielsystem Atari 800 XL/XE. ②: der Quelltext mit ANTIC und GTIA. ③: die Ausgabe als XEX.")

![Das MEGA65-Beispielprojekt hello](bild:system-vergleich-mega65 "Das MEGA65-Beispiel hello: ①: Zielsystem MEGA65. ②: das Makro zum Freischalten des VIC-IV. ③: das Bauergebnis.")

## Häufige Stolpersteine

| Problem | Ursache und Lösung |
|---|---|
| Der Emulator startet nicht | Pfad prüfen: Einstellungen ▸ **Emulatoren**. Bleibt das Feld leer, sucht die IDE selbst. |
| Beim MEGA65 meldet die IDE eine fehlende ROM | Lege die ROM-Datei in den ROM-Ordner (Einstellungen ▸ **ROMs**, Hilfe „ROM-Dateien“). |
| Das VIC-IV-Register tut nichts | Der VIC-IV ist gesperrt: `mega65_enable_vic4` am Programmanfang aufrufen. |
| Das VC-20-Programm läuft nur auf dem Grundgerät | Eine feste Adresse (`.org $1001`) gilt nur dort. Nimm `.org BASIC_START`. |
| Atari-Programm startet nicht | `.run start` fehlt. |
| Bei Plus/4 gibt es keinen Sprite-Editor | Der TED hat keine Sprites, und es gibt noch keine Profile dafür. |
| Debugger auf dem Atari hält nicht an | atari800 lässt sich nicht per Knopf anhalten: Haltepunkt setzen oder **F8** im Emulatorfenster. |
| Ich habe ein Programm für den C64 und will es auf dem C128 starten | Ändere `.target` auf `c128` (C128-Modus; BASIC und Startadresse unterscheiden sich) oder bleibe bei `c64`. |

## Wie geht es weiter?

- Dein erstes Projekt auf einem anderen System: [Dein erstes Projekt](kapitel:erstes-projekt).
- Die Grafik- und Klangwerkzeuge: [Grafik und Klang im Überblick](kapitel:grafik-ueberblick).
- Nachschlagen: [C128 und Plus/4](hilfe:c128-plus4), [VC-20](hilfe:vic20), [MEGA65](hilfe:mega65), [MEGA65 am Gerät](hilfe:mega65-hardware), [ROM-Dateien](hilfe:roms).
