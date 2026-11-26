---
id: bauen-starten
titel: Bauen und Starten
reihenfolge: 8
stufe: Einsteiger
zeit: 25 Minuten
siehe_auch: [erstes-projekt, assembler, basic, debugger]
referenz: [build, settings, mega65-hardware, memmap, turbo-disk]
---

# Bauen und Starten

**Ziel:** Du weißt genau, was beim Bauen passiert, wo du das Ergebnis und die Meldungen findest und wie du ein Programm im Emulator startest. Du liest die Ausgabe und behebst Fehler mit dem Reiter „Probleme“.

## Das brauchst du

- Ein Projekt mit einem Quelltext, zum Beispiel das aus [Dein erstes Projekt](kapitel:erstes-projekt).
- Einen Emulator, den VintageCoder gefunden hat. Die Startseite zeigt ihn im Kasten **Toolchain und Hardware** als „bereit“ (siehe [Installation und Ersteinrichtung](kapitel:installation)).

## Bauen und Starten

Es gibt drei Wege vom Quelltext zum laufenden Programm. **Bauen** und **Starten** findest du in der Kopfzeile (Symbole **Bauen** und **Starten**) und im Menü **Projekt**, **Debuggen** im Menü **Projekt** und im Bereich **Debug**:

| Aktion | Taste | Was passiert |
|---|---|---|
| **Bauen** | **F7** | baut das Projekt: alle Programme der Projektdatei in Baureihenfolge. Hat das Projekt keine Quellen in der Projektdatei, wird die aktive Datei gebaut. |
| **Starten** | **F5** | baut das **Start-Programm** und startet es auf dem Startziel im Emulator. Es ist egal, welche Datei gerade aktiv ist. Das Symbol **Starten** in der Kopfzeile tut dasselbe. |
| **Debuggen** | **F6** | baut das Start-Programm, startet es im Emulator mit einer **Debug-Sitzung** und öffnet den Bereich **Debug**. Dort kannst du anhalten, Schritt für Schritt gehen und Register und Speicher ansehen (siehe [Der Debugger](kapitel:debugger)). |
| **Beenden** | **Umschalt+F5** | beendet den laufenden Emulator oder die Debug-Sitzung (im Menü **Projekt** ▸ **Debuggen**: „Debuggen beenden“). In der Kopfzeile erscheint das Symbol **Beenden**, solange der Emulator läuft. |

Mit **Alt+Umschalt** (**Alt+Umschalt+F7**, **Alt+Umschalt+F5**, **Alt+Umschalt+F6**) gilt die **aktive Datei** (die im Editor geöffnete) statt des Start-Programms. Im Menü **Projekt** heißen die Einträge „Projekt bauen“, „Startprogramm starten“, „Startprogramm debuggen“ sowie im Untermenü **Aktive Datei** „Aktive Datei bauen“, „Aktive Datei starten“ und „Aktive Datei debuggen“. Auf dem Mac steht Alt für die Wahltaste. Bevor gebaut wird, speichert VintageCoder alle offenen Dateien.

> **Info:** Für das Debuggen gibt es kein Symbol in der Kopfzeile. Du startest es mit **F6**, über das Menü **Projekt** oder im Bereich **Debug** mit dem Knopf **Debuggen starten**. Schlägt der letzte Bau fehl, startet es nicht: Behebe zuerst die Fehler.

*[Bild folgt in E13: bauen-knopf]*

Während ein Bau läuft, ist das Symbol **Bauen** gesperrt, und die Statusleiste zeigt „baut …“. Danach steht dort „gebaut: N Bytes“ (bei Fehlern „Bau: N Fehler“). Die Ergebnisse liegen im Ordner `build/` deines Projekts:

| Datei | Inhalt |
|---|---|
| `….prg` (Atari: `.xex`, Module: `.crt`, `.car`) | das fertige Programm |
| `.lst` | das **Listing**: Adresse, Bytes und Quelltext nebeneinander |
| `.vs` | die Labels für den Emulator (VICE) |
| `.vcdbg` | Debug-Information |

Für Assembler-Quellen kommen weitere Dateien hinzu (zum Beispiel `.sym.txt` mit der Symboltabelle und `.map.txt` mit der Speicherbelegung), siehe die Hinweise zu den Zwischendateien in der Ausgabe.

### Der Bereich „Projekt“, Seite „Bauen und Starten“

Der Bereich **Projekt** (**Cmd/Strg+Alt+0**) hat eine Seite **Bauen und Starten**. Sie zeigt den **letzten Bau** dieser Sitzung („Gebaut um …, ohne Fehler“ oder die Zahl der Fehler) und hat die Knöpfe **Bauen** und **Starten**. Darunter steht **Emulator je Ziel**: Hier stehen nur Überschreibungen, die dieses Projekt festlegt. Die Karte **Pakete (Registry)** nennt die geladenen Pakete mit Herkunft.

