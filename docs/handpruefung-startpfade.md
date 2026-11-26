# Handprüfung: Start mit Pfaden

| # | Schritte | Erwartet | Ergebnis |
|---|---|---|---|
| 1 | macOS: `open -a VintageCoder.app <Projektordner>` (App nicht gestartet) | Das Projekt öffnet direkt, keine Startseite | |
| 2 | Bei laufender App: `open -a VintageCoder.app <Datei im Projekt>` | Die laufende App öffnet die Datei als Reiter (keine zweite Instanz) | |
| 3 | `open -a VintageCoder.app <Diskettenabbild>`; `… <.vcsprite außerhalb des Projekts>` | Abbild öffnet im Disk-Editor; die Sprite-Datei fragt „Ins Projekt kopieren und öffnen?“ (wie beim Ziehen) | |
| 4 | Doppelklick auf eine `.vcprj` oder `.vcsprite` im Finder | VintageCoder öffnet das Projekt bzw. die Datei | |
| 5 | Kommandozeile: `VintageCoder.app/Contents/MacOS/vintagecoder <Projekt> --open <Datei> --view disk` (Windows/Linux entsprechend `VintageCoder <Projekt> …`) | Projekt offen, die Datei als Reiter, danach die Ansicht Daten ▸ Disk | |
| 6 | Windows/Linux (und direkter Aufruf auf macOS): zweiter Aufruf `VintageCoder <Pfad>` bei laufender App | Es startet keine zweite Instanz; die laufende App öffnet den Pfad und kommt nach vorn | |

Pfade und Schalter: `<pfad> […]` (Ordner oder `.vcprj` = Projekt; Dateien wie beim Ziehen), `--open <datei>` (Datei im angegebenen Projekt), `--view <sprites|disk|<bereich>[:<unterseite>]>`. Dateizuordnungen: `.vcprj`, `.vcsprite`, `.vcpmg`, `.vcchar`, `.vctiles`, `.vcsong`, `.vcscreen`, `.vcpal`.

- Windows: Strg+S von Hand einmal mit per Doppelklick im Projektbaum geöffnetem Reiter prüfen (Punkt erscheint nach Tippen, Strg+S schreibt die Datei).
