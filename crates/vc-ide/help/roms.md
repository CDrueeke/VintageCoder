# ROM-Dateien

VintageCoder liefert **keine Original-ROMs** (KERNAL, BASIC, Zeichensätze, Betriebssysteme) mit. Wer sie braucht, legt seine **eigenen Abbilder** in den ROM-Ordner. VintageCoder erkennt sie am **Inhalt** (Größe und Prüfsumme); der Dateiname ist egal. Es lädt nichts aus dem Netz und verlinkt auf keine ROM-Sammlung.


## Wozu ROMs?
- **Emulatoren:** VICE bekommt `-kernal`, `-basic`, `-chargen` usw., atari800 `-xlxe_rom`, `-osb_rom`, `-basic_rom`, Xemu (MEGA65) `-rom`. Ohne ROM benutzen VICE und atari800 ihre eigenen bzw. eingebauten Ersatz-ROMs; **Xemu braucht eine MEGA65-ROM-Datei von genau 131072 Byte**. Altirra verwaltet seine Firmware selbst.
- **Zeichensätze:** Der Zeichensatz-Editor liest die Schrift des Systems aus deinem ROM. Fehlt es, erscheint der **eigene Standardsatz** von VintageCoder (VC-Standard, CC0, weitergebbar) mit Hinweis; im Dialog **Neues Projekt erstellen** wählst du unter **Zeichensatz des Projekts**, ob er als Kopie ins Projekt kommt (`vcc new --charset`).

## Wohin damit?
Die einfachste Stelle ist die Seite **Einstellungen ▸ ROMs**; dieselbe Seite zeigt der Schritt **ROMs** im Konfigurationsassistenten (**Konfigurationsassistent: nur ROMs …** im Menü **Hilfe** und in der Befehlspalette, siehe [Konfigurationsassistent](help:wizard)). Alternativ legst du die Dateien von Hand in den ROM-Ordner (siehe die Suchreihenfolge unten) oder auf der Kommandozeile ab: `vcc roms list|check|path|where|import`. `vcc roms where` zeigt, wo VintageCoder sucht.

![Einstellungen, Seite ROMs. Links die Liste der Seiten, rechts oben die Knöpfe ROM-Ordner öffnen und Neu prüfen, darunter ROM-Datei oder Zip wählen, ROMs anderer Programme suchen und Fundliste kopieren. Reiter je System (C64 gewählt) und je Rolle der Status Fehlt mit erwarteter Größe, unten die durchsuchten Orte.](bild:einstellungen-roms "Die Seite ROMs zeigt je System und Rolle, ob eine Datei gefunden wurde; hier fehlt noch alles.")

Die Seite hat diese Teile:

- **ROM-Ordner öffnen** zeigt den Ordner, **Neu prüfen** liest ihn neu ein.
- **ROMs anderer Programme mitbenutzen (VICE, Xemu, atari800)** (Standard: aus): VintageCoder liest dann auch die ROM-Ordner dieser Programme. Es wird nur gelesen; nichts wird kopiert oder verändert.
- **ROMs übernehmen:** **ROM-Datei oder Zip wählen …** und **ROMs anderer Programme suchen** zeigen eine Liste dessen, was erkannt wurde, mit System, Rolle und der Aktion („wird kopiert“ oder „schon vorhanden“). Jede Zeile hat einen Haken; erst **n übernehmen** kopiert die angehakten Dateien in den ROM-Ordner. **Fundliste kopieren** legt die Liste in die Zwischenablage. Dateien lassen sich auch auf das Fenster ziehen. Es wird nichts ohne Bestätigung kopiert, und ROMs werden nie heruntergeladen.
- **Systeme:** je System ein Reiter mit einer Ampel (grün: alles Nötige erkannt, gelb: Ersatz oder unbekannte Revision, rot: Pflichtrolle fehlt). Darunter steht je Rolle (zum Beispiel KERNAL, BASIC, Zeichensatz) der Status (*Erkannt*, *Unbekannte Revision*, *Ersatz (frei)*, *Fehlt*, *Nicht benutzbar*), der Dateiname und, wenn gefunden, Fundort, Revision, Größe, CRC-32 und SHA-1. Fehlt eine Datei, steht die erwartete Größe in Byte. Gibt es mehrere Dateien derselben Rolle, wählst du eine fest (**Automatisch** oder ein Dateiname); die Wahl landet in `roms.json`.
- **Durchsuchte Orte:** alle Orte der Suchreihenfolge mit der Zahl der Dateien.

