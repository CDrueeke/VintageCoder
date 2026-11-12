# VC-20 (VIC-20)

Target `vic20`: CPU 6502 (`nmos6502`), Videochip **VIC-I** (6560/6561), zwei **VIA 6522**, BASIC V2 (Dialekt `cbm_v2`), Emulator VICE (`xvic`).

## Speicherausbau
Der VC-20 gibt es im Grundausbau mit 5 KB und mit Steckmodul-Erweiterungen. Davon hängen BASIC-Anfang, Bildschirm- und Farb-RAM ab. Der Ausbau steht in der Projektdatei beim Target (`"memory": "8k"`) und lässt sich unter *Projekt ▸ Projekt-Einstellungen* wählen; die Vorlagen nehmen das Grundgerät.

| Ausbau | BASIC ab | Bildschirm | Farb-RAM | VICE `-memory` |
|---|---|---|---|---|
| `unexpanded` (Grundgerät, 5 KB) | `$1001` | `$1E00` | `$9600` | `none` |
| `3k` (+3 KB, `$0400–$0FFF`) | `$0401` | `$1E00` | `$9600` | `3k` |
| `8k`, `16k`, `24k`, `35k` | `$1201` | `$1000` | `$9400` | `8k`, `16k`, `24k`, `all` |

- **Assembler:** Beim Bauen definiert VintageCoder `MEM_SCREEN_RAM`, `MEM_COLOR_RAM` und `MEM_BASIC_START`. Die Systemdatei `#include <vic20/hardware.asm>` (oder `vic20/kernal.asm`) macht daraus `SCREEN_RAM`, `COLOR_RAM` und `BASIC_START`. Ohne Einstellung gilt das Grundgerät. Auf der Kommandozeile: `vcc asm prog.asm --target vic20 --memory 8k`.
- **`.org` und `.basicstub`:** `.org BASIC_START` und `.basicstub start` passen sich dem Ausbau an. Ein festes `.org $1001` gilt nur für das Grundgerät und die Vorlagen.
- **BASIC:** `vcc basic prog.bas --target vic20 --memory 8k` (und der normale Bau mit der Projekteinstellung) setzt die Startadresse; das Programm muss in den freien Speicher passen.
- **Start und Debuggen:** F5 und F6 starten `xvic` mit dem passenden `-memory`-Argument. Den Pfad stellt man unter *Einstellungen ▸ Emulatoren* ein (leer = automatisch suchen).

## Steckmodul
Mit `.cartridge "a0", "NAME"` wird aus dem Quelltext ein rohes 8-KB-Abbild; die Dateiendung nennt den Speicherblock: `.a0` (`$A000`–`$BFFF`), `.20`, `.40`, `.60` (`$2000`, `$4000`, `$6000`). Ungenutzte Bytes sind `$FF`. Ein Modul bei `$A000` startet das System selbst, wenn bei `$A000` und `$A002` die Kalt- und Warmstartadresse stehen und bei `$A004` die Kennung `A0CBM` (`.byte $41,$30,$c3,$c2,$cd`); fehlt sie, meldet der Assembler einen Fehler. Das System ist beim Start noch nicht initialisiert: das Modul ruft `RAMTAS`, `RESTOR`, `IOINIT` und `CINT` (und `CLRCHN`) selbst auf. Die Vorlage *Steckmodul* zeigt das. Auf der Kommandozeile: `vcc asm modul.asm --target vic20` (Endung folgt dem Block) oder `--format a0|20|40|60`. F5 und der Debugger legen das Abbild als Modul ein (`-cartA`, `-cart2` …).

## Chips
- `vic1` (`$9000–$900F`): Bildursprung, Spalten/Zeilen, Speicherzeiger (`$9005`), Raster, drei Tonstimmen und Rauschen (`$900A–$900D`), Lautstärke/Hilfsfarbe, Rahmen/Hintergrund (`$900F`). Beispiel: `sta vic1.color`.
- `via1` (`$9110`: NMI, Userport, Joystick, serieller Bus) und `via2` (`$9120`: IRQ-Timer, Tastatur).
- Konstanten: `#include <vic20/hardware.asm>` (Register, Speicherbereiche, Farben `COL_*`) und `#include <vic20/kernal.asm>` (KERNAL-Sprungtabelle wie beim C64, aber erst ab `RESTOR` (`$FF8A`); `RAMTAS`, `IOINIT` und `CINT` sind direkte ROM-Adressen; dazu Vektoren und Systemvariablen).

