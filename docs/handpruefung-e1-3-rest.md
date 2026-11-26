# Handprüfung E1.3 Rest (kurz)

Stand: 2026-11-21 · Branch `claude/ui-e1-3` · App: `~/Development/abnahme-ui/e1-3-rest/VintageCoder.app` (unsigniert: Rechtsklick ▸ Öffnen) · Beispielkopien in `beispiele/` (`c64/jumprun` mit `common/`). **Bitte das Projekt aus diesem Ordner öffnen** (Zuletzt-Liste zeigt sonst ein altes).

Schritt 0: Port 1420 frei, App starten. Projekt `…/e1-3-rest/beispiele/c64/jumprun` öffnen, F7.

| Nr | Handlung | Erwartet | Ergebnis |
|---|---|---|---|
| 1 | Nach dem Bau `lib/music.inc` öffnen, Ablage ▸ Probleme | Statt 15 einzelner Warnungen eine Zeile „▸ Hinweis · 15 zu weite Sprünge automatisch verlängert (`.branchlong auto`)“ mit „einzeln anzeigen“; Klick klappt die einzelnen Zeilen auf (Klick auf eine springt zur Zeile); die Statuszeile zeigt „1 Hinweis“, nicht 15 Warnungen; im Editor gepunktete statt rote Unterstreichung | |
| 2 | Maus auf den Speicherstreifen in der Statuszeile | Tooltip erscheint **oberhalb** und ist vollständig zu sehen (gilt für alle Tooltips am Fensterrand) | |
| 3 | Titelleiste: Pfad des Projekts | Lange Pfade sind **in der Mitte** gekürzt (`/Users/…/e1-3-rest/beispiele/c64/jumprun`), Anfang und Ende bleiben lesbar; Maus darauf zeigt den vollen Pfad | |
| 4 | `jumprun.asm` öffnen (lange Kommentarzeilen), Panel ⌘⌥B öffnen und schließen | Lange Zeilen brechen um; Zeilennummer, Faltmarke und Haltepunkt stehen an der **ersten** Zeile des umgebrochenen Blocks, nichts springt in die Mitte | |
| 5 | Ablage ▸ Probleme mit vielen Einträgen ganz nach unten scrollen | Die letzte Zeile ist vollständig sichtbar (Abstand am unteren Rand); ebenso in Ausgabe, Tests, Suche und Gliederung | |
| 6 | Prettyprint bei Direktiven: `.org     $0801  ; x` tippen, Enter | Leerzeichen werden zusammengefasst (`.org $0801`), Kommentar rückt nach Spalte 41; Direktiven stehen bewusst in **Spalte 1** (wie in den Beispielen), Befehle in Spalte 9. Soll eine bestimmte Art Direktive (z. B. `.byte`) auch in Spalte 9 stehen: bitte melden | |
| 7 | (offen aus E1.3) Starten ohne eingerichteten Emulator (in den Einstellungen deaktivieren oder umbenennen), F5 | Sichtbare Fehlermeldung „Starten fehlgeschlagen: …“ und Eintrag in der Ausgabe | |
