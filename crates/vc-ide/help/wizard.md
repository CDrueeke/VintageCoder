# Willkommen und Projekt-Wizard

## Willkommen
Ohne geöffnetes Projekt zeigt der Tab **Willkommen**:
- **Neues Projekt**, **Projekt öffnen** (Strg/Cmd+O) und **Beispiel öffnen** (der Ordner `examples/` neben der Hardware-Datenbank).
- **Zuletzt geöffnete Projekte** mit Chips für Systeme, Sprachen und Emulator. Ein Klick auf den Namen oder **Öffnen** öffnet das Projekt, ☆ heftet es oben an, × nimmt es aus der Liste. Fehlt der Ordner, steht „Ordner fehlt“.
- **Toolchain und Hardware** (rechts): welche Emulator-Sorten gefunden wurden und welche Maschinen eingetragen sind. Beim Start findet kein Netzwerkzugriff statt, deshalb steht Hardware auf „ungeprüft“, bis du **Testen** klickst. „Einstellungen öffnen“ springt zu [Einstellungen](help:settings).
- **Studios:** Schaltflächen für Sprites, Zeichensatz, Bildschirm, Kacheln, Tracker und Palette. Sie legen eine neue Datei im geöffneten Projekt an und sind ohne Projekt gesperrt.
- **Schnellreferenz:** Links in dieses Handbuch.

## Neues Projekt
**Datei > Neues Projekt …**, die Schaltfläche auf dem Willkommen-Tab oder `vcc new` legt ein Projekt an: Projektdatei `.vcprj`, Quelldateien aus einer Vorlage und den Ordner `src/<system>/`. Der Dialog hat drei Spalten:

1. **Zielsystem:** C64, C128, C16/Plus/4, VC-20, Atari 800 XL/XE oder MEGA65, dazu das Hardware-Profil (CPU und Chips).
2. **Vorlage:** zuerst die Sprache, **Assembler (vcasm)** oder **BASIC**, dann die Vorlage. Gibt es für die Kombination keine, steht das da. BASIC wird im Dialekt des Zielsystems geschrieben, ohne Zeilennummern und mit Sprungmarken (`@name`); beim Bauen entsteht das native Programm.
3. **Projekt-Details:** Projektname (Buchstaben, Ziffern, `_` und `-`), Speicherort, BASIC-Dialekt (wo es mehrere gibt), Videonorm PAL oder NTSC (nicht beim MEGA65), beim VC-20 der Speicherausbau, der **Standard-Emulator** und das **Startziel (F5)** (beide aus den [Einstellungen](help:settings), hier nur angezeigt) und die **Diskette**: keine oder ein Format, das zum System passt (C64 D64, C128 D64, D71 oder D81, Atari ATR, MEGA65 D81). Dazu zwei Schalter: README.md mit Bauhinweisen und ein Git-Repository mit `.gitignore` für `build/`. Unter „Weitere Zielsysteme“ lassen sich zusätzliche Systeme der Vorlage anhaken; „Dateistruktur“ zeigt, was angelegt wird.

**Projekt erstellen und öffnen** legt alles an und öffnet das Projekt.

## Projektstruktur
Im Wizard wählst du unter **Projektstruktur**, welche Ordner das Projekt bekommt: *Einfach* (Vorgabe, keine zusätzlichen Ordner), *Standard*, *Groß, modular*, *Mehrere Systeme*, *Bibliothek* oder *Spiel mit Levels*. Die Vorschau zeigt die Ordner. Die Struktur legt die Ordner an und trägt im Abschnitt `layout` der Projektdatei ein, wohin abgelegte Dateien je Kategorie kommen (siehe [Editor](help:editor), „Dateien ins Projekt ziehen“). Bei *Mehrere Systeme* entsteht je gewähltem Zielsystem ein Ordner in `src/` und `res/`. Mit angehaktem Git-Repository bekommen neue leere Ordner eine `.gitkeep`. Die mitgelieferten Vorlagen stehen in `data/templates/structures.json`, eigene (aus den Projekt-Einstellungen ▸ Projektstruktur ▸ *Als Vorlage speichern*) in `structures.json` im Konfigurationsordner. Die Vorschau zeigt die Ordner als Baum.

## Kommandozeile und Vorlagen
- Systeme: `c64`, `c128`, `plus4`, `vic20`, `atari8`, `mega65` (auch mehrere, kommagetrennt).
- Vorlagen: `asm-hello`, `basic-hello`, `empty`. `asm-hello` gibt es zusätzlich als Variante **Steckmodul** (`--variant cartridge`; C64, Atari, VC-20): gleiche Projektdatei, der Startquelltext enthält `.cartridge`, die Adresse des Modulbereichs und die Kennung des Systems. `--template cartridge` bleibt als Kurzform erhalten.
- Ordner: relativ zum geöffneten Ordner; `.` nur, wenn dort noch kein Projekt liegt (Start der IDE in einem leeren Ordner öffnet den Wizard von selbst).
- Grafik und Klang: die Studios bieten nur Profile der Zielsysteme des Projekts an; Sprites, Zeichensätze und Songs anderer Systeme lassen sich nicht öffnen oder anlegen.

        vcc new meinspiel --target c64,mega65 --template asm-hello
        vcc new meinmodul --target c64 --template asm-hello --variant cartridge
        vcc new --list

Eigene Vorlagen: Ordner und Eintrag in `data/templates/index.json` (`{system}` im Pfad, `{{name}}` im Text). Eine Vorlage kann unter `variants` Varianten mit eigenen Systemen, Dateien und Quellen haben; sie ersetzen die Dateien der Vorlage.

Konstanten für den Assembler stehen im Target der Projektdatei (wie `vcc asm -D NAME=Zahl`), zum Beispiel `"defines": { "DEBUG": 1 }` für `.if DEBUG`; `true` und `false` zählen als 1 und 0. Sie gelten beim Bauen und in den Live-Fehlern des Editors.

Mit `--structure <id>` (`vcc new --list` nennt die Strukturen) legt `vcc new` dieselben Ordner und das `layout` an, z. B. `vcc new spiel --target c64 --structure gross`.
