---
id: oberflaeche
titel: Die Oberfläche im Überblick
reihenfolge: 3
stufe: Einsteiger
zeit: 20 Minuten
siehe_auch: [erstes-projekt, projekte, editor]
referenz: [layout, settings, shortcuts, overview]
---

# Die Oberfläche im Überblick

**Ziel:** Du kennst den Aufbau des VintageCoder-Fensters und weißt, wofür die Hauptbereiche, die Seitenleiste und die Ablage unter dem Editor da sind. Du kannst Bereiche ein- und ausblenden, einen Bereich groß machen, den **Zen-Modus** benutzen, die Befehlspalette aufrufen und Farbschema und Sprache ändern.

## Das brauchst du

- Ein geöffnetes Projekt, zum Beispiel das aus dem Kapitel [Dein erstes Projekt](kapitel:erstes-projekt). Ist kein Projekt offen, zeigt der Hauptbereich **Code** die **Startseite** (neues Projekt, zuletzt geöffnete Projekte, Beispiele). Du holst sie mit dem Befehl **Startseite** (Menü **Datei**) jederzeit zurück; das schließt das Projekt.

## Das Gerüst des Fensters

Oben stehen die **Kopfzeile** und darunter, wo es sie gibt, eine **zweite Reihe**. Darunter liegt die Arbeitsfläche, ganz unten die **Statusleiste**. Die Hauptbereiche wechselst du in der Kopfzeile; was die Arbeitsfläche zeigt, hängt vom Hauptbereich ab.

![Das Hauptfenster im Bereich „Code“. Oben die Kopfzeile mit dem Projektnamen und den Hauptbereichen von „Startseite“ bis „Debug“. Links die Seitenleiste „PROJEKT“ mit dem Dateibaum, in der Mitte der Editor mit jumprun.asm, darunter die Ablage mit dem Reiter „Ausgabe“, rechts das Panel „Maschine“, ganz unten die Statusleiste.](bild:ui-gesamt "Das Hauptfenster im Überblick: Kopfzeile, Seitenleiste, Editor, Ablage, Panel Maschine und Statusleiste.")

| Zone | Wofür sie da ist |
|---|---|
| **Menüleiste** | Datei, Bearbeiten, Ansicht, Projekt, Hilfe. Auf dem Mac liegt sie in der Menüleiste des Systems. Lange Menüs (Bearbeiten, Ansicht, Projekt) haben Untermenüs. Nicht mögliche Einträge sind grau (zum Beispiel ohne offenes Projekt). |
| **Kopfzeile** | Knopf für die Seitenleiste, Projektname mit Pfad, die **Hauptbereiche**, die Knöpfe **Bauen**, **Starten** und (solange ein Programm läuft) **Beenden**, der **Ziel**-Knopf, die Menüs für **Sprache** und **Farbschema** und das Zahnrad für die Einstellungen und das Suchsymbol für die Befehlspalette |
| **Zweite Reihe** | Unterbereiche des gewählten Hauptbereichs (bei **Projekt**, **Grafik** und **Daten**) |
| **Seitenleiste** (links, nur im Hauptbereich **Code**) | der Dateibaum des Projekts |
| **Editor** (Mitte) | die geöffneten Dateien als **Reiter** |
| **Ablage** (unter dem Editor) | **Probleme**, **Ausgabe**, **Tests**, **Suche**, **Gliederung** |
| **Panel Maschine** (rechts, nur im Hauptbereich **Code**) | Ziel, Zustand und Ergebnis des letzten Baus; in einer Debug-Sitzung auch Register, Flags und das Bild des Emulators |
| **Statusleiste** | Projektname, Art der Datei und Zeilenende, Zahl der Probleme, Zeile und Spalte, Zustand des Baus, bei laufender Debug-Sitzung der Programmzähler, bei einer neuen Version der Update-Hinweis und der Hinweis auf die Befehlspalette |

## Die Hauptbereiche

In der Kopfzeile stehen die Hauptbereiche nebeneinander. Ein Klick wechselt den Bereich; der gewählte ist hervorgehoben.

