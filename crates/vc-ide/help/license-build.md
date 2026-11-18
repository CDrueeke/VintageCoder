# Lizenzhinweis beim Übersetzen

Beim Übersetzen eines Assembler-Programms liest VintageCoder in jeder eingebundenen Quelldatei die Zeile `SPDX-License-Identifier:` im Kopf der Datei (und, falls vorhanden, die Urheberzeile). So weiß es, unter welchen Lizenzen der Quelltext steht, der in dein Programm eingeht.

```
; SPDX-License-Identifier: MIT-0
; SPDX-FileCopyrightText: 2026 Erika Muster
```

Enthält das Programm Quelltext unter einer **Copyleft-Lizenz** (GPL, LGPL, AGPL), erscheint im Protokoll (und als Information unter **Probleme**) ein sachlicher Hinweis:

> Hinweis (L001): Das Programm enthält Quelltext unter GPL-3.0-only (lib/rmtplayr.asm, …). Wer das Programm weitergibt, muss die Bedingungen dieser Lizenz einhalten (u. a. den Quelltext bereitstellen). Mit --license-report erhältst du die Liste.

Der Hinweis ist **kein Fehler** und kein Rechtsurteil; der Bau gelingt wie sonst. Das bekannte Beispiel ist der RMT-Player im Atari-Beispiel `music_demo`; das Beispiel `prallball` (nur MIT-0) löst nichts aus.

## Bericht
`vcc build --license-report` listet alle beitragenden Dateien mit Lizenz, Urheber und Fundstelle (Kopfzeile oder Ordnervermerk), auch die unter MIT-0, CC0 oder eigene, und schreibt dieselbe Liste nach `build/LICENSES.txt`. Mit `--json` kommt der Bericht als JSON. Die Systemdateien der Zielsysteme erscheinen als `data/sys/…`.

## Erlauben und sperren
In der Projektdatei (`.vcprj`) steht dafür der Abschnitt `licenses`:

```json
"licenses": { "deny": ["GPL-3.0-only", "GPL-3.0-or-later"], "allow": ["LGPL-2.1-only"] }
```

- **`deny`**: Der Bau scheitert mit Fehler `L002`, wenn ein beitragender Quelltext unter einer dieser Lizenzen steht, und es entsteht kein Ergebnis. Ein `*` am Ende wirkt als Platzhalter (`GPL-*`).
- **`allow`**: Für diese Lizenzen erscheint kein Hinweis `L001`. Im Bericht stehen sie weiterhin.
- Auf der Kommandozeile sperrt `vcc build --deny-license GPL-3.0-only,GPL-3.0-or-later` für einen Lauf.

## Grenzen
- Das Ergebnis ist **nur so gut wie die Kopfzeilen**. Dateien ohne `SPDX-License-Identifier:` bleiben neutral (sie fehlen im Bericht und lösen nichts aus); eine falsche Kopfzeile führt zu einem falschen Ergebnis.
- Als Ersatz für eine fehlende Kopfzeile gilt ein **Ordnervermerk**: eine Datei `LICENSE` oder `LICENSE.txt` im Ordner der Datei oder darüber, die `SPDX-License-Identifier: …` oder `(SPDX: …)` enthält (so tragen die mitgelieferten Systemdateien, Vorlagen und Beispiele MIT-0).
- **Binärdaten** (Musik-, Bild- und Programmdateien, die du mit `.incbin` einbindest) werden nicht erkannt, ebenso wenig BASIC-Programme.
- Die Prüfung gilt für Assembler-Quelltexte; sie ersetzt keine Prüfung deiner Fremdanteile.

Mehr zur Lizenz von VintageCoder und zu deinen Ausgaben: [Lizenz und freie Nutzung](help:license).
