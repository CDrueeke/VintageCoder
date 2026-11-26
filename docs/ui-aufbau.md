# Neue Oberfläche (Tauri): Aufbau und Arbeitsweise

Stand: Etappe E0 (2026-11-20). Maßgeblich ist das Konzept `assets/docs/Konzept-Umbau-Oberflaeche-Tauri.md` (lokal, nicht in Git). Diese Seite beschreibt, was im Repository liegt und wie man damit arbeitet. Die Iced-Crates (`vc-gui`, `vc-gui-ui`, `vc-gui-editor`) sind eingefroren und bleiben bis E13 unverändert.

## Teile

| Teil | Ort | Aufgabe |
|---|---|---|
| `vc-commands` | `crates/vc-commands` | UI-freie Befehlstabelle (`BEFEHLE`, `UNTERMENUES`, Kürzel, Alt-Buchstaben, Filter der Palette). Herausgelöst aus `vc-gui-ui/src/befehle.rs`; die Iced-Kopie bleibt bis E13. Befehlskennungen und Katalogschlüssel `cmd-<id>` sind unverändert (auch `studio_*`: Kennungen werden nie umbenannt, nur Texte) |
| `vc-shell` | `crates/vc-shell` | Tauri 2: Hauptfenster, Befehle (`cmds.rs`, ein Einstieg `vc` mit Befehlsnamen und JSON), `DocStore` (Gerüst für E1 ff.), Ereignisbus (`events.rs`), native Menüleiste aus der Tabelle (`menu.rs`, Neuaufbau beim Sprachwechsel), Prüfbrücke (`testbridge.rs`) |
| `ui/` | `ui/` | Svelte 5, TypeScript, Vite; Komponenten in `src/lib/components`, Ansichten in `src/views`, Kern-Anbindung `src/lib/api.ts`, Katalog-Umschlag `src/lib/i18n.ts` |
| Kataloge | `crates/vc-i18n/catalog/{de,en}/ui.ftl` | Alle Texte der neuen Oberfläche (Schlüssel `<bereich>-<ort>-<zweck>`); vorhandene Schlüssel (`cmd-*`, `startui-*`, `menu-*`, `key-*`) werden wiederverwendet |

## Zusammenspiel

- **Rust bleibt die Quelle der Wahrheit.** Die Oberfläche hält nur Anzeigezustand. Fehler des Kerns sind `{code, args}`; die Oberfläche formatiert sie mit dem Katalog (`err-<code>`).
- **Befehle** gehen alle durch `cmds::dispatch(app, name, json)`. Tauri (`vc`) und die Prüfbrücke rufen dieselbe Funktion; `BEFEHLE` (Rust) und `COMMANDS` (`api.ts`) werden von einem Test verglichen, ebenso `EREIGNISSE`/`EVENTS`. **Typerzeugung:** `tauri-specta` gibt es für Tauri 2 nur als Vorabversion (`2.0.0-rc.*`) und setzt Einzelbefehle voraus; deshalb sind die DTO-Typen in `api.ts` von Hand geschrieben und der Namensvergleich fängt Abweichungen. Ein eigener Erzeuger lohnt erst mit den Editor-DTOs (E3).
- **Sprache:** `i18n_ftl(lang)` liefert die eingebetteten `.ftl`-Quellen (Deutsch immer, Englisch auf Wunsch); `@fluent/bundle` formatiert lokal. Rückfall Deutsch, dann sichtbarer Hinweis `⟦schlüssel⟧`. Pseudo-Sprache `xx` wird in `pseudo.ts` aus dem Deutschen erzeugt (gleiche Abbildung wie `vc-i18n/src/pseudo.rs`). `lang_set` speichert Deutsch/Englisch/Automatisch in den Einstellungen (gemeinsam mit der Iced-Oberfläche), baut die Menüleiste neu und sendet `lang:changed`.
- **Tastatur:** Menükürzel kommen als Ereignis `menu:cmd` aus dem nativen Menü. Editor-Eingaben gehen später nach `KeyboardEvent.code` (`keys.ts`); die Beschriftung folgt dem aus Ereignissen gelernten Layout.
- **Sicherheit:** Inhaltsrichtlinie in `crates/vc-shell/tauri.conf.json` (`default-src 'self'`, keine Fremdadressen); die Berechtigungen des Fensters (`capabilities/default.json`) umfassen nur `core:default`, Fensterziehen und Maximieren; kein Datei-, Netz- oder Shell-Zugriff, kein Updater. Die Ordnerauswahl läuft in Rust (`tauri-plugin-dialog`), nicht über das Frontend.

