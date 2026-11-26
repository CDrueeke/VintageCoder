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

Alle Kürzel gelten auf dem Mac **und** unter Windows und Linux. Die Taste **Cmd** (⌘) auf dem Mac heißt unter Windows und Linux **Strg**; **Umschalt** ist ⇧, **Alt** ist ⌥. In den Tabellen steht deshalb **Cmd/Strg+S**: Auf dem Mac nimmst du Cmd, sonst Strg. Hinter jedem Befehl in den Menüs und in der Befehlspalette (**Cmd/Strg+K**) steht sein Kürzel.

> **Info:** Einige Befehle gibt es in dieser Version noch nicht; Menü und Befehlspalette zeigen sie nicht: Zoom, Raster und Onion Skin der Editoren per Menü (die Knöpfe und Tasten im Editor gibt es), BASIC in Assembler umwandeln, Kodierung umschalten, BASIC: Hex-Vorschläge und die Importe von Zeichensätzen (Standardsatz, ROM, PNG-Raster). Die Tasten der Editoren (Tabellen unten) sind davon nicht betroffen.


## Bauen, Starten, Debuggen

| Tastenkürzel | Wirkung | Menü |
|---|---|---|
| F7 | Projekt bauen | Projekt ▸ Projekt bauen |
| F5 | Startprogramm starten (baut zuerst) | Projekt ▸ Startprogramm starten |
| F6 | Startprogramm debuggen (baut, startet den Emulator mit einer Debug-Sitzung, öffnet den Bereich „Debug“) | Projekt ▸ Startprogramm debuggen |
| Alt+Umschalt+F7 | aktive Datei bauen | Projekt ▸ Aktive Datei ▸ Aktive Datei bauen |
| Alt+Umschalt+F5 | aktive Datei starten | Projekt ▸ Aktive Datei ▸ Aktive Datei starten |
| Alt+Umschalt+F6 | aktive Datei debuggen | Projekt ▸ Aktive Datei ▸ Aktive Datei debuggen |
| Alt+F8 | Tests ausführen | Projekt ▸ Tests ausführen |
| F8 / Umschalt+F8 | nächstes / voriges Problem | Projekt ▸ Nächstes Problem, Voriges Problem |
| Cmd/Strg+Umschalt+E | Exportieren … (Export-Dialog) | Datei ▸ Exportieren … |

Das **Startprogramm** legst du im Hauptbereich **Projekt** unter **Quellen** fest (Stern ★ an der Datei). Die Kürzel mit Alt+Umschalt gelten für die Datei im aktiven Reiter. Bauen und Starten sind nur möglich, wenn ein Projekt offen ist und gerade kein Bau läuft. Läuft schon eine Debug-Sitzung, bringt dich F6 nur in den Bereich **Debug** zurück.

## Im Debugger (nur wenn das Programm angehalten ist)

| Tastenkürzel | Wirkung | Menü |
|---|---|---|
| Alt+F5 | Weiter bis zum nächsten Haltepunkt | Projekt ▸ Debuggen ▸ Weiter |
| Alt+F6 | Anhalten | Projekt ▸ Debuggen ▸ Anhalten |
| F11 | Schritt: ein Befehl, ein `JSR` springt hinein | Projekt ▸ Debuggen ▸ Schritt hinein |
| F10 | Über: ein Befehl, ein `JSR` läuft durch | Projekt ▸ Debuggen ▸ Schritt über |
| Umschalt+F11 | Raus: bis zum Rücksprung des Unterprogramms | Projekt ▸ Debuggen ▸ Schritt hinaus |
| Umschalt+F5 | Debuggen beenden | Projekt ▸ Debuggen ▸ Debuggen beenden |
| – | Bis hierher ausführen (bis zur Zeile mit dem Cursor) | Projekt ▸ Debuggen ▸ Bis hierher ausführen |
| F9 | Haltepunkt umschalten (Zeile mit dem Cursor) | Bearbeiten ▸ Haltepunkt umschalten |
| Klick auf die Zeilennummer | Haltepunkt setzen oder löschen | – |

