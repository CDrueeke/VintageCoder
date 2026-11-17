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


## Bauen, Starten, Debuggen

| Tastenkürzel (Mac / Windows, Linux) | Wirkung | Menü |
|---|---|---|
| F7 | Projekt bauen | Projekt ▸ Projekt bauen |
| F5 | Startprogramm starten (baut zuerst) | Projekt ▸ Startprogramm starten |
| F6 | Startprogramm debuggen (baut, startet den Emulator, öffnet den Reiter „Debug“ in der Schublade) | Projekt ▸ Startprogramm debuggen |
| Alt+Umschalt+F7 | aktive Datei bauen | Projekt ▸ Aktive Datei ▸ Aktive Datei bauen |
| Alt+Umschalt+F5 | aktive Datei starten | Projekt ▸ Aktive Datei ▸ Aktive Datei starten |
| Alt+Umschalt+F6 | aktive Datei debuggen | Projekt ▸ Aktive Datei ▸ Aktive Datei debuggen |
| Alt+F8 | Tests ausführen | Projekt ▸ Tests ausführen |
| F8 / Umschalt+F8 | nächstes / voriges Problem | Projekt |
| Cmd+Umschalt+E | Exportieren … (Export-Dialog) | Datei ▸ Exportieren |

Das **Startprogramm** legst du in den Projekt-Einstellungen fest (★/☆ an der Datei). Die Kürzel mit Alt+Umschalt gelten für die Datei im aktiven Reiter. Bauen und Starten sind nur möglich, wenn ein Projekt offen ist und gerade kein Bau läuft. F6 startet den Debugger nur, wenn keine Debug-Sitzung läuft.

## Im Debugger (nur wenn das Programm angehalten ist)

| Tastenkürzel | Wirkung | Menü |
|---|---|---|
| Alt+F5 | Weiter bis zum nächsten Haltepunkt | Projekt ▸ Debuggen ▸ Weiter |
| Alt+F6 | Anhalten | Projekt ▸ Debuggen ▸ Anhalten |
| F11 | Schritt: ein Befehl, ein `JSR` springt hinein | Projekt ▸ Debuggen ▸ Schritt hinein |
| F10 | Über: ein Befehl, ein `JSR` läuft durch | Projekt ▸ Debuggen ▸ Schritt über |
| Alt+D, Alt+B, Alt+A, Alt+P, Alt+H | Menü Datei, Bearbeiten, Ansicht, Projekt, Hilfe öffnen (nur Windows und Linux: Menüleiste im Fenster; auf dem Mac gilt die Menüleiste des Systems, Option+Buchstabe bleibt Text; der Buchstabe ist im Menünamen unterstrichen; AltGr löst nichts aus) | Menüleiste |
| Umschalt+F11 | Raus: bis zum Rücksprung des Unterprogramms | Projekt ▸ Debuggen ▸ Schritt hinaus |
| Umschalt+F5 | Debuggen beenden | Projekt ▸ Debuggen ▸ Debuggen beenden |
| F9 | Haltepunkt umschalten (Zeile mit dem Cursor) | Bearbeiten ▸ Haltepunkt umschalten |
| Klick auf die Zeilennummer | Haltepunkt setzen oder löschen | – |

Bei atari800 lässt sich das Programm nicht per Knopf anhalten: Dort drückst du **F8 im Emulatorfenster**.

## Dateien und Projekte

| Tastenkürzel | Wirkung |
|---|---|
| Cmd+Alt+N / Strg+Alt+N | Neues Projekt … |
| Cmd+Umschalt+O / Strg+Umschalt+O | Projekt öffnen … (Projektdatei) |
| Cmd+O / Strg+O | Datei öffnen … |
| Cmd+N / Strg+N | Neuer unbenannter Reiter (auch ohne Projekt); Name beim Speichern |
| Cmd+Umschalt+T / Strg+Umschalt+T | Neue Notiz |
| Cmd+Alt+→ / Cmd+Alt+← (Strg+Alt+→ / ←) | Nächster / vorheriger Reiter |
| Cmd+S / Strg+S | Speichern (auch in den Grafik- und Song-Editoren) |
| Cmd+Alt+S / Strg+Alt+S | Alle speichern |
| Cmd+W / Strg+W | Datei schließen |
| Cmd+, / Strg+, | Einstellungen |
| Cmd+Q / Strg+Q | VintageCoder beenden |

