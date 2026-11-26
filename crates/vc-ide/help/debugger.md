# Debugger

**F6** baut und startet das Startprogramm im Emulator, angehalten am Einstiegspunkt, und öffnet die Debug-Sitzung im Hauptbereich **Debug** (**Alt+Umschalt+F6** debuggt die aktive Datei). Ohne Sitzung zeigt der Bereich die Karte „Keine Debug-Sitzung“ mit dem Knopf **Debuggen starten**. Welcher Emulator oder welche Maschine startet, bestimmt das Startziel aus den [Einstellungen](help:settings). Alle Debug-Befehle stehen im Menü **Projekt ▸ Debuggen** und in der Befehlspalette (**Cmd/Strg+K**). In der Statusleiste steht während einer Sitzung „Debug: angehalten bei $PC“; ein Klick bringt dich in den Bereich Debug zurück.

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
In der Werkzeugleiste in der Mitte des Bereichs Debug stehen Symbolknöpfe; die Maus darüber zeigt den Namen.

| Name (Knopf, Menü Projekt ▸ Debuggen) | Taste | Wirkung |
|---|---|---|
| Debuggen: Weiter | Alt+F5 | läuft bis zum nächsten Haltepunkt (ohne Sitzung startet es eine) |
| Debuggen: Anhalten | Alt+F6 | hält an (VICE, Ultimate; atari800: **F8** im Emulatorfenster) |
| Debuggen: Schritt über | F10 | ein Befehl, `JSR` wird übersprungen |
| Debuggen: Schritt hinein | F11 | ein Befehl, `JSR` springt hinein |
| Debuggen: Schritt hinaus | Umschalt+F11 | läuft bis zum Rücksprung (`RTS`/`RTI`) des aktuellen Unterprogramms; Aufrufe darin werden übersprungen, sonst Einzelschritte. Nach 10 s ohne Rücksprung bricht es mit Meldung ab (dann Haltepunkt am Rücksprung setzen) |
| Debuggen beenden | Umschalt+F5 | beendet die Sitzung |
| Debuggen: Bis hierher ausführen | keine | läuft bis zur Zeile am Cursor im Editor (einmaliger Haltepunkt) |

Neben den Knöpfen steht der Zustand („Startet …“, „Läuft“, „Angehalten bei $0810 datei:zeile“, „Wartet auf den Emulator“) und rechts der Name des Emulators. Kann ein Emulator einen Schritt gerade nicht, steht der Grund darunter. Eine Sitzung, die aus dem Panel Tests mit **Diesen Test debuggen** entstand, trägt die Marke „Test-Sitzung“.

## Die Debug-Fläche
Der Bereich Debug hat drei Spalten. In schmalen Fenstern rutschen sie untereinander.

- **Links:** **Haltepunkte** (siehe [Haltepunkte](help:breakpoints)), **Beobachten**, **Momentaufnahme** und **Chips**.
- **Mitte:** Werkzeugleiste, **Disassembler** und **Speicher**.
- **Rechts:** **Register**, **Stapel** und **Bildschirm**.

Einzelheiten:

- **Register** zeigt PC, A, X, Y, SP als Hexwerte (geänderte sind hervorgehoben) und die Flags N V B D I Z C als Kästchen (nur Anzeige). Darunter stehen weitere Werte des Emulators, bei VICE und atari800 etwa die Rasterposition. Der **Stapel** listet die Bytes ab dem Stapelzeiger mit ihrer Adresse.
- **Disassembler** ab dem PC mit Adresse, Bytes, Befehl und Zyklen. Der runde Knopf vor einer Zeile setzt oder löscht dort einen Haltepunkt; in der Zeile des PC führt der Link `datei:zeile` in den Editor.
- **Speicher** (16 Byte je Zeile, hex und als Zeichen, geänderte Bytes hervorgehoben): Adresse oder Symbol eingeben, Enter oder **Anzeigen**.
- **Beobachten:** Ausdrücke (Register, Symbole, `[adresse]`) werden bei jedem Halt neu ausgewertet und hexadezimal und dezimal gezeigt; ✕ entfernt einen. Ausdrücke sind auch vor dem Start der Sitzung möglich.
- **Chips:** Kontrollkästchen je Baustein des Zielsystems; ein angehakter Chip zeigt seine Register mit Adresse, Name und aktuellem Wert (Maus darüber: Beschreibung). Ohne Chipbeschreibung steht „Für dieses Zielsystem gibt es keine Chipbeschreibung.“

Läuft das Programm, steht in den Spalten „Das Programm läuft. Anhalten zeigt Register, Disassembler und Speicher.“ Rechtsklickmenüs und Blättern per Knopf im Speicher gibt es nicht. Das Panel **Maschine** zeigt in einer Sitzung zusätzlich Register, Flags und das Bild in klein.