Bei atari800 lässt sich das Programm nicht per Knopf anhalten: Dort drückst du **F8 im Emulatorfenster**.

Auf dem Mac gilt die Menüleiste des Systems, unter Windows und Linux die Menüleiste im Fenster. Alles, was im Menü steht, steht auch in der Befehlspalette (**Cmd/Strg+K**).

## Dateien und Projekte

| Tastenkürzel | Wirkung |
|---|---|
| Cmd/Strg+Alt+N | Neues Projekt … |
| Cmd/Strg+Umschalt+O | Projekt öffnen … (Projektdatei) |
| Cmd/Strg+Umschalt+W | Projekt schließen |
| Cmd/Strg+Alt+0 | Bereich Projekt (Einstellungen und Überblick des Projekts) |
| Cmd/Strg+O | Datei öffnen … |
| Cmd/Strg+N | Neuer unbenannter Reiter (auch ohne Projekt); Name beim Speichern |
| Cmd/Strg+Umschalt+T | Neue Notiz |
| Cmd/Strg+Alt+→ / Cmd/Strg+Alt+← | Nächster / vorheriger Reiter |
| Cmd/Strg+S | Speichern (auch im Sprite-, Zeichensatz- und Paletten-Editor) |
| Cmd/Strg+Alt+S | Alle speichern |
| Cmd/Strg+W | Datei schließen |
| Cmd/Strg+Komma | Einstellungen |
| Cmd/Strg+Q | VintageCoder beenden |

## Bearbeiten und Suchen

| Tastenkürzel | Wirkung |
|---|---|
| Cmd/Strg+Z | Rückgängig (Quelltext-, Sprite-, Zeichensatz- und Paletten-Editor) |
| Cmd/Strg+Umschalt+Z | Wiederholen |
| Cmd/Strg+F | Suchen … |
| Cmd/Strg+Alt+F | Ersetzen … |
| Cmd/Strg+L | Gehe zu Zeile … |
| Cmd/Strg+Umschalt+F | In Projekt suchen … (öffnet in der Ablage die Seite „Suche“) |
| Cmd/Strg+. | Korrigieren (Quick-Fix) … |
| Cmd/Strg+Alt+C | Kommentar umschalten |
| Cmd/Strg+Alt+K | Lesezeichen umschalten |
| Cmd/Strg+Alt+J / Cmd/Strg+Alt+H | nächstes / voriges Lesezeichen |
| F1 | Hilfe zum Wort unter dem Cursor (öffnet die Hilfe und schlägt das Wort nach) |
| F12 | Gehe zu Definition |
| Cmd/Strg+I | Markdown: Kursiv |
| Cmd/Strg+E | Markdown: Code |
| Cmd/Strg+Umschalt+X | Markdown: Durchgestrichen |
| Cmd/Strg+Umschalt+K | Markdown: Link einfügen |

Vorschläge beim Schreiben erscheinen von selbst (nach dem Tippen, nach „.“ und nach „@“); eine Taste zum Öffnen gibt es nicht, aber den Befehl **Vervollständigung öffnen** in der Befehlspalette. **Fett** hat kein Kürzel (Befehl „Markdown: Fett“ in der Befehlspalette).

In den Eingabefeldern der Suche und von „Gehe zu Zeile“ führt **Enter** die Suche aus (in der Suche mit **Umschalt+Enter** zum vorherigen Treffer), **Esc** schließt das Feld.

## Ansicht und Oberfläche

