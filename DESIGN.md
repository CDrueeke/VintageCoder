# DESIGN – Gestaltung der VintageCoder-Oberfläche

Diese Datei fasst zusammen, wie die Oberfläche (`vc-gui`, Bausteine in `crates/vc-gui-ui`) aussieht und aufgebaut ist. Die Zahlen stammen aus dem Code
(`muster.rs`, `theme.rs`, `fonts.rs`); steht hier etwas anderes als dort, gilt der Code, und diese Datei wird angepasst. Wer ein Studio oder eine Seite
baut, liest zuerst diese Datei und benutzt dann die Bausteine, statt eigene zu schreiben.

## 1. Grundsätze

1. **Ein Muster für alles.** Jedes Studio folgt dem Studio-Rahmen mit sechs Zonen (Abschnitt 4). Ein Studio ohne `muster::studio_rahmen` wird nicht angenommen.
2. **Werkzeug und Dokument trennen.** Was nur beim Arbeiten gilt (Pinsel, Oktave, Zoom), steht in Karten „Werkzeug“ und wird nicht gespeichert; was in die Datei geht, steht in Karten „Dokument“.
3. **Die Fläche gehört der Arbeit.** Die Arbeitsfläche ist eingepasst und hat keine Scrollbalken; die Seitenspalte ist der einzige Scrollbereich.
4. **Farbe ist nie der einzige Unterschied.** Zustände tragen zusätzlich Form oder Text (✓, Rahmen, Marke „ASM“, „▶“).
5. **Nur Rollenfarben.** Keine festen Farbwerte in Studios; alles kommt aus `theme::Pal` und gilt in Hell, Dunkel und Hohem Kontrast.
6. **Jede Aktion hat einen Weg ohne Maus.** Kürzel stehen im Tooltip; alle Befehle stehen in der Befehlstabelle und damit in der Palette.

## 2. Farben (`theme::Pal`)

| Rolle | Hell | Dunkel | Hoher Kontrast | Verwendung |
|---|---|---|---|---|
| `bg` | `#f4f5f7` | `#14161a` | `#000000` | Grund, Arbeitsumgebung |
| `panel` | `#ffffff` | `#1b1e24` | `#0a0a0a` | Karten, Kopf- und Fußzeile |
| `panel2` | `#eceef2` | `#22262e` | `#1a1a1a` | Leisten, Knöpfe, Taktband |
| `line` | `#d5d9e0` | `#2e333d` | `#9a9a9a` | Rahmen, Trenner (im Hohen Kontrast 2 px) |
| `text` / `mute` | `#1c2027` / `#5d6675` | `#e6e8ec` / `#8b93a1` | `#ffffff` / `#d0d0d0` | Text / gedämpfter Text, Hinweise |
| `accent` | `#1f5be0` | `#4f8cff` | `#7dbaff` | Auswahl, Cursor, Marken, Hauptsymbol |
| `ok` / `warn` / `err` | `#0f7655` / `#925300` / `#c02424` | `#3ecf8e` / `#f0a63a` / `#ff6b6b` | hell abgestuft | Hauptknopf und Erfolg / Warnung / Fehler |
| `editor` | `#fbfbfc` | `#171a1f` | `#000000` | Fläche hinter Text und Raster |
| `sel` | `#dfe9ff` | `#1b2840` | `#0f2552` | gewählter Eintrag (Füllung) |

Syntaxfarben (`kw`, `num`, `string`, `cmt`, `lab`, `dir`) gehören dem Editor. Schrift auf Akzentflächen: `Pal::accent_text()`. Die Art-Marke einer Datei im Baum
kommt aus `Pal::art_farbe(Gruppe)`; die Marke steht immer als Text daneben (Legende in der Hilfe). Ein Kontrasttest prüft alle Rollen gegen alle Flächen.

## 3. Schrift

