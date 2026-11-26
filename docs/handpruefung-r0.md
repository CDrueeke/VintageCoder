# Handprüfung Registry (R0)

Die Registry selbst ist Technik: Sie ändert nichts, solange keine Erweiterung da ist. Damit du die Wirkung siehst, liegt im Paket `pakete/` (Demo-Paket `autor.demo`, ein kaputtes und ein falsch benanntes Paket) und eine eingeschränkte Release-Datei. Die App ist ein **Prüfbau** mit dem Entwicklerschalter `VC_EXTENSIONS_DEV` (im normalen Auslieferungsbau wirkungslos).
Starten mit `start-mit-paketen.command` (Doppelklick im Terminal-Fenster oder `./start-mit-paketen.command`), mit Release-Datei: `./start-mit-paketen.command release`; zum Vergleich `./start-ohne-pakete.command`.

| # | Schritte | Erwartet | Ergebnis |
|---|---|---|---|
| 1 | `start-ohne-pakete.command`: Menüleiste ansehen (Projekt, Datei, Bearbeiten …), Befehlspalette (Cmd+K) öffnen | Alles wie bisher; **kein** Untermenü „Projekt ▸ Erweiterungen“ | |
| 2 | `start-mit-paketen.command`: Menü **Projekt** ansehen | Es gibt das Untermenü **Erweiterungen** mit „Hallo sagen“, „Projekt-Info der Erweiterung“ (ausgegraut ohne Projekt) und „Nur mit Sprite-Editor“; „Hallo sagen“ zeigt das Kürzel Cmd+Alt+Umschalt+K | |
| 3 | Befehlspalette: „hallo“ eingeben, Eingabe | Der Eintrag trägt die Herkunft „autor.demo“; Ausführen meldet, dass die Werkzeuge später folgen | |
| 4 | Startseite ▸ „Alle Beispiele“ | Neben den Beispielen des Programms steht „Demo-Beispiel aus dem Paket“ mit „· autor.demo“ (nur Kopieren) | |
| 5 | Neues Projekt: Struktur-Auswahl | Neben den bekannten Strukturen steht „Demo-Struktur“ | |
| 6 | Sprache auf Englisch stellen (Menü Ansicht ▸ Sprache oder Palette) | Das Untermenü heißt „Extensions“; der Befehl „Say hello“ | |
| 7 | `start-mit-paketen.command release` (Release nur Tracker und Disk): Navigation oben ansehen | Der Hauptbereich **Grafik** fehlt ganz; Musik und Daten sind da (Daten ohne „Speicher“); im Menü Bearbeiten fehlt „PNG-Bild in Sprite laden“; der Befehl „Nur mit Sprite-Editor“ fehlt | |
| 8 | Im selben Start eine `.vcsprite` im Dateibaum doppelklicken | Sie öffnet als Text; Hinweis „… Der Sprite-Editor ist in diesem Release nicht enthalten.“ | |
| 9 | Alle anderen Funktionen kurz ausprobieren (Projekt öffnen, Bauen, Sprite-Editor ohne Release-Datei, Disk-Editor) | Unverändert | |

Nicht Teil von R0: Installieren und Prüfen von Erweiterungen mit Zustimmung (E1), Werkzeuge ausführen (E2), Hilfeseiten aus Paketen in der Oberfläche (Hilfe-Etappe; der Kern liest sie schon), Einstellungsseite „Erweiterungen“ (E12). Die Meldungen zu fehlerhaften Paketen (`kaputt`, `falsch`) stehen im Kern (`registry_info`), eine Anzeige kommt mit E12.
