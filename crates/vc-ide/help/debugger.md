# Debugger

**F6** baut und startet das Startprogramm im Emulator, angehalten am Einstiegspunkt, und öffnet die Debug-Sitzung (**Alt+Umschalt+F6** debuggt die aktive Datei). Die Kopfzeile zeigt dann **▶ Weiter**, **⏸ Anhalten**, **Schritt über**, **Hinein**, **Hinaus** und **■ Beenden** statt Starten und Debuggen. Die Schublade bekommt den Reiter **Debug**, das rechte Panel den Reiter **Bild**. Welcher Emulator oder welche Maschine startet, bestimmt das Startziel aus den [Einstellungen](help:settings) und dem Zielfeld der Kopfzeile. Alle Debug-Befehle stehen im Menü **Projekt**.

## Was der Adapter kann
Nicht jeder Adapter liefert alles. Die Debug-Zeile zeigt den Zustand („läuft“, „Angehalten bei $0810 · datei:zeile“); kann ein Adapter einen Schritt gerade nicht, steht der Grund daneben.

| Adapter | Register | Schritt | Haltepunkte | Datenhaltepunkte |
|---|---|---|---|---|
| VICE (C64, C128, Plus/4, VC-20) | ja | ja | mehrere | ja (Hardware-Checkpoints) |
| Fujisan (Atari 8 Bit, TCP) | ja (auch setzen, per Vorspann) | ja (auch „Über“) | mehrere | nein |
| atari800 (Textmonitor) | ja | ja | nur einer zugleich | nein |
| MEGA65 (UART-Monitor, Xemu oder Hardware) | ja | ja | einer | als Schreib-Beobachtung per Einzelschritt (langsam) |
| Ultimate 64 / II+ (REST) | nein | nein | nein | nein |

**Atari:** Für `.xex`-Programme nimmt der Debugger **Fujisan**, wenn es gefunden wird (Einstellung `emulators.fujisan.path`, sonst automatische Suche, auch die mit `vcc emulator install fujisan` geladene Version), sonst atari800. Mit `emulators.fujisan.debug_for_atari` = `atari800` oder `fujisan` lässt sich eine Wahl erzwingen. Fujisan läuft als eigenes Programm, wird über seinen TCP-Server (Port 6502) gesteuert und lässt sich per Knopf anhalten; Register setzt der Adapter selbst (Vorspann im RAM), Datenhaltepunkte und Zyklenzählung gibt es nicht, Speicher schreibt Fujisan Byte für Byte (langsam, etwa 50 Byte je Sekunde). Diskettenabbilder und Module debuggt weiter atari800 (**F8** im Emulatorfenster zum Anhalten). Mehr: [Kommandozeile](help:cli) (`vcc emulator`).

atari800 lässt sich nicht per Knopf anhalten (**F8** im Emulatorfenster). Bei der Ultimate sind Speicher lesen und schreiben, Pause und Weiter möglich.

## Bedienung
| Knopf (Menü Projekt) | Taste | Wirkung |
|---|---|---|
| ▶ Weiter (Debuggen: Weiter) | Alt+F5 | läuft bis zum nächsten Haltepunkt |
| ⏸ Anhalten (Debuggen: Anhalten) | Alt+F6 | hält an (VICE, Ultimate; atari800: **F8** im Emulatorfenster) |
| Schritt über (Debuggen: Schritt über) | F10 | ein Befehl, `JSR` wird übersprungen |
| Hinein (Debuggen: Schritt hinein) | F11 | ein Befehl, `JSR` springt hinein |
| Hinaus (Debuggen: Schritt hinaus) | Umschalt+F11 | läuft bis zum Rücksprung (`RTS`/`RTI`) des aktuellen Unterprogramms; Aufrufe darin werden übersprungen, sonst Einzelschritte. Nach 10 s ohne Rücksprung bricht es mit Meldung ab (dann Haltepunkt am Rücksprung setzen) |
| ■ Beenden (Debuggen beenden) | Umschalt+F5 | beendet die Sitzung |
| Debuggen: Bis hierher ausführen | – | läuft bis zur Zeile am Cursor (einmaliger Haltepunkt); im Menü Projekt und im Rechtsklickmenü des Editors |

## Die Debug-Fläche
Der Reiter **Debug** der Schublade zeigt in Spalten nebeneinander **Register**, **Disassembler**, **Speicher** und rechts **Beobachten**, **Haltepunkte** und **Momentaufnahme**. In schmalen Fenstern (unter 900 Punkten) ist es eine Spalte mit Reitern. Oben steht der Zustand der Sitzung.