- **Inter** für die Oberfläche (Regular, SemiBold als `UI_FETT`), **JetBrains Mono** für Code, Adressen, Hex, Register und Rasterzellen (`MONO`, `MONO_FETT`).
- Leiter (`muster::schrift`): Titel 20 · Abschnitt 15 (fett) · Karte 13 (fett) · Text 13 · Wert 12 · Hinweis 11. Studiotitel in der Kopfzeile 14 fett.
- Hinweise und Erklärungen immer gedämpft (`mute`), nie kleiner als 11.

## 4. Maße

Alles im 4-px-Raster; Abstände aus `muster::abstand`:

| Name | px | Wofür |
|---|---|---|
| `ENG` | 4 | zwischen Knöpfen einer Gruppe |
| `GRUPPE` | 8 | zwischen Gruppen, Innenrand der Liste |
| `KARTE` | 12 | Innenabstand der Karte |
| `ABSCHNITT` | 16 | zwischen Abschnitten |
| `BEREICH` | 24 / `SEITENRAND` 32 | Seiten (Einstellungen, Start) |

Zonen und Elemente: Kopfzeile 36 · Leiste 38 · Fußzeile 24 · Liste 130 (breite Klasse) · Seitenspalte 290 · **Knopf 28 hoch** · Werkzeugknopf 30 · Radius Knopf 6, Karte 8.

## 5. Studio-Rahmen: sechs Zonen

1. **Kopfzeile**: Art-Chip (SPR, TRK, MAP …), Name, ● ungespeichert, Maße („Frame 0 von 2 · 12×21“), rechts **alle** Datei-Aktionen (Exportieren, Speichern). In der schmalen Klasse nur Name und Speichern.
2. **Leiste**: genau **eine** Reihe, gegliedert durch Trenner: Modi/Werkzeuge · Bearbeiten (↶ ↷) · Transformieren ▾ · Ansicht. Was nicht passt, steht in einem ⋯- oder „Transformieren ▾“-Menü über der Fläche, nie in einer zweiten Reihe.
3. **Liste**: Frames, Kacheln, Muster, Instrumente als Karten mit „+“ im Kopf; Duplizieren und Löschen darunter.
4. **Arbeitsfläche**: eingepasst, Zoom nur ganzzahlig, „Einpassen“ wählt die größte passende Stufe und gilt, bis von Hand gezoomt wird. Leertaste + Ziehen verschiebt, Cmd + Mausrad zoomt (`flaeche::Ansicht`, `Gesten`).
5. **Seitenspalte** aus Karten (feste Breite, Abstand 8): Werkzeug · Dokument · Info (je nach Studio auch weitere, etwa Palette oder Export); einklappbar, Zustand im Modell des Studios; die wichtigsten Karten stehen oben und offen, seltene (Block, Dokument, Info) zu.
6. **Fußzeile**: links Position/Zelle/Zoom/Status, rechts „? Tasten“ (Tooltip mit allen Kürzeln).

**Statusleiste des Fensters (unter allen Zonen):** links „Bereit“/„baut …“, rechts optional (Einstellung „Takt in der Statusleiste“, Standard **aus**) Prozessor · Takt · Norm · Zeilen · Zyklen je Zeile des Zielsystems (`6510 · 0,985 MHz · PAL · 312 Zeilen · 63 Zyklen/Zeile`), beim Debuggen dazu der Programmzähler. Rein lokale Anzeige aus `data/targets/*.target.json`, nichts wird erhoben oder gesendet; nur Werte, die das System sicher hergibt (MEGA65: nur Takt, PET: nur Takt); Monospace, Farbe `mute`; unter 760 px Fensterbreite ausgeblendet.

### Zen-Modus

Cmd/Strg+**Umschalt**+B (Befehl `Zen`, Menü Ansicht): blendet Seitenleiste, Schublade und Panel samt Streifen aus und in jedem Studio Liste (Zone 3) und Seitenspalte (Zone 5); es bleiben Kopfzeile, Leiste, Fläche und Fußzeile. Noch einmal schaltet das vorige Layout (einschließlich Maximierung) wieder her; gespeichert wird nie der Zen-Zustand, sondern das Layout davor; eine Debug-Sitzung beendet Zen. Technisch: `Layout::zen_umschalten` und `muster::zen()` (von `studio_rahmen` ausgewertet). Cmd/Strg+B bleibt die linke Spalte.

