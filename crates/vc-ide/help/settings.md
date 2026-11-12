# Einstellungen

**Datei > Einstellungen …** (Strg/Cmd+Komma) öffnet die Einstellungen als Tab. Sie gelten für **alle Projekte** und liegen in `settings.json` im Konfigurationsordner (`~/.config/vintagecoder`, unter Windows `%APPDATA%\vintagecoder`). Die Datei ist nur für dich lesbar (Rechte 0600), weil sie die Passwörter der Hardware enthält. Mit der Umgebungsvariable `VC_SETTINGS` lässt sich eine andere Datei wählen.

Änderungen an Zielsystemen und Hardware gelten erst nach **Änderungen anwenden**; **Zurücksetzen** verwirft sie. Unten steht, wo die Datei liegt, und ein Hinweis, solange etwas ungespeichert ist.

Die Seiten: **Allgemein**, **Zielsysteme und Hardware**, **Toolchain**, **Editor**, **Tastenkürzel**.

## Zielsysteme und Hardware
Links stehen die Zielsysteme, nach Hersteller gruppiert (Commodore 8 Bit, Atari 8 Bit, Next-Gen 8 Bit), darüber ein Filter (zum Beispiel `VICE`, `C64` oder eine IP-Adresse). Darunter steht der Standard-Emulator (und die Zahl der Maschinen); ein grüner Punkt heißt, dass er gefunden wurde, ein roter, dass er fehlt, ein grauer, dass keiner eingetragen ist. **Zielsystem hinzufügen** (links unten) nimmt eines der sechs Systeme auf: C64, C128, C16/Plus/4, VC-20, Atari 800 XL/XE, MEGA65. **Zielsystem entfernen** löscht nur die Einstellungen dieses Systems; eingetragene Maschinen bleiben erhalten.

Rechts steht das gewählte System. Im Kopf zeigt ein Chip, ob ein Emulator bereit ist, und **Startziel (F5)** wählt, wohin F5 und F6 für dieses System starten: der Standard-Emulator oder eine Maschine des Systems.

### Emulatoren
Ein System kann **mehrere Emulatoren** haben, genau einer davon ist der **Standard** (Chip STANDARD, „Als Standard setzen“). Der Standard-Emulator startet bei F5 und beim Debuggen, wenn das Startziel auf Emulator steht. **Weiteren Emulator hinzufügen** bietet die Sorten des Systems an:

| System | Sorten |
|---|---|
| C64 | VICE x64sc (taktgenau), VICE x64 (schnell) |
| C128 | VICE x128 |
| C16 / Plus/4 | VICE xplus4 |
| VC-20 | VICE xvic |
| Atari 800 XL/XE | Altirra (nur Windows), atari800; Debugger: Fujisan (wenn gefunden), atari800 |
| MEGA65 | Xemu (xmega65) |

Je Emulator: Homepage und Lizenz stehen in der Karte; fehlt das Programm, lässt es sich dort **portabel installieren** (siehe [Emulatoren](help:emulators)); das Feld **Quelle** wählt, welche Version gilt (automatisch, portable Versionen, System).
- **Programm:** ein leerer Pfad heißt, dass automatisch gesucht wird (`PATH` und die üblichen Ordner); „Automatisch suchen“ trägt den gefundenen Pfad ein. Ein Chip zeigt „vorhanden“ oder „nicht gefunden“. Ein eingetragener Pfad wird auf Vorhandensein und Ausführbarkeit geprüft (bei `.app`-Bündeln gilt das Programm darin).
- **Parameter:** werden an die Kommandozeile angehängt und gelten für F5 und für das Debuggen. Leerzeichen trennen; mit `"…"` oder `'…'` lassen sich Pfade mit Leerzeichen angeben.
- atari800: das **OS-ROM** (XL/XE, wird mit `-xlxe_rom` übergeben). Xemu: der **Ordner mit `MEGA65.ROM`**. Beides sind Einzelfall-Angaben und gelten vor dem ROM-Verzeichnis; im Normalfall legt man die ROMs unter Einstellungen ▸ **ROMs** ab (Hilfe „ROM-Dateien“).
- **Test-Start** startet den Emulator mit genau diesen Parametern und beendet ihn nach kurzer Zeit wieder. Die Meldung zeigt, ob der Start gelang.

Ohne Emulator lässt sich ein System nur auf Hardware starten.

