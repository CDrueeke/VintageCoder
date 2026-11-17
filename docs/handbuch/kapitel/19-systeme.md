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

Ein **Zielsystem** ist der Rechner, für den dein Programm gebaut wird. Du wählst es im [Projekt-Assistenten](kapitel:erstes-projekt) aus, und es steht in der Projektdatei. Die Kopfzeile zeigt es in einem Chip zusammen mit dem Ziel, zum Beispiel „C64 · Emulator (VICE x64sc) ▾“. Mit `.target` im Quelltext legst du es für eine Datei fest (zum Beispiel `.target c64`).

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

Für alle gilt: Baue mit **F7**, starte das Startprogramm mit **F5**, debugge es mit **F6**. Welcher Emulator startet, steht in den Einstellungen unter **Zielsysteme und Hardware** (Karte **Konfigurierte Emulatoren**). Ein Klick auf den Chip in der Kopfzeile wählt statt des Standard-Emulators ein anderes Gerät, zum Beispiel eine Ultimate 64.

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
- **Echte Hardware:** Die **Ultimate 64 / II+** lässt sich über das Netzwerk anbinden (Starten, Speicher lesen, Postfach-Tests, Zyklenmessung auf der Ultimate 64).
- **Beispiel:** `examples/c64/border` (Rahmenfarbe), `examples/c64/sprite_demo` (Sprites), `examples/c64/rasterbars` (Rasterbalken), `examples/c64/prallball`.

![Das Beispielprojekt border im Editor: links der Projektbaum, in der Mitte der Quelltext, oben das Zielsystem Commodore 64.](bild:system-vergleich-c64 "Das C64-Beispiel border: Projektbaum, Quelltext und das Zielsystem in der Kopfzeile")

## Commodore 128 (`c128`)

Der C128 startet in dieser IDE im **C128-Modus** mit CPU 8502 (derselbe Befehlssatz wie der 6510). Willst du den C64-Modus, nimmst du das Zielsystem `c64`. Den Z80 gibt es nicht.

- **BASIC:** BASIC 7.0 (Dialekt `basic7`); Programme beginnen bei `$1C01`. `.basicstub` erzeugt bei `.org $1c01` den `SYS 7181`.
- **Chips:** VIC-IIe (mit `$D02F` Tastatur und `$D030` 2-MHz-Modus), SID, zwei CIA, die Speichersteuerung MMU (`$D500`, `$FF00`) und der **VDC** für den 80-Spalten-Bildschirm (`$D600`, `$D601`).
- **Grafik und Klang:** Die Profile des C64 (Sprites, Zeichensätze, Bildschirme, Kacheln, Tracker) gibt es auch im C128-Projekt. VDC-Grafik und eine MMU-Hilfe im Assembler gibt es noch nicht.
- **Emulator:** VICE **x128**; der Debugger spricht dessen Binär-Monitor. Im Panel **Bild** (rechtes Panel, nur in einer Debug-Sitzung) wählst du zwischen **VIC-II** (40 Spalten) und **VDC** (80 Spalten).
- **Tests:** Postfach bei `$0B00`.
- **Disketten:** D64, D71 und D81.
- **Beispiel:** `examples/c128/border`, `examples/c128/prallball`, `examples/c128/prallball_vdc`.

## C16 und Plus/4 (`c16`, `plus4`)

- **BASIC:** BASIC 3.5 (Dialekt `basic35`); Programme beginnen bei `$1001`. `.basicstub` erzeugt bei `.org $1001` den `SYS 4109`.
- **Chip:** Der **TED** macht Bild und Ton in einem. Er hat **keine Sprites**, und es gibt keinen SID.
- **Grafik und Klang:** Es gibt noch keine Editor-Profile für Sprites, Zeichensätze oder Songs dieses Systems. Dein Programm schreibst du also direkt gegen die TED-Register.
- **Speicher:** Farb-RAM bei `$0800`, Bildschirm bei `$0C00`. Der **C16/C116** (Target `c16`) hat dieselbe Hardware, aber nur 16 KB RAM (Anwenderspeicher bis `$3FFF`). Wähle beim Neuen Projekt „Commodore 16 / 116“, dann startet F5 den Emulator als C16 (`-model c16`). Ein C16-Programm läuft auch auf dem Plus/4.
- **Emulator:** VICE **xplus4**.
- **Beispiel:** `examples/plus4/border`, `examples/plus4/prallball`, für den C16 `examples/c16/border`.

## VC-20 (`vic20`)

