# Handprüfung E3: Sprite-Editor

Spalten: Nr | Schritt | Erwartung | Ergebnis. Sprite-Dateien nur in **Kopien** prüfen. Der Sprite-Editor steht unter **Grafik ▸ Sprites**. Gegliedert nach Teilschritten; ein Teilschritt kommt erst in das Paket, wenn er gebaut ist.

## E3.1 Grundlage und Zeichenfläche

**Zu wissen:** Die Werkzeuge Linie, Rahmen, Füllen, Auswahl, Spiegeln und Verschieben (E3.2), Bilderliste, Modus, Flags und Onion Skin (E3.3) sowie Animation, Export und PNG (E3.4) folgen. Die Einzeltasten wirken nur, wenn die Zeichenfläche den Fokus hat.

| Nr | Schritt | Erwartung | Ergebnis |
|---|---|---|---|
| 1 | Grafik ▸ Sprites ohne geöffnete Datei | Hinweis „Kein Sprite geöffnet“ mit der Liste der Sprite-Dateien des Projekts und „Sprite-Datei öffnen …“ | |
| 2 | Eine Datei der Liste wählen; zusätzlich im Projektbaum eine `.vcsprite` anklicken | Öffnet im Sprite-Editor (kein Textreiter); Fußzeile nennt Namen, Maße und „Bild 1 von n“ | |
| 3 | Mit der Maus malen (Stift), mit großen schnellen Zügen | Der Strich erscheint sofort unter der Maus und ist ohne Lücken | |
| 4 | Nach dem Strich Rückgängig (Knopf, ⌘Z, Menü) | Ein Schritt nimmt den ganzen Strich zurück; Wiederholen stellt ihn her | |
| 5 | Rechte Maustaste auf einen gemalten Punkt | Der Punkt wird gelöscht (kein Menü des Browsers) | |
| 6 | Radierer (E), Pipette (I): Pipette auf einen gemalten Punkt | Radierer löscht; die Pipette wählt den Farbwert des Punkts | |
| 7 | Farbwerte per Ziffer 1 bis 4 und per Klick auf die Chips rechts | Gewählter Wert ist hervorgehoben; durchsichtiger Wert zeigt Schachbrett; Hinweis „Dieser Wert ist durchsichtig“ statt Palette | |
| 8 | Palettenfeld anklicken (Multicolor: auch Multicolor 1 und 2 über deren Werte) | Die Farbe der Rolle ändert sich im Bild; Datei ist „ungespeichert“; Rückgängig stellt die Farbe her | |
| 9 | Palettenansicht „Projekt / PAL / NTSC“ (nur bei Atari-Dateien; bei C64 gibt es keine Wahl, weil die Palette gleich ist) | Palette wechselt; die Datei bleibt unverändert (nur Ansicht) | |
| 10 | Vergrößern, Verkleinern, Einpassen, Taste +, − und 0, ⌘/Strg + Mausrad | Zoom in ganzen Stufen von 100 % bis 3200 %; Anzeige „2200 %“ o. ä. | |
| 11 | Hohe Zoomstufe (1600 % und mehr) malen und Raster | Kein Ruckeln, Raster scharf; mit G ein- und ausschalten; kräftigere Linie je Byte | |
| 12 | Leertaste halten und ziehen (oder mittlere Maustaste) | Fläche verschiebt sich, es wird nichts gemalt | |
| 13 | Maus über die Fläche | Rechts unten „x … y …“ der Zelle unter der Maus | |
| 14 | **Ohne Maus:** Tab bis zur Fläche, Pfeiltasten, Eingabetaste, Entf, Ziffern, D/E/I | Zellmarke bewegt sich; Eingabetaste setzt, Entf löscht; Ansage „Zelle x, y: Wert“ (VoiceOver: Prüfung in E13) | |
| 15 | Speichern (Knopf, ⌘S) | „ungespeichert“ und Punkt in der Dateiliste verschwinden; Datei auf der Platte ist geändert und lässt sich neu öffnen, Pixel und Farben wie gespeichert | |
| 16 | Datei öffnen, nichts ändern, speichern, noch einmal speichern; Dateien vergleichen (`diff`) | Byte-gleich | |
| 17 | Mit Änderungen eine andere Sprite-Datei wählen | Rückfrage Speichern / Verwerfen / Abbrechen | |
| 18 | Mit Änderungen Projekt schließen oder ⌘Q | Beenden-Liste nennt die Sprite-Datei | |
| 19 | Hell und Dunkel; Englisch | Fläche, Chips, Palette lesbar; Texte übersetzt | |
| 20 | Eine Atari-Datei (`.vcpmg`) und eine MEGA65-Datei öffnen | Öffnen und Malen funktioniert; Farbwerte stimmen mit dem Ziel | |