## Zeichensatz, PETSCII
- Die Zeichensätze des VC-20 (Groß-/Grafik und Groß-/Kleinschreibung, je Zeichen-ROM-Revision) kommen aus deinem ROM (Hilfe „ROM-Dateien“) und liegen unter *Datei ▸ Standardzeichensatz hinzufügen …* bereit; das Profil `vic20.char.hires` ist 8×8, einfarbig. Daneben gibt es `vic20.char.multicolor` (4×8, vier Pixelwerte: Hintergrund, Rahmen, Zeichenfarbe, Hilfsfarbe; die Farben liegen in `$900F`, im Farb-RAM und in `$900E`) und `vic20.char.hires16` (8×16 für `$9003` Bit 0). Beide beginnen leer; Zeichen- und Farbfelder arbeiten mit der VIC-I-Palette.
- Der Dialog *Zeichen einfügen* (PETSCII) arbeitet mit diesen Zeichensätzen; `.petscii`, `.screen` und `{clr}`-Schreibweise sind wie beim C64.

## Bildschirm-Editor
Profil `vic20.screen.text` (22×23 Zeichen, mit Farb-RAM; Rahmen, Hintergrund und Zeichenfarbe pro Zeichen). Der Editor liefert nur die Daten (`chars` und `attrs` als Export oder `.include`); sie müssen zur Laufzeit nach `SCREEN_RAM` und `COLOR_RAM` kopiert werden, deren Adressen vom Speicherausbau abhängen (siehe oben). Beim Farb-RAM zählen nur Farben 0–7; ab 8 schaltet der VIC-I das Zeichen auf Mehrfarb (mit der Farbe n − 8), die Farbfelder des Editors zeigen diesen Unterschied nicht. Rahmen: nur Farben 0–7, Hintergrund und Hilfsfarbe 0–15.

## Kacheln und Tracker
- **Kacheln:** Profil `vic20.tilemap.2x2` (11×11 Kacheln aus je 2×2 Zeichen = 22×22 des 22×23-Bildschirms, mit Farb-RAM). Der Export wie bei den anderen Systemen; kopiert wird zur Laufzeit nach `SCREEN_RAM` und `COLOR_RAM`.
- **Tracker:** Profil `vic20.vic.song` (4 Stimmen: Bass `$900A`, Alt `$900B`, Sopran `$900C`, Rauschen `$900D`; 32 Zeilen, höchstens 8 Instrumente). Beispiel mit Spielroutine: `examples/vic20/music_demo` (`lib/music.inc`, Frequenztabellen `lib/vic_freq.inc`, erzeugt von `tools/gen_vic_freq.py`). Der Takt kommt vom KERNAL-IRQ (PAL 50 Hz, NTSC 60 Hz); die Geschwindigkeit des Songs hängt also am Fernsehstandard.
- **Einschränkungen des Chips:** Der VIC-I hat nur **eine gemeinsame Lautstärke** (`$900E`, Bit 0–3). Die Lautstärkehülle eines Instruments schaltet die Stimme deshalb nur ein (Wert > 0) oder aus (0), die Lautstärke bleibt 15; die Verzerrung wird ignoriert. Die Tonhöhe wird in 7 Bit gestuft (`f = Takt · m / (256 · (127 − N))`, m = 1, 2, 4 für Bass, Alt, Sopran, in VICE gemessen); Töne außerhalb des Umfangs einer Stimme werden oktaviert, hohe Töne sind ungenau. Das Rauschen hat keine feste Tonhöhe. Die Hörprobe im Tracker kennt diese Stufung nicht und klingt reiner als der Chip.

## Grenzen
- Steckmodule: nur 8-KB-Abbilder ohne Bänke (keine Mega-Cart oder Final Expansion).
- Beim Import aus Diskettenimages wird BASIC ab `$1201` und `$0401` als VC-20 erkannt; `$1001` bleibt dem Plus/4 zugeordnet (die Zeilentexte sind dieselben).
