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

## Wofür ist das Studio gedacht?

Das Disk-Studio zeigt und bearbeitet **Diskettenabbilder**. Du brauchst es, wenn dein Programm von Diskette laden soll, wenn du Dateien aus einem alten Archiv holen willst oder wenn du prüfen möchtest, ob dein Abbild in Ordnung ist.

Typische Einsätze aus echten Retro-Projekten:

- **Ein Spiel auf Diskette ausliefern:** Hauptprogramm, Level, Musik und Titelbild als Dateien auf einem D64.
- **Ein altes Archiv ansehen:** Was steht auf der Diskette, was davon kann ich in mein Projekt übernehmen (BASIC als lesbarer Text)?
- **Fehler suchen:** Warum lädt das Programm nicht? Die Belegungskarte und **Validieren** zeigen kaputte Ketten.
- **Eine Boot-Diskette** für Atari ohne Dateisystem prüfen.

Im fertigen Programm ist das ein **Abbild** (`.d64`, `.d71`, `.d81`, `.atr`), das ein Emulator oder ein Diskettenersatz (SD-Karte) wie eine echte Diskette einlegt.

## Grundbegriffe ohne Vorwissen

- **Diskettenabbild:** eine Datei, die den Inhalt einer Diskette Sektor für Sektor enthält.
- **Spur und Sektor:** Ringe und Tortenstücke einer Diskette; ein Sektor hat meist 256 Byte.
- **BAM / VTOC:** die Belegungstabelle (**B**lock **A**vailability **M**ap bei Commodore, VTOC bei Atari): welche Sektoren frei sind.
- **Verzeichnis:** die Liste der Dateien auf der Diskette.
- **Kette:** eine Datei ist eine Folge von Sektoren, jeder verweist auf den nächsten.
- **PRG / SEQ:** Dateiarten bei Commodore (Programm / Textfolge).
- **Interleave:** der Abstand, mit dem aufeinanderfolgende Sektoren einer Datei auf der Diskette liegen.
- **Segment, Bank:** Abschnitte des Programmspeichers (Teil 2).
- **Speicherkarte (Memory Map):** Übersicht, welcher Speicher belegt ist.

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

Jede Image-Datei (`.d64`, `.d71`, `.d81`, `.atr`) öffnet im Disk-Studio. Es gibt mehrere Wege:

- **Doppelklick** auf die Datei in der linken Spalte (oder Rechtsklick ▸ **Öffnen**),
- **Cmd/Strg+K** und den Dateinamen eingeben,
- **Datei ▸ Datei öffnen …** (du wählst eine beliebige Datei).

![Das Disk-Studio mit einer D64-Datei: oben das Verzeichnis, darunter die Belegungskarte der Spuren mit der Kette der gewählten Datei, rechts die Karten Zustand, Sektor und „In das Projekt übernehmen“.](bild:disk-studio "Das Disk-Studio: Verzeichnis, Belegungskarte, Sektoranzeige und Übernahme in das Projekt")

### Was du im Studio siehst

- **Verzeichnis:** alle Dateien mit Name (bei Commodore in **PETSCII**, bei Atari in **ATASCII**, den Zeichensätzen dieser Rechner), Art (PRG, SEQ …) und Größe. Du kannst Dateien aus dem Dateimanager auf die Fläche **ziehen**, dann werden sie auf die Diskette geschrieben.
- **Belegung:** eine Karte aus Spuren und Sektoren. Farben unterscheiden Boot oder VTOC, Kopf oder BAM, Verzeichnis, Sektoren einer Datei, belegte Sektoren ohne Datei und Sektoren, die von mehreren Ketten erreicht werden (ein Fehler). Ein Klick wählt den Sektor.
- **Karte „Sektor“:** zeigt die Bytes des Sektors und was der Kettenzeiger sagt. **Folgen ▶** springt zum nächsten Sektor der Kette, **◀ Zurück** geht den Weg zurück.

### Aufbau der Oberfläche und die Knöpfe

Das Studio folgt den sechs Zonen. Die Aktionen stehen in der Kopfzeile, in der Leiste, im Kontextmenü der Fläche und in der Befehlspalette (**Cmd/Strg+K**, „Studio: …“):

| Aktion | Wirkung |
|---|---|
| **Speichern** (**Cmd/Strg+S**) | schreibt das Abbild |
| **Speichern unter …** | speichert eine Kopie unter neuem Namen |
| **Validieren** | prüft BAM/VTOC, Verzeichnis und Dateiketten und meldet die Befunde |
| **Einlegen (Laufwerk 8)** / **Einlegen (D1:)** | startet den Standard-Emulator des Systems mit dieser Diskette |
| **Datei hinzufügen …** | legt eine Datei auf die Diskette (oder du ziehst sie ins Fenster) |
| **Folgen ▶** | folgt dem Kettenzeiger zum nächsten Sektor |
| **Zurück** | zum zuvor besuchten Sektor |
| **Kette anzeigen / ausblenden** | zeichnet die Kette der gewählten Datei in der Belegungskarte |
| **In das Projekt übernehmen** | holt gewählte Dateien ins Projekt (siehe unten) |

Neue leere Diskette: **＋** in der linken Spalte ▸ im Dialog **Neue Datei** die Art **Diskette D64**, **D71**, **D81** oder **ATR**, Namen eingeben, **OK**.

Das Abbild muss vor „Einlegen“ gespeichert sein. **Ungespeicherte Änderungen gehen beim Schließen verloren**, das Studio zeigt „Image ungespeichert“.

![Das Disk-Studio nach „Validieren“: die Karte „Zustand“ meldet, dass BAM, Verzeichnis und Ketten übereinstimmen.](bild:disk-validieren "Validieren prüft BAM, Verzeichnis und Dateiketten und meldet die Befunde")

