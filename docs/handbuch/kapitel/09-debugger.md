---
id: debugger
titel: Der Debugger
reihenfolge: 9
stufe: Fortgeschritten
zeit: 30 Minuten
siehe_auch: [bauen-starten, editor, tests, systeme]
referenz: [debugger, breakpoints, memmap]
---

# Der Debugger

**Ziel:** Du lernst, ein Programm im Emulator anzuhalten, Schritt für Schritt auszuführen und dabei zu beobachten, was in den Registern und im Speicher passiert. So findest du Fehler, die man im Quelltext nicht sieht.

Dieses Kapitel gehört zur Stufe „Fortgeschritten“: Es setzt voraus, dass du ein erstes Assemblerprogramm bauen und starten kannst (siehe [Assembler für Einsteiger](kapitel:assembler) und [Bauen und Starten](kapitel:bauen-starten)). Was ein Register oder ein Flag ist, steht im Glossar (`GLOSSAR.md`).

## Das brauchst du

- Ein geöffnetes Projekt, das sich fehlerfrei bauen lässt.
- Einen Emulator für dein Zielsystem. Für C64, C128, Plus/4 und VC-20 ist das VICE, für den Atari 8-Bit atari800, für den MEGA65 Xemu (zur Einrichtung siehe [Willkommen](kapitel:willkommen) und [Zielsysteme im Vergleich](kapitel:systeme)).
- Das Übungsprogramm unten. Du kannst auch das fertige Beispiel `examples/c64/border` benutzen; die Schritte sind dieselben.

## Das Übungsprogramm

Lege ein neues C64-Projekt mit dem Namen `farbwechsel` an (siehe [Dein erstes Projekt](kapitel:erstes-projekt)) und ersetze den Inhalt der Quelldatei durch dieses Programm. Es schaltet in einer Endlosschleife die Rahmenfarbe weiter:

```asm
; Rahmenfarbe weiterschalten – kleines Übungsprogramm für Debugger und Tests (C64)
.target c64
.cpu 6510 { illegals = deny }
.const BORDER  = $d020
.const ZAEHLER = $fb            ; Zeropage: wie oft schon weitergeschaltet

.org $0801
.basicstub start

start:  lda #0
        sta ZAEHLER
schleife:
        jsr naechste_farbe
        jmp schleife

; erhöht den Zähler und zeigt dessen unteres Halbbyte als Rahmenfarbe
naechste_farbe:
        inc ZAEHLER
        lda ZAEHLER
        and #15
        sta BORDER
        rts
```

Das Programm ist absichtlich ohne Rastersynchronisation geschrieben, es läuft also so schnell, wie die CPU kann. Baue es mit **F7**; es entstehen 34 Bytes. Zeilennummern in diesem Kapitel beziehen sich auf genau diesen Quelltext: `inc ZAEHLER` steht in Zeile 18, `sta BORDER` in Zeile 21, `jsr naechste_farbe` in Zeile 13.

## Der Debugger-Modus

Bei einer **Debug-Sitzung** ändert sich die Oberfläche: Die Kopfzeile zeigt statt Starten, Debuggen und Bauen die Steuerknöpfe **▶ Weiter**, **⏸ Anhalten**, **Schritt über**, **Hinein**, **Hinaus** und **■ Beenden**. Die Schublade bekommt den Reiter **Debug** mit Registern, Disassembler, Speicher und mehr. Das rechte Panel bekommt den Reiter **Bild**. Beim Beenden kehrt alles zum vorigen Zustand zurück.

So startest du eine Sitzung:

1. Drücke **F6** (oder klicke in der Kopfzeile auf **Debuggen**, oder wähle **Projekt ▸ Startprogramm debuggen**).
2. VintageCoder baut das Projekt, startet den Emulator und hält das Programm **am Einstiegspunkt** an.
3. Die Oberfläche zeigt die Debug-Ansichten von selbst.

