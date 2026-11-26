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

Das Programm ist absichtlich ohne Rastersynchronisation geschrieben, es läuft also so schnell, wie die CPU kann. Baue es mit **F7** (Projekt ▸ Projekt bauen); es entstehen 34 Bytes. Zeilennummern in diesem Kapitel beziehen sich auf genau diesen Quelltext: `inc ZAEHLER` steht in Zeile 18, `sta BORDER` in Zeile 21, `jsr naechste_farbe` in Zeile 13.

## Der Bereich Debug

Der Debugger hat in VintageCoder einen **eigenen Hauptbereich**: **Debug** in der Hauptreihe oben (neben Projekt, Code, Grafik, Musik und Daten). Ohne laufende Sitzung zeigt er die Karte **Keine Debug-Sitzung** mit dem Knopf **Debuggen starten**. Während einer Sitzung steht in der Statusleiste unten ein Eintrag (zum Beispiel „Debug: angehalten bei $0817“); ein Klick darauf bringt dich zurück in den Bereich Debug.

So startest du eine Sitzung:

1. Drücke **F6** (oder wähle **Projekt ▸ Startprogramm debuggen**, oder klicke im Bereich Debug auf **Debuggen starten**).
2. VintageCoder baut das Projekt, startet den Emulator und hält das Programm **am Einstiegspunkt** an.
3. Die Oberfläche wechselt von selbst in den Bereich **Debug**.

Mit **Alt+Umschalt+F6** (Menü **Projekt ▸ Aktive Datei ▸ Aktive Datei debuggen**) debuggst du statt des Startprogramms die Datei, die gerade im Editor aktiv ist. Ist der letzte Bau fehlgeschlagen, steht im Bereich Debug der Hinweis „Der letzte Bau ist fehlgeschlagen. Erst die Fehler beheben.“ Dann schau in den Reiter **Probleme**.

![Die Debug-Ansicht (Reiter Debug aktiv): links die Spalte mit HALTEPUNKTE, BEOBACHTEN, MOMENTAUFNAHME und CHIPS, in der Mitte eine Werkzeugleiste, der Disassembler mit Zeilen ab $0801 und darunter der Speicher ab $0400, rechts REGISTER mit dem Zustand ANGEHALTEN, Stapel und BILDSCHIRM.](bild:debug-ueberblick "Beispielansicht, es läuft kein Emulator: die Debug-Ansicht mit Haltepunkten, Disassembler, Speicher und Registern.")

Der Bereich ist in drei Spalten aufgeteilt:

- **Links:** **Haltepunkte**, **Beobachten**, **Momentaufnahme** und **Chips**.
- **In der Mitte:** die Werkzeugleiste mit den Schrittknöpfen und dem Zustand der Sitzung, darunter der **Disassembler** und der **Speicher**.
- **Rechts:** **Register** mit den Flags, der **Stapel** und der **Bildschirm** des Emulators.

In einem schmalen Fenster rutschen die Spalten untereinander. Das Panel **Maschine** (**Cmd/Strg+Alt+B**) zeigt in einer Sitzung ebenfalls die Register und Flags sowie das Bild des Emulators in klein; die ganze Ansicht hast du im Bereich Debug.

> **Info:** Während der Emulator läuft, zeigen Register, Disassembler und Speicher nichts an: Der Zustand der Maschine ist nur im angehaltenen Zustand lesbar. Du siehst dann den Hinweis „Das Programm läuft. Anhalten zeigt Register, Disassembler und Speicher.“

### Was der Emulator kann

Nicht jeder Emulator kann alles. Die Übersicht:

| Emulator | Register | Einzelschritt | Haltepunkte | Datenhaltepunkte |
|---|---|---|---|---|
| VICE (C64, C128, Plus/4, VC-20) | ja | ja | mehrere | ja |
| atari800 (Atari 8-Bit) | ja | ja | nur einer zugleich | nein |
| MEGA65 (Xemu oder Hardware) | ja | ja | einer | nur als langsame Schreib-Beobachtung per Einzelschritt |
| Ultimate 64 / II+ (Netzwerk) | nein | nein | nein | nein |

