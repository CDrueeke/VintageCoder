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

**Ziel:** Du weißt genau, was beim Bauen passiert, wo du das Ergebnis und die Meldungen findest und wie du ein Programm im Emulator oder auf echter Hardware startest. Du liest das Protokoll, behebst Fehler mit dem Reiter „Probleme“ und gibst dem Emulator eigene Startparameter mit.

## Das brauchst du

- Ein Projekt mit einem Quelltext, zum Beispiel das aus [Dein erstes Projekt](kapitel:erstes-projekt).
- Einen Emulator, den VintageCoder gefunden hat (siehe [Installation und Ersteinrichtung](kapitel:installation)).

## Bauen, Starten, Debuggen

Es gibt drei Wege vom Quelltext zum laufenden Programm. Du findest sie in der Kopfzeile und im Menü **Projekt**:

| Aktion | Taste | Was passiert |
|---|---|---|
| **Bauen** | **F7** | baut das Projekt: alle Programme der Projektdatei in Baureihenfolge. Hat das Projekt keine Quellen in der Projektdatei, wird die aktive Datei gebaut. |
| **Starten** | **F5** | baut das **Startprogramm** (★ im Projektbaum) und startet es auf dem Startziel: im Emulator oder auf einer eingetragenen Maschine. Es ist egal, welche Datei gerade aktiv ist. Der Kopfzeilenknopf **Starten** tut dasselbe. |
| **Debuggen** | **F6** | baut das Startprogramm, startet es im Debugger und schaltet auf die Debug-Sitzung (siehe [Der Debugger](kapitel:debugger)). |

Mit **Alt+Umschalt** (**Alt+Umschalt+F7**, **Alt+Umschalt+F5**, **Alt+Umschalt+F6**) gilt die **aktive Datei** (die im Editor geöffnete) statt des Startprogramms. Im Menü **Projekt** heißen die Einträge „Projekt bauen“, „Startprogramm starten“, „Startprogramm debuggen“ und „Aktive Datei bauen“, „Aktive Datei starten“, „Aktive Datei debuggen“. Auf dem Mac steht Alt für die Wahltaste. Bevor gebaut wird, speichert VintageCoder alle offenen Dateien.

![Die Kopfzeile mit dem Zielfeld „Commodore 64 · VICE x64sc“ und den Knöpfen Starten, Debuggen und Bauen.](bild:bauen-knopf "Bauen, Starten und Debuggen in der Kopfzeile")

Während ein Bau läuft, zeigt der Knopf **… Baut**. Danach stehen die Ergebnisse im Ordner `build/` deines Projekts:

| Datei | Inhalt |
|---|---|
| `….prg` (Atari: `.xex`, Module: `.crt`, `.car`) | das fertige Programm |
| `.lst` | das **Listing**: Adresse, Bytes und Quelltext nebeneinander |
| `.vs` | die Labels für den Emulator (VICE) |
| `.vcdbg` | Debug-Information für den Debugger |

Für Assembler-Quellen kommen weitere Dateien hinzu (zum Beispiel `.sym.txt` mit der Symboltabelle und `.map.txt` mit der Speicherbelegung), siehe die Hinweise zu den Zwischendateien im Protokoll.

## Das Protokoll lesen

Die Schublade (unten) hat die Reiter **Protokoll**, **Probleme** und **Tests**. In einer Debug-Sitzung kommt der Reiter **Debug** dazu. Beim Bauen wechselt die Schublade zum Reiter **Protokoll**. Dort steht Zeile für Zeile, was passiert ist.

1. Baue dein Projekt mit **F7**.
2. Schau in den Reiter **Protokoll**. Fehlt die Schublade, blende sie mit **Cmd/Strg+J** ein.
3. Bei Erfolg steht dort zum Beispiel „Fertig: 34 Bytes“ und der Pfad des Programms. Hat dein Projekt eine Diskette, kommt eine Zeile mit deren Namen und Größe dazu.
4. Die erzeugten Dateien öffnest du über den Projektbaum. Ordner wie `build/` sind dort sichtbar. Zum Beispiel das **Listing** (`.lst`) zeigt dir, was aus deinen Zeilen geworden ist. Eine Datei als Hexdump siehst du über das Rechtsklickmenü im Projektbaum mit **Als Hex ansehen**.

