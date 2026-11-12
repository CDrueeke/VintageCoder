# Emulatoren

VintageCoder startet und debuggt Programme in externen Emulatoren. Die Emulatoren gehören nicht zu VintageCoder: sie werden **nicht mitgeliefert**, sondern von dir installiert oder auf Wunsch portabel geladen. Jeder Emulator läuft als eigener Prozess; die Programme der Emulatoren stehen unter ihren eigenen Lizenzen.

## Übersicht

<!-- EMULATOR-TABLE -->

<!-- EMULATOR-SECTIONS -->
## Portabel installieren

Wo es für dein System ein Paket gibt, lädt VintageCoder auf Wunsch eine **feste Version** von der Seite des Projekts, prüft die SHA-256-Summe und packt sie unterhalb von VintageCoder aus (`emulators/<Emulator>/<Version>` im Konfigurationsordner, unter Windows `%APPDATA%\VintageCoder`). Es wird nie ohne deinen Klick geladen, und nichts wird verändert außerhalb dieses Ordners.

- **Oberfläche:** Einstellungen ▸ Zielsysteme ▸ Emulator-Karte ▸ **Portabel installieren …** (erst nach „Jetzt laden“ geschieht der Download). Gibt es mehrere Versionen, wählst du sie in der Liste daneben. Für Fujisan gibt es auf der Atari-Seite eine eigene Karte.
- **Kommandozeile:** `vcc emulator list`, `vcc emulator status [emulator]`, `vcc emulator install <emulator>[@version]` (mit `--dry-run` nur anzeigen). Siehe [Kommandozeile](help:cli).

Wo es kein Paket gibt (zum Beispiel macOS oder VICE unter Linux), nennt die Karte die Download-Seite oder den Weg über die Paketverwaltung. Dann trägst du den Pfad ein oder lässt automatisch suchen.

## Mehrere Versionen

Von einem Emulator können mehrere Versionen nebeneinander liegen: portabel installierte (jede in einem eigenen Ordner) und die, die dein System selbst mitbringt (`PATH`, Programmordner). Die Emulator-Karte in den Einstellungen hat dafür das Feld **Quelle**: *automatisch*, jede portable Version („portabel 3.9“) oder *System*. Die Wahl trägt den Pfad ein; ein eigener Pfad bleibt jederzeit möglich. Für verschiedene Projekte lassen sich mehrere Einträge desselben Emulators mit unterschiedlichen Pfaden anlegen und im Projekt oder Wizard als Standard-Emulator wählen.

Auf der Kommandozeile wählt `vcc test --backend vice@3.9` eine bestimmte portable Version, `vcc emulator install vice@3.9` installiert sie. Versionen werden nach Zahlen verglichen („3.10“ ist neuer als „3.9“). Eine neue Version eines Emulators wird nicht automatisch angeboten: der Katalog liegt im Programm und wächst mit neuen Fassungen von VintageCoder; neuere Versionen kannst du selbst installieren und als Pfad eintragen.

## Wie VintageCoder den Emulator findet

1. der **Pfad** in den Einstellungen (leer = automatisch suchen),
2. eine mit `@version` gewählte portable Version (nur auf der Kommandozeile),
3. das, was das **System** bietet: `PATH` und die üblichen Programmordner (Windows: `Programme\…`, Linux: `/usr/games`, `~/.local/bin` u. a., macOS: Homebrew),
4. zuletzt die **neueste portabel installierte** Version.

Siehe auch [Einstellungen](help:settings), [Debugger](help:debugger) und [ROM-Dateien](help:roms).