Bei der Ultimate kannst du Speicher lesen und schreiben sowie pausieren und fortsetzen, mehr nicht. atari800 lässt sich nicht per Knopf anhalten: Dort drückst du **F8 im Emulatorfenster** oder setzt vorher einen Haltepunkt. Kann der Emulator einen Einzelschritt gerade nicht, steht der Grund als Hinweis unter der Werkzeugleiste. Rechts oben in der Werkzeugleiste steht der Name des Emulators, mit dem die Sitzung läuft.

## Haltepunkte

Ein **Haltepunkt** sagt dem Emulator: „Halte an, bevor du diese Zeile ausführst.“

### Haltepunkt auf eine Zeile

1. Wechsle in den Bereich **Code** und **klicke in den Rand links neben der Zeilennummer** von Zeile 18 (`inc ZAEHLER`). Dort erscheint ein roter Punkt. Oder setze den Cursor in die Zeile und drücke **F9** (Menü **Bearbeiten ▸ Haltepunkt umschalten**).
2. Ein zweiter Klick (oder **F9**) entfernt den Haltepunkt wieder.
3. Drücke **F6**. Das Programm startet und hält an, sobald es Zeile 18 erreicht. In der Werkzeugleiste steht dann zum Beispiel `Angehalten bei $0817`, daneben die Quellzeile.

*[Bild folgt in E13: debug-haltepunkt-gutter]*

> **Info:** Haltepunkte im Editor kannst du schon vor dem Start setzen. VintageCoder ordnet sie über die Zeilenzuordnung des letzten Baus einer Adresse zu. Hast du den Quelltext seit dem Bauen geändert, baue neu (F7 oder einfach F6, das baut mit).

Im **Disassembler** setzt ein Klick auf den runden Knopf vor einer Zeile ebenfalls einen Haltepunkt (siehe unten).

Die Liste **Haltepunkte** links im Bereich Debug zeigt alle Punkte der Sitzung: die Art (Ausführen, Lesen, Schreiben oder Zugriff), die Adresse oder den Adressbereich, eine Bedingung (`if …`), die Zahl der zu ignorierenden Treffer (`×3`) und die Zahl der bisherigen Treffer (`2×`). Das **✕** am Zeilenende löscht den Punkt. Haltepunkte aus dem Editor stehen ebenfalls in der Liste, blass und mit dem Zusatz **Editor** und `datei:zeile`. Gibt es keine, steht dort „Keine Haltepunkte. Ein Klick links neben eine Zeile setzt einen.“

### Haltepunkte von Hand anlegen: Adresse, Bedingung, Datenzugriff

Unter der Liste liegt ein kleines Formular für Haltepunkte, die du nicht über eine Quelltextzeile setzen willst:

1. Wähle die **Art des Punkts**: **Ausführen**, **Lesen**, **Schreiben** oder **Zugriff**. Die drei letzten sind **Datenhaltepunkte** (der Emulator hält an, wenn auf eine Speicherstelle zugegriffen wird) und nur wählbar, wenn der Emulator das kann, also mit VICE.
2. Gib die **Adresse** ein, zum Beispiel `$D020`, `0x0400` oder `2049`. Ein Symbolname aus deinem Programm geht in diesem Feld nicht; bei einer ungültigen Eingabe erscheint „Das ist keine Adresse. Beispiele: $D020, 0x0400, 2049.“
3. Bei Datenhaltepunkten kommt das Feld **Ende (bei Bereich)** dazu. Lässt du es leer, gilt der Punkt für die eine Adresse; sonst für den Bereich von Adresse bis Ende.
4. Bei **Bedingung** schreibst du optional einen Ausdruck, zum Beispiel `a == 5`. Der Punkt hält dann nur an, wenn der Ausdruck stimmt. Die Schreibweise steht unten im Abschnitt „Beobachten“.
5. Bei **Ignorieren** gibst du an, wie viele Treffer übersprungen werden, bevor der Punkt anhält.
6. Klicke auf **Hinzufügen**.

