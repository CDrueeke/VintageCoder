# Einstellungen

Das Zahnrad ⚙ in der Kopfzeile, **Strg/Cmd+Komma** oder (Mac) **Ansicht ▸ Einstellungen** öffnet die Einstellungen als Reiter. Sie gelten für **alle Projekte** und liegen in `settings.json` im Konfigurationsordner (`~/.config/vintagecoder`, unter Windows `%APPDATA%\vintagecoder`). Die Datei ist nur für dich lesbar (Rechte 0600), weil sie die Passwörter der Hardware enthält. Mit der Umgebungsvariable `VC_SETTINGS` lässt sich eine andere Datei wählen. Die neue und die bisherige Oberfläche teilen sich diesen Konfigurationsordner und lesen dieselbe `settings.json` und dieselbe Liste der zuletzt geöffneten Projekte (`recent.json`); das Aussehen der neuen Oberfläche (Farbschema, Größe, Schrift, Anordnung der Bereiche) steht in einer eigenen Datei, `oberflaeche.json`.

Änderungen an Zielsystemen und Hardware gelten erst nach **Speichern**; **Verwerfen** nimmt sie zurück. Unten erscheinen die beiden Knöpfe und der Hinweis „● Ungespeicherte Änderungen“, solange etwas offen ist. Wo die Datei liegt, steht auf der Seite **Allgemein** im Abschnitt **Dateien**. Auswahlen aus mehreren Knöpfen (Farbschema, Editor-Schalter, Update-Häufigkeit, Debug-Protokoll, Bilder laden) wirken sofort und werden gleich gespeichert.

Die Seiten: **Allgemein**, **Zielsysteme und Hardware**, **ROMs**, **Toolchain**, **Editor**, **Tastenkürzel**.

## Zielsysteme und Hardware
Links stehen die Zielsysteme, nach Hersteller gruppiert (Commodore 8 Bit, Atari 8 Bit, Next-Gen 8 Bit), darüber ein Filter (zum Beispiel `VICE`, `C64` oder eine IP-Adresse). Darunter steht der Standard-Emulator (und die Zahl der Maschinen); ein grüner Punkt heißt, dass er gefunden wurde, ein roter, dass er fehlt, ein grauer, dass keiner eingetragen ist. **Zielsystem hinzufügen** (links unten) nimmt eines der sechs Systeme auf: C64, C128, C16/Plus/4, VC-20, Atari 800 XL/XE, MEGA65. **Zielsystem deaktivieren** (rechts oben) blendet das System aus; die Einträge bleiben erhalten, und das System lässt sich über **Zielsystem hinzufügen** wieder einblenden (es steht dort dann als „deaktiviert“).

Rechts steht das gewählte System. Im Kopf zeigt ein Chip, ob ein Emulator bereit ist, und **Startziel (F5)** wählt, wohin F5 und F6 für dieses System starten: der Standard-Emulator oder eine Maschine des Systems.

### Emulatoren
Ein System kann **mehrere Emulatoren** haben, genau einer davon ist der **Standard** (Chip STANDARD, „Als Standard setzen“). Der Standard-Emulator startet bei F5 und beim Debuggen, wenn das Startziel auf Emulator steht. Die Knöpfe **+ VICE x64sc** und ähnliche über den Karten fügen einen weiteren Emulator der jeweiligen Sorte hinzu. Die Sorten:

| System | Sorten |
|---|---|
| C64 | VICE x64sc (taktgenau), VICE x64 (schnell) |
| C128 | VICE x128 |
| C16 / Plus/4 | VICE xplus4 |
| VC-20 | VICE xvic |
| Atari 800 XL/XE | Altirra (nur Windows), atari800; Debugger: Fujisan (wenn gefunden), atari800 |
| MEGA65 | Xemu (xmega65) |

