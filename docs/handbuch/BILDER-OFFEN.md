# Handbuchbilder: Stand

Stand 2026-11-17, Branch `claude/neue-gui`. **Alle Bilder, auf die Handbuch und Hilfe verweisen, stammen aus der neuen Oberfläche** (1600×1007, dunkel, ohne Marken; Beispielprojekt „Hugo & Gumbo“ oder ein anderes Beispiel der Auslieferung, Projektordner unter `/tmp/vc-handbuch`, keine Rechnerpfade im Bild). Die Bildunterschriften enthalten keine Zahlenmarken.

## So entstehen die Bilder

1. `cargo test -p vc-gui handbuch_bilder -- --ignored` schreibt die Rohbilder nach `build/handbuch/` (nur Auswahl: `VC_ZUSTAENDE=editor-suchen,asm-fehler`). Die Szenen stehen in `crates/vc-gui/src/handbuchbilder.rs`.
2. `python3 tools/handbuchbilder.py` (macOS, nutzt `sips`) legt jedes Bild unter `docs/handbuch/bilder/<kennung>.png` ab und schneidet dabei Ausschnitte zu (Spalten des Assistenten, Debug-Spalten, Zeile mit Haltepunkt, Atari-Palette, ROM-Rollen).
3. Eine neue oder geänderte Kennung braucht einen Eintrag in `docs/handbuch/bilder/manifest.json` (Alt-Text, Unterschrift, Aufnahmehinweis). Die ältere Aufnahme mit Marken (`werkzeug/shot.sh`, `annotate.swift`) gilt nur noch für Bilder, die nicht mehr verwendet werden.

Die ersten elf Bilder (Sprite, Zeichensatz-Überblick, Bildschirm, Kachel-Editor, -Bank, -Karte, Tracker-Muster und -Instrument, Grafik-Export, Editor-Syntax, Projektbaum) und die Website-Bilder entstehen mit `cargo test -p vc-gui website_bilder -- --ignored` (`build/web/`); die Website-Nachträge (Disk-Studio, Paletten-Studio, Debugger, Menüleiste) mit `website_bilder_nachtrag`.

## Kennzeichnung „Beispielansicht“

Debug-Bilder (`debug-*`, `memory-map`, `segmente-baenke`, `emulator-reiter`) zeigen eine Attrappe über dem Quelltext von „Hugo & Gumbo“: **Beispielansicht, es läuft kein Emulator.** Die Bildunterschrift sagt das. Ebenso sind die Ergebnisse in `tests-panel` und `tests-ergebnis` Beispieldaten (zwei Prüfungen), und der Klavier-Zustand in `tracker-hoerprobe` ist gesetzt, nicht gespielt.

## Bewusst anders als früher beschrieben (Text angepasst)

| Kennung | Stand |
|---|---|
| `asm-erstes-programm` | zeigt den Quelltext eines neuen Projekts; ein Emulator-Bildschirm ist nicht möglich |
| `asm-makro` | Quelltext mit dem Makro; ein Listing daneben gibt es in der Oberfläche nicht |
| `basic-marken`, `basic-langenamen` | Quelltext der Beispiele; die Zuordnung (Zeilen, Kurznamen) steht in Dateien unter `build/` und im Bauprotokoll, nicht in einem eigenen Fenster |
| `emulator-reiter` | der Reiter „Bild“ **ohne** Bild (ein Emulatorbild lässt sich ohne laufenden Emulator nicht ehrlich zeigen) |
| `disk-neu` | der Dialog „Neue Datei“ mit Art „Diskette D64“ (es gibt keinen eigenen Dialog mit Format, Name und ID) |
| `einstellungen-roms`, `rom-verzeichnis` | Ausschnitt der Rollenliste (alle „Fehlt“); ROM-Ordner und Suchorte stehen nicht im Bild, weil sie Rechnerpfade zeigen |
| `palette-atari` | Ausschnitt der Karte „Dokument“ im Atari-PMG-Editor (die Atari-Palette hat kein eigenes Studio) |
| `wizard-system`, `wizard-sprache`, `wizard-details` | drei Ausschnitte desselben Dialogs „Neues Projekt erstellen“ |
| `debug-ueberblick` | das rechte Panel ist zugeklappt; der Reiter „Bild“ steht in `emulator-reiter` |
| `editor-falten` | die gefalteten Zeilen sind einfach versteckt; eine Marke mit der Zeilenzahl gibt es nicht (Text angepasst) |

## Aus Handbuch und Hilfe entfernt (kein ehrliches Bild möglich)

- `zeichensatz-mega65-farbe`: kein Beispiel mit Vollfarbzeichen (`mega65.char.fcm`) in der Auslieferung.
- `tracker-import-rmt`: der Import-Dialog für RMT-Module hat keinen Zustand ohne eine solche Datei.
- `kuerzel-karte`: die Übersichtskarte war von Hand gesetzt (`werkzeug/kuerzel-karte.swift`) und nennt Kürzel, die es nicht mehr gibt (zum Beispiel Strg+Leertaste, „Inspektor“). Die Tabellen in Kapitel 20 und die Kürzelliste der Hilfe stimmen mit der Befehlstabelle überein. Eine neue Karte müsste aus `BEFEHLE` erzeugt werden.

## Neu dazugekommen

- `menueleiste` (Kapitel 03): die Menüleiste im Fenster, Kürzel in der Schreibweise von Windows und Linux (`kuerzel_windows` gibt es nur für Bilder).

## Offen

Nichts, was Handbuch oder Hilfe brauchen. Bilddateien und Manifest-Einträge der **nicht mehr verwendeten** alten Aufnahmen (alte egui-Oberfläche, mit Marken) liegen noch im Ordner und können gelöscht werden, wenn der Inhaber das möchte: `ui-gesamt`, `ui-zonen`, `ui-reiter`, `ui-modus-code`, `ui-modus-debug`, `theme-hell-dunkel`, `einstellungen`, `befehlspalette`, `projekt-uebersicht`, `bauprotokoll`, `editor-zeichenwaehler`, `editor-referenz`, `debug-bedingung`, `debug-datenhaltepunkt`, `debug-momentaufnahme`, `live-bild`, `sprite-c64`, `tracker-ueberblick`, `basic-nach-asm`, `emulator-bild-projektbild`, `roms-einrichten`, `start-splash`, `assistent-*` (acht Bilder), `include-eingefuegt`, `include-abgelehnt`.
