# Nach Updates suchen

VintageCoder kann nachsehen, ob es eine neue Version gibt. Es zeigt dir dann, **was sich geändert hat**, und bietet einen **Link zur Releaseseite** an, von der du die neue Version selbst holst. **VintageCoder lädt und installiert nichts selbst.**

## Ohne deine Zustimmung geschieht nichts

Solange du unter **Einstellungen ▸ Allgemein ▸ Updates** nichts gewählt hast, steht dort *Noch nicht gefragt* und nichts geht ins Netz. Deine Wahl: *Nur auf Wunsch*, *Täglich* oder *Wöchentlich* (bei *Täglich* und *Wöchentlich* prüft VintageCoder beim Start). Die Wahl gilt nach **Speichern**. Die Seite **Info und Update** hat den Knopf **Jetzt nach Updates suchen**; er stellt immer genau eine Anfrage, ohne die Einstellung zu ändern. Das Feld **Adresse der Versionsliste** (leer = Standard) lenkt die Anfrage auf eine andere Liste um.

![Die Karte „Updates“ mit dem Text „Noch nie nach Updates gesucht.“, dem Knopf „Jetzt nach Updates suchen“ und dem Hinweis, dass nur die Versionsliste abgefragt wird und die Häufigkeit unter „Allgemein“ eingestellt wird.](bild:einstellungen-update "Die Karte „Updates“ mit dem Knopf zum Suchen nach Updates.")

## Was übertragen wird

Eine **Anfrage nach der Versionsliste** (`update.json`, wenige Kilobyte) an `github.com`. GitHub sieht dabei deine **IP-Adresse**, die Adresse der Datei und den Programmnamen „VintageCoder-update“, ohne Versionsnummer. Sonst nichts: keine Kennung, kein Konto, keine Angaben zu Rechner, Projekten oder Einstellungen, keine Zähler. Es gibt keinen eigenen Server des Projekts.

Jede Prüfung wird in einem Prüfprotokoll festgehalten (Zeit, Quelle, Ergebnis). Der Befund und das Protokoll liegen im Ordner `updates` im Konfigurationsordner (siehe [Einstellungen](help:settings)); das Ergebnis der letzten Prüfung steht auch auf der Seite **Info und Update** („Letzte Prüfung …“ oder „Noch nie nach Updates gesucht.“).

## Wenn es eine neue Version gibt

In der Statusleiste erscheint **Update X verfügbar**. Ein Klick öffnet **Einstellungen ▸ Info und Update**. Dort steht im Abschnitt **Updates**:

- „Neue Version X verfügbar“ mit dem Datum,
- bei einem neueren Projektformat eine **Warnung**: Ältere Versionen können deine Projekte danach nicht mehr lesen,
- die **Änderungen** seit deiner Version, als Text (Links und Bilder erscheinen nur als Text und lassen sich nicht anklicken),
- **Änderungen ansehen**: öffnet die Seite der neuen Version in deinem Browser. Der Link ist im Programm fest (`github.com/CDrueeke/VintageCoder/releases/tag/v…`); eine Adresse aus der Versionsliste wird nie übernommen.
- **Später erinnern**: der Hinweis ruht drei Tage.
- **Diese Version überspringen**: diese Nummer wird nicht mehr gemeldet; eine neuere meldet sich wieder.

Ist die installierte Version die neueste, steht dort „Du hast die neueste Version.“; wurde sie zurückgezogen, steht dort die Bitte zu aktualisieren.

Pakete deiner Paketverwaltung (zum Beispiel `.deb`, `.rpm`) aktualisierst du mit dieser. Der Dialog selbst lädt und installiert nie etwas.

## Wenn es nicht klappt

Automatische Prüfungen scheitern **still**: kein Netz ist kein Ereignis, nichts wird angezeigt, der nächste Versuch kommt später (nach einem Fehler frühestens nach sechs Stunden, dann seltener). Nur bei **Jetzt nach Updates suchen** erscheint eine Meldung mit der Ursache, zum Beispiel *keine Verbindung*, *Zeitüberschreitung* oder *Die Versionsliste ist ungültig*. Dabei wird nie etwas an deiner Installation geändert.

## Sicherheit

In dieser Fassung ist die Versionsliste **nicht signiert**. Deshalb kann sie nichts bewirken außer einer Anzeige: Der Link kommt aus einer festen Adresse im Programm, die Änderungstexte sind reiner Text, Größen und Anzahlen sind begrenzt, und es wird nichts geladen oder ausgeführt. Eine gefälschte Liste könnte höchstens eine falsche Versionsnummer oder einen falschen Text zeigen. Lade neue Versionen nur von der Releaseseite im Repository `CDrueeke/VintageCoder`.

## Auf der Kommandozeile

`vcc update` zeigt dasselbe an und endet mit Code 10, wenn es eine neuere Version gibt; siehe [Kommandozeile](help:cli). Andere `vcc`-Befehle fragen nie nach Updates. Mit `VC_UPDATE_URL` lässt sich die Quelle der Liste umstellen (zum Beispiel auf einen Ordner zum Ausprobieren); das ändert nie das Ziel des Links.

## Pakete ohne Update-Prüfung

Ein Paket kann ohne die Update-Prüfung gebaut sein (Auswahl `IDE-UPDATE`, siehe [Release-Auswahl](help:release-auswahl)). Dann gibt es weder diesen Abschnitt noch eine solche Verbindung.
