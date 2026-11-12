# Assembler: Systembibliothek

Die Seite gehört zum [Assembler-Handbuch](help:assembler).

Eine **Systembibliothek** spart dir das Abtippen von Adressen und Routinen. Mit `#include <pfad>` bindest du fertige Konstantenlisten für das Zielsystem ein: Hardware-Register, Betriebssystem-Einsprünge, Systemvariablen und Vektoren. Die Dateien liegen unter `data/sys/`; eine gleichnamige Datei im Projekt hat Vorrang. `//` gilt wie `;` als Kommentar.

```asm
#include <c64/hardware.asm>
#include <c64/kernal.asm>

    lda #15              // auch // ist ein Kommentar
    sta VIC_BORDERCOLOR
    jsr CHROUT
```

| Datei | Inhalt |
|---|---|
| `c64/hardware.asm` | VIC-II, SID, CIA 1/2, Farb-RAM, Farbnamen `COL_*` |
| `c64/kernal.asm` | KERNAL-Sprungtabelle (`CHROUT`, `GETIN` …), Vektoren, Nullseite, RAM-Vektoren |
| `c64/basic.asm` | BASIC-Zeiger, Gleitkomma-Register, ROM-Routinen (`STROUT` …) |
| `c128/hardware.asm` `kernal.asm` | VIC-IIe, MMU, VDC (80 Spalten), KERNAL-Erweiterungen (`JSRFAR`, `INDFET` …), Makro `mmu_config` |
| `plus4/hardware.asm` `kernal.asm` | TED, ACIA, ROM-Bankwahl, Farben `TED_COL_*`, KERNAL-Sprungtabelle (auch für C16) |
| `mega65/hardware.asm` | wie C64, dazu VIC-III/IV (`VIC4_*`), DMAgic (`DMA_*`), Hypervisor/System (`M65_*`) |
| `mega65/gs4502.asm` | Makros `mega65_enable_vic4`, `mega65_fast`, `ld32` |
| `mega65/kernal.asm` | Sprungtabelle (in allen ROM-Versionen gleich) |
| `mega65/rom_920418.asm` | Systemvariablen der geschlossenen ROM-Version 920418 |
| `mega65/rom_open.asm` | OpenROMs: nur die Sprungtabelle ist zugesichert |
| `atari8bit/antic.asm` `gtia.asm` `pokey.asm` | Chip-Register mit den Atari-Standardnamen (`DMACTL`, `COLPF0`, `AUDF1` …) |
| `atari8bit/os.asm` | PIA, Nullseite, Schattenregister (`SDMCTL`, `COLOR0` …), IOCB, `CIOV`, `SETVBV` |
| `vcs2600/tia.asm` | TIA, RIOT, Speicherbereiche, Makros `CLEAN_START`, `VERTICAL_SYNC`, `WAIT_LINES` |
| `common/types.asm` | Bitmasken `BIT0`–`BIT7`, Statusbits `P_*`, Makros `set_bits`, `ld16`, `ptr_set`, `mov16`, `mov8`/`movx8`/`movy8` (Byte kopieren), `add8`/`sub8` … |
| `common/math.asm` | 16-Bit-Makros (`add16`, `sub16`, `cmp16`, `inc16` …) und die Unterprogramme `mul16`/`div16` (mit `math_routines` einsetzen; Nullseite über `MATH_ZP` wählbar) |
| `c64/tiles.asm` (mit `common/tiles.asm`) | Kachel-Bibliothek: `tiles_use`, `tile_draw`, `tile_draw_color`, `tile_fill`, `screen_select`, `screen_draw` (Makros) und mit `tiles_runtime` die Unterprogramme `tile_put`, `screen_draw_all`, `screen_draw_rows`, `screen_map_get`, `tile_prop`; Kachelgröße B×H beliebig, Schalter `TILE_SMC`, `TILE_ZP`, `TILE_SCREEN`, `TILE_MAPRAM`. Beispiel `examples/c64/tiles_demo`, Befehle im Kopf der Datei |
| `atari8bit/tiles.asm`, `plus4/tiles.asm`, `vic20/tiles.asm`, `c128/tiles.asm`, `mega65/tiles.asm` | dieselbe Kachel-Bibliothek (`common/tiles.asm`) für die weiteren Systeme, gleiche Befehle und Schalter; nur die Systemteile unterscheiden sich (siehe unten). Beispiele `examples/<system>/tiles_demo` |
| `c64/notes.inc`, `c128/notes.inc`, `mega65/notes.inc`, `plus4/notes.inc`, `vic20/notes.inc`, `atari8bit/notes.inc` | Notenkonstanten `NOTE_C0` … `NOTE_B7` und die Frequenztabelle des Klangchips (PAL/NTSC), siehe Abschnitt „Noten und Frequenztabellen“ unten |

