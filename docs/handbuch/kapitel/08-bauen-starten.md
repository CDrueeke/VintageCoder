---
id: bauen-starten
titel: Bauen und Starten
reihenfolge: 8
stufe: Einsteiger
zeit: 25 Minuten
siehe_auch: [erstes-projekt, assembler, basic, debugger]
referenz: [build, settings, mega65-hardware, memmap]
---

# Bauen und Starten

**Ziel:** Du weißt genau, was beim Bauen passiert, wo du das Ergebnis und die Meldungen findest und wie du ein Programm im Emulator oder auf echter Hardware startest. Du liest das Bauprotokoll, behebst Fehler mit dem Reiter „Probleme“ und gibst dem Emulator eigene Startparameter mit.

## Das brauchst du

- Ein Projekt mit einem Quelltext, zum Beispiel das aus [Dein erstes Projekt](kapitel:erstes-projekt).
- Einen Emulator, den VintageCoder gefunden hat (siehe [Installation und Ersteinrichtung](kapitel:installation)).

## Bauen, Starten, Debuggen

Es gibt drei Wege vom Quelltext zum laufenden Programm. Du findest sie in der Befehlsleiste und im Menü **Projekt**:

| Aktion | Taste | Was passiert |
|---|---|---|
| **Bauen** | **F7** | baut alle Programme der Projektdatei in Baureihenfolge. Hat das Projekt keine Quellen in der Projektdatei, wird die aktive Datei gebaut. |
| **Starten** | **F5** | baut das **Startprogramm** (★ im Projektbaum) und startet es auf dem Startziel: im Emulator oder auf einer eingetragenen Maschine. |
| **Debuggen** | **F6** | baut, startet im Debugger und schaltet auf den Arbeitsplatz Debug (siehe [Der Debugger](kapitel:debugger)). |

Mit der **Umschalt**-Taste (**Umschalt+F7**, **Umschalt+F5**, **Umschalt+F6**) gilt die **aktive Datei** statt des Startprogramms; im Menü **Projekt** heißen diese Einträge „Aktive Datei bauen“, „Aktive Datei starten“ und „Aktive Datei debuggen“. Bevor gebaut wird, speichert VintageCoder alle offenen Dateien.

![Die Befehlsleiste mit dem Feld „Auf: C64 PAL → VICE x64sc“ und den Knöpfen Starten (F5), Debuggen (F6) und Bauen (F7)](bild:bauen-knopf "Bauen, Starten und Debuggen in der Befehlsleiste")

Während ein Bau läuft, zeigt der Knopf **Baue …**, und ein Kreuz daneben bricht ab. Danach wird dein Programm gebaut und die Ergebnisse stehen im Ordner `build/` deines Projekts:

| Datei | Inhalt |
|---|---|
| `….prg` (Atari: `.xex`, Module: `.crt`, `.car`) | das fertige Programm |
| `.lst` | das **Listing**: Adresse, Bytes und Quelltext nebeneinander |
| `.vs` | die Labels für den Emulator (VICE) |
| `.vcdbg` | Debug-Information für den Debugger |

Für Assembler-Quellen kommen weitere Dateien hinzu (zum Beispiel `.sym.txt` mit der Symboltabelle und `.map.txt` mit der Speicherbelegung), siehe unten im Bauprotokoll.

## Schritt für Schritt: Das Bauprotokoll lesen

Nach jedem Bau öffnet sich im Fußpanel (sofern es offen ist) der Reiter **Bauprotokoll**. Es zeigt je Programm die **Stufen** des Baus mit Dauer und Meldungen:

1. Baue dein Projekt mit **F7**.
2. Klicke im Fußpanel auf den Reiter **Bauprotokoll**. Fehlt das Fußpanel, blende es mit **Cmd/Strg+J** ein.
3. Du siehst die Stufen von oben nach unten:

| Stufe | Assembler | BASIC |
|---|---|---|
| Vorbereitung | Quelle, Zielsystem, CPU, Include-Suche | Quelle, Zielsystem, Dialekt |
| Medien exportieren | veraltete Sprites, Zeichensätze, Songs nach `build/export/` | – |
| Quelltext und Includes | – | Includes eingesetzt, Zeilen je Datei |
| Assemblieren / Übersetzen | Dateien, Makros, Durchläufe, Symbole, Segmente, Startadresse, Warnungen | Strukturen auflösen, Zeilennummern, Variablen, Marken |
| Ausgabe schreiben | Format und Größe des Programms | Größe, Ladeadresse |
| Zwischendateien | Listing (`.lst`), Symboltabelle, Speicherbelegung, VICE-Labels, Debuginformation | BASIC mit Zeilennummern, Variablen, Marken |

