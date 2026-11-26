# Tracker

Songs für **SID** (C64, 3 Stimmen) oder **POKEY** (Atari, 4 Stimmen). Eine Zelle: `C-4 01` = Note + Instrument (Hex), `---` nichts, `===` Note aus.

## Eingabe
Die Tasten gelten, solange das Raster den Fokus hat.

| Taste | Wirkung |
|---|---|
| Z S X D C V G B H N J M | Noten der unteren Reihe (aktuelle Oktave); auf QWERTZ: Y S X D C V G B H N J M |
| Q 2 W 3 E R 5 T 6 Y 7 U I | Noten der oberen Reihe (Oktave + 1); auf QWERTZ: Q 2 W 3 E R 5 T 6 Z 7 U I |
| Entf / Rücktaste | Zelle leeren |
| `1` | Note aus |
| `[` `]` | Instrument wählen |
| `,` `.` | Oktave tiefer / höher |
| Alt+↑ ↓ | Instrument der Zelle (oder der Auswahl) wechseln |
| Pfeile, Bild auf/ab, Pos1/Ende | bewegen (mit Umschalt: auswählen) |
| Leertaste | Song abspielen / stoppen (mit Umschalt: nur das Muster) |
| Cmd/Strg+C, X, V, A | Block kopieren, ausschneiden, einfügen, alles wählen |
| Cmd/Strg+1, 2, 3 | Fokus auf Raster, Instrumente, Reihenfolge |

## Aufbau
Der Editor hat drei Bereiche nebeneinander (in schmalen Fenstern rutschen sie untereinander).

![Der Reiter Musik mit der Datei tune.vcsong: links Reihenfolge (Muster 1, 2, 1, 2), Instrumente Puls, Bass und Snare, in der Mitte die Werkzeugleiste mit Mitscrollen, Tempo, Zeilen, Oktave und Schritt und das Raster mit drei Stimmen, rechts die Karten „Instrument 00 · Puls“, „Klaviatur“ und „Lautstärke“. Der Cursor steht auf Zeile 02, unten steht „ungespeichert“.](bild:tracker-ueberblick "Der Tracker mit Reihenfolge, Raster mit drei Stimmen und Instrumenten.")

- **Links** der Song: Name mit **Neues Muster** (Plus), die **Reihenfolge** der Muster (**Anhängen**, **Setzen**, **Entfernen**, Pfeile zum Verschieben, **Wiederholung ab Eintrag**; ↻ markiert die Wiederholung), **Duplizieren** und **Löschen** für das gewählte Muster und die Liste der **Instrumente** (**Neues Instrument**, **Duplizieren**, **Löschen**: nur das letzte, unbenutzte). Bei mehreren Songs im Projekt folgen die **Songs**.
- **In der Mitte** die Werkzeugleiste mit Rückgängig, Wiederholen, **Song abspielen**, **Muster ▶**, **Wiedergabe stoppen**, **Mitscrollen** und den Feldern **Tempo** (Bilder je Zeile), **Zeilen**, **Oktave** und **Schritt**, dazu **Halbton ↑**, **Halbton ↓**, **Instrument anwenden**, **Übernehmen …**, **RMT-Modul importieren …** (nur POKEY), **Exportieren …** und **Speichern**. Darunter die Auswahl der **Stimmen**, die Muster als Knöpfe und das Raster. Jede Stimme hat im Kopf **M** (stumm), **S** (solo) und einen Pegelbalken.
- **Rechts** das Instrument mit seinen Reglern, siehe [Hüllkurve](help:envelope), darunter die **Klaviatur** und die **Lautstärke**.

Neue Songs legt der Dialog **Neue Datei** in dieser Version noch nicht an.

