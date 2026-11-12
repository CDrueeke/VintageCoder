# ROM-Tabelle

`known.json` führt **nur Größen und Prüfsummen** (CRC-32, SHA-1) bekannter ROM-Abbilder, damit VintageCoder Dateien im ROM-Ordner am Inhalt erkennt
(egal wie sie heißen). Es sind **keine ROM-Daten** und keine Zeichensätze enthalten. Die Einträge werden aus echten Dateien erzeugt, nie von Hand getippt:

```
vcc roms emit-known --into data/roms/known.json --vice <VICE-Datenordner> --file mega65.rom --as mega65/rom --rev Open-ROMs --free
```

Die Revisionsnummern stammen aus den Dateinamen der Quellen (VICE, Xformer-Satz) und sind nicht unabhängig geprüft.

## Wo VintageCoder ROM-Dateien sucht (erste Fundstelle je Rolle gewinnt)

1. Einstellungen (einzelne Datei, z. B. atari800 „OS-ROM“)
2. `<Projekt>/roms/<system>/`
3. **portabler Ordner** `roms/<system>/` neben dem Programm (siehe unten)
4. Benutzerordner `<config>/roms/<system>/`: Windows `%APPDATA%\vintagecoder\roms`, macOS `~/.config/vintagecoder/roms`, Linux `$XDG_CONFIG_HOME/vintagecoder/roms` (sonst `~/.config/vintagecoder/roms`)
5. alte Ablage `<config>/mega65/MEGA65.ROM`
6. nur auf Wunsch: ROM-Ordner anderer Programme (VICE-Datenordner, Xemu, `~/.atari800.cfg`), nur lesend
7. mitgelieferte freie Ersatz-ROMs `data/roms-open/<system>/`

Die Umgebungsvariable `VC_ROMS` ersetzt 3 bis 5 durch einen einzigen Ordner (Tests, feste Ablage).

Systemordner: `c64`, `c128`, `vic20`, `plus4`, `atari8`, `mega65`. Dateinamen sind frei; Konvention: `kernal.bin`, `basic.bin`, `chargen.bin`,
`mega65.rom`, `os-xl.rom`, `os-b.rom`, `os-a.rom`. Ein Unterordner pro System darf eine Ebene tiefer gehen. Mit `roms.json` im ROM-Ordner lässt sich eine Datei
festnageln: `{"c64": {"kernal": "kernal-901227-02.bin"}}`.

## Portabler Modus

Aktiv, wenn neben dem Programm ein Ordner `roms/` existiert oder eine Datei `portable` liegt.

| Plattform | „Neben dem Programm“ |
|---|---|
| Windows, Linux | Ordner der ausführbaren Datei (AppImage: Ordner der `.AppImage`-Datei) |
| macOS | der Ordner, der `VintageCoder.app` enthält (z. B. `Programme/roms`) |

**Nie** innerhalb des macOS-App-Bundles (auch nicht `Contents/Resources`): Schreiben ändert ein signiertes/notarisiertes Bundle und macht die Signatur ungültig.
Läuft die App noch unter „App-Translocation“ (frisch heruntergeladen, nicht verschoben), ist der Modus nicht verfügbar. Neue ROMs (Import, Drag-and-drop)
gehen in den portablen Ordner, wenn er aktiv und beschreibbar ist (unter „Programme“ bzw. „Program Files“ meist nicht), sonst in den Benutzerordner.

`vcc roms where` zeigt Schreibziel, portablen Modus und alle gesuchten Orte.
