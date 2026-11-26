# Handprüfung E0: Grundlage der neuen Oberfläche

Stand: 2026-11-20 · Branch `claude/ui-e0` · App: `~/Development/abnahme-ui/e0/VintageCoder.app` (unsigniert: beim ersten Start Rechtsklick ▸ Öffnen) · Beispielprojekte (nur Kopien): `~/Development/abnahme-ui/e0/beispiele/{c64,atari,mega65}/jumprun`.

**Zu wissen:** E0 ist die Grundlage. Es gibt noch keinen Code-Editor, keine Studios und kein Bauen; die Bereiche zeigen einen Platzhalter mit der Etappe, in der sie kommen. Geprüft werden Fenster, Navigation, Design, Sprachwechsel, Befehlspalette, Menüleiste, Dateibaum (nur Anzeige), Projekt öffnen und die Bausteine der Komponentenbibliothek. Die App schreibt **nichts in das Projekt**; sie führt nur die Zuletzt-Liste (`recent.json`) und speichert die Sprachwahl in den Einstellungen, **beides gemeinsam mit der bisherigen Oberfläche** (gleiche Dateien im Konfigurationsordner).

Vorbereitung: Prüfsumme der Kopien vor dem Test merken:
```bash
cd ~/Development/abnahme-ui/e0/beispiele && find . -type f -exec shasum {} + | sort -k2 | shasum
```
Sprache zu Beginn: Deutsch (Globus-Symbol oben rechts ▸ Deutsch). Tastatur: deutsch (QWERTZ).

Je Punkt: **ok** oder **Fehler** (Beschreibung, Bild).

## Teil 1: Maus und Fenster

| Nr | Handlung | Erwartet | Ergebnis |
|---|---|---|---|
| 1 | App starten | Fenster 1440×900, **vereinte Titelleiste**: Ampelknöpfe links, daneben Seitenleisten-Knopf und „Kein Projekt“, in der Mitte die fünf Bereiche, rechts Globus, Sonne/Mond, Lupe. Kein zweiter Fenstertitel | |
| 2 | Am leeren Bereich der Titelleiste ziehen; doppelklicken | Fenster bewegt sich; Doppelklick maximiert und stellt wieder her | |
| 3 | Startansicht prüfen | Titel „VintageCoder“, Knopf „Projekt öffnen“, darunter „Zuletzt geöffnete Projekte“ (leer: Hinweistext) | |
| 4 | „Projekt öffnen“ ▸ Ordner `…/beispiele/c64/jumprun` wählen | Dateibaum links (Ordner `graphics`, `lib`, `music`, dann Dateien mit Marke ASM/TXT), Titelleiste „C64: jumprun“ mit Pfad, Chip „C64“ | |
| 5 | Ordner anklicken, dann Datei anklicken | Ordner klappt auf/zu (Pfeil dreht); Datei wird hervorgehoben, ihr Pfad steht im Code-Bereich | |
| 6 | Fenster schließen, App neu starten | In der Startansicht steht `jumprun` mit „C64“; Klick öffnet es wieder | |
| 7 | Navigation durchklicken: Code, Grafik, Musik, Daten, Debug | Zweite Reihe **nur** bei Grafik (Sprites, Zeichensatz, Kacheln, Bildschirm, Karten, Palette) und Daten (Disk, Hex, Speicher); Platzhalter nennt die Etappe (z. B. Sprite-Editor ▸ E3, Musik-Editor ▸ E5, Disk-Editor ▸ E2, Debugger ▸ E6) | |
| 8 | Seitenleisten-Knopf (links oben) und Menü Ansicht ▸ „Linke Spalte ein-/ausklappen“ (⌘B) | Dateibaum verschwindet und kommt wieder | |
| 9 | Sonne/Mond-Menü: System, Hell, Dunkel | Wechsel sofort; „System“ folgt der macOS-Einstellung (in den Systemeinstellungen umschalten); Wahl bleibt nach Neustart. Farben wie in den Mockups (`neu-1…14`) | |

## Teil 2: Sprache (de, en, xx)

| Nr | Handlung | Erwartet | Ergebnis |
|---|---|---|---|
| 10 | Globus ▸ English | **Ohne Neustart** alles englisch, auch die Menüleiste am Bildschirmrand (File, Edit, View, Project, Help) und der Alt-/Kürzeltext; keine Umlaute | |
| 11 | Globus ▸ Pseudo-Sprache | Alle Texte verfremdet in `[…~~~]` und länger; **nichts** erscheint normal außer Projektname, Pfad und Dateinamen; nichts abgeschnitten oder überlappt | |
| 12 | Zurück auf Deutsch; Menüleiste ansehen | Datei, Bearbeiten, Ansicht, Projekt, Hilfe; Untermenüs unter Bearbeiten und Ansicht | |

