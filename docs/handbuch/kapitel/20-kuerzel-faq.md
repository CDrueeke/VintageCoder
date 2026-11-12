---
id: kuerzel-faq
titel: Tastenkürzel, FAQ, Fehlersuche
reihenfolge: 20
stufe: Einsteiger
zeit: 15 Minuten
siehe_auch: [oberflaeche, editor, debugger, bauen-starten, systeme]
referenz: [shortcuts, roms, settings]
---

# Tastenkürzel, FAQ, Fehlersuche

**Ziel:** Hier findest du alle Tastenkürzel von VintageCoder auf einen Blick und Antworten auf die Fragen, die am häufigsten auftauchen.

Alle Kürzel gelten auf dem Mac **und** unter Windows und Linux. Die Taste **Cmd** (⌘) auf dem Mac heißt unter Windows und Linux **Strg**; **Umschalt** ist ⇧, **Alt** ist ⌥. Wo nichts anderes steht, steht in der Tabelle zuerst die Mac-Form, dann die andere.

![Übersichtskarte der Tastenkürzel](bild:kuerzel-karte "Die wichtigsten Tastenkürzel von VintageCoder auf einer Karte: Bauen und Starten, Debugger, Dateien, Suchen, Ansicht.")

## Bauen, Starten, Debuggen

| Tastenkürzel (Mac / Windows, Linux) | Wirkung | Menü |
|---|---|---|
| F7 | Projekt bauen | Projekt ▸ Projekt bauen |
| Umschalt+F7 | aktive Datei bauen | Projekt ▸ Aktive Datei bauen |
| F5 | bauen und Startprogramm starten | Projekt ▸ Startprogramm starten |
| Umschalt+F5 | aktive Datei starten | Projekt ▸ Aktive Datei starten |
| F6 | Startprogramm debuggen (baut, startet den Emulator, wechselt auf den Arbeitsplatz Debug) | Projekt ▸ Startprogramm debuggen |
| Umschalt+F6 | aktive Datei debuggen | Projekt ▸ Aktive Datei debuggen |

Bauen und Starten sind nur möglich, wenn ein Projekt offen ist und gerade kein Bau läuft. F6 startet den Debugger nur, wenn keine Debug-Sitzung läuft.

## Im Debugger (nur wenn das Programm angehalten ist)

| Tastenkürzel | Wirkung | Menü |
|---|---|---|
| F8 | Weiter bis zum nächsten Haltepunkt | Debug ▸ Weiter |
| F11 | Schritt: ein Befehl, ein `JSR` springt hinein | Debug ▸ Schritt |
| F10 | Über: ein Befehl, ein `JSR` läuft durch | Debug ▸ Über |
| Umschalt+F11 | Raus: bis zum Rücksprung des Unterprogramms | Debug ▸ Raus (bis Rücksprung) |
| Klick auf die Zeilennummer | Haltepunkt setzen oder löschen | – |

Ohne Tastenkürzel, aber im Menü **Debug**: **Pause**, **Debug-Sitzung beenden**, **Speicher anzeigen** und **Emulator-Bild anzeigen**.

Bei atari800 lässt sich das Programm nicht per Knopf anhalten: Dort drückst du **F8 im Emulatorfenster**.

## Dateien und Projekte

| Tastenkürzel | Wirkung |
|---|---|
| Cmd+Umschalt+N / Strg+Umschalt+N | Neues Projekt … |
| Cmd+O / Strg+O | Projekt öffnen … |
| Cmd+N / Strg+N | Neue Quelldatei … (nur mit offenem Projekt) |
| Cmd+S / Strg+S | Speichern (auch in den Grafik- und Song-Editoren) |
| Cmd+Alt+S / Strg+Alt+S | Alle speichern |
| Cmd+W / Strg+W | Datei schließen |
| Cmd+, / Strg+, | Einstellungen |

## Bearbeiten und Suchen