Mit **Alt+Umschalt+F6** (Menü **Projekt ▸ Aktive Datei ▸ Aktive Datei debuggen**) debuggst du statt des Startprogramms die Datei, die gerade im Editor aktiv ist. Läuft schon eine Sitzung, fragt VintageCoder bei **F6** nach, bevor es neu startet.

![Die Debug-Sitzung (Beispielansicht): Editor mit markierter Zeile, unten die Schublade mit den Spalten Register, Disassembler, Speicher und Beobachten.](bild:debug-ueberblick "Beispielansicht, es läuft kein Emulator: die Debug-Sitzung nach F6 mit Kopfzeile, Editor und Schublade")

Das Bild zeigt die Aufteilung (Beispielansicht, es läuft kein Emulator):

- Die **Kopfzeile** oben. Neben den Steuerknöpfen zeigt sie, ob der Emulator läuft oder angehalten ist (zum Beispiel `angehalten · $080D`). Fährst du mit der Maus über einen Knopf, siehst du Name und Taste.
- Der **Editor** in der Mitte, mit deinem Quelltext. Die Zeile, an der das Programm steht, ist markiert.
- Das **rechte Panel** (im Bild zugeklappt). Hier liegt in einer Sitzung der Reiter **Bild** mit dem Emulatorbild (siehe unten).
- Die **Schublade** unten mit den Reitern **Debug**, **Protokoll**, **Probleme** und **Tests**. Der Reiter **Debug** hat nebeneinander die Spalten **Register**, **Disassembler**, **Speicher** und rechts davon **Beobachten**, **Haltepunkte** und **Momentaufnahme**. In einem schmalen Fenster (unter 900 Punkten) ist es eine Spalte mit den Reitern Register, Disassembler, Speicher und Beobachten.

> **Tipp:** Fehlt dir eines der Felder, blendest du es mit **Strg/Cmd+B** (linke Spalte), **Strg/Cmd+Alt+B** (rechtes Panel) oder **Strg/Cmd+J** (Schublade) ein.

> **Info:** Während der Emulator läuft, zeigen Register, Disassembler und Speicher nichts an: Der Zustand der Maschine ist nur im angehaltenen Zustand lesbar. Du siehst dann den Hinweis „Das Programm läuft. „Anhalten“ zeigt die Werte.“

### Was der Emulator kann

Nicht jeder Emulator kann alles. Die Übersicht:

| Emulator | Register | Einzelschritt | Haltepunkte | Datenhaltepunkte |
|---|---|---|---|---|
| VICE (C64, C128, Plus/4, VC-20) | ja | ja | mehrere | ja |
| atari800 (Atari 8-Bit) | ja | ja | nur einer zugleich | nein |
| MEGA65 (Xemu oder Hardware) | ja | ja | einer | nur als langsame Schreib-Beobachtung per Einzelschritt |
| Ultimate 64 / II+ (Netzwerk) | nein | nein | nein | nein |

Bei der Ultimate kannst du Speicher lesen und schreiben sowie pausieren und fortsetzen, mehr nicht. atari800 lässt sich nicht per Knopf anhalten: Dort drückst du **F8 im Emulatorfenster** oder setzt vorher einen Haltepunkt. Kann der Emulator einen Einzelschritt gerade nicht, nennt ein Hinweis in der Debug-Zeile den Grund.

## Haltepunkte

Ein **Haltepunkt** sagt dem Emulator: „Halte an, bevor du diese Zeile ausführst.“

### Haltepunkt auf eine Zeile

1. Wechsle zum Editor und **klicke in den Rand links neben der Zeilennummer** von Zeile 18 (`inc ZAEHLER`). Dort erscheint ein roter Punkt. Oder setze den Cursor in die Zeile und drücke **F9**.
2. Ein zweiter Klick (oder **F9**) entfernt den Haltepunkt wieder. Das geht auch über das Kontextmenü (rechte Maustaste im Editor): **Haltepunkt umschalten**.
3. Drücke **F6**. Das Programm startet und hält an, sobald es Zeile 18 erreicht. Die Zeile ist markiert; die Kopfzeile zeigt `angehalten · $0817`.