| Tastenkürzel | Wirkung |
|---|---|
| Cmd/Strg+K | Befehl oder Datei suchen … (Befehlspalette) |
| Cmd/Strg+B | Seitenleiste ein- und ausklappen |
| Cmd/Strg+Alt+Z | Zen-Modus: nur Kopfzeile und Editor (Seitenleiste, Ablage, Panel Maschine und Statusleiste ausblenden); noch einmal stellt das Layout wieder her |
| Cmd/Strg+Alt+B | Panel Maschine ein- und ausblenden |
| Cmd/Strg+J | Ablage unter dem Editor ein- und ausklappen |
| Cmd/Strg+Umschalt+Enter | Bereich mit dem Fokus maximieren und wiederherstellen |
| Cmd/Strg+Alt+T | Editor teilen bzw. Teilung aufheben |
| Cmd/Strg+Plus / Cmd/Strg+Minus / Cmd/Strg+0 | Oberfläche vergrößern, verkleinern, Originalgröße (Menü Ansicht ▸ Zoom und Schrift) |
| Cmd/Strg+Komma | Einstellungen |
| Cmd/Strg+Alt+0 | Bereich Projekt |

Im Menü **Ansicht** gibt es ohne Tastenkürzel noch **Hell/Dunkel umschalten**, **Hoher Kontrast**, **Layout zurücksetzen**, **Editor-Schrift größer/kleiner**, **Darstellung zurücksetzen**, **Zeilenumbruch im Editor** und die Befehle zum Maximieren einzelner Bereiche (Seitenleiste, Dokumentenfeld, Ablage, Panel Maschine). Das Farbschema wählst du auch im Menü **Farbschema** der Kopfzeile.

## Hilfe

| Tastenkürzel | Wirkung |
|---|---|
| F1 | Hilfe öffnen; im Editor schlägt F1 das Wort unter dem Cursor nach |

Das Menü **Hilfe** enthält außerdem **Hilfe im Reiter öffnen**, den **Konfigurationsassistenten** (ganz, oder nur ROMs, nur Emulatoren, nur echte Hardware, nur Design und Editor, Nach Neuem suchen), **Feedback senden …**, **Nach Updates suchen …** und **Über VintageCoder**. Konfiguration exportieren, importieren und zurücksetzen findest du unter Einstellungen ▸ Sicherung.

## Einzeltasten in den Editoren

In den Editoren gibt es Einzeltasten, sobald die Zeichenfläche den Fokus hat und kein Textfeld aktiv ist. Du kannst sie unter **Einstellungen ▸ Tastenkürzel** im Abschnitt **Tasten** mit dem Schalter **Einzeltasten der Editoren** abschalten (Standard: an). Pfeile, Leertaste, Esc und Entf gelten auch dann.

| Editor | Tasten |
|---|---|
| Sprites und Zeichensätze | **D** Stift, **E** Radierer, **I** Pipette, **F** Füllen, **L** Linie, **R** Rechteck, **S** Auswahl, **X** / **Y** waagerecht / senkrecht spiegeln, **1** bis **4** Pixelwert (Farbe), **G** Raster, **O** Onion Skin (nur Sprites), **Pfeiltasten** bewegen die Zellmarke (**Eingabe** oder **Leertaste** setzt einen Punkt, **Entf** löscht ihn), **Bild auf** / **Bild ab** oder **Alt+Pfeil links/rechts** voriges / nächstes Bild oder Zeichen, **Esc** hebt die Auswahl auf |
| Palette | **Pfeiltasten** wählen ein Farbfeld, **Bild auf** / **Bild ab** springen vier Zeilen, **Pos1** / **Ende** zum ersten und letzten Eintrag |
| Bildschirm | **D** Zeichen, **T** Text, **F** Füllen, **R** Rahmen, **I** Pipette, **E** Löschen; **Pfeiltasten** bewegen den Cursor, **Eingabe** oder **Leertaste** setzt das Zeichen (beim Rahmen: erste und zweite Ecke). Im Textwerkzeug tippst du Zeichen ein, **Esc** schließt die Eingabe ab bzw. bricht einen angefangenen Rahmen ab. |
| Kacheln | **Pfeiltasten** bewegen die Zellmarke, **Eingabe** oder **Leertaste** setzt das gewählte Zeichen in die Zelle, **I** Pipette. |
| Sprites und Zeichensätze (Zoom) | **+** und **−** Zoom, **0** Einpassen (auch Cmd/Strg+Mausrad) |

