# Einstellungen

Das Zahnrad ⚙ in der Kopfzeile, **Cmd/Strg+Komma** oder das Menü **Ansicht ▸ Einstellungen** öffnet die Einstellungen. Sie gelten für **alle Projekte** und liegen in `settings.json` im Konfigurationsordner (`~/.config/vintagecoder`, unter Windows `%APPDATA%\vintagecoder`). Die Datei ist nur für dich lesbar (Rechte 0600), weil sie die Passwörter der Hardware enthält. Mit der Umgebungsvariable `VC_SETTINGS` lässt sich eine andere Datei wählen. Das Aussehen der Oberfläche (Farbschema, Größe, Schrift, Anordnung der Bereiche) steht in einer eigenen Datei, `oberflaeche.json`.

![Die Einstellungen auf der Seite „Allgemein“: links die Seitenliste (Allgemein, Ziele und Hardware, ROMs, Aussehen, Editor, Toolchain, Tastenkürzel, Sicherung, Info und Update), rechts die Karten Sprache (Deutsch gewählt), Updates, Startbild, Hilfe, Projektordner und Einrichtung. Unten stehen die Knöpfe „Speichern“ und „Verwerfen“.](bild:einstellungen "Die Einstellungen, Seite „Allgemein“.")

Links stehen die Seiten, rechts der Inhalt als Karten. Die Seiten heißen **Allgemein**, **Ziele und Hardware**, **ROMs**, **Aussehen**, **Editor**, **Toolchain**, **Tastenkürzel**, **Sicherung** und **Info und Update**.

Zwei Arten von Werten gibt es:

- **Sofort wirksam und gleich gespeichert:** Sprache, Farbschema, Zoom, Editor-Schrift, Zeilenumbruch, **Zeile beim Verlassen formatieren**, Startbild, die beiden Schalter unter **Tastenkürzel** und das Debug-Protokoll.
- **Entwurf mit Speichern:** Alles auf den Seiten **Allgemein** (Updates, Handbuchbilder, Projektordner), **Ziele und Hardware**, **ROMs** und **Toolchain**. Unten auf der Seite stehen **Speichern** und **Verwerfen** und, solange etwas offen ist, der Hinweis „Ungespeicherte Änderungen“. **Verwerfen** nimmt den Entwurf zurück.

## Allgemein
Die Seite besteht aus Karten:

- **Sprache:** *Automatisch*, *Deutsch* oder *English*. Die Wahl gilt sofort. Englisch ist maschinell übersetzt und noch nicht geprüft.
- **Updates:** ob VintageCoder nach einer neuen Version sucht (*Noch nicht gefragt*, *Nur auf Wunsch*, *Täglich*, *Wöchentlich*) und die **Adresse der Versionsliste** (leer = Standard). Es wird nichts ohne deine Zustimmung gesendet. Siehe [Nach Updates suchen](help:updates).
- **Startbild:** **Startbild beim Programmstart zeigen** (An oder Aus).
- **Hilfe:** **Handbuchbilder online laden** (*Fragen*, *Immer laden*, *Nie laden*) und der **Ort der Handbuchbilder** (leer = Standardadresse). Die Bilder des Handbuchs liegen im Internet, nicht in der Programmdatei. Beim Laden wird nur die Adresse des Bildes abgerufen; die Bilder bleiben im Zwischenspeicher. Ein Ordner statt einer Webadresse wird gelesen statt geladen. Die Umgebungsvariable `VC_HELP_IMAGES` hat Vorrang.
- **Projektordner:** der Ort, an dem **Neues Projekt** neue Projekte vorschlägt (mit **Wählen …**). Leer = Voreinstellung. Das Ändern verschiebt nichts.
- **Einrichtung:** startet den Konfigurationsassistenten: **Schnelleinrichtung (7 Schritte)**, **Alles selbst einstellen (12 Schritte)** oder einen Teil (**ROMs …**, **Emulatoren …**, **Hardware …**, **Darstellung und Editor …**). Siehe [Konfigurationsassistent](help:wizard).
- **Fehlersuche:** der Schalter **Debug-Protokoll in eine Datei schreiben** (siehe [Feedback senden](help:feedback)).
- **Einstellungsdatei:** der Ort der Datei, mit dem Knopf **Im Ordner zeigen**.