*[Bild folgt in E13: debug-bedingung]*

Stimmt etwas mit einer Bedingung nicht, zeigt eine rote Meldung unter der Liste den Fehler. Die Ausdrücke selbst kennst du auch aus dem Kapitel [Automatisch testen](kapitel:tests).

## Einzelschritt

Ist das Programm angehalten, steuerst du es mit den Knöpfen der Werkzeugleiste oder mit den Tasten. Die Knöpfe haben Symbole; fährst du mit der Maus darüber, erscheint der Name. Alle Befehle stehen auch im Menü **Projekt ▸ Debuggen** und in der Befehlspalette (**Cmd/Strg+K**):

| Name (Knopf und Menü) | Taste | Was passiert |
|---|---|---|
| **Debuggen: Weiter** | Alt+F5 | läuft bis zum nächsten Haltepunkt |
| **Debuggen: Schritt hinein** | F11 | führt genau einen Befehl aus; ein `JSR` springt *hinein* in das Unterprogramm |
| **Debuggen: Schritt über** | F10 | führt einen Befehl aus; ein `JSR` läuft *ganz durch* und hält dahinter an |
| **Debuggen: Schritt hinaus** | Umschalt+F11 | läuft bis zum Rücksprung (`RTS` oder `RTI`) des aktuellen Unterprogramms |
| **Debuggen: Anhalten** | Alt+F6 | hält den laufenden Emulator an (bei atari800: F8 im Emulatorfenster) |
| **Debuggen beenden** | Umschalt+F5 | beendet die Debug-Sitzung |
| **Debuggen: Bis hierher ausführen** | keine | läuft bis zur Zeile, in der im Editor der Cursor steht |

> **Info:** Schritt über, hinein und hinaus wirken nur, wenn das Programm angehalten ist; sonst sind die Knöpfe grau. **Anhalten** ist nur wählbar, solange das Programm läuft.

**Probiere es aus.** Entferne vorher alle Haltepunkte und setze einen auf Zeile 13 (`jsr naechste_farbe`), bevor du die Sitzung startest:

1. Drücke **F6**. Das Programm hält in Zeile 13 an.
2. Drücke **F11** (Schritt hinein). Du bist *in* `naechste_farbe` gelandet, auf `inc ZAEHLER` (Zeile 18).
3. Drücke noch viermal **F11**: `inc`, `lda`, `and` und `sta` werden ausgeführt, dann steht der Programmzähler auf dem `rts`. Beobachte nach jedem Schritt die Register (siehe unten).
4. Drücke **Umschalt+F11** (Schritt hinaus). Das `rts` wird ausgeführt, und das Programm hält hinter dem Aufruf an, in Zeile 14 (`jmp schleife`).
5. Drücke **F10** (Schritt über) zweimal. Jetzt hält der Debugger in Zeile 13 wieder an, dann steht er hinter dem `jsr`. **Schritt über** hat das ganze Unterprogramm in einem Rutsch ausgeführt.

**Schritt hinaus** zählt mit: Aufrufe innerhalb des Unterprogramms werden übersprungen, sonst führt VintageCoder Einzelschritte aus. Nach zehn Sekunden ohne Rücksprung bricht es mit einer Meldung ab. Setze dann einen Haltepunkt direkt am Rücksprung.

> **Tipp:** Willst du nur bis zu einer bestimmten Stelle laufen, ohne einen dauerhaften Haltepunkt zu setzen: Setze im Editor den Cursor in die Zeile und wähle **Projekt ▸ Debuggen ▸ Debuggen: Bis hierher ausführen**. Das ist ein einmaliger Haltepunkt, der nach dem nächsten Halt verschwindet. Steht der Cursor in keiner Quelldatei, kommt der Hinweis „Cursor in eine Quelldatei setzen, dann „Bis hierher ausführen“.“

## Der Disassembler

