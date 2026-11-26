# Abnahmepaket E0 (Grundlage der neuen Oberfläche)

Stand: 2026-11-20 · Zweig `claude/ui-e0` (Sammelzweig `claude/tauri-ui`), privat gepusht, nicht in `main`.

| Teil | Ort |
|---|---|
| App (gebaut, unsigniert) | `~/Development/abnahme-ui/e0/VintageCoder.app`, Beispiele nur als Kopien in `…/e0/beispiele/` |
| Handprüfliste (30 Punkte) | `docs/handpruefung-e0.md` |
| Bilder je Ansicht in de, en, xx, Hell und Dunkel (12 Ansichten × 3 Sprachen × 2 = 72) und Layoutprüfung in drei Fensterbreiten (0 Befunde) | `~/Development/abnahme-ui/e0/bilder/<de|en|xx>/`, `…/bilder/layout-befunde.txt`, Bild des echten Fensters: `…/bilder/nativ/` |
| Schriftvergleich (F2): JetBrains Mono, IBM Plex Mono, Source Code Pro, je Code, Hex-Zeile, Tracker-Zeile | `~/Development/abnahme-ui/e0/schriften/schriften-{dark,light}.png` |
| Messwerte G1 | `docs/messwerte-e0.md` (macOS, Windows und Linux gemessen; Linux nur unter Software-Rendering in einer VM) |
| Paritätsmatrix (563 Zeilen, 25 Bereiche) | `docs/paritaet-oberflaeche.md` |
| Lokale CI | `~/Development/abnahme-ui/e0/ci/` (Bau und Tests des Workspace: 1 422 Tests grün); weitere Schritte siehe unten |
| Aufbau und Arbeitsweise | `docs/ui-aufbau.md` |

## Lokale CI (alles lokal, nichts auf GitHub)

`cargo build --workspace --all-targets` grün · `cargo test --workspace` 1 422 bestanden, 0 fehlgeschlagen (darunter 8 in `vc-commands`, 7 in `vc-shell`) · `npm ci --ignore-scripts`, `npm run check` (svelte-check und tsc: 0 Fehler, 0 Warnungen), `npm test` (9 Tests), `npm run build` · `python3 tools/release.py check` (0 Fehler, jetzt auch Inhaltsrichtlinie, Prüfbrücke und Fremdadressen im Frontend) · `tools/check_schreibweise.py`, `tools/check_spdx.py` (jetzt mit `ui`), `tools/check_i18n.py` (jetzt mit Scanner für `ui/`), `tools/ui_notices.py --check`, `python3 -m unittest tools.test_gui_notices` · Clippy ist im Workspace nicht eingerichtet. Playwright-Lauf (Bilder, Layout): `ui/tools/shots.mjs`, 0 Befunde. Die Release-Datei enthält weder Prüfbrücke noch Selbstmessung (`strings`: 0 Treffer).

## Entscheidungen und Abweichungen (bitte zur Kenntnis)

1. **Typerzeugung:** `tauri-specta` gibt es für Tauri 2 nur als Vorabversion (`2.0.0-rc.*`); stattdessen ein Einstiegsbefehl `vc(cmd, args)` mit handgeschriebenen Typen und Test, der die Namenslisten (Rust und TypeScript) vergleicht. Ein eigener Erzeuger lohnt mit den Editor-DTOs in E3.
2. **TypeScript 6.0.3 statt 7:** `svelte-check` verlangt TypeScript 5 oder 6.
3. **Befehlskennungen bleiben:** In `vc-commands` heißen einige Kennungen noch `studio_*` (und Untermenü „Studio“ im Katalog). Kennungen und Schlüssel werden nie umbenannt; die sichtbaren Texte in der Befehlstabelle tragen noch das Wort „Studio“ (Katalog `cmd.ftl`, `submenu.ftl`, 19 Werte laut Konzept V6). Das Umstellen ist eine Katalogarbeit, die ich nicht still vorgenommen habe (Wortlaut vorhandener Texte nicht ändern).
4. **Gemeinsame Dateien mit der Iced-Oberfläche:** Zuletzt-Liste (`recent.json`) und Sprachwahl (`language` in den Einstellungen) liegen in denselben Dateien; „xx“ wird nie gespeichert.
5. **Neue Katalogdatei** `ui.ftl` (de und en, 114 Schlüssel); wiederverwendet: `cmd-*`, `startui-*`, `menu-*`, `key-*`.
6. **Zusätzliche Bauarten** nur für Entwicklung: Feature `testbridge` (Prüfbrücke) und `selftest` (Selbstmessung); beide stehen nicht in den Standard-Features.

## Bekannte Lücken

- **Windows und Linux:** nicht von mir gemessen. Windows ist von der Windows-Session gemessen (siehe `docs/messwerte-e0.md`; Messbild der Tabelle folgt); Linux ist unter Software-Rendering in einer VM gemessen. Unter Windows und Linux gilt zunächst die **normale Fensterleiste des Systems**; die vereinte Titelleiste mit eigenen Fensterknöpfen ist nur für macOS umgesetzt (Spike und Mockups sind macOS).
- **VoiceOver:** Rollen, Namen und Ansagen sind im DOM gesetzt (Baum, Auswahlgruppen, Tabelle mit Zeilen, Dialog, Trefferzahl), aber von mir nicht mit VoiceOver gehört; Teil 7 der Handprüfung.
- **Alt-Buchstaben der Menüs (Windows/Linux)** und das Verhalten der Kürzel mit Plus/Minus unter Windows/Linux sind ungeprüft; auf macOS kommen die Kürzel aus dem nativen Menü.
- **Rechtes Panel** (Maschine, Farben, Eigenschaften) und die **Statusleiste mit Speicherstreifen** fehlen noch (E1.3 ff.); die Statusleiste zeigt Projektname und den Hinweis „Alle Befehle“.
- **Dateibaum:** nur Anzeige, ohne Beobachten von Änderungen im Dateisystem (`fs:changed` folgt mit E1.1).
- **Befehlspalette:** Befehle, Orte und Sprachen; Dateien und Zuletzt-Projekte als Treffer folgen (E1.1, E1.4).
- **Hoher Kontrast / Systemfarben** nur über `forced-colors` grob berücksichtigt (E12).
- **Festbreitenschrift** noch nicht gebündelt (F2 offen); die Oberfläche nutzt Systemschriften.
- **Medianwerte** der Messung haben wegen WebKit nur 1 ms Auflösung (Mittelwert ist genau); Zeigerereignisse sind synthetisch.
- **Playwright** prüft bisher nur Bilder und Überlauf, noch keine Bedienabläufe (kommen mit den Editoren).
- **Der Entwicklungsbau** (`tauri dev`) wurde in dieser Sitzung nicht benutzt, nur Vite plus Prüfbrücke im Browser und der Release-Bau; die Befehle `npm run dev` und `tauri dev` in `docs/ui-aufbau.md` sind deshalb nicht in dieser Form durchgespielt.