## Ziele und Hardware
Links stehen die **Zielsysteme**, die du eingetragen hast. Darunter wählst du mit **System hinzufügen …** und **Hinzufügen** ein weiteres der sechs Systeme auf: C64, C128, C16/Plus/4, VC-20, Atari 800 XL/XE, MEGA65. **System entfernen** (im Kopf des gewählten Systems) nimmt es heraus.

Rechts steht das gewählte System, im Kopf mit Hersteller und Prozessor. Die Karte **Startziel** wählt, wohin F5 und der Start-Knopf für dieses System starten: **Emulator (Standard des Systems)** oder eine Maschine des Systems.

### Emulatoren
Ein System kann **mehrere Emulatoren** haben, genau einer davon ist der **Standard** (Auswahlknopf **Standard**). Der Standard-Emulator startet bei F5 und beim Debuggen, wenn das Startziel auf Emulator steht. Mit **Emulator hinzufügen …** und **Hinzufügen** fügst du einen weiteren der gewählten Art hinzu, **Entfernen** nimmt einen heraus. Die Arten:

| System | Arten |
|---|---|
| C64 | VICE x64sc (taktgenau), VICE x64 (schnell) |
| C128 | VICE x128 |
| C16 / Plus/4 | VICE xplus4 |
| VC-20 | VICE xvic |
| Atari 800 XL/XE | Altirra (nur Windows), atari800; Debugger: Fujisan (wenn gefunden), atari800 |
| MEGA65 | Xemu (xmega65) |

Je Emulator hat die Karte diese Felder:

- **Name** und **Art des Emulators.**
- **Programm (leer = automatisch suchen):** ein leerer Pfad heißt, dass automatisch gesucht wird (`PATH` und die üblichen Ordner). **Suchen** trägt den gefundenen Pfad ein. Eine Zeile darunter zeigt, ob der Emulator gefunden wurde („Gefunden: …“) oder nicht („Nicht gefunden. Pfad von Hand eintragen.“). Ein eingetragener Pfad wird auf Vorhandensein und Ausführbarkeit geprüft (bei `.app`-Bündeln gilt das Programm darin).
- **Testen:** startet den Emulator mit genau diesen Angaben und beendet ihn nach kurzer Zeit wieder. Die Meldung zeigt, ob der Start gelang.
- **Installieren:** fehlt das Programm und gibt es ein festes Paket, bietet die Karte **Portabel installieren (Version) …** an. Sie nennt vorher Name, Version, Größe, Quelle und Ziel; geladen wird erst nach **Jetzt laden** (siehe [Emulatoren](help:emulators)). Gibt es kein Paket, nennt die Karte die Download-Seite oder den Weg über die Paketverwaltung.
- **ROM** (nur atari800, Xemu und MEGA65): bei atari800 das Betriebssystem-ROM, bei Xemu der Ordner mit `MEGA65.ROM`. Das sind Einzelfall-Angaben und gelten vor dem ROM-Ordner; im Normalfall legst du die ROMs über die Seite **ROMs** ab (Hilfe „ROM-Dateien“).
- **Startparameter** (Karte **Argumente**): werden an die Kommandozeile angehängt und gelten für F5 und für das Debuggen. Leerzeichen trennen; mit `"…"` oder `'…'` lassen sich Pfade mit Leerzeichen angeben.

Ohne Emulator lässt sich ein System nur bauen, und es startet nur auf einer Maschine.

### Echte Hardware
Unter **Echte Hardware** stehen die Maschinen des Systems. Mit **… anbinden** trägst du eine ein. Welche es gibt, hängt vom System und vom Paket ab:

- **Ultimate 64 / II+** (C64): **Adresse** (`192.168.1.50` oder `host:port`) und, falls in der Firmware eingestellt, das **Netzwerkpasswort** (verdeckt, **Zeigen** blendet es ein). Der Schalter **Maschine vor der Übertragung zurücksetzen** setzt die Maschine zurück, bevor das Programm kommt. Gestartet werden `.prg`-Dateien über die REST-Schnittstelle (Reset, Laden, RUN). Beim Debuggen sind Speicherzugriff, Pause und Weiter möglich, aber keine Register, kein Einzelschritt und keine Haltepunkte.
- **MEGA65** (Hardware; in den Paketen dieser Version ausgeblendet, im Quelltext vorhanden): Die **Verbindung** ist **USB** (**Serielle Schnittstelle**, Liste der gefundenen, **Baudrate**; leer: 2000000; das Programm wird über den Monitor geladen und gestartet) oder **Ethernet** (`etherload`, `mega65_ftp`, **Netzwerkschnittstelle des Rechners**, **Ordner auf der SD-Karte**; ungeprüft: noch nicht an einem Gerät getestet). Siehe die Hilfeseite „MEGA65 am Gerät“.

