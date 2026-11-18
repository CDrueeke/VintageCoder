# Lizenz und freie Nutzung

Diese Seite erklärt in einfachen Worten, **was du mit VintageCoder, mit der Sprache VCASM und mit den Dateien tun darfst, die du damit erzeugst**. Der verbindliche Teil ist Abschnitt 3 („Erklärung des Rechteinhabers“).

## 1. Kurzfassung

- **Was du mit VintageCoder baust, gehört dir.** Programme, Quelltexte, Diskettenabbilder, Grafiken, Musik und alle anderen Dateien, die du mit VintageCoder erzeugst, darfst du beliebig verwenden und weitergeben: auch **kommerziell**, auch **ohne deinen Quelltext offenzulegen**, auch **ohne einen Hinweis auf VintageCoder**.
- **Die Sprache VCASM ist frei.** Ihre Syntax und Bedeutung (Befehle, Direktiven, Ausdrücke, Makros) und die Dateiformate darf jeder in eigenen Werkzeugen umsetzen, unabhängig von der Lizenz von VintageCoder.
- **`vcc` und `vc-lsp` darfst du aus beliebiger Software aufrufen**, auch aus nicht freier (Editoren, Build-Systeme, automatische Prüfungen), über die Kommandozeile, Dateien, die Standardein- und -ausgabe oder das Language-Server-Protokoll.
- **Die GPL gilt für den Quelltext von VintageCoder selbst.** Wer dieses Programm ändert und weitergibt, beachtet die GNU General Public License, Version 3 oder später.

## 2. Fragen und Antworten

**Darf ich ein Spiel verkaufen, das ich mit VintageCoder gebaut habe?**
Ja. Das Spiel unterliegt keiner Lizenz von VintageCoder. Ausnahme sind Fremdanteile, die du selbst einbindest (zum Beispiel der RMT-Player, siehe Frage unten, oder Musik, Bilder und ROMs Dritter).

**Muss ich meinen Quelltext veröffentlichen?**
Nein. Das gilt für deine eigenen Quelltexte und für die Programme, die `vcc` daraus erzeugt.

**Darf ich die mitgelieferten Systemdateien, Vorlagen und Beispiele in meine Programme übernehmen?**
Ja. Die Systemdateien (`data/sys/`), die Vorlagen (`data/templates/`) und die Beispielprojekte (`examples/`) stehen unter der **MIT-No-Attribution-Lizenz (MIT-0)**: Du darfst sie ohne Auflagen verwenden, ändern und weitergeben, auch ohne Urheberhinweis.

**Darf mein Editor oder mein Build-System `vcc` oder `vc-lsp` aufrufen, auch wenn es nicht frei ist?**
Ja. Aufrufen und Nutzen über Kommandozeile, Dateien, Standardein- und -ausgabe und das Language-Server-Protokoll ist keine Bearbeitung des Programms und macht dein Programm nicht zu einem „abgeleiteten Werk“.

**Darf ich einen eigenen Assembler oder Editor bauen, der VCASM versteht?**
Ja. Die Sprache und die Dateiformate sind frei. Du darfst dazu aber keinen Quelltext aus VintageCoder übernehmen, außer unter den Bedingungen der jeweiligen Lizenz der Dateien (GPL für das Programm, MIT-0 für die Systemdateien, Vorlagen und Beispiele).

**Darf ich VintageCoder ändern und meine Fassung weitergeben?**
Ja, unter der GPL: Du gibst den Quelltext deiner Fassung ebenfalls unter der GPL weiter und lässt die Lizenzhinweise stehen.

**Was ist mit dem RMT-Player in den Atari-Beispielen?**
Der RMT-Player (Raster Music Tracker, Dateien `rmtplayr.asm` und `rmt.inc` in den Beispielen `music_demo` und `jumprun` für den Atari) stammt von **Radek Štěrba / Raster / C.P.U.** und steht unter der **GNU General Public License, Version 3**. Er gehört **nicht** zu den freigegebenen Teilen. Ein Programm, das ihn enthält, darfst du nur nach den Bedingungen dieser Lizenz weitergeben (unter anderem mit dem Quelltext des gesamten Programms unter der GPL). Wenn du das nicht willst, verwende ihn nicht und nimm einen anderen Spieler.

