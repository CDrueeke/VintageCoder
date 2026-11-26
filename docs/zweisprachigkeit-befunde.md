# Zweisprachigkeit der Oberfläche: Befunde (Stand 2026-11-24, nur Zählung und Fundstellen)

Auftrag der Hauptsession: prüfen, ohne zu übersetzen. Rohdaten: Katalogvergleich (Skript im Verlauf), `tools/check_i18n.py`.

## 1. Kataloge
- `catalog/de` und `catalog/en`: je **6620** Schlüssel, **0** fehlen, **0** überzählig. Der englische Katalog ist vollständig, aber maschinell übersetzt (nicht abgenommen).
- **290** Schlüssel haben im Englischen denselben Text wie im Deutschen: überwiegend Namen, Kürzel und Einheiten; größere Gruppen in `ui.ftl` (79), `studio.ftl`/`songstudio.ftl` (je 15), `gui.ftl` (12), `texte.ftl`, `projeinst.ftl` (je 11). Jeder davon ist zu sichten (Name oder vergessen?). Liste: Skriptausgabe, auf Wunsch als Datei.
- **1** Schlüssel mit deutschen Wörtern im englischen Text: `info-urheber`.
- `tools/check_i18n.py` findet **0** feste deutsche Texte in `ui/src`; dazu stehen **173** begründete Ausnahmen in `tools/i18n_ausnahmen.txt` (Namen, Dateiformate, Bezeichner).

## 2. Deutsche Texte im Rust-Teil außerhalb des Katalogs (grobe Zählung, `t!(` ausgeschlossen)
Rund **59** Stellen mit Umlauten oder deutschen Funktionswörtern in Zeichenketten, vor allem `vc-ide`: `help.rs` (18, Themennamen wie „Überblick“, „Tastenkürzel“), `debug.rs` (6), `listen.rs` (4), `cond.rs` (4), `streams.rs` (3), `roms.rs` (3), `args.rs` (3, Beschreibungen im Parameterkatalog), `vc-shell/lsp.rs` (3). Diese erscheinen auch in der englischen Oberfläche deutsch. Nicht gezählt: Daten in `data/` (Kataloge, Hilfe, Vorlagen), die eigene Übersetzungsdateien haben.

## 3. e2e-Läufe in englischer Oberfläche
- Alle 33 Läufe setzen die Sprache am Anfang selbst (`setLanguage("de")`); eine Umgebungsvariable dafür gibt es noch nicht.
- Von den 33 Läufen suchen **12** stark nach deutschen Beschriftungen (`getByRole(name)`, `getByText`, `hasText`, Regex): `disk` (31 Stellen), `start` (28), `editor` (24), `neuprojekt` (21), `sprite` (20), `turbo` (18), `bau` (16), `einstellungen` (13), `sitzung` (6), `tracker`, `lsp`, `include` (je 4). Die übrigen 21 nutzen `data-…`-Kennungen und liefen mit einer Sprachvariable unverändert.
- Vorschlag: Variable `VC_E2E_LANG` (de|en) in `e2e-alle.sh`; Beschriftungen in den 12 Läufen über eine kleine Hilfe `T(key)` aus dem Katalog holen (wie in `handbuchbilder.mjs`). Aufwand grob 4 bis 6 Stunden. Die Prüfung `Keine Schlüssel im Text` gibt es schon (`e2e-projekt`).

## 4. Stellen, die bei Sprachwechsel zur Laufzeit nicht neu zeichnen
Das Gerüst stimmt: `t()` liest `state.rev`; das Ereignis `lang:changed` lädt die Katalogtexte und die Befehlstabelle neu, der Kern baut die Menüleiste neu und setzt die Sprache der Sprachdienste. Offen sind Texte, die einmal **vom Kern oder beim Erzeugen** entstehen und danach stehen bleiben:
1. **CodeMirror:** Suchleistentexte (`phrases()` in `cm.ts`, beim Erzeugen jedes Reiter-Zustands) und `aria`-Beschriftungen; bestehende Reiter behalten die alte Sprache bis zum Neuöffnen. Faltmarken-Tooltip (`cm.ts`) bis zum nächsten Zeichnen.
2. **Vom Kern geladene Listen mit fertigem Text:** Zuletzt-Liste („vor 3 Min.“), Beispiele (Beschreibungen), erkannte Programme (`tools_list`), Projektseiten (`projekt_info`: Meldungen, Warnungen), ROMs (`roms_info`: Statuswörter, Rollennamen), Ziele (`ziele_info`), Registry/Pakete, Info/Update, Assistent (`wiz_*`: Beschriftungen aus dem Kern), Hilfe (Themenliste, geöffnetes Thema), Disk-/Hex-/Speicher-Ansichten (Befunde), Tests-Ergebnisse, Debug-Meldungen. Nur Turbo lädt bei `lang:changed` neu.
3. **Bau-Protokoll und Problemliste:** Zeilen entstehen in der Sprache des Bauzeitpunkts (natürlich; neu bauen erzeugt neue).
Abhilfe (Vorschlag): ein `langRev`-Zähler, den die betroffenen Module beobachten und dann neu laden (je Modul eine Zeile), und `phrases()`/`aria` über eine Zustandsänderung neu setzen (Aufwand 3 bis 5 Stunden).

## 5. Beobachtet beim Probelauf `handbuchbilder.sh --lang en`
Der Sprite-Editor zeigt in der englischen Oberfläche die Farbrolle „Sprite-Farbe“ (und „Multicolor 1/2“) deutsch: die Rollennamen kommen aus dem Kern (Medienprofil), nicht aus dem UI-Katalog. Weitere Funde dieser Art ergeben sich, wenn die Handbuchbilder und `bilder-*.mjs` einmal komplett mit `--lang en` laufen (zählt nicht zu den 59 Stellen oben).