Der Schalter **Nach der Übertragung RUN ausführen** gilt für jede Maschine. **Verbindung testen** prüft, ob die Maschine antwortet; die Meldung zeigt das Ergebnis. Beim Start der Anwendung findet kein Netzwerkzugriff statt, die Verbindung wird nur auf Klick geprüft. Für die übrigen Systeme gibt es keine anbindbare Hardware.

## ROMs
Wo deine ROM-Dateien liegen und was erkannt wurde; siehe [ROM-Dateien](help:roms).

## Aussehen
- **Farbschema:** *System*, *Dunkel*, *Hell* oder *Hoher Kontrast*. Beim allerersten Start werden Farbschema, Größe und Schrift aus der `view.json` einer älteren Installation übernommen (nur gelesen, die Datei bleibt unverändert).
- **Zoom:** die Größe der ganzen Oberfläche mit **−**, **+** und **100 %**; auch mit Cmd/Strg+Plus, Cmd/Strg+Minus und Cmd/Strg+0.
- **Darstellung zurücksetzen (Farbschema, Zoom, Editor):** stellt die Vorgaben wieder her. Der Befehl **Darstellung zurücksetzen (Größe, Schrift, Layout)** in der Befehlspalette und im Menü **Ansicht** setzt zusätzlich das Layout zurück.

## Editor
- **Editor-Schrift:** **−** und **+**, Größe in Punkt.
- **Zeilenumbruch** und **Zeile beim Verlassen formatieren** (bei Assembler und BASIC wird die bearbeitete Zeile ausgerichtet, sobald du sie verlässt).
- **Menüleiste im Fenster:** unter macOS steht die Menüleiste des Systems zur Verfügung, sonst ist die Menüleiste im Fenster eingeschaltet. Diese Karte zeigt nur den Stand an.

Markdown hat keine eigenen Einstellungen; die Markdown-Befehle stehen im Menü **Bearbeiten** und in der Befehlspalette (siehe [Markdown](help:markdown)).

## Toolchain
Die Seite zeigt, was eingebaut ist:
- **Assembler:** der eingebaute `vcasm` („Eingebaut (vc-asm). Kein weiteres Programm nötig.“). Die Karte **Assembler-Optionen beim Bauen** nimmt zusätzliche Optionen auf, zum Beispiel `-I` für Suchordner.
- **Include-Suche:** zuerst der Ordner des Projekts, dann zusätzliche Ordner (`-I`), zuletzt die Systemdateien von VintageCoder.
- **BASIC-Dialekt je Zielsystem:** das BASIC eines Projekts folgt dem Dialekt seines Zielsystems. Zeilennummern sind optional (Bereich Projekt, Seite Quellen).

Ob ein Projekt beim Öffnen die Reiter der letzten Sitzung wiederherstellt, stellst du in den Einstellungen des Projekts ein (Bereich Projekt, Seite Allgemein), nicht hier.

## Tastenkürzel
- **Einzeltasten der Editoren:** Buchstaben und Ziffern wählen Werkzeuge und tragen Noten ein. Aus: nur Kürzel mit Cmd/Strg oder Alt wirken.
- **Computertasten auf der Klaviatur anzeigen:** zeigt im Tracker die Tasten auf der Klaviatur. Die Tasten gelten nach ihrer Lage auf der Tastatur, daher ist keine Wahl des Tastaturlayouts nötig.
- **Tastenkürzel:** eine Liste aller Befehle mit Kürzel und ein Suchfeld **Befehl oder Kürzel suchen**. Die Kürzel sind fest; ein Anpassen ist nicht möglich. Siehe auch [Tastenkürzel](help:shortcuts).

## Sicherung
Hier exportierst, importierst und setzt du die Konfiguration zurück; siehe [Konfiguration sichern](help:config).

## Info und Update
Name, Version, Lizenz, Webseite, Quelltext, Feedback, die Suche nach Updates und die Drittanbieter-Hinweise. Siehe [Über VintageCoder](help:about) und [Nach Updates suchen](help:updates).
