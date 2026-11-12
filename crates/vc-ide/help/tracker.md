# Tracker

Songs für **SID** (C64, 3 Stimmen) oder **POKEY** (Atari, 4 Stimmen). Eine Zelle: `C-4 01` = Note + Instrument (Hex), `---` nichts, `===` Note aus.

## Eingabe
| Taste | Wirkung |
|---|---|
| Z S X D C V G B H N J M | Noten der unteren Reihe (aktuelle Oktave) |
| Q 2 W 3 E R 5 T 6 Y 7 U I | Noten der oberen Reihe (Oktave + 1) |
| `.` / Entf | Zelle leeren |
| `^` oder `1` | Note aus |
| `[` `]` | Instrument wählen |
| Alt+↑ ↓ | Oktave |
| Pfeile, Bild, Tab | bewegen |
| Leertaste | Wiedergabe (WebAudio) |

## Aufbau
- **Reihenfolge** (links): Positionen = Patterns hintereinander; ↺ markiert, wo die Wiederholung beginnt.
- **Patterns:** neu, duplizieren; Zeilenzahl und Takt (Bilder je Zeile) oben.
- **Instrumente** (rechts): neu, duplizieren, löschen (Patterns werden nachgeführt); bearbeiten im Panel [Hüllkurve](help:envelope).
- **Konvertieren von …** holt einen Song aus einem anderen System ins Projekt: Du wählst eine Datei, auch aus einem anderen Projektordner. Sie wird für den Chip dieses Trackers umgewandelt (SID ↔ POKEY) und als neues Asset angelegt. Möglich sind ein Song-Asset (`*.asset.json`) eine **GoatTracker-Datei** (`*.sng`, GoatTracker 2) oder eine **SID-Datei** (`*.sid`).
  - *GoatTracker:* Übernommen werden Songaufbau (Orderlisten mit Wiederholung und Transponierung), Noten, Note-aus, Instrumentnummern, ADSR, erste Wellenform und Pulsbreite sowie das Tempo (Befehl `F`). Die drei Stimmen werden auf eine gemeinsame Zeitachse gelegt und in gleich lange Patterns geschnitten. Nicht übernommen werden Wellen-, Puls- und Filtertabellen, Vibrato und Patternbefehle (Portamento, Filter, …); dazu erscheinen Hinweise in der Ausgabe. Nur der erste Subtune wird geladen. Ein GoatTracker-Song im POKEY-Tracker wird über SID nach POKEY umgewandelt.
  - *SID-Datei (`.sid`, PSID/RSID):* Eine SID-Datei enthält ein Abspielprogramm, keine Patterns. Deshalb läuft das Programm in einem eingebauten 6502-Kern, und pro Bild wird der Stand der SID-Register aufgezeichnet. Daraus entstehen Noten (aus Gate und Tonhöhe) und Instrumente (aus Wellenform, ADSR und Pulsbreite); der Zeilentakt wird aus den Noteneinsätzen bestimmt. Im Dialog wählst du den **Subtune** und die **Länge in Sekunden** (Standard 30; die Aufnahme endet dort, auch wenn das Stück weiterläuft). Das Ergebnis ist eine Annäherung an den Klang: Vibrato, Portamento, Filter, Ringmodulation und Synchronisation entfallen, Arpeggios erscheinen als Einzelnoten, der Wiederholungspunkt wird nicht erkannt. Nur der erste SID-Chip wird aufgenommen; RSID-Dateien, die einen echten C64 erwarten, können scheitern.
  - Auf der Kommandozeile: `vcc media import-song <datei> --name NEU [--profile ZIELPROFIL] [--song N] [--seconds 30]` (`--song` und `--seconds` gelten für `.sid`).
  **⇩ RMT importieren …** lädt ein Raster-Music-Tracker-Modul.

## Ausgabe
- SID: `vcc media export` → Tabellen; Spielroutine `examples/c64/music_demo/lib/music.inc`.
- POKEY: Export als **RMT-Modul**; Wiedergabe mit dem Original-RMT-Player (`examples/atari/music_demo/lib/rmtplayr.asm`, Makros `rmt.inc`). C-3 ist die tiefste Note.

## Tempo
„Takt“ ist die Zahl der **Bilder je Zeile**. Die Hörprobe rechnet mit 50 Bildern pro Sekunde (PAL); für NTSC oder ein Programm mit 60-Hz-Interrupt stellst du oben „Bildrate“ auf 60 Hz. Wichtig im eigenen Programm: Die Spielroutine einmal je **Bild** aufrufen (Raster-Interrupt). Der KERNAL-Interrupt des C64 läuft auch am PAL-Gerät mit 60 Hz und spielt zu schnell; das Beispiel `examples/c64/music_demo/music_demo.asm` benutzt deshalb einen Raster-IRQ.

## Anzeigen: Pegel, Oszilloskop, Spektrum
Beim Abspielen zeigt der Tracker, was der eingebaute Synthesizer erzeugt (nicht den echten Chip):
- **Kanalköpfe** über den Spalten: je Stimme ein Oszilloskop und zwei LED-Pegelbalken (Grün–Gelb–Rot, Spitzenmarke hält kurz), dazu **M** (stumm) und **S** (Solo). Stumm und Solo wirken auch auf den Klang.
- **Master-Pegel** (links/rechts) in der Werkzeugleiste.
- **Spektrum** (24 Bänder von 60 Hz bis 16 kHz) und **Summen-Oszilloskop** in der Seitenleiste links.
- **Aufleuchten:** In der laufenden Zeile leuchten die Zellen, in denen ein Ton einsetzt, kurz orange auf; die laufende Zeile ist blau hinterlegt.
- **Tastatur** unten: Die Tasten der klingenden Töne leuchten in der Farbe ihrer Stimme; ein Klick spielt den Ton mit dem gewählten Instrument.
Unter **Anzeige** in der Seitenleiste lässt sich jedes Teil abschalten; der Regler *Abfall* stellt ein, wie schnell die Balken fallen. Die Einstellungen gelten für die Sitzung (sie werden nicht gespeichert). Der PLAY-Tracker (MEGA65) hat dieselben Anzeigen.
