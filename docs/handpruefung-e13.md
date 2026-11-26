# Handprüfung E13 Hilfe und Handbuch

Start: `start-hilfe.command` (öffnet die Hilfe; Menü Hilfe, F1 oder Befehl „Hilfe im Reiter öffnen“).

| # | Schritte | Erwartet | Ergebnis |
|---|---|---|---|
| 1 | Hilfe öffnen (Befehlspalette „Hilfe im Reiter“) | Links Suche und Themen nach Gruppen (Handbuch zuerst), rechts Willkommen | |
| 2 | Ein Handbuchkapitel wählen | Überschriften, Absätze, Listen, Tabellen, Code, Hervorhebungen gut lesbar; Zeilenbreite begrenzt | |
| 3 | Links zwischen Themen (`hilfe:…`) anklicken, dann „Zurück“ | Thema wechselt im selben Fenster; Zurück geht den Weg zurück | |
| 4 | Suche: `lda`, `sta`, ein Register (`$d020`), ein BASIC-Wort | Treffer mit Auszug; Klick öffnet die Beschreibung | |
| 5 | **F1** im Editor auf einem Wort (Befehl, Direktive, Register, BASIC-Wort) | Hilfe öffnet direkt die Beschreibung des Worts; ohne Eintrag eine freundliche Meldung | |
| 6 | Hilfe je Zielsystem und Sprache: Projekt eines anderen Systems öffnen, dasselbe Wort nachschlagen | Beschreibung passt zum System; Deutsch/Englisch folgt der Sprache | |
| 7 | **Handbuchbilder:** Kapitel mit Bildern (Dein erstes Projekt) | Beim ersten Mal die Frage „Handbuchbilder laden?“ (Nur dieses Mal, Immer, Nie); Bilder erscheinen; „Nie“ lässt sie weg ohne Fehler | |
| 8 | Einstellungen ▸ Allgemein ▸ Hilfe: Bilder und Ort (Adresse oder Ordner) | Wirkt beim nächsten Öffnen | |
| 9 | **VoiceOver:** Themenliste, Suchfeld, Text | Themen als Knöpfe mit Namen; Überschriften als Überschriften; „Zurück“-Knopf | |

Hinweis: Die Texte der Kapitel beschreiben teils noch die bisherige Oberfläche; die Anpassung der Handbuchtexte und Bilder an die neue Oberfläche ist Sache der Dokumentation.

## E13-Rest: Markdown (Vorschau, Schreibhilfen) und Handbuchbilder

Bilder: `~/Development/abnahme-ui/e13-markdown/bilder`. Handbuchbilder: `docs/handbuchbilder-neue-oberflaeche.md`.

| # | Schritte | Erwartet | Ergebnis |
|---|---|---|---|
| M1 | Eine `.md`-Datei aus dem Projektbaum öffnen (z. B. `README.md`) | Der Reiter zeigt die Vorschau mit Überschriften, Listen, Tabellen, Codeblöcken; oben „Vorschau / Quelltext“ | |
| M2 | „Quelltext“, Wort anklicken, Leiste: **B**, *I*, S, `</>`, Link, Bild, H+/H−, Listen, Aufgabe, Tabelle | Jede Hilfe wirkt auf Wort oder Auswahl; Cmd/Strg+Z macht sie in einem Schritt rückgängig; Befehle auch in der Befehlspalette (Markdown: …) | |
| M3 | Vorschau: Kästchen einer Aufgabenliste anklicken | Das Kästchen im Quelltext wechselt, der Reiter gilt als geändert | |
| M4 | Vorschau: Links anklicken (Datei im Projekt, `datei.asm:14`, Web, Anker) | Datei öffnet (Zeile 14); Web-Link öffnet den Systembrowser **nur auf Klick**; Anker springt; gefährliche Schemen (`javascript:`) sind kein Link | |
| M5 | Ein Bild im Projekt einbinden (`![x](bilder/a.png)`), ein fehlendes | Bild erscheint, fehlendes als Platzhalter; Bilder außerhalb des Projekts erscheinen nie | |
| M6 | Ein sehr langes Dokument öffnen | Blockweise Anzeige mit „Weitere Blöcke anzeigen“, kein Einfrieren | |
| M7 | Befehl „Markdown: Datei als Markdown behandeln“ auf einer `.txt` | Vorschau erscheint; erneut ausführen = wieder Text | |