> **Info:** Eine Tabelle mit den einzelnen Stufen des Baus, mit Dauer und Dateiliste, gibt es in der Oberfläche noch nicht. Auf der Kommandozeile zeigt `vcc build` den Ablauf.

Bei Fehlern steht im Protokoll „Bauen fehlgeschlagen“ mit der Zahl der Probleme.

## Schritt für Schritt: Fehler beheben mit dem Reiter „Probleme“

1. Baue dein Programm. Treten Fehler auf, wechselt die Schublade von selbst zum Reiter **Probleme**.
2. Jede Zeile nennt Datei, Zeilennummer und Meldung. Ein Fehler hat einen Code wie `E202`. Fehler stehen in Rot, Warnungen in Gelb.
3. Ein **Klick** auf die Meldung springt im Editor zur Zeile. Die Zeile ist rot unterstrichen. Ein **Rechtsklick** auf eine Meldung bietet „Problem: Zur Zeile springen“, „Problem: Meldung kopieren“ und „Alle Probleme kopieren“.
4. Behebe den Fehler und baue erneut. Die Zahl hinter dem Reiter **Probleme** („Probleme (1)“) zählt die Meldungen. Mit **F8** und **Umschalt+F8** springst du zum nächsten und zum vorigen Problem.

![Der Reiter „Probleme“ mit dem Eintrag „jumprun.asm:61 E202: unbekanntes Symbol „ende““; die Zeile im Editor darüber ist markiert und rot unterstrichen.](bild:probleme-panel "Der Reiter „Probleme“ listet Fehler und Warnungen, ein Klick springt zur Zeile")

Schlägt der Bau fehl, ohne dass eine Meldung zu einer Zeile gehört (zum Beispiel weil die Projektdatei fehlerhaft ist), zeigt der Reiter **Protokoll** den Grund.

### Der Reiter „Protokoll“

Der Reiter **Protokoll** zeigt die Meldungen aller Vorgänge: Bauen, Starten und Debuggen. Hier findest du auch den Grund, wenn ein Start nicht klappt.

![Der Reiter „Protokoll“ mit den Zeilen eines Baus und Starts: Bauen, Fertig, Gestartet.](bild:ausgabe-panel "Der Reiter „Protokoll“ meldet Bauen, Starten und Debuggen")

## Starten im Emulator

Mit **F5** baust du und startest das Startprogramm. Die Kopfzeile zeigt, wohin:

1. Schau auf das Zielfeld in der Kopfzeile, zum Beispiel „C64 · VICE x64sc ▾“. Links steht das Zielsystem, rechts das **Startziel**.
2. Ein Klick auf das Feld öffnet eine Auswahl. Das Häkchen steht vor der aktuellen Wahl:
   - **Emulator (…):** der Emulator des Systems, mit dem Standard aus den Einstellungen,
   - eine **Maschine**, die du in den Einstellungen als echte Hardware eingetragen hast (siehe unten).
3. Drücke **F5**. Das Fenster des Emulators öffnet sich. Dein Programm wird automatisch geladen und gestartet.

Hat dein Projekt eine **Diskette** (siehe [Projekte und Dateien](kapitel:projekte)) und steht das Startprogramm vorn, startet VICE die Diskette. So kann dein Programm weitere Dateien nachladen.

> **Tipp:** Das Zielfeld wählt nur, *wo* das Programm läuft. Welches Zielsystem gebaut wird, steht im Quelltext (`.target`) bzw. in den Projekt-Einstellungen.

### Das Bild des Emulators (Debug-Sitzung)