Je Emulator: Homepage und Lizenz stehen in der Karte; fehlt das Programm, lässt es sich dort **portabel installieren** (siehe [Emulatoren](help:emulators)); geladen wird erst nach deiner Bestätigung mit **Jetzt laden**.
- **Pfad:** ein leerer Pfad heißt, dass automatisch gesucht wird (`PATH` und die üblichen Ordner); „Automatisch suchen“ trägt den gefundenen Pfad ein. Ein Chip zeigt **Automatisch erkannt**, **Gefunden** oder **Fehlt**. Ein eingetragener Pfad wird auf Vorhandensein und Ausführbarkeit geprüft (bei `.app`-Bündeln gilt das Programm darin).
- **Startparameter:** werden an die Kommandozeile angehängt und gelten für F5 und für das Debuggen. Leerzeichen trennen; mit `"…"` oder `'…'` lassen sich Pfade mit Leerzeichen angeben.
- atari800: das **OS-ROM** (XL/XE, wird mit `-xlxe_rom` übergeben). Xemu: der **Ordner mit `MEGA65.ROM`**. Beides sind Einzelfall-Angaben und gelten vor dem ROM-Verzeichnis; im Normalfall legt man die ROMs unter Einstellungen ▸ **ROMs** ab (Hilfe „ROM-Dateien“).
- **▶ Test-Start** startet den Emulator mit genau diesen Parametern und beendet ihn nach kurzer Zeit wieder. Die Meldung zeigt, ob der Start gelang.

Ohne Emulator lässt sich ein System nur auf Hardware starten.

### Echte Hardware
Unter „Physische Zielhardware und Netzwerkgeräte“ stehen die Maschinen des Systems. Anbindbar sind:
- **Ultimate 64 / II+** (C64): Netzwerkadresse (`192.168.1.50` oder `host:port`) und, falls in der Firmware eingestellt, das Netzwerkpasswort. Gestartet werden `.prg`-Dateien über die REST-Schnittstelle (Reset, Laden, RUN). Beim Debuggen sind Speicherzugriff, Pause und Weiter möglich, aber keine Register, kein Einzelschritt und keine Haltepunkte.
- **MEGA65** (Hardware; in den Paketen dieser Version ausgeblendet, im Quelltext vorhanden): Standardverbindung **USB** (serielle Schnittstelle, Liste der gefundenen, Baudrate; leer: 2000000; Programm wird über den Monitor geladen und gestartet) oder **Ethernet** (`etherload`, `mega65_ftp`, ungeprüft: noch nicht an einem Gerät getestet). Starten, Kopieren und Debuggen können je eine eigene Verbindung bekommen. Siehe die Hilfeseite „MEGA65 am Gerät“.

**Verbindung testen** prüft, ob die Maschine antwortet; der Status wechselt von „ungeprüft“ zu „Online“ oder „Nicht erreichbar“. Beim Start der Anwendung findet kein Netzwerkzugriff statt, deshalb bleibt der Status bis zum Test „ungeprüft“. Das Passwort ist verdeckt, **Zeigen** blendet es ein. Für die übrigen Systeme gibt es keine anbindbare Hardware.

## Startziel in der Kopfzeile
In der Kopfzeile steht der Knopf **Ziel** („C64 · VICE x64sc ▾“). Er zeigt das System des Projekts und das Ziel, auf das **Starten (F5)** und **Debuggen (F6)** gehen. Ein Klick öffnet eine Liste mit dem **Emulator** des Systems und den eingetragenen Maschinen; die Wahl gilt für die laufende Sitzung. Ohne Wahl gilt das **Startziel (F5)** aus diesen Einstellungen (oder aus den Projekt-Einstellungen, wenn das Projekt es überschreibt). Eine nicht passende Kombination (zum Beispiel ein Atari-Programm auf der Ultimate) meldet das Protokoll.