## E3.2 Werkzeuge

**Zu wissen:** „Alles auswählen“ im Menü Bearbeiten wirkt im Sprite-Editor nicht (das Menü gehört dem System); stattdessen Befehlspalette ⌘K „Alles auswählen“ oder das Menü der Auswahl (Rechtsklick). Kopieren, Ausschneiden und Einfügen (⌘C, ⌘X, ⌘V) wirken bei fokussierter Zeichenfläche.

| Nr | Schritt | Erwartung | Ergebnis |
|---|---|---|---|
| 21 | Linie (L): Anfang ziehen bis Ende, mit der rechten Taste | Vorschau der Linie schon beim Ziehen; beim Loslassen gezeichnet; mit rechter Taste in Wert 0 (löscht) | |
| 22 | Rahmen (R) ziehen | Nur der Rand, innen unverändert; ein Rückgängig nimmt ihn zurück | |
| 23 | Füllen (F) in eine zusammenhängende Fläche und in eine durchsichtige | Fläche mit dem gewählten Wert gefüllt, Rand bleibt | |
| 24 | Auswahl (S) aufziehen; einzelnen Klick ohne Ziehen | Gestrichelter Rahmen, Fußzeile „Auswahl 3 × 3“; Klick ohne Ziehen hebt die Auswahl auf | |
| 25 | Kopieren, Einfügen an anderer Stelle (Maus dorthin), Ausschneiden | Inhalt erscheint an der Zelle unter der Maus; Ausschneiden leert den Ursprung | |
| 26 | Rechtsklick bei Werkzeug Auswahl | Menü: Kopieren, Ausschneiden, Einfügen, Alles auswählen, Auswahl löschen, Auswahl aufheben | |
| 27 | Escape bei bestehender Auswahl; Entf bei Auswahl | Auswahl aufgehoben; Entf löscht den Inhalt der Auswahl | |
| 28 | **Ohne Maus:** Linie wählen (L), Zellmarke bewegen, Eingabetaste (Anfang), Pfeile, Eingabetaste (Ende); Escape bricht ab | Ansage „Anfang bei x, y gesetzt …“; Linie mit Vorschau; ebenso Rahmen und Auswahl; Füllen mit Eingabetaste | |
| 29 | Waagerecht (X) und senkrecht (Y) spiegeln; Verschieben nach links, rechts, oben, unten; Bild leeren | Pixel gespiegelt bzw. umlaufend verschoben; jeder Schritt einzeln rückgängig | |

## E3.3 Bilder, Modus, Flags, Onion Skin, Vorschau im Spiel

**Zu wissen:** Hardware-Flags (doppelt breit/hoch, hinter dem Hintergrund) gibt es bei C64, C128 und MEGA65 (VIC-II-Modus). Weitere MEGA65- und Atari-Eigenschaften folgen nach Entscheidung des Inhabers. Bild wechseln mit **Alt+Pfeil links/rechts** oder **Bild-auf/Bild-ab** (die Pfeiltasten allein bewegen die Zellmarke). „Vorschau im Spiel“ hat einen festen Hintergrund (Himmel und Wiese); ein Bildschirm aus dem Projekt kommt später.