| Tastenkürzel | Wirkung |
|---|---|
| Cmd+Z / Strg+Z | Rückgängig (Quelltext-, Grafik- und Song-Editoren) |
| Cmd+Umschalt+Z / Strg+Umschalt+Z | Wiederholen |
| Cmd+F / Strg+F | Suchen … |
| Cmd+Alt+F / Strg+Alt+F | Ersetzen … |
| Cmd+L / Strg+L | Gehe zu Zeile … |
| Cmd+Umschalt+F / Strg+Umschalt+F | In Projekt suchen … (öffnet die Seitenleiste „Suche“) |
| Cmd+Umschalt+Y / Strg+Umschalt+Y | Zeichen einfügen … (PETSCII/ATASCII, öffnet den Zeichenwähler; nur mit offenem Projekt) |
| Strg+Leertaste (auch auf dem Mac die Control-Taste) | Vervollständigung öffnen; Pfeile wählen, Enter oder Tab übernimmt, Esc schließt |
| F1 | Hilfe zum Wort unter dem Cursor |

In den Eingabefeldern der Suche und von „Gehe zu Zeile“ führt **Enter** die Suche aus (in der Suche mit **Umschalt+Enter** zum vorherigen Treffer), **Esc** schließt das Feld.

## Ansicht und Oberfläche

| Tastenkürzel | Wirkung |
|---|---|
| Cmd+K / Strg+K | Befehle, Dateien und Studios suchen (Befehlspalette). Auch Cmd/Strg+P. |
| Cmd+B / Strg+B | Seitenleiste ein- und ausblenden (im schmalen Fenster: Schublade) |
| Cmd+Alt+B / Strg+Alt+B | Inspektor ein- und ausblenden |
| Cmd+J / Strg+J | Fußpanel ein- und ausblenden |
| Cmd+Umschalt+Enter / Strg+Umschalt+Enter | Seitenleiste, Inspektor und Fußpanel zeitweise ausblenden (Maximieren) und zurückholen |
| Cmd+\ / Strg+\ | Editor teilen bzw. Teilung aufheben |
| Cmd++ / Cmd+- / Cmd+0 (Strg+…) | Oberfläche vergrößern, verkleinern, Originalgröße (Menü Ansicht) |
| Einfachklick / Doppelklick / Cmd+Klick im Projektbaum | Vorschau / fester Reiter / zweiter Editorbereich (Strg+Klick unter Windows und Linux) |

Im Menü **Ansicht** gibt es ohne Tastenkürzel noch **Arbeitsplatz Code** und **Arbeitsplatz Debug**, **Layout zurücksetzen**, **Farbschema** (Dunkel, Hell, Wie das System), **Editor-Schrift größer/kleiner**, **Darstellung zurücksetzen**, **Adressen im Editor** und **Einzeltasten im Grafik-Editor**.

## Hilfe

| Tastenkürzel | Wirkung |
|---|---|
| F1 | Hilfe öffnen; steht der Cursor auf einem Wort im Editor, zeigt sie dazu den Eintrag |

Das Menü **Hilfe** enthält außerdem **Tastenkürzel**, das Untermenü **Konfigurationsassistent** (alles einrichten, nur ROMs, nur Emulatoren, nur echte Hardware, nur Design und Editor, Konfiguration exportieren, importieren, auf Standard zurücksetzen), **ROMs einrichten …**, **ROM-Ordner öffnen**, **ROMs prüfen** und **Über VintageCoder**.

## Einzeltasten in den Grafik-Editoren

In den Editoren für Sprites, Zeichensätze und Bildschirme gibt es Einzeltasten, sobald der Mauszeiger über dem Editor steht und kein Textfeld aktiv ist. Du kannst sie im Menü **Ansicht ▸ Einzeltasten im Grafik-Editor** abschalten (Standard: an).

| Taste | Wirkung |
|---|---|
| D | Stift |
| E | Radierer |
| F | Füllen |
| L | Linie |
| R | Rahmen |
| S | Auswahl |
| I | Pipette |
| X / Y | horizontal / vertikal spiegeln |
| C | zwischen Hires und Multicolor umschalten (wenn das Bild beides kennt) |
| 1 bis 4 | Pixelwert (Farbe) wählen |
| ← / → | vorheriger / nächster Frame |

Mit gedrückter Cmd-, Strg- oder Alt-Taste wirken die Einzeltasten nicht. Der Bildschirm-Editor wechselt außerdem mit **Tab** zwischen Kachel- und Zellenmodus (wenn kein Textfeld aktiv ist).

## Im Tracker