## Bearbeiten und Suchen

| Tastenkürzel | Wirkung |
|---|---|
| Cmd+Z / Strg+Z | Rückgängig (Quelltext-, Grafik- und Song-Editoren) |
| Cmd+Umschalt+Z / Strg+Umschalt+Z | Wiederholen |
| Cmd+F / Strg+F | Suchen … |
| Cmd+Alt+F / Strg+Alt+F | Ersetzen … |
| Cmd+L / Strg+L | Gehe zu Zeile … |
| Cmd+Umschalt+F / Strg+Umschalt+F | In Projekt suchen … (öffnet in der linken Spalte „Suche“) |
| F1 | Hilfe zum Wort unter dem Cursor |
| F12 | Gehe zu Definition |
| Cmd+I / Strg+I | Markdown: Kursiv |
| Cmd+E / Strg+E | Markdown: Code |
| Cmd+Umschalt+X / Strg+Umschalt+X | Markdown: Durchgestrichen |
| Cmd+Umschalt+K / Strg+Umschalt+K | Markdown: Link einfügen |

Vorschläge beim Schreiben erscheinen von selbst (nach dem Tippen, nach „.“ und nach „@“); eine Taste zum Öffnen gibt es nicht. Zeichen und Steuerzeichen fügst du über das Zeichen-Panel (Ω) im rechten Panel ein. **Fett** hat kein Kürzel (Befehl „Markdown: Fett“ in der Befehlspalette).

In den Eingabefeldern der Suche und von „Gehe zu Zeile“ führt **Enter** die Suche aus (in der Suche mit **Umschalt+Enter** zum vorherigen Treffer), **Esc** schließt das Feld.

## Ansicht und Oberfläche

| Tastenkürzel | Wirkung |
|---|---|
| Cmd+K / Strg+K | Befehl oder Datei suchen … (Befehlspalette) |
| Cmd+B / Strg+B | linke Spalte ein- und ausklappen |
| Cmd+Umschalt+B / Strg+Umschalt+B | Zen-Modus: nur Fläche (linke Spalte, Schublade, Panel, Liste und Seitenspalte der Studios ausblenden); noch einmal stellt das Layout wieder her |
| Menü-Taste / Umschalt+F10 | Kontextmenü mit den Aktionen des Studios im aktiven Reiter |
| Cmd+Alt+B / Strg+Alt+B | rechtes Panel ein- und ausblenden |
| Cmd+J / Strg+J | Schublade ein- und ausklappen |
| Cmd+Umschalt+Enter / Strg+Umschalt+Enter | Bereich mit dem Fokus maximieren und wiederherstellen |
| Cmd+Alt+T / Strg+Alt+T | Editor teilen bzw. Teilung aufheben |
| Cmd+Plus / Cmd+Minus / Cmd+0 (Strg+…) | Oberfläche vergrößern, verkleinern, Originalgröße (Menü Ansicht) |
| Cmd+Komma / Strg+Komma | Einstellungen (Menü Ansicht) |

Im Menü **Ansicht** gibt es ohne Tastenkürzel noch **Hell/Dunkel umschalten**, **Hoher Kontrast**, **Layout zurücksetzen**, **Editor-Schrift größer/kleiner**, **Darstellung zurücksetzen**, **Zeilenumbruch im Editor** und die Befehle zum Maximieren einzelner Bereiche. Das Farbschema (Dunkel, Hell, System) und die Einzeltasten der Studios stellst du in den Einstellungen ein.

## Hilfe

| Tastenkürzel | Wirkung |
|---|---|
| F1 | Hilfe öffnen; steht der Cursor auf einem Wort im Editor, zeigt sie dazu den Eintrag |

Das Menü **Hilfe** enthält außerdem **Hilfe im Reiter öffnen**, den **Konfigurationsassistenten** (ganz, oder nur ROMs, nur Emulatoren, nur echte Hardware, nur Design und Editor, Nach Neuem suchen), **Feedback senden …**, **Nach Updates suchen …** und **Über VintageCoder**. Konfiguration exportieren, importieren und zurücksetzen findest du unter Einstellungen ▸ Allgemein.

## Einzeltasten in den Studios