| Nr | Schritt | Erwartung | Ergebnis |
|---|---|---|---|
| 30 | Bilderliste links bei einer Datei mit mehreren Bildern | Je Bild eine Vorschau mit Nummer; das gewählte ist hervorgehoben; Anzahl in der Überschrift | |
| 31 | „Neu“, „Duplizieren“, „Löschen“ | Neues leeres Bild wird angehängt und gewählt; Duplikat nach dem gewählten; Löschen entfernt das gewählte (das letzte Bild bleibt); jeder Schritt per Rückgängig umkehrbar | |
| 32 | Bild per Klick, Alt+Pfeil und Bild-auf/ab wechseln | Fußzeile „Bild n von m“; VoiceOver-Ansage in E13 | |
| 33 | Schalter „Mehrfarbig“ (Sprite) | Bild wird auf 24 Punkte hires bzw. 12 Punkte Multicolor umgerechnet, Farbwerte folgen; Rückgängig stellt her; zurückschalten gibt das Ausgangsbild | |
| 34 | „Doppelt breit“, „Doppelt hoch“, „Hinter dem Hintergrund“ | Hinweis „Im Spiel auf 48 / 42 Punkte“; in „Vorschau im Spiel“ wird der Sprite entsprechend größer; Datei ist ungespeichert; speichern und neu öffnen: Schalter stehen wie gesetzt | |
| 35 | Onion Skin (Knopf oder O) bei Bild 2 oder später | Vorheriges Bild blass dahinter, nur wo das gewählte durchsichtig ist; beim ersten Bild ausgegraut; Schreibweise „Onion Skin“ in beiden Sprachen | |
| 36 | Vorschau im Spiel, Karte „Speicher“ | Sprite zweimal (vergrößert und original) vor Himmel/Wiese in Systemfarben; „64 Byte je Bild · n Bilder · … Byte“ | |
| 37 | Atari-Datei (`.vcpmg`) öffnen | Feste Bilder P0–P3: keine Schaltflächen Neu/Duplizieren/Löschen; keine Flags | |

## E3.4 Animation, PNG, Export, neue Datei

**Zu wissen:** Beim Laden eines PNG speichert der Kern die Datei zuerst; danach ist ein Rückgängig möglich. Export: „Als Include einfügen“ bindet die erzeugte Datei in den Quelltext ein (`.include` bzw. bei BASIC `#include`). „In Quelltext kopieren“ schreibt den erzeugten Text selbst an die Cursorzeile, ohne Include. BASIC DATA kann eine Einleseroutine mitliefern (BASIC V2 kennt kein `RESTORE zeile`: die Routine steht deshalb vor den DATA-Zeilen und liest ab dem aktuellen DATA-Zeiger).