## Allgemein
- **Farbschema:** Dunkel, Hell, System oder Hoher Kontrast. Beim allerersten Start der neuen Oberfläche werden Farbschema, Größe und Schrift aus der `view.json` der bisherigen Oberfläche übernommen (nur gelesen, die Datei bleibt unverändert); die Anordnung der Bereiche beginnt neu.
- **Oberfläche:** die Größe der ganzen Oberfläche; auch mit Strg/Cmd+Plus und Strg/Cmd+Minus, zurück mit Strg/Cmd+0. Der Befehl „Darstellung zurücksetzen“ (Befehlspalette, Mac-Menü **Ansicht**) stellt Größe, Editor-Schrift und Layout wieder her.
- **Konfiguration sichern:** Exportieren, Importieren und Zurücksetzen der Einstellungen, siehe [Konfiguration sichern](help:config).
- **Einrichtung:** der Knopf **Assistent starten** öffnet den Konfigurationsassistenten.
- **Fehlersuche:** das Debug-Protokoll (siehe [Feedback senden](help:feedback)).
- **Hilfe, Bilder laden:** Die Bilder des Handbuchs liegen im Internet, nicht in der Programmdatei. „Fragen“ (Standard) fragt beim ersten Öffnen eines Kapitels mit Bildern, „Immer“ lädt ohne Nachfrage, „Nie“ zeigt nur Platzhalter. Beim Laden wird nur die Adresse des Bildes abgerufen; die Bilder bleiben im Zwischenspeicher. Das Feld „Adresse“ lenkt auf einen anderen Server oder einen Ordner um (Umgebungsvariable `VC_HELP_IMAGES` hat Vorrang).
- **Updates:** Hier stellst du ein, ob VintageCoder nach einer neuen Version sucht (*Nur auf Wunsch*, *Täglich*, *Wöchentlich*), und kannst **Jetzt prüfen** und eine übersprungene Version zurücknehmen. Ohne deine Zustimmung geht nie etwas ins Netz; VintageCoder lädt und installiert nichts selbst. Der Abschnitt fehlt in Paketen ohne Update-Prüfung. Siehe [Nach Updates suchen](help:updates).
- **Dateien:** der Ort der Einstellungsdatei, mit dem Knopf **Im Ordner zeigen**.

## Toolchain
Die Seite zeigt, was eingebaut ist:
- **Assembler:** der eingebaute `vcasm`; ein externer Assembler wird nicht gebraucht.
- **Include-Suche:** zuerst der Ordner des Projekts, dann die Systemdateien aus `data/sys`. Zusätzliche Suchpfade stellst du in den **Assembler-Optionen** der Seite mit `-I` ein.
- **BASIC-Dialekt je Zielsystem:** das BASIC eines Projekts folgt dem Dialekt seines Zielsystems; je Quelle lässt er sich in der Projektdatei ändern.
- **Beim Öffnen die zuletzt geöffneten Dateien wiederherstellen:** Standard aus. Ist der Schalter an (Projekt-Einstellungen, Abschnitt Allgemein; in der `.vcprj` als `restore_tabs`), öffnet das Projekt die Reiter der letzten Sitzung wieder, den aktiven zuletzt. Die offenen Dateien merkt sich VintageCoder nur als Pfade in `session.json` neben `layout.json`; fehlende Dateien werden übersprungen, jede Datei erscheint nur einmal als Reiter.
- **Export beim Bauen:** ob Sprites, Zeichensätze und Songs vor dem Assemblieren nach `build/export/` geschrieben werden. Das gilt je Projekt und steht in den Projekt-Einstellungen.

## ROMs
Wo deine ROM-Dateien liegen und was erkannt wurde; siehe [ROM-Dateien](help:roms).

## Editor und Tastenkürzel
- **Editor:** Schriftgröße, Zeilenumbruch, **Zeile beim Verlassen formatieren**, **Menüleiste anzeigen** (nur Windows und Linux), **Einzeltasten der Studios** (D Stift, E Radierer, F Füllen …), **Kürzel-Diagnose**, **Tastaturlayout** und **Tasten auf der Klaviatur** für den Tracker, **Startbild** (An oder Aus), **Takt in der Statusleiste** (siehe [Takt in der Statusleiste](help:clock)) und **Sicherung** unbenannter Reiter nach einem Absturz.
- **Markdown:** hat keine eigenen Einstellungen; die Markdown-Befehle stehen im Menü **Bearbeiten** und in der Befehlspalette (siehe [Markdown](help:markdown)).
- **Tastenkürzel:** die Liste ist fest, Anpassen ist noch nicht möglich. Siehe auch [Tastenkürzel](help:shortcuts).