Mit gedrückter Cmd-, Strg- oder Alt-Taste wirken die Einzeltasten nicht. In den Editoren gelten außerdem Cmd/Strg+C, +X, +V und +A.

## Im Tracker

| Taste | Wirkung |
|---|---|
| Pfeiltasten | Cursor im Muster bewegen (mit Umschalt: Auswahl) |
| Bild auf / Bild ab | 8 Zeilen hoch / runter |
| Pos1 / Ende | zur ersten / letzten Zeile |
| Leertaste | Wiedergabe starten und anhalten |
| Komma / Punkt | Oktave tiefer / höher |
| Umschalt+Leertaste | nur das gewählte Muster abspielen |
| Cmd/Strg+1 / Cmd/Strg+2 / Cmd/Strg+3 | Modus Noten / Instrumente / Ablauf |
| [ / ] | vorheriges / nächstes Instrument |
| Z S X D C V G B H N J M | Klaviertasten, untere Oktave (Z = C, S = Cis, X = D …); nach Tastenposition, auf QWERTZ steht an der Stelle des Z ein Y |
| Q 2 W 3 E R 5 T 6 Y 7 U I | Klaviertasten, nächste Oktave |
| Entf oder Rücktaste | Zelle leeren |
| Alt+↑ / Alt+↓ | Instrument der Zelle ändern |
| 1 | Noten-Aus-Zeichen („=== --“) eintragen |
| Cmd/Strg+S, Cmd/Strg+Z | Speichern, Rückgängig (mit Umschalt: Wiederholen) |
| Cmd/Strg+C / X / V / A | Kopieren, Ausschneiden, Einfügen, Alles wählen |

Die Klaviertasten tragen eine Note ein und rücken eine Zeile weiter. Im Modus Instrumente wählen ↑ und ↓ das Instrument; Klaviertasten und Leertaste spielen es nur vor. Der Tracker liegt im Hauptbereich **Musik**; Details dazu im Kapitel [Der Tracker](kapitel:tracker).

> **Info:** Das Nachschlagewerk [Tastenkürzel](hilfe:shortcuts) hat eine kürzere Tabelle. Maßgeblich sind die Kürzel, die im Menü neben den Befehlen stehen.

## Häufige Fragen

### Installation und erster Start

**Wie richte ich VintageCoder beim ersten Mal ein?** Beim allerersten Start öffnet sich der **Konfigurationsassistent**. Wähle die **Schnelleinrichtung** (7 Schritte, etwa 2 Minuten). Du kannst ihn jederzeit über **Hilfe ▸ Konfigurationsassistent …** wieder aufrufen. Mehr dazu in [Installation und Ersteinrichtung](kapitel:installation).

**Wo ist das Startbild?** Das Startbild beim Programmstart schaltest du unter **Einstellungen ▸ Allgemein** im Abschnitt **Startbild** ein oder aus. Ohne Projekt zeigt der Hauptbereich **Code** die **Startseite** mit neuem Projekt, zuletzt geöffneten Projekten und Beispielen; mit dem Befehl **Startseite** (Menü **Datei**) kommst du jederzeit dorthin.

**Wo bekomme ich die Emulatoren her?** VintageCoder bringt keine Emulatoren mit. Installiere VICE (C64, C128, Plus/4, VC-20), atari800 oder Altirra (Atari) und Xemu (MEGA65) selbst und trage ihren Pfad unter **Einstellungen ▸ Ziele und Hardware** im Abschnitt **Emulatoren** ein. Bleibt der Pfad leer, sucht die IDE selbst. Wo es für dein System ein festes Paket gibt, bietet die Emulator-Karte **Portabel installieren …** an; geladen wird erst nach deiner Bestätigung. Mehr dazu im Kapitel [Installation und Ersteinrichtung](kapitel:installation), für Linux im Abschnitt [Emulatoren unter Linux](kapitel:systeme).