**Darf ich die Namen und das Logo „VintageCoder“ verwenden?**
Nein, nicht frei. Der Name und das Logo (Dateien unter `branding/`) gehören nicht zu den freigegebenen Teilen. Du darfst aber wahrheitsgemäß sagen, dass ein Programm mit VintageCoder gebaut wurde.

**Was ist mit ROMs, Zeichensätzen, Musik und Bildern?**
Für Inhalte, die du selbst einbindest, bist du verantwortlich: Rechte Dritter bleiben bestehen. VintageCoder liefert **keine ROMs** mit. Die mitgelieferten eigenen Zeichensätze („VC-Standard“, `data/charsets`) stehen unter **CC0-1.0**.

## 3. Erklärung des Rechteinhabers

**VintageCoder** (die Entwicklungsumgebung `vintagecoder`, das Kommandozeilenprogramm `vcc`, der Sprachserver `vc-lsp` und alle zugehörigen Werkzeuge) steht unter der GNU General Public License, Version 3 oder später (`GPL-3.0-or-later`, siehe [LICENSE](LICENSE)). Der Rechteinhaber erklärt darüber hinaus **ausdrücklich und unwiderruflich** (zusätzliche Erlaubnis nach Abschnitt 7 der GPLv3):

**1. Ausgaben sind frei.** Alle Dateien, die mit VintageCoder erzeugt werden, gehören dem Anwender und sind vollkommen frei nutzbar: übersetzte Programme (zum Beispiel PRG, XEX, BIN, CRT), Diskettenabbilder, Quelltexte (Assembler, BASIC), Projekt- und Assetdateien (zum Beispiel `.vcprj`, `.vcsprite`, `.vcchar`, `.vcpal`, `.vcplay`, `.vctest`), Grafiken, Berichte, Protokolle und alle sonst erzeugten Dateien. Sie unterliegen nicht der GPL und keiner anderen Beschränkung durch den Rechteinhaber; der Anwender darf sie beliebig, auch kommerziell und auch ohne Offenlegung des Quelltextes, verwenden, verändern und weitergeben.

**2. Eingefügte Teile sind eingeschlossen.** Das gilt auch für Teile, die VintageCoder beim Übersetzen oder Erzeugen aus eigenen Beständen in die Ausgabe einsetzt oder einbindet: Systemdateien, Vorlagen, Bibliotheks- und Laufzeitroutinen, Spielroutinen, Tabellen, Kopfzeilen und erzeugte Textbausteine.

**3. Die Sprache ist frei.** Die Syntax und die Bedeutung des VCASM (Befehle, Direktiven, Ausdrücke, Makros, Kennzeichnungen) sowie die Dateiformate der Projekt-, Asset- und Austauschdateien sind frei nutzbar. Jeder darf sie in eigenen Werkzeugen (Assemblern, Editoren, Konvertern, Emulatoren) umsetzen oder unterstützen, unabhängig von der Lizenz von VintageCoder. Der Rechteinhaber macht keine Rechte an der Sprache und den Formaten geltend, auch keine Patent- und Markenrechte an der Syntax als solcher.

**4. Die Werkzeuge dürfen frei aufgerufen werden.** Die Verwendung von `vcc` und `vc-lsp` über die Kommandozeile, über Dateien, über die Standardein- und -ausgabe und über das Language-Server-Protokoll begründet kein abgeleitetes Werk. Aufrufende Programme (Editoren, Entwicklungsumgebungen, Build-Systeme, automatische Prüfungen) dürfen jeder Lizenz unterliegen.