In Deutschland hieß der VIC-20 „VC-20“.

- **Chips:** VIC-I (Bild und Ton, Rahmen und Hintergrund in `$900F`), zwei VIA.
- **BASIC:** BASIC V2.
- **Speicherausbau:** Das ist die Besonderheit dieses Systems. Der Ausbau steht in der Projektdatei; im Projekt-Assistenten und in den Projekt-Einstellungen wählst du ihn. Davon hängen BASIC-Anfang, Bildschirm und Farb-RAM ab:

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
- **Emulator und Debugger:** **atari800** oder **Altirra**. Zum Debuggen dient atari800; er kennt **nur einen** Haltepunkt zugleich, keine Datenhaltepunkte und lässt sich nicht per Knopf anhalten (**F8 im Emulatorfenster**). Das Panel „Bild“ zeigt für den Atari nichts.
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
- **Debugger:** Register, Speicher, Pause/Weiter und Einzelschritt sind möglich, aber nur **ein** Haltepunkt je Sitzung. Xemu hält erst *nach* dem Befehl am Haltepunkt an; der Programmzähler steht dann auf dem folgenden Befehl. Datenhaltepunkte gibt es nur als langsame Beobachtung per Einzelschritt. Das Panel „Bild“ zeigt für den MEGA65 nichts.
- **Echte Hardware:** Die Anbindung per USB oder Ethernet ist eingebaut, ist aber noch nicht am echten Gerät geprüft.
- **Beispiel:** `examples/mega65/hello`, `examples/mega65/pal_demo`, `examples/mega65/play_demo`, `examples/mega65/music_demo`.

> **Achtung:** Der MEGA65 hat in diesem Kapitel die meisten Stolpersteine: ROM-Datei nötig, VIC-IV muss freigeschaltet werden, nur ein Haltepunkt. Beginne am besten mit `examples/mega65/hello`.

## PET/CBM und CBM-II

Der **Commodore PET / CBM** und der **CBM-II** sind in dieser Version **nicht enthalten**: kein Eintrag im Projekt-Assistenten, keine Emulator-Einstellung, kein Start mit F5 und kein Debugger. Beschrieben sind hier nur die Systeme, die du auswählen kannst.

## Ein Beispiel für jedes System ausprobieren

1. Öffne ein Beispielprojekt aus dem Ordner `examples/` über die Startseite oder **Datei ▸ Projekt öffnen …** (**Cmd/Strg+Umschalt+O**), zum Beispiel `c64/border`.
2. Drücke **F7** (Bauen). Im Reiter **Protokoll** der Schublade siehst du das Ergebnis, zum Beispiel `border.asm -> build/border.prg (36 Bytes)`.
3. Drücke **F5**, um das Programm im Emulator zu starten, oder **F6**, um es zu debuggen.
4. Wiederhole das mit `c128/border`, `plus4/border`, `atari/colorbars`, `mega65/hello` und `vic20/tiles_demo`. Diese Beispiele bauen fehlerfrei.

![Das Atari-Beispielprojekt colorbars im Editor, oben das Zielsystem Atari 800 XL/XE.](bild:system-vergleich-atari "Das Atari-Beispiel colorbars: Quelltext mit ANTIC und GTIA, oben das Zielsystem")

![Das MEGA65-Beispielprojekt hello im Editor, oben das Zielsystem MEGA65.](bild:system-vergleich-mega65 "Das MEGA65-Beispiel hello: Quelltext mit dem Makro zum Freischalten des VIC-IV")

## Häufige Stolpersteine

| Problem | Ursache und Lösung |
|---|---|
| Der Emulator startet nicht | Pfad prüfen: Einstellungen ▸ **Zielsysteme und Hardware**, Karte **Konfigurierte Emulatoren**. Bleibt das Feld leer, sucht die IDE selbst. Unter Linux siehe den Abschnitt „Emulatoren unter Linux“ unten. |
| Beim MEGA65 meldet die IDE eine fehlende ROM | Lege die ROM-Datei in den ROM-Ordner (Einstellungen ▸ **ROMs**, Hilfe „ROM-Dateien“). |
| Das VIC-IV-Register tut nichts | Der VIC-IV ist gesperrt: `mega65_enable_vic4` am Programmanfang aufrufen. |
| Das VC-20-Programm läuft nur auf dem Grundgerät | Eine feste Adresse (`.org $1001`) gilt nur dort. Nimm `.org BASIC_START`. |
| Atari-Programm startet nicht | `.run start` fehlt. |
| Bei Plus/4 gibt es keinen Sprite-Editor | Der TED hat keine Sprites, und es gibt noch keine Profile dafür. |
| Debugger auf dem Atari hält nicht an | atari800 lässt sich nicht per Knopf anhalten: Haltepunkt setzen oder **F8** im Emulatorfenster. |
| Ich habe ein Programm für den C64 und will es auf dem C128 starten | Ändere `.target` auf `c128` (C128-Modus; BASIC und Startadresse unterscheiden sich) oder bleibe bei `c64`. |

