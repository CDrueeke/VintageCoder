# Tastenkürzel

Die Liste folgt der Befehlstabelle der Oberfläche: Was hier steht, steht auch im Menü neben dem Befehl und in der Befehlspalette (**Cmd/Strg+K**). Die Kürzel sind fest; Anpassen ist noch nicht möglich.

**Cmd** ist die Befehlstaste des Mac. Unter Windows und Linux nimmst du **Strg**. **Umschalt** ist die Hochstelltaste, **Alt** auf dem Mac die Wahltaste.

Einige Befehle gibt es in dieser Version noch nicht; Menü und Befehlspalette zeigen sie nicht: Zoom, Raster und Onion Skin der Editoren per Menü (die Knöpfe und Tasten im Editor gibt es), BASIC in Assembler umwandeln, Kodierung umschalten, BASIC: Hex-Vorschläge und die Importe von Zeichensätzen (Standardsatz, ROM, PNG-Raster). Sie sind hier nicht aufgeführt.

## Bauen, Starten, Tests
| Taste | Wirkung |
|---|---|
| F7 | Projekt bauen |
| F5 | Startprogramm starten (baut zuerst) |
| F6 | Startprogramm debuggen (baut zuerst, öffnet den Bereich **Debug**) |
| Alt+Umschalt+F7 | aktive Datei bauen |
| Alt+Umschalt+F5 | aktive Datei starten |
| Alt+Umschalt+F6 | aktive Datei debuggen |
| Umschalt+F5 | laufenden Emulator oder die Debug-Sitzung beenden |
| Alt+F8 | Tests ausführen |
| F8 / Umschalt+F8 | nächstes / voriges Problem |
| Cmd/Strg+Umschalt+E | Exportieren … |

Das Startprogramm legst du im Bereich **Projekt** auf der Seite **Quellen** fest.

## Debuggen
Die Befehle stehen im Menü **Projekt ▸ Debuggen**. Die Schritte wirken, wenn eine Debug-Sitzung läuft und das Programm angehalten ist (siehe [Debugger](help:debugger)).

| Taste | Wirkung |
|---|---|
| Alt+F5 | Weiter (bis zum nächsten Haltepunkt); ohne Sitzung startet es eine |
| Alt+F6 | Anhalten |
| F10 | Schritt über (ein `JSR` läuft durch) |
| F11 | Schritt hinein (ein `JSR` springt hinein) |
| Umschalt+F11 | Schritt hinaus (bis zum Rücksprung des Unterprogramms) |
| Umschalt+F5 | Debuggen beenden |
| F9 | Haltepunkt in der Zeile mit dem Cursor umschalten |

**Bis hierher ausführen** (bis zur Cursorzeile) hat kein Kürzel.

## Dateien und Projekte
| Taste | Wirkung |
|---|---|
| Cmd/Strg+Alt+N | Neues Projekt … |
| Cmd/Strg+Umschalt+O | Projekt öffnen … (Projektdatei) |
| Cmd/Strg+Umschalt+W | Projekt schließen |
| Cmd/Strg+Alt+0 | Bereich Projekt |
| Cmd/Strg+O | Datei öffnen … |
| Cmd/Strg+N | Neuer Reiter (unbenannt) |
| Cmd/Strg+Umschalt+T | Neue Notiz |
| Cmd/Strg+S | Speichern (auch im Sprite-, Zeichensatz- und Palette-Editor) |
| Cmd/Strg+Alt+S | Alle speichern |
| Cmd/Strg+W | Datei schließen |
| Cmd/Strg+Q | VintageCoder beenden |

## Bearbeiten und Suchen
| Taste | Wirkung |
|---|---|
| Cmd/Strg+Z / Cmd/Strg+Umschalt+Z | Rückgängig / Wiederholen (Quelltext, Sprite-, Zeichensatz- und Palette-Editor) |
| Cmd/Strg+F | Suchen … |
| Cmd/Strg+Alt+F | Ersetzen … |
| Cmd/Strg+L | Gehe zu Zeile … |
| Cmd/Strg+Umschalt+F | In Projekt suchen … (öffnet in der Ablage den Reiter **Suche**) |
| F12 | Gehe zu Definition |
| Cmd/Strg+. | Korrigieren (Quick-Fix) |
| Cmd/Strg+Alt+C | Kommentar umschalten |
| Cmd/Strg+Alt+K | Lesezeichen umschalten |
| Cmd/Strg+Alt+J / Cmd/Strg+Alt+H | nächstes / voriges Lesezeichen |
| F9 | Haltepunkt in der Zeile mit dem Cursor umschalten |

Vorschläge beim Tippen erscheinen von selbst (nach dem Tippen, nach „.“ und nach „@“); eine eigene Taste zum Öffnen gibt es nicht, aber den Befehl **Vervollständigung öffnen** in der Befehlspalette. Die Markdown-Befehle im Menü Bearbeiten haben die Kürzel Cmd/Strg+I (Kursiv), Cmd/Strg+E (Code), Cmd/Strg+Umschalt+X (Durchgestrichen) und Cmd/Strg+Umschalt+K (Link einfügen); **Fett** hat kein Kürzel.