**5. Was unberührt bleibt.**
- (a) Der Quelltext von VintageCoder selbst bleibt unter der GPL; wer ihn ändert und weitergibt, beachtet die GPL.
- (b) Rechte Dritter an Inhalten, die der Anwender einbindet (zum Beispiel ROMs, fremde Zeichensätze, Musik, Bilder, Disketten), bleiben unberührt; dafür ist der Anwender selbst verantwortlich.
- (c) Namen und Marken (zum Beispiel „VintageCoder“) und das Logo bleiben geschützt, soweit sie es sind; die freie Nutzung der Syntax und der Ausgaben schließt die Nutzung des Namens als Herkunftsangabe nicht ein.

**5a. Fremde Anteile, insbesondere der RMT-Player.** Diese Erklärung erfasst nicht Quelltext, der von Dritten stammt und unter einer eigenen Lizenz steht. Das gilt insbesondere für den RMT-Player (Raster Music Tracker, Dateien `rmtplayr.asm` und `rmt.inc` in den Atari-Beispielen `music_demo` und `jumprun`; © Radek Štěrba / Raster / C.P.U. 2002–2009, GNU General Public License Version 3). Wer ein Programm weitergibt, das solchen Quelltext enthält, muss die Bedingungen der jeweiligen Lizenz einhalten; für den RMT-Player heißt das, dass das Programm der GPL unterliegt. Der Rechteinhaber von VintageCoder erteilt für fremde Anteile keine zusätzlichen Erlaubnisse.

**6. Wirkung.** Diese Erklärung gilt ab ihrer Veröffentlichung für alle früheren und künftigen Fassungen, soweit der Rechteinhaber über die Rechte verfügt; sie ergänzt die Lizenz und ersetzt sie nicht.

## 4. Welche Lizenz gilt wo?

| Teil | Lizenz | Hinweis |
|---|---|---|
| Programm (Rust-Quelltext: `crates/`, Werkzeuge in `tools/`, Installer) | `GPL-3.0-or-later` | Kopfzeile `SPDX-License-Identifier` in den Dateien; zusätzliche Erlaubnis nach Abschnitt 3 |
| Systemdateien `data/sys/` | `MIT-0` | frei einsetzbar, auch ohne Urheberhinweis |
| Vorlagen `data/templates/` | `MIT-0` | wie oben |
| Beispiele `examples/` | `MIT-0` | **Ausnahme:** RMT-Player (`rmtplayr.asm`, `rmt.inc`): GPLv3, Radek Štěrba |
| Zeichensätze `data/charsets/` | `CC0-1.0` | eigene Zeichensätze „VC-Standard“ |
| Logo und Name (`branding/`) | nicht frei | siehe Abschnitt 3, Punkt 5(c) |
| Abhängigkeiten (Bibliotheken) | jeweils eigene Lizenz | siehe [THIRD-PARTY-NOTICES.md](THIRD-PARTY-NOTICES.md) |

Der Lizenztext der MIT-No-Attribution-Lizenz steht in [LICENSES/MIT-0.txt](LICENSES/MIT-0.txt), der der GPL in [LICENSE](LICENSE). In den Ordnern `data/sys`, `data/templates` und `examples` liegt je eine Datei `LICENSE` mit dem Vermerk.

## 5. Ältere Fassungen

Die Fassungen 0.1.0 und 0.2.0 wurden unter der GPL veröffentlicht. Die Erklärung in Abschnitt 3 und die Lizenz **MIT-0** für Systemdateien, Vorlagen und Beispiele gelten **ab ihrer Veröffentlichung**; sie erlauben jedem, die Dateien ab diesem Zeitpunkt unter diesen Bedingungen zu verwenden.

## 6. Kein Rechtsrat, keine Gewähr

Diese Seite erklärt die Absicht des Rechteinhabers in einfachen Worten und ist **keine Rechtsberatung**. Im Zweifel gilt der Wortlaut der Lizenztexte und der Erklärung in Abschnitt 3. VintageCoder wird ohne jede Gewährleistung bereitgestellt (siehe [LICENSE](LICENSE)).

Fragen zur Lizenz: siehe [README.md](README.md) (Kontakt und Website).
