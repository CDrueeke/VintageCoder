---
id: disk-studio
titel: Disk-Editor, Segmente, ROMs
reihenfolge: 18
stufe: Fortgeschritten
zeit: 30 Minuten
siehe_auch: [bauen-starten, assembler, debugger]
referenz: [disks, cartridges, roms, memmap, asm-directives]
---

# Disk-Editor, Segmente, ROMs

**Ziel:** Du öffnest Diskettenabbilder, holst Dateien daraus in dein Projekt und lässt dein Programm auf eine Diskette schreiben. Du ordnest den Speicher deines Programms mit **Segmenten** und weißt, wo VintageCoder die **ROM-Dateien** deiner Systeme sucht.

Dieses Kapitel ist **fortgeschritten**: Es nimmt an, dass du schon Assemblerprogramme baust ([Assembler für Einsteiger](kapitel:assembler)) und weißt, wie Speicheradressen aussehen.

## Das brauchst du

- ein Diskettenabbild zum Ansehen (`.d64`, `.d71`, `.d81` oder `.atr`) oder eine neue, leere Diskette, die du im Disk-Editor anlegst,
- ein geöffnetes Projekt, wenn du Dateien aus dem Abbild ins Projekt übernehmen willst (zum Ansehen, Prüfen und Anlegen genügt der Disk-Editor allein),
- für das Beispiel das Projekt `examples/c64/prallball`.

## Wofür ist der Editor gedacht?

Der Disk-Editor zeigt und bearbeitet **Diskettenabbilder**. Du brauchst ihn, wenn dein Programm von Diskette laden soll, wenn du Dateien aus einem alten Archiv holen willst oder wenn du prüfen möchtest, ob dein Abbild in Ordnung ist.

Typische Einsätze aus echten Retro-Projekten:

- **Ein Spiel auf Diskette ausliefern:** Hauptprogramm, Level, Musik und Titelbild als Dateien auf einem D64.
- **Ein altes Archiv ansehen:** Was steht auf der Diskette, was davon kann ich in mein Projekt übernehmen (BASIC als lesbarer Text)?
- **Fehler suchen:** Warum lädt das Programm nicht? Die Belegungskarte und **Prüfen** zeigen kaputte Ketten.
- **Eine Boot-Diskette** für Atari ohne Dateisystem ansehen.

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

## Teil 1: Der Disk-Editor

Früher kamen Programme auf **Disketten**. Heute gibt es dafür **Diskettenabbilder** (englisch *Disk Images*), Dateien, die den Inhalt einer Diskette Sektor für Sektor enthalten. Emulatoren und viele Geräte lesen sie. Der Disk-Editor kennt vier Formate:

| Format | Rechner/Laufwerk | Aufbau |
|---|---|---|
| D64 | C64, Laufwerk 1541 | 35 Spuren, 683 Sektoren |
| D71 | C64/C128, Laufwerk 1571 | 70 Spuren (zwei Seiten), 1366 Sektoren |
| D81 | C64/C128/MEGA65, Laufwerk 1581 | 80 Spuren zu 40 Sektoren, 3200 Sektoren |
| ATR | Atari mit DOS 2 | 720 Sektoren zu 128 Byte |

> **Info:** Eine Diskette ist in **Spuren** (Ringe) und **Sektoren** (Tortenstücke, meist 256 Byte) eingeteilt. Eine Datei ist eine **Kette** von Sektoren: Jeder Sektor verrät, wo der nächste liegt. Eine Übersichtstabelle, bei Commodore **BAM** (Block Availability Map), bei Atari **VTOC**, merkt sich, welche Sektoren belegt sind.

### Ein Abbild öffnen

Der Disk-Editor wohnt im Hauptbereich **Daten**, in der zweiten Reihe unter **Disk**. Solange keine Diskette offen ist, steht dort „Keine Diskette geöffnet“ mit den Knöpfen **Diskette öffnen …** und **Neue Diskette …**. Ein Abbild öffnest du auf diesen Wegen:

