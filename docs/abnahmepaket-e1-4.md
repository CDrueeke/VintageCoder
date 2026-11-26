# Abnahmepaket E1.4 (Rahmen)

Stand: 2026-11-21 · Branch `claude/ui-e1-4` (gebaut aus `claude/ui-e1-3`) · nichts in `main`, nichts auf GitHub außer dem privaten Branch.

## Inhalt

- **Sitzung** je Projekt (`session.json` wie in der alten Oberfläche, Reihenfolge bleibt, aktive Datei vorn), Schalter `restore_tabs` in der `.vcprj` (Standard aus).
- **Zwischenkopie unbenannter Reiter** (`scratch.json`, 14 Tage, abschaltbar im Kern) mit Dialog beim nächsten Start; Verwerfen beim Beenden löscht sie.
- **Beenden** (⌘Q, Fensterknopf, Projekt schließen/öffnen) mit Liste der ungespeicherten Reiter.
- **Layout** merken und zurücksetzen, Größen per Trennlinie (Maus, Tastatur), **Maximieren** (Kopfknopf, Doppelklick, Befehle, ⌘⇧Enter), Seitenleistenstreifen mit Symbolen, Ablage im Zen eingeklappt.
- **Geteilte Ansicht**: zwei Reitergruppen (⌘⌥T, in die andere Hälfte, Ziehen an den rechten Rand), unter 1000 Punkten untereinander.
- **Breitenklassen**: schmal legt Seitenleiste und Panel über die Fläche.
- **Neues Projekt** (Dialog, Kern-Port von `vc-gui/src/neuprojekt.rs`), **Startseite** (Zuletzt: anheften, entfernen, Hinweis „Ordner fehlt“; Beispiele öffnen/kopieren; erkannte Emulatoren; Tastenkürzel), **Projekt kopieren**, Projektdatei wählen, zuletzt geöffnete Projekte in der Befehlspalette.
- **Fenster**: Größe und Lage merken, kleiner Bildschirm startet maximiert, abgestecktes Zweitgerät fängt auf.
- **Bedienhilfen**: Seitensprache, Namen der Bereiche, Trennlinien als `separator` mit Wert.

## Prüfungen (lokal)

`ui/tools/e2e-alle.sh` fährt alle Ablauftests nacheinander gegen den echten Kern (Prüfbrücke, unsichtbar): editor 82, lsp 74, bau 50, layout 44, split 24, sitzung 17, start 23, neuprojekt 22. Dazu Vitest (Layoutregeln, Gruppenlogik), Rust-Tests (`sitzung`, `neuprojekt`, `start`, `fenster`).

## Nicht geprüft / Hinweise

- Fenstergröße und -lage (Punkt 26/27 der Handprüfliste): die Entscheidung ist als reine Rechnung getestet, das Anwenden im echten Fenster braucht die Handprüfung (die Prüfbrücke hat ein verstecktes Fenster).
- Bildschirmleser: Namen und Rollen sind geprüft, der Test mit VoiceOver steht aus.
- Startbild entfällt vorerst (Startseite); Banner/Feedback/Assistent folgen mit E12.
