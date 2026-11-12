# Kommandozeile: vcc

Alles, was die Oberfläche tut, geht auch auf der Kommandozeile: `vcc` übersetzt, baut Projekte, legt Disketten an und führt Tests aus. Das ist nützlich für Skripte, automatische Prüfungen und Rechner ohne Oberfläche. Die Oberfläche selbst heißt `vintagecoder`, der Sprachserver für andere Editoren `vc-lsp`; beide kennen `--help` und `--version`.

## Hilfe im Terminal

```
vcc --help                  Übersicht aller Befehle
vcc help <befehl>           Hilfe eines Befehls (auch: vcc <befehl> --help)
vcc help <befehl> <form>    nur eine Form, z. B. vcc help basic ast
vcc help --all              alle Befehle mit allen Optionen
vcc help env                Umgebungsvariablen
vcc help exit-codes         Rückgabewerte
```

Die Hilfe nennt je Befehl den Aufruf, alle Argumente und Optionen mit Standardwerten und erlaubten Werten, Beispiele und Rückgabewerte. Bei einem Tippfehler (`vcc buidl`) schlägt `vcc` den ähnlichsten Befehl vor, statt die ganze Übersicht zu zeigen. Ohne Argumente zeigt `vcc` die kurze Übersicht und endet mit Code 2.

## Befehle

### Programme bauen

| Befehl | Zweck |
|---|---|
| `vcc asm` | Assembler-Quelltext übersetzen |
| `vcc basic` | BASIC tokenisieren, zurücklesen, als Syntaxbaum ausgeben |
| `vcc basic2asm` | BASIC-Programm in Assembler-Quelltext umwandeln |
| `vcc build` | Projekt laut .vcprj bauen |

### Projekte

| Befehl | Zweck |
|---|---|
| `vcc add` | Dateien und Ordner ins Projekt übernehmen |
| `vcc migrate` | Projekt vom alten Dateischema auf das neue umstellen |
| `vcc new` | Neues Projekt aus einer Vorlage anlegen |
| `vcc organize` | Projektdateien nach der Projektstruktur umsortieren |

### Medien und Daten

| Befehl | Zweck |
|---|---|
| `vcc dis` | Maschinenprogramm disassemblieren |
| `vcc disk` | Diskettenimages anlegen, auflisten und Dateien herausholen |
| `vcc import` | Programme und Dateien von Disketten oder einzeln ins Projekt holen |
| `vcc inc` | Include-Datei mit Hardwarekonstanten ausgeben |
| `vcc media` (auch `vcc asset`) | Sprites, Zeichensätze, Songs und andere Mediendateien bearbeiten |
| `vcc roms` | ROM-Dateien anzeigen, prüfen und importieren |

### Testen und Messen

| Befehl | Zweck |
|---|---|
| `vcc emulator` | Emulatoren anzeigen, finden und portabel installieren |
| `vcc streams` | Bild-, Ton- und Debug-Ströme der Ultimate 64 empfangen |
| `vcc test` | Tests aus *.vctest prüfen und im Emulator oder auf Hardware ausführen |

### Hilfe und Rückmeldung

| Befehl | Zweck |
|---|---|
| `vcc debuglog` | Debug-Protokoll der IDE auswerten |
| `vcc feedback` | Rückmeldung per E-Mail mit Angaben zur Installation vorbereiten |

Mehrere Befehle haben Unterformen, etwa `vcc disk d64`, `vcc roms check` oder `vcc basic ast`; `vcc help <befehl>` listet sie.

## Beispiele

```
vcc new spiel --target c64
vcc build examples/c64/cart_demo
vcc asm examples/c64/border/border.asm -o border.prg
vcc test examples/c64/tests_demo
vcc roms check
```

Die genauen Optionen jedes Befehls zeigt `vcc help <befehl>`, dort stehen auch weitere Beispiele.

## Umgebungsvariablen

| Variable | Bedeutung |
|---|---|
| `VC_DATA` | Ordner der Hardware-Datenbank (data); sonst neben dem Programm oder im Arbeitsordner. |
| `VC_ROMS` | ROM-Ordner des Benutzers (statt roms/ im Konfigurationsordner). |
| `VC_XEMU` | Pfad zum Xemu-Programm (MEGA65-Emulator). |
| `VC_XEMU_SD` | Ordner mit MEGA65.ROM für Xemu. |
| `VC_XEMU_ARGS` | zusätzliche Argumente beim Start von Xemu, durch Leerzeichen getrennt. |
| `VC_STRUCTURES` | Datei mit eigenen Projektvorlagen (sonst structures.json im Konfigurationsordner). |
| `VC_DEBUG_LOG` | Debug-Protokoll einschalten: 1 (Standarddatei logs/vintagecoder.log im Konfigurationsordner) oder ein Dateipfad; 0 schaltet aus. Gilt für vintagecoder und vcc. |
| `VC_WIN` | Fenstergröße der IDE, z. B. 1400x860 (nur vintagecoder). |
| `COLUMNS` | Breite für den Zeilenumbruch der Hilfe (Standard: 100). |
| `NO_COLOR` | keine Farben in der Ausgabe der Hilfe. |

## Rückgabewerte

| Wert | Bedeutung |
|---|---|
| `0` | fehlerfrei |
| `1` | Fehler (im Quelltext, in einer Prüfung oder bei der Ausführung) |
| `2` | falscher Aufruf (unbekannter Befehl oder unbekannte Option) |

## Handbuchseiten, Vervollständigung, Handbuch

- **man-Seiten (Linux):** `vcc man <befehl>` schreibt die Seite im man-Format nach stdout; fertige Seiten liegen unter `installer/linux/man`, installiert werden sie mit `installer/linux/install-man.sh`. Danach geht `man vcc` und `man vcc-build`.
- **Vervollständigung:** `vcc completion powershell|bash|zsh|fish` gibt das Skript für die jeweilige Shell aus; es vervollständigt Befehle, Formen und Optionen, bei festen Wertelisten auch die Werte.
- **Handbuch:** `vcc help --markdown` erzeugt das ganze Handbuch als Markdown (im Quellbaum `docs/cli.md`).

Alle diese Ausgaben entstehen aus denselben Beschreibungen unter `data/cli`; Tests prüfen, dass Beschreibung und Optionen des Programms übereinstimmen.

## Siehe auch

[Bauen und Starten](help:build) · [Tests mit vcc test](help:tests) · [ROM-Dateien](help:roms) · [Disk-Studio](help:disks) · [Linux](help:linux)
