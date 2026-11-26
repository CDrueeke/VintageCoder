# MEGA65 am Gerät (USB und Ethernet)

> **Ungeprüft:** Der Ethernet-Weg (Starten, Kopieren, Einbinden) wurde bisher **nur gegen Attrappen-Befehle** getestet, nicht an einem echten MEGA65. Aufrufzeilen und Fehlerhinweise stammen aus dem MEGA65-Wiki und GitHub-Beiträgen. Der USB-Weg (eigener Monitor) ist über Xemu abgesichert, aber ebenfalls nicht am echten Gerät erneut geprüft.

## Zwei Verbindungen
Eine Maschine vom Typ MEGA65 trägst du unter Einstellungen ▸ **Ziele und Hardware** ▸ MEGA65 ▸ **Echte Hardware** mit dem Knopf **… anbinden** ein (diese Anbindung ist in den Paketen dieser Version ausgeblendet). Die Karte hat das Feld **Verbindung** mit **USB** und **Ethernet**. Die Oberfläche stellt eine Verbindung je Maschine ein; die Tabelle zeigt, was sie je Aktion bedeutet.

![Einstellungen, Seite „Ziele und Hardware“ mit gewähltem System Commodore 64. Die Karte „Emulatoren“ zeigt VICE x64sc (taktgenau) als Standard, das Programmfeld mit „automatisch“, die Knöpfe „Suchen“ und „Testen“ und die grüne Meldung „Gefunden“ mit dem Pfad. Darunter stehen „Startparameter“ (eingeklappt), „Entfernen“, „Emulator hinzufügen ...“ und unter „ECHTE HARDWARE“ der Knopf „Ultimate 64 / II+ (Netzwerk) anbinden“.](bild:einstellungen-emulator "Die Emulator-Karte: Programm suchen und mit „Testen“ prüfen.")

| Aktion | USB | Ethernet |
|---|---|---|
| Starten (F5) | ja, eigener Monitor, kein Fremdwerkzeug | `etherload -r` (Fremdwerkzeug) |
| Disketten kopieren und einbinden | nein (zurückgestellt) | `mega65_ftp` (`put`), dann `etherload -m` |
| Debuggen (F6) | ja | nein |

Die IDE liefert die Werkzeuge **nicht** mit und lädt nichts nach. `etherload` und `mega65_ftp` gehören zum Paket `mega65-tools` (github.com/MEGA65/mega65-tools). Die Pfade (Felder `etherload` und `mega65_ftp`) stehen bei der Maschine, sobald **Ethernet** gewählt ist; leer heißt: im `PATH` suchen.

## Ethernet einrichten (am Gerät)
1. Core **0.96 oder neuer**; `etherload` gilt noch als BETA.
2. DIP-Schalter 2 auf ON (neuere Geräte haben ihn schon gesetzt).
3. Gültige **MAC-Adresse** (nicht `00:00:00:00:00:00`): ALT beim Einschalten, `1`, Abschnitt NETWORK, `R`.
4. **Fernzugriff einschalten mit Shift + £** (die LED blinkt grün/gelb). Das gilt nur bis zum nächsten Reset oder Ausschalten.
5. PC und MEGA65 im selben Netz, am sichersten direkt per Kabel oder über einen einfachen Switch (IPv6 Link-Local). WLAN und Firmennetze blockieren das oft.
6. Im Feld **Netzwerkschnittstelle des Rechners** die Schnittstelle des PCs eintragen (macOS z. B. `en0`, Linux `eth0`, Windows der Index).

**Sicherheit:** Der Fernzugriff steht jedem im selben Netz offen (Steuerung und SD-Karte). Nicht in fremden Netzen einschalten.

## Programm starten
F5 mit der Maschine als Startziel. Über Ethernet läuft `etherload -r <Programm>` (lädt und startet; ohne den Schalter **Nach der Übertragung RUN ausführen** wird nur geladen). Die Ausgabe des Werkzeugs steht im Protokoll. Antwortet das Gerät nicht, erinnert die Meldung an Fernzugriff (Shift+£), DIP-Schalter, MAC-Adresse und Netz.

## Diskette auf die SD-Karte, einbinden, starten
- **Disk-Editor:** bei einer `.d81` erscheint „Auf MEGA65 kopieren“: Kopieren per `mega65_ftp` in den **Ordner auf der SD-Karte** (Feld der Maschine; leer = oberste Ebene; der Ordner wird angelegt) und Einbinden per `etherload -m`. Der Knopf steht im Disk-Editor der neuen Oberfläche noch nicht zur Verfügung.
- **F5 mit Diskette:** Hat das Projekt eine `.d81`-Diskette mit dem Startprogramm vorn und stehen Starten **und** Kopieren auf Ethernet, wird die Diskette kopiert, eingebunden und das Programm gestartet.
- Dateinamen ohne Leerzeichen verwenden; `mega65_ftp` bekommt die Befehle als Zeilen.

## Verbindung testen
Der Knopf **Verbindung testen** der Maschine prüft die gewählte Verbindung. USB prüft, ob der Monitor antwortet; Ethernet prüft, ob beide Werkzeuge gefunden werden und `mega65_ftp` das Verzeichnis lesen kann.

## Aufrufzeilen anpassen
Die Optionen der Werkzeuge stehen in einer Datendatei (`data/mega65/tools.json`, eingebaut). Eine Datei `mega65-tools.json` im Konfigurationsordner (`~/.config/vintagecoder`, Windows `%APPDATA%\vintagecoder`) ersetzt sie, falls sich Optionen der Werkzeuge ändern oder am Gerät anders herausstellen. Platzhalter: `{iface}`, `{file}`, `{image}`, `{dir}`, `{localdir}`, `{name}`; `{iface_args}` entfällt, wenn keine Schnittstelle eingetragen ist.

## Offen bis zum ersten Test am Gerät
Genaue Optionen und Schnittstellenangabe von `etherload` und `mega65_ftp`, Verhalten bei vorhandenem SD-Ordner, ob `etherload -m … -r …` Einbinden und Starten in einem Aufruf erledigt, Meldungen der Werkzeuge bei Fehlern. Eigene Übertragung über USB: zurückgestellt.
