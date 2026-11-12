# Sprachserver für andere Editoren (`vc-lsp`)

Zu VintageCoder gehört ein Sprachserver, `vc-lsp`. Er spricht das Language Server Protocol über Standardeingabe und Standardausgabe und lässt sich deshalb in jeden Editor einbinden, der Sprachserver unterstützt. Die Entwicklungsumgebung `vintagecoder` selbst braucht ihn nicht; sie hat ihre eigene Prüfung eingebaut.

## Was er kann

- Fehler und Warnungen des Assemblers beim Tippen (Diagnosen)
- Hover: bei Zahlen der Wert in mehreren Schreibweisen und, wenn die Zahl eine Chip-Adresse ist, das passende Register; bei Befehlen die Erklärung mit Flags und Opcodes; bei Direktiven ihre Beschreibung
- Vervollständigung (auch nach `.` und `$`)
- Gehe zu Definition
- Gliederung der Datei (Symbole)

Er arbeitet mit dem Assembler `vcasm` (Dateien mit der Endung `.asm`). Die Daten zu Zielsystemen, Prozessoren und Chips liest er aus dem Datenordner `data/`.

## Einbinden

1. Den Editor so einrichten, dass er für `.asm`-Dateien den Befehl `vc-lsp` ohne Argumente startet. Der Server wird vom Editor gestartet, nicht von Hand; ohne Eingabe wartet er und gibt nichts aus.
2. Zielsystem und Datenordner lassen sich beim Start über die `initializationOptions` des Protokolls mitgeben:
   - `target`: Kennung des Zielsystems, Standard ist `c64`
   - `dataDir`: Pfad zum Ordner `data/` (Alternative: Umgebungsvariable `VC_DATA`)
3. Fehlen beide Angaben, sucht der Server `data/` im Projektordner und neben dem Programm, bei einer Installation auch unter `<prefix>/share/vintagecoder/data`. Findet er nichts, meldet er das als Fehler statt zu raten.

## Prüfen

`vc-lsp --version` zeigt die Version, `vc-lsp --help` die Kurzhilfe. Die man-Seite heißt `vc-lsp(1)`.

Der Sprachserver ist ein Hilfsmittel für den Assembler. Für BASIC-Dateien (`.bas`) bietet er derzeit keine Unterstützung. Bauen, Starten, Debuggen und die Grafik- und Musikeditoren gibt es in der Entwicklungsumgebung und auf der Kommandozeile (`vcc`).