4. Jede Datei am Ende lässt sich öffnen: Textdateien mit einem Klick auf den Namen im Editor, jede Datei mit **Hex** als Hexdump und mit **Zeigen** im Dateimanager. So öffnest du zum Beispiel das **Listing** und siehst, was aus deinen Zeilen geworden ist.

![Das Bauprotokoll für ein Assembler-Programm: die Stufen Vorbereitung, Assemblieren und Ausgabe schreiben mit Dauer und Meldungen, darunter die Liste der erzeugten Dateien mit den Knöpfen Hex und Zeigen](bild:bauprotokoll "Das Bauprotokoll zeigt jede Stufe des Baus und die erzeugten Dateien")

Bei Fehlern steht die fehlgeschlagene Stufe mit den Meldungen im Protokoll.

## Schritt für Schritt: Fehler beheben mit dem Reiter „Probleme“

1. Baue dein Programm. Treten Fehler auf, öffnet sich das Fußpanel von selbst mit dem Reiter **Probleme**, auch wenn du es eingeklappt hattest.
2. Jede Zeile nennt die Meldung, die Datei und die Zeilennummer. Ein Fehler hat einen Code wie `E202`.
3. Ein **Klick** auf die Meldung springt im Editor zur Zeile. Die Zeile ist rot unterstrichen.
4. Behebe den Fehler und baue erneut. Die Zahlenmarke am Reiter **Probleme** zeigt rot die Zahl der Fehler, sonst gelb die der Warnungen.

![Der Reiter „Probleme“ mit einem Fehler „E202: unbekanntes Symbol ‚ende‘“ und der Datei- und Zeilenangabe; die Zeile im Editor darüber ist markiert](bild:probleme-panel "Der Reiter „Probleme“ listet Fehler und Warnungen, ein Klick springt zur Zeile")

Ein Rechtsklick auf eine Meldung bietet **Zur Zeile springen**, **Meldung kopieren** und **Alle Probleme kopieren**.

Schlägt der Bau fehl, ohne dass eine Meldung zu einer Zeile gehört (zum Beispiel weil die Projektdatei fehlerhaft ist), zeigt der Reiter **Ausgabe** den Grund.

### Der Reiter „Ausgabe“

Der Reiter **Ausgabe** ist das Protokoll aller Vorgänge: Meldungen vom Bauen, Starten und Debuggen. Hier findest du auch den Grund, wenn ein Start nicht klappt. Ein Rechtsklick bietet **Zeile kopieren**, **Alles kopieren** und **Ausgabe leeren**.

![Der Reiter „Ausgabe“ mit den Zeilen eines Baus und Starts: Projekt gebaut, Emulator gestartet](bild:ausgabe-panel "Der Reiter „Ausgabe“ protokolliert Bauen, Starten und Debuggen")

## Starten im Emulator

Mit **F5** baust du und startest das Startprogramm. Die Befehlsleiste zeigt, wohin:

1. Schau auf das Feld **Auf:** in der Befehlsleiste, zum Beispiel „C64 PAL → VICE x64sc“. Links steht das Zielsystem, rechts das **Startziel**.
2. Ein Klick auf das Feld öffnet eine Auswahl. Im Abschnitt **Läuft auf** wählst du:
   - **Standard des Systems:** das Startziel, das du in den Einstellungen für dieses System eingestellt hast,
   - **Emulator:** der Emulator des Systems,
   - eine **Maschine**, die du in den Einstellungen als echte Hardware eingetragen hast (siehe unten).
3. Drücke **F5**. Das Fenster des Emulators öffnet sich. Dein Programm wird automatisch geladen und gestartet.

Hat dein Projekt eine **Diskette** (siehe [Projekte und Dateien](kapitel:projekte)) und steht das Startprogramm vorn, startet VICE die Diskette. So kann dein Programm weitere Dateien nachladen.