In den Studios gibt es Einzeltasten, sobald das Studio den Fokus hat und kein Textfeld aktiv ist. Du kannst sie unter **Einstellungen ▸ Editor ▸ Einzeltasten der Studios** abschalten (Standard: an). Pfeile, Leertaste, Esc und Entf gelten auch dann.

| Studio | Tasten |
|---|---|
| Sprites und Zeichensätze | **D** Stift, **E** Radierer, **F** Füllen, **L** Linie, **R** Rahmen, **S** Auswahl, **I** Pipette, **X** / **Y** horizontal / vertikal spiegeln, **1** bis **4** Pixelwert (Farbe), **C** Hires ↔ Multicolor (nur Sprites mit beiden Modi), **←** / **→** vorheriger / nächster Frame, **Entf** oder **Rücktaste** leert die Auswahl, **Esc** hebt sie auf |
| Bildschirm | **D** Zeichen, **T** Text, **F** Füllen, **R** Rahmen, **I** Pipette, **E** Löschen; Pfeile bewegen den Cursor. Im Textwerkzeug tippst du Zeichen ein, **Esc** beendet es. |
| Bank-Studio | wie im Bildschirm-Editor, dazu **L** Linie, **A** Auswahl, **S** Stempel; die Leertaste setzt; **Tab** wechselt zwischen Kachel- und Zellenmodus |
| Alle Studios mit Fläche (auch Kacheln) | **+** und **−** Zoom, **0** Einpassen (auch Cmd/Strg+Mausrad) |

Mit gedrückter Cmd-, Strg- oder Alt-Taste wirken die Einzeltasten nicht. In den Studios gelten außerdem Cmd/Strg+C, +X, +V und +A. **Menü-Taste** oder **Umschalt+F10** öffnet das Kontextmenü mit den Aktionen des Studios.

## Im Tracker

| Taste | Wirkung |
|---|---|
| Pfeiltasten | Cursor im Muster bewegen (mit Umschalt: Auswahl) |
| Bild auf / Bild ab | 8 Zeilen hoch / runter |
| Pos1 / Ende | zur ersten / letzten Zeile |
| Leertaste | Wiedergabe starten und anhalten |
| Komma / Punkt | Oktave tiefer / höher |
| Umschalt+Leertaste | nur das gewählte Muster abspielen |
| Cmd+1 / Cmd+2 / Cmd+3 (Strg+…) | Modus Noten / Instrumente / Ablauf |
| [ / ] | vorheriges / nächstes Instrument |
| Z S X D C V G B H N J M | Klaviertasten, untere Oktave (Z = C, S = Cis, X = D …); nach Tastenposition, auf QWERTZ steht an der Stelle des Z ein Y |
| Q 2 W 3 E R 5 T 6 Y 7 U I | Klaviertasten, nächste Oktave |
| Entf oder Rücktaste | Zelle leeren |
| Alt+↑ / Alt+↓ | Instrument der Zelle ändern |
| 1 | Noten-Aus-Zeichen („=== --“) eintragen |
| Cmd+S / Strg+S, Cmd+Z / Strg+Z | Speichern, Rückgängig (mit Umschalt: Wiederholen) |
| Cmd+C / X / V / A (Strg+…) | Kopieren, Ausschneiden, Einfügen, Alles wählen |

Die Klaviertasten tragen eine Note ein und rücken eine Zeile weiter. Im Modus Instrumente wählen ↑ und ↓ das Instrument; Klaviertasten und Leertaste spielen es nur vor. Details dazu im Kapitel [Der Tracker](kapitel:tracker).

> **Info:** Das Nachschlagewerk [Tastenkürzel](hilfe:shortcuts) hat eine kürzere Tabelle. Maßgeblich sind die Kürzel, die im Menü neben den Befehlen stehen und unter Einstellungen ▸ **Tastenkürzel** aufgelistet sind.

## Häufige Fragen

### Installation und erster Start

**Wie richte ich VintageCoder beim ersten Mal ein?** Beim allerersten Start öffnet sich der **Konfigurationsassistent**. Wähle die **Schnelleinrichtung** (7 Schritte, etwa 2 Minuten). Du kannst ihn jederzeit über **Hilfe ▸ Konfigurationsassistent** oder den Knopf auf der Willkommensseite wieder aufrufen. Mehr dazu in [Installation und Ersteinrichtung](kapitel:installation).

