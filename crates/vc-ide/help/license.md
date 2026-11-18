# Lizenz und freie Nutzung

Kurz gesagt: **Was du mit VintageCoder baust, gehört dir.** Die ausführliche Fassung steht in der Datei **LICENSING.md** (im Installationsordner unter `share/doc` bzw. `Resources`, im Quelltext im Wurzelordner); der verbindliche Teil ist ihr Abschnitt „Erklärung des Rechteinhabers“.

## Was du darfst

- **Deine Ausgaben sind frei.** Programme (PRG, XEX, BIN, CRT), Diskettenabbilder, Quelltexte, Projekt- und Assetdateien, Grafiken, Berichte und alle anderen Dateien, die du mit VintageCoder erzeugst, darfst du beliebig verwenden und weitergeben: auch kommerziell, auch ohne deinen Quelltext offenzulegen, auch ohne Hinweis auf VintageCoder.
- **Die Sprache VCASM und die Dateiformate sind frei.** Jeder darf sie in eigenen Werkzeugen (Assemblern, Editoren, Konvertern, Emulatoren) umsetzen, unabhängig von der Lizenz von VintageCoder.
- **`vcc` und `vc-lsp` darfst du aus beliebiger Software aufrufen** (Editoren, Build-Systeme, automatische Prüfungen), über die Kommandozeile, Dateien, Standardein- und -ausgabe oder das Language-Server-Protokoll.
- **Systemdateien, Vorlagen und Beispiele** (`data/sys`, `data/templates`, `examples`) stehen unter **MIT-0** („MIT ohne Namensnennung“): Du darfst sie ohne Auflagen in eigene Programme übernehmen, ändern und weitergeben, auch ohne Urheberhinweis.

## Wofür die GPL gilt

Der Quelltext von VintageCoder selbst (die Entwicklungsumgebung, `vcc`, `vc-lsp`) steht unter der **GNU General Public License, Version 3 oder später**. Wer das Programm ändert und weitergibt, beachtet die GPL: Quelltext der geänderten Fassung unter der GPL, Lizenzhinweise stehen lassen.

## Was nicht dazugehört

- **Fremdanteile** behalten ihre eigene Lizenz. Das bekannteste Beispiel ist der **RMT-Player** (`rmtplayr.asm`, `rmt.inc` in den Atari-Beispielen `music_demo` und `jumprun`): Er steht unter der GPL Version 3. Ein Programm, das ihn enthält, darfst du nur nach den Bedingungen dieser Lizenz weitergeben (unter anderem mit dem Quelltext des ganzen Programms). Wenn du das nicht willst, nimm einen anderen Spieler. Beim Übersetzen weist VintageCoder darauf hin, siehe [Lizenzhinweis beim Übersetzen](help:license-build).
- **Rechte Dritter** an Inhalten, die du einbindest (ROMs, fremde Zeichensätze, Musik, Bilder), bleiben bestehen; dafür bist du selbst verantwortlich. VintageCoder liefert keine ROMs mit.
- **Name und Logo** „VintageCoder“ sind nicht frei. Du darfst aber wahrheitsgemäß sagen, dass ein Programm mit VintageCoder gebaut wurde.

Auf der Kommandozeile zeigt `vcc --license` einen kurzen Hinweis. Die Drittanbieter-Hinweise (`THIRD-PARTY-NOTICES.md`) erreichst du über **Hilfe ▸ Über VintageCoder**, siehe [Über VintageCoder](help:about).

Dies ist eine Erklärung in einfachen Worten und keine Rechtsberatung.