![Roter Haltepunkt-Punkt neben der Zeilennummer 60 und die angehaltene Zeile 61 (Beispielansicht).](bild:debug-haltepunkt-gutter "Beispielansicht, es läuft kein Emulator: der rote Punkt links neben der Zeilennummer ist ein Haltepunkt")

![Die angehaltene Zeile 61 mit hellem Streifen (Beispielansicht).](bild:debug-pc-zeile "Beispielansicht, es läuft kein Emulator: der helle Streifen zeigt die Zeile, an der das Programm steht; der Befehl in dieser Zeile ist noch nicht ausgeführt")

> **Info:** Haltepunkte im Editor kannst du schon vor dem Start setzen. VintageCoder ordnet sie über die Zeilenzuordnung des letzten Baus einer Adresse zu. Hast du den Quelltext seit dem Bauen geändert, baue neu (F7 oder einfach F6, das baut mit).

Im Disassembler setzt ein Klick auf den Rand vor einer Zeile ebenfalls einen Haltepunkt (siehe unten).

Die Spalte **Haltepunkte** in der Debug-Fläche listet alle Punkte der Sitzung mit Art (`exec` für Ausführung), Adresse, Quellzeile, Bedingung (`wenn …`) und Trefferzähler (`3×`). Ein Klick auf das kleine **✕** entfernt einen Punkt.

### Bedingte Haltepunkte und Datenhaltepunkte

Ein **bedingter Haltepunkt** hält nur an, wenn ein Ausdruck stimmt. Ein **Datenhaltepunkt** hält an, wenn auf eine Speicherstelle zugegriffen wird. Beides kann der Debugger im Kern, und die Liste zeigt Bedingungen an. Ein Formular, um solche Haltepunkte in der Oberfläche anzulegen (mit Art, Adresse, Bedingung und Ignorieren), gibt es **noch nicht**. Auch das Anlegen von Haltepunkten per Rechtsklick auf eine Speicherzeile gibt es nicht. Datenhaltepunkte gibt es nur mit VICE.

Die Ausdrücke selbst kennst du später noch aus dem Abschnitt „Beobachten“ und aus dem Kapitel [Automatisch testen](kapitel:tests).

## Einzelschritt

Ist das Programm angehalten, steuerst du es mit diesen Knöpfen und Tasten. Alle stehen auch im Menü **Projekt**:

| Knopf | Taste | Menüeintrag | Was passiert |
|---|---|---|---|
| **▶ Weiter** | Alt+F5 | Debuggen: Weiter | läuft bis zum nächsten Haltepunkt |
| **Hinein** | F11 | Debuggen: Schritt hinein | führt genau einen Befehl aus; ein `JSR` springt *hinein* in das Unterprogramm |
| **Schritt über** | F10 | Debuggen: Schritt über | führt einen Befehl aus; ein `JSR` läuft *ganz durch* und hält dahinter an |
| **Hinaus** | Umschalt+F11 | Debuggen: Schritt hinaus | läuft bis zum Rücksprung (`RTS` oder `RTI`) des aktuellen Unterprogramms |
| **⏸ Anhalten** | Alt+F6 | Debuggen: Anhalten | hält den laufenden Emulator an (bei atari800: F8 im Emulatorfenster) |
| **■ Beenden** | Umschalt+F5 | Debuggen beenden | beendet die Debug-Sitzung |

> **Info:** Weiter, Schritt über, Hinein und Hinaus wirken nur, wenn das Programm angehalten ist; sonst sind die Knöpfe grau.

**Probiere es aus.** Entferne vorher alle Haltepunkte und setze einen auf Zeile 13 (`jsr naechste_farbe`), bevor du die Sitzung startest:

