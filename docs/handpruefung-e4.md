# Handprüfung E4 Zeichensatz-Editor (E4.1 Zeichenfläche und Raster, E4.2 Bereiche, Vorlage, Vergleich, Neu, E4.3 Export)

Beispiel: `arbeitskopie/gfx_demo` (C64) und `arbeitskopie/atari_gfx_demo`; Start mit den Skripten im Paket (öffnen die Zeichensatz-Datei direkt).

| # | Schritte | Erwartet | Ergebnis |
|---|---|---|---|
| 1 | `graphics/font.vcchar` öffnen (Projektbaum, Doppelklick; oder Startskript) | Grafik ▸ **Zeichensatz** öffnet; links die Übersicht aller 256 Zeichen (16 × 16) in den Farben des Satzes, in der Mitte der 8×8-Editor, rechts die Vorschau im Bild | |
| 2 | In der Übersicht ein Zeichen anklicken; Pfeiltasten, Pos1/Ende, Bild auf/ab | Das Zeichen wird gewählt (Rahmen), Marke „Zeichen n · $hh“; Pfeil unten = eine Zeile tiefer | |
| 3 | Mit Stift, Radierer, Füllen, Linie, Rechteck, Auswahl zeichnen; Spiegeln; Verschieben (Pfeile); Rückgängig | Wie im Sprite-Editor; die Übersicht zeigt jede Änderung sofort | |
| 4 | Zoom (+/−), Raster (G), Einpassen | Die 8 × 8 Punkte füllen die Fläche (Zoom bis über 3200 % bei so kleinen Bildern) | |
| 5 | Rechts „Vorschau im Bild“: den Text ändern | Der Text erscheint in den Zeichen dieses Satzes auf dem Hintergrund des Systems; Zeilenumbruch mit Eingabe | |
| 6 | Umgebung (Nachbarzeichen) anklicken; Zeichen leeren; Speichern | Nachbar wird gewählt; Zeichen wird leer (Rückgängig holt es zurück); Speichern schreibt die Datei, Punkt „ungespeichert“ verschwindet | |
| 7 | **Tastatur und Bildschirmleser:** Übersicht mit Tab fokussieren, Pfeiltasten benutzen | Der Bildschirmleser sagt „Zeichen n, $hh“ an; alles ist ohne Maus bedienbar | |
| 8 | Eine Sprite-Datei öffnen, dann Grafik ▸ Zeichensatz | Zeichensatz zeigt „Kein Zeichensatz geöffnet“ mit der Dateiliste; Sprite-Editor und Zeichensatz-Editor stören sich nicht | |

| 9 | Links unter der Übersicht **Bereiche**: Buchstaben, Ziffern, Satzzeichen, Alle; „Frei“ mit von/bis | Der Bereich wird in der Übersicht hell markiert, das erste Zeichen ist gewählt (Buchstaben = A, Ziffern = 0); ein zweiter Klick hebt auf; Bildschirmleser sagt den Bereich an | |
| 10 | Rechts **Vorlage**: „Standard (eigen)“ wählen, „Vorlage laden“ | Alle Zeichen werden ersetzt, Datei „geändert“; die Karte „Herkunft“ nennt den Satz; Rückgängig bringt den alten Stand zurück | |
| 11 | Ein Zeichen ändern | Punkt in der Übersicht an diesem Zeichen; Karte sagt „Gegenüber der Vorlage geändert“, nennt die Zahl der abweichenden Zeichen | |
| 12 | „Zeichen auf Vorlage zurücksetzen“ | Nur dieses Zeichen ist wieder wie die Vorlage (Rückgängig macht es rückgängig) | |
| 13 | Vorlage aus **eigenem ROM**: nur wenn ein ROM des Anwenders im Datenordner liegt; sonst steht der Eintrag ausgegraut („ROM nicht gefunden“) | Mit ROM: der Satz wird als Verweis geführt, Original-Zeichen stehen nicht in der Datei; ohne ROM: nicht wählbar | |
| 14 | **Neuer Zeichensatz …** (Werkzeugleiste oder leerer Zustand): Profil, Name, Ordner | Die Datei wird angelegt und im Zeichensatz-Editor geöffnet; das Profil bestimmt Einfarbig/Mehrfarbig | |

| 15 | **Export …**: Reiter Assembler, Name und Adresse ändern; „In Projekt speichern“ | Vorschau zeigt die 2048 Byte des Satzes als Quelltext; `build/export/<name>.asm` entsteht und erscheint im Projektbaum | |
| 16 | Export: „In Quelltext kopieren“ bzw. „Einbinden“ (Include-Zeile), PRG-Reiter | Wie beim Sprite-Export: Text steht im Quelltext, die Include-Zeile genau einmal | |

Nicht in E4: Anpinnen an Kachelsätze (kommt mit E8), Umschalter Einfarbig/Mehrfarbig innerhalb einer Datei (der Modus ist Sache des Profils, daher beim Anlegen zu wählen).
