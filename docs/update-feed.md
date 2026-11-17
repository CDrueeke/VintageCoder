# Update-Feed: so pflegt der Projektinhaber den Release `update-feed`

Die Update-Prüfung der IDE (und `vcc update`) liest **eine** kleine Datei, `update.json`. Sie liegt als Anhang an einem **festen Release ohne Versionsnummer** im
öffentlichen Repository. Das Programm lädt nichts anderes und öffnet nur einen Link zur Releaseseite, der im Programm selbst gebildet wird. Dieses Dokument
beschreibt, wie die Datei entsteht und gepflegt wird. **Das Skript lädt nichts hoch; das Veröffentlichen ist Handarbeit des Projektinhabers.**

## Die Adresse

```
https://github.com/CDrueeke/VintageCoder/releases/download/update-feed/update.json
```

`update-feed` ist ein Tag und ein Release mit diesem festen Namen (kein `v…`, keine Versionsnummer). Er wird **einmal angelegt** und danach nur noch gepflegt:
Bei jeder neuen Version wird der Anhang `update.json` **ersetzt**. Warum ein eigener Release? Die Adresse `…/releases/latest/…` liefert nur das neueste Release, das
**nicht** als „Pre-release“ markiert ist (ungeprüft); solange 0.x als Pre-release läuft, würde sie ins Leere führen. Der Release `update-feed` selbst wird ebenfalls
als Pre-release markiert, damit er nicht als „neueste Version“ auf der Startseite des Repositorys erscheint.

## Einmal einrichten

1. Auf GitHub: Releases ▸ „Draft a new release“, Tag `update-feed` (neu anlegen, Ziel `main`), Titel „Update-Liste (kein Programm)“,
   Beschreibung: „Technischer Release für die Update-Prüfung von VintageCoder. Hier gibt es nichts herunterzuladen; die Programme stehen bei den Versionen `v0.x.y`.“,
   Haken „Set as a pre-release“. Anhang: die erste `update.json` (siehe unten). Veröffentlichen.

## Bei jeder neuen Version

1. Die Version steht in `Cargo.toml` und in `release/CHANGELOG.public.md` als Abschnitt `## X.Y.Z (Zusatz)` (der Abschnitt „Unveröffentlicht“ wird dabei zur neuen Version umbenannt).
2. Datei erzeugen. Die bisherige Datei ist `--previous`, damit die fortlaufende Nummer `sequence` steigt:

   ```
   curl -fsSL -o alt-update.json https://github.com/CDrueeke/VintageCoder/releases/download/update-feed/update.json
   python3 tools/release.py manifest --out build/update-feed --previous alt-update.json --project-format <N>
   ```

   `--project-format` ist das höchste Projektformat (`"format"` in den Projektdateien), das die neue Version schreibt; ist es höher als das der installierten Version,
   warnt der Dialog „Update verfügbar“ („ältere Versionen können diese Projekte danach nicht mehr lesen“). Weglassen, wenn sich das Format nicht ändert.
3. `build/update-feed/update.json` ansehen. Der Text der Änderungen kommt unverändert aus dem öffentlichen Änderungsprotokoll; Bilder und Links werden im Programm nicht
   dargestellt.
4. **Erst nachdem** der Release `v<Version>` mit seinen Anhängen veröffentlicht ist: am Release `update-feed` den Anhang `update.json` durch die neue Datei ersetzen
   (Edit ▸ alten Anhang löschen, neuen hochladen). Reihenfolge wichtig: die Liste darf eine Version erst nennen, wenn deren Releaseseite
   `https://github.com/CDrueeke/VintageCoder/releases/tag/v<Version>` existiert, weil der Link darauf zeigt.
5. Prüfen, ohne das Netz zu verändern: `VC_UPDATE_URL=build/update-feed vcc update --check` liest den lokalen Ordner. Danach mit dem veröffentlichten Stand:
   `vcc update --check` (fragt GitHub, ausdrücklich durch den Aufruf).

## Eine Version zurückziehen („yanked“)

Eine fehlerhafte Version bleibt als Release stehen, soll aber nicht mehr empfohlen werden.

1. Die Nummer in `release/yanked.txt` eintragen (eine je Zeile).
2. Die Datei neu erzeugen (Schritt 2 oben, auch wenn es keine neue Version gibt; `--version` ist dann die höchste **nicht** zurückgezogene Version, z. B. `--version 0.1.0`).
3. Anhang ersetzen. Wer die zurückgezogene Version nutzt, sieht danach einen Hinweis und, falls es eine neuere gibt, das Angebot dazu.

## Was in der Datei steht (und was nicht)

| Feld | Bedeutung |
|---|---|
| `format` | Aufbau der Datei (heute 1). Ein höheres Format zeigt das Programm als „Liste neuer als bekannt“ an, ohne etwas anzuzeigen. |
| `sequence` | Fortlaufende Nummer (Lesehilfe; ohne Signatur wird sie nicht ausgewertet). |
| `channel` | Immer `stable`. |
| `published` | Zeitpunkt der Erzeugung (UTC). |
| `latest` | Höchste nicht zurückgezogene Version. |
| `project_format` | Optional: höchstes Projektformat der neuen Version. |
| `yanked` | Zurückgezogene Versionen. |
| `releases` | Die neuesten Abschnitte des Änderungsprotokolls (Version, Titel, Text). |

**Keine Adressen:** Die Datei enthält absichtlich keine Links. Die Adresse `…/releases/tag/v<Version>` bildet das Programm selbst, nachdem es die Versionsnummer streng
geprüft hat (genau drei Zahlen). Eine manipulierte Datei kann deshalb höchstens falschen Text oder eine falsche Nummer zeigen, aber nicht auf eine fremde Seite verweisen.

**Keine Signatur in dieser Fassung.** Die Datei ist in dieser Fassung unsigniert. Das Format bleibt nachrüstbar: eine Signaturdatei `update.json.minisig` kann später daneben
liegen, ohne dass sich `format` ändert; Programme dieser Fassung ignorieren sie.