## Ansicht und Oberfläche
| Taste | Wirkung |
|---|---|
| Cmd/Strg+K | Befehl oder Datei suchen … (Befehlspalette) |
| Cmd/Strg+B | Seitenleiste ein- oder ausklappen |
| Cmd/Strg+J | Ablage unter dem Editor ein- oder ausklappen |
| Cmd/Strg+Alt+B | Panel Maschine ein- oder ausklappen |
| Cmd/Strg+Alt+Z | Zen-Modus: nur Kopfzeile, zweite Reihe und Arbeitsfläche; noch einmal: zurück |
| Cmd/Strg+Umschalt+Enter | Bereich mit dem Fokus maximieren / wiederherstellen |
| Cmd/Strg+Alt+T | Editor teilen / Teilung aufheben |
| Cmd/Strg+Alt+→ / Cmd/Strg+Alt+← | nächster / vorheriger Reiter |
| Cmd/Strg+Plus / Cmd/Strg+Minus / Cmd/Strg+0 | Oberfläche vergrößern / verkleinern / Originalgröße |
| Cmd/Strg+Komma | Einstellungen |
| F1 | Hilfe zum Wort unter dem Cursor (öffnet die Hilfe) |

Ohne Kürzel, aber im Menü **Ansicht**: Hell/Dunkel umschalten, Hoher Kontrast, Layout zurücksetzen, Editor-Schrift größer und kleiner, Darstellung zurücksetzen, Zeilenumbruch und die Befehle zum Maximieren einzelner Bereiche.

## Einzeltasten in den Editoren
Sobald die Zeichenfläche eines Editors den Fokus hat und kein Textfeld aktiv ist, gelten Einzeltasten (Buchstaben, Ziffern, Plus und Minus). Mit **Cmd**, **Strg** oder **Alt** gedrückt wirken sie nicht.

| Editor | Tasten |
|---|---|
| Sprites und Zeichensätze | D Stift, E Radierer, I Pipette, F Füllen, L Linie, R Rechteck, S Auswahl, X / Y waagerecht / senkrecht spiegeln, 1 bis 4 Pixelwert, G Raster, O Onion Skin (nur Sprites), Pfeiltasten bewegen die Zellmarke (Eingabe oder Leertaste setzt einen Punkt, Entf löscht ihn), Bild auf / Bild ab oder Alt+Pfeil links/rechts voriges / nächstes Bild oder Zeichen, Esc hebt die Auswahl auf, + / − / 0 Zoom (auch Cmd/Strg und Mausrad) |
| Palette | Pfeiltasten wählen ein Farbfeld, Bild auf / Bild ab springen vier Zeilen, Pos1 / Ende zum ersten und letzten Eintrag |

In den Editoren gelten außerdem Cmd/Strg+C, +X, +V und +A. Die Einzeltasten der anderen Editoren stehen unten. Du kannst die Einzeltasten unter **Einstellungen ▸ Tastenkürzel** (Schalter **Einzeltasten der Editoren**) abschalten.

| Editor | Tasten |
|---|---|
| Bildschirm | D Zeichen, T Text, F Füllen, R Rahmen, I Pipette, E Löschen; Pfeiltasten bewegen den Cursor, Eingabe oder Leertaste setzt das Zeichen, Esc schließt die Texteingabe ab bzw. bricht einen Rahmen ab |
| Kacheln | Pfeiltasten bewegen die Zellmarke, Eingabe oder Leertaste setzt das gewählte Zeichen, I Pipette |

## Tracker
Die Tasten gelten, wenn die Musterfläche den Fokus hat.

| Taste | Wirkung |
|---|---|
| Pfeiltasten | Cursor bewegen (mit Umschalt: Auswahl) |
| Bild auf / Bild ab | 8 Zeilen hoch / runter |
| Pos1 / Ende | erste / letzte Zeile |
| Leertaste / Umschalt+Leertaste | ganzen Song / nur das Muster abspielen und anhalten |
| Komma / Punkt | Oktave tiefer / höher |
| [ / ] | voriges / nächstes Instrument |
| Cmd/Strg+1 / Cmd/Strg+2 / Cmd/Strg+3 | Modus Noten / Instrumente / Ablauf |
| Klaviertasten Z S X D C V G B H N J M und Q 2 W 3 E R 5 T 6 Y 7 U I | Note eintragen (nach Tastenposition) |
| 1 | Noten-Aus-Zeichen eintragen |
| Entf oder Rücktaste | Zelle leeren |
| Alt+↑ / Alt+↓ | Instrument der Zelle ändern |

## Maus
| Aktion | Wirkung |
|---|---|
| Klick auf die Zeilennummer | Haltepunkt setzen oder löschen |
| Klick auf ▾ oder ▸ am Rand | Bereich falten oder entfalten |