**Wo bekomme ich ROMs her?** VintageCoder liefert keine Original-ROMs mit und lädt keine aus dem Netz. Wenn du ROMs brauchst, legst du deine eigenen Abbilder in den ROM-Ordner: **Einstellungen ▸ ROMs** (Knopf **ROM-Datei oder Zip wählen …**, oder die Datei auf das Fenster ziehen), oder **Hilfe ▸ Konfigurationsassistent: nur ROMs …**. VICE und atari800 laufen ohne ROM mit eigenen Ersatz-ROMs; **Xemu braucht eine MEGA65-ROM-Datei von 131072 Byte**. Die Datei wird am Inhalt erkannt, der Name ist egal. Ob du ein Abbild benutzen darfst, klärst du selbst.

### Bauen und Starten

**F5 oder F6 tut nichts.** Das Programm wurde nicht gebaut, weil es Fehler gibt: Schau in der Ablage in den Reiter **Probleme**; ein Klick auf eine Meldung springt zur Zeile. Läuft schon ein Bau, ist F5 gesperrt; läuft schon eine Debug-Sitzung, bringt F6 dich nur in den Bereich **Debug** (erst beenden: **Beenden** in der Kopfzeile oder Umschalt+F5).

**Der Emulator startet nicht.** Prüfe den Pfad unter **Einstellungen ▸ Ziele und Hardware**; die Emulator-Karte zeigt „Gefunden: …“ oder „Nicht gefunden“, und **Testen** probiert den Emulator aus. Es läuft immer nur **ein** Emulator zugleich.

**Mein Programm startet, aber es passiert nichts.** Baue neu (F7) und prüfe das Ziel im **Startziel**-Knopf in der Kopfzeile. Prüfe, dass die Startadresse stimmt: Beim C64 braucht ein Programm `.basicstub` bei `.org $0801`, beim Atari `.run start`.

**Das Programm läuft auf dem falschen Gerät.** Der **Startziel**-Knopf in der Kopfzeile (System und Startziel) steht auf einem Gerät, das du nicht wolltest. Ein Klick öffnet die Liste mit dem Emulator und den eingetragenen Geräten. Das System selbst bestimmt das Projekt (oder `.target` im Quelltext).

**Was bedeutet die Warnung W104?** Der Assembler meldet, dass ein Befehl nicht zum offiziellen Befehlssatz der CPU gehört (ein undokumentierter Opcode wie `lax`). Das Beispiel `examples/c64/border` tut das absichtlich. Mit `illegals = deny` im `.cpu`-Befehl wird daraus ein Fehler, mit `illegals = warn` bleibt es eine Warnung.

### Debugger und Tests

**Haltepunkte werden ignoriert.** Baue neu; die Zuordnung von Zeile zu Adresse kommt aus dem letzten Bau. Bei atari800 und MEGA65 gilt außerdem: nur ein Haltepunkt zugleich.

**Register und Speicher sind leer.** Das Programm läuft. Drücke **⏸** (Anhalten, Alt+F6) oder halte an einem Haltepunkt an.

**Ich kann die Flags und Chip-Register nicht ändern.** Sie sind reine Anzeige, damit Lesezugriffe nichts verfälschen. Ändere das Programm im Quelltext und starte neu.

**Meine Tests werden alle übersprungen.** Die Auswahl **Ziel** im Reiter **Tests** der Ablage steht auf „Nur statische Prüfungen“, oder die Tests brauchen den Emulator (VICE). Siehe [Automatisch testen](kapitel:tests).

### Grafik und Klang

**Der Zeichensatz-Editor zeigt den eigenen Satz von VintageCoder.** Es fehlt das System-ROM mit der Schrift; sie erscheint ersatzweise als eingebaute Schrift ohne Grafikzeichen. Lege das ROM wie oben beschrieben ab oder importiere die Schrift auf der Kommandozeile mit `vcc media import-rom` (siehe [Editoren im Überblick](hilfe:graphics)).