1. Drücke **F6**. Das Programm hält in Zeile 13 an.
2. Drücke **F11** (Hinein). Du bist *in* `naechste_farbe` gelandet, auf `inc ZAEHLER` (Zeile 18).
3. Drücke noch viermal **F11**: `inc`, `lda`, `and` und `sta` werden ausgeführt, dann steht der Programmzähler auf dem `rts`. Beobachte nach jedem Schritt die Register (siehe unten).
4. Drücke **Umschalt+F11** (Hinaus). Das `rts` wird ausgeführt, und das Programm hält hinter dem Aufruf an, in Zeile 14 (`jmp schleife`).
5. Drücke **F10** (Schritt über) zweimal. Jetzt hält der Debugger in Zeile 13 wieder an, dann steht er hinter dem `jsr`. **Schritt über** hat das ganze Unterprogramm in einem Rutsch ausgeführt.

**Hinaus** zählt mit: Aufrufe innerhalb des Unterprogramms werden übersprungen, sonst führt VintageCoder Einzelschritte aus. Nach zehn Sekunden ohne Rücksprung bricht es mit einer Meldung ab. Setze dann einen Haltepunkt direkt am Rücksprung.

> **Tipp:** Willst du nur bis zu einer bestimmten Stelle laufen, ohne einen dauerhaften Haltepunkt zu setzen: Setze im Editor den Cursor in die Zeile und wähle im Kontextmenü (rechte Maustaste) oder im Menü **Projekt** den Eintrag **Debuggen: Bis hierher ausführen**. Das ist ein einmaliger Haltepunkt, der nach dem nächsten Halt verschwindet.

Läuft der Einzelschritt aus einem Grund nicht (zum Beispiel weil der Emulator es nicht kann), nennt ein Hinweis in der Debug-Zeile den Grund.

## Die Register

Die Spalte **Register** zeigt den Zustand der CPU:

![Die Spalte Register: PC, A, X, Y, SP und die Flag-Leiste (Beispielansicht).](bild:debug-register "Beispielansicht, es läuft kein Emulator: der Programmzähler PC, die Rechenregister A, X und Y, der Stapelzeiger SP und die Flags N V B D I Z C")

- **PC**: Adresse des nächsten Befehls (der noch *nicht* ausgeführt ist).
- **A**, **X**, **Y**: die Rechenregister.
- **SP**: der Stapelzeiger (der Stapel liegt auf Seite 1, also ab `$0100`).
- Die **Flags** N V B D I Z C als Leiste aus Kästchen. Ein Flag, das gesetzt ist, ist gefüllt. Die Leiste ist reine Anzeige; ein Klick ändert nichts im Emulator.
- Je Register stehen Hex- und Dezimalwert in der Tabelle. Hat sich ein Wert beim letzten Schritt verändert, ist er farbig hervorgehoben, und daneben steht, was vorher drin war („war 05“). Geänderte Flags sind umrandet.
- Bei VICE kommen unter den Flags weitere Werte des Emulators hinzu, zum Beispiel die Rasterposition.

> **Info:** Eine Ansicht der Chip-Register (VIC, SID, CIA …) gibt es in der Oberfläche noch nicht. Die Register eines Chips kannst du über die Adresse im Speicher ansehen, zum Beispiel `D020` für die Rahmenfarbe.

## Der Disassembler

Die Spalte **Disassembler** zeigt den **Maschinencode ab dem Programmzähler**, so wie die CPU ihn sieht: Rand, Adresse, Bytes, Befehl und rechts die Zahl der Zyklen.

![Der Disassembler mit Programmzähler-Zeile und Haltepunkt (Beispielansicht).](bild:debug-disassembler "Beispielansicht, es läuft kein Emulator: der Disassembler mit dem Pfeil an der Zeile des Programmzählers, einem roten Punkt für den Haltepunkt, Adresse, Bytes und Befehl")