In einer Debug-Sitzung gibt es im rechten Panel den Reiter **Bild** (er hat auch ein Symbol in der schmalen Leiste daneben). Er zeigt das **Bild** des laufenden VICE-Emulators (C64, C128, VC-20, Plus/4) direkt in VintageCoder. Das Bild kommt über den Monitor des Emulators. Mit dem Knopf **Bild holen** holst du es sofort. Das Häkchen **bei jedem Halt** aktualisiert es bei jedem Halt des Programms (Haltepunkt, Einzelschritt, Pause). Das Häkchen **Zeitlupe live** aktualisiert das Bild laufend; die Maschine läuft dann nur mit etwa 40 bis 80 Prozent ihres Tempos und ohne Ton. Unter dem Bild steht ein Feld, mit dem du Text an den Emulator sendest (zum Beispiel `RUN`). Mehr zum Debuggen steht im Kapitel [Der Debugger](kapitel:debugger).

> **Info:** Das Bild als Datei speichern, in die Zwischenablage kopieren oder als Projektbild setzen kann die Oberfläche noch nicht.

![Das rechte Panel, Reiter „Bild“, in einer Debug-Sitzung vor dem ersten Bild: der Knopf „Bild holen“, die Häkchen „bei jedem Halt“ und „Zeitlupe live“ und der Hinweis „Noch kein Bild“.](bild:emulator-reiter "Beispielansicht, es läuft kein Emulator: der Reiter „Bild“ vor dem ersten Bild")

## Startparameter für den Emulator

Jeder Emulator hat Startparameter: ein Schalter für das Tempo, die Bildschirmdarstellung, den Ton und vieles mehr. VintageCoder kennt die wichtigsten und bietet sie als Kästchen und Auswahlen an, ohne dass du die Kommandozeile kennen musst.

1. Öffne die **Einstellungen** (Cmd/Strg+Komma) und wechsle zur Seite **Zielsysteme und Hardware**. Wähle links dein System.
2. Beim gewählten Emulator steht der Parametereditor (**Startparameter**). Die Parameter sind in Gruppen geordnet, zum Beispiel für Bild oder System. Fährst du mit der Maus über einen Eintrag, erscheint ein Hinweis mit dem Schalter, dem Standardwert und einem Beispiel.
3. Setze die gewünschten Haken, zum Beispiel **Warp-Modus beim Start** für VICE.
4. Brauchst du etwas, das der Katalog nicht kennt, trägst du es im Feld **Weitere Parameter** ein. Leerzeichen trennen; Pfade mit Leerzeichen setzt du in Anführungszeichen. Platzhalter wie `{project_dir}` (Projektordner), `{output}` (das gebaute Programm) oder `{disk}` (die Diskette des Projekts) kannst du anklicken, um sie einzufügen.
5. Die **Vorschau** zeigt die fertige Kommandozeile mit einem Beispielprojekt („demo“). **Auf Standard zurücksetzen** entfernt alle Parameter wieder.
6. Mit **▶ Test-Start** startest du den Emulator mit diesen Parametern zur Probe; er wird nach kurzer Zeit wieder beendet. Klicke danach auf **Speichern**.

![Einstellungen, Zielsysteme: der aufgeklappte Parametereditor „Startparameter“ mit den Gruppen Ablauf, Bild, Ton und weiteren, dem Feld „Weitere Parameter“, den Platzhaltern und der Vorschau der Kommandozeile.](bild:argumente "Startparameter für den Emulator mit Vorschau der Kommandozeile")

Die Parameter gelten für **F5 und F6**. Einige Schalter setzt VintageCoder selbst (etwa den Autostart des Programms); sie dürfen im freien Feld nicht stehen, und der Editor warnt dich dann.

## Starten auf echter Hardware

Dein Programm läuft auf echter Hardware, wenn du die Maschine eintragen kannst:

- **Ultimate 64 / II+** (für den C64)
- **MEGA65** (am Gerät)

Für die anderen Systeme gibt es keine anbindbare Hardware.

### Schritt für Schritt: Eine Ultimate 64 eintragen