| Hauptbereich | Wofür er da ist |
|---|---|
| **Projekt** | Einstellungen und Überblick für das ganze Projekt. Zweite Reihe: **Übersicht**, **Allgemein**, **Ziele**, **Quellen**, **Bauen und Starten**, **Diskette**, **Weitergeben**. Mehr dazu im Kapitel [Projekte und Dateien](kapitel:projekte). Kürzel: **Cmd/Strg+Alt+0** |
| **Code** | Quelltexte schreiben (Seitenleiste, Editor, Ablage, Panel Maschine) und ohne Projekt die Startseite. Mehr dazu im Kapitel [Der Code-Editor](kapitel:editor) |
| **Grafik** | Zweite Reihe: **Sprites**, **Zeichensatz**, **Kacheln**, **Bildschirm**, **Karten**, **Palette**. Hier arbeiten der Sprite-Editor, der Zeichensatz-Editor, der Kachel-Editor, der Bildschirm-Editor, der Karten-Editor und die Palette. Mehr dazu im Kapitel [Grafik und Klang im Überblick](kapitel:grafik-ueberblick) |
| **Musik** | der Tracker für Songs (Dateien `.vcsong` und `.vcplay`). Mehr dazu im Kapitel [Der Tracker](kapitel:tracker) |
| **Daten** | Zweite Reihe: **Disk**, **Hex**, **Speicher**. **Disk** ist der Disk-Editor (siehe [Disk-Editor, Segmente, ROMs](kapitel:disk-studio)), **Hex** zeigt Binärdateien des Projekts nur zum Ansehen, **Speicher** ist die grafische Speicherkarte des letzten Baus |
| **Debug** | die Debug-Sitzung mit Haltepunkten, Einzelschritt, Registern, Speicher und Bild des Emulators. Mehr dazu im Kapitel [Der Debugger](kapitel:debugger) |

Ein Bereich oder Unterbereich kann fehlen, wenn die Ausgabe von VintageCoder, die du benutzt, den zugehörigen Editor nicht enthält.

## Die Kopfzeile

Von links nach rechts: der Knopf für die **Seitenleiste** (Tipp zeigt das Kürzel), der **Projektname** mit dem gekürzten Pfad darunter, die **Hauptbereiche**, dann bei offenem Projekt die Knöpfe **Bauen** (F7) und **Starten** (F5). Läuft ein Programm im Emulator, kommt der Knopf **Beenden** dazu. Der **Ziel**-Knopf zeigt das Zielsystem und den Namen des Startziels; ein Klick öffnet eine Liste mit dem Emulator und den eingetragenen Maschinen, ein Haken steht beim aktuellen Startziel. Es folgen zwei Menüs mit Symbol, **Sprache** (Automatisch, Deutsch, English) und **Farbschema** (System, Hell, Dunkel), dann das Zahnrad ⚙ für die **Einstellungen** (**Cmd/Strg+Komma**) und zuletzt das Suchsymbol für die Befehlspalette (**Cmd/Strg+K**).

Einen eigenen Speichern-Knopf gibt es im Fenster nicht: **Cmd/Strg+S** speichert. Vor jedem Bauen wird ohnehin gespeichert.

## Die Seitenleiste

Die Seitenleiste zeigt im Hauptbereich **Code** den **Dateibaum** des Projekts (Überschrift „Projekt“). Oben in ihr stehen die Knöpfe **Dateien filtern**, **Neue Datei**, **Neuer Ordner** und **Projekt schließen**. Die Pfeiltasten wählen Einträge, **→** und **←** klappen Ordner auf und zu, **Enter** öffnet, **F2** benennt um, **Entf** löscht. Alles Weitere zu Dateien steht im Kapitel [Projekte und Dateien](kapitel:projekte).

**Cmd/Strg+B** klappt die Seitenleiste ein und aus. Ist sie eingeklappt, bleibt ein schmaler Streifen mit Symbolen (Dateien, Suche), über den du sie wieder öffnest.

## Der Editor

Im Editor öffnen Dateien als **Reiter**. **Cmd/Strg+W** schließt einen Reiter (bei Ungespeichertem fragt VintageCoder), **Cmd/Strg+N** öffnet einen **unbenannten Reiter**, in dem du sofort schreibst. **Cmd/Strg+Umschalt+T** öffnet eine **Notiz**; unbenannte Reiter und Notizen sichert VintageCoder alle paar Sekunden lokal. Mit **Cmd/Strg+Alt+T** teilst du den Editor in zwei Hälften und kannst Reiter in die andere Hälfte schicken. **Cmd/Strg+Alt+Rechts** und **Cmd/Strg+Alt+Links** wechseln zum nächsten und vorigen Reiter.

Die Editoren für Grafik, Musik und Disketten sind eigene Hauptbereiche (**Grafik**, **Musik**, **Daten**). Ihre Dateien öffnest du dort, nicht im Hauptbereich **Code**.

## Die Ablage unter dem Editor

Die **Ablage** liegt unter dem Editor und hat fünf Reiter:

- **Probleme:** Fehler und Warnungen mit Datei und Zeile (Klick springt zur Stelle).
- **Ausgabe:** was beim Bauen und Starten geschehen ist, mit den Knöpfen **Kopieren** und **Leeren**.
- **Tests:** die Testergebnisse (siehe [Tests](kapitel:tests)).
- **Suche:** sucht Text in allen Textdateien des Projekts (**Cmd/Strg+Umschalt+F** öffnet sie).
- **Gliederung:** die Marken, Konstanten und Makros der Datei im Reiter; ein Klick springt dorthin.

