# Feedback senden

Fehler, Wünsche und Fragen schickst du per E-Mail an **vintagecoder@drueeke.de**. VintageCoder bereitet die Mail vor, damit der Betreff und ein Textblock die wichtigen Angaben zu deiner Installation enthalten. Du schreibst nur noch, was passiert ist.

## So geht es

1. Menü **Hilfe ▸ Feedback senden …** oder die Befehlspalette (Cmd/Strg+K, Wort „Feedback“). Den Dialog erreichst du auch über **Einstellungen ▸ Info und Update** (Knopf **Feedback …**).

![Der Dialog „Feedback senden“ über der Seite „Info und Update“. Er zeigt einen Hinweis, das Feld „Betreff“ mit Versionsangabe, das Feld „Text“ mit technischen Angaben zur Installation, einen Schalter für das Debug-Protokoll und die Knöpfe „Abbrechen“, „In die Zwischenablage“ und „Mail vorbereiten“.](bild:feedback-dialog "Der Dialog „Feedback senden“ mit Betreff, Text und Mail-Knopf.")

2. Der Dialog **Feedback senden** zeigt **Betreff** und **Text** der Mail, so wie sie später im Mailprogramm stehen. Lies sie durch.
3. **Mail vorbereiten** startet dein Standard-Mailprogramm mit einer neuen Nachricht. Schreibe die Fehlerbeschreibung in die **leere Zeile über dem Angabenblock**: was du getan hast, was du erwartet hast und was stattdessen geschah.
4. Hänge **Screenshots und Dateien selbst an** (zum Beispiel das Projekt oder das Protokoll aus dem Reiter **Ausgabe**). VintageCoder fügt nichts davon automatisch bei.
5. Gesendet wird erst, wenn du im Mailprogramm auf **Senden** klickst.

Hast du kein Mailprogramm eingerichtet (etwa nur ein Webmail), nimm **In die Zwischenablage** und füge den Text in eine neue Mail an die genannte Adresse ein.

## Was in der Mail steht

Im Betreff: Version, Commit des Baus, Betriebssystem mit Version und Architektur sowie die Installationsart. Beispiel:

`VintageCoder 0.3.0 (Commit abc1234) · macOS 27.2 arm64 · Paket: macOS-Programmpaket`

Im Text unter **Angaben zur Installation**: Version, Commit, System, Installationsart (Windows-Installer, macOS-Programmpaket, Linux-Paket, Quellbau oder unbekannt), Installationsordner, Datenordner, Konfigurationsordner, das Zielsystem des offenen Projekts und die Release-Auswahl dieser Installation (welche Editoren, Anbindungen und Zielsysteme enthalten sind).

## Datenschutz

- Es wird **nichts ins Netz geschickt** und nichts ohne deinen Klick gesendet; VintageCoder sammelt keine Nutzungsdaten.
- Die Mail enthält **keine Namen von Projekten oder Dateien** und keine Quelltexte, nur das Zielsystem (zum Beispiel `c64`).
- Pfade in deinem persönlichen Ordner stehen als `~` (statt `/Users/<name>/…`), damit dein Benutzername nicht in der Mail steht.
- Du kannst den Text im Mailprogramm vor dem Senden ändern oder kürzen.

## Lange Mails

Links zum Mailprogramm sind auf rund 1900 Zeichen begrenzt (Windows verarbeitet etwa 2000). Passt der Text nicht hinein, lässt VintageCoder zuerst die Release-Auswahl, dann die Ordner weg und schreibt im Text, was fehlt. **In die Zwischenablage** enthält immer alles. In diesem Fall steht im Dialog ein Hinweis.

## Debug-Protokoll für schwer beschreibbare Fehler

Lässt sich ein Fehler schwer beschreiben, schalte das **Debug-Protokoll** ein (Einstellungen ▸ Allgemein ▸ Fehlersuche, Schalter **Debug-Protokoll in eine Datei schreiben**, oder derselbe Schalter im Dialog „Feedback senden“; für `vcc` und die IDE auch die Umgebungsvariable `VC_DEBUG_LOG=1`). Danach schreibt VintageCoder in eine Datei auf deinem Rechner (`logs/vintagecoder.log` im Konfigurationsordner, Zeitstempel in UTC):

- Kopf mit denselben Angaben wie die Feedback-Mail,
- Menübefehle, Meldungen des Protokolls (Bau, Start, Emulator, Debugger), Fehler und Warnungen,
- bei einem Absturz die Meldung mit Rückverfolgung.

Wiederhole den Fehler, schließe VintageCoder und hänge die Datei an deine Mail (sie liegt in `logs/vintagecoder.log` im Konfigurationsordner). Das Protokoll wird **nie automatisch gesendet oder angehängt**. Pfade im persönlichen Ordner stehen als `~`, Passwörter und Tokens werden geschwärzt; Dateinamen und Fehlermeldungen deines Projekts können vorkommen, sieh die Datei vor dem Anhängen an. Ab 5 MB beginnt eine neue Datei, die alte bleibt als `vintagecoder.log.1`.

Die Datei besteht aus JSON-Zeilen (`ts`, `seq`, `kind`, `msg`) und lässt sich automatisch auswerten: `vcc debuglog datei.log` fasst sie zusammen (Programm, Arten, Abstürze, Fehler, letzte Ereignisse), `--json` liefert dasselbe maschinenlesbar.

## Auf der Kommandozeile

`vcc feedback` gibt Betreff und Text aus, `vcc feedback --open` öffnet das Mailprogramm, `vcc feedback --json` liefert alles maschinenlesbar, `vcc debuglog` wertet ein Debug-Protokoll aus. Siehe [Kommandozeile](help:cli).
