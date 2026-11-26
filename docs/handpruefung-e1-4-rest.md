# Handprüfung E1.4 Rest (Nachbesserungen nach dem Handtest E1.4)

| Nr | Schritt | Erwartung | Ergebnis |
|---|---|---|---|
| 1 | Neuer unbenannter Reiter mit Text, dazu eine geänderte Datei; ⌘Q; im Dialog „Alle speichern“; Namen vergeben | Alle werden gespeichert (je ein Namensdialog für jeden unbenannten Reiter), danach **beendet sich das Programm** | |
| 2 | Wie 1, aber roter Fensterknopf | Wie 1 | |
| 3 | Wie 1, aber „Projekt schließen“ | Nach dem Speichern schließt das Projekt (Startseite) | |
| 4 | Wie 1, im Namensdialog „Abbrechen“ / Esc | Beenden bzw. Schließen ist abgebrochen, Reiter bleiben | |
| 5 | Suchleiste im Editor öffnen (⌘F); im Text rechtsklicken | Die Suchleiste schließt; es erscheint genau ein Menü | |
| 6 | Ein Menü offen (z. B. Reiter-Menü), dann in den Text oder auf einen anderen Reiter rechtsklicken | Das alte Menü schließt, nur das neue ist offen | |
| 7 | Rechtsklick im Suchfeld, in den Namensfeldern, im Filterfeld | Eigenes Menü Ausschneiden/Kopieren/Einfügen/Alles auswählen (Ausschneiden/Kopieren nur mit Auswahl); nie das Menü mit „Neu laden“/„Autofill“ | |
| 8 | Rechtsklick auf freie Fläche (Kopfzeile, Statuszeile) | Kein Menü | |
| 9 | Startseite (kein Projekt) | „Beispiel öffnen“ ist da; die Liste zeigt die Beispiele; nur „Kopieren“ (kein „Öffnen“, das Programmpaket ist schreibgeschützt); Kopie öffnet sich danach | |
| 10 | Startseite mit mehr als fünf früheren Projekten | Höchstens 5 Einträge | |
| 11 | Neues Projekt, System VC-20 | Die Zeile „Speicherausbau“ bricht um, kein waagerechtes Scrollen im Dialog | |
| 12 | Fenstergröße ändern (Ziehen an der Ecke), ca. 2 s warten, Programm beenden, neu starten | Das Fenster hat wieder dieselbe Größe und Lage; `window.json` zeigt die neue Größe | |
| 13 | Fenster verschieben, dann sofort ⌘Q (ohne zu warten) | Neue Lage wird beim Beenden gespeichert | |
| 14 | Neues Projekt: Sprache BASIC wählen; Vorlagenkarte, Statuszeile, Art-Auswahl bei „Neue Datei“, Bearbeiten ▸ Art | Überall steht nur „BASIC“, nirgends „BASIC (ohne Zeilennummern)“ | |
| 15 | Fenster verschieben oder Größe ändern, dann sofort ⌘Q (Menü „Beenden“ ebenso) | `window.json` hat die neue Lage und Größe (beim Beenden wird synchron gespeichert, nicht nur entprellt) | |
| 16 | Suchleiste im Editor offen (⌘F), dann Rechtsklick auf Reiter, auf ein Suchfeld, in den Text | Jedes Menü liegt vollständig **über** der Suchleiste; nie zwei Menüs gleichzeitig | |
| 17 | Startseite: „Zuletzt geöffnete Projekte“ | Kompakte Zeile je Projekt (System, Name, Pfad in der Mitte gekürzt, Alter, Stern, „…“-Menü); angeheftete oben unter eigener Überschrift, darunter nach Öffnungszeit absteigend; Klick auf die Zeile öffnet | |
| 18 | In der Liste: ↑/↓ zwischen den Einträgen, Enter öffnet; „…“-Menü: Öffnen, Kopieren …, Im Ordner zeigen, Aus der Liste entfernen | Funktioniert; „Im Ordner zeigen“ öffnet den Finder auf dem Projektordner | |
| 19 | Projektordner umbenennen, Startseite neu laden | Eintrag ausgegraut mit „Ordner fehlt“, nicht anklickbar, aber entfernbar | |
| 20 | Mehr als 5 frühere Projekte | Es erscheinen 5, darunter „Weitere … (n)“; nochmal klicken: „Weniger anzeigen“ | |
| 21 | Startseite, hell und dunkel | Karten (Kopf, Zuletzt, Beispiele, Toolchain, Schnellreferenz) heben sich von der Seitenfläche ab; Einträge beim Überfahren/Fokus eine Stufe heller; Text gut lesbar | |
| 22 | Startseite: Beispiele | Wenige empfohlene Beispiele mit Beschreibung unter der Zuletzt-Liste; „Alle Beispiele …“ und „Beispiel öffnen“ öffnen einen Dialog mit Suchfeld, Gruppen je System (zugeklappt, bei Suche offen); im Programmpaket nur „Kopieren“ | |
| 23 | Seitenleiste, Kopfzeile „Projekt“ | Kreuz „Projekt schließen“ sichtbar (mit Rückfrage bei ungespeicherten Änderungen) | |
| 24 | Startseite, Schnellreferenz | ⌘⌥N, ⌘⇧O u. ä. mit richtigen Zeichen (⌥ nicht als „\“, ⇧O nicht als „⇧0“) | |