## Emulatoren unter Linux

VintageCoder liefert **nie** einen Emulator mit. Du installierst ihn selbst: aus den Paketquellen deiner Distribution, beim Hersteller oder, wo es ein festes Paket gibt, mit dem Knopf **Portabel installieren …** (siehe unten). VintageCoder sucht den Emulator danach im `PATH` und in einigen üblichen Ordnern (`/usr/games`, `/usr/local/games`, `/usr/local/bin`, `/snap/bin`, `/opt/vice/bin`, `/opt/xemu`, `~/.local/bin`, `~/bin`). Liegt er woanders, trägst du den Pfad von Hand ein.

> **Stand prüfen:** Die Angaben zu den Paketquellen stammen aus einer Abfrage der Standardquellen in frischen Containern (Debian 13, Ubuntu 24.04, Fedora 45, openSUSE Tumbleweed). Wie man die Quellen einschaltet, wurde nicht in jedem Fall auf einer echten Installation durchgespielt. Alles, was unten als **ungeprüft** steht, ist nicht bestätigt. Distributionsstände ändern sich mit jeder Version: Sieh im Zweifel mit dem Paketmanager nach.

Welchen Emulator du brauchst:

| Zielsystem | Emulator | Programmname, nach dem VintageCoder sucht |
|---|---|---|
| C64 | VICE | `x64sc` (taktgenau) oder `x64` (schnell) |
| C128 | VICE | `x128` |
| Plus/4, C16 | VICE | `xplus4` |
| VC-20 | VICE | `xvic` |
| Atari 800 XL/XE | atari800 (Altirra gibt es nur für Windows) | `atari800` |
| MEGA65 | Xemu | `xmega65` (auch `xemu-xmega65`, `xmega65.native`, `xemu-mega65`) |

> **Info:** Für den **Debugger** braucht VICE die **Version 3.9 oder neuer**. Ob das bloße Starten mit **F5** auch mit älteren Versionen klappt, ist nicht geprüft. Prüfe nach der Installation die Version (`x64sc --version`).

### Was die Standardquellen anbieten

| Distribution | VICE | atari800 | Xemu (MEGA65) | Fujisan |
|---|---|---|---|---|
| **Debian 13** | 3.9, nur in `contrib` | 5.2.0, nur in `contrib` | kein Paket | kein Paket |
| **Ubuntu 24.04** | 3.7.1, in `multiverse` | 5.0.0, in `multiverse` | kein Paket | kein Paket |
| **Fedora 45** | nicht in den Fedora-Quellen | 7.1.2, Repository `fedora` | kein Paket | kein Paket |
| **openSUSE Tumbleweed** | nicht in oss, non-oss, update | nicht in oss, non-oss, update | kein Paket | kein Paket |

- **Debian 13:** VICE und atari800 gibt es erst, wenn du die Paketquellen-Sektion `contrib` einschaltest.
- **Ubuntu 24.04:** VICE und atari800 gibt es erst, wenn du die Sektion `multiverse` einschaltest. VICE ist dort **3.7.1**, also **älter als 3.9**: Für den Debugger von VintageCoder ist das laut diesem Handbuch zu alt. Zum Debuggen brauchst du ein neueres VICE (selbst bauen oder von der VICE-Seite).
- **Fedora 45:** atari800 ist ohne Zusatzquellen da, VICE nicht.
- **openSUSE Tumbleweed:** Weder VICE noch atari800 stehen in den Standardquellen.
- **Xemu** und **Fujisan** (ein zweiter Atari-Emulator, den der Debugger für `.xex`-Programme bevorzugt) gibt es in keiner der vier Distributionen als Paket. Der Eintrag `xemu` in den openSUSE-Quellen ist ein **Xbox-Emulator**, **nicht** der für den MEGA65: Installiere ihn nicht dafür.

### Debian 13

