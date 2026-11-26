# Oberfläche anpassen

Das Fenster hat ein **festes Gerüst**: Kopfzeile mit den Hauptbereichen, darunter die zweite Reihe, dann Arbeitsfläche und Statusleiste. Im Bereich **Code** kommen die Seitenleiste, die Ablage unter dem Editor und das Panel Maschine dazu. Du ziehst die Größen, klappst Bereiche ein, machst einen groß oder blendest alles bis auf die Arbeitsfläche aus.

![Das Hauptfenster im Bereich „Code“. Oben die Kopfzeile mit dem Projektnamen und den Hauptbereichen von „Startseite“ bis „Debug“. Links die Seitenleiste „PROJEKT“ mit dem Dateibaum, in der Mitte der Editor mit jumprun.asm, darunter die Ablage mit dem Reiter „Ausgabe“, rechts das Panel „Maschine“, ganz unten die Statusleiste.](bild:ui-gesamt "Das Hauptfenster im Überblick: Kopfzeile, Seitenleiste, Editor, Ablage, Panel Maschine und Statusleiste.")

| Zone | Inhalt | Ein-/Ausblenden |
|---|---|---|
| Kopfzeile | Knopf für die Seitenleiste, Projektname, die Hauptbereiche (Projekt, Code, Grafik, Daten, ggf. Musik und Debug), Bauen, Starten, Beenden, Startziel, Sprache, Farbschema, Befehlspalette | immer da |
| Zweite Reihe | die Unterbereiche des gewählten Hauptbereichs (zum Beispiel bei Grafik: Sprites, Zeichensatz, Palette) | immer da |
| Seitenleiste „Projekt“ | Dateien (Projektbaum) | Cmd/Strg+B |
| Dokumentenfeld | die Datei oder der Editor des gewählten Bereichs; im Bereich Code alle Dateien als Reiter | immer da |
| Ablage unter dem Editor | Probleme, Ausgabe, Tests, Suche, Gliederung | Cmd/Strg+J |
| Panel „Maschine“ | Startziel, Zustand und Ergebnis des letzten Baus | Cmd/Strg+Alt+B |
| Statusleiste | Projekt, Sprache der Datei, Zeilenende, Probleme, Zeile und Spalte, Ergebnis des Baus; rechts der Hinweis „Alle Befehle“ | immer da |

Seitenleiste, Ablage und Panel Maschine gibt es nur im Bereich **Code**. Ist die Seitenleiste zu, bleibt ein schmaler Streifen mit den Knöpfen für Dateien und Suche.

Mit **Ansicht ▸ Bereiche ▸ Bereich Projekt** (Cmd/Strg+Alt+0) springst du in den Hauptbereich Projekt.

## Zen-Modus
**Cmd/Strg+Alt+Z** blendet Seitenleiste, Ablage, Panel Maschine und Statusleiste aus. Es bleiben Kopfzeile, zweite Reihe und die Arbeitsfläche. Derselbe Befehl bringt alles zurück.

## Bereiche groß machen
Jeder Bereich hat oben einen Knopf **Bereich maximieren**. Maximieren klappt die anderen zu; **Bereich wiederherstellen** oder ein Doppelklick auf die Kopfzeile der Ablage bringt die Aufteilung zurück. **Cmd/Strg+Umschalt+Enter** schaltet den Bereich mit dem Fokus um. Für jeden Bereich gibt es auch einen eigenen Befehl in **Ansicht ▸ Bereiche** und in der Befehlspalette.

## Größen ändern
Die Trennlinien zwischen den Bereichen lassen sich ziehen oder mit den Pfeiltasten verstellen (Umschalt für große Schritte, Pos1 und Ende für kleinste und größte Größe). Ein Doppelklick stellt die Vorgabe her.

## Schmale Fenster
Wird die Arbeitsfläche sehr schmal, schweben Seitenleiste und Panel über dem Dokument. Die Editoren zeigen dann weniger nebeneinander und ordnen ihre Karten untereinander an.

**Ansicht ▸ Bereiche ▸ Layout zurücksetzen** (Mac-Menü; sonst die Befehlspalette) stellt die Standardgrößen wieder her. Größen und Sichtbarkeit merkt sich VintageCoder selbst; ein maximierter Bereich wird nicht gemerkt.

Siehe auch [Einstellungen](help:settings), [Tastenkürzel](help:shortcuts) und [Dateibaum](help:filetree).