| Nr | Schritt | Erwartung | Ergebnis |
|---|---|---|---|
| 38 | Reiter „Animationen“ links, „Neu“, umbenennen (z. B. „laufen“; auch einen ungültigen Namen wie „1 falsch“) | Animation mit dem gewählten Bild als erstem Schritt; ungültiger Name wird mit Meldung abgelehnt | |
| 39 | Schritte: „Gewähltes Bild anhängen“, Dauer in Bildern (1/50 s) ändern, nach oben/unten, entfernen | Liste und Zeitleiste folgen; jeder Schritt per Rückgängig umkehrbar | |
| 40 | Ablauf: Schleife, Hin und zurück, Einmal (Reiter und Knopf in der Zeitleiste) | Wechselt; Dauer in der Zeitleiste „n Bilder · x s“ stimmt | |
| 41 | Abspielen und Anhalten in der Zeitleiste | Das laufende Schritt-Bild ist hervorgehoben, die kleine Vorschau zeigt das Bild; Tempo stimmt mit der Dauer überein (50 Bilder je Sekunde) | |
| 42 | Schritt in der Zeitleiste anklicken | Wählt das Bild des Schritts | |
| 43 | „PNG laden …“ mit einem PNG im passenden Format | Das gewählte Bild zeigt das PNG; Meldung nennt Farben; Rückgängig stellt das Bild her; Datei wurde dabei gespeichert | |
| 44 | Export (Knopf „Exportieren …“, ⌘⇧E): Formate Assembler, Konstanten, Binär, PRG, BASIC DATA | Vorschau je Format; PRG und Konstanten verlangen eine gültige Adresse (Fehler im Status, Speichern gesperrt); Hex-Reiter bei Binärformaten | |
| 45 | „In Projekt speichern“ | Datei unter `build/export/` im Projektbaum; Meldung nennt den Pfad | |
| 46 | Einen Assembler-Quelltext öffnen, Export, „Als Include einfügen“ | Zeile `.include "…/build/export/….asm"` steht an der Cursorzeile; ein zweites Mal: Ablehnung „schon eingebunden“ | |
| 47 | Derselbe Export, „In Quelltext kopieren“ | Der erzeugte Assembler-Text steht im Quelltext (ohne Include); Dialog schließt; Code-Ansicht öffnet | |
| 48 | BASIC-Quelltext öffnen, Format „BASIC DATA“, mit und ohne „Einleseroutine mitliefern“ (Adresse z. B. `$2000`), „In Quelltext kopieren“ und „Als Include einfügen“ | DATA-Zeilen, mit Routine `GOSUB @name_load` und POKE-Schleife; Include `#include "…data.bas"`; Ziel nur BASIC-Quelltexte | |
| 49 | Export ohne passenden geöffneten Quelltext | Hinweis „Kein passender Quelltext ist geöffnet“; Einfügen gesperrt, Speichern bleibt möglich | |
| 50 | „Im Emulator testen“ | Speichert als Assembler, baut und startet (nur mit eingerichtetem Emulator prüfbar) | |
| 51 | „Neue Sprite-Datei …“: Profil, Name, Ordner | Ungültiger Name gesperrt; Datei erscheint im Projektbaum und öffnet leer im Editor; vorhandener Name wird abgelehnt | |
| 52 | **Rundlauf:** Alles aus E3 mit einer Datei durchspielen, speichern, Datei im Textbetrachter öffnen und mit dem Ausgangsstand vergleichen | Nur die gewollten Änderungen; Formatierung der Datei unverändert (Byte-gleich nach Speichern ohne Änderung) | |

## E3.5 Atari Player/Missile: Breite und Priorität (Kernänderung)

**Kernänderung (additiv):** `vc-ide-model` (`pmg_from_hw`, `pmg_to_hw`) und `vc-media` (`sprite::export_consts`): Player/Missile-Dateien können in `hw` die Breite je Spieler (`sizep`: 0 normal, 1 doppelt, 3 vierfach, Register `SIZEP0–3` `$D008–$D00B`) und die Priorität (`prior`: 1, 2, 4 oder 8, Register `PRIOR` `$D01B`) tragen. Ohne `hw` ist die Ausgabe unverändert (Test); alte Dateien bleiben Byte-gleich. Die eingefrorene Iced-Oberfläche nutzt denselben Kern: sie zeigt die neuen Werte nicht an, behält sie aber beim Speichern (Metadaten bleiben erhalten).