Die Seite gehört zum Entwurf der Einstellungen: **ROMs anderer Programme mitbenutzen** gilt erst nach **Speichern**.

Suchreihenfolge (die erste Fundstelle je Rolle gilt):
1. Einstellung für eine einzelne Datei (z. B. atari800 „OS-ROM“, Xemu „Ordner mit MEGA65.ROM“)
2. `roms/<system>/` im **Projekt** (steht in `.gitignore`; nur für dieses Projekt)
3. `roms/<system>/` **neben dem Programm** (portabler Modus, siehe unten)
4. **Benutzerordner:** Windows `%APPDATA%\vintagecoder\roms`, macOS `~/.config/vintagecoder/roms`, Linux `$XDG_CONFIG_HOME/vintagecoder/roms` (sonst `~/.config/vintagecoder/roms`)
5. alte Ablage `~/.config/vintagecoder/mega65/MEGA65.ROM`
6. nur auf Wunsch (**„ROMs anderer Programme mitbenutzen“**, Standard aus): ROM-Ordner von VICE, Xemu und atari800, nur lesend
7. mitgelieferte freie Ersatz-ROMs (derzeit keine)

`VC_ROMS` ersetzt 3 bis 5 durch einen einzigen Ordner. **`vcc roms import` kopiert erkannte Dateien in den ROM-Ordner.

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

Mehrere Dateien derselben Rolle sind erlaubt; die Standardrevision gewinnt, eine Datei lässt sich **festnageln** (`roms.json`; auf der Seite **ROMs** wählst du sie je Rolle aus). Eine Datei mit passendem Namen und passender Größe, aber unbekannter Prüfsumme, gilt als **Unbekannte Revision** und wird mit Warnung benutzt. Unterordner (eine Ebene) sind erlaubt.

## Portabler Modus
Liegt neben dem Programm ein Ordner `roms/` (oder eine Datei `portable`), gilt er zuerst und neue ROMs werden dort abgelegt, wenn er beschreibbar ist. „Neben dem Programm“ heißt unter Windows und Linux der Ordner der ausführbaren Datei (AppImage: neben der `.AppImage`), unter macOS der Ordner, der `VintageCoder.app` enthält. **Nie im App-Bundle:** Schreiben dort würde die Signatur ungültig machen. Läuft die App noch aus ihrem Download-Ordner („App-Translocation“), ist der Modus nicht verfügbar; einmal nach „Programme“ verschieben. `vcc roms where` zeigt alles.

## Zeichensätze als Verweis
Eine Zeichensatzdatei kann statt der Zeichen einen **Verweis** enthalten: `"base": "rom:c64/chargen#upper"` (Format 2) plus nur deine eigenen Änderungen unter `overrides`. Die Original-Zeichen stehen so weder im Projekt noch im Repository. Verweise: `rom:<system>/<rolle>[@<revision>]#<satz>`, z. B. `rom:c64/chargen#lower`, `rom:atari8/os#standard`, `rom:plus4/kernal#upper` (Plus/4: `upper` und `lower`; `standard` gilt weiter als `upper`). Mit `@<revision>` (z. B. `@906143-02`) braucht es genau dieses Abbild. Alte Dateien (Format 1) bleiben lesbar.

## Fehlt etwas?
- *„ROM … nicht gefunden“* beim Start: Datei in den ROM-Ordner legen (`vcc roms where` nennt ihn). Die Meldung nennt Zielordner, erwartete Größe und alle gesuchten Orte.
- Zeichensatz sieht „falsch“ aus: vermutlich der eigene Standardsatz statt deines ROMs (Karte **Herkunft** im Zeichensatz-Editor). ROM ablegen.
- Unbekannte Revision: funktioniert, aber Symbole und Offsets können abweichen.

## Der Emulator beendet sich sofort
Beendet sich der Emulator kurz nach dem Start mit einem Fehler (zum Beispiel ein über die Paketverwaltung installiertes VICE ohne ROM-Dateien), schreibt VintageCoder die letzten Zeilen seiner Ausgabe (etwa „Couldn't load kernal ROM“) in die **Ausgabe** (Reiter in der Ablage unter dem Editor) und nennt den Hinweis: fehlende ROM-Dateien, siehe diese Seite. Ein Emulator, den du selbst schließt, löst keine Meldung aus.
