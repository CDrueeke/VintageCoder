# Handbuchbilder aus der neuen Oberfläche (Stand 2026-11-24)

Werkzeug: `zsh ui/tools/handbuchbilder.sh <Ausgabeordner> [kennung …]` (Playwright/WebKit über die Prüfbrücke gegen den echten Kern, 1600×1007, Deutsch, dunkel, ohne Marken, eigenes HOME, neutraler Pfad `/tmp/demo/Retro`; Debug-Bilder sind Beispielansichten mit Attrappe, kein Emulator). Ohne Kennungen folgt ein zweiter Lauf mit leerer Konfiguration (Startbild, Einrichtungsassistent). Die Szenen stehen in `ui/tools/handbuchbilder.mjs`; **geschrieben wird nie nach `docs/`**: die Übernahme ins Handbuch (`docs/handbuch/bilder/<kennung>.png`, Manifest, Alt-Texte, Bildplatzhalter-Regel) entscheidet die Dokumentation (Branch `claude/doku-030`).

## Vorhanden (52 von 105 Kennungen des Manifests)

`argumente`, `asm-fehler`, `assistent-design`, `assistent-fertig`, `assistent-projekte`, `assistent-systeme`, `assistent-toolchain`, `assistent-weg-waehlen`, `assistent-zusammenfassung`, `bauprotokoll`, `befehlspalette`, `bildschirm-editor`, `datei-neu-dialog`, `datei-umbenennen-dialog`, `debug-beobachten`, `debug-disassembler`, `debug-register`, `debug-speicher`, `debug-ueberblick`, `disk-studio`, `editor-fehlerzeile`, `editor-gehe-zu`, `editor-hilfe-f1`, `editor-kontextmenue`, `editor-suchen`, `editor-syntax`, `editor-vervollstaendigung`, `einstellungen`, `einstellungen-emulator`, `einstellungen-roms`, `grafik-exportieren`, `kachel-editor`, `kachel-karte`, `memory-map`, `palette-mega65`, `probleme-panel`, `projekt-einstellungen`, `projekt-uebersicht`, `sprite-animation`, `sprite-c64`, `start-splash`, `tests-ergebnis`, `tests-panel`, `tracker-hoerprobe`, `tracker-huellkurve`, `tracker-instrument`, `tracker-muster`, `tracker-ueberblick`, `ui-gesamt`, `willkommen-seite`, `zeichensatz-standard`, `zeichensatz-ueberblick`

## Offen (53)

- **Dialoge, Ziehen, Assistent-Teile** (12): `wizard-system`, `wizard-sprache`, `wizard-details`, `drag-drop-rueckfrage`, `bauen-knopf`, `roms-einrichten`, `projekt-neu-ergebnis`, `assistent-wiederaufruf`, `include-ziehen`, `include-eingefuegt`, `include-abgelehnt`, `menueleiste`
- **Debug im Halt mit Haltepunkt, Bedingung, Datenhaltepunkt, Momentaufnahme, Live-Bild** (8): `debug-haltepunkt-gutter`, `debug-pc-zeile`, `debug-bedingung`, `debug-datenhaltepunkt`, `debug-momentaufnahme`, `emulator-reiter`, `live-bild`, `segmente-baenke`
- **weitere Editor-, Grafik-, Musik-, Disk-, BASIC- und Vergleichsbilder** (33): `editor-falten`, `editor-zeichenwaehler`, `ui-zonen`, `ui-reiter`, `projektbaum`, `ui-modus-code`, `ui-modus-debug`, `theme-hell-dunkel`, `editor-ersetzen`, `editor-referenz`, `zeichensatz-multicolor`, `sprite-c64-multicolor`, `sprite-atari-pmg`, `kachel-bank`, `palette-atari`, `tracker-pokey`, `play-tracker`, `disk-validieren`, `rom-verzeichnis`, `asm-erstes-programm`, `ausgabe-panel`, `basic-marken`, `basic-langenamen`, `basic-nach-asm`, `zeichensatz-png-import`, `system-vergleich-c64`, `system-vergleich-atari`, `system-vergleich-mega65`, `emulator-bild-projektbild`, `disk-neu`, `editor-zweiter-bereich`, `asm-makro`, `tracker-sid-pokey`

Bilder: `~/Development/abnahme-ui/handbuchbilder/bilder/`. Beschriftete Marken (Rahmen, Nummern) gibt es nicht; wo das Handbuch Marken nennt, ist der Text anzupassen. Die Bilder zeigen Rechnerpfade der Testumgebung (`/tmp/demo/Retro`) und, wo vorhanden, die Fundorte installierter Emulatoren (`/Applications/…`).

## Nicht machbar in 0.3.0 – für 0.3.1 (Stand 2026-11-25)

Zehn Kennungen lassen sich nicht aufnehmen, weil die gezeigte Funktion in der neuen Oberfläche (noch) fehlt oder nicht im Prüfstand sichtbar ist. Entweder die Funktion nachbauen und aufnehmen, oder Bild und Text im Handbuch streichen bzw. anpassen:

| Kennung | Grund |
|---|---|
| `editor-zeichenwaehler` | Zeichenwähler über dem Editor gibt es nicht |
| `editor-referenz` | Referenz-Reiter in der Ablage gibt es nicht (Ersatz: Hover-Karte, F1 → Hilfeseite = `editor-hilfe-f1`) |
| `menueleiste` | native Menüleiste, im Browser-Prüfstand nicht sichtbar; nur als Bildschirmfoto der echten App von Hand |
| `live-bild` | Hinweis „keine Ultimate 64 eingerichtet“ gibt es nicht; die Debug-Ansicht sagt nur „Dieser Emulator liefert kein Bild“ |
| `basic-nach-asm` | Der Befehl hat in der neuen Oberfläche keinen Dialog |
| `emulator-bild-projektbild` | Knopf „Als Projektbild setzen“ im Emulator-Reiter fehlt |
| `zeichensatz-png-import` | Dialog „Aus PNG-Raster“ fehlt (nur „PNG laden …“ mit Dateiwahl des Systems) |
| `include-ziehen`, `include-eingefuegt`, `include-abgelehnt` | Ziehen einer Datei aus dem Projektbaum in den Quelltext ist nicht gebaut (der Editor nimmt nur Reiter an); „Als Include einfügen“ gibt es nur im Export-Dialog (= `basic-medium-include`) |