![Der Reiter Code mit der Datei jumprun.asm und dem geöffneten Fach unter dem Editor, Reiter „Probleme“ aktiv (daneben Ausgabe, Tests, Suche, Gliederung). Es steht dort „Projekt: 2 Meldungen in 2 Dateien, davon 0 Fehler“ und zwei Hinweise zu lib/core.asm und lib/music.inc über automatisch verlängerte Sprünge.](bild:probleme-panel "Das Fach unter dem Editor mit dem Reiter „Probleme“ (hier nur Hinweise, keine Fehler).")

Die Ablage öffnet sich von selbst, wenn es etwas zu zeigen gibt (Baufehler, Ausgabe). **Cmd/Strg+J** klappt sie ein und aus; mit **×** rechts oben schließt du sie. Zwischen den Reitern wechselst du mit den Pfeiltasten.

## Das Panel Maschine

Rechts steht auf Wunsch das Panel **Maschine**. Es zeigt das System und das Startziel, den Zustand (bereit, baut, läuft) und das Ergebnis des letzten Baus (Quelle, Programm, Größe, Startadresse und die Segmente), dazu die Knöpfe **Bauen**, **Starten** und **Beenden**. Während einer Debug-Sitzung zeigt es zusätzlich die Register, die Flags und, wo der Emulator es liefert, das Bild der Maschine. **Cmd/Strg+Alt+B** blendet das Panel ein und aus.

## Schritt für Schritt: Bereiche ein- und ausblenden

1. **Cmd/Strg+B**: Seitenleiste. *Erwartet:* Der Dateibaum verschwindet, der Editor wird breiter.
2. **Cmd/Strg+J**: Ablage. *Erwartet:* Sie klappt unter dem Editor aus oder ein.
3. **Cmd/Strg+Alt+B**: Panel Maschine. *Erwartet:* Es erscheint rechts oder verschwindet.
4. **Zen-Modus (Cmd/Strg+Alt+Z):** blendet Seitenleiste, Ablage, Panel und Statusleiste aus. *Erwartet:* Es bleiben Kopfzeile und Editor. Derselbe Befehl bringt alles zurück.
5. Die Größen änderst du, indem du den Rand zwischen zwei Bereichen mit der Maus ziehst. Mit den Pfeiltasten geht es auch (Umschalt für große Schritte); ein Doppelklick auf den Rand stellt die Vorgabe her.

## Einen Bereich groß machen

Seitenleiste, Ablage und Panel haben oben rechts einen Knopf **Bereich maximieren**. Maximieren klappt die anderen Bereiche ein; **Bereich wiederherstellen** (oder ein Doppelklick auf die Kopfzeile der Ablage) bringt die vorige Aufteilung zurück. Mit der Tastatur: **Cmd/Strg+Umschalt+Enter** schaltet den Bereich mit dem Fokus um. Der Editor wird nur maximiert, nie eingeklappt.

> **Tipp:** Wenn du dich verklickt hast und nichts mehr stimmt: **Ansicht ▸ Bereiche ▸ Layout zurücksetzen** stellt die Standardgrößen wieder her.

Alle diese Befehle stehen im Menü **Ansicht** und in der Befehlspalette.

## Schmale Fenster

Das Fenster darf schmal werden. Ist der Editor schmaler als etwa 640 Punkte, schweben Seitenleiste und Panel über dem Editor; ein Klick in den Editor klappt sie wieder zu. Zwei Hälften des geteilten Editors stehen bei schmalem Platz untereinander statt nebeneinander.

## Die Befehlspalette

Die **Befehlspalette** ist die schnellste Art, etwas zu finden: ein Befehl, eine Datei, ein Ort in der Navigation oder ein zuletzt geöffnetes Projekt.

1. Drücke **Cmd/Strg+K** oder klicke auf das Suchsymbol in der Kopfzeile. Es öffnet sich ein Fenster mit einem Eingabefeld.
2. Tippe ein paar Buchstaben, zum Beispiel `bauen`. Alle Wörter müssen im Namen vorkommen. Die Liste ist in Gruppen geordnet: **Befehle**, **Dateien**, **Projekte** und **Gehe zu** (die Hauptbereiche und Unterbereiche).
3. Wähle mit den Pfeiltasten und drücke Enter, oder klicke auf einen Eintrag. Esc schließt das Fenster.

