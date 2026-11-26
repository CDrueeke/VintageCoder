# Handprüfung E8 Kachel-Editor

Start: `start-kacheln.command` im Paket (öffnet `arbeitskopie/gfx_demo`, Datei `graphics/level_tiles.vctiles`, C64) und `start-kacheln-atari.command` (Atari, `alevel_tiles.vctiles`).

| # | Schritte | Erwartet | Ergebnis |
|---|---|---|---|
| 1 | Datei öffnen (Startskript) | Grafik ▸ **Kacheln**; links der Kachelsatz (Vorschau, Name, Verwendungszähler „n×“), in der Mitte die große Kachel und die Zeichentafel, rechts die Karte der Kachel | |
| 2 | Kachel anklicken; Pfeiltasten, Pos1/Ende | Die Kachel wird gewählt, die große Kachel zeigt sie; Pfeile wechseln die Kachel | |
| 3 | Eine Zelle anklicken, dann ein Zeichen in der Tafel anklicken | Das Zeichen steht in der Zelle und in der Kachelliste; der Satz ist „geändert“ | |
| 4 | Über mehrere Zellen ziehen (Maustaste gedrückt) | Alle überstrichenen Zellen bekommen das Zeichen; ein Rückgängig nimmt den ganzen Zug zurück | |
| 5 | Alt+Klick auf eine Zelle (Pipette); Pfeiltasten in den Zellen, Enter, I | Zeichen wird aufgenommen; Cursor bewegt sich; Enter setzt, I nimmt auf | |
| 6 | Spiegeln, 90°-Drehen (nur quadratisch), Rollen (4 Pfeile), Leeren, Kopieren, Einfügen | Wirkt auf die ganze Kachel, jeweils rückgängig machbar | |
| 7 | Name ändern; Farbe der Kachel wählen; Hintergrundfarbe | Name erscheint in der Liste; Farben folgen sofort | |
| 8 | „Je Zelle“ wählen | Der Pinselmodus (Beides, Nur Zeichen, Nur Farbe) und die Pinselfarbe erscheinen | |
| 9 | Eigenschaften ein- und ausschalten; neue Eigenschaft anlegen | Wird in der Datei gespeichert (nach Speichern) | |
| 10 | Größe der Kacheln ändern (größer; kleiner mit belegten Zellen) | Größer ohne Rückfrage; kleiner fragt („gehen belegte Zellen verloren“); Abbrechen lässt alles | |
| 11 | + (neue Kachel), Duplizieren, Zusammenführen, Löschen einer benutzten Kachel | Duplikat wird als „=“ markiert; Zusammenführen entfernt es; Löschen einer in Bildschirmbanken benutzten Kachel fragt nach und schreibt die Banken um | |
| 12 | Zielzeichensatz: Füllstand, zweiten Zeichensatz hinzufügen/entfernen | Balken „n von 256 belegt · m frei“; ein benutzter Zeichensatz lässt sich nicht entfernen | |
| 13 | Export …, Speichern (Cmd/Strg+S) | Export-Dialog mit Vorschau; Speichern schreibt die Datei | |
| 14 | **Tastatur und VoiceOver:** Liste, Zellen und Tafel mit Tab/Pfeilen | Ansage je Kachel („Kachel n, Name, n mal verwendet“), Zelle („Zelle x, y: Zeichen n, $hh“), Zeichen („Zeichen n, $hh, n von 256“) | |

Nicht in E8: Kartenbearbeitung (E10, nach 0.3.1), Zoom der großen Kachel (passt sich der Breite an).
