# Befunde Handtest E1.1 Teil 2, Block A: Behebung

Stand: 2026-11-20 · Branch `claude/ui-e1-1` (Quelle: `ergebnisse-handtest-e1-1-teil2.md`, Punkte 28 bis 36). Geprüft im Ablauftest `ui/tools/e2e-editor.mjs` (76 Prüfungen).

| Befund | Behebung | Prüfung |
|---|---|---|
| (a) Verkleinern: Editor bleibt breit, waagerechte Leiste; Wunsch Zeilenumbruch | Zeilenumbruch ist **Standard**, Umschalter Menü Ansicht ▸ „Zeilenumbruch im Editor“ (Befehl `umbruch`, gemerkt) | Umbruch an, bei 900 px keine waagerechte Leiste, abschaltbar |
| (b) Suchtreffer schlecht lesbar | Treffer: gelbe Fläche mit Text in Textfarbe, aktueller Treffer kräftig gelb mit dunklem Text und Rahmen; Kommentarfarbe und Kursiv gelten im Treffer nicht | Kontrast gemessen: hell 8,7 (aktuell) / 14,3, dunkel 12,1 / 7,2 (alle ≥ 4,5, WCAG AA) |
| (c) Fokus nach „Gehe zu Zeile“ | Nach dem Schließen eines Dialogs oder der Befehlspalette fällt der Fokus an den Editor, wenn sonst nichts ihn hält | Fokus auf `.cm-content`, sofort tippbar |
| (d) Faltung zu unauffällig | Dreieck 16 px, eingeklappt hervorgehoben (Akzentfarbe, Fläche), Platzhalter „…“ mit Rahmen und fett | Bild `07-faltung` |
| (e) Lesezeichen gehen nicht | In der Oberfläche gehen Umschalt+Klick, ⌘F2 und F2; **⌘F2 fängt macOS ab** (Menüleiste), F2 braucht oft Fn. Neu: Befehle `lesezeichen_umschalten` (⌘⌥K), `naechstes_lesezeichen` (⌘⌥J), `voriges_lesezeichen` (⌘⌥H) in Menü Bearbeiten und Palette, auch im Kontextmenü | Setzen, Springen (rundum), Umschalt+Klick im Rand |
| (f) Rechtsklick englisch | Eigenes deutsches Editor-Kontextmenü (Ausschneiden, Kopieren, Einfügen, Alles auswählen, Gehe zu Zeile, Lesezeichen, Haltepunkt, Alles falten/entfalten) | Menüeinträge deutsch, kein Webview-Menü |
| (g) Tooltip des Streifens unten | Bei hohen Auslösern erscheint der Tooltip am Mauszeiger | Tooltip nahe dem Zeiger |
| (h) Umbenennen: Include anpassen | `file_rename`/`dir_rename` laufen wie Verschieben über `vc_ide::reorganize` (Include-Pfade und Projektverweise, Sicherung, `tree_undo`); ohne Projektdatei bleibt es beim einfachen Umbenennen; offene Reiter werden neu gelesen, Meldung „n Include-Pfad(e) angepasst“; Umbenennen verlangt wie Verschieben vorher Speichern | Rusttest `umbenennen_passt_include_und_projektdatei_an…` und Ablauftest (Include, offener Reiter, Rückgängig) |
