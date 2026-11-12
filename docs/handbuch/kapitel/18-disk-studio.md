---
id: disk-studio
titel: Disk-Studio, Segmente, ROMs
reihenfolge: 18
stufe: Fortgeschritten
zeit: 30 Minuten
siehe_auch: [bauen-starten, assembler, debugger]
referenz: [disks, cartridges, roms, memmap, asm-directives]
---

# Disk-Studio, Segmente, ROMs

**Ziel:** Du öffnest Diskettenabbilder, holst Dateien daraus in dein Projekt und lässt dein Programm auf eine Diskette schreiben. Du ordnest den Speicher deines Programms mit **Segmenten** und weißt, wo VintageCoder die **ROM-Dateien** deiner Systeme sucht.

Dieses Kapitel ist **fortgeschritten**: Es nimmt an, dass du schon Assemblerprogramme baust ([Assembler für Einsteiger](kapitel:assembler)) und weißt, wie Speicheradressen aussehen.

## Das brauchst du

- ein geöffnetes Projekt (das Disk-Studio öffnet nur in einem Projekt, weil Dateien aus dem Abbild dorthin übernommen werden),
- für das Beispiel das Projekt `examples/c64/prallball`.

## Teil 1: Das Disk-Studio

Früher kamen Programme auf **Disketten**. Heute gibt es dafür **Diskettenabbilder** (englisch *Disk Images*), Dateien, die den Inhalt einer Diskette Sektor für Sektor enthalten. Emulatoren und viele Geräte lesen sie. Das Disk-Studio kennt vier Formate:

| Format | Rechner/Laufwerk | Aufbau |
|---|---|---|
| D64 | C64, Laufwerk 1541 | 35 Spuren, 683 Sektoren |
| D71 | C64/C128, Laufwerk 1571 | 70 Spuren (zwei Seiten), 1366 Sektoren |
| D81 | C64/C128/MEGA65, Laufwerk 1581 | 80 Spuren zu 40 Sektoren, 3200 Sektoren |
| ATR | Atari mit DOS 2 | 720 Sektoren zu 128 Byte |

> **Info:** Eine Diskette ist in **Spuren** (Ringe) und **Sektoren** (Tortenstücke, meist 256 Byte) eingeteilt. Eine Datei ist eine **Kette** von Sektoren: Jeder Sektor verrät, wo der nächste liegt. Eine Übersichtstabelle, bei Commodore **BAM** (Block Availability Map), bei Atari **VTOC**, merkt sich, welche Sektoren belegt sind.

### Ein Abbild öffnen

Es gibt drei Wege:

- Rechtsklick auf eine Image-Datei im Projektbaum ▸ **Diskettenimage öffnen …**,
- das **…**-Menü in der Kopfzeile des Projektbaums ▸ **Diskettenimage öffnen …** (du wählst eine beliebige Datei),
- auf der **Projekt-Übersicht** die Schaltfläche **Disk-Image**.

![Disk-Studio mit Verzeichnis, Belegungskarte und Sektor-Inspektor](bild:disk-studio "Das Disk-Studio: ① Verzeichnis, ② Belegungskarte, ③ Sektor-Inspektor, ④ Übernahme in das Projekt")

### Was du im Studio siehst

- **Verzeichnis:** alle Dateien mit Name (bei Commodore in **PETSCII**, bei Atari in **ATASCII**, den Zeichensätzen dieser Rechner), Art (PRG, SEQ …) und Größe. Du kannst Dateien aus dem Dateimanager auf die Fläche **ziehen**, dann werden sie auf die Diskette geschrieben.
- **Belegung:** eine Karte aus Spuren und Sektoren. Farben unterscheiden Boot oder VTOC, Kopf oder BAM, Verzeichnis, Sektoren einer Datei, belegte Sektoren ohne Datei und Sektoren, die von mehreren Ketten erreicht werden (ein Fehler). Ein Klick wählt den Sektor.
- **Sektor-Inspektor:** zeigt die Bytes des Sektors und was der Kettenzeiger sagt. **Folgen ▶** springt zum nächsten Sektor der Kette, **◀ Zurück** geht den Weg zurück.

