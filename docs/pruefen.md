# Automatische Sichtprüfung der neuen Oberfläche

Die neue Oberfläche (`vc-gui`) lässt sich ohne Fenster prüfen: Die Ansichten werden mit dem Software-Renderer (tiny-skia) in Bilder gerechnet, der Widget-Baum wird ausgemessen, Farben und Klang werden mit Kennwerten geprüft. Es wird dabei nie ein Fenster geöffnet, nie Ton ausgegeben und nie ein fremdes Programm gestartet.

| Was | Befehl | Läuft in der CI | Dauer |
|---|---|---|---|
| Galerie aller Ansichten | `cargo test -p vc-gui galerie -- --ignored --nocapture` | nein (ignoriert) | ~45 s |
| Farbkanäle (R/B nicht vertauscht) | `cargo test -p vc-gui bildfarben` | ja | < 2 s |
| Layout-Prüfung | `cargo test -p vc-gui layout_pruefung` | ja | ~4 s |
| Bildvergleich mit Referenzen | `cargo test -p vc-gui bildvergleich` | ja (die sechs Referenzen liegen nur lokal unter `crates/vc-gui/tests/ref/`, nicht im Repository; fehlt eine, wird die Ansicht übersprungen) | ~3 s |
| Kontraste (3 Paletten) | `cargo test -p vc-gui farbkontraste` | ja | < 1 s |
| Kopfzeile in jeder Breite | `cargo test -p vc-gui kopfzeile_erreichbar` | ja | ~2 s |
| Klang | `cargo test -p vc-ide-model --test klang` | ja | < 1 s |

## Galerie

`galerie` rendert alle Zustände (Studios, Editor mit Suche und Menü, Debug-Fläche, Karte, Dialoge, geteiltes Fenster, Leerzustand …) in Dunkel, Hell und Hoher Kontrast und in drei Fenstergrößen (1280×800, 900×700, 720×800) nach `build/galerie/`. Öffne `build/galerie/index.html` im Browser: Das ist der Kontaktbogen, ein Klick zeigt das Bild in voller Größe. Neue Ansichten trägst du in `galerie_zustaende()` (in `crates/vc-gui/src/main.rs`, Testteil) ein: Name und eine Funktion, die die App in den gewünschten Zustand bringt.

## Layout-Prüfung

`layout_pruefung` misst für jede Ansicht in den drei Größen den Widget-Baum (Iced-`Operation`) und meldet: Text mit Breite 0 (unsichtbar), kurze Texte, die umbrechen (Knopfbeschriftungen), Texte, die aus ihrem Behälter ragen, Elemente außerhalb des Fensters, sich überlappende Texte (nur ohne Überlagerung durch Dialog oder Menü) und zu kleine Klickflächen bei Eingabefeldern. Die Prüfung ist eine Heuristik: Sie kennt keine Schriftmessung außerhalb dessen, was das Layout schon berechnet hat, und sieht Knöpfe nur als Behälter mit Text.

Der Stand der bekannten Befunde steht in `crates/vc-gui/tests/layout_bekannt.txt`. Der Test schlägt **nur bei neuen Befunden** fehl; behobene Befunde werden gemeldet. Nach dem Beheben die Liste neu schreiben (nur nach Absprache, damit nichts Neues versehentlich als „bekannt“ gilt):

```bash
VC_UPDATE_REFS=1 cargo test -p vc-gui layout_pruefung
```

## Bildvergleich

`bildvergleich_mit_referenzbildern` vergleicht ausgewählte Ansichten (Dunkel, 1280×800) mit Referenzbildern in `crates/vc-gui/tests/ref/`. Es zählt der Anteil der Pixel, die in einem Farbkanal um mehr als 24 abweichen; mehr als 0,5 % ist ein Fehler. Bei einem Fehler legt der Test `build/bildvergleich/<name>-ist.png` und `-diff.png` (abweichende Pixel in Magenta) ab. Fehlt eine Referenz, wird die Ansicht übersprungen.