- **Klick auf die Datei** (`.d64`, `.d71`, `.d81`, `.atr`) im Projektbaum: Der Disk-Editor öffnet sich von selbst,
- der Knopf **Diskette öffnen …** im Disk-Editor (du wählst eine beliebige Abbild-Datei),
- die Abbild-Datei aus Finder oder Explorer auf das Fenster ziehen,
- **Baum ▸ Diskette öffnen …** (Mac-Menü; sonst über die Befehlspalette **Cmd/Strg+K**).

Es ist immer nur **eine** Diskette offen. Öffnest du eine zweite, wird die erste geschlossen; hat sie ungespeicherte Änderungen, fragt der Disk-Editor vorher nach: **Speichern**, **Verwerfen** oder **Abbrechen**.

![Der Bereich „Daten“ mit der Seite „Disk“ und dem Diskettenabbild beispiel.d64. Links die Karten „DISKETTE“, „BELEGUNG“ und „AKTIONEN“, in der Mitte das Verzeichnis mit den drei Dateien HUGO, GUMBO und MUSIK und die Belegungskarte, rechts die Sektoranzeige „SEKTOR 18/0“ als Hexdump.](bild:disk-studio "Der Disk-Editor mit Verzeichnis, Belegungskarte und Sektoranzeige.")

### Was du im Editor siehst

Der Disk-Editor hat drei Karten nebeneinander. In schmalen Fenstern rutscht die rechte Karte nach unten.

**Links: Diskette.**

- das Format (D64, D71, D81 oder ATR; das aktuelle ist hervorgehoben),
- der Dateiname des Abbilds und seine Größe; ein Punkt davor heißt „ungespeichert“; darunter steht der Aufbau (Spuren und Sektoren),
- die **Belegung** als Balken mit der Zahl der belegten und freien Blöcke,
- die **Aktionen** (siehe unten),
- bei Bedarf das Feld **Übernahme** (Zielordner und Optionen) und darunter der **Prüfbericht** und das Ergebnis der Übernahme.

**Mitte: Verzeichnis und Belegungskarte.**

- **Verzeichnis:** alle Dateien mit Name (bei Commodore in **PETSCII**, bei Atari in **ATASCII**, den Zeichensätzen dieser Rechner), Typ, Blöcken, Größe und Startadresse. Ein Schloss zeigt eine geschützte Datei, **Autostart** die Datei, die beim Laden von selbst startet. Ein Klick (oder die Pfeiltasten und Enter) wählt eine Datei.
- **Reihenfolge ändern:** Die Reihenfolge im Verzeichnis bestimmt, in welcher Folge ein Lader oder ein Verzeichnislisting die Dateien zeigt. Ziehe eine Zeile an eine andere Stelle; eine Linie zeigt, wo sie landet. Mit der Tastatur wählst du die Zeile und drückst **Alt+Pfeil hoch** oder **Alt+Pfeil runter**; die Knöpfe **Nach oben** und **Nach unten** über der Liste tun dasselbe. Jeder Zug ist ein Schritt für **Rückgängig** (Cmd/Strg+Z, auch der Knopf neben der Liste) und **Wiederholen**. Es wandern nur die Verzeichniseinträge, die Dateien selbst bleiben unverändert. Bei Atari geht das für Atari DOS 2 und 2.5 (128-Byte-Sektoren) und nur im Hauptverzeichnis; bei anderen Abbildern meldet die Seite, warum es nicht geht. Auch **Dateien ablegen** lässt sich zurücknehmen. Die Änderung steht erst nach **Speichern** in der Datei.
- **Belegungskarte:** eine Zeile je Spur, ein Kästchen je Sektor; große Disketten (ab 41 Spuren) stehen in zwei Blöcken nebeneinander. Die Legende nennt vier Farben: **Gewählte Datei**, **Belegt**, **Verzeichnis** und **Kopf/BAM**. Sektoren mit einem Fehler (zum Beispiel von mehreren Ketten erreicht) sind rot. Mit **Kette zeigen** färbt die Karte alle Sektoren der gewählten Datei. Ein Klick wählt den Sektor; mit den Pfeiltasten wanderst du über die Karte.