### Breitenklassen

Die Schwellen für schmale Fenster (Kopfzeile, Seitenleiste, Panel) stehen in `docs/pruefen.md`, Abschnitt „Schmale Fenster“; dieser Abschnitt nennt die Klassen des Studio-Rahmens.

Nach der Breite des **Dokumentenfelds**, nicht des Fensters (`muster::klasse`): **breit ≥ 900 px** (Liste · Fläche · Seitenspalte), **mittel 640–899** (Liste als Streifen unter der Fläche, ⋯ für Seltenes), **schmal < 640** (eine Spalte, Seitenspalte darunter). Studios mit sehr vielen Werkzeugen (Bank) legen sie unter einer eigenen Schwelle in ⋯.

## 6. Bausteine (`vc-gui-ui::muster`)

| Baustein | Zweck |
|---|---|
| `studio_rahmen(pal, klasse, Studio { … })` | setzt die sechs Zonen zusammen |
| `kopfzeile`, `leiste`, `trenner`, `fusszeile` | Zonen 1, 2, 6 |
| `karte(pal, titel, symbol, unterzeile, einklappen, inhalt)` | Karte: Titel 13 fett, Symbol in Akzent, Unterzeile „gilt für …“ |
| `seitenspalte(klasse, karten)` | Zone 5 |
| `knopf(pal, text, tipp, wirkt, aktiv, primaer)` / `knopf_el` | der einzige Knopf (siehe 7) |
| `segment`, `zeile`, `abschnitt` | Auswahlketten, Beschriftung + Wert, Seitenabschnitte |

## 7. Knöpfe und Zustände

- Höhe 28, innen 10 px, Text 13, Radius 6, Rahmen 1 px (Hoher Kontrast 2 px). Symbolknöpfe quadratisch (`knopf_el(…, quadrat = true)`).
- **Normal** `panel2` mit `line`-Rahmen · **Hover** `line`-Füllung · **Gewählt** `sel`-Füllung mit `accent`-Rahmen (und, wo es um An/Aus geht, ✓ im Text: „Raster ✓“, „Stumm ✓“) · **Nicht verfügbar** `mute`-Text, keine Aktion · **Hauptknopf** (`primaer`) `ok`-Füllung, Schrift `accent_text`; höchstens einer je Leiste („▶ Song“).
- Jeder Knopf hat einen Tooltip mit Wirkung und Kürzel; Tooltip-Text 11.
- Listeneinträge (Muster, Instrumente, Kacheln) sind Karten: Rahmen 1 px, gewählt 2 px in `accent` mit `sel`-Füllung.

## 8. Eingabe und Tastatur

- Kürzel und Befehle stehen in der **Befehlstabelle** (`befehle.rs`); die Palette findet sie. Studio-Einzeltasten (D, E, F, Z S X …) gelten nur bei Studio-Fokus und lassen sich in den Einstellungen abschalten („Einzeltasten der Studios“); Kürzel mit Cmd/Strg/Alt bleiben immer.
- Rechtsklick: Kontextmenü (Sprite, Tracker, Baum; Palette und Disk mit den Aktionen des Studios); in Bildschirm, Kachelsatz und Bank ist die rechte Taste ein Werkzeug („aufnehmen“, „löschen“). **Menü-Taste (oder Umschalt+F10)** öffnet in jedem Studio das Menü mit seinen Aktionen; dieselben Aktionen stehen in der Befehlspalette unter „Studio: …“ (`studioaktionen.rs`, eine Liste je Studio). Vorläufige Entscheidung 2026-11-16.
- **Zen-Modus** Cmd/Strg+Umschalt+B (Abschnitt 5). Klaviertasten des Trackers gelten nach der Tastenposition (Layout nur für die Anzeige, `tastatur.rs`).
- Mausziehen im Baum verschiebt (mit Pfadnachführung); ein Klick ohne Bewegung öffnet.