Die Referenzen sind auf macOS entstanden; die Schrift wird je Betriebssystem etwas anders gerastert, auf anderen Systemen kann der Vergleich deshalb abweichen (die Schwelle von 0,5 % fängt kleine Unterschiede meist ab). Referenzen werden **nur nach Freigabe** angelegt oder erneuert, weil sie das gewollte Aussehen festschreiben und das Repository vergrößern (rund 0,3 MB je Bild):

```bash
VC_UPDATE_REFS=1 cargo test -p vc-gui bildvergleich
```

## Kontrast

`farbkontraste_aller_rollen_in_allen_paletten` rechnet das WCAG-Kontrastverhältnis aller Text- und Flächenfarben (Schrift, Zweitschrift, Akzent, Erfolg, Warnung, Fehler, Syntaxfarben, Knopfschrift) in drei Paletten: Dunkel und Hell gegen 4,5:1, **Hoher Kontrast** gegen 7:1 (dazu Rahmenlinien gegen 3:1). Es gibt keine Ausnahmeliste: jede Unterschreitung lässt den Test fehlschlagen. „Hoher Kontrast“ wählst du in den Einstellungen (Farbschema) oder mit dem Befehl „Hoher Kontrast ein/aus“; die Systemeinstellung für hohen Kontrast wird noch nicht übernommen (Iced meldet nur Hell/Dunkel).

## Klang

`crates/vc-ide-model/tests/klang.rs` rechnet die Beispiel-Songs (C64, Atari, MEGA65) über den Sequenzer `synth::Engine` in einen Puffer: nicht still, nicht übersteuert, Effektivwert je halbe Sekunde gegen `klang_ref.json`. Dazu die Grundfrequenz einzelner Töne gegen die Tonhöhentabelle (2 %) und Stille ohne Noten bzw. nach dem Notenende. Die Referenz erneuerst du mit `VC_UPDATE_REFS=1 cargo test -p vc-ide-model --test klang`.

## Regeln

- Tests schreiben nie nach `examples/`: Beispielprojekte werden in temporäre Kopien kopiert.
- Referenzen (Bilder, Klang, Befundlisten) nur mit ausdrücklichem `VC_UPDATE_REFS=1` ändern und die Änderung im Commit begründen.

## Schmale Fenster: was heute gilt (720–900 px)

* **Kopfzeile (Überlaufregel, `kopf_stufe`)**: ab 1180 px alles ausgeschrieben; ab 960 px Starten/Debuggen/Bauen nur als Symbol mit Tooltip (Kürzel steht darin), Suchfeld als „Suchen ⌘K“; ab 640 px das Suchfeld als Lupe (öffnet die Palette), Ziel-Chip als Kurzname, Wortmarke als Symbol; darunter wandern Debuggen und Bauen in ein „⋯“-Menü mit denselben Einträgen und Kürzeln. „Starten“ und „⚙“ bleiben immer sichtbar; der Projektname wird mit „…“ gekürzt. Im Debuggen zeigt die Kopfzeile ab 960 px die Symbolgruppen, der Zustand steht im Tooltip. Der Test `kopfzeile_erreichbar_in_jeder_breite` prüft 480–1600 px.
* **Seitenleiste**: unter 800 px bleibt von ihr der Streifen (Maximieren zeigt sie weiter).
* **Panel**: unter 900 px liegt es als Überlagerung rechts über der Mitte.
* **Studio-Leisten, Seitenspalten, Suchleiste, Panel- und Schubladenreiter** brechen um statt abzuschneiden; die Tracker-Leisten (Block, Muster, Kanäle, Tastenhilfe) lassen sich einklappen („Leisten ▾“) und sind unter 780 px Höhe vorgabemäßig eingeklappt; das Disk-Studio scrollt Verzeichnis und Belegung.
* **Bekannte Grenzfälle** (`layout_bekannt.txt`; die Datei ist zurzeit leer, es gibt keine bekannten Befunde): zwei Pixel-Studios nebeneinander (geteiltes Dokumentenfeld) brauchen etwa 1000 px; darunter wird die Zeichenfläche eines Studios schmal und Beschriftungen der Seitenspalte brechen um. Ob unter 900 px untereinander geteilt oder das Teilen gar nicht angeboten wird, entscheidet Konzept 12.
