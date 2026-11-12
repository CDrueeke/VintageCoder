# ROM-Dateien

VintageCoder liefert **keine Original-ROMs** (KERNAL, BASIC, Zeichensätze, Betriebssysteme) mit. Wer sie braucht, legt seine **eigenen Abbilder** in den ROM-Ordner. VintageCoder erkennt sie am **Inhalt** (Größe und Prüfsumme); der Dateiname ist egal. Es lädt nichts aus dem Netz und verlinkt auf keine ROM-Sammlung.

*Keine Rechtsberatung: Ob und wie du ein Abbild benutzen darfst, klärst du selbst.*

## Wozu ROMs?
- **Emulatoren:** VICE bekommt `-kernal`, `-basic`, `-chargen` usw., atari800 `-xlxe_rom`, `-osb_rom`, `-basic_rom`, Xemu (MEGA65) `-rom`. Ohne ROM benutzen VICE und atari800 ihre eigenen bzw. eingebauten Ersatz-ROMs; **Xemu braucht eine MEGA65-ROM-Datei von genau 131072 Byte**. Altirra verwaltet seine Firmware selbst.
- **Zeichensätze:** Der Zeichensatz-Editor, der Zeichenwähler und „Standardzeichensatz hinzufügen“ lesen die Schrift des Systems aus deinem ROM. Fehlt es, erscheint eine **eingebaute Ersatzschrift** (Eigenbau, ohne Grafikzeichen) mit Hinweis.

## Wohin damit?
Einstellungen ▸ **ROMs** zeigt den Ordner, den Status je Rolle und bietet „Ordner öffnen“, „Neu prüfen“, „Auswählen …“ und eine Ablagezone: ROM-Dateien oder **ZIP-Archive** dorthin ziehen, die Vorschau bestätigen, fertig. Der Assistent „ROMs einrichten …“ (Willkommensseite, Menü *Datei*, Befehlspalette) führt in drei Schritten dorthin. Auf der Kommandozeile: `vcc roms list|check|path|where|import`.

Suchreihenfolge (die erste Fundstelle je Rolle gilt):
1. Einstellung für eine einzelne Datei (z. B. atari800 „OS-ROM“, Xemu „Ordner mit MEGA65.ROM“)
2. `roms/<system>/` im **Projekt** (steht in `.gitignore`; nur für dieses Projekt)
3. `roms/<system>/` **neben dem Programm** (portabler Modus, siehe unten)
4. **Benutzerordner:** Windows `%APPDATA%\vintagecoder\roms`, macOS `~/.config/vintagecoder/roms`, Linux `$XDG_CONFIG_HOME/vintagecoder/roms` (sonst `~/.config/vintagecoder/roms`)
5. alte Ablage `~/.config/vintagecoder/mega65/MEGA65.ROM`
6. nur auf Wunsch (**„ROMs anderer Programme mitbenutzen“**, Standard aus): ROM-Ordner von VICE, Xemu und atari800, nur lesend
7. mitgelieferte freie Ersatz-ROMs (derzeit keine)

`VC_ROMS` ersetzt 3 bis 5 durch einen einzigen Ordner. **„ROMs aus VICE importieren …“** zeigt die gefundenen, erkannten Dateien und kopiert sie nach Bestätigung in den ROM-Ordner.

## Ordner und Rollen
`roms/<system>/` mit `c64`, `c128`, `vic20`, `plus4`, `atari8`, `mega65`. Dateinamen sind frei; üblich sind:

| System | Rollen (Größe in Byte) |
|---|---|
| C64 | `kernal` (8192), `basic` (8192), `chargen` (4096); optional Laufwerke `dos1541` u. a. |
| C128 | `kernal`, `basiclo`, `basichi` (je 16384), `chargen` (8192); optional `kernal64`, `basic64` |
| VC-20 | `kernal`, `basic` (je 8192), `chargen` (4096) |
| Plus/4, C16 | `kernal`, `basic` (je 16384); optional `functionlo`, `functionhi` |
| Atari 8 Bit | `os-xl` (16384), `os-b`, `os-a` (je 10240), `basic` (8192) – ein Betriebssystem genügt |
| MEGA65 | `rom` (131072) |

Mehrere Dateien derselben Rolle sind erlaubt; die Standardrevision gewinnt, in der Seite „ROMs“ lässt sich eine Datei **festnageln** (`roms.json`). Eine Datei mit passendem Namen und passender Größe, aber unbekannter Prüfsumme, gilt als **Unbekannte Revision** und wird mit Warnung benutzt. Unterordner (eine Ebene) sind erlaubt.

## Portabler Modus
Liegt neben dem Programm ein Ordner `roms/` (oder eine Datei `portable`), gilt er zuerst und neue ROMs werden dort abgelegt, wenn er beschreibbar ist. „Neben dem Programm“ heißt unter Windows und Linux der Ordner der ausführbaren Datei (AppImage: neben der `.AppImage`), unter macOS der Ordner, der `VintageCoder.app` enthält. **Nie im App-Bundle:** Schreiben dort würde die Signatur ungültig machen. Läuft die App noch aus ihrem Download-Ordner („App-Translocation“), ist der Modus nicht verfügbar; einmal nach „Programme“ verschieben. `vcc roms where` zeigt alles.

## Zeichensätze als Verweis
Eine Zeichensatzdatei kann statt der Zeichen einen **Verweis** enthalten: `"base": "rom:c64/chargen#upper"` (Format 2) plus nur deine eigenen Änderungen unter `overrides`. Die Original-Zeichen stehen so weder im Projekt noch im Repository. Verweise: `rom:<system>/<rolle>[@<revision>]#<satz>`, z. B. `rom:c64/chargen#lower`, `rom:atari8/os#standard`, `rom:plus4/kernal#standard`. Mit `@<revision>` (z. B. `@906143-02`) braucht es genau dieses Abbild. Alte Dateien (Format 1) bleiben lesbar.

## Fehlt etwas?
- *„ROM … nicht gefunden“* beim Start: Datei ablegen (Fehlerdialog ▸ „ROMs einrichten …“). Die Meldung nennt Zielordner, erwartete Größe und alle gesuchten Orte.
- Zeichensatz sieht „falsch“ aus: vermutlich die Ersatzschrift (Hinweis im Editor). ROM ablegen, „Neu prüfen“.
- Unbekannte Revision: funktioniert, aber Symbole und Offsets können abweichen.
