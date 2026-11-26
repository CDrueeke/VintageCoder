# Turbo BASIC XL: Diskette

Turbo BASIC XL ist ein schnelleres, erweitertes Atari-BASIC. Auf dem Atari wird es von einer **Diskette** geladen. Deshalb braucht ein Turbo-BASIC-XL-Projekt in VintageCoder eine Diskette, die **du selbst bereitstellst**.

Turbo BASIC XL ist kein Bestandteil von VintageCoder und muss vom Anwender beigesteuert werden. Wähle hier eine Diskette aus einer Quelle, die du verwenden darfst.

VintageCoder sendet keine Daten: Es gibt **keine Telemetrie**, die Diskette wird nirgends hin übertragen und liegt nur lokal.

## So funktioniert es

1. Das Projekt bekommt eine **Kopie** deiner Diskette im Ordner `turbo/` (unter dem Originalnamen, z. B. `turbo/Turbo_Basic_XL.atr`). Dein Original bleibt unverändert.
2. Beim **Bauen** entsteht aus dieser Kopie `build/<name>.atr`: Dein übersetztes Programm wird als `AUTORUN.BAS` hineingeschrieben. Turbo BASIC XL lädt `AUTORUN.BAS` nach dem Start selbst.
3. **Starten** (F5) bootet diese Diskette im Atari-Emulator (atari800), ohne eingebautes BASIC. Am Programmende steht `READY`: Turbo BASIC XL wartet dann auf Eingaben.

## Diskette bereitstellen

Das geht an drei Stellen: im Bereich **Projekt** auf der Seite **Diskette** über die Karte „Turbo BASIC XL“ (Knopf **Bereitstellen …**, mit schon vorhandener Kopie **Ersetzen …**), über das Banner „Dieses Projekt braucht eine Turbo-BASIC-XL-Diskette“, das beim Starten ohne Diskette über dem Editor erscheint (Knopf **Bereitstellen …**), und über die Befehlspalette („Turbo-BASIC-Diskette bereitstellen …“). Die Karte erscheint nur bei Projekten mit dem BASIC-Dialekt „Turbo BASIC XL“ für den Atari. Auf der Kommandozeile: `vcc turbo provide`; dazu gibt es `vcc turbo check` (prüfen), `vcc turbo reduce` (verkleinern) und `vcc turbo run` (bauen und starten).

*[Bild folgt in E13: turbo-karte-diskette]*

Der Assistent „Turbo-BASIC-Diskette bereitstellen“ hat drei Schritte (Datei wählen, Prüfung, Kopieren). Er prüft die Datei (ATR, DOS 2, `AUTORUN.SYS` mit der Kennung „TURBO-BASIC XL“, Version, freie Sektoren) und zeigt die Befunde. Sieht die Datei nicht nach Turbo BASIC XL aus, fragt er „Trotzdem verwenden?“. Das geschieht dann auf eigene Verantwortung. Unterstützt sind Abbilder von 90 KB (720 Sektoren zu 128 Byte); andere Größen meldet die Prüfung als „noch nicht unterstützt“.

### Platz und „Verkleinern“

Auf der Originaldiskette sind nur etwa 103 Sektoren (rund 12,9 KB) frei. Der Assistent bietet deshalb an, die **Kopie im Projekt** auf die drei Systemdateien `DOS.SYS`, `DUP.SYS` und `AUTORUN.SYS` zu verkleinern. Dann sind etwa 482 Sektoren (rund 60 KB) frei. Das geschieht nur nach deiner Bestätigung und nur in der Projektkopie. Entfernt werden Beispielprogramme sowie `COMPILER.COM`, `RUNTIME.COM` und `RAMDISK.COM`. Lehnst du ab, bleibt die volle Kopie; zu große Programme melden dann beim Bauen: „Das Programm braucht … Sektoren, auf der Diskette sind … frei.“

## Einstellungen im Projekt

Die Angaben stehen im Feld `turbo` der Projektdatei (`.vcprj`):

| Feld | Bedeutung |
|---|---|
| `disk` | Pfad der Kopie im Projekt |
| `sha256` | Prüfsumme der Kopie beim Bereitstellen (erkennt eine veränderte Datei) |
| `program` | Name des Programms auf der Diskette (Standard `AUTORUN.BAS`; nur dieser Name startet von selbst) |
| `files` | Weitere Dateien auf der Diskette: `Quelle` oder `Quelle:NAME` (8.3-Name, Großbuchstaben) |
| `reduced` | Die Kopie wurde verkleinert |

Dieselben Angaben bearbeitest du auf der Karte „Turbo BASIC XL“: Feld „Programmname auf der Diskette“ und die Liste „Zusatzdateien auf der Diskette“ (eine Datei je Zeile als `Quelle` oder `Quelle:NAME`), dann **Speichern**. Die Karte zeigt außerdem den Zustand („Diskette bereit“ oder „Diskette fehlt“), die freien Sektoren und die Dateien auf der Diskette und bietet **Prüfen**, **Verkleinern …** und **Im Disk-Editor öffnen**; diese Befehle gibt es auch in der Befehlspalette.

## Weitergeben

Die Datei `turbo/*.atr` steht in der `.gitignore` des Projekts und gehört nicht in die Versionsverwaltung. Wer dein Projekt bekommt, stellt seine eigene Diskette bereit. Beim Kopieren eines Projekts mit Turbo-Diskette erscheint nach dem Kopieren der Hinweis: „Das Projekt enthält eine Diskette, die nicht jeder weitergeben darf.“

## Grenzen

- **Debuggen** ist für Turbo-BASIC-Projekte noch nicht möglich („Turbo-BASIC-Programme lassen sich noch nicht schrittweise ausführen“); der Knopf ist ausgegraut.
- Nur atari800 und Altirra sind als Startziel vorgesehen (Altirra-Start mit ATR noch ungeprüft); Ultimate und echte Hardware später.
- Das Übersetzen mit `COMPILER.COM` auf der Diskette (Compiler-Stufe) ist nicht Teil dieser Fassung.

## Fehlerbilder

| Meldung | Bedeutung und Abhilfe |
|---|---|
| „Dieses Projekt braucht eine Turbo-BASIC-XL-Diskette …“ | Es ist keine Diskette bereitgestellt: Karte oder Banner, dann Bereitstellen …. |
| „Das sieht nicht nach Turbo BASIC XL aus …“ | Falsche Datei, oder eine andere Version/Verpackung. Andere Datei wählen oder „trotzdem verwenden“. |
| „Die Kopie … hat sich seit dem Bereitstellen verändert“ | Die Prüfsumme weicht ab. Meist harmlos (du hast die Kopie selbst bearbeitet); sonst neu bereitstellen. |
| Programm passt nicht auf die Diskette | Kopie verkleinern (Karte, Knopf Verkleinern …), oder das Programm verkleinern. |

Siehe auch: [BASIC: Dialekte und Zielsysteme](hilfe:basic-dialects), [Bauen und Starten](hilfe:build), [ROM-Dateien](hilfe:roms).