| Nr | Schritt | Erwartung | Ergebnis |
|---|---|---|---|
| 53 | `.vcpmg` öffnen, Karte „Player/Missile“: Breite je Spieler (Normal, Doppelt, Vierfach) | „Vorschau im Spiel“ skaliert den gewählten Spieler (1×, 2×, 4× breit); Datei ungespeichert; Rückgängig stellt her | |
| 54 | Priorität wählen (Spieler vor Spielfeld, P0/P1 – Spielfeld – P2/P3, PF0/PF1 – Spieler – PF2/PF3, Spielfeld vor Spielern, Nicht gesetzt) | Wert gespeichert; mit „Nicht gesetzt“ und Breite „Normal“ für alle entfällt `hw` in der Datei wieder | |
| 55 | Export (Assembler) einer Datei mit gesetzten Werten | Konstanten `<name>_sizep0` … `_sizep3` und `<name>_prior` stehen im erzeugten Quelltext, immer alle fünf (ohne eingestellte Werte: Breite 0, Priorität 1) | |
| 56 | Dieselbe `.vcpmg` mit gesetzten Werten in der alten (Iced-)Oberfläche öffnen, etwas malen, speichern | Datei öffnet; `hw` bleibt erhalten | |
| 57 | PNG laden: im Sprite-Editor die Werkzeugleiste ansehen; Befehlspalette (Cmd/Strg+K) „PNG“ eingeben | „PNG laden …“ steht in der Werkzeugleiste neben „Exportieren …“; die Befehlspalette bietet „PNG-Bild in Sprite laden …“ (auch Menü Bearbeiten) | |
| 58 | `examples/atari/pmg_ship` bauen und im Emulator starten (atari800) | Vier Raumschiffe nebeneinander in Rosa, Blau, Grün, Weiß am oberen Bildrand; vorher war nichts zu sehen (Zeile 0 bis etwa 32 liegt über dem Bild, die Farben gingen in COLPM statt in die Schattenregister) | |
| 59 | Player/Missile: Breite und Priorität einstellen, Export ansehen; danach auf Standardwerte zurück | Immer alle fünf Konstanten `<name>_sizep0` bis `_sizep3` und `<name>_prior` (Standard: Breite 0, Priorität 1); auch nach Speichern und Neuöffnen. Unter der Priorität steht ein Hinweis zur sichtbaren Fläche | |
| 60 | Beispieldatei öffnen und ohne Änderung speichern (z. B. `examples/c64/jumprun/graphics/enemy_base.vcsprite`) | Die Datei bleibt byte-gleich; alle Mediendateien der Pixel-Editoren in `examples/` liegen im Speicherformat der Anwendung (ein Test sichert das) | |
| 61 | Bilderliste: Bild anklicken, mit Umschalt ein weiteres, dann mit Strg/Cmd einzelne dazu oder weg; Rechtsklick auf ein Bild | Markierte Bilder haben Rahmen und Häkchen; unter der Liste steht „n Bilder markiert“ mit „Animation aus Auswahl“ und „Auswahl aufheben“; das Kontextmenü bietet „Animation aus Auswahl …“, „Alle Bilder markieren“, „Auswahl aufheben“; Strg/Cmd+A in der Liste markiert alle | |
| 62 | „Animation aus Auswahl“ wählen, Namen und Tempo (Bilder pro Sekunde) eingeben | Das Fenster zeigt die Umrechnung in Ticks; nach „Animation erzeugen“ steht die neue Animation im Reiter „Animation“ und läuft sofort in der Vorschau; Reihenfolge wie in der Bilderliste | |
| 63 | Danach Rückgängig | Die ganze Animation verschwindet in einem Schritt; ein ungültiger oder schon vergebener Name wird im Fenster abgelehnt | |
| 64 | Reiter „Animation“: Filmstreifen ansehen; einen Schritt auf einen anderen ziehen; Tempo (Bilder pro Sekunde) ändern; „+“ drücken | Jeder Schritt zeigt eine Miniatur mit Nummer, Dauer in Ticks und Pfeilen/Entfernen; Ziehen ordnet um (Pfeile auch); „Tempo“ setzt alle Schritte (10 Bilder/s = 5 Ticks) und zeigt die Umrechnung; „+“ hängt das gewählte Bild an; Klick auf die Miniatur wählt das Bild zum Bearbeiten | |
| 65 | Reiter „Animation“: Knopf „Vorschau vergrößern“ (neben dem Abspielknopf) | Ein großes Fenster zeigt die Animation sofort laufend; Anhalten/Abspielen, Ablauf (Schleife/Hin und zurück/Einmal) und Zoom (Einpassen, 4 ×, 8 ×, 16 ×, 24 ×) lassen sich umstellen; „Schritt n von m“ läuft mit | |
| 66 | In der großen Vorschau die Schritte darunter bearbeiten: Dauer ändern, Schritt ziehen oder mit den Pfeilen verschieben, entfernen, „+“, Tempo für alle | Wie im Reiter „Animation“; der gerade laufende Schritt ist hervorgehoben; Änderungen wirken sofort in der Vorschau; Rückgängig im Editor nimmt sie zurück | |

**Vorgemerkt (nicht in E3):** MEGA65 VIC-IV-Sprites (16 oder 64 Punkte breit, 16 Farben) als eigenes Paket nach E3.

**Terminologie in diesem Stand:** Englisch „VIC-20“, Deutsch „VC-20“ in den Katalogtexten und im Systemnamen der Beispiele; „Studio“ ist in allen Katalogwerten durch „Editor“ ersetzt (die Schreibweisenprüfung sichert das).
