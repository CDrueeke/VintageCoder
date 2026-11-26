# Handprüfung E1.2 Rest (kurz)

Stand: 2026-11-20 · Branch `claude/ui-e1-2` (Sammelzweig `claude/tauri-ui` enthält E1.1/E1.2 freigegeben) · App: `~/Development/abnahme-ui/e1-2-rest/VintageCoder.app` (unsigniert: Rechtsklick ▸ Öffnen) · Beispielkopien in `beispiele/`.

Schritt 0: Port 1420 frei (`lsof -nP -iTCP:1420 -sTCP:LISTEN` leer), App starten, Oberfläche erscheint. Sprache Deutsch, Tastatur QWERTZ.

| Nr | Handlung | Erwartet | Ergebnis |
|---|---|---|---|
| 1 | In einer `.asm`-Datei am Zeilenanfang `ld` tippen, **Tab** | Der ausgewählte Vorschlag wird übernommen (wie Enter); ohne offene Liste fügt Tab ein Tabulatorzeichen ein | |
| 2 | `.text "ab{cl` tippen, Vorschlag `clr` mit Tab oder Enter übernehmen | Es entsteht `{clr}` mit **einer** schließenden Klammer | |
| 3 | Kommentar umschalten mit **⌘⌥C** (⌘/ geht nur mit Umschalt+7 und bleibt zusätzlich) | Zeile wird Kommentar und wieder normal; mit markierten Zeilen alle | |
| 4 | Rechtsklick in den Editor | Eintrag „Kommentar umschalten“ steht im Menü und wirkt | |
| 5 | Menü Bearbeiten ▸ Formatieren | Eintrag „Kommentar umschalten“ (⌘⌥C) steht dort und wirkt | |
| 6 | Neue Datei `t.bas` mit `10 fori=1to10:next` (ohne Leerzeichen) | `for`, `to` und `next` sind als Schlüsselwörter gefärbt (wie mit Leerzeichen); `i`, `1`, `10` nicht | |
| 7 | In `t.bas` formatieren (Menü Bearbeiten ▸ Formatieren) | Leerzeichen werden wie bisher zusammengefasst, **keine Einrückung** (Entscheidung des Inhabers: im BASIC-Konzept) | |
| 8 | Fehlerkarte: `lda nirgendwo` in einer `.asm`-Datei, Maus auf die rote Wellenlinie, hell und dunkel | Karte mit „Fehler“, Code und Meldung, gut lesbar (im gebauten Fenster per Selbstmessung belegt: `ci/selftest-fehlerkarte-echtes-fenster.json`) | |
