# Handprüfung E12.1/E12.2 Einstellungen, Ziele und ROMs

Start: `start-einstellungen.command` (öffnet die App; Zahnrad oben rechts oder Cmd+Komma). Das Paket läuft mit **eigenem** Einstellungsordner (Kopie deiner Einstellungen), deine echten bleiben unberührt.

| # | Schritte | Erwartet | Ergebnis |
|---|---|---|---|
| 1 | Zahnrad oben rechts oder Cmd/Strg+Komma | Einstellungen öffnen im Dokumentenbereich; links sechs Seiten | |
| 2 | **Allgemein:** Sprache, Update-Wahl, Adresse der Versionsliste, Hilfe-Bilder, Projektordner (Wählen …), Debug-Protokoll | Sprache und Debug-Protokoll wirken sofort; die übrigen Werte brauchen „Speichern“ (oder „Verwerfen“); Hinweis „Ungespeicherte Änderungen“ | |
| 3 | Projektordner: Ordnerdialog, „~“ als Kurzform des persönlichen Ordners | Gewählter Ordner erscheint; nach Speichern merkt ihn „Neues Projekt“ als Vorschlag | |
| 4 | Einstellungsdatei: „Im Ordner zeigen“ | Das Betriebssystem zeigt `settings.json` | |
| 5 | **Aussehen:** Farbschema System, Dunkel, Hell, Hoher Kontrast; Zoom − + 100 % (und Cmd/Strg +, −, 0) | Wirkt sofort auf die ganze Oberfläche; Hoher Kontrast: schwarz/weiß/gelb, gut lesbar; Zoom bleibt nach Neustart | |
| 6 | Darstellung zurücksetzen | System, 100 %, Schrift 13, Umbruch an | |
| 7 | **Editor:** Schriftgröße, Zeilenumbruch, Formatieren beim Verlassen | Wirkt in offenen Editoren | |
| 8 | **Toolchain:** weitere Assembler-Schalter (`-I pfad`), BASIC-Dialekt je System | Speichern; beim nächsten Bauen gelten die Schalter | |
| 9 | **Tastenkürzel:** Suche (Befehl oder Kürzel) | Liste filtert; die Kürzel sind fest | |
| 10 | **Sicherung:** exportieren (Dateidialog), importieren (Rückfrage mit Zusammenfassung), zurücksetzen (Rückfrage) | Export ohne Passwörter; Import behält Passwörter bekannter Maschinen; Zurücksetzen räumt alles | |
| 11 | **Ziele und Hardware:** System hinzufügen/entfernen, Startziel (Emulator oder Maschine) | Liste links, Speichern übernimmt; F5 startet auf dem gewählten Ziel | |
| 13 | Emulator: Pfad eintragen/„Suchen“, „Testen“, Standard, Entfernen; ROM-Feld (atari800, Xemu) | Anzeige „Gefunden …“ oder „nicht gefunden“; „Testen“ startet den Emulator kurz und meldet Erfolg oder die Fehlermeldung des Emulators | |
| 14 | Ultimate 64: Adresse, Passwort (Zeigen/Verbergen), Zurücksetzen, RUN danach; „Verbindung testen“ | Meldung mit Zeit oder Fehler; Passwort nur im Klartext, wenn gezeigt | |
| 15 | MEGA65: USB-Schnittstelle (Liste der Ports), Baudrate, Ethernet (Hinweis „nur gegen Attrappen getestet“), „Verbindung testen“ | Test je gewählter Verbindung | |
| 16 | **ROMs:** Systeme und Rollen, Prüfsummen, durchsuchte Orte, „ROM-Ordner öffnen“, „Neu prüfen“, fremde ROMs | Status je Rolle (Erkannt, Ersatz, Fehlt); Ordner öffnet im Dateimanager | |
| 18 | **Info und Update:** Seite Info: Version, Lizenz, Webseite, Quelltext, Drittanbieter-Hinweise | Echte Links öffnen im Browser; nichts geht von selbst ins Netz | |
| 19 | „Jetzt nach Updates suchen“ (Netz nötig); bei neuer Version: Änderungen ansehen, Später erinnern, Überspringen; Hinweis „Update … verfügbar“ in der Statusleiste | Ohne Netz verständliche Meldung; „Später“ und „Überspringen“ blenden den Hinweis aus; Häufigkeit unter Allgemein | |
| 20 | **Feedback** (Info ▸ Feedback oder Befehl „Feedback“): „Mail vorbereiten“, „In die Zwischenablage“ | Mailprogramm öffnet mit vorbereitetem Text (Version, System, keine Namen); gesendet wird erst dort | |
| 21 | **Einrichtungsassistent:** erster Start ohne Einstellungen (oder Einstellungen ▸ Allgemein ▸ Einrichtung): Willkommen mit Sprachwahl, Schnelleinrichtung (7) oder Alles selbst (12) | Öffnet sich beim ersten Start von selbst; Schrittliste links; Weiter, Zurück, Später fortsetzen, Esc bricht ab | |
| 22 | Design (Farbschema, Zoom, Schrift als Vorschau), Zielsysteme (Schalter), Emulatoren/Hardware/ROMs (wie in den Einstellungen), Projekte (Ordner, Beispiele kopieren) | Vorschau wirkt sofort; Abbrechen stellt die Darstellung zurück; nichts wird vor „Übernehmen“ geschrieben | |
| 23 | Zusammenfassung: Änderungen mit Haken abwählen, „Übernehmen“, „Rückgängig“ | Sicherung der alten Einstellungen (settings.json.bak); Rückgängig stellt Einstellungen und kopierte Beispiele wieder her | |
| 24 | Teil-Assistenten (ROMs, Emulatoren, Hardware, Darstellung und Editor) | Kurze Wege mit Zusammenfassung | |
| 25 | **VoiceOver:** Seitenliste, Schalter, Auswahlgruppen | „Aussehen, aktuelle Seite“, „Farbschema, Auswahl“, Schalter mit Zustand | |