![Die Befehlspalette über dem Code-Editor: oben das Suchfeld „Befehl oder Datei suchen ...“, darunter unter BEFEHLE eine Liste mit Einträgen wie „Über VintageCoder“, „Konfigurationsassistent ...“ und mehreren Markdown-Befehlen, teils mit Tastenkürzeln. Unten steht der Hinweis zur Bedienung mit Pfeiltasten und Esc.](bild:befehlspalette "Die Befehlspalette findet jeden Befehl und jede Datei.")

Befehle, die jetzt nicht möglich sind, sind ausgegraut (zum Beispiel ohne offenes Projekt). Hinter den Befehlen steht ihr Tastenkürzel, so lernst du die Kürzel nebenbei.

## Die Menüs

Die Menüs bieten dieselben Befehle wie die Befehlspalette, geordnet nach Aufgabe:

- **Datei:** Neues Projekt, Projekt öffnen, Startseite, neue Datei, Speichern, Projekt kopieren, Exportieren und mehr.
- **Bearbeiten:** Rückgängig, Suchen und Ersetzen, Falten, Formatieren, Markdown und weitere Hilfen für den Quelltext.
- **Ansicht:** Befehlspalette, Bereiche (Seitenleiste, Ablage, Panel, Maximieren, Layout), Editor-Schrift, Farbschema, Reiter und Teilen, Zen-Modus, Bereich Projekt.
- **Projekt:** Bauen, Starten, Probleme und Protokoll, Tests, Projekt schließen.
- **Hilfe:** Hilfe und weitere Einträge.

Die Kürzel stehen hinter den Einträgen. Eine Liste der wichtigsten findest du im Kapitel [Kürzel und häufige Fragen](kapitel:kuerzel-faq).

## Farbschema, Sprache und Schrift

Das Farbschema wählst du im Menü **Farbschema** der Kopfzeile: **System** (folgt dem Betriebssystem), **Hell** oder **Dunkel**. Im Menü **Ansicht ▸ Farbschema** und in der Befehlspalette gibt es **Hell/Dunkel umschalten**. Die Sprache der Oberfläche wählst du im Menü **Sprache** der Kopfzeile.

Die Schrift im Quelltext änderst du mit **Editor-Schrift größer** und **Editor-Schrift kleiner** (Menü **Ansicht ▸ Zoom und Schrift** oder Befehlspalette).

## Einstellungen

Die **Einstellungen** gelten für alle Projekte. Du öffnest sie mit dem Zahnrad ⚙ in der Kopfzeile oder mit **Cmd/Strg+Komma**. Links stehen die Seiten **Allgemein**, **Ziele und Hardware**, **ROMs**, **Aussehen**, **Editor**, **Toolchain**, **Tastenkürzel**, **Sicherung** und **Info und Update**, rechts der Inhalt. Dort stellst du zum Beispiel Sprache, Farbschema, Startbild, Emulatoren, ROM-Dateien und die Update-Suche ein (Hilfeseite [Einstellungen](hilfe:settings)). Alles, was zu einem einzelnen Projekt gehört, steht im Hauptbereich **Projekt** (siehe [Projekte und Dateien](kapitel:projekte)). Emulatoren, ROMs und Hardware richtest du auch mit dem Konfigurationsassistenten ein (Kapitel [Installation und Ersteinrichtung](kapitel:installation)).

## Dein Layout wird gemerkt

Du musst Größen und Sichtbarkeit der Bereiche nicht speichern: VintageCoder merkt sie sich selbst. Gespeichert werden nur Zahlen und Schalter, nie Inhalte. Ein maximierter Bereich und der Zen-Modus werden nicht gemerkt: Beim nächsten Start ist das gemerkte Normal-Layout da.

## Häufige Stolpersteine

- **Seitenleiste, Ablage und Panel fehlen.** Du hast vielleicht den **Zen-Modus** (Cmd/Strg+Alt+Z) oder einen maximierten Bereich eingeschaltet; derselbe Befehl bringt sie zurück. Außerdem zeigen nur der Hauptbereich **Code** (mit offenem Projekt) Seitenleiste und Panel.
- **Das Panel ist nicht da.** Es ist nur auf Wunsch sichtbar: **Cmd/Strg+Alt+B**.
- **Das Layout ist verkorkst.** **Ansicht ▸ Bereiche ▸ Layout zurücksetzen**.
- **Cmd/Strg+B baut nicht.** Das Kürzel blendet die Seitenleiste ein und aus. Gebaut wird mit F7.
- **Ein Unterbereich fehlt in der zweiten Reihe.** Ein Bereich fehlt, wenn die Ausgabe von VintageCoder, die du benutzt, den zugehörigen Editor nicht enthält.

## Wie geht es weiter?

Im nächsten Kapitel geht es um das, was in der Seitenleiste steht und was der Hauptbereich **Projekt** zeigt: [Projekte und Dateien](kapitel:projekte). Danach lernst du den [Code-Editor](kapitel:editor) kennen.