Das **Start-Programm** legst du auf der Seite **Quellen** fest. Dort steht es mit „★ Start-Programm“. Bei den anderen Programmen wählst du **Als Start-Programm**. Ohne Angabe gilt das erste Programm der Quellen.

### Das Panel „Maschine“

Rechts im Bereich **Code** zeigt das Panel **Maschine** (ein- und ausblenden mit **Cmd/Strg+Alt+B**) das **System**, das **Startziel** und den **Zustand** („bereit“, „baut …“, „läuft“). Darunter steht der **letzte Bau**: Quelle, Programm, Größe, Start (SYS) und eine Tabelle der Segmente mit Adresse von, bis und Länge. Die Knöpfe **Bauen**, **Starten** und **Beenden** im Panel tun dasselbe wie die Kopfzeile. Läuft eine Debug-Sitzung, zeigt das Panel zusätzlich den Abschnitt **Debug-Sitzung** mit PC, A X Y, SP und den Flags. Liefert der Emulator ein Bild, steht es dort mit dem Knopf **Bild aktualisieren**. Alle Einzelheiten stehen im Bereich **Debug**.

![Das Panel „Maschine“ mit System Commodore 64, Startziel VICE x64sc (taktgenau) und Zustand „bereit“, den Knöpfen „Bauen“, „Starten“ und „Beenden“. Unter „Letzter Bau“ stehen Quelle jumprun.asm, Programm build/jumprun.prg, Größe 16.605 Bytes und eine Tabelle mit vier Segmenten.](bild:panel-maschine "Das Panel „Maschine“ mit dem letzten Bau und seinen Segmenten.")

## Die Ausgabe lesen

Die **Ablage** unter dem Editor hat die Reiter **Probleme**, **Ausgabe**, **Tests**, **Suche** und **Gliederung**. Im Reiter **Ausgabe** steht Zeile für Zeile, was beim Bauen und Starten passiert ist.

1. Baue dein Projekt mit **F7**.
2. Öffne die Ablage mit **Cmd/Strg+J** und wähle den Reiter **Ausgabe**. Oder klicke in der Statusleiste auf „gebaut: N Bytes“: Das öffnet die Ablage direkt auf der Ausgabe.
3. Bei Erfolg steht dort zum Beispiel „Fertig: 34 Bytes in …“ mit dem Pfad des Programms. Hat dein Projekt eine Diskette, kommt eine Zeile mit deren Namen und Größe dazu.
4. Die erzeugten Dateien öffnest du über die Seitenleiste. Ordner wie `build/` sind dort sichtbar. Zum Beispiel das **Listing** (`.lst`) zeigt dir, was aus deinen Zeilen geworden ist.

Die Knöpfe **Kopieren** und **Leeren** im Reiter und das Kontextmenü (Rechtsklick) kopieren die Ausgabe oder leeren sie. Dieselben Befehle stehen im Menü **Projekt** als „Protokoll kopieren“ und „Protokoll leeren (Ausgabe leeren)“. Zeilen mit einer Dateiangabe springen bei einem Klick zur Stelle im Editor.

> **Info:** Die Ausgabe gliedert den Bau in Stufen (Zeilen mit „▸“). Eine Tabelle mit Dauer und Dateiliste gibt es in der Oberfläche nicht. Auf der Kommandozeile zeigt `vcc build` den Ablauf.

Bei Fehlern steht in der Ausgabe eine Zeile je Meldung, und ein Hinweis nennt „Bauen fehlgeschlagen“ mit der Zahl der Fehler.

*[Bild folgt in E13: ausgabe-panel]*

## Schritt für Schritt: Fehler beheben mit dem Reiter „Probleme“

1. Baue dein Programm. Treten Fehler auf, wechselt die Ablage von selbst zum Reiter **Probleme**.
2. Jede Zeile nennt den Schweregrad (**Fehler**, **Warnung**, **Hinweis**), einen Code wie `E202`, die Meldung und die Stelle. Die Meldungen des Baus stehen unter „Bauergebnis“, die Meldungen der Sprachdienste nach Dateien geordnet darunter.
3. Ein **Klick** auf die Meldung springt im Editor zur Zeile. Ein **Rechtsklick** auf eine Meldung bietet „Problem: Zur Zeile springen“, „Problem: Meldung kopieren“ und „Alle Probleme kopieren“.
4. Behebe den Fehler und baue erneut. Die Statusleiste zählt die Meldungen der Datei im Reiter („Keine Probleme“ oder die Zahl der Fehler und Warnungen). Ein Klick darauf öffnet den Reiter **Probleme**. Im Editor springst du mit **F8** und **Umschalt+F8** zum nächsten und zum vorigen Problem.

