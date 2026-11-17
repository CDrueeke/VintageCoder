# Nach Updates suchen

VintageCoder kann nachsehen, ob es eine neue Version gibt. Es zeigt dir dann, **was sich geändert hat**, und bietet einen **Link zur Releaseseite** an, von der du die neue Version selbst holst. **VintageCoder lädt und installiert nichts selbst.**

## Ohne deine Zustimmung geschieht nichts

Solange du unter **Einstellungen ▸ Allgemein ▸ Updates** bei **Nach Updates suchen** nichts gewählt hast, steht dort „Noch nicht entschieden: es wird nichts gesendet“ und nichts geht ins Netz. Deine Wahl: *Nur auf Wunsch*, *Täglich beim Start* oder *Wöchentlich beim Start*. Auch **Über VintageCoder** hat den Knopf **Jetzt prüfen**. „Jetzt prüfen“ stellt immer genau eine Anfrage, ohne die Einstellung zu ändern.

## Was übertragen wird

Eine **Anfrage nach der Versionsliste** (`update.json`, wenige Kilobyte) an `github.com`. GitHub sieht dabei deine **IP-Adresse**, die Adresse der Datei und den Programmnamen „VintageCoder-update“, ohne Versionsnummer. Sonst nichts: keine Kennung, kein Konto, keine Angaben zu Rechner, Projekten oder Einstellungen, keine Zähler. Es gibt keinen eigenen Server des Projekts.

Jede Prüfung wird in einem Prüfprotokoll festgehalten (Zeit, Quelle, Ergebnis). Der Befund und das Protokoll liegen im Ordner `updates` im Konfigurationsordner (siehe [Einstellungen](help:settings)); das Ergebnis der letzten Prüfung steht auch neben **Jetzt prüfen**.

## Wenn es eine neue Version gibt

In der Statusleiste erscheint **Update X verfügbar**. Ein Klick öffnet den Dialog:

- die **Änderungen** seit deiner Version, als Text (Links und Bilder erscheinen nur als Text und lassen sich nicht anklicken),
- bei einem neueren Projektformat eine **Warnung**: Nach dem Öffnen mit der neuen Version können ältere Versionen deine Projekte nicht mehr lesen,
- **Ansehen …**: öffnet die Seite der neuen Version in deinem Browser. Der Link ist im Programm fest (`github.com/CDrueeke/VintageCoder/releases/tag/v…`); eine Adresse aus der Versionsliste wird nie übernommen.
- **Später erinnern**: der Hinweis ruht drei Tage.
- **Diese Version überspringen**: diese Nummer wird nicht mehr gemeldet; eine neuere meldet sich wieder. Unter Einstellungen ▸ Allgemein ▸ Updates kannst du das mit **Zurücknehmen** rückgängig machen.
- **Schließen**: schließt den Dialog, ohne etwas zu ändern.

Pakete deiner Paketverwaltung (zum Beispiel `.deb`, `.rpm`) aktualisierst du mit dieser. Der Dialog selbst lädt und installiert nie etwas.

## Wenn es nicht klappt

Automatische Prüfungen scheitern **still**: kein Netz ist kein Ereignis, nichts wird angezeigt, der nächste Versuch kommt später (nach einem Fehler frühestens nach sechs Stunden, dann seltener). Nur bei **Jetzt prüfen** erscheint eine Meldung mit der Ursache, zum Beispiel *keine Verbindung*, *Zeitüberschreitung* oder *Die Versionsliste ist ungültig*. Dabei wird nie etwas an deiner Installation geändert.

## Sicherheit

In dieser Fassung ist die Versionsliste **nicht signiert**. Deshalb kann sie nichts bewirken außer einer Anzeige: Der Link kommt aus einer festen Adresse im Programm, die Änderungstexte sind reiner Text, Größen und Anzahlen sind begrenzt, und es wird nichts geladen oder ausgeführt. Eine gefälschte Liste könnte höchstens eine falsche Versionsnummer oder einen falschen Text zeigen. Lade neue Versionen nur von der Releaseseite im Repository `CDrueeke/VintageCoder`.

## Auf der Kommandozeile

`vcc update` zeigt dasselbe an und endet mit Code 10, wenn es eine neuere Version gibt; siehe [Kommandozeile](help:cli). Andere `vcc`-Befehle fragen nie nach Updates. Mit `VC_UPDATE_URL` lässt sich die Quelle der Liste umstellen (zum Beispiel auf einen Ordner zum Ausprobieren); das ändert nie das Ziel des Links.

## Pakete ohne Update-Prüfung

Ein Paket kann ohne die Update-Prüfung gebaut sein (Auswahl `IDE-UPDATE`, siehe [Release-Auswahl](help:release-auswahl)). Dann gibt es weder diesen Abschnitt noch eine solche Verbindung.