1. Öffne die **Einstellungen** und wechsle zur Seite **Zielsysteme und Hardware**. Wähle das System **Commodore 64**.
2. Rechts steht der Abschnitt **Physische Zielhardware & Netzwerkgeräte**. Klicke auf **+ Ultimate 64 / II+ (Netzwerk)**. Gibt es den Knopf nicht, ist diese Anbindung in deinem Paket nicht enthalten.
3. Trage die **Adresse** deiner Ultimate ein, zum Beispiel `192.168.1.50` (oder `host:port`). Hat die Ultimate ein **Netzwerkpasswort** (in ihrer Firmware eingestellt), gib es im Feld **Passwort** ein; „Zeigen“ blendet es ein. Das Passwort steht in der Einstellungsdatei, die nur für dich lesbar ist.
4. Klicke auf **Verbindung testen**. Das Ergebnis erscheint neben dem Knopf. Beim Start von VintageCoder findet kein Netzwerkzugriff statt; erst dieser Knopf prüft die Verbindung.
5. Wähle bei **Startziel (F5)** deiner Ultimate als Ziel, oder wähle sie im Zielfeld der Kopfzeile.
6. Drücke **F5**. VintageCoder baut die `.prg`-Datei und schickt sie über die Netzwerkschnittstelle der Ultimate: Reset, Laden, RUN.

> **Info:** Auf der Ultimate lassen sich beim Debuggen Speicher lesen und schreiben, pausieren und fortsetzen. Register, Einzelschritt und Haltepunkte gibt es dort nicht. Eine nicht passende Kombination, zum Beispiel ein Atari-Programm auf der Ultimate, meldet das Protokoll.

### MEGA65 am Gerät

Die Hardware **MEGA65** verbindest du per **USB** (serielle Schnittstelle; das Programm wird über den Monitor geladen und gestartet) oder per **Ethernet** (`etherload`, `mega65_ftp`). Für Starten, Kopieren und Debuggen kann jede Aktion eine eigene Verbindung bekommen. Die Einrichtung am Gerät (Core, Netzwerk, Fernzugriff) beschreibt die Hilfeseite [MEGA65 am Gerät](hilfe:mega65-hardware).

> **Achtung:** Der Ethernet-Weg des MEGA65 ist **noch nicht an einem echten Gerät getestet**. Verlasse dich erst darauf, wenn du es selbst ausprobiert hast.

## Häufige Stolpersteine

- **F5 startet nichts.** Prüfe das Zielfeld in der Kopfzeile und den Emulator in den Einstellungen (rot = nicht gefunden). Die Meldung steht im Reiter **Protokoll**.
- **Nach dem Bau sehe ich nichts.** Die Schublade ist eingeklappt. Blende sie mit **Cmd/Strg+J** ein. Bei Fehlern wechselt sie von selbst zum Reiter **Probleme**.
- **Fehler, aber keine Zeile im Editor markiert.** Schau in den Reiter **Protokoll**, dort steht der Grund.
- **Es startet das falsche Programm.** **F5** und der Kopfzeilenknopf **Starten** starten das **Startprogramm** (★ im Projektbaum). **Alt+Umschalt+F5** startet die aktive Datei. Das Startprogramm änderst du in den Projekt-Einstellungen (Menü **Projekt**, Reiter **Quellen**) mit dem Stern vor der Datei.
- **Das Programm läuft, aber der Emulator zeigt „READY.“.** Bei Assembler-Programmen mit `.basicstub` tippst du im Emulator `RUN`, falls der Autostart das Programm nicht gestartet hat.
- **Zwei Emulator-Fenster.** Schließe das alte Fenster, bevor du erneut startest.
- **Die Ultimate ist „Nicht erreichbar“.** Prüfe Adresse und Passwort und ob Rechner und Ultimate im selben Netz sind.

## Wie geht es weiter?

Läuft dein Programm nicht wie erwartet, lernst du im Kapitel [Der Debugger](kapitel:debugger), wie du es anhältst und Schritt für Schritt verfolgst. Das Kapitel [Automatisch testen](kapitel:tests) zeigt, wie VintageCoder dein Programm prüft. Zum Nachschlagen: die Hilfeseiten [Bauen und Starten](hilfe:build) und [Einstellungen](hilfe:settings).