- **Register** zeigt PC, A, X, Y, SP als Tabelle (Hex und dezimal; geänderte Werte sind hervorgehoben mit „war …“) und die Flags N V B D I Z C als Kästchen (nur Anzeige). Darunter stehen weitere Werte des Emulators, bei VICE und atari800 etwa die Rasterposition.
- **Disassembler** ab dem PC mit Adresse, Bytes, Befehl und Zyklen; ein Klick auf den Rand vor einer Zeile setzt oder löscht dort einen Haltepunkt (über die Zeilenzuordnung des letzten Baus).
- **Speicher** (hex und ASCII, 8 Byte je Zeile, geänderte Bytes hervorgehoben): Adresse oder Name eingeben, Enter.
- **Beobachten:** Ausdrücke (Register, Symbole, `[adresse]`) werden bei jedem Halt neu ausgewertet; ✕ entfernt einen.
- **Haltepunkte:** siehe [Haltepunkte](help:breakpoints).

Noch nicht in der Oberfläche: Rechtsklickmenüs in diesen Spalten, Blättern im Speicher per Knopf, das Speichern eines Speicherbereichs als Zeichensatz und eine Ansicht der Chip-Register. Läuft das Programm, steht in den Spalten „Das Programm läuft. „Anhalten“ zeigt die Werte.“

## Momentaufnahme (Screen, Zeichensatz, Sprites)

In der Spalte **Momentaufnahme** liest die Funktion aus dem angehaltenen Rechner (C64, Atari 8-Bit) den aktuellen Bildschirm, den Zeichensatz und die aktiven Sprites und legt sie unter `graphics/snapshot/` an.

- **Bildschirm erkennen** wertet die Hardware aus: C64 – VIC-Bank (`$DD00`), `$D018`, Farb-RAM, Sprite-Zeiger; Atari – Display-Liste (`$0230`), `CHBAS` (`$02F4`) und Farb-Schattenregister. Das Ergebnis steht darunter: Modus und Bildschirmadresse, Zeichensatz, Zahl der Sprites.
- Mit den Häkchen **Sprites speichern** und **Zeichen als Kacheln** wählst du, was zusätzlich entsteht. Liegt die Standardschrift im ROM, kommt **Zeichensatz aus dem ROM lesen** dazu.
- Dann gibst du einen **Name** ein und klickst **Als Dateien speichern**. Es entstehen `<name>_screen.vcscreen`, `<name>_charset.vcchar`, `<name>_sprite0.vcsprite` … und `<name>_tiles.vctiles`.
- Der C64-Speicher wird als RAM gelesen, auch unter ROM und E/A (VICE). Hinweis: Die Standardschriften gehören den Herstellern – gespeicherte ROM-Zeichensätze nicht in öffentliche Repositories einchecken.
- Nicht unterstützt: Bitmap-Modi (kein Textbildschirm) und ANTIC-Modi außer 2.

Noch nicht in der Oberfläche: eine Vorschau und das Ändern der erkannten Adressen.

## Emulator-Bild (VICE)
Der Reiter **Bild** im rechten Panel (nur in einer Debug-Sitzung, Symbol ◉) zeigt das Bild des VICE-Emulators in VintageCoder: C64, C128 (VIC-II und VDC), VC-20 und Plus/4. Atari und MEGA65 liefern kein Bild.
- **Bild holen** holt das Bild sofort über den Monitor des Emulators. Mit dem Häkchen **bei jedem Halt** holt VintageCoder es bei jedem Halt (Haltepunkt, Einzelschritt, Pause) selbst; es ist genau der Zustand, an dem die Maschine steht.
- **Zeitlupe live:** Mit dem Häkchen hält VintageCoder die Maschine alle paar Millisekunden an, holt das Bild und setzt sie fort. Die Maschine läuft dabei mit etwa 40–80 % ihres Tempos und ohne Ton; zum Spielen ist das nicht gedacht.
- **C128:** Knöpfe über dem Bild schalten zwischen VIC-II (40 Spalten) und VDC (80 Spalten) um.
- **Text senden:** Das Feld unter dem Bild schreibt Text in den Tastaturpuffer des Emulators (Enter sendet), zum Beispiel `RUN`. Das ist nur Text: Tastatur und Joystick werden nicht an den Emulator weitergegeben.
- Das Bild hat die volle Größe des VICE-Bildpuffers (C64 504 × 312, C128 VIC-II 384 × 272, VDC 856 × 288, Plus/4 384 × 288, VC-20 568 × 284); nicht gezeichnete Ränder sind schwarz.

Noch nicht in der Oberfläche: Bild als PNG speichern, in die Zwischenablage kopieren und als Projektbild setzen.