**Rechts: der Sektor.**

- Der Kopf nennt Spur und Sektor. Darunter steht, zu welcher Datei der Sektor gehört („Block 2 von 5“).
- Darunter siehst du die Bytes des Sektors in Hex und als Zeichen. Zeigerbytes (der Verweis auf den nächsten Sektor) sind farbig hervorgehoben.
- **Folgen ▶** springt zum nächsten Sektor der Kette, am Ende steht dort „Kettenende“. **Zurück** geht den Weg zurück.
- Unten stehen **Nächster Sektor** und **Bytes in diesem Sektor**.

### Die Knöpfe

Alle Aktionen stehen links in der Karte **Diskette**:

| Aktion | Wirkung |
|---|---|
| **Im Emulator einlegen** | startet den Standard-Emulator des Systems mit dieser Diskette (Laufwerk 8). Das Abbild muss vorher gespeichert sein; sonst ist der Knopf gesperrt („Erst speichern, dann einlegen“). |
| **Dateien ablegen …** | wählt Dateien aus und legt sie auf die Diskette. Bei einer Diskette ohne Dateisystem (Boot-Diskette) ist der Knopf gesperrt. |
| **Prüfen** | prüft BAM bzw. VTOC, Verzeichnis und Dateiketten und schreibt das Ergebnis in den **Prüfbericht**. |
| **Ins Projekt übernehmen** | holt die vorgemerkten Dateien ins Projekt (siehe unten); die Zahl der Vormerkungen steht dahinter. |
| **Speichern** | schreibt das Abbild (nur aktiv bei Änderungen). |
| **Speichern unter …** | speichert eine Kopie unter neuem Namen. |
| **Diskette öffnen …** | öffnet ein anderes Abbild. |
| **Neue Diskette …** | legt eine neue, leere Diskette an. |
| **Diskette schließen** | schließt das Abbild; bei Änderungen kommt die Rückfrage. |

**Cmd/Strg+S** speichert im Disk-Editor nicht; nimm den Knopf **Speichern**.

Neue leere Diskette: **Neue Diskette …**, im Dialog das **Format** (D64, D71, D81 oder ATR), den **Namen der Diskette** (bis 16 Zeichen) und die **Kennung** (2 Zeichen) eingeben, **Anlegen**. Bei ATR gibt es weder Name noch Kennung. Die neue Diskette bleibt „ungespeichert“, bis du sie mit **Speichern** ablegst.

*[Bild folgt in E13: disk-neu]*

**Ungespeicherte Änderungen gehen beim Schließen verloren**, der Editor zeigt „Image ungespeichert“ und fragt vorher nach.

### Eine Diskette prüfen

Klicke **Prüfen**. Im Feld **Prüfbericht** steht danach entweder „Keine Probleme gefunden.“ oder je Befund eine Zeile. Ist etwas auffällig, zeigt die Karte **Diskette** außerdem die Zahl der Probleme an.

*[Bild folgt in E13: disk-validieren]*

### Dateien ins Projekt übernehmen

Du hast eine alte Diskette (zum Beispiel aus einem Archiv) und möchtest Programme aus ihr in dein Projekt holen. Dazu muss ein Projekt geöffnet sein, sonst ist der Knopf gesperrt („Zum Übernehmen muss ein Projekt geöffnet sein.“).