**Kachel-Bibliothek je System.** Alles Gemeinsame steht in `common/tiles.asm`; die Systemdatei legt nur `TILE_COLS`, `TILE_ROWS`, `TILE_SCREEN` und `TILE_COLORRAM` fest (alle vor dem Einbinden überschreibbar, z. B. mit `vcc asm -D TILE_SCREEN=$0800`).

| System | Bildschirm | Farbspeicher | Besonderheit |
|---|---|---|---|
| C64 | 40×25 bei `$0400` | `$D800`, 4 Bit | Bit 3 = Mehrfarbe |
| Atari 8 Bit | 40×24 bei `$4000` | keiner (`TILE_NOCOLOR`) | Farbwahl im Mehrfarbmodus 4 über Zeichenbit 7 (Zellflag `sel` im Export); der Bildschirm muss in einem 4-KB-Block liegen (prüft die Datei); `tile_draw_color` zeichnet nur das Zeichen |
| Plus/4, C16 | 40×25 bei `$0C00` | `$0800` (vor dem Bildschirm, Abstand −`$0400`) | Attribut-Byte = Helligkeit·16 + Farbton, `col` im Export ist das fertige Byte |
| VC-20 | 22×23, Grundgerät `$1E00` (ab +8 KB `$1000`) | `$9600` (ab +8 KB `$9400`), 4 Bit | Adressen aus `vic20/hardware.asm` nach Speicherausbau des Projekts; Kachelbreite 3 gibt 7 Kacheln je Zeile |
| C128 | 40×25 (VIC-IIe) bei `$0400` | `$D800` | der VDC (80 Spalten) wird nicht unterstützt; `TILE_MAPRAM` darf nicht in einem ROM-Fenster liegen (`$4000`–`$BFFF` zeigt BASIC-ROM) |
| MEGA65 | 40×25 bei `$0800` | `$D800` | VIC-II-Verhalten des VIC-IV; Zyklenbudgets mit `TILE_CYC_LDA = 5` (40-MHz-Tabelle des 45GS02) |

Geprüft (`vcc test`, Emulator): C64, Plus/4, VC-20 und C128 mit VICE (Trap-Modus), Atari 8 Bit mit atari800 (Postfach-Modus); MEGA65 nur gebaut und statisch geprüft (Xemu lief hier nicht an), die Zyklen kommen aus dem `.cycles`-Block. Das VC-20-Beispiel ist für das Grundgerät gebaut. Die Zeropage-Gruppe `TILE_ZP` (`$F7`–`$FE`) ist auf Plus/4, C128, VC-20 und MEGA65 noch nicht gegen die KERNAL-Belegung geprüft (die Beispiele laufen mit gesetztem I-Flag ohne KERNAL).


**Einbinden und Namen.** `#include <…>` bindet jede Datei je Namensraum nur einmal ein; Bibliotheken dürfen sich also gegenseitig einbinden. Dieselben Namen können bei verschiedenen Systemen verschiedene Adressen haben (z. B. `SPIN_SPOUT` beim C128 und beim MEGA65). Binde deshalb nur die Bibliothek deines Zielsystems ein, sonst meldet der Assembler „Symbol doppelt definiert“. Willst du mehrere Systeme in einer Datei, gib jeder einen Namensraum: `#include <c128/kernal.asm> as c128` und dann `jsr c128.SPIN_SPOUT`.

**MEGA65-ROMs.** Die Sprungtabelle (`mega65/kernal.asm`) ist in allen ROM-Versionen gleich. Variablen und interne Adressen hängen von der ROM-Version ab: binde genau eine der Dateien `mega65/rom_*.asm` ein (zwei gleichzeitig ergeben einen Fehler). Im Quelltext lässt sich die Version mit `.if MEGA65_ROM == 920418` abfragen; für die OpenROMs ist der Wert 1.

## Beispiel: Konstanten und Makros benutzen

```asm
#include <c64/hardware.asm>
#include <c64/kernal.asm>
#include <common/types.asm>

        lda #COL_RED                 ; Farbname statt Zahl
        sta VIC_BORDERCOLOR          ; Register mit Standardnamen
        ld16 $fb, $0400              ; 16-Bit-Wert nach $fb/$fc
        set_bits $d015, %00000011    ; Sprites 0 und 1 einschalten
        jsr CHROUT
;=* a9 02 8d 20 d0 a9 00 85 fb a9 04 85 fc
```