## Momentaufnahme (Screen, Zeichensatz, Sprites)

Im Abschnitt **Momentaufnahme** (links) liest die Funktion aus dem angehaltenen Rechner (C64, Atari 8-Bit) den aktuellen Bildschirm, den Zeichensatz und die aktiven Sprites und legt sie unter `graphics/snapshot/` an.

- **Erkennen, was im Speicher liegt** wertet die Hardware aus: C64 – VIC-Bank (`$DD00`), `$D018`, Farb-RAM, Sprite-Zeiger; Atari – Display-Liste (`$0230`), `CHBAS` (`$02F4`) und Farb-Schattenregister. Das Ergebnis steht darunter: Modus und Bildschirmadresse, Zeichensatz, Zahl der Sprites.
- Mit den Schaltern **Sprites übernehmen** und **Kacheln aus dem Bildschirm bilden** wählst du, was zusätzlich entsteht. Liegt die Standardschrift im ROM, kommt **Zeichensatz aus dem ROM holen** dazu.
- Dann gibst du einen Namen ein und klickst **Als Dateien sichern**. Die Momentaufnahme geht nur bei angehaltenem Programm. Es entstehen `<name>_screen.vcscreen`, `<name>_charset.vcchar`, `<name>_sprite0.vcsprite` … und `<name>_tiles.vctiles`.
- Der C64-Speicher wird als RAM gelesen, auch unter ROM und E/A (VICE). Hinweis: Die Standardschriften gehören den Herstellern – gespeicherte ROM-Zeichensätze nicht in öffentliche Repositories einchecken.
- Nicht unterstützt: Bitmap-Modi (kein Textbildschirm) und ANTIC-Modi außer 2.

Eine Vorschau und das Ändern der erkannten Adressen gibt es nicht.

## Emulator-Bild (VICE)
Der Abschnitt **Bildschirm** (rechts im Bereich Debug) zeigt das Bild des VICE-Emulators in VintageCoder: C64, C128 (VIC-II und VDC), VC-20 und Plus/4. Atari und MEGA65 liefern kein Bild („Dieser Emulator liefert kein Bild.“). Das Panel **Maschine** zeigt das Bild kleiner.
- **Bild aktualisieren** holt das Bild sofort über den Monitor des Emulators. Mit dem Schalter **Bei jedem Halt aktualisieren** holt VintageCoder es bei jedem Halt (Haltepunkt, Einzelschritt, Pause) selbst; es ist genau der Zustand, an dem die Maschine steht.
- **Live (Zeitlupe):** VintageCoder hält die Maschine alle paar Millisekunden an, holt das Bild und setzt sie fort. Die Maschine läuft dabei mit etwa 40 bis 80 Prozent ihres Tempos und ohne Ton; zum Spielen ist das nicht gedacht.
- **Bildquelle:** Beim C128 schaltet eine Auswahlliste zwischen VIC-II (40 Spalten) und VDC (80 Spalten) um.
- **Text an den Emulator senden:** Das Feld schreibt Text in den Tastaturpuffer des Emulators (Enter oder **Senden**), zum Beispiel `RUN`. Das ist nur Text: Tastatur und Joystick werden nicht an den Emulator weitergegeben.
- Das Bild hat die volle Größe des VICE-Bildpuffers (C64 504 × 312, C128 VIC-II 384 × 272, VDC 856 × 288, Plus/4 384 × 288, VC-20 568 × 284); nicht gezeichnete Ränder sind schwarz.

Bild als PNG speichern, in die Zwischenablage kopieren und als Projektbild setzen gibt es nicht.

## Hex-Ansicht (Daten ▸ Hex)
Die Hex-Ansicht zeigt Binärdateien des Projekts nur zum Lesen. Links wählst du unter **Binärdateien** eine Datei (im Dateibaum geht auch **Als Hex ansehen (nur Lesen)**). In der Mitte stehen Adresse, Bytes und Text; die Auswahl geht mit Klick, Umschalt, Ziehen und den Pfeiltasten. Rechts zeigt **Gedeutet als** das Byte (dezimal, hex, binär), das Wort (kleines Byte vorn) und den Text, bei Programmdateien auch die Ladeadresse und die Speicheradresse der Auswahl. **Suchen** nimmt Bytes (`$A9 00`) oder Text. **16 Byte** und **8 Byte** stellen die Zeilenbreite ein. Gelesen werden höchstens die ersten 4 MiB, angezeigt 4 KB je Seite; die Knöpfe der Seitenwahl blättern. **Im Standardprogramm öffnen** übergibt die Datei an das Betriebssystem.