**Wie werde ich das Startbild los?** Ein Klick oder eine Taste schließt es. Dauerhaft schaltest du es unter **Einstellungen ▸ Editor ▸ Startbild** auf **Aus**.

**Wo bekomme ich die Emulatoren her?** VintageCoder bringt keine Emulatoren mit. Installiere VICE (C64, C128, Plus/4, VC-20), atari800 oder Altirra (Atari) und Xemu (MEGA65) selbst und trage ihren Pfad unter **Einstellungen ▸ Zielsysteme und Hardware** in der Karte **Konfigurierte Emulatoren** ein. Bleibt der Pfad leer, sucht die IDE selbst. Wo es für dein System ein festes Paket gibt, bietet die Karte **Portabel installieren …** an; geladen wird erst nach deiner Bestätigung. Mehr dazu im Kapitel [Installation und Ersteinrichtung](kapitel:installation), für Linux im Abschnitt [Emulatoren unter Linux](kapitel:systeme).

**Wo bekomme ich ROMs her?** VintageCoder liefert keine Original-ROMs mit und lädt keine aus dem Netz. Wenn du ROMs brauchst, legst du deine eigenen Abbilder in den ROM-Ordner: **Einstellungen ▸ ROMs** (Ablagezone), oder **Hilfe ▸ Konfigurationsassistent: nur ROMs …**. VICE und atari800 laufen ohne ROM mit eigenen Ersatz-ROMs; **Xemu braucht eine MEGA65-ROM-Datei von 131072 Byte**. Die Datei wird am Inhalt erkannt, der Name ist egal. Ob du ein Abbild benutzen darfst, klärst du selbst.

### Bauen und Starten

**F5 oder F6 tut nichts.** Das Programm wurde nicht gebaut, weil es Fehler gibt: Schau in der Schublade in den Reiter **Probleme**; ein Klick auf eine Meldung springt zur Zeile. Läuft schon ein Bau, ist F5 gesperrt; läuft schon eine Debug-Sitzung, startet F6 nicht (erst beenden: **■** in der Kopfzeile oder Umschalt+F5).

**Der Emulator startet nicht.** Prüfe den Pfad unter **Einstellungen ▸ Zielsysteme und Hardware**; der Chip der Emulator-Karte zeigt **Gefunden**, **Automatisch erkannt** oder **Fehlt**, und **▶ Test-Start** probiert den Emulator aus. Es läuft immer nur **ein** Emulator zugleich.

**Mein Programm startet, aber es passiert nichts.** Baue neu (F7) und prüfe das Ziel im Chip in der Kopfzeile (zum Beispiel „C64 · Emulator (VICE x64sc) ▾“). Prüfe, dass die Startadresse stimmt: Beim C64 braucht ein Programm `.basicstub` bei `.org $0801`, beim Atari `.run start`.

**Das Programm läuft auf dem falschen Gerät.** Der Chip in der Kopfzeile (System und Ziel, zum Beispiel „C64 · Emulator (…) ▾“) steht auf einem Gerät, das du nicht wolltest. Ein Klick öffnet das Menü mit dem Emulator und den eingetragenen Geräten. Das System selbst bestimmt das Projekt (oder `.target` im Quelltext).

**Was bedeutet die Warnung W104?** Der Assembler meldet, dass ein Befehl nicht zum offiziellen Befehlssatz der CPU gehört (ein undokumentierter Opcode wie `lax`). Das Beispiel `examples/c64/border` tut das absichtlich. Mit `illegals = deny` im `.cpu`-Befehl wird daraus ein Fehler, mit `illegals = warn` bleibt es eine Warnung.

### Debugger und Tests

**Haltepunkte werden ignoriert.** Baue neu; die Zuordnung von Zeile zu Adresse kommt aus dem letzten Bau. Bei atari800 und MEGA65 gilt außerdem: nur ein Haltepunkt zugleich.

**Register und Speicher sind leer.** Das Programm läuft. Drücke **⏸** (Anhalten, Alt+F6) oder halte an einem Haltepunkt an.

**Ich kann die Flags und Chip-Register nicht ändern.** Sie sind reine Anzeige, damit Lesezugriffe nichts verfälschen. Ändere das Programm im Quelltext und starte neu.

