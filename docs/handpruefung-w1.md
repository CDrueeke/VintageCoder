# Handprüfung W1: Dateien aus dem Dateimanager in das Fenster ziehen

Die Ablauftests prüfen nur die Verarbeitung mit gegebenen Pfaden (`handleDrop`) und das Ziehen im Fenster mit Zeigerereignissen; das echte Ziehen aus Finder, Explorer oder Dateimanager (Tauris natives Drop-Ereignis) lässt sich nur von Hand prüfen. **Besonders wichtig auf Windows und Linux.**

| # | Schritte | Erwartet | Ergebnis |
|---|---|---|---|
| 1 | Projekt öffnen. Eine `.asm`-Datei **aus dem Projektordner** aus dem Dateimanager auf das Fenster ziehen | Beim Darüberziehen erscheint „Dateien hier ablegen …“; nach dem Loslassen öffnet die Datei als Reiter, ohne Rückfrage | |
| 2 | Eine `.vcsprite` **von außerhalb** des Projekts ziehen | Rückfrage „Ins Projekt kopieren und öffnen?“ mit Dateinamen; „Kopieren und öffnen“ legt sie nach `graphics/` und öffnet den Sprite-Editor; „Abbrechen“ kopiert nichts; dieselbe Datei nochmals: `…_2`, nichts überschrieben | |
| 3 | Eine `.d64`, `.d81` oder `.atr` ziehen | Öffnet im Disk-Editor (auch ohne Kopie) | |
| 4 | Eine `.vcprj` (oder einen Projektordner) ziehen | Projekt öffnet (bei Ungespeichertem erst die übliche Rückfrage) | |
| 5 | Mehrere Dateien zugleich ziehen | Verarbeitung nacheinander; Dateien außerhalb in einer gemeinsamen Rückfrage | |
| 6 | Eine Datei mit unbekannter Endung ziehen; ohne offenes Projekt eine `.asm` ziehen | Hinweis „… nicht öffnet“ bzw. „Öffne zuerst ein Projekt“, nichts wird kopiert | |
| 7 | Eine Datei ziehen und mit Esc/aus dem Fenster heraus abbrechen | Die Hinweisfläche verschwindet, nichts passiert | |
| 8 | **Ziehen im Fenster (neu mit Zeigerereignissen):** Datei im Dateibaum auf einen Ordner ziehen; auf die leere Fläche des Baums (Wurzel); Reiter umordnen; Reiter an den rechten Rand ziehen (teilt); im Sprite-Editor einen Animationsschritt auf einen anderen ziehen | Alles wie bisher; beim Ziehen hängt eine kleine Marke mit dem Namen am Zeiger; Klicks (Datei öffnen, Reiter wählen, Schließen-Kreuz) funktionieren unverändert | |

Hinweis zur Technik: Das natürliche HTML5-Ziehen im Fenster ist durch das native Drop-Ereignis in Tauri nicht mehr verfügbar (auf allen Systemen); darum laufen Dateibaum, Reiter, Teilen und der Filmstreifen jetzt über Zeigerereignisse (`ui/src/lib/dnd.svelte.ts`).