Der **Disassembler** in der Mitte zeigt den **Maschinencode ab dem Programmzähler**, so wie die CPU ihn sieht: Haltepunkt-Knopf, Adresse, Bytes, Befehl und rechts die Zahl der Zyklen (ein `+` dahinter heißt: unter Umständen ein Zyklus mehr).

![Der Disassembler als Liste: pro Zeile ein Haltepunkt-Kreis, Adresse, Bytes, Befehl und rechts die Zyklenzahl. Die erste Zeile ab $0801 ist hervorgehoben und trägt rechts den Verweis auf jumprun.asm, darunter steht die Marke „start:“.](bild:debug-disassembler "Beispielansicht: der Disassembler mit Adresse, Bytes, Befehl und Zyklen je Zeile.")

Praktisch daran:

- Die Zeile, an der das Programm steht, ist hervorgehoben. Rechts in dieser Zeile steht ein Link `datei:zeile`; ein Klick darauf springt in den Editor an diese Quellzeile (der Bereich wechselt zu **Code**).
- Der **runde Knopf** am linken Rand jeder Zeile ist ein Haltepunkt: Ein Klick setzt ihn (er wird rot), ein zweiter Klick entfernt ihn. Der Haltepunkt gilt für die Adresse dieser Zeile.
- Stehen Marken aus deinem Quelltext an einer Adresse, siehst du sie vor dem Befehl (`naechste_farbe:`).
- Befehle, die gar nicht zum offiziellen Befehlssatz gehören, sind farbig abgesetzt.

## Die Register

Die Spalte rechts oben zeigt den Zustand der CPU. Daneben steht, ob die Sitzung **ANGEHALTEN** ist oder **Läuft**.

![Schmale Seitenspalte mit den Registern PC $0801, A $00, X $00, Y $00 und SP $F8, den Flags N, V, B, D, I, Z und C mit hervorgehobenem I, dem Stapel, einer leeren Bildschirmanzeige und Schaltern. Oben rechts steht „ANGEHALTEN“.](bild:debug-register "Beispielansicht: die Debug-Spalte mit Registern, Flags, Stapel und Bildschirm.")

- **PC**: Adresse des nächsten Befehls (der noch *nicht* ausgeführt ist).
- **A**, **X**, **Y**: die Rechenregister.
- **SP**: der Stapelzeiger (der Stapel liegt auf Seite 1, also ab `$0100`).
- Die **Flags** N V B D I Z C als Leiste aus Kästchen. Ein Flag, das gesetzt ist, ist hervorgehoben. Die Leiste ist reine Anzeige; ein Klick ändert nichts im Emulator.
- Jedes Register steht als Hexwert in einem Feld. Hat sich ein Wert beim letzten Schritt verändert, ist sein Feld hervorgehoben; ein geändertes Flag ist umrandet.
- Bei manchen Emulatoren kommen weitere Werte dazu, zum Beispiel die Rasterposition (bei VICE).

Darunter zeigt der **Stapel** die Bytes, die auf dem Stapel liegen, jeweils mit ihrer Adresse (`$01FF` abwärts). Ist er leer, steht dort „Stapel leer“.

### Die Chips

Links im Bereich Debug steht der Abschnitt **Chips**. Für dein Zielsystem listet er die Bausteine (zum Beispiel der Videochip oder der Soundchip) als Kontrollkästchen. Hakst du einen an, erscheint darunter ein Fenster mit den **Registern dieses Chips**: Adresse, Name und aktueller Wert (`$..`). Fährst du mit der Maus über ein Register, siehst du seine Beschreibung. Die Werte aktualisieren sich bei jedem Halt; während das Programm läuft, stehen die zuletzt gelesenen Werte da. Hat dein Zielsystem keine Chipbeschreibung, steht dort „Für dieses Zielsystem gibt es keine Chipbeschreibung.“

## Der Speicher

Unter dem Disassembler zeigt der **Speicher** den Arbeitsspeicher des Rechners. Die Überschrift nennt die Anfangsadresse („Speicher ab $00FB“). Jede Zeile hat die Adresse, 16 Bytes in Hex und rechts dieselben Bytes als Zeichen.