**Meine Tests werden alle übersprungen.** Das Feld **Läuft auf:** in der Schublade „Tests“ steht auf „nur statisch“, oder die Tests brauchen den Emulator (VICE). Siehe [Automatisch testen](kapitel:tests).

### Grafik und Klang

**Das Zeichensatz-Studio zeigt den eigenen Satz von VintageCoder.** Es fehlt das System-ROM mit der Schrift; sie erscheint ersatzweise als eingebaute Schrift ohne Grafikzeichen. Lege das ROM wie oben beschrieben ab oder importiere die Schrift über **Datei ▸ Zeichensatz aus ROM importieren …**.

**Für den Plus/4 gibt es keinen Sprite-Editor.** Der TED hat keine Sprites, und für Plus/4 gibt es noch keine Profile für Sprites, Zeichensätze oder Songs.

**Das Panel „Bild“ zeigt nichts.** Das Emulator-Bild erscheint im rechten Panel, nur in einer Debug-Sitzung. Nur VICE liefert ein Bild (C64, C128, Plus/4, VC-20). Atari und MEGA65 liefern keines.

### Dateien

**Wo liegen meine Grafiken und Songs?** Als Textdateien im Projekt, zum Beispiel `font.vcchar` oder `hero.vcsprite`. Beim Bauen werden sie nach `build/export/` exportiert; einbinden mit `.include "../build/export/hero.asm"`.

**Welche Dateien kann ich bedenkenlos löschen?** Den Ordner `build/` baust du jederzeit mit F7 neu.

### Markdown

**Warum zeigt die Vorschau mein HTML nicht?** Rohes HTML wird nie ausgeführt; es steht in der Vorschau als Text. Schreibe es in Markdown um.

**Warum lädt die Vorschau mein Bild aus dem Internet nicht?** Webbilder werden nicht geladen (Datenschutz). Lege eine PNG-Datei ins Projekt und verweise relativ darauf: `![Alt](pics/bild.png)`.

**Warum ist ein Link rot?** Die Zieldatei fehlt oder der Pfad führt aus dem Projekt hinaus. Pfade gelten relativ zur Markdown-Datei.

**Cmd+B und Cmd+K machen etwas anderes als in anderen Editoren.** Sie klappen die linke Spalte ein/aus und öffnen die Befehlspalette. Fett hat deshalb kein Kürzel (Cmd+Umschalt+B ist der Zen-Modus); der Link liegt auf Cmd+Umschalt+K.

## Fehlersuche: die ersten drei Schritte

1. **Schau in den Reiter Probleme** (Schublade). Dort stehen Fehler und Warnungen mit Datei und Zeile; ein Klick springt hin.
2. **Schau in den Reiter Protokoll** (Schublade). Dort steht der Ablauf des Baus; schlägt ein Bau ohne Meldung zu einer Zeile fehl, steht dort der Grund. Siehe [Bauen und Starten](kapitel:bauen-starten).
3. **Prüfe die Einrichtung:** Emulator (Chip in **Zielsysteme und Hardware**) und ROM (**Einstellungen ▸ ROMs**).

Hilft das nicht, grenze das Problem ein: Baue das kleinste Beispiel des Zielsystems (`examples/c64/border`, `examples/atari/colorbars`, `examples/mega65/hello`). Baut es, liegt das Problem in deinem Projekt; baut es nicht, liegt es an der Einrichtung (ROM, Emulator, Pfad).

## Wie geht es weiter?

- Fachwörter nachschlagen: Glossar (`GLOSSAR.md`).
- Die Oberfläche kennenlernen: [Die Oberfläche im Überblick](kapitel:oberflaeche).
- Einen Fehler im laufenden Programm finden: [Der Debugger](kapitel:debugger).
- Zum Nachschlagen: [Tastenkürzel](hilfe:shortcuts), [ROM-Dateien](hilfe:roms), [Einstellungen](hilfe:settings).

## Linux

Für Linux gibt es Pakete für Debian, Ubuntu, Fedora und openSUSE sowie ein Archiv (siehe [Installation und Ersteinrichtung](kapitel:installation)); X11 und Wayland werden unterstützt. Alle Tastenkürzel gelten unverändert mit **Strg** statt **Cmd**. Wie du die Emulatoren unter Linux installierst, steht im Abschnitt [Emulatoren unter Linux](kapitel:systeme) im Kapitel „Zielsysteme im Vergleich“.