**Für den Plus/4 gibt es keinen Sprite-Editor.** Der TED hat keine Sprites, und für Plus/4 gibt es noch keine Profile für Sprites, Zeichensätze oder Songs.

**Das Emulator-Bild fehlt.** Es erscheint im Bereich **Debug** im Abschnitt **Bildschirm** und im Panel **Maschine**, nur in einer Debug-Sitzung; **Bild aktualisieren** holt es. Nur VICE liefert ein Bild (C64, C128, Plus/4, VC-20). Atari und MEGA65 liefern keines.

### Dateien

**Wo liegen meine Grafiken und Songs?** Als Textdateien im Projekt, zum Beispiel `font.vcchar` oder `hero.vcsprite`. Zum Einbinden gibst du die Datei selbst an, zum Beispiel `.include "graphics/hero.vcsprite"`; beim Bauen wandelt VintageCoder sie um.

**Welche Dateien kann ich bedenkenlos löschen?** Den Ordner `build/` baust du jederzeit mit F7 neu.

### Markdown

**Warum zeigt die Vorschau mein HTML nicht?** Rohes HTML wird nie ausgeführt; es steht in der Vorschau als Text. Schreibe es in Markdown um.

**Warum lädt die Vorschau mein Bild aus dem Internet nicht?** Webbilder werden nicht geladen (Datenschutz). Lege eine PNG-Datei ins Projekt und verweise relativ darauf: `![Alt](pics/bild.png)`.

**Warum ist ein Link rot?** Die Zieldatei fehlt oder der Pfad führt aus dem Projekt hinaus. Pfade gelten relativ zur Markdown-Datei.

**Cmd+B und Cmd+K machen etwas anderes als in anderen Editoren.** Sie klappen die Seitenleiste ein/aus und öffnen die Befehlspalette. Fett hat deshalb kein Kürzel; der Link liegt auf Cmd/Strg+Umschalt+K.

## Fehlersuche: die ersten drei Schritte

1. **Schau in den Reiter Probleme** (Ablage). Dort stehen Fehler und Warnungen mit Datei und Zeile; ein Klick springt hin.
2. **Schau in den Reiter Ausgabe** (Ablage). Dort steht der Ablauf des Baus; schlägt ein Bau ohne Meldung zu einer Zeile fehl, steht dort der Grund. Siehe [Bauen und Starten](kapitel:bauen-starten).
3. **Prüfe die Einrichtung:** Emulator (**Einstellungen ▸ Ziele und Hardware**) und ROM (**Einstellungen ▸ ROMs**).

Hilft das nicht, grenze das Problem ein: Baue das kleinste Beispiel des Zielsystems (`examples/c64/border`, `examples/atari/colorbars`, `examples/mega65/hello`). Baut es, liegt das Problem in deinem Projekt; baut es nicht, liegt es an der Einrichtung (ROM, Emulator, Pfad).

## Wie geht es weiter?

- Fachwörter nachschlagen: Glossar (`GLOSSAR.md`).
- Die Oberfläche kennenlernen: [Die Oberfläche im Überblick](kapitel:oberflaeche).
- Einen Fehler im laufenden Programm finden: [Der Debugger](kapitel:debugger).
- Zum Nachschlagen: [Tastenkürzel](hilfe:shortcuts), [ROM-Dateien](hilfe:roms), [Einstellungen](hilfe:settings).

## Linux

Für Linux gibt es Pakete für Debian, Ubuntu, Fedora und openSUSE sowie ein Archiv (siehe [Installation und Ersteinrichtung](kapitel:installation)); X11 und Wayland werden unterstützt. Alle Tastenkürzel gelten unverändert mit **Strg** statt **Cmd**. Wie du die Emulatoren unter Linux installierst, steht im Abschnitt [Emulatoren unter Linux](kapitel:systeme) im Kapitel „Zielsysteme im Vergleich“.
