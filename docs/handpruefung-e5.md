# Handprüfung E5 Musik-Editor (Tracker)

Teilschritte E5.1 (Noten, Instrumente, Ablauf), E5.2 (Wiedergabe) und E5.3 (Klaviatur, Stimmen, Konvertieren/Import). Start: `start-tracker-c64.command` (SID, `music/tune.vcsong`) und `start-tracker-atari.command` (POKEY, `music/tune_pokey.vcsong`).

| # | Schritte | Erwartet | Ergebnis |
|---|---|---|---|
| 1 | Datei öffnen (Startskript) | Bereich **Musik**; links Reihenfolge der Muster und Instrumente, in der Mitte das Raster (Zeilen × Stimmen, Note und Instrument je Zelle), rechts der Instrumenteneditor | |
| 2 | Zelle anklicken; Pfeiltasten, Bild auf/ab, Pos1/Ende | Cursor bewegt sich (Rahmen), Bild auf/ab = acht Zeilen; Fuß nennt Reihe und Stimme | |
| 3 | Klaviertasten Z S X D C V G B H N J M und Q 2 W 3 E R 5 T 6 Y 7 U I | Trägt Noten in der gewählten Oktave ein, der Cursor rückt um die Schrittweite weiter; die Tasten gelten nach ihrer Position (auch bei QWERTZ oder AZERTY) | |
| 4 | Punkt und Komma (Oktave), [ und ] (Instrument), 1 (Notenende), Entf | Wirkt auf Eingabe und Zelle; Entf leert | |
| 5 | Umschalt+Pfeile; mit der Maus aufziehen; Cmd/Strg+C, X, V, A | Auswahl erscheint heller; Kopieren, Ausschneiden, Einfügen, Alles wählen | |
| 6 | Halbton ↑/↓, „Instrument anwenden“, Alt+Pfeil hoch/runter in einer Zelle | Block transponiert; Instrument der Noten gesetzt bzw. gewechselt | |
| 7 | Tempo, Zeilen, Oktave, Schritt ändern | Tempo und Zeilenzahl wirken auf alle Muster; Schrittweite 0–16 | |
| 8 | Muster neu, Duplizieren, Löschen; Reihenfolge anhängen, setzen, entfernen, verschieben; Wiederholung ab Eintrag | Reihenfolge und Wiederholungszeichen ↻ folgen | |
| 9 | Instrument wählen: SID: Klangform, Anschlag/Abfall/Halten/Ausklang, Pulsbreite; POKEY: Verzerrung und Hüllkurve je Bild | Regler wirken sofort; ein Zug ist ein Rückgängig-Schritt | |
| 10 | Instrument neu, Duplizieren, Löschen (nur das letzte unbenutzte) | Löschen ist sonst gesperrt mit Erklärung | |
| 11 | Rückgängig/Wiederholen, Speichern (Cmd/Strg+S), Export … | Export mit Vorschau; Datei als „geändert“ gekennzeichnet | |
| 12 | **Tastatur und VoiceOver:** Zellen, Reihenfolge, Instrumente | Ansage „Reihe 04, Stimme 2: C-4, Instrument 01“; alles ohne Maus bedienbar | |

| 13 | Zelle anklicken, **Leertaste** (oder Knopf ▶) | Song spielt hörbar ab dem gewählten Eintrag; Zeile blau hinterlegt, folgt der Wiedergabe; Fuß: „Wiedergabe: Eintrag …, Zeile …“; bei Mitscrollen springt die Ansicht mit zum jeweiligen Muster | |
| 14 | Pegel beobachten | Je Stimme ein Balken im Spaltenkopf, schlägt im Takt der Noten aus | |
| 15 | M und S im Spaltenkopf während des Spielens | Stimme verstummt bzw. nur die Solo-Stimme ist zu hören; ohne Wiedergabe merkt sich der Song die Schalter | |
| 16 | Während des Spielens Noten ändern, Instrument-Regler ziehen | Änderungen sind sofort zu hören, die Wiedergabe läuft weiter | |
| 17 | **Umschalt+Leertaste** oder Knopf „Muster ▶“ | Spielt nur das gewählte Muster einmal ab der Cursorzeile und endet von selbst | |
| 18 | Stopp (Knopf ■ oder Leertaste); Datei wechseln/schließen während des Spielens | Ton endet sauber, kein Knacken, kein Hängenbleiben | |
| 19 | Noten per Klaviertaste eintragen | Der Ton ist beim Eintragen kurz zu hören (Vorhören) | |
| 20 | Wiedergabe ohne Audiogerät (Gerät sperren/abziehen) | Meldung „Wiedergabe nicht möglich: kein Audio-Ausgabegerät gefunden“, sonst nichts kaputt | |
| 21 | **VoiceOver:** M/S-Knöpfe, Abspielen/Stopp | „Stimme 1 stumm, Schalter, gedrückt/nicht gedrückt“; Knöpfe sind mit Namen ansagbar | |

| 22 | **Klaviatur** (rechts unter dem Instrument): Tasten anklicken; Computertasten Z S X D … | Klick spielt die Note mit dem gewählten Instrument vor, ändert aber den Song nicht; die Taste leuchtet auch beim Eintragen und bei der Wiedergabe; beschriftet ist die Computertaste | |
| 23 | **Stimmen** 1 2 3 … über dem Raster ein-/ausblenden, „Alle einblenden“ | Spalte verschwindet, klingt aber weiter; Pfeil nach rechts überspringt ausgeblendete Stimmen; die letzte sichtbare lässt sich nicht ausblenden | |
| 24 | **Übernehmen …** (nur Song): eine .sid, .sng (GoatTracker) oder einen Song eines anderen Systems wählen | Dialog nennt Datei und Art (bei SID Titel, Autor, Subtunes), Name und Zielsystem einstellbar; bei SID Subtune und Aufnahmesekunden; neue Datei erscheint im Projekt und öffnet sich; Hinweise der Umwandlung als Meldung | |
| 25 | Dasselbe mit schon vorhandenem Namen | Fehlermeldung „existiert bereits“, nichts überschrieben | |
| 26 | Atari-Song: **RMT-Modul importieren …** (.rmt wählen) | Neuer POKEY-Song im Projekt; ohne gültiges Modul eine verständliche Fehlermeldung | |
| 27 | Export … eines Songs | Songdaten als asm/inc/bin/prg/DATA; die Spielroutine liegt dem Beispiel als lib/music.inc bei (die bisherige Oberfläche erzeugt keine eigene) | |