**Übernehmen …** holt einen Song aus einem anderen System ins Projekt: Du wählst eine Datei, auch aus einem anderen Projektordner. Sie wird für den Chip dieses Trackers umgewandelt (SID ↔ POKEY) und als neue Datei angelegt. Möglich sind ein Song, eine **GoatTracker-Datei** (`*.sng`, GoatTracker 2) oder eine **SID-Datei** (`*.sid`).
  - *GoatTracker:* Übernommen werden Songaufbau (Orderlisten mit Wiederholung und Transponierung), Noten, Note-aus, Instrumentnummern, ADSR, erste Wellenform und Pulsbreite sowie das Tempo (Befehl `F`). Die drei Stimmen werden auf eine gemeinsame Zeitachse gelegt und in gleich lange Patterns geschnitten. Nicht übernommen werden Wellen-, Puls- und Filtertabellen, Vibrato und Patternbefehle (Portamento, Filter, …); dazu erscheinen Hinweise in der Ausgabe. Nur der erste Subtune wird geladen. Ein GoatTracker-Song im POKEY-Tracker wird über SID nach POKEY umgewandelt.
  - *SID-Datei (`.sid`, PSID/RSID):* Eine SID-Datei enthält ein Abspielprogramm, keine Patterns. Deshalb läuft das Programm in einem eingebauten 6502-Kern, und pro Bild wird der Stand der SID-Register aufgezeichnet. Daraus entstehen Noten (aus Gate und Tonhöhe) und Instrumente (aus Wellenform, ADSR und Pulsbreite); der Zeilentakt wird aus den Noteneinsätzen bestimmt. Im Dialog wählst du den **Subtune** und die **Aufnahme in Sekunden** (Standard 30; die Aufnahme endet dort, auch wenn das Stück weiterläuft). Das Ergebnis ist eine Annäherung an den Klang: Vibrato, Portamento, Filter, Ringmodulation und Synchronisation entfallen, Arpeggios erscheinen als Einzelnoten, der Wiederholungspunkt wird nicht erkannt. Nur der erste SID-Chip wird aufgenommen; RSID-Dateien, die einen echten C64 erwarten, können scheitern.
  - Auf der Kommandozeile: `vcc media import-song <datei> --name NEU [--profile ZIELPROFIL] [--song N] [--seconds 30]` (`--song` und `--seconds` gelten für `.sid`).

**RMT-Modul importieren …** (nur POKEY) lädt ein Raster-Music-Tracker-Modul.

## Ausgabe
- SID: `vcc media export` → Tabellen; Spielroutine `examples/c64/music_demo/lib/music.inc`.
- POKEY: Export als **RMT-Modul**; Wiedergabe mit dem Original-RMT-Player (`examples/atari/music_demo/lib/rmtplayr.asm`, Makros `rmt.inc`). C-3 ist die tiefste Note.

## Tempo
Das **Tempo** (auch „Takt“) ist die Zahl der **Bilder je Zeile**. Wichtig im eigenen Programm: Die Spielroutine einmal je **Bild** aufrufen (Raster-Interrupt). Der KERNAL-Interrupt des C64 läuft auch am PAL-Gerät mit 60 Hz und spielt zu schnell; das Beispiel `examples/c64/music_demo/music_demo.asm` benutzt deshalb einen Raster-IRQ.

## Anzeigen: Pegel und Klaviatur
Beim Abspielen zeigt der Tracker, was der eingebaute Synthesizer erzeugt (nicht den echten Chip):
- Im Kopf jeder Stimme zeigt ein Balken den **Pegel**.
- **M** (stumm) und **S** (solo) wirken auch auf den Klang. Die Knöpfe der **Stimmen** über dem Raster blenden eine Stimme im Raster aus (nur die Ansicht: sie spielt weiter, wenn sie nicht stumm ist, und die Datei ändert sich nicht); **Alle einblenden** holt alle zurück. Eine Stimme bleibt immer sichtbar. Bei vielen Stimmen (zum Beispiel 12 beim MEGA65) hat das Raster eine waagerechte Bildlaufleiste; die Kanalköpfe und Zeilennummern bleiben stehen, der Cursor bleibt sichtbar.
- Die laufende Zeile ist im Raster markiert; **Mitscrollen** lässt das Raster mitlaufen.
- Die **Klaviatur** zeigt die klingenden Töne; ein Klick spielt den Ton mit dem gewählten Instrument vor, trägt aber nichts ein.
Änderungen im Raster hörst du sofort, auch während der Song spielt.

Die Tasten gelten nach ihrer **Position** auf der Tastatur: Die Buchstaben sind die der US-QWERTY-Tastatur; auf einer deutschen Tastatur (QWERTZ) heißt die erste Taste der unteren Reihe **Y** (statt Z) und in der oberen Reihe steht **Z** statt Y. Die Hinweise im Tracker nennen die Buchstaben Ihres Layouts (Einstellungen, Tastaturlayout).