Die Chip-Register stehen außerdem ohne `#include` als `chip.register` zur Verfügung (zum Beispiel `vic2.border`), solange das Zielsystem sie kennt; **F1** auf einem Wort zeigt die Beschreibung.

Zurück zu: [Assembler-Handbuch](help:assembler), [Makros](help:asm-macros), [Spickzettel](help:asm-index).

## Noten und Frequenztabellen

Jedes System mit Tonerzeugung hat eine Datei `notes.inc` mit benannten Notenkonstanten und der passenden Frequenztabelle des Klangchips:

```
#include <c64/notes.inc>        ; oder c128, mega65, plus4, vic20, atari8bit
```

Die Dateien werden von `tools/gen_notes.py` aus den physikalischen Formeln erzeugt (gleichstufig, A4 = 440 Hz) und nicht von Hand geändert; `python3 tools/gen_notes.py --check` prüft, ob sie noch zum Skript passen.

**Konstanten.** Der Notenwert zählt wie im Tracker: `NOTE_C0 = 1`, `NOTE_A4 = 58` (440 Hz), `NOTE_B7 = 96`. Die Namen sind englisch: `NOTE_CS4` ist Cis (C#), `NOTE_DB4` Des – derselbe Ton wie `NOTE_CS4`; `NOTE_B4` ist das deutsche H, `NOTE_BB4` das deutsche B (`NOTE_AS4`). Mit Rechnung bekommst du Intervalle: `NOTE_C4 + 7` ist die Quinte, `NOTE_C4 + 12` die nächste Oktave. Dazu `NOTE_HOLD = 0` (Note halten), `NOTE_REST = 96` (Pause), `NOTE_FIRST`, `NOTE_LAST`. Im Liedformat der Spielroutinen (`sound.asm`, `music.inc`) ist 96 die Pause: als Liednote kann man deshalb `NOTE_C0` bis `NOTE_AS7` (1..95) schreiben, `NOTE_B7` nur als Tabellenindex.

**Tabellen.** Sie sind mit dem Notenwert indiziert (Eintrag 0 = keine Note). PAL ist die Voreinstellung; `vcc asm -D NTSC=1` (oder `.const NTSC = 1` vor dem Include) wählt NTSC. `NOTES_NO_TABLE` (vorher definiert) bindet nur die Konstanten ein. Mehrfaches Einbinden ist harmlos.

| System | Tabellen | Wertebereich und Grenzen |
|---|---|---|
| C64, C128, MEGA65 | `note_sid_lo`, `note_sid_hi` | 16-Bit-Frequenzzähler `Fn = f · 2²⁴ / Takt` (PAL 985248 Hz, NTSC 1022727 Hz). `NOTE_TABLE_HIGHEST`: PAL `NOTE_AS7`, NTSC `NOTE_B7`; darüber ist der Zähler voll (65535) |
| Plus/4 | `note_ted_lo`, `note_ted_hi` | 10-Bit-Tonwert `N = 1024 − Takt/8 / f`. Unter etwa 108 Hz ist der TED am Ende: Töne unter `NOTE_TABLE_LOWEST` (A2) stehen um ganze Oktaven höher |
| VC-20 | `note_vic_sopr`, `note_vic_alto`, `note_vic_bass` | Registerwerte `$80 \| N` für `$900C`, `$900B`, `$900A`; jede Stimme etwa sieben Oktaven, Töne außerhalb sind oktaviert |
| Atari 8 Bit | `note_pokey8`, `note_pokey16_lo`, `note_pokey16_hi` | 8 Bit (64 kHz): ab `NOTE_POKEY8_LOWEST` (PAL C3, NTSC C#3), darunter `$FF`; 16 Bit (gekoppelte Kanäle): alle 96 Noten |

Beispiel, eine C-Dur-Tonleiter auf SID-Stimme 1 (C64): die Tonhöhe holen und an `$D400`/`$D401` schreiben.

```
#include <c64/notes.inc>

scale:  ldy #0
@next:  ldx major,y             ; Notenwert
        lda note_sid_lo,x
        sta $d400
        lda note_sid_hi,x
        sta $d401
        ; … Gate setzen und warten …
        iny
        cpy #8
        bne @next
        rts
major:  .byte NOTE_C4, NOTE_D4, NOTE_E4, NOTE_F4, NOTE_G4, NOTE_A4, NOTE_B4, NOTE_C5
```