### Die Knöpfe

| Knopf | Wirkung |
|---|---|
| **Neu …** | legt ein leeres Abbild an: Format wählen (D64, D71, D81, ATR), bei Commodore Name und ID eingeben, **Anlegen** |
| **Öffnen …** | lädt ein Abbild |
| **Speichern** | schreibt das Abbild (aktiv, wenn es geändert wurde und schon eine Datei hat) |
| **Exportieren …** | speichert eine Kopie unter neuem Namen |
| **Validieren** | prüft BAM/VTOC, Verzeichnis und Dateiketten und meldet Abweichungen |
| **Im Emulator einlegen (Laufwerk 8)** | startet den Standard-Emulator mit dieser Diskette (bei Atari **(D1:)**) |

Das Abbild muss vor „Im Emulator einlegen“ gespeichert sein. **Ungespeicherte Änderungen gehen beim Schließen verloren**, der Studio-Fuß weist darauf hin.

![Validierung eines Abbilds ohne Fehler](bild:disk-validieren "Validieren prüft BAM, Verzeichnis und Dateiketten und meldet die Befunde")

![Dialog „Neue Diskette“ mit Formatwahl, Name und ID](bild:disk-neu "Neue Diskette: Format, Name und ID wählen")

### Dateien ins Projekt übernehmen

Du hast eine alte Diskette (zum Beispiel aus einem Archiv) und möchtest Programme aus ihr in dein Projekt holen:

1. Setze links in der Liste ein **Häkchen** bei jeder Datei, die du übernehmen willst. Die Knöpfe **Alle**, **Keine**, **Nur Programme** und **Nur BASIC** wählen mehrere auf einmal.
2. Gib den **Zielordner** an (er muss innerhalb des Projekts liegen, Vorgabe `import`).
3. Optionen:
   - **BASIC als Quelltext:** ein BASIC-Programm (auf Diskette in einer Kurzform, den **Token**) wird in lesbaren, nummerierten Text zurückübersetzt.
   - **Original behalten:** zusätzlich die unveränderte Datei.
   - **zusätzlich als Assembler:** Maschinenprogramme als Quelltext. Das ist rein mechanisch, Daten erscheinen als Befehle und müssen von Hand geprüft werden.
4. Klicke **n Datei(en) übernehmen**.

### Diskette beim Bauen schreiben lassen

Ein Projekt kann sein Abbild beim Bauen schreiben lassen. In der Projektdatei steht dann ein Abschnitt `disk`; im Projekt-Assistenten wählst du dafür „Diskette“. Das Beispielprojekt `examples/c64/prallball` hat zum Beispiel:

```json
"disk": {
  "format": "d64"
}
```

1. Öffne `examples/c64/prallball`, baue mit **F7**. Neben dem Programm entsteht `build/disk.d64`.
2. Öffne `build/disk.d64` im Disk-Studio (Rechtsklick ▸ **Diskettenimage öffnen …**). Im Verzeichnis steht `PRALLBALL`.
3. Drücke **Validieren**: Es sollte „keine Fehler“ erscheinen.
4. Mit **Im Emulator einlegen (Laufwerk 8)** startest du VICE mit der Diskette.

Mit der Kommandozeile geht dasselbe: `vcc disk list build/disk.d64` zeigt das Verzeichnis, `vcc disk extract build/disk.d64 PRALLBALL -o datei.prg` holt eine Datei zurück.

## Teil 2: Segmente und Bänke

Dein Programm besteht aus Blöcken von Bytes an festen Adressen. Jeder Block heißt **Segment**: Jedes `.org` beginnt ein neues. Mit `.segment` gibst du einem Block einen **Namen** und einen **erlaubten Bereich**. Verlässt ein Byte diesen Bereich, bricht der Bau mit der Meldung **E301** ab. So fängst du früh Fehler wie „Code läuft in meine Daten hinein“.