> **Tipp:** Das **Target** im Feld „Auf:“ steht in der Regel auf „automatisch“. Dann gilt das `.target` aus dem Quelltext bzw. das erste Zielsystem der Projektdatei. Hat dein Projekt mehrere Zielsysteme, wählst du im Feld „Auf:“ das richtige.

### Der Reiter „Emulator“ (Debug-Arbeitsplatz)

Im Arbeitsplatz **Debug** zeigt der Reiter **Emulator** im Fußpanel das **Bild** des laufenden VICE-Emulators (C64, C128, VC-20, Plus/4) direkt in VintageCoder. Das Bild kommt über den Monitor des Emulators: Beim Anhalten des Programms (Haltepunkt, Einzelschritt, Pause) wird es aktualisiert. Mit dem Knopf **Bild holen** holst du es sofort. Der Schalter **Zeitlupen-Live** aktualisiert das Bild laufend; die Maschine läuft dann nur mit etwa 40 bis 80 Prozent ihres Tempos und ohne Ton.

Der Reiter bietet außerdem **Als PNG speichern** (Ordner `aufnahmen/` im Projekt), **In Zwischenablage** und **Als Projektbild setzen**. Mehr zum Debuggen steht im Kapitel [Der Debugger](kapitel:debugger).

![Der Reiter „Emulator“ im Debug-Arbeitsplatz: das Bild des C64 mit rotem Rahmen, darüber die Knöpfe Bild holen, Zeitlupen-Live und Als PNG speichern](bild:emulator-reiter "Der Reiter „Emulator“ zeigt das Bild des laufenden C64 in VintageCoder")

![Der Knopf „Als Projektbild setzen“ im Reiter „Emulator“ und rechts die Willkommensseite, auf der das Bild als Vorschau des Projekts erscheint](bild:emulator-bild-projektbild "Das Emulatorbild als Projektbild speichern")

## Startparameter für den Emulator

Jeder Emulator hat Startparameter: ein Schalter für das Tempo, die Bildschirmdarstellung, den Ton und vieles mehr. VintageCoder kennt die wichtigsten und bietet sie als Kästchen und Auswahlen an, ohne dass du die Kommandozeile kennen musst.

1. Öffne die **Einstellungen** (Cmd/Strg+Komma) und wechsle zur Seite **Zielsysteme und Hardware**. Wähle links dein System.
2. Unter **Emulatoren** steht beim gewählten Emulator der Parametereditor. Die Parameter sind in Gruppen geordnet, zum Beispiel für Bild oder System. Fährst du mit der Maus über einen Eintrag, erscheint ein Hinweis mit dem Schalter, dem Standardwert und einem Beispiel.
3. Setze die gewünschten Haken, zum Beispiel **Warp-Modus beim Start** für VICE.
4. Brauchst du etwas, das der Katalog nicht kennt, trägst du es im Feld **Weitere Parameter** ein. Leerzeichen trennen; Pfade mit Leerzeichen setzt du in Anführungszeichen. Platzhalter wie `{project_dir}` (Projektordner), `{output}` (das gebaute Programm) oder `{disk}` (die Diskette des Projekts) kannst du anklicken, um sie einzufügen.
5. Die **Vorschau** zeigt die fertige Kommandozeile mit einem Beispielprojekt („demo“). Mit **Kopieren** legst du sie in die Zwischenablage. **Auf Standard zurücksetzen** entfernt alle Parameter wieder.
6. Mit **▶ Test-Start** startest du den Emulator mit diesen Parametern zur Probe; er wird nach kurzer Zeit wieder beendet. Klicke danach unten auf **Änderungen anwenden**.

![Der Parametereditor eines Emulators: gruppierte Kästchen und Auswahlen, das Feld „Weitere Parameter“ und darunter die Vorschau der Kommandozeile](bild:argumente "Startparameter für den Emulator mit Vorschau der Kommandozeile")

Die Parameter gelten für **F5 und F6**. Einige Schalter setzt VintageCoder selbst (etwa den Autostart des Programms); sie dürfen im freien Feld nicht stehen, und der Editor warnt dich dann.

## Starten auf echter Hardware

Dein Programm läuft auf echter Hardware, wenn du die Maschine eintragen kannst:

- **Ultimate 64 / II+** (für den C64)
- **MEGA65** (am Gerät)

Für die anderen Systeme gibt es keine anbindbare Hardware.

### Schritt für Schritt: Eine Ultimate 64 eintragen