![Oben der Disassembler mit Adressen, Bytes und Befehlen, darunter die Karte „Speicher ab $0400“ mit einem Hexdump: links die Adressen von 0400 bis 04F0, in der Mitte 16 Bytes je Zeile, alle 00, rechts die Zeichen als Punkte.](bild:debug-speicher "Beispielansicht: Adresse, 16 Bytes in Hex und rechts dieselben Bytes als Zeichen.")

1. Gib im Feld **Adresse oder Symbol** eine Hex-Zahl ein, zum Beispiel `00FB`, oder einen Namen aus deinem Programm (`ZAEHLER`), und drücke Enter (oder klicke auf **Anzeigen**).
2. Führe einen Schritt aus oder drücke **Weiter** bis zum nächsten Halt: Bytes, die sich *seit dem vorherigen Halt* geändert haben, sind farbig und fett. So siehst du auf einen Blick, was dein Programm verändert hat.

Eine Schaltfläche zum Blättern und einen Rechtsklick auf den Zeilen gibt es nicht; gib einfach die nächste Adresse ein. Den Inhalt *deiner Programmdatei* (nicht des laufenden Rechners) siehst du weiter unten in der Hex-Ansicht, die Belegung deines gebauten Programms in der Speicherkarte.

## Beobachten

Im Abschnitt **Beobachten** legst du **Ausdrücke** an, die VintageCoder bei *jedem* Halt neu ausrechnet. So musst du nicht ständig in den Speicher schauen.

1. Tippe einen Ausdruck in das Feld **Ausdruck, Adresse oder Symbol**, zum Beispiel `a`, `x + 1`, `[$d020]` oder `[ZAEHLER]`.
2. Drücke Enter oder klicke auf **Hinzufügen**.
3. Schritt für Schritt (F11) siehst du, wie sich der Wert ändert. Er steht hexadezimal und dezimal da, zum Beispiel `$05 · 5`.

![Schmale Seitenspalte der Debug-Ansicht mit den Bereichen „Haltepunkte“, „Beobachten“, „Momentaufnahme“ und „Chips“. Unter „Beobachten“ stehen drei Ausdrücke a, x und $d020, jeweils mit dem Wert „$00 · 0“ und einem Knopf zum Entfernen, darunter ein Eingabefeld und der Knopf „Hinzufügen“.](bild:debug-beobachten "Beispielansicht, es läuft kein Emulator: der Bereich Beobachten mit drei Ausdrücken und ihren Werten in hexadezimaler und dezimaler Schreibweise.")

Ausdrücke kannst du auch vor dem Start der Sitzung anlegen; sie gelten dann beim nächsten Start. Solange nichts zu zeigen ist, steht „…“ dort. In Ausdrücken sind erlaubt:

| Was | Schreibweise |
|---|---|
| Register | `a`, `x`, `y`, `sp`, `pc`, `p` |
| Flags | `fn`, `fv`, `fb`, `fd`, `fi`, `fz`, `fc` |
| Speicher | `[$d020]` oder `peek($d020)`; ein Wort mit `peekw($fffe)` |
| Rasterposition | `lin` (Zeile), `cyc` (Zyklus) |
| Namen aus deinem Programm | zum Beispiel `[ZAEHLER]` (Marken und Konstanten der letzten Übersetzung) |

Operatoren gelten wie in C (`==`, `!=`, `<`, `>`, `&&`, `||`, `&`, `|`). Vergleiche binden stärker als `&` und `|`; im Zweifel setze Klammern, zum Beispiel `(x & 15) == 5 && fc`. Ist ein Ausdruck ungültig, steht an seiner Stelle die Fehlermeldung. Mit dem **✕** neben dem Ausdruck entfernst du ihn.

## Momentaufnahme