**Eingriff in die Paketquellen.** `contrib` ist eine Sektion der offiziellen Debian-Paketquellen mit freier Software, die von Software außerhalb des Hauptarchivs abhängt. Das Einschalten ändert die Konfiguration deines Systems: Danach sind **alle** Pakete aus `contrib` installierbar, nicht nur VICE und atari800. Mach das nur, wenn du das willst. Lege vorher eine Sicherungskopie der Datei an.

1. Finde die Datei mit deinen Paketquellen. Bei einer neuen Debian-13-Installation ist es meist `/etc/apt/sources.list.d/debian.sources`; ältere oder umgestellte Systeme haben stattdessen `/etc/apt/sources.list`.

   ```bash
   ls /etc/apt/sources.list.d/
   grep -n -i -E "^(Components|deb )" /etc/apt/sources.list.d/*.sources /etc/apt/sources.list
   ```

2. Sichere die Datei und öffne sie mit einem Editor (hier `debian.sources` als Beispiel):

   ```bash
   sudo cp /etc/apt/sources.list.d/debian.sources /etc/apt/sources.list.d/debian.sources.bak
   sudo nano /etc/apt/sources.list.d/debian.sources
   ```

3. Suche die Zeile `Components: main` und ergänze `contrib`, sodass dort `Components: main contrib` steht. In einer klassischen `sources.list` hängst du `contrib` hinter `main` an, zum Beispiel `deb http://deb.debian.org/debian trixie main contrib`.

4. Lies die Paketliste neu ein und installiere:

   ```bash
   sudo apt update
   sudo apt install vice atari800
   ```

   Du kannst auch nur eines der beiden installieren.

5. Prüfe, was angekommen ist:

   ```bash
   apt policy vice atari800
   x64sc --version
   command -v atari800
   ```

   Bei `atari800` liegt das Programm üblicherweise unter `/usr/games`; auch dort sucht VintageCoder.

Zum Rückgängigmachen nimmst du `contrib` wieder aus der Datei (oder spielst die `.bak`-Datei zurück) und führst `sudo apt update` aus. Installierte Pakete bleiben bestehen; entfernen kannst du sie mit `sudo apt remove vice atari800`.

### Ubuntu 24.04

**Eingriff in die Paketquellen.** `multiverse` ist eine Sektion der offiziellen Ubuntu-Paketquellen für freie Software, bei der Canonical keine Sicherheitsaktualisierungen zusagt (die Pflege liegt bei der Gemeinschaft). Das Einschalten ändert die Paketquellen für **alle** Pakete aus `multiverse`. Mach das nur, wenn du das willst.

1. Prüfe, ob `multiverse` schon an ist: `apt policy | grep -i multiverse`. Kommt eine Ausgabe, ist sie aktiv, und du kannst zu Schritt 3.
2. Schalte `multiverse` ein:

   ```bash
   sudo add-apt-repository multiverse
   sudo apt update
   ```

   Fehlt der Befehl, installiere ihn mit `sudo apt install software-properties-common`. Du kannst die Sektion auch in „Software & Updates“ einschalten.
3. Installiere und prüfe:

   ```bash
   sudo apt install vice atari800
   apt policy vice atari800
   x64sc --version
   ```

   Du siehst VICE **3.7.1** und atari800 **5.0.0**.

> **Achtung:** VICE 3.7.1 ist älter als die für den Debugger nötige Version 3.9. Das Starten mit F5 kannst du ausprobieren (**▶ Test-Start** in der Emulator-Karte). Für den Debugger brauchst du ein neueres VICE. Ob atari800 5.0.0 mit dem Debugger zusammenarbeitet, ist **ungeprüft**.

Zum Rückgängigmachen: `sudo add-apt-repository --remove multiverse`, danach `sudo apt update`. Installierte Pakete entfernst du mit `sudo apt remove vice atari800`. **Ungeprüft:** der genaue Handgriff auf frischen und auf aktualisierten Systemen.

### Fedora 45

atari800 steht in den Standardquellen (Repository `fedora`, Version 7.1.2). Dafür änderst du nichts an den Paketquellen:

```bash
sudo dnf install atari800
dnf info atari800
command -v atari800
```

**VICE** steht nicht in den Fedora-Quellen. Eine mögliche Quelle ist das Drittrepository **RPM Fusion**. **Ungeprüft:** ob es VICE dort gibt, in welcher Version und unter welchem Namen. RPM Fusion ist **nicht** Teil von Fedora; es einzubinden ist ein Eingriff in die Paketquellen. Halte dich an die aktuelle Anleitung des Projekts (rpmfusion.org).

### openSUSE Tumbleweed