### Echte Hardware
Unter „Physische Zielhardware und Netzwerkgeräte“ stehen die Maschinen des Systems. Anbindbar sind:
- **Ultimate 64 / II+** (C64): Netzwerkadresse (`192.168.1.50` oder `host:port`) und, falls in der Firmware eingestellt, das Netzwerkpasswort. Gestartet werden `.prg`-Dateien über die REST-Schnittstelle (Reset, Laden, RUN). Beim Debuggen sind Speicherzugriff, Pause und Weiter möglich, aber keine Register, kein Einzelschritt und keine Haltepunkte.
- **MEGA65** (Hardware): Standardverbindung **USB** (serielle Schnittstelle, Liste der gefundenen, Baudrate; leer: 2000000; Programm wird über den Monitor geladen und gestartet) oder **Ethernet** (`etherload`, `mega65_ftp`, ungeprüft: noch nicht an einem Gerät getestet). Starten, Kopieren und Debuggen können je eine eigene Verbindung bekommen. Siehe die Hilfeseite „MEGA65 am Gerät“.

**Verbindung testen** prüft, ob die Maschine antwortet; der Status wechselt von „ungeprüft“ zu „Online“ oder „Nicht erreichbar“. Beim Start der Anwendung findet kein Netzwerkzugriff statt, deshalb bleibt der Status bis zum Test „ungeprüft“. Das Passwort ist verdeckt, „anzeigen“ blendet es ein. Für die übrigen Systeme gibt es keine anbindbare Hardware.

## Läuft auf
In der Befehlsleiste steht das Feld **Auf: Ziel → Startziel**; im Auswahlmenü gibt es den Abschnitt „Läuft auf“: „Standard des Systems“ (das Startziel aus den Einstellungen), „Emulator“ oder eine eingetragene Maschine. **Starten (F5)** und **Debuggen (F6)** gehen dann dorthin; die Wahl wird gemerkt. „Maschinen und Emulatoren verwalten …“ öffnet diese Einstellungen. Eine nicht passende Kombination (zum Beispiel ein Atari-Programm auf der Ultimate) meldet die Ausgabe.

## Allgemein
- **Farbschema:** Dunkel, Hell oder System.
- **Oberfläche:** die Größe der ganzen Oberfläche; auch mit Strg/Cmd+Plus und Strg/Cmd+Minus, zurück mit Strg/Cmd+0. „Darstellung zurücksetzen“ stellt Schema und Größe wieder her.
- **Hilfe, Bilder laden:** Die Bilder des Handbuchs liegen im Internet, nicht in der Programmdatei. „Fragen“ (Standard) fragt beim ersten Öffnen eines Kapitels mit Bildern, „Immer“ lädt ohne Nachfrage, „Nie“ zeigt nur Platzhalter. Beim Laden wird nur die Adresse des Bildes abgerufen; die Bilder bleiben im Zwischenspeicher. Das Feld „Adresse“ lenkt auf einen anderen Server oder einen Ordner um (Umgebungsvariable `VC_HELP_IMAGES` hat Vorrang).
- Der Ort der Einstellungsdatei.

## Toolchain
Die Seite zeigt, was eingebaut ist:
- **Assembler:** der eingebaute `vcasm`; ein externer Assembler wird nicht gebraucht.
- **Include-Suche:** zuerst der Ordner des Projekts, dann die Systemdateien aus `data/sys`. Zusätzliche Suchpfade lassen sich noch nicht einstellen.
- **BASIC-Dialekt je Zielsystem:** das BASIC eines Projekts folgt dem Dialekt seines Zielsystems; je Quelle lässt er sich in der Projektdatei ändern.
- **Beim Öffnen die zuletzt geöffneten Dateien wiederherstellen:** Standard aus. Ist der Schalter an (Projekt-Einstellungen, Abschnitt Allgemein; in der `.vcprj` als `restore_tabs`), öffnet das Projekt die Reiter der letzten Sitzung wieder, den aktiven zuletzt. Die offenen Dateien merkt sich VintageCoder nur als Pfade in `session.json` neben `layout.json`; fehlende Dateien werden übersprungen, jede Datei erscheint nur einmal als Reiter.
- **Export beim Bauen:** ob Sprites, Zeichensätze und Songs vor dem Assemblieren nach `build/export/` geschrieben werden. Das gilt je Projekt und steht in den Projekt-Einstellungen.

## Editor und Tastenkürzel
- **Editor:** Schriftgröße, die Adressspalte (bei BASIC Zeilennummern) und die Einzeltasten in den Grafik-Editoren (D Stift, E Radierer, F Füllen …).
- **Tastenkürzel:** die Liste ist fest, Anpassen ist noch nicht möglich. Siehe auch [Tastenkürzel](help:shortcuts).
