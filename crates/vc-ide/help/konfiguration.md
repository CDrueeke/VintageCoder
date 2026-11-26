# Konfiguration sichern und zurücksetzen

Unter **Einstellungen ▸ Sicherung** (Karte **Sicherung**) sicherst du die Konfiguration und holst sie zurück. Zur Konfiguration gehören Zielsysteme, Emulatoren, Maschinen, die Hilfe-Wahl und die Update-Wahl. **Passwörter der Maschinen sind nie dabei.** Mit der Konfiguration lässt sich ein zweiter Rechner schnell so einrichten wie der erste.

![Die Einstellungen mit der Liste links (Allgemein bis „Info und Update“), „Sicherung“ ist gewählt. Rechts die Karte „SICHERUNG“ mit einem kurzen Erklärtext und den Knöpfen „Konfiguration exportieren ...“, „Konfiguration importieren ...“ und „Auf Vorgabe zurücksetzen ...“.](bild:einstellungen-sicherung "Die Karte „Sicherung“ in den Einstellungen mit Export, Import und Zurücksetzen.")

Die Karte hat drei Knöpfe:

- **Konfiguration exportieren …** schreibt die Konfiguration in eine JSON-Datei, die du im Dateidialog benennst. Die Meldung nennt den Ort („Exportiert nach … (ohne Passwörter)“).
- **Konfiguration importieren …** liest eine solche Datei. Vorher zeigt ein Dialog, was drinsteht („Die Datei enthält … Zielsysteme und … Maschinen. Die aktuellen Einstellungen werden ersetzt.“). Erst **Übernehmen** ersetzt die jetzigen Einstellungen. Passwörter bereits bekannter Maschinen bleiben erhalten.
- **Auf Vorgabe zurücksetzen …** setzt alle Einstellungen auf die Vorgabe zurück, auch Zielsysteme, Emulatoren, Maschinen und ihre Passwörter. Ein Dialog fragt vorher nach; das lässt sich nicht rückgängig machen.

Der Konfigurationsassistent (siehe [Startseite, Konfigurationsassistent und neues Projekt](help:wizard)) gehört nicht dazu; er legt vor dem Schreiben eine eigene Sicherung an und bietet danach **Rückgängig**. Projekte bleiben von der Konfiguration unberührt. Farbschema und Zoom der Oberfläche gehören nicht zur Konfiguration, ebenso wenig die Anordnung der Bereiche. Siehe [Einstellungen](help:settings).
