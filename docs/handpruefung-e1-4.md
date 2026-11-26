# Handprüfung E1.4 (Rahmen: Sitzung, Layout, Teilen, Startseite, Neues Projekt)

Stand: 2026-11-21 · Branch `claude/ui-e1-4` · App: `~/Development/abnahme-ui/e1-4/VintageCoder.app` (unsigniert: Rechtsklick ▸ Öffnen) · Beispielkopien in `beispiele/` (`c64/jumprun` mit `common/`).

Schritt 0: Port 1420 frei, App starten. Die Konfiguration (Sitzung, Layout, Zwischenkopie) liegt im Konfigurationsordner; zum Zurücksetzen `~/.config/vintagecoder/session.json`, `scratch.json`, `window.json` löschen und in der App „Layout zurücksetzen“ wählen (Befehlspalette).

## A Sitzung, Zwischenkopie, Beenden

| Nr | Handlung | Erwartet | Ergebnis |
|---|---|---|---|
| 1 | Projekt `beispiele/c64/jumprun` öffnen, drei Dateien öffnen (Reihenfolge merken), eine davon aktiv lassen, Projekt schließen, wieder öffnen | Es öffnen sich **keine** Reiter (Standard: aus) | |
| 2 | In `jumprun.vcprj` auf oberster Ebene (neben `"project"`) die Zeile `"restore_tabs": true,` eintragen, App neu starten, Projekt öffnen (bis zum Einstellungsdialog in E12 geht der Schalter nur so) | Dieselben Reiter in **derselben Reihenfolge**, die zuletzt aktive Datei ist vorn | |
| 3 | ⌘N (neuer Reiter), `lda #$42` tippen, 2 Sekunden warten, App **hart beenden** (Aktivitätsanzeige ▸ Sofort beenden), neu starten, Projekt öffnen | Dialog „Unbenannte Reiter wiederherstellen“ mit Namen und Zeichenzahl; „Wiederherstellen“ legt den Reiter mit Text als ungespeichert an; „Verwerfen“ löscht die Zwischenkopie | |
| 4 | Zwei Dateien ändern, einen unbenannten Reiter mit Text anlegen, ⌘Q | „Ungespeicherte Änderungen“ mit **Liste** jedes Reiters (unbenannte mit Hinweis); Abbrechen bleibt im Programm; „Alle speichern“ fragt bei unbenannten nach dem Namen | |
| 5 | Dasselbe mit dem roten Fensterknopf und mit „Projekt schließen“ | Gleiche Liste und gleiches Verhalten | |

## B Layout, Maximieren, Streifen

| Nr | Handlung | Erwartet | Ergebnis |
|---|---|---|---|
| 6 | Ablage öffnen (⌘J), Trennlinie über der Ablage mit der Maus ziehen; Seitenleiste und Panel (⌘⌥B) ebenso | Höhe bzw. Breite folgt der Maus; Doppelklick auf die Linie stellt die Standardgröße her; mit Tab auf die Linie und Pfeiltasten (Umschalt = groß), Pos1/Ende | |
| 7 | App beenden und neu starten | Seitenleiste, Ablage (mit Reiter), Panel und alle Größen wie zuletzt | |
| 8 | Befehlspalette ▸ „Layout zurücksetzen“ | Alles auf der Vorgabe (Seitenleiste auf, Ablage und Panel zu); Hinweis erscheint | |
| 9 | Knopf rechts im Kopf von Ablage, Dokumentenfeld, Seitenleiste, Panel; Doppelklick auf den Kopf der Ablage | Der Bereich füllt die Fläche, die anderen werden Streifen bzw. verschwinden; derselbe Knopf stellt wieder her; das gemerkte Layout ändert sich dadurch nicht | |
| 10 | ⌘⇧Enter mit Cursor im Editor, in der Ablage, in der Seitenleiste | Der Bereich unter dem Fokus wird maximiert, erneut: wiederhergestellt | |
| 11 | Seitenleiste zuklappen (⌘B) | Streifen mit drei Symbolen: aufklappen, Dateien, Suche; „Suche“ öffnet die Suche in der Ablage | |
| 12 | Zen-Modus (⌘⌥Z) | Seitenleiste, Panel, **Ablage** und Statuszeile weg (neu: Ablage klappt ein); Maximierung endet | |

## C Geteilte Ansicht