1. Setze in der Spalte **Vormerken** ein **Häkchen** bei jeder Datei, die du übernehmen willst. Die Knöpfe **Alle**, **Keine**, **Programme** und **BASIC** über dem Verzeichnis wählen mehrere auf einmal.
2. Sobald etwas vorgemerkt ist, erscheint links das Feld **Übernahme**. Gib den **Zielordner im Projekt** an. Er muss innerhalb des Projekts liegen, sonst erscheint „Der Zielordner muss im Projekt liegen.“
3. Optionen:
   - **BASIC als Quelltext:** ein BASIC-Programm (auf Diskette in einer Kurzform, den **Token**) wird in lesbaren, nummerierten Text zurückübersetzt.
   - **Originaldatei behalten:** zusätzlich die unveränderte Datei.
   - **Auch als Assembler:** Maschinenprogramme als Quelltext. Das ist rein mechanisch, Daten erscheinen als Befehle und müssen von Hand geprüft werden.
4. Klicke **Ins Projekt übernehmen**. Was geschrieben wurde, steht danach in der Liste **Übernahme**; der Projektbaum liest sich neu ein.

*[Bild folgt in E13: disk-editor-uebernahme]*

### Diskette beim Bauen schreiben lassen

Ein Projekt kann sein Abbild beim Bauen schreiben lassen. In der Projektdatei steht dann ein Abschnitt `disk`; im Dialog „Neues Projekt“ wählst du dafür bei **Diskette** ein Format. Das Beispielprojekt `examples/c64/prallball` hat zum Beispiel:

```json
"disk": {
  "format": "d64"
}
```

Im Bereich **Projekt** zeigt die Übersicht in der Zeile zur Diskette, ob eine eingerichtet ist (zum Beispiel „Diskette D64, 1 Programme“).

1. Öffne `examples/c64/prallball` (Startseite ▸ **Beispiel öffnen**), baue mit **F7**. Neben dem Programm entsteht `build/disk.d64`; das Protokoll nennt die Diskette.
2. Öffne `build/disk.d64`: Klick auf die Datei im Projektbaum. Im Verzeichnis steht `PRALLBALL`.
3. Klicke **Prüfen**: Es sollte „Keine Probleme gefunden.“ erscheinen.
4. Mit **Im Emulator einlegen** startest du den Emulator mit der Diskette.

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

Nach einem Bau listet das rechte Panel **Maschine** unter **Letzter Bau** die Segmente als Tabelle: Nummer, Anfangsadresse (**von**), Endadresse (**bis**) und **Länge**. Daneben stehen Quelle, Programm, Größe und die Startadresse (SYS). Das Panel blendest du mit **Cmd/Strg+Alt+B** ein und aus. Den Adressraum als Raster siehst du in der **Speicherkarte** (Hauptbereich **Daten**, zweite Reihe **Speicher**): 256 Seiten zu je 256 Byte, das Programm aus dem letzten Bau darin eingezeichnet, mit **Filter** (Alle, Programm, RAM, ROM, E/A) und, wo das System es kennt, der **Bankumschaltung**. Die Karte entsteht erst nach einem Bau (**F7**). Die Einzelheiten stehen in der Referenz [Memory Map](hilfe:memmap).

*[Bild folgt in E13: segmente-baenke]*

> **Achtung:** Beim C64-PRG-Format wird alles zu **einem** Block ab der niedrigsten Adresse; Lücken füllt der Assembler mit Nullbytes. Das obige Beispiel ergibt deshalb eine Datei von rund 47 KB, obwohl nur wenige Bytes Inhalt da sind. Beim Atari-XEX bleibt jedes Segment ein eigener Block.

### Module und Bänke

Ein **Modul** (englisch *Cartridge*) ist ein Steckmodul mit Festwertspeicher (ROM): Der Rechner sieht seinen Inhalt sofort, das Programm startet beim Einschalten. Module sind oft größer als der Adressraum, den der Rechner dafür bereithält (beim C64 8 KB bei `$8000`). Der Rest steckt in weiteren **Bänken**; zur Laufzeit schaltet das Programm eine Bank ein, und ihr Inhalt erscheint an derselben Adresse.