In den Standardquellen (oss, non-oss, update) gibt es **weder VICE noch atari800**. Sieh nach, ob sich bei dir etwas geändert hat:

```bash
zypper search vice atari800
```

Fehlt das Paket, gilt der Abschnitt „Wenn das Paket fehlt oder zu alt ist“. **Ungeprüft:** ob es in einem Zusatzrepository (zum Beispiel einem Gemeinschaftsrepository) Pakete gibt. Eine Zusatzquelle einzubinden ist ein Eingriff in die Paketquellen; prüfe vorher, wem sie gehört und wie aktuell sie ist. Wenn das Paket erreichbar ist, installierst du es mit `sudo zypper install vice atari800`.

### Wenn das Paket fehlt oder zu alt ist

Dann holst du den Emulator beim Hersteller.

| Emulator | Wo | Hinweis |
|---|---|---|
| **VICE** | vice-emu.sourceforge.io, Abschnitt „Download“ | Für Linux gibt es dort den Quelltext zum Selbstbauen. Ob die Seite fertige Linux-Pakete anbietet, prüfst du dort. VintageCoder lädt VICE unter Linux nicht selbst. |
| **atari800** | github.com/atari800/atari800/releases | Im Katalog von VintageCoder steht für Linux (x86_64) eine **AppImage**-Datei (7.2.1 und 7.1.2). Das ist eine einzelne ausführbare Datei. Ob die Seite sie in dieser Form noch anbietet, prüfst du dort. |
| **Xemu (MEGA65)** | github.com/lgblgblgb/xemu/releases und github.com/lgblgblgb/xemu/wiki | Das Projekt gibt nur Vorabversionen heraus. Für Linux nennt der Katalog ein `.deb`-Paket (`xemu_<Datum>_amd64.deb`) oder Selbstbauen. Auf Debian und Ubuntu installierst du es mit `sudo apt install ./xemu_<Datum>_amd64.deb`; auf Fedora und openSUSE hilft das `.deb` nicht. Das Programm heißt `xmega65`. Du brauchst außerdem eine MEGA65-ROM-Datei. |
| **Fujisan** | github.com/pedgarcia/fujisan/releases | Optional, nur für den Atari-Debugger. Für Linux nennt der Katalog Archive (`.tar.gz`) für x86_64 und aarch64. |
| **Altirra** | nur Windows | unter Linux atari800 oder Fujisan nehmen |

**Portabel installieren.** Auf der Seite **Einstellungen ▸ Zielsysteme und Hardware** zeigt die Emulator-Karte, wo es ein festes Paket gibt, einen Knopf **Portabel installieren …**. Nach einer Rückfrage („Jetzt laden“) lädt VintageCoder das Paket von der Seite des Projekts, prüft die SHA-256-Summe und legt es in einem eigenen Ordner unterhalb des Konfigurationsordners ab. Außerhalb dieses Ordners ändert es nichts, und ohne deinen Klick lädt es nichts. Unter Linux gibt es dieses Angebot nach dem Katalog für **atari800** (AppImage, es wird nur gespeichert und ausführbar gemacht) und für **Fujisan** (Archiv, wird ausgepackt). Für VICE und Xemu steht in der Karte stattdessen ein Hinweis mit der Download-Seite oder dem Weg über die Paketverwaltung.

**Flatpak und Snap.** **Ungeprüft:** ob es VICE, atari800, Xemu oder Fujisan als Flatpak oder Snap gibt und ob VintageCoder damit zurechtkommt. Flatpak-Programme heißen nach ihrer Kennung und werden nicht automatisch gefunden: Du müsstest den Pfad eines Skripts eintragen, das zum Beispiel `flatpak run <Kennung> "$@"` ausführt. Snap-Programme liegen unter `/snap/bin`; dort sucht VintageCoder.

### Danach in VintageCoder einrichten

Es gibt zwei Wege, beide ändern am Ende dasselbe.

**Weg 1: Konfigurationsassistent.** Beim ersten Start öffnet er sich von selbst; sonst startest du ihn mit **Einstellungen ▸ Allgemein ▸ Assistent starten** oder mit **Hilfe ▸ Konfigurationsassistent: nur Emulatoren …**. Im Schritt **Toolchain** zeigt die Liste je Emulator „bereit“ oder „nicht gefunden“. Der Assistent installiert nichts: Fehlende Emulatoren installierst du selbst (siehe oben) oder trägst sie unter **Einstellungen ▸ Zielsysteme und Hardware** ein; dort gibt es auch die portable Installation.