1. Öffne die **Einstellungen** und wechsle zur Seite **Zielsysteme und Hardware**. Wähle das System **Commodore 64**.
2. Rechts steht der Abschnitt **Physische Zielhardware & Netzwerkgeräte**. Klicke auf **+ Hardware hinzufügen** und wähle **Ultimate 64 / II+ (Netzwerk)**.
3. Trage die **Adresse** deiner Ultimate ein, zum Beispiel `192.168.1.50` (oder `host:port`). Hat die Ultimate ein **Netzwerkpasswort** (in ihrer Firmware eingestellt), gib es im Feld **Passwort** ein; „Zeigen“ blendet es ein. Das Passwort steht in der Einstellungsdatei, die nur für dich lesbar ist.
4. Klicke auf **Verbindung testen**. Der Status wechselt von „ungeprüft“ zu „Online“ oder „Nicht erreichbar“. Beim Start von VintageCoder findet kein Netzwerkzugriff statt, deshalb bleibt der Status bis zum Test „ungeprüft“.
5. Wähle bei **Startziel (F5)** deiner Ultimate als Ziel, oder wähle sie im Feld **Auf:** der Befehlsleiste im Abschnitt **Läuft auf**.
6. Drücke **F5**. VintageCoder baut die `.prg`-Datei und schickt sie über die Netzwerkschnittstelle der Ultimate: Reset, Laden, RUN.

> **Info:** Auf der Ultimate lassen sich beim Debuggen Speicher lesen und schreiben, pausieren und fortsetzen. Register, Einzelschritt und Haltepunkte gibt es dort nicht. Eine nicht passende Kombination, zum Beispiel ein Atari-Programm auf der Ultimate, meldet die Ausgabe.

### MEGA65 am Gerät

Die Hardware **MEGA65** verbindest du per **USB** (serielle Schnittstelle; das Programm wird über den Monitor geladen und gestartet) oder per **Ethernet** (`etherload`, `mega65_ftp`). Für Starten, Kopieren und Debuggen kann jede Aktion eine eigene Verbindung bekommen. Die Einrichtung am Gerät (Core, Netzwerk, Fernzugriff) beschreibt die Hilfeseite [MEGA65 am Gerät](hilfe:mega65-hardware).

> **Achtung:** Der Ethernet-Weg des MEGA65 ist **noch nicht an einem echten Gerät getestet**. Verlasse dich erst darauf, wenn du es selbst ausprobiert hast.

## Häufige Stolpersteine

- **F5 startet nichts.** Prüfe das Feld „Auf:“ in der Befehlsleiste und den Emulator in den Einstellungen (rot = nicht gefunden). Die Meldung steht im Reiter **Ausgabe**.
- **Der Reiter „Bauprotokoll“ fehlt nach dem Bau.** Das Fußpanel ist eingeklappt; ein fehlerfreier Bau öffnet es nicht von allein. Blende es mit **Cmd/Strg+J** ein. Bei Fehlern öffnet es sich von selbst mit „Probleme“.
- **Fehler, aber keine Zeile im Editor markiert.** Schau in den Reiter **Ausgabe**, dort steht der Grund.
- **Es startet das falsche Programm.** F5 startet das **Startprogramm** (★). Mit **Umschalt+F5** startest du die aktive Datei, mit einem Rechtsklick im Projektbaum ▸ **★ Als Startprogramm festlegen** änderst du das Startprogramm.
- **Das Programm läuft, aber der Emulator zeigt „READY.“.** Bei Assembler-Programmen mit `.basicstub` tippst du im Emulator `RUN`, falls der Autostart das Programm nicht gestartet hat.
- **Zwei Emulator-Fenster.** Schließe das alte Fenster, bevor du erneut startest.
- **Die Ultimate bleibt „ungeprüft“ oder „Nicht erreichbar“.** Prüfe Adresse und Passwort und ob Rechner und Ultimate im selben Netz sind.

## Wie geht es weiter?

Läuft dein Programm nicht wie erwartet, lernst du im Kapitel [Der Debugger](kapitel:debugger), wie du es anhältst und Schritt für Schritt verfolgst. Das Kapitel [Automatisch testen](kapitel:tests) zeigt, wie VintageCoder dein Programm prüft. Zum Nachschlagen: die Hilfeseiten [Bauen und Starten](hilfe:build) und [Einstellungen](hilfe:settings).
