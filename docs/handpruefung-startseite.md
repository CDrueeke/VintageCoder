# Handprüfung Startseite (Entwurf A)

Start: `start-startseite.command` (startet die App ohne Projekt mit leerem Zuletzt-Stand im eigenen Ordner `home/`; die Beispielkarten zeigen im Paket echte Aufnahmen).

| # | Schritte | Erwartet | Ergebnis |
|---|---|---|---|
| 1 | App ohne Projekt starten | Logo „VintageCoder.“, Beschreibung, Version (ohne Alpha-Hinweis); drei Aktionskarten Neues Projekt (hervorgehoben, mit Kürzel), Projekt öffnen, Beispiel öffnen | |
| 2 | „Neues Projekt“, „Projekt öffnen“, „Beispiel öffnen“, „Projektdatei wählen …“ | Jeweils der bekannte Dialog; Kürzel stimmen | |
| 3 | Zuletzt geöffnet: Karten mit Projektbild (`projektbild.png` im Projektordner) oder Ersatzkachel mit Anfangsbuchstaben | System, Alter, Stern (anheften), Menü (Öffnen, Kopieren, Im Ordner zeigen, Entfernen); fehlender Ordner als „Ordner fehlt“ | |
| 4 | Beispiele: Filter nach System (Alle, C64, Atari …), Karten mit Bild, Titel, System, Beschreibung | Höchstens acht Karten; „Öffnen“ nur bei beschreibbaren Beispielen, „Kopieren …“ immer; „Alle Beispiele …“ öffnet den Dialog mit Suche | |
| 5 | Erkannte Emulatoren, Tastenkürzel | Haken für gefundene, „nicht gefunden“ für fehlende | |
| 6 | Schmales Fenster (unter 700 px), helles Farbschema, Englisch | Karten untereinander; alles lesbar | |
| 7 | **VoiceOver:** Aktionskarten, Zuletzt, Beispielkarten, Filter | Knöpfe mit Namen; Filter als Schalter | |

Noch offen: Troxx-Bild (Platzhalter), Startbild (Splash), Willkommenskarte beim Erststart, Hauptnavigation in schmalen Fenstern.