Du hast ein Programm, das etwas Schönes auf den Bildschirm bringt, und möchtest daraus einen Zeichensatz oder Sprites in dein Projekt übernehmen? Die **Momentaufnahme** (links im Bereich Debug) liest aus dem angehaltenen Rechner den aktuellen Bildschirm, den Zeichensatz und die aktiven Sprites und legt sie als Dateien im Projekt ab (unter `graphics/snapshot/`). Das geht für den C64 und den Atari 8-Bit.

1. Halte das Programm an der Stelle an, die du festhalten willst (Haltepunkt oder **Anhalten**). Läuft das Programm, steht dort „Die Momentaufnahme geht nur, wenn das Programm angehalten ist.“
2. Klicke auf **Erkennen, was im Speicher liegt**. VintageCoder erkennt aus den Hardware-Registern, wo Bildschirm und Zeichensatz liegen, und zeigt Modus, Bildschirmadresse, Zeichensatz (Adresse, im ROM oder unbekannt, gegebenenfalls „mehrfarbig“) und die Zahl der Sprites an. Hinweise zur Erkennung stehen darunter.
3. Schalte ein, was gespeichert werden soll: **Sprites übernehmen** und **Kacheln aus dem Bildschirm bilden**. Liegt der Zeichensatz im ROM, gibt es außerdem **Zeichensatz aus dem ROM holen**.
4. Vergib einen **Namen** (nur Buchstaben, Ziffern, `_` und `-`) und klicke **Als Dateien sichern**.

*[Bild folgt in E13: debug-momentaufnahme]*

Es entstehen je nach Auswahl ein Bildschirm (`.vcscreen`), ein Zeichensatz (`.vcchar`), Sprites (`.vcsprite`) und ein Kachelsatz (`_tiles.vctiles`). Nicht möglich sind Bitmap-Modi und ANTIC-Modi außer Modus 2.

> **Info:** Eine Vorschau und das Ändern der erkannten Adressen gibt es nicht.

> **Achtung:** Die Standardschriften der Rechner gehören den Herstellern. Checke einen so gespeicherten Zeichensatz nicht in ein öffentliches Repository ein.

## Das Bild des Emulators

Rechts unter dem Stapel zeigt der Abschnitt **Bildschirm** das **Bild des Emulators direkt in VintageCoder**. Das geht für VICE-Systeme (C64, C128, Plus/4, VC-20), nicht für Atari und MEGA65; dort steht „Dieser Emulator liefert kein Bild.“ Ein kleineres Einzelbild zeigt auch das Panel **Maschine**.

*[Bild folgt in E13: live-bild]*

- **Bild aktualisieren** holt das Bild sofort. Solange noch keines da ist, steht „Noch kein Bild. „Bild aktualisieren“ holt eines.“
- **Bei jedem Halt aktualisieren**: Mit diesem Schalter holt VintageCoder das Bild bei jedem Halt (Haltepunkt, Einzelschritt, Pause) selbst. Du siehst also genau den Zustand, in dem die Maschine steht.
- **Live (Zeitlupe)**: VintageCoder hält die Maschine alle paar Millisekunden kurz an, holt das Bild und setzt fort. Die Maschine läuft dabei mit etwa 40 bis 80 Prozent ihres Tempos und ohne Ton. Zum Spielen ist das nicht gedacht, zum Beobachten von Änderungen reicht es.
- **Bildquelle**: Gibt es mehrere Quellen (beim C128 **VIC-II** mit 40 Spalten und **VDC** mit 80 Spalten), wählst du sie in einer Auswahlliste.
- **Text an den Emulator senden** schreibt Text in den Tastaturpuffer des Emulators, zum Beispiel `RUN`. Enter oder **Senden** schickt ihn ab. Mehr als Text wird nicht weitergegeben: Tastatur und Joystick gehen nicht zum Emulator.

> **Info:** Das Bild als PNG speichern, in die Zwischenablage kopieren oder als Projektbild setzen kann die Oberfläche nicht.

## Einen Test im Debugger ansehen