## 9. Tracker (Besonderheiten)

Drei Modi (Noten · Instrumente · Ablauf, Cmd+1/2/3). **Raster**: Spalten elastisch 112–200 px (viele Kanäle ab 84), Zeilennummern dezimal, kräftig gefüllter Cursor, Zeilenband über die ganze Breite (Taktband alle 4 Zeilen, Cursorzeile leicht in `accent`), ▶ an der gespielten Zeile; Kanalkopf „Kanal 1 · Puls“. **Seitenspalte (Noten)**: Werkzeug (Instrumente mit ▶) · Klaviatur (einklappbar, auch Eingabegerät) · Kanäle (Solo/Stumm ✓) · Pegel · Block · Dokument. **Instrumente**: Klang-Editor eingepasst, keine Klaviatur quer. **Ablauf**: Reihenfolge als Kartenkette (↺ Wiederholung, ▶ gespielt).

## 10. Dateibaum

Verzeichnisbaum, kein vorgegebenes Projektschema. Zeilen: Marke (3 Buchstaben, Farbe nach Gruppe), Name, ★ Startprogramm, ● ungespeichert; aktiv = kräftig (Füllung und Akzentbalken), geöffnet = leichte Füllung und Punkt. Ordner mit geöffneten Dateien zeigen ●. Aktionen im Kontextmenü: Neu, Umbenennen (nur der Name), Löschen, Als Hex ansehen, Im Ordner zeigen.

## 11. Neues Studio: Checkliste

1. `studio_rahmen` mit allen sechs Zonen; Leiste in **einer** Reihe, Seltenes in ⋯.
2. Alle Knöpfe über `muster::knopf`; keine eigenen Stile, keine festen Farben.
3. Karten: Werkzeug (nicht in der Datei) und Dokument (wird gespeichert) getrennt, Zustand „zu“ im Modell, nicht im Inhalt.
4. Breitenklassen mit `iced::widget::responsive`; schmal: Kopfzeile nur Name und Speichern.
5. Befehle in `befehle.rs`, Kürzel im Tooltip, Einzeltasten über `studio_taste_erlaubt`.
6. Tests: Eintrag in `galerie_zustaende`, `layout_pruefung` (keine abgeschnittenen oder unsichtbaren Elemente in 1280×800, 900×700, 720×800), Bedienung mit echten Mausereignissen, Handprüfliste (lokal beim Projektinhaber, nicht im Repository).
7. **Zen-Modus unterstützen:** automatisch, wenn das Studio `studio_rahmen` benutzt (es zeigt dann nur die Fläche); nichts, was zum Arbeiten nötig ist, darf nur in Liste oder Seitenspalte stehen (Kürzel und Befehlspalette müssen es erreichen). Aktionen des Studios als Liste in `studioaktionen.rs` eintragen.
8. Vergleich mit Mockup und Muster: Bild `studio_vergleich` (`VC_ZUSTAENDE`, `VC_VERGLEICH`) vorher/nachher.

## 12. Wo steht was

- Code: `crates/vc-gui-ui/src/{muster,theme,fonts,flaeche,befehle,kontext,bereich,texte}.rs` (Bereiche und feste Texte der Oberfläche in `bereich` und `texte`), Studios in `crates/vc-gui/src/*studio.rs`; Wegweiser durch alle Dateien: [docs/architektur.md](docs/architektur.md).
- Konzepte (lokal, nicht im Repository): `Konzept-UX-Ueberarbeitung.md`, `Konzept-Neue-GUI-Iced.md` (Entscheidungen 24–27), Mockups in `assets/docs/mockups/ux-ueberarbeitung/`.
- Prüfungen: `cargo test -p vc-gui` (Galerie, Layout, Farbprüfung, Bildvergleich gegen die Referenzbilder unter `crates/vc-gui/tests/ref/`; sie liegen nur lokal beim Projektinhaber, nicht im Repository, und fehlende werden übersprungen).