## Teil 3: Befehlspalette, Menüleiste, Bausteine

| Nr | Handlung | Erwartet | Ergebnis |
|---|---|---|---|
| 13 | ⌘K (und Lupe) | Palette öffnet sofort, Eingabefeld hat den Fokus; Gruppen „Befehle“ und „Gehe zu“ | |
| 14 | „sprache“ tippen, Pfeil ab, Eingabe | Einträge „Sprache: …“ erscheinen; Eingabe wechselt die Sprache; Palette schließt | |
| 15 | Befehl wählen, den es noch nicht gibt (z. B. „Projekt bauen“) | Meldung unten rechts „…folgt in einer späteren Etappe“; Befehle, die ein Projekt brauchen, sind ohne Projekt grau und lassen sich nicht ausführen | |
| 16 | Esc in der Palette | Schließt; der Fokus kehrt zum vorherigen Element zurück | |
| 17 | Menüleiste: Datei ▸ „Projekt öffnen …“ (⇧⌘O) | Ordnerdialog; Einträge, die ein Projekt oder einen Editor brauchen, sind ohne Projekt/Editor ausgegraut, mit Projekt (Projekt/Datei-Gruppe) aktiv | |
| 18 | Palette ▸ „Komponentenbibliothek“ | Karte, Segment, Schalter, Schieber, Tabelle, Dialog, Menü, Tooltip, Kürzelanzeige, Tastenfang an einem Ort | |
| 19 | Dort: Dialog öffnen; Menü öffnen; Tooltip; Tastenfang (Y und Z drücken) | Dialog hält den Fokus, Esc schließt; Menü mit Pfeiltasten; Tooltip auch bei Tab-Fokus; Tastenfang zeigt **den Buchstaben auf der Taste** (QWERTZ: Y/Z nicht vertauscht) | |
| 20 | Im Palettenfeld ⌘A, ⌘C, ⌘V | Markieren, Kopieren, Einfügen funktionieren | |

## Teil 4: Nur Tastatur (QWERTZ)

| Nr | Handlung | Erwartet | Ergebnis |
|---|---|---|---|
| 21 | Mit Tab durch das Fenster | Sichtbarer Fokusring überall; Reihenfolge: Seitenleiste, Bereiche, zweite Reihe, Sprache, Farbschema, Suche, Inhalt | |
| 22 | In der Bereichsleiste Pfeil links/rechts | Wechselt den Bereich (Fokus bleibt in der Leiste); Tab verlässt sie | |
| 23 | Im Dateibaum Pfeil auf/ab, rechts/links, Eingabe | Zeile wechseln, Ordner auf-/zuklappen, Datei wählen | |

## Teil 5: Dateien

| Nr | Handlung | Erwartet | Ergebnis |
|---|---|---|---|
| 24 | Prüfsumme der Kopien erneut bilden (Befehl oben) | **Gleich** wie vorher (E0 schreibt nicht ins Projekt) | |

## Teil 6: Fenstergröße und Darstellung

| Nr | Handlung | Erwartet | Ergebnis |
|---|---|---|---|
| 25 | Fenster auf Mindestgröße (900×600) ziehen, in Deutsch, English und xx | Mindestgröße greift; keine abgeschnittenen Knöpfe, keine Überlappung in der Titelleiste | |
| 26 | Vollbild (grüner Knopf) | Layout füllt das Fenster, nichts springt | |

## Teil 7: VoiceOver (⌘F5)

| Nr | Handlung | Erwartet | Ergebnis |
|---|---|---|---|
| 27 | Navigation, Dateibaum, Palette mit VoiceOver ansteuern | Bereiche als „Hauptbereiche“ mit „aktuelle Seite“; Baum mit Ebene und aufgeklappt/zugeklappt; Palette: Eingabefeld, Trefferzahl wird angesagt, gewählter Eintrag wird vorgelesen | |
| 28 | Komponentenbibliothek mit VoiceOver | Schalter „ein/aus“, Segment als Auswahlgruppe, Tabelle mit Zeilen und Spalten, Dialog wird als Dialog angesagt | |

## Teil 8: Leistung

| Nr | Handlung | Erwartet | Ergebnis |
|---|---|---|---|
| 29 | Gefühlte Verzögerung: Palette öffnen, Sprache wechseln, Bereich wechseln | Alles ohne spürbare Wartezeit | |
| 30 | Palette ▸ „Messung (Prüfpunkt G1)“ ▸ „Messung starten“ | Tabelle mit Mittelwert, Median und Höchstwert; Zahlen mit denen aus `docs/messwerte-e0.md` vergleichbar | |

**Ergebnis:** je Punkt ok / Fehler → Freigabe: ja / nach Korrektur.