Schlägt ein Test fehl, kannst du ihn im Debugger ansehen: Wähle im Panel **Tests** den Test aus und klicke auf **Diesen Test debuggen** (siehe [Automatisch testen](kapitel:tests)). VintageCoder bereitet den Test selbst vor (ohne neu zu bauen) und öffnet eine Sitzung, die am Anfang des Tests angehalten ist. In der Werkzeugleiste steht dann zusätzlich **Test-Sitzung**.

## Die Speicherkarte

Die **Speicherkarte** hat mit dem laufenden Programm nichts zu tun, sie zeigt dir den Speicher *deines gebauten Programms*. Du findest sie unter **Daten ▸ Speicher**. Wichtig: Sie arbeitet mit dem Ergebnis des **letzten Baus**. Ohne Bau steht dort „Noch nichts gebaut. Die Karte entsteht aus dem letzten Bau. Erst bauen (F7).“

![Der Bereich Daten mit der Seite Speicher: links die Bereiche (Zeropage, Stack, BASIC, ROM, E/A, frei), in der Mitte eine Matrix aus 16 mal 16 Kästchen, nach Art gefärbt, rechts die Angaben zur gewählten Seite $08 mit Prüfungen und die Auswahl des Prozessorports.](bild:memory-map "Beispielansicht: die Speicherkarte mit 256 Seiten, nach Art gefärbt, und den Angaben zur gewählten Seite.")

- Die **Karte** in der Mitte besteht aus 16 mal 16 Kästchen, jedes steht für eine Seite von 256 Byte. Links neben jeder Reihe steht ihre Anfangsadresse (`$0000`, `$1000` bis `$F000`). Die Farben zeigen den Zweck (zum Beispiel Zeropage, Stapel, Bildschirm, Sprites, Zeichensatz, Musik, E/A, ROM, BASIC, frei); ein Balken im Kästchen zeigt, wie viel davon dein Programm belegt.
- **Filter** über der Karte: **Alle**, **Programm**, **RAM**, **ROM**, **E/A**. Alles, was nicht passt, wird blass.
- Mit den **Pfeiltasten** wählst du eine Seite; ein Klick geht auch.
- Links stehen die **Bereiche** (ein Klick springt zur ersten Seite dieser Art) und das **Programm**: wie viele Byte es belegt und wie viel RAM frei ist.
- Rechts zeigt **Gewählte Seite** Adresse, Art des Speichers, wie viele Bytes dein Programm dort belegt, die Symbole auf der Seite und die Quellzeilen (ein Klick springt in den Editor).
- Unter **Prüfungen** stehen Hinweise mit einem Zeichen für die Schwere (✓ in Ordnung, · Information, ⚠ Warnung, ✗ Fehler): Überlappungen, ob der Stapel frei ist, wie viel Zeropage dein Programm belegt.
- Beim C64, C128 und Atari 800 XL/XE schaltet die **Bankumschaltung** unten rechts die Sicht um (zum Beispiel beim C64 „Standard (BASIC, E/A, KERNAL)“ oder „Alles RAM“). Über der Liste steht das Register mit seiner Adresse.

Das Nachschlagewerk beschreibt alles Weitere: [Memory Map](hilfe:memmap).

## Die Hex-Ansicht

Die **Hex-Ansicht** (**Daten ▸ Hex**) zeigt dir den Inhalt einer Binärdatei deines Projekts Byte für Byte, zum Beispiel ein gebautes Programm. Sie ist **nur zum Lesen**; sie verändert nie eine Datei.

![Der Bereich „Daten“ mit der Seite „Hex“. Links die Liste „BINÄRDATEIEN“ mit jumprun.prg gewählt, in der Mitte ein Hexdump der Datei mit Adressen, Bytewerten und Textspalte, darüber ein Suchfeld, „16 Byte“, „8 Byte“ und „Nur lesen“. Rechts die Karte „AUSWAHL“ mit Dateiname, Größe, Ladeadresse $0801 und Seitenwahl 1 / 5.](bild:hex-ansicht "Die Hex-Ansicht zeigt eine Binärdatei Byte für Byte, nur zum Lesen.")

