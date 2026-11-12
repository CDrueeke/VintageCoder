# Hilfe der Kommandozeile (`data/cli/*.json`)

Die Hilfe von `vcc` (und die man-Seiten, die Vervollständigung und die IDE-Seiten) entsteht aus **einer** Beschreibung: den Dateien unter `data/cli/`. Sie liegen im Programm eingebettet (`crates/vc-cli/src/help.rs`), `vcc` braucht also keinen Datenordner für die Hilfe. Schlüssel sind englisch, Anzeigetexte deutsch (wie bei `data/args`, siehe `args-catalog.md`).

Aufrufe:

```
vcc                       kurze Übersicht (Exit-Code 2)
vcc --help | -h | help    Übersicht (stdout, Exit-Code 0)
vcc help --all            alle Befehle mit allen Optionen
vcc help <befehl> [form]  Hilfe eines Befehls (auch: vcc <befehl> --help)
vcc help env | exit-codes Themenseiten
vcc man [befehl]          man-Seite (roff) nach stdout
vcc completion <shell>    Vervollständigung: powershell | bash | zsh | fish
vcc help --markdown        Handbuch als Markdown (docs/cli.md)
```

## Dateien

| Datei | Inhalt |
|---|---|
| `data/cli/_vcc.json` | Programm: Zusammenfassung, Gruppen der Übersicht, Umgebungsvariablen, Rückgabewerte |
| `data/cli/<befehl>.json` | ein Befehl (Name wie auf der Kommandozeile), eine Datei je Befehl |

## Befehl (`<befehl>.json`)

```json
{
  "command": "inc",
  "group": "daten",
  "summary": "Include-Datei mit Hardwarekonstanten ausgeben",
  "description": ["Optionale Absätze (Array von Strings), die unter dem Aufruf stehen."],
  "aliases": [],
  "forms": [
    {
      "id": "",
      "usage": "vcc inc --target <c64|atari8> [optionen]",
      "summary": "nur nötig, wenn der Befehl mehrere Formen hat",
      "args": [ { "name": "datei.asm", "required": true, "repeat": false, "help": "Quelltext" } ],
      "option_groups": [ { "id": "ziel", "label": "Zielsystem" } ],
      "options": [
        { "flag": "--target", "short": "", "value": "system", "group": "ziel",
          "help": "Zielsystem der Konstanten.", "choices": ["c64", "atari8"],
          "default": "", "default_text": "", "repeat": false, "required": true }
      ],
      "examples": [ { "title": "C64-Konstanten in eine Datei", "cmd": "vcc inc --target c64 -o c64.inc" } ],
      "notes": ["Hinweise, Grenzen"],
      "exit": { "0": "fehlerfrei", "1": "Fehler" },
      "see_also": ["asm"]
    }
  ]
}
```

Felder:

- `command`: Name des Befehls. `group`: Kennung einer Gruppe aus `_vcc.json`. `summary`: eine Zeile, höchstens 70 Zeichen, ohne Schlusspunkt. `aliases`: weitere Namen, die dasselbe tun (z. B. `asset` für `media`).
- `forms`: eine Form je Aufrufart. Hat ein Befehl nur eine, ist `id` leer. Hat er mehrere (z. B. `basic --tokenize`, `basic ast`), bekommt jede eine `id`, einen `usage` und eine `summary`; `vcc help basic ast` zeigt eine Form.
- `usage`: genau die Zeile, die man tippt, mit Platzhaltern in spitzen Klammern und Optionalem in eckigen: `vcc asm <datei.asm> [optionen]`.
- `args`: Positionsargumente in der Reihenfolge des Aufrufs. `required`, `repeat` (mehrfach erlaubt).
- `options`: `flag` ist die Schreibweise im Code (`--output` oder `-D`), `short` die Kurzform (leer, wenn es keine gibt). `value`: Platzhalter des Wertes ohne Klammern (leer bei Schaltern ohne Wert). `choices`: nur, wenn der Code eine feste Menge prüft; genau die Werte aus dem Code. `default`: der Wert, den der Code ohne Option benutzt; `default_text`: Klartext, wenn der Standard eine Regel ist („Format des Zielsystems“). `repeat`: mehrfach erlaubt (`-D`, `-I`). `group`: Kennung aus `option_groups` (Überschrift in der Hilfe).
- `examples`: **mindestens eines je Form**, bei größeren Befehlen zwei bis vier, vom einfachsten bis zu einem realen Fall. `cmd` muss mit dem Parser des Befehls gültig sein. `"run": true` markiert ein Beispiel, das der Test mit Beispieldateien wirklich ausführen darf (nur wenn es ohne Emulator, Netz und Hardware läuft).
- `exit`: Rückgabewerte, nur wenn der Befehl andere als die Standardwerte (0 fehlerfrei, 1 Fehler, 2 falscher Aufruf) hat.
- `see_also`: Namen anderer Befehle.

## Regeln für die Texte

- Deutsch, sachlich, kurze Sätze. Beschreibe, was der Code **tut**, nicht was er tun sollte. Wenn du etwas nicht sicher weißt, schreibe weniger.
- Jede Option, die der Parser des Befehls kennt, steht in der Beschreibung, jede Option der Beschreibung im Parser (der Test prüft das gegen `main.rs`).
- Hilfetext einer Option: ein bis zwei Sätze; was sie bewirkt, nicht wie sie heißt. Keine Wiederholung von Standardwert oder Werteliste (das zeigt die Hilfe aus `default` und `choices`).
- Beispiele: reale Dateinamen aus den Beispielprojekten (`examples/…`) sind gut; jedes Beispiel hat einen kurzen Titel in der Befehlsform („übersetzen“, nicht „Dies übersetzt“).
- Keine Zeilenumbrüche in Strings; die Hilfe bricht selbst um.

## Erzeugte Dateien und Prüfungen

- `installer/linux/man/*.1`: erzeugt mit `vcc man --all installer/linux/man` (20 Seiten: `vcc`, je ein Befehl, `vintagecoder`, `vc-lsp`).
- `docs/cli.md`: erzeugt mit `vcc help --markdown > docs/cli.md`.
- Nach jeder Änderung an `data/cli/*.json` beide neu erzeugen; die Tests in `crates/vc-cli/src/help.rs` schlagen sonst fehl (sie nennen den Befehl).
- Die Tests vergleichen außerdem alle Optionen im Quelltext von `main.rs` und `roms.rs` mit der Beschreibung (beide Richtungen) und prüfen die Beispiele auf bekannte Befehle und Optionen.