Seit E12.2b/E12.3 vorhanden (siehe Tabelle „E12-Reste“ unten): Argumente-Editor, ROM-Import, Emulator portabel installieren, „Neues erkannt“ und „Erstes Projekt“; Einzeltasten-Schalter und Tastaturlayout der Klaviertasten.

**Hinweis zur Sicherheit:** Das Startskript läuft mit eigenem Einstellungsordner (`home/` neben dem Skript, Kopie der echten `settings.json`); die echten Einstellungen unter `~/.config/vintagecoder` bleiben unberührt. Sichern geht mit `cp -R ~/.config/vintagecoder ~/vintagecoder-sicherung`.

## E12-Reste: ROM-Import, Emulator portabel installieren, Erstes Projekt, Neues erkannt

Bilder: `bilder/r-*.png`, `bilder/a-*.png`. Echte Netzzugriffe: nur Punkt 5 (mit Einwilligung), alles andere lokal.

| # | Schritte | Erwartet | Ergebnis |
|---|---|---|---|
| R1 | Einstellungen ▸ ROMs ▸ „ROM-Datei oder Zip wählen …“: eine eigene ROM-Datei, ein Zip mit ROMs und eine fremde Datei wählen | Plan: erkannte ROMs „wird kopiert“ mit Haken, Unbekanntes mit Grund; nichts wird kopiert, bevor „Übernehmen“ | |
| R2 | „… übernehmen“ klicken | ROM-Ordner enthält die Dateien je System; Stand der Rollen wird grün; zweiter Lauf zeigt „schon vorhanden“ | |
| R3 | ROM-Datei aus dem Finder auf das Fenster ziehen, solange die ROM-Seite offen ist | Derselbe Plan (keine Projekt-Rückfrage) | |
| R4 | „ROMs anderer Programme suchen“ (VICE, Xemu, atari800 installiert) | Liste der erkannten ROMs mit Herkunft; nur angehakte werden kopiert | |
| R5 | Bei einer Rolle mit mehreren Treffern die Datei im Auswahlfeld festlegen, danach „Automatisch“ | Meldung „in roms.json gespeichert“; Stand folgt | |
| R6 | „Fundliste kopieren“, in einen Texteditor einfügen | Systeme, Rollen, Zustand, Prüfsummen als Text, keine ROM-Inhalte | |
| E1 | Ziele und Hardware ▸ Emulator-Karte ▸ „Portabel installieren (Version) …“ | Rückfrage mit Version, Größe, Quelle, Zielordner; ohne „Jetzt laden“ passiert nichts | |
| E2 | **Mit Netz und Einwilligung:** „Jetzt laden“ | Laden, Prüfsumme, Auspacken; Meldung „Installiert: …“; Programm wird gefunden („Testen“) | |
| A1 | Einrichtung bis „Fertig“ | „Erstes Projekt“: „Neues Projekt anlegen …“ und „Beispiel öffnen: …“ öffnen das Gewünschte | |
| A2 | Nach dem Einrichten einen weiteren Emulator installieren, App neu starten | Startseite zeigt „Neues erkannt“ mit „Emulatoren einrichten“, „Als bekannt merken“, „Nicht mehr zeigen“ | |
