# Sicherheit

## Sicherheitslücken melden

Bitte melde Sicherheitslücken **nicht öffentlich** (kein Issue, keine Pull Request), sondern privat:

- per E-Mail an **vintagecoder@drueeke.de** mit dem Betreff „Sicherheit“, oder
- über die private Meldung von GitHub („Report a vulnerability“ auf der Seite *Security* des Repositorys), falls sie dort eingeschaltet ist.

Nenne bitte die Version (in der Anwendung unter *Hilfe ▸ Feedback senden …* oder mit `vcc --version`), das Betriebssystem, eine Beschreibung und, wenn möglich, eine Schritt-für-Schritt-Anleitung zum Nachstellen. Schicke keine Zugangsdaten oder fremde urheberrechtlich geschützte Dateien (zum Beispiel ROMs) mit.

VintageCoder ist ein Projekt einer einzelnen Person im Alpha-Stadium. Es gibt **keine Zusage auf eine Antwortzeit** und keine Prämien für Meldungen. Ich lese jede Meldung und versuche, berechtigte Fehler in einer folgenden Version zu beheben; auf Wunsch nenne ich dich dann im Änderungsprotokoll.

## Unterstützte Versionen

Unterstützt wird nur die **jeweils neueste veröffentlichte Version**. Ältere Versionen bekommen keine Korrekturen.

## Was als Sicherheitsproblem gilt

VintageCoder läuft lokal, hat keinen Server und sendet keine Daten von selbst (siehe Hilfe „Feedback senden“). Relevant sind zum Beispiel: Ausführung fremden Codes durch das Öffnen einer Projektdatei, eines Disk-Images oder einer Medien-Datei, das Schreiben von Dateien außerhalb des Projekts, das Preisgeben von Zugangsdaten (etwa des Netzwerkpassworts für Ultimate-Geräte) oder Schwächen in den Installern. Fehler, die nur zum Absturz oder zu falschen Ergebnissen führen, melde bitte als gewöhnlichen Fehler.

Die Pakete sind **nicht signiert** (siehe README); das ist bekannt und keine Sicherheitsmeldung.