```asm
.target c64
.cpu 6510 { illegals = deny }

.segment "CODE", $0801, $0fff       ; Programm: muss zwischen $0801 und $0FFF bleiben
.org $0801
.basicstub start
start:  lda #5
        sta $d020
        jmp *

.segment "DATEN", $c000, $c0ff      ; Tabelle in ihrem eigenen Bereich
.org $c000
tabelle: .byte 1,2,3,4
```

Die Namen erscheinen im Bauprotokoll, im Debug-Info und in der **Memory Map** (Reiter im Inspektor, nach einem Bau; siehe [Der Debugger](kapitel:debugger) und die Referenz [Memory Map](hilfe:memmap)). Die Memory Map zeigt den Adressraum als Raster aus 256 Seiten zu je 256 Byte, gefärbt nach Art des Speichers (RAM, ROM, E/A …) und überlagert mit deinen Segmenten. Ein Klick auf eine Seite nennt das Segment, das sie belegt.

![Memory Map mit Segmenten und belegten Seiten](bild:segmente-baenke "Die Memory Map zeigt, welche Segmente welche Speicherseiten belegen")

> **Achtung:** Beim C64-PRG-Format wird alles zu **einem** Block ab der niedrigsten Adresse; Lücken füllt der Assembler mit Nullbytes. Das obige Beispiel ergibt deshalb eine Datei von rund 47 KB, obwohl nur wenige Bytes Inhalt da sind. Beim Atari-XEX bleibt jedes Segment ein eigener Block.

### Module und Bänke

Ein **Modul** (englisch *Cartridge*) ist ein Steckmodul mit Festwertspeicher (ROM): Der Rechner sieht seinen Inhalt sofort, das Programm startet beim Einschalten. Module sind oft größer als der Adressraum, den der Rechner dafür bereithält (beim C64 8 KB bei `$8000`). Der Rest steckt in weiteren **Bänken**; zur Laufzeit schaltet das Programm eine Bank ein, und ihr Inhalt erscheint an derselben Adresse.

- `.cartridge "typ", "NAME"` macht aus dem Programm ein Modulabbild (`.crt` beim C64, `.car` beim Atari).
- `.bank n` wählt die Bank (0 bis 1023). Jede Bank hat ihren eigenen Programmzähler. Bänke gibt es nur in Modulformaten.
- Das Beispiel `examples/c64/cart_demo` hat zwei 8-KB-Bänke und schaltet im RAM zwischen ihnen um. Mit **F7** entsteht `build/cart_demo.crt`.

Alle Direktiven, Modultypen und Beispiele stehen in der Referenz [Segmente, Bänke, Module](hilfe:cartridges).

## Teil 3: ROM-Dateien

Ein **ROM** (*Read-Only Memory*) ist ein Festspeicher, der ein Betriebssystem, BASIC oder einen Zeichensatz enthält. Emulatoren brauchen die **Abbilder** dieser ROMs, um den echten Rechner nachzubilden.

> **Achtung:** VintageCoder liefert **keine Original-ROMs** mit und lädt auch keine aus dem Netz. Du legst deine **eigenen** Abbilder ab. Ob du ein Abbild benutzen darfst, klärst du selbst; hier steht keine Rechtsberatung.

VintageCoder erkennt ROM-Dateien am **Inhalt** (Größe und Prüfsumme), der Dateiname ist egal.

### ROMs einrichten

1. Wähle **Datei ▸ ROMs einrichten …** (oder auf der Willkommenseite den entsprechenden Eintrag). Der Assistent führt in drei Schritten durch.
2. Alternativ: **Einstellungen ▸ ROMs**. Die Seite zeigt den ROM-Ordner und den **Status je Rolle**, dazu **Ordner öffnen**, **Neu prüfen** und **Auswählen …**. Du kannst ROM-Dateien oder **ZIP-Archive** auf die Ablagezone ziehen, die Vorschau bestätigen, fertig.
3. **ROMs aus VICE importieren …** zeigt die in VICE gefundenen Abbilder und kopiert sie nach Bestätigung in den ROM-Ordner.
4. Im Menü **Datei** gibt es außerdem **ROM-Ordner öffnen** und **ROMs prüfen**.