1. Wähle links unter **Binärdateien** eine Datei. Gibt es keine, steht dort „Keine Binärdateien im Projekt.“ Im Dateibaum öffnest du eine Datei auch über den Eintrag **Als Hex ansehen (nur Lesen)**.
2. In der Mitte stehen Adresse (ADR), die Bytes und rechts der Text. Ein Klick auf ein Byte wählt es, mit gedrückter Umschalttaste oder durch Ziehen wählst du mehrere. Auch die Pfeiltasten, Bild auf, Bild ab, Pos1 und Ende wirken.
3. Rechts zeigt **Auswahl** den gewählten Bereich und **Gedeutet als** das erste Byte (dezimal, hexadezimal, binär), das Wort aus zwei Bytes (kleines Byte vorn, wie beim 6502) und den Text. Bei Programmdateien steht außerdem die **Ladeadresse** und zur Auswahl die Speicheradresse, an der das Byte nach dem Laden liegt.
4. Im Feld **Suchen** gibst du Bytes (`$A9 00`) oder Text ein; **Suchen** springt zum nächsten Treffer, oder meldet „Nichts gefunden.“
5. Mit **16 Byte** oder **8 Byte** stellst du ein, wie viele Bytes in einer Zeile stehen. Große Dateien liest die Ansicht seitenweise (4 KB je Seite) und höchstens die ersten 4 MiB; die Knöpfe unter **Seitenwahl** blättern.
6. **Im Standardprogramm öffnen** öffnet die Datei in dem Programm, das dein Betriebssystem dafür vorsieht.

## Das Live-Bild (Ultimate 64)

> **Info:** Ein Live-Bild des Bildschirms einer Ultimate 64 gibt es in dieser Version nicht. Starten, Speicher lesen und Tests über das Netzwerk sind möglich (siehe [Zielsysteme im Vergleich](kapitel:systeme)).

## Häufige Stolpersteine

| Problem | Ursache und Lösung |
|---|---|
| Nach F6 passiert nichts | Der Bau ist fehlgeschlagen: Schau in den Reiter **Probleme**. Oder es läuft schon eine Debug-Sitzung: beende sie mit **Debuggen beenden** (Umschalt+F5). F6 bringt dich sonst in den Bereich Debug zurück. |
| Register, Disassembler und Speicher sind leer | Der Emulator läuft. Drücke **Debuggen: Anhalten** (Alt+F6) oder setze einen Haltepunkt. |
| Der Haltepunkt wird nie getroffen | Die Zeile wird nicht erreicht, oder du hast den Quelltext nach dem Bau geändert: neu bauen. Bei atari800 gilt: nur ein Haltepunkt gleichzeitig. |
| Der Einzelschritt geht nicht | Der Emulator unterstützt den Schritt in dieser Lage nicht; der Hinweis unter der Werkzeugleiste nennt den Grund. |
| Ein Ausdruck zeigt „unbekannt“ | Prüfe die Schreibweise und die Namen. Die Fehlermeldung steht darunter. Klammere bei `&` und `|`. |
| **Hinaus** bricht nach zehn Sekunden ab | Es gab keinen Rücksprung. Setze einen Haltepunkt auf das `RTS` des Unterprogramms. |
| Der Reiter Bild zeigt kein Bild | Dein System liefert keines (Atari, MEGA65), oder es wurde noch keines geholt: **Bild aktualisieren**. |

## Wie geht es weiter?

- Prüfe dein Programm automatisch statt von Hand: [Automatisch testen](kapitel:tests).
- Alles über die Besonderheiten deines Zielsystems: [Zielsysteme im Vergleich](kapitel:systeme).
- Die Belegung deines Programms und die Hex-Ansicht stehen oben in diesem Kapitel; mehr zu den Bereichen findest du in [Die Oberfläche](kapitel:oberflaeche).
- Alle Tasten auf einen Blick: [Tastenkürzel, FAQ, Fehlersuche](kapitel:kuerzel-faq).
- Zum Nachschlagen: [Debugger](hilfe:debugger), [Haltepunkte](hilfe:breakpoints), [Memory Map](hilfe:memmap).
