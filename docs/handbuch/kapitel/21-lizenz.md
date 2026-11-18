---
id: lizenz
titel: Lizenz und freie Nutzung
reihenfolge: 21
stufe: Einsteiger
zeit: 10 Minuten
siehe_auch: [willkommen, assembler, bauen-starten, kuerzel-faq]
referenz: [license, license-build, about]
---

# Lizenz und freie Nutzung

**Ziel:** Du weißt, was du mit VintageCoder, mit der Sprache VCASM und mit den Dateien tun darfst, die du damit erzeugst, und wann ein Hinweis auf eine fremde Lizenz erscheint.

## Das Wichtigste in drei Sätzen

1. **Was du mit VintageCoder baust, gehört dir:** Programme, Quelltexte, Diskettenabbilder, Grafiken, Musik und alle anderen Dateien darfst du beliebig verwenden und weitergeben, auch kommerziell, auch ohne deinen Quelltext offenzulegen und ohne Hinweis auf VintageCoder.
2. **Die Sprache VCASM und die Dateiformate sind frei.** Jeder darf sie in eigenen Werkzeugen umsetzen, und `vcc` und `vc-lsp` darfst du aus jeder Software aufrufen, auch aus nicht freier.
3. **Die GPL gilt für den Quelltext von VintageCoder selbst** (Version 3 oder später). Wer das Programm ändert und weitergibt, beachtet sie.

Der Wortlaut steht in der Datei **LICENSING.md** (im Installationsordner unter `share/doc` bzw. `Resources`); verbindlich ist ihr Abschnitt „Erklärung des Rechteinhabers“. Hier folgt dieselbe Aussage in Worten für den Alltag. Eine Rechtsberatung ist das nicht.

## Systemdateien, Vorlagen und Beispiele: MIT-0

Die Systemdateien (`data/sys`), die Vorlagen (`data/templates`) und die Beispielprojekte (`examples`) stehen unter **MIT-0** (MIT ohne Namensnennung). Du darfst sie in eigene Programme einsetzen, ändern und weitergeben, ohne einen Urheberhinweis aufzunehmen. Der Text der Lizenz liegt im Ordner `LICENSES` (`MIT-0.txt`).

## Fremdanteile behalten ihre Lizenz

Die Freigabe gilt nicht für Quelltext, den andere geschrieben haben. Das wichtigste Beispiel ist der **RMT-Player** der Atari-Beispiele `music_demo` und `jumprun` (Dateien `rmtplayr.asm` und `rmt.inc`). Er stammt von Radek Štěrba (Raster / C.P.U.) und steht unter der GPL Version 3. Ein Programm, das ihn enthält, darfst du nur nach den Bedingungen dieser Lizenz weitergeben, also mit dem Quelltext des ganzen Programms unter der GPL. Willst du das nicht, verwende einen anderen Spieler.

Ebenso bleiben die Rechte an ROMs, fremden Zeichensätzen, Musik und Bildern bei ihren Inhabern, wenn du sie einbindest. VintageCoder liefert keine ROMs mit.

## Der Hinweis beim Übersetzen

Beim Übersetzen liest VintageCoder die Zeile `SPDX-License-Identifier:` in den Kopfzeilen deiner Quelldateien. Steht in einer Datei eine Copyleft-Lizenz (GPL, LGPL, AGPL), erscheint im **Protokoll** und unter **Probleme** der Hinweis **L001**. Er ist **kein Fehler**: Er erinnert dich nur daran, dass du beim Weitergeben des Programms die Bedingungen dieser Lizenz einhalten musst. Beim Beispiel `music_demo` (RMT-Player) siehst du ihn, beim Beispiel `prallball` nicht.

Eine Liste aller Dateien mit Lizenz und Urheber gibt `vcc build --license-report`; sie steht auch in `build/LICENSES.txt`. Wer eine Lizenz ausschließen will, trägt sie in der Projektdatei unter `licenses` ▸ `deny` ein, dann scheitert der Bau. Die Einzelheiten und Grenzen stehen in [Lizenzhinweis beim Übersetzen](hilfe:license-build). Das Ergebnis ist nur so gut wie die Kopfzeilen deiner Dateien.

## Namen und Logo

Der Name und das Logo „VintageCoder“ sind nicht frei. Du darfst aber wahrheitsgemäß sagen, dass ein Programm mit VintageCoder gebaut wurde.

## Wie geht es weiter?

- Die Seite zum Nachschlagen: [Lizenz und freie Nutzung](hilfe:license).
- Der Hinweis beim Übersetzen im Detail: [Lizenzhinweis beim Übersetzen](hilfe:license-build).
- Bauen und Starten: [Bauen und Starten](kapitel:bauen-starten).