![ROM-Seite der Einstellungen: ① Status je Rolle, ③ Knöpfe](bild:rom-verzeichnis "Einstellungen ▸ ROMs: Ordner, Status je Rolle und Knöpfe; die Ablagezone liegt weiter unten auf der Seite")

### Wo sucht VintageCoder?

Die erste Fundstelle je Rolle gilt:

1. Einstellung für eine einzelne Datei (zum Beispiel atari800 „OS-ROM“),
2. `roms/<system>/` im **Projekt** (steht in `.gitignore`),
3. `roms/<system>/` **neben dem Programm** (portabler Modus),
4. der **Benutzerordner**: Windows `%APPDATA%\vintagecoder\roms`, macOS `~/.config/vintagecoder/roms`, Linux `$XDG_CONFIG_HOME/vintagecoder/roms` (sonst `~/.config/vintagecoder/roms`),
5. die alte Ablage `~/.config/vintagecoder/mega65/MEGA65.ROM`,
6. nur auf Wunsch (Schalter **ROMs anderer Programme mitbenutzen**, Standard aus): ROM-Ordner von VICE, Xemu und atari800, nur lesend.

Die Unterordner heißen `c64`, `c128`, `vic20`, `plus4`, `atari8` und `mega65`. Üblich sind:

| System | Dateien (Größe in Byte) |
|---|---|
| C64 | `kernal` (8192), `basic` (8192), `chargen` (4096) |
| Atari 8 Bit | ein Betriebssystem genügt: `os-xl` (16384), `os-b` oder `os-a` (je 10240); dazu `basic` (8192) |
| MEGA65 | `rom` (131072) |

> **Info:** **Xemu** (der MEGA65-Emulator) braucht eine ROM-Datei von **genau 131072 Byte**. VICE und atari800 benutzen eigene Ersatz-ROMs, wenn keine da sind.

Hast du mehrere Abbilder derselben Rolle, gewinnt die Standardrevision. In der ROM-Seite lässt sich eine Datei **festnageln**. Eine Datei mit passendem Namen und Größe, aber unbekannter Prüfsumme gilt als **Unbekannte Revision** und wird mit Warnung benutzt.

Von der Kommandozeile gibt es `vcc roms list`, `check`, `path`, `where` und `import`.

## Häufige Stolpersteine

- **„ROM … nicht gefunden“ beim Start:** Lege die Datei ab (Fehlerdialog ▸ „ROMs einrichten …“). Die Meldung nennt Zielordner, erwartete Größe und alle gesuchten Orte.
- **Ein Zeichensatz sieht „falsch“ aus:** Meist zeigt der Editor die eingebaute Ersatzschrift, weil das ROM fehlt. Ablegen und **Neu prüfen**.
- **E301 beim Bauen:** Ein Segment überschreitet seinen Bereich oder überlappt ein anderes. Das Bauprotokoll nennt Segment und Adressen.
- **Die Diskette im Emulator ist leer:** Das Abbild war nicht gespeichert, als du „Im Emulator einlegen“ gedrückt hast.
- **Eine Datei lässt sich nicht übernehmen:** Der Zielordner muss im Projekt liegen (keine absoluten Pfade, kein `..`).

## Wie geht es weiter?

Mit den Systemunterschieden beschäftigt sich [Zielsysteme im Vergleich](kapitel:systeme). Nachschlagen: [Disk-Studio](hilfe:disks), [Segmente, Bänke, Module](hilfe:cartridges), [ROM-Dateien](hilfe:roms), [Memory Map](hilfe:memmap).