| Taste | Wirkung |
|---|---|
| Pfeiltasten | Cursor im Muster bewegen |
| Bild auf / Bild ab | 8 Zeilen hoch / runter |
| Pos1 / Ende | zur ersten / letzten Zeile |
| Leertaste | Wiedergabe starten und anhalten |
| F9 / F10 | Oktave tiefer / höher |
| [ / ] | vorheriges / nächstes Instrument |
| Z S X D C V G B H N J M | Klaviertasten, untere Oktave (Z = C, S = Cis, X = D …) |
| Q 2 W 3 E R 5 T 6 Y 7 U I | Klaviertasten, nächste Oktave |
| Entf, Rücktaste oder . | Zelle leeren (der Cursor rückt eine Zeile weiter) |
| 1 | Noten-Aus-Zeichen („=== --“) eintragen |
| Cmd+S / Strg+S, Cmd+Z / Strg+Z | Speichern, Rückgängig (mit Umschalt: Wiederholen) |

Die Klaviertasten tragen eine Note ein und rücken eine Zeile weiter. Details dazu im Kapitel [Der Tracker](kapitel:tracker).

> **Info:** Das Nachschlagewerk [Tastenkürzel](hilfe:shortcuts) hat eine kürzere Tabelle. Maßgeblich sind die Kürzel, die im Menü neben den Befehlen stehen (auf dem Mac in der Menüleiste am Bildschirmrand, sonst in der Titelzeile).

## Häufige Fragen

### Installation und erster Start

**Wie richte ich VintageCoder beim ersten Mal ein?** Beim allerersten Start öffnet sich der **Konfigurationsassistent**. Wähle die **Schnelleinrichtung** (7 Schritte, etwa 2 Minuten). Du kannst ihn jederzeit über **Hilfe ▸ Konfigurationsassistent** oder den Knopf auf der Willkommensseite wieder aufrufen. Mehr dazu in [Installation und Ersteinrichtung](kapitel:installation).

**Wie werde ich den Startbildschirm los?** Hake dort **Beim nächsten Start nicht mehr anzeigen** an. Das gilt nur für die installierte Version; nach einem Versionswechsel erscheint er einmal wieder.

**Wo bekomme ich die Emulatoren her?** VintageCoder bringt keine Emulatoren mit. Installiere VICE (C64, C128, Plus/4, VC-20), atari800 oder Altirra (Atari) und Xemu (MEGA65) selbst und trage ihren Pfad unter **Einstellungen ▸ Emulatoren** ein. Bleibt der Pfad leer, sucht die IDE selbst. Mehr dazu im Kapitel [Willkommen](kapitel:willkommen).

**Wo bekomme ich ROMs her?** VintageCoder liefert keine Original-ROMs mit und lädt nichts aus dem Netz. Wenn du ROMs brauchst, legst du deine eigenen Abbilder in den ROM-Ordner: **Einstellungen ▸ ROMs** (Ablagezone), oder **Hilfe ▸ ROMs einrichten …**. VICE und atari800 laufen ohne ROM mit eigenen Ersatz-ROMs; **Xemu braucht eine MEGA65-ROM-Datei von 131072 Byte**. Die Datei wird am Inhalt erkannt, der Name ist egal. Ob du ein Abbild benutzen darfst, klärst du selbst.

### Bauen und Starten

**F5 oder F6 tut nichts.** Das Programm wurde nicht gebaut, weil es Fehler gibt: Schau im Fußpanel in den Reiter **Probleme**; ein Klick auf eine Meldung springt zur Zeile. Läuft schon ein Bau, ist F5 gesperrt; läuft schon eine Debug-Sitzung, startet F6 nicht (erst **Stop**).

**Der Emulator startet nicht.** Prüfe den Pfad unter **Einstellungen ▸ Emulatoren**. Die Statusleiste unten nennt den Emulator des Zielsystems und zeigt, ob sein Programm gefunden wurde. Es läuft immer nur **ein** Emulator zugleich.

**Mein Programm startet, aber es passiert nichts.** Baue neu (F7) und prüfe das Zielsystem im Feld **Auf:**. Prüfe, dass die Startadresse stimmt: Beim C64 braucht ein Programm `.basicstub` bei `.org $0801`, beim Atari `.run start`.

**Das Programm läuft auf dem falschen System.** Das Feld **Auf:** in der Befehlsleiste steht auf einer Maschine, die du nicht wolltest. Mit „automatisch“ nimmt die IDE `.target` aus dem Quelltext.