- `.cartridge "typ", "NAME"` macht aus dem Programm ein Modulabbild (`.crt` beim C64, `.car` beim Atari).
- `.bank n` wählt die Bank (0 bis 1023). Jede Bank hat ihren eigenen Programmzähler. Bänke gibt es nur in Modulformaten.
- Das Beispiel `examples/c64/cart_demo` hat zwei 8-KB-Bänke und schaltet im RAM zwischen ihnen um. Mit **F7** entsteht `build/cart_demo.crt`.

Alle Direktiven, Modultypen und Beispiele stehen in der Referenz [Segmente, Bänke, Module](hilfe:cartridges).

## Teil 3: ROM-Dateien

Ein **ROM** (*Read-Only Memory*) ist ein Festspeicher, der ein Betriebssystem, BASIC oder einen Zeichensatz enthält. Emulatoren brauchen die **Abbilder** dieser ROMs, um den echten Rechner nachzubilden.

> **Achtung:** VintageCoder liefert **keine Original-ROMs** mit und lädt auch keine aus dem Netz. Du legst deine **eigenen** Abbilder ab.

VintageCoder erkennt ROM-Dateien am **Inhalt** (Größe und Prüfsumme), der Dateiname ist egal.

### ROMs einrichten

Der einfachste Weg ist der Assistent: **Hilfe ▸ Konfigurationsassistent: nur ROMs …** führt dich durch die Einrichtung. Du kannst die Dateien auch selbst in einen der Ordner legen, die der nächste Abschnitt nennt. Dieselbe Arbeit erledigt die Seite **Einstellungen ▸ ROMs** (siehe unten).

### Wo sucht VintageCoder?

Die erste Fundstelle je Rolle gilt:

1. Einstellung für eine einzelne Datei (zum Beispiel atari800 „OS-ROM“),
2. `roms/<system>/` im **Projekt** (steht in `.gitignore`),
3. `roms/<system>/` **neben dem Programm** (portabler Modus),
4. der **Benutzerordner**: Windows `%APPDATA%\vintagecoder\roms`, macOS `~/.config/vintagecoder/roms`, Linux `$XDG_CONFIG_HOME/vintagecoder/roms` (sonst `~/.config/vintagecoder/roms`),
5. die alte Ablage `~/.config/vintagecoder/mega65/MEGA65.ROM`,
6. nur auf Wunsch (Schalter **ROMs anderer Programme mitbenutzen**, Standard aus): ROM-Ordner von VICE, Xemu und atari800, nur lesend.

Die Unterordner heißen `c64`, `c128`, `vic20`, `plus4`, `atari8` und `mega65`. Auf der Seite **Einstellungen ▸ ROMs** siehst du je System und Rolle, ob eine passende Datei gefunden wurde; **ROM-Datei oder Zip wählen …** übernimmt Dateien nach deiner Bestätigung in den ROM-Ordner, **ROM-Ordner öffnen** zeigt ihn und unten stehen die **Durchsuchten Orte**. Üblich sind:

| System | Dateien (Größe in Byte) |
|---|---|
| C64 | `kernal` (8192), `basic` (8192), `chargen` (4096) |
| Atari 8 Bit | ein Betriebssystem genügt: `os-xl` (16384), `os-b` oder `os-a` (je 10240); dazu `basic` (8192) |
| MEGA65 | `rom` (131072) |

> **Info:** **Xemu** (der MEGA65-Emulator) braucht eine ROM-Datei von **genau 131072 Byte**. VICE und atari800 benutzen eigene Ersatz-ROMs, wenn keine da sind.

Hast du mehrere Abbilder derselben Rolle, gewinnt die Standardrevision. Eine Datei mit passendem Namen und Größe, aber unbekannter Prüfsumme gilt als **Unbekannte Revision** und wird mit Warnung benutzt.

Von der Kommandozeile gibt es `vcc roms list`, `check`, `path`, `where` und `import`.