![Der Reiter Code mit der Datei jumprun.asm und dem geöffneten Fach unter dem Editor, Reiter „Probleme“ aktiv (daneben Ausgabe, Tests, Suche, Gliederung). Es steht dort „Projekt: 2 Meldungen in 2 Dateien, davon 0 Fehler“ und zwei Hinweise zu lib/core.asm und lib/music.inc über automatisch verlängerte Sprünge.](bild:probleme-panel "Das Fach unter dem Editor mit dem Reiter „Probleme“ (hier nur Hinweise, keine Fehler).")

Schlägt der Bau fehl, ohne dass eine Meldung zu einer Zeile gehört (zum Beispiel weil die Projektdatei fehlerhaft ist), zeigt der Reiter **Ausgabe** den Grund.

## Starten im Emulator

Mit **F5** baust du und startest das Start-Programm. Die Kopfzeile zeigt, wohin:

1. Schau auf das Feld **Startziel** in der Kopfzeile, zum Beispiel „C64 · VICE x64sc“. Links steht das Zielsystem, rechts das Startziel.
2. Ein Klick auf das Feld öffnet eine Auswahl der möglichen Startziele. Das Häkchen steht vor der aktuellen Wahl.
3. Drücke **F5**. Das Fenster des Emulators öffnet sich. Dein Programm wird automatisch geladen und gestartet.

Hat dein Projekt eine **Diskette** (siehe [Projekte und Dateien](kapitel:projekte)) und steht das Start-Programm vorn, startet VICE die Diskette. So kann dein Programm weitere Dateien nachladen.

**Turbo BASIC XL (Atari):** F5 baut hier eine Diskette (`build/<name>.atr`) aus deiner bereitgestellten Turbo-Diskette und bootet sie im Emulator. Fehlt die Diskette, erscheint über dem Editor ein Banner mit dem Knopf **Bereitstellen …**. Siehe [Turbo BASIC XL starten](kapitel:basic) und die Hilfeseite [Turbo BASIC XL: Diskette](hilfe:turbo-disk). Turbo-BASIC-Programme lassen sich noch nicht schrittweise debuggen; F6 meldet das.

> **Tipp:** Das Startziel wählt nur, *wo* das Programm läuft. Welches Zielsystem gebaut wird, steht im Quelltext (`.target`) bzw. in den Projekt-Einstellungen (Bereich **Projekt**, Seite **Ziele**).

> **Info:** Emulator-Pfade, Startparameter für den Emulator und echte Hardware (Ultimate 64 / II+, MEGA65) trägst du unter **Einstellungen ▸ Ziele und Hardware** ein (Karten **Emulatoren** und **Echte Hardware**; Menü **Ansicht ▸ Einstellungen** oder **Cmd/Strg+Komma**). Zur Hardware siehe die Hilfeseiten [MEGA65 am Gerät](hilfe:mega65-hardware) und [Einstellungen](hilfe:settings).

## Häufige Stolpersteine

- **F5 startet nichts.** Prüfe das Feld **Startziel** in der Kopfzeile und den Emulator auf der Startseite (Kasten **Toolchain und Hardware**: „nicht gefunden“ heißt, er fehlt). Die Meldung steht im Reiter **Ausgabe**.
- **Nach dem Bau sehe ich nichts.** Die Ablage ist eingeklappt. Blende sie mit **Cmd/Strg+J** ein. Bei Fehlern wechselt sie von selbst zum Reiter **Probleme**.
- **Fehler, aber keine Zeile im Editor markiert.** Schau in den Reiter **Ausgabe**, dort steht der Grund.
- **Es startet das falsche Programm.** **F5** und das Symbol **Starten** starten das **Start-Programm** (★ auf der Seite **Quellen** im Bereich **Projekt**). **Alt+Umschalt+F5** startet die aktive Datei. Das Start-Programm änderst du auf der Seite **Quellen** mit **Als Start-Programm**.
- **Das Programm läuft, aber der Emulator zeigt „READY.“.** Bei Assembler-Programmen mit `.basicstub` tippst du im Emulator `RUN`, falls der Autostart das Programm nicht gestartet hat.
- **F6 startet keine Debug-Sitzung.** Der letzte Bau ist fehlgeschlagen, oder es läuft schon eine Sitzung (erst mit **Umschalt+F5** beenden). Bei Turbo BASIC XL gibt es das Debuggen noch nicht.
- **Zwei Emulator-Fenster.** Schließe das alte Fenster oder klicke auf **Beenden**, bevor du erneut startest.

## Wie geht es weiter?

Das Kapitel [Der Debugger](kapitel:debugger) zeigt, wie du ein laufendes Programm anhältst und untersuchst. Das Kapitel [Automatisch testen](kapitel:tests) zeigt, wie VintageCoder dein Programm prüft. Zum Nachschlagen: die Hilfeseiten [Bauen und Starten](hilfe:build) und [Einstellungen](hilfe:settings).
