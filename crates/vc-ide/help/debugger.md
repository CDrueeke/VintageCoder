# Debugger

**F6** baut und startet das Programm im Emulator, angehalten am Einstiegspunkt, und schaltet auf den Arbeitsplatz **Debug** (Segment „Code | Debug“ rechts in der Befehlsleiste). Die Befehlsleiste zeigt dann Pause/Weiter, Schritt, Über, Raus und Stop statt Starten und Debuggen; das Zielfeld bleibt, ist aber gesperrt. Welcher Emulator oder welche Maschine startet, bestimmt das Startziel aus den [Einstellungen](help:settings) und dem Feld „Auf:“.

## Was der Adapter kann
Die Oberfläche blendet aus, was der Adapter nicht liefert. Unter dem Status („läuft“, „angehalten $0810 datei:zeile“) zeigen Chips die Fähigkeiten; fehlende sind gedimmt und nennen im Hinweis den Grund.

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
| Taste/Knopf | Wirkung |
|---|---|
| Weiter | läuft bis zum nächsten Haltepunkt |
| Pause | hält an (VICE, Ultimate; atari800: **F8** im Emulatorfenster) |
| Schritt / Über | ein Befehl / ein Befehl, `JSR` wird übersprungen |
| Raus (Umschalt+F11) | läuft bis zum Rücksprung (`RTS`/`RTI`) des aktuellen Unterprogramms; Aufrufe darin werden übersprungen, sonst Einzelschritte. Nach 10 s ohne Rücksprung bricht es mit Meldung ab (dann Haltepunkt am Rücksprung setzen) |
| Stop | beendet die Sitzung |

## Panels
- **Register** zeigt PC, A, X, Y, SP und die Flags N V B D I Z C als Pillen (nur Anzeige), dazu das Panel Chip-Register (siehe unten).
- **Disassembler** ab dem PC; ein Klick auf eine Zeile setzt oder löscht dort einen Haltepunkt (über die Zeilenzuordnung des letzten Baus).
- **Speicher** (hex, ASCII; „Bereich als Zeichensatz speichern“ liest einen Zeichensatz ab einer Adresse aus dem Speicher des laufenden Rechners und legt ihn unter `graphics/dump/` als Asset an): Adresse eingeben oder mit ±256 blättern; die Rasterposition steht bei VICE und atari800 unter den Registern (`LIN`, `CYC`).
- **Beobachten:** Ausdrücke (Register, Symbole, `[adresse]`) werden bei jedem Halt neu ausgewertet.
- **Haltepunkte:** siehe [Haltepunkte](help:breakpoints).

## Chip-Register
Im Register-Panel zeigt **Chip-Register** die Register eines Chips des Zielsystems (zum Beispiel `vic2`, `sid`, `cia1`, `antic`, `pokey`), mit Adresse, Name und Wert in Hex. Der Chip wird in der Auswahl gewählt; Namen und Beschreibung kommen aus der Hardware-Datenbank (`data/chips`), der Hinweis eines Namens zeigt die Beschreibung. Gelesen wird der Adressraum der CPU, solange das Programm angehalten ist; läuft es, erscheinen die Werte beim nächsten Halt. Schreibregister zeigen „–“, weil ihr Wert nicht lesbar ist. Die Anzeige ist nur lesend: Schreiben ist bewusst nicht möglich, weil Leseseiteneffekte und Schreibregister das Bild verfälschen würden. Adapter ohne Register (Ultimate) zeigen keine Werte.

## Momentaufnahme (Screen, Zeichensatz, Sprites)

Im Reiter **Momentaufnahme** (Fußpanel, Arbeitsplatz Debug) liest die Funktion aus dem angehaltenen Rechner (C64, Atari 8-Bit) den aktuellen Bildschirm, den Zeichensatz und die aktiven Sprites und legt sie unter `graphics/snapshot/` an.

- **Erkennen** wertet die Hardware aus: C64 – VIC-Bank (`$DD00`), `$D018`, Farb-RAM, Sprite-Zeiger; Atari – Display-Liste (`$0230`), `CHBAS` (`$02F4`) und Farb-Schattenregister.
- Die erkannten **Adressen sind editierbar**. Wurde ein Bildschirm oder Zeichensatz an eine andere Stelle kopiert (z. B. Puffer, der später in den Bildschirm kopiert wird), trägst du dessen Adresse ein.
- Der C64-Speicher wird als RAM gelesen, auch unter ROM und E/A (VICE); Auch die Standardschrift im Zeichen-ROM (C64) bzw. Betriebssystem-ROM (Atari) wird direkt aus dem Emulator gelesen; eine Adresse brauchst du nur, wenn das Programm den Zeichensatz im RAM hält. Hinweis: Die Standardschriften gehören den Herstellern – gespeicherte ROM-Zeichensätze nicht in öffentliche Repositories einchecken.
- Nicht unterstützt: Bitmap-Modi (kein Textbildschirm) und ANTIC-Modi außer 2.

