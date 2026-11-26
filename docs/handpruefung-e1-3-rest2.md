# Handprüfung E1.3 Rest 2 (vollständig)

Spalten: Nr | Schritt | Erwartung | Ergebnis

| 1 | Projekt mit Fehler in einer Quelldatei (z. B. `kaputt.asm`, Fehler in Zeile 3): „Aktive Datei bauen“; in der Ausgabe auf `kaputt.asm:3` klicken | Die Datei öffnet sich an Zeile 3 (nur die Ausgabe-Zeile mit `Datei:Zeile` ist klickbar, Zeiger wird zur Hand) | |
| 2 | Alle Dateien schließen (kein Reiter offen), Ablage (Ausgabe, Probleme) öffnen | Die Ablage lässt sich auch ohne offene Datei öffnen, schließen und in der Höhe ändern | |
| 3 | Ausgabe löschen, dann bauen | Neue Bauausgabe erscheint wieder mit klickbaren `Datei:Zeile`-Angaben | |
| 4 | Projekt mit eingerichtetem VICE: F5; im Emulator-Fenster VICE selbst beenden (⌘Q im Emulator) | Innerhalb etwa einer Sekunde: Maschine-Panel „bereit“, Beenden-Knopf in der Kopfzeile weg, Ausgabe „Emulator beendet.“ (bei Fehlercode: „… (Rückgabewert n)“ und Meldung); F5 startet sofort wieder | |
| 5 | F5, dann im Maschine-Panel „Beenden“ | Emulator-Fenster geht zu, Zustand „bereit“, Ausgabe „Emulator beendet.“ bzw. Meldung „Emulator beendet“ | |
| 6 | F5, dann in der Kopfzeile das Stopp-Symbol „Beenden“ | Wie 5 | |
| 7 | F5, dann Befehlspalette „Debuggen beenden“ (⇧F5) | Wie 5 (Debuggen selbst kommt mit E6; bis dahin beendet der Befehl den Emulator) | |
| 8 | F5 zweimal kurz hintereinander | Es läuft nur ein Emulator; der erste wird beendet, nicht vergessen | |