Praktisch daran:

- Der **Rand** ganz links zeigt `▶` an der Zeile, an der das Programm steht, und `●` an Haltepunkten. Ein **Klick auf den Rand** setzt dort einen Haltepunkt oder entfernt ihn. Der Haltepunkt gilt über die Zeilenzuordnung des Quelltexts; an Adressen ohne Quellzeile meldet das Protokoll, dass kein Haltepunkt möglich ist.
- Stehen Marken aus deinem Quelltext an einer Adresse, siehst du sie vor dem Befehl (`naechste_farbe:`).
- Befehle, die gar nicht zum offiziellen Befehlssatz gehören, sind farbig abgesetzt.

## Der Speicher

Die Spalte **Speicher** zeigt den Arbeitsspeicher des laufenden Rechners:

![Die Spalte Speicher mit Hexdump (Beispielansicht).](bild:debug-speicher "Beispielansicht, es läuft kein Emulator: oben das Adressfeld, darunter der Hexdump mit 8 Bytes je Zeile und rechts die Zeichen")

1. Gib im Adressfeld eine Hex-Zahl ein, zum Beispiel `00FB`, oder einen Namen aus deinem Programm (`ZAEHLER`), und drücke Enter.
2. Führe einen Schritt aus oder drücke **Weiter** bis zum nächsten Halt: Bytes, die sich *seit dem vorherigen Halt* geändert haben, sind farbig und fett. So siehst du auf einen Blick, was dein Programm verändert hat.

Blättern per Knopf, ein Rechtsklickmenü auf den Zeilen und das Speichern eines Speicherbereichs als Zeichensatz gibt es in der Oberfläche noch nicht. Einen Zeichensatz aus dem Rechner übernimmst du mit der **Momentaufnahme** (siehe unten).

## Beobachten

In der Spalte **Beobachten** legst du **Ausdrücke** an, die VintageCoder bei *jedem* Halt neu ausrechnet. So musst du nicht ständig in den Speicher schauen.

1. Tippe einen Ausdruck in das Feld, zum Beispiel `a`, `x + 1`, `[$d020]` oder `[ZAEHLER]`.
2. Drücke Enter.
3. Schritt für Schritt (F11) siehst du, wie sich der Wert ändert. Er steht dezimal und hexadezimal da, zum Beispiel `5 · $5`.

![Die Spalte Beobachten mit drei Ausdrücken und die Liste der Haltepunkte (Beispielansicht).](bild:debug-beobachten "Beispielansicht, es läuft kein Emulator: oben das Eingabefeld, darunter die Ausdrücke mit ihrem Wert dezimal und in Hex")

In Ausdrücken sind erlaubt:

| Was | Schreibweise |
|---|---|
| Register | `a`, `x`, `y`, `sp`, `pc`, `p` |
| Flags | `fn`, `fv`, `fb`, `fd`, `fi`, `fz`, `fc` |
| Speicher | `[$d020]` oder `peek($d020)`; ein Wort mit `peekw($fffe)` |
| Rasterposition | `lin` (Zeile), `cyc` (Zyklus) |
| Namen aus deinem Programm | zum Beispiel `[ZAEHLER]` (Marken und Konstanten der letzten Übersetzung) |

Operatoren gelten wie in C (`==`, `!=`, `<`, `>`, `&&`, `||`, `&`, `|`). Vergleiche binden stärker als `&` und `|`; im Zweifel setze Klammern, zum Beispiel `(x & 15) == 5 && fc`. Ist ein Ausdruck ungültig, steht an seiner Stelle „unbekannt“ und darunter die Fehlermeldung in Rot. Mit dem **✕** neben dem Ausdruck entfernst du ihn.

## Momentaufnahme

