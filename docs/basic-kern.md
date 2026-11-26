# BASIC-Kern: Zeilennummern, Labels, Dialekte, Prüfung (Stand der Etappen B0–B6)

Dieses Dokument fasst die Regeln der Kernfunktionen zusammen (Konzept „BASIC-Editor und Dialekte“, Entscheidungen B1–B19).

## Zeilennummern oder Labels
* Der **Modus** einer Datei ergibt sich aus dem Inhalt. Mischung meldet `B160`; beide Umwandlungen lösen sie auf.
* **Markerzeile `;@name`** vor einer Zeile hält den Labelnamen im Nummernmodus fest. `;@name auto` kennzeichnet einen automatisch erzeugten Namen:
  das Kennzeichen wird **nur gelesen** und im Nummernmodus erhalten; im Labelmodus gilt **jedes Label als manuell** (es gibt dort keinen Platz dafür).
* `REM`-Zeilen bleiben Programmzeilen (sonst wäre das Programm nicht byte-gleich); der Name eines automatischen Labels kommt aus dem `REM`- oder `;`-Text davor, sonst `z<alte Nummer>`.
* Berechnete Sprungziele (`GOTO A*10`) brechen die Umwandlung in Labels ab (`B162`).

## Start und Schrittweite der Nummerierung (Standard 10/10)
Rangfolge: Direktive `#numbering start=N step=N` in der Datei → Projekt (`.vcprj`, Objekt `basic`: `start`, `step`) → Standard. Die Kommandozeile
`--start`/`--step` ersetzt die Projektvorgabe; die Datei schlägt beides. Wertebereich: Schrittweite ≥ 1, Start ≤ größte Nummer des Dialekts (63999 bzw. 32767; `B191`);
reichen die Nummern nicht, nennt `B192` eine kleinere Schrittweite. Gilt für Bau, Umwandlung und Neu-Nummerieren (auch `--range`).
**Schnittstelle für die Oberfläche:** `vc_basic::numbering::{Numbering, resolve, directive_line}`; `Numbering::from_json`/`to_json` lesen und schreiben das Objekt `basic` der Projektdatei.

## Bau in der IDE (Stand Anbindung 1–4)
Der Bau der IDE (`vc-ide/src/build.rs`) und die Oberfläche (Projekt-Einstellungen, Karte „BASIC“) nutzen denselben Kern wie `vcc basic`:
* Zielsystem für `#if SYSTEM_…` und die Konstanten des Targets (`defines` der Projektdatei) kommen aus dem Projekt; `#include` wird vor dem Übersetzen aufgelöst, `#asm` mit dem Assembler der IDE (Commodore: hinter dem BASIC-Text, MEGA65/Atari: als DATA-Zeilen).
* Objekt `basic` der Projektdatei: `start`, `step` (Nummerierung), `compact` (Bauoption „kompakt“), `check` (`false`: Syntaxprüfung aus), `disabled` (abgeschaltete Prüfcodes, z. B. `["B152"]`). Standardwerte werden nicht geschrieben.
* Warnungen `B150`–`B167` erscheinen in der Problemliste (Gruppe „Bauergebnis“); Fehler `B191`/`B192` bei ungültiger oder zu knapper Nummerierung.

## Bekannte Grenze
`vcc basic` löst `#include` auf (Dateien und Medien); Meldungen zeigen auf Datei und Zeile der Quelle. Die Umwandlung (`--to-labels` usw.) berücksichtigt eingebundene Dateien und schreibt Medien nie zurück.

## Medien einbinden (`#include "graphics/hero.vcsprite"`)

Der Kern bindet Medien wie Dateien ein, über einen **Provider** (`vc_basic::include::IncludeHooks`): `Some(Ok(text))` = Medium (erzeugte DATA-Zeilen mit
`@name:`-Marke, ohne Zeilennummern), `Some(Err(grund))` = Fehler `B193`, `None` = gewöhnliche Datei. `vc-basic` hängt nicht von `vc-media` ab; der Bau setzt den
Provider mit `vc_media::include::resolve_include` (Format DATA).

* Schnittstellen: `expand_includes_with` (Bau, liefert Text, Zeilenzuordnung, Warnungen, eingebundene Medien), `labels::load_set_with` und
  `include::load_set_media` (Umwandlung; die Medien-Einheiten sind **schreibgeschützt**, nicht zurückschreiben). Die alten Funktionen ohne `_with` bleiben.
* Zeilenzuordnung: Medienzeilen zeigen auf den eingebundenen Pfad (Zeilen 1…n des erzeugten Textes).
* Ein Medium wird je Programm einmal eingebunden; ein zweites `#include` desselben Mediums wird still übersprungen.
* Nummerierung (`#numbering`, Projektwerte) und `#if` um das `#include` wirken wie bei jeder anderen Zeile; das Medium trägt keine eigenen Nummern.
* `#asm "player.asm" with "music/tune.vcsong"`: der Kern reicht die Namen unverändert an den Assembler-Aufruf weiter; ein Medium dort löst der Assembler-Provider auf.
* Meldungen: `B193` (Medium nicht gefunden/nicht als DATA exportierbar), `B194` (Altpfad `build/export/…`, mit Quelle), `B195` (Programm größer als der freie
  Speicher: C64 38911, VC-20 unerweitert 3583, Plus/4 60671 Byte; andere Systeme ungeprüft), `B196` (mehrere Medien und `RESTORE @marke` in CBM V2).
* `vcc basic` setzt den Provider (Projektordner = nächster Ordner mit Projektdatei, Zielsystem = `--target`); Test `basic_media_cmd.rs` belegt Bytegleichheit zum Altpfad.
* Zeilennummer-Konstanten (Konzept 7.8, Entscheidung 8): zu **jeder** Marke `<name>` gibt es `<name>_line`; sie wird beim Nummerieren durch die Zeilennummer der Marke
  ersetzt (nicht in Zeichenketten, `REM`, `DATA`; Vergleich ohne Groß-/Kleinschreibung). Mehrteilige Medien: je Teil `<medium>_<teil>_line` (Marke `@font_chars` →
  `font_chars_line`). Wirksam nur, wenn mindestens ein Medium eingebunden ist (`PlainOptions.media > 0`); sonst ist der Name eine gewöhnliche Variable. Die Prüfung
  (`B150`/`B154`) behandelt jeden Namen auf `_line` als Zahl. Das Medium wird an der Stelle des `#include` automatisch nummeriert; im reinen Nummernmodus muss die
  folgende explizite Zeilennummer größer sein (sonst `B100`).