![Der Dialog „Neue Datei“ mit gewählter Art „Diskette D64“, dem Ordner und dem Feld Name.](bild:disk-neu "Neue Diskette: im Dialog „Neue Datei“ die Art Diskette wählen")

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
2. Öffne `build/disk.d64` im Disk-Studio (Doppelklick in der linken Spalte). Im Verzeichnis steht `PRALLBALL`.
3. Drücke **Validieren**: Es sollte „keine Fehler“ erscheinen.
4. Mit **Einlegen (Laufwerk 8)** startest du VICE mit der Diskette.

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

Die Namen erscheinen im Protokoll, im Debug-Info und in der **Memory Map** (Panel **Karte** im rechten Panel, nach einem Bau; siehe [Der Debugger](kapitel:debugger) und die Referenz [Memory Map](hilfe:memmap)). Die Memory Map zeigt den Adressraum als Raster aus 256 Seiten zu je 256 Byte, gefärbt nach Art des Speichers (RAM, ROM, E/A …) und überlagert mit deinen Segmenten. Ein Klick auf eine Seite nennt das Segment, das sie belegt.

![Das rechte Panel mit der Karte neben Editor und Debug-Schublade (Beispielansicht).](bild:segmente-baenke "Beispielansicht, es läuft kein Emulator: die Memory Map zeigt, welche Segmente welche Speicherseiten belegen")

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

1. Wähle **Hilfe ▸ Konfigurationsassistent: nur ROMs …**. Der Assistent führt dich durch die Einrichtung.
2. Alternativ: **Einstellungen ▸ ROMs**. Die Seite zeigt den ROM-Ordner und den **Status je Rolle**, dazu **Ordner öffnen**, **Neu prüfen** und **Auswählen …**. Du kannst ROM-Dateien oder **ZIP-Archive** auf die Ablagezone ziehen, die Vorschau bestätigen, fertig.
3. **ROMs aus VICE importieren …** zeigt die in VICE gefundenen Abbilder und kopiert sie nach Bestätigung in den ROM-Ordner.

![Einstellungen, Seite „ROMs“ (Ausschnitt): je System ein Reiter, je Rolle der Status, die erwartete Größe und der Knopf „Auswählen …“.](bild:rom-verzeichnis "Einstellungen ▸ ROMs: Status je Rolle und Knopf „Auswählen …“; ROM-Ordner und Ablagezone stehen darüber und darunter auf der Seite")

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

## Mein erstes Disk-Image (am Beispiel `prallball`)

1. **Beispiel öffnen:** Startseite ▸ Beispiele ▸ `examples/c64/prallball`. *Erwartet:* Das Projekt ist offen; in der Projektdatei ist eine D64-Diskette eingetragen.
2. **Bauen:** **F7**. *Erwartet:* Das Protokoll meldet, dass die Diskette geschrieben wurde.
3. **Abbild öffnen:** Doppelklick auf die gebaute `.d64`-Datei (unter `build/`). *Erwartet:* Das Disk-Studio zeigt das Verzeichnis und die Belegungskarte.
4. **Datei ansehen:** eine Datei im Verzeichnis wählen. *Erwartet:* Die Belegungskarte färbt ihre Sektoren, die Karte „Sektor“ zeigt die Bytes.
5. **Validieren.** *Erwartet:* „Validierung: 0 Problem(e)“.
6. **Einlegen (Laufwerk 8).** *Erwartet:* Der Emulator startet mit der Diskette.

## Hardware-Besonderheiten und Grenzen

- **D64** hat nur 683 Sektoren (rund 170 KB); mehr passt nicht. **D81** fasst 800 KB.
- Dateinamen auf Commodore-Disketten sind höchstens **16 Zeichen** lang und stehen in PETSCII; auf Atari **8+3** in ATASCII.
- Eine **Boot-Diskette** (Atari) hat keine Dateien, nur Boot-Sektoren.
- Beim echten Laufwerk bestimmt der **Interleave** die Ladezeit.

## Nachbar-Studios und Glossar

- Das Programm auf der Diskette entsteht mit dem [Assembler](kapitel:assembler) und den Studios für Grafik und Klang; die Daten kommen aus deren Exporten.
- Die Speicherkarte (Panel „Karte“) zeigt, wo dein Programm im Speicher liegt, siehe Teil 2.
- Begriffe: Glossar (Datei `GLOSSAR.md`): Diskettenabbild, Sektor, BAM, Kette.

## Häufige Stolpersteine

- **„ROM … nicht gefunden“ beim Start:** Lege die Datei ab (**Hilfe ▸ Konfigurationsassistent: nur ROMs …**). Die Meldung nennt Zielordner, erwartete Größe und alle gesuchten Orte.
- **Ein Zeichensatz sieht „falsch“ aus:** Meist zeigt der Editor die eigenen Standardsatz von VintageCoder, weil das ROM fehlt. Ablegen und **Neu prüfen**.
- **E301 beim Bauen:** Ein Segment überschreitet seinen Bereich oder überlappt ein anderes. Das Protokoll nennt Segment und Adressen.
- **Die Diskette im Emulator ist leer:** Das Abbild war nicht gespeichert, als du „Einlegen“ gedrückt hast.
- **Eine Datei lässt sich nicht übernehmen:** Der Zielordner muss im Projekt liegen (keine absoluten Pfade, kein `..`).

## Wie geht es weiter?

Mit den Systemunterschieden beschäftigt sich [Zielsysteme im Vergleich](kapitel:systeme). Nachschlagen: [Disk-Studio](hilfe:disks), [Segmente, Bänke, Module](hilfe:cartridges), [ROM-Dateien](hilfe:roms), [Memory Map](hilfe:memmap).