Du hast ein Programm, das etwas Schönes auf den Bildschirm bringt, und möchtest daraus einen Zeichensatz oder Sprites in dein Projekt übernehmen? Die **Momentaufnahme** (rechts in der Debug-Fläche) liest aus dem angehaltenen Rechner den aktuellen Bildschirm, den Zeichensatz und die aktiven Sprites und legt sie als Dateien im Projekt ab (unter `graphics/snapshot/`). Das geht für den C64 und den Atari 8-Bit.

1. Halte das Programm an der Stelle an, die du festhalten willst (Haltepunkt oder **Anhalten**).
2. Klicke bei **Momentaufnahme** auf **Bildschirm erkennen**. VintageCoder erkennt aus den Hardware-Registern, wo Bildschirm und Zeichensatz liegen, und zeigt die Adressen und die Zahl der Sprites an.
3. Hake an, was zusätzlich gespeichert werden soll: **Sprites speichern**, **Zeichen als Kacheln**. Liegt der Zeichensatz im ROM, gibt es außerdem **Zeichensatz aus dem ROM lesen**.
4. Vergib einen **Namen** (nur Buchstaben, Ziffern, `_` und `-`) und klicke **Als Dateien speichern**.

Es entstehen je nach Auswahl ein Bildschirm (`.vcscreen`), ein Zeichensatz (`.vcchar`), Sprites (`.vcsprite`) und ein Kachelsatz (`_tiles.vctiles`). Nicht möglich sind Bitmap-Modi und ANTIC-Modi außer Modus 2.

> **Info:** Eine Vorschau und das Ändern der erkannten Adressen gibt es in der Oberfläche noch nicht.

> **Achtung:** Die Standardschriften der Rechner gehören den Herstellern. Checke einen so gespeicherten Zeichensatz nicht in ein öffentliches Repository ein.

## Die Memory Map

Die **Memory Map** hat mit dem laufenden Programm nichts zu tun, sie zeigt dir den Speicher *deines gebauten Programms*. Sie heißt im rechten Panel **Karte** (Symbol ▦). Wichtig: Sie arbeitet mit dem Ergebnis der **letzten Übersetzung**. Ohne Bau steht dort „Noch nichts gebaut: Nach dem Bauen zeigt die Karte, wo dein Programm im Speicher liegt.“

![Die Karte im rechten Panel: 16×16 Seiten, gefärbt nach Art (Zeropage, Stack, BASIC, ROM, E/A, frei, Programm), darunter die Auswahl des Prozessorports und die Angaben zur gewählten Seite (Beispielansicht).](bild:memory-map "Beispielansicht, es läuft kein Emulator: die Matrix aus 256 Seiten, gefärbt nach Art des Speichers, mit Filter und den Angaben zur gewählten Seite")

- Jede Kachel der **Matrix** ist eine Seite von 256 Byte. Die Farben zeigen RAM, ROM, E/A und deine Segmente; ein Balken in der Kachel zeigt, wie viel davon dein Programm belegt.
- **Filter**: Alle, Programm, RAM, ROM, E/A.
- Ein Klick auf eine Seite zeigt darunter Adresse, Art des Speichers, wie viele Bytes dein Programm dort belegt, die Symbole auf der Seite und die Quellzeilen.
- Unter **Prüfung** stehen Hinweise: Überlappungen, ob der Stapel frei ist, wie viel Zeropage dein Programm belegt.
- Beim C64, C128 und Atari 800 XL/XE schalten die **Bank-Presets** die Sicht um (zum Beispiel beim C64 „Standard (BASIC, E/A, KERNAL)“ oder „Alles RAM“).

Das Nachschlagewerk beschreibt alles Weitere: [Memory Map](hilfe:memmap).

## Das Bild des Emulators

Der Reiter **Bild** im rechten Panel (nur in einer Debug-Sitzung) zeigt das **Bild des Emulators direkt in VintageCoder**. Das geht für VICE-Systeme (C64, C128, Plus/4, VC-20), nicht für Atari und MEGA65.