**Weg 2: Von Hand in den Einstellungen.**

1. Öffne die Einstellungen: **Ansicht ▸ Einstellungen** oder **Strg+Komma**.
2. Wechsle zur Seite **Zielsysteme und Hardware** und wähle links dein System.
3. In der Karte **Konfigurierte Emulatoren** steht beim Emulator ein Chip **Gefunden**, **Automatisch erkannt** oder **Fehlt**. Das Feld **Pfad** kannst du leer lassen: Dann sucht VintageCoder selbst. **Automatisch suchen** trägt den gefundenen Pfad ein. Ein eigener Pfad geht auch.
4. Fehlt der Emulator in der Liste, bieten die Knöpfe **+ …** über den Karten die Sorten an, die es für das System gibt.
5. **Als Standard setzen** macht ihn zum Emulator für **F5** und **F6** (es sei denn, du hast ein anderes Startziel gewählt).
6. Beim MEGA65 legst du die ROM-Datei im ROM-Ordner ab (**Einstellungen ▸ ROMs**; unter Linux `~/.config/vintagecoder/roms`, falls `XDG_CONFIG_HOME` nicht gesetzt ist). Ohne MEGA65-ROM startet Xemu nicht.
7. Klicke auf **Speichern**.

### Die Einrichtung prüfen

1. **Chip:** Bei deinem System steht „Emulator bereit“, bei der Emulator-Karte **Gefunden** oder **Automatisch erkannt**.
2. **Test-Start:** **▶ Test-Start** in der Emulator-Karte öffnet den Emulator kurz zur Probe; die Meldung daneben zeigt, ob es geklappt hat.
3. **Hello World:** Öffne ein Beispielprojekt, drücke **F7** (Bauen) und **F5** (Starten).
4. **Im Terminal (optional):** `command -v x64sc`, `command -v atari800`, `command -v xmega65`. Eine leere Ausgabe heißt: nicht im `PATH`. Unter Debian und Ubuntu liegt `atari800` meist in `/usr/games`; VintageCoder sucht dort mit, im Terminal ist der Ordner nicht immer im `PATH`.

### Wenn etwas nicht klappt

| Problem | Ursache und Abhilfe |
|---|---|
| `E: Unable to locate package vice` (Debian, Ubuntu) | `contrib` beziehungsweise `multiverse` ist nicht eingeschaltet, oder `sudo apt update` fehlt nach dem Ändern. |
| „Unable to find a match“ (`dnf`) oder „Kein Treffer“ (`zypper`) | Das Paket steht in deinen Quellen nicht (siehe Tabelle oben). Weiter mit „Wenn das Paket fehlt oder zu alt ist“. |
| Chip bleibt **Fehlt**, obwohl installiert | Programm im `PATH` prüfen (`command -v …`), sonst den Pfad in der Emulator-Karte eintragen. |
| Debugger startet nicht oder meldet eine zu alte Version | VICE ist älter als 3.9 (zum Beispiel Ubuntu 24.04 mit 3.7.1). Neueres VICE beim Hersteller besorgen. |
| Xemu startet nicht | MEGA65-ROM (genau 131072 Byte) fehlt: **Einstellungen ▸ ROMs**. |
| Du hast das Paket `xemu` aus openSUSE installiert | Das ist ein Xbox-Emulator. Entfernen (`sudo zypper remove xemu`) und Xemu für den MEGA65 beim Hersteller laden. |

> **Stand prüfen (offene Punkte):** Programmnamen nach der Installation (`x64sc`, `x128`, `xplus4`, `xvic` im `PATH`); ob Starten und Debugger mit VICE 3.7.1 und atari800 5.0.0 (Ubuntu), VICE 3.9 und atari800 5.2.0 (Debian) sowie atari800 7.1.2 (Fedora) funktionieren; ob die im Katalog genannten Herstellerdateien noch so angeboten werden; Flatpak und Snap; Zusatzquellen für Fedora und openSUSE.

## Wie geht es weiter?

- Dein erstes Projekt auf einem anderen System: [Dein erstes Projekt](kapitel:erstes-projekt).
- Die Grafik- und Klangwerkzeuge: [Grafik und Klang im Überblick](kapitel:grafik-ueberblick).
- Nachschlagen: [C128 und Plus/4](hilfe:c128-plus4), [VC-20](hilfe:vic20), [MEGA65](hilfe:mega65), [MEGA65 am Gerät](hilfe:mega65-hardware), [ROM-Dateien](hilfe:roms).