Links zeigt eine **Vorschau**, was erkannt wurde (Textbildschirm mit Zeichensatz und Farben, ohne Rahmen und Sprites). Das Zeichen-ROM wird nur für die Vorschau gelesen und nie gespeichert.

## Emulator-Bild (VICE)
Der Reiter **Emulator** (Fußpanel im Debug-Arbeitsplatz, auch Menü Debug ▸ „Emulator-Bild anzeigen“) zeigt das Bild des VICE-Emulators in der IDE: C64, C128 (VIC-II und VDC), VC-20 und Plus/4. Atari und MEGA65 liefern kein Bild.
- **Bild beim Halt:** Bei jedem Halt (Haltepunkt, Einzelschritt, Pause) holt die IDE das Bild über den Monitor des Emulators; es ist genau der Zustand, an dem die Maschine steht. „Bild holen“ holt es sofort (eine laufende Maschine wird dafür kurz angehalten und danach fortgesetzt).
- **Zeitlupen-Live:** Auf Knopfdruck hält die IDE die Maschine alle paar Millisekunden an, holt das Bild und setzt sie fort. Die Maschine läuft dabei mit etwa 40–80 % ihres Tempos und ohne Ton; zum Spielen ist das nicht gedacht. Eine stehende Maschine wird dafür fortgesetzt. Der Modus endet beim Schließen des Reiters oder wenn ein anderer Reiter vorn liegt.
- **C128:** Eine Auswahl schaltet zwischen VIC-II (40 Spalten) und VDC (80 Spalten) um.
- **Text einspeisen:** schreibt Text in den Tastaturpuffer des Emulators (Buchstaben kommen als Großbuchstaben an, Zeilenumbruch ist RETURN), zum Beispiel `RUN`. Das ist nur Text: Tastatur und Joystick werden nicht an den Emulator weitergegeben. Steht die Maschine, wird der Text beim Fortsetzen eingegeben.
- Daneben steht ein **Speicherfenster** (wie im Reiter Speicher). **Als PNG speichern** legt das Bild unter `aufnahmen/emulator-bild-….png` im Projekt ab, „In Zwischenablage“ kopiert es.
- **Projektbild:** „Als Projektbild setzen“ speichert das Bild (zugeschnitten) als `projektbild.png` im Projektordner. Das Bild erscheint in „Zuletzt geöffnete Projekte“ auf der Willkommensseite. Beim ersten Halt legt die IDE es selbst an, falls es noch keines gibt. In den Projekt-Einstellungen kannst du das Projektbild ausschalten, durch eine Bilddatei ersetzen oder löschen.
- Das Bild hat die volle Größe des VICE-Bildpuffers (C64 504 × 312, C128 VIC-II 384 × 272, VDC 856 × 288, Plus/4 384 × 288, VC-20 568 × 284); nicht gezeichnete Ränder sind schwarz.

## Kontextmenüs (rechte Maustaste)
- **Disassembler:** Haltepunkt setzen/entfernen (auch an Adressen ohne Quelltext), **Bis hierher ausführen** (einmaliger Haltepunkt, verschwindet nach dem nächsten Halt), Speicher ab dieser Adresse anzeigen, Adresse als Ausdruck beobachten, **Zur Quellzeile**, Adresse oder Zeile kopieren.
- **Register:** Wert kopieren (hex oder dezimal), als Ausdruck beobachten; bei PC und SP den Speicher dort anzeigen.
- **Speicher:** Zeile als `[$adresse]` beobachten, Schreib- oder Lese-Haltepunkt auf den Zeilenanfang, Adresse oder Zeile kopieren.
- **Beobachten:** Wert (hex/dezimal) oder Ausdruck kopieren, Ausdruck oder alle entfernen.
- **Haltepunkte:** Entfernen, **Bearbeiten** (übernimmt Art, Bereich, Bedingung und Zähler ins Formular „Neu“ und entfernt den alten), Speicher anzeigen, Zur Quellzeile, Adresse kopieren.