## Mein erstes Disk-Image (am Beispiel `prallball`)

1. **Beispiel öffnen:** Startseite ▸ **Beispiel öffnen** ▸ `examples/c64/prallball`. *Erwartet:* Das Projekt ist offen; in der Projektdatei ist eine D64-Diskette eingetragen.
2. **Bauen:** **F7**. *Erwartet:* Das Protokoll meldet, dass die Diskette geschrieben wurde.
3. **Abbild öffnen:** Klick auf die gebaute `.d64`-Datei (unter `build/`) im Projektbaum. *Erwartet:* Der Bereich Daten ▸ Disk zeigt das Verzeichnis und die Belegungskarte.
4. **Datei ansehen:** eine Datei im Verzeichnis wählen. *Erwartet:* Die Belegungskarte färbt ihre Sektoren (mit **Kette zeigen**), die Karte rechts zeigt die Bytes des Sektors.
5. **Prüfen.** *Erwartet:* Im Prüfbericht steht „Keine Probleme gefunden.“
6. **Im Emulator einlegen.** *Erwartet:* Der Emulator startet mit der Diskette.

## Hardware-Besonderheiten und Grenzen

- **D64** hat nur 683 Sektoren (rund 170 KB); mehr passt nicht. **D81** fasst 800 KB.
- Dateinamen auf Commodore-Disketten sind höchstens **16 Zeichen** lang und stehen in PETSCII; auf Atari **8+3** in ATASCII.
- Eine **Boot-Diskette** (Atari) hat keine Dateien, nur Boot-Sektoren.
- Beim echten Laufwerk bestimmt der **Interleave** die Ladezeit.

## Nachbar-Editoren und Glossar

- Das Programm auf der Diskette entsteht mit dem [Assembler](kapitel:assembler) und den Editoren für Grafik und Klang; die Daten kommen aus deren Exporten.
- Das Panel **Maschine** zeigt nach einem Bau, wo dein Programm im Speicher liegt, siehe Teil 2.
- Begriffe: Glossar (Datei `GLOSSAR.md`): Diskettenabbild, Sektor, BAM, Kette.

## Häufige Stolpersteine

- **„ROM … nicht gefunden“ beim Start:** Lege die Datei ab (**Einstellungen ▸ ROMs** oder **Hilfe ▸ Konfigurationsassistent: nur ROMs …**). Die Meldung nennt Zielordner, erwartete Größe und alle gesuchten Orte.
- **Ein Zeichensatz sieht „falsch“ aus:** Meist zeigt VintageCoder den eigenen Standardsatz, weil das ROM fehlt. Lege das ROM ab und öffne die Ansicht neu.
- **E301 beim Bauen:** Ein Segment überschreitet seinen Bereich oder überlappt ein anderes. Das Protokoll nennt Segment und Adressen.
- **„Im Emulator einlegen“ ist grau:** Das Abbild ist nicht gespeichert (oder hat noch keinen Speicherort). Erst **Speichern**, dann einlegen.
- **Die Diskette im Emulator ist leer:** Das gespeicherte Abbild enthielt keine Dateien. Prüfe das Verzeichnis im Disk-Editor.
- **Eine Datei lässt sich nicht übernehmen:** Der Zielordner muss im Projekt liegen (keine absoluten Pfade, kein `..`).
- **„Dateien ablegen …“ ist grau:** Die Diskette hat kein Dateisystem (Boot-Diskette).

## Wie geht es weiter?

Mit den Systemunterschieden beschäftigt sich [Zielsysteme im Vergleich](kapitel:systeme). Nachschlagen: [Disk-Editor](hilfe:disks), [Segmente, Bänke, Module](hilfe:cartridges), [ROM-Dateien](hilfe:roms), [Memory Map](hilfe:memmap). Zur Fehlersuche im Programm selbst gehört der Debugger, siehe [Der Debugger](kapitel:debugger).