| Nr | Handlung | Erwartet | Ergebnis |
|---|---|---|---|
| 13 | Drei Dateien öffnen, ⌘⌥T | Zwei Hälften nebeneinander, die aktive Datei steht rechts, jede Hälfte hat eigene Reiterleiste und Fokusstrich | |
| 14 | In beiden Hälften tippen, ⌘S | Es ändert sich nur die Datei der jeweiligen Hälfte; Statuszeile folgt der fokussierten Hälfte; Hover, Vervollständigung und Prettyprint arbeiten in **beiden** Hälften | |
| 15 | Reiter per Maus in die andere Hälfte ziehen; Reiter ziehen und am **rechten Rand** der ungeteilten Fläche ablegen | Wechselt die Hälfte bzw. teilt (Zone „Rechts ablegen“ erscheint) | |
| 16 | Befehlspalette ▸ „In die andere Hälfte schieben“; letzten Reiter einer Hälfte schließen; ⌘⌥T erneut | Teilung endet, wenn eine Hälfte leer wird; erneutes ⌘⌥T vereint | |
| 17 | Fenster schmaler als etwa 1300 Punkte ziehen | Unter 1000 Punkten Breite der Dokumentenfläche stehen die Hälften **untereinander** | |
| 18 | Fenster auf Mindestgröße (900 Punkte), Seitenleiste offen | Seitenleiste (und Panel) **liegen über** der Fläche; Klick in die Fläche schließt sie, der Streifen öffnet sie wieder | |

## D Startseite, Neues Projekt, Kopieren

| Nr | Handlung | Erwartet | Ergebnis |
|---|---|---|---|
| 19 | Projekt schließen (Startseite) | Kopf mit Version und vier Knöpfen; Zuletzt-Liste mit Zielsystem, Sprache, Emulator, Alter; Stern **heftet an** (Eintrag rückt nach vorn, bleibt nach Neustart), × entfernt; Ordner umbenennen: „Ordner fehlt“ und gesperrt | |
| 20 | „Beispiel öffnen“ | Liste der Beispielprojekte mit „Öffnen“ und „Kopieren …“ | |
| 21 | „Kopieren …“ bei `c64 / jumprun`, Speicherort wählen, „Kopieren“ | Name „jumprun Kopie“ vorgeschlagen; Anzahl und Größe genannt; Kopie entsteht und wird geöffnet; zweites Mal mit demselben Namen: Hinweis „gibt es schon“, Knopf gesperrt | |
| 22 | „Neues Projekt“ (⌘⌥N): Zielsystem wechseln (VC-20, MEGA65), Assembler/BASIC, Vorlage, Name `a b` | VC-20 zeigt Speicherausbau, MEGA65 keine Videonorm, BASIC den Dialekt; ungültiger Name: roter Hinweis und „erstellen“ gesperrt; Vorschau der Dateistruktur folgt den Wahlen | |
| 23 | Neues Projekt anlegen (README und Git an), Speicherort wählen | Projekt entsteht, wird geöffnet; Ordner enthält `.vcprj`, `README.md`, `.gitignore`; gleicher Name noch einmal: **Fehler bleibt im Dialog** | |
| 24 | Befehlspalette: „Projekt öffnen: …“ tippen | Zuletzt geöffnete Projekte als Treffer; Enter öffnet | |
| 25 | „Projektdatei (.vcprj) wählen …“ | Dateidialog, öffnet das Projekt | |

## E Fenster

| Nr | Handlung | Erwartet | Ergebnis |
|---|---|---|---|
| 26 | Fenster verschieben und in der Größe ändern, App beenden, neu starten | Größe und Lage wie zuletzt (Datei `window.json`); maximiert merkt nur das Kennzeichen | |
| 27 | Zweiten Bildschirm abstecken, auf dem das Fenster zuletzt war, App starten | Fenster erscheint mittig auf dem Hauptbildschirm | |
| 28 | VoiceOver an (⌘F5): Tab durch die Oberfläche | Bereiche tragen Namen (Seitenleiste, Dokumentenfeld links/rechts, Ablage, Maschine); Trennlinien melden Wert und lassen sich mit Pfeilen verstellen; Dialoge kündigen ihren Titel an | |

Nicht Teil dieses Pakets (bewusst zurückgestellt, siehe `docs/paritaet-oberflaeche.md`): Startbild, Banner „Einrichtung unvollständig“, Feedback/Assistent (E12), neue Datei je Editor-Art auf der Startseite (E3 bis E5), Hardware-Probe (E12), Schalter „Sitzung wiederherstellen“ und „Absturzsicherung“ als Dialogzeile (E12; bis dahin `.vcprj` bzw. Kernbefehl).