**Was bedeutet die Warnung W104?** Der Assembler meldet, dass ein Befehl nicht zum offiziellen Befehlssatz der CPU gehört (ein undokumentierter Opcode wie `lax`). Das Beispiel `examples/c64/border` tut das absichtlich. Mit `illegals = deny` im `.cpu`-Befehl wird daraus ein Fehler, mit `illegals = warn` bleibt es eine Warnung.

### Debugger und Tests

**Haltepunkte werden ignoriert.** Baue neu; die Zuordnung von Zeile zu Adresse kommt aus dem letzten Bau. Bei atari800 und MEGA65 gilt außerdem: nur ein Haltepunkt zugleich.

**Register und Speicher sind leer.** Der Emulator läuft. Drücke **Pause** oder halte an einem Haltepunkt an.

**Ich kann die Flags und Chip-Register nicht ändern.** Sie sind reine Anzeige, damit Lesezugriffe nichts verfälschen. Ändere das Programm im Quelltext und starte neu.

**Meine Tests werden alle übersprungen.** Das Feld **Läuft auf:** im Fußpanel „Tests“ steht auf „nur statisch“, oder die Tests brauchen den Emulator (VICE). Siehe [Automatisch testen](kapitel:tests).

### Grafik und Klang

**Das Zeichensatz-Studio zeigt eine Ersatzschrift.** Es fehlt das System-ROM mit der Schrift; sie erscheint ersatzweise als eingebaute Schrift ohne Grafikzeichen. Lege das ROM wie oben beschrieben ab oder importiere die Schrift über **Datei ▸ Zeichensatz aus ROM importieren …**.

**Für den Plus/4 gibt es keinen Sprite-Editor.** Der TED hat keine Sprites, und für Plus/4 gibt es noch keine Profile für Sprites, Zeichensätze oder Songs.

**Der Emulator-Reiter zeigt kein Bild.** Nur VICE liefert ein Bild (C64, C128, Plus/4, VC-20). Atari und MEGA65 liefern keines.

### Dateien

**Wo liegen meine Grafiken und Songs?** Als Textdateien im Projekt, zum Beispiel `font.vcchar` oder `hero.vcsprite`. Beim Bauen werden sie nach `build/export/` exportiert; einbinden mit `.include "../build/export/hero.asm"`.

**Welche Dateien kann ich bedenkenlos löschen?** Den Ordner `build/` baust du jederzeit mit F7 neu.

## Fehlersuche: die ersten drei Schritte

1. **Schau in den Reiter Probleme** (Fußpanel). Dort stehen Fehler und Warnungen mit Datei und Zeile; ein Klick springt hin.
2. **Schau ins Bauprotokoll.** Es zeigt jede Stufe des Baus mit Dauer und Meldungen; die fehlgeschlagene Stufe ist markiert. Siehe [Bauen und Starten](kapitel:bauen-starten).
3. **Schau in den Reiter Ausgabe.** Schlägt ein Bau ohne Meldung zu einer Zeile fehl, steht dort der Grund.

Hilft das nicht, grenze das Problem ein: Baue das kleinste Beispiel des Zielsystems (`examples/c64/border`, `examples/atari/colorbars`, `examples/mega65/hello`). Baut es, liegt das Problem in deinem Projekt; baut es nicht, liegt es an der Einrichtung (ROM, Emulator, Pfad).

## Wie geht es weiter?

- Fachwörter nachschlagen: Glossar (`GLOSSAR.md`).
- Die Oberfläche kennenlernen: [Die Oberfläche im Überblick](kapitel:oberflaeche).
- Einen Fehler im laufenden Programm finden: [Der Debugger](kapitel:debugger).
- Zum Nachschlagen: [Tastenkürzel](hilfe:shortcuts), [ROM-Dateien](hilfe:roms), [Einstellungen](hilfe:settings).

## Linux

Linux ist ein Entwicklungsziel (Debian, Ubuntu, Fedora, openSUSE; X11 und Wayland). Es gibt noch keine fertigen Pakete (kein AppImage, kein .deb): VintageCoder wird aus dem Quellbaum gebaut. Das Skript `tools/linux-vm-test.sh` richtet eine frische Maschine ein, baut und prüft den Start. Alle Tastenkürzel gelten unverändert mit **Strg** statt **Cmd**.