## Entwickeln

```bash
cd ui && npm ci --ignore-scripts          # einmalig
npm run dev                               # Vite auf 127.0.0.1:1420
cd ../crates/vc-shell && ../../ui/node_modules/.bin/tauri dev        # Fenster (lädt Vite)
```

**Prüfbrücke** (nur Entwicklung): `VC_TESTBRIDGE=17420 cargo run -p vc-shell --features testbridge` startet den Kern mit WebSocket auf `127.0.0.1:17420`; die Oberfläche läuft dann auch im Browser: `http://127.0.0.1:1420/?bridge=17420` (nur mit `npm run dev`; im Auslieferungsbau fehlt der Brückencode, `release.py check` prüft das). `VC_NO_SAVE=1` verhindert, dass die Sprachwahl in die Einstellungen geschrieben wird; `VC_RECENT=<Datei>` leitet die Zuletzt-Liste um.

**Bilder und Layoutprüfung:** `node ui/tools/shots.mjs <Ausgabe> <Projektkopie>` (Playwright/WebKit gegen die laufende Brücke): alle Ansichten in de, en, xx, Hell und Dunkel, dazu Überlaufprüfung in drei Fensterbreiten (`layout-befunde.txt`). Playwright-Browser einmalig: `cd ui && npx playwright install webkit`.

**Bau der App:** `cd crates/vc-shell && ../../ui/node_modules/.bin/tauri build --config tauri.release.conf.json` (Ergebnis `target/release/bundle/macos/VintageCoder.app`). Die Zusatzdatei legt den Ordner `data/` (Prozessortabellen, Ziele, Direktiven, Zeichen) als Ressource ins Bündel (`Contents/Resources/data`), den die Sprachdienste brauchen. Sie steht bewusst nicht in `tauri.conf.json`: Tauri würde `data/` sonst bei jedem Entwicklungsbau nach `target/debug/data` kopieren, und die Tests der alten Oberfläche (die den Datenordner neben dem Programm suchen) fänden diesen statt des Projektordners.

**Selbstmessung (G1):** Frontend mit `VITE_SELFTEST=1` bauen und den Kern mit `--features selftest`: `VITE_SELFTEST=1 tauri build --features selftest --no-bundle`; Start mit `VC_SELFTEST_OUT=<Datei> target/release/vintagecoder` schreibt Start- und Rundlaufzeiten als JSON und beendet sich. Nur für Abnahme und Messung; nie im Auslieferungsbau.

## Rahmen (E1.4)

- **Sitzung und Zwischenkopie**: `ui/src/lib/sitzung.svelte.ts` (merkt Reiter je Projekt, stellt sie wieder her, sichert unbenannte Reiter); Kern: `vc-shell/src/sitzung.rs` auf `vc-ide-model::{session, scratchsave}`.
- **Layout**: `ui/src/lib/layout.ts` (reine Regeln: Maximieren, Größen, Breitenklassen, gemerkter Teil), `layout.svelte.ts` (Zustand, Merken in der Oberfläche, Zurücksetzen, Fokus-Maximieren), `components/Splitter.svelte`, `AreaButton.svelte`.
- **Geteilte Ansicht**: `ui/src/lib/groups.ts` (reine Gruppenlogik, Entsprechung zu `vc-gui/src/gruppen.rs`), `editor.svelte.ts` (`gstate`, `focusGroup`, `splitToggle`, `moveToGroup`); je Hälfte eine `TabBar` und ein `CodeEditor` (`group`).
- **Neues Projekt, Startseite, Kopieren**: `vc-shell/src/{neuprojekt,start}.rs` (Logik wie in der alten Oberfläche, ohne Oberfläche), `ui/src/lib/{neuprojekt,kopieren}.svelte.ts`, `components/{NewProjectDialog,CopyDialog}.svelte`, `views/StartView.svelte`.
- **Fenster**: `vc-shell/src/fenster.rs` (`window.json`).

## Lokale CI (zusätzlich zu den Rust-Schritten)

```bash
cd ui && npm ci --ignore-scripts && npm run check && npm test && npm run build
python3 tools/check_i18n.py            # prüft auch ui/src (Schlüssel, Parameter, feste deutsche Texte)
python3 tools/ui_notices.py --check    # npm-Lizenzen und Paketbudget, THIRD-PARTY-NOTICES.md aktuell
python3 tools/release.py check         # prüft auch Inhaltsrichtlinie, Brücke und Fremdadressen im gebauten Frontend
```