![Das rechte Panel, Reiter „Bild“, in einer Debug-Sitzung vor dem ersten Bild: der Knopf „Bild holen“, die Häkchen „bei jedem Halt“ und „Zeitlupe live“ und der Hinweis „Noch kein Bild“.](bild:emulator-reiter "Beispielansicht, es läuft kein Emulator: der Reiter Bild: oben die Knöpfe und Häkchen, darunter das Bild der Maschine, sobald es geholt ist, und das Feld für Text")

- **Bild holen** holt das Bild sofort. Mit dem Häkchen **bei jedem Halt** (Haltepunkt, Einzelschritt, Pause) holt VintageCoder es selbst. Du siehst also genau den Zustand, in dem die Maschine steht.
- **Zeitlupe live**: VintageCoder hält die Maschine alle paar Millisekunden kurz an, holt das Bild und setzt fort. Die Maschine läuft dabei mit etwa 40 bis 80 Prozent ihres Tempos und ohne Ton. Zum Spielen ist das nicht gedacht, zum Beobachten von Änderungen reicht es.
- **Text an den Emulator senden** schreibt Text in den Tastaturpuffer des Emulators, zum Beispiel `RUN`. Enter sendet ihn. Mehr als Text wird nicht weitergegeben: Tastatur und Joystick gehen nicht zum Emulator.
- Beim C128 wählst du über Knöpfe über dem Bild zwischen **VIC-II** (40 Spalten) und **VDC** (80 Spalten).

> **Info:** Das Bild als PNG speichern, in die Zwischenablage kopieren oder als Projektbild setzen kann die Oberfläche noch nicht.

## Das Live-Bild (Ultimate 64)

> **Info:** Ein Live-Bild des Bildschirms einer Ultimate 64 gibt es in dieser Version nicht. Starten, Speicher lesen und Tests über das Netzwerk sind möglich (siehe [Zielsysteme im Vergleich](kapitel:systeme)).

## Häufige Stolpersteine

| Problem | Ursache und Lösung |
|---|---|
| Nach F6 passiert nichts | Der Bau ist fehlgeschlagen: Schau in den Reiter **Probleme**. Oder es läuft schon eine Debug-Sitzung: beende sie mit **■ Beenden** (Umschalt+F5). |
| Register, Disassembler und Speicher sind leer | Der Emulator läuft. Drücke **⏸ Anhalten** oder setze einen Haltepunkt. |
| Der Haltepunkt wird nie getroffen | Die Zeile wird nicht erreicht, oder du hast den Quelltext nach dem Bau geändert: neu bauen. Bei atari800 gilt: nur ein Haltepunkt gleichzeitig. |
| Der Einzelschritt geht nicht | Der Emulator unterstützt den Schritt in dieser Lage nicht; der Hinweis in der Debug-Zeile nennt den Grund. |
| Ein Ausdruck zeigt „unbekannt“ | Prüfe die Schreibweise und die Namen. Die Fehlermeldung steht darunter. Klammere bei `&` und `|`. |
| **Hinaus** bricht nach zehn Sekunden ab | Es gab keinen Rücksprung. Setze einen Haltepunkt auf das `RTS` des Unterprogramms. |
| Der Reiter Bild zeigt kein Bild | Dein System liefert keines (Atari, MEGA65), oder die Maschine läuft: „Bild holen“. |

## Wie geht es weiter?

- Prüfe dein Programm automatisch statt von Hand: [Automatisch testen](kapitel:tests).
- Alles über die Besonderheiten deines Zielsystems: [Zielsysteme im Vergleich](kapitel:systeme).
- Alle Tasten auf einen Blick: [Tastenkürzel, FAQ, Fehlersuche](kapitel:kuerzel-faq).
- Zum Nachschlagen: [Debugger](hilfe:debugger), [Haltepunkte](hilfe:breakpoints), [Memory Map](hilfe:memmap).
