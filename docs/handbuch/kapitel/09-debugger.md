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

VintageCoder hat zwei **Arbeitsplätze**: *Code* zum Schreiben und *Debug* zum Untersuchen. Zwischen beiden schaltest du mit dem Umschalter **Code | Debug** rechts in der Befehlsleiste oder über das Menü **Ansicht ▸ Arbeitsplatz Code** bzw. **Arbeitsplatz Debug**.

Normalerweise musst du aber gar nicht von Hand umschalten:

1. Drücke **F6** (oder klicke in der Befehlsleiste auf **Debuggen**, oder wähle **Projekt ▸ Startprogramm debuggen**).
2. VintageCoder baut das Projekt, startet den Emulator und hält das Programm **am Einstiegspunkt** an.
3. Die Oberfläche wechselt selbst auf den Arbeitsplatz **Debug**.

Mit **Umschalt+F6** (Menü **Projekt ▸ Aktive Datei debuggen**) debuggst du statt des Startprogramms die Datei, die gerade im Editor aktiv ist.

![Der Debugger-Modus: Editor in der Mitte, rechts Register, Beobachten und Haltepunkte, unten Disassembler und Speicher](bild:debug-ueberblick "Der Arbeitsplatz Debug nach F6. ①: Befehlsleiste mit Weiter, Schritt, Über, Raus und Stop. ②: der Editor mit der aktuellen Zeile. ③: Inspektor rechts (Register, Beobachten, Haltepunkte). ④: Fußpanel unten (Disassembler, Speicher, Emulator, Momentaufnahme).")

Das Bild zeigt die Aufteilung (die Nummern sind die Marken im Bild):

- **①** Die **Befehlsleiste** oben. Sie zeigt, ob der Emulator gerade läuft oder angehalten ist (zum Beispiel `● angehalten · $080D`), und die Knöpfe zur Steuerung. Statt *Starten* und *Debuggen* stehen jetzt *Pause* oder *Weiter*, *Schritt*, *Über*, *Raus* und *Stop*. Das Zielfeld „Auf:“ bleibt sichtbar, ist aber gesperrt.
- **②** Der **Editor** in der Mitte, mit deinem Quelltext. Die Zeile, an der das Programm steht, ist cyanfarben markiert.
- **③** Der **Inspektor** rechts mit den Reitern **Register**, **Beobachten**, **Haltepunkte** und **Hilfe**.
- **④** Das **Fußpanel** unten mit den Reitern **Disassembler**, **Speicher**, **Emulator**, **Momentaufnahme**, **Live-Bild**, **Ausgabe** und **Probleme**.

> **Tipp:** Fehlt dir eines der Panels, blendest du es mit **Strg/Cmd+B** (Seitenleiste), **Strg/Cmd+Alt+B** (Inspektor) oder **Strg/Cmd+J** (Fußpanel) ein.

> **Info:** Während der Emulator läuft, zeigen Register, Disassembler und Speicher nichts an: Der Zustand der Maschine ist nur im angehaltenen Zustand lesbar. Du siehst dann den Hinweis „Der Emulator läuft. Anhalten mit „Pause“ oder durch einen Haltepunkt.“

### Was der Emulator kann

Nicht jeder Emulator kann alles. Unter dem Status im Reiter **Register** zeigen kleine Schilder (Chips), was dein Emulator beherrscht: *Register*, *Einzelschritt*, *Haltepunkte*, *Datenhaltepunkte*, *Pausieren* und *mehrere Haltepunkte*. Was fehlt, ist gedimmt; im Hinweis steht der Grund. Die Übersicht:

| Emulator | Register | Einzelschritt | Haltepunkte | Datenhaltepunkte |
|---|---|---|---|---|
| VICE (C64, C128, Plus/4, VC-20) | ja | ja | mehrere | ja |
| atari800 (Atari 8-Bit) | ja | ja | nur einer zugleich | nein |
| MEGA65 (Xemu oder Hardware) | ja | ja | einer | nur als langsame Schreib-Beobachtung per Einzelschritt |
| Ultimate 64 / II+ (Netzwerk) | nein | nein | nein | nein |

Bei der Ultimate kannst du Speicher lesen und schreiben sowie pausieren und fortsetzen, mehr nicht. atari800 lässt sich nicht per Knopf anhalten: Dort drückst du **F8 im Emulatorfenster** oder setzt vorher einen Haltepunkt.

## Haltepunkte

Ein **Haltepunkt** sagt dem Emulator: „Halte an, bevor du diese Zeile ausführst.“ Es gibt drei Arten: Haltepunkte auf eine Zeile (oder Adresse), Haltepunkte mit **Bedingung** und **Datenhaltepunkte**, die auf Speicherzugriffe reagieren.

### Haltepunkt auf eine Zeile

1. Wechsle zum Editor und **klicke auf die Zeilennummer** von Zeile 18 (`inc ZAEHLER`). Links neben der Nummer erscheint ein roter Punkt.
2. Ein zweiter Klick auf die Zeilennummer entfernt den Haltepunkt wieder. Alternativ geht es über das Kontextmenü (rechte Maustaste im Editor): **Haltepunkt setzen (Zeile 18)**.
3. Drücke **F6**. Das Programm startet und hält an, sobald es Zeile 18 erreicht. Die Zeile bekommt den cyanfarbenen Streifen; die Befehlsleiste zeigt `● angehalten · $0817`.

![Roter Haltepunkt-Punkt neben der Zeilennummer 18](bild:debug-haltepunkt-gutter "Der rote Punkt links neben der Zeilennummer ist ein Haltepunkt (①).")

![Die angehaltene Zeile 18 mit cyanfarbenem Streifen](bild:debug-pc-zeile "Der helle Streifen (①) zeigt die Zeile, an der das Programm steht. Der Befehl in dieser Zeile ist noch nicht ausgeführt.")

> **Info:** Haltepunkte im Editor kannst du schon vor dem Start setzen. Die IDE ordnet sie über die Zeilenzuordnung des letzten Baus einer Adresse zu. Hast du den Quelltext seit dem Bauen geändert, baue neu (F7 oder einfach F6, das baut mit).

Das Panel **Haltepunkte** (Reiter im Inspektor) listet alle Punkte mit Art, Adresse, Bedingung, „ignorieren“ und Trefferzähler (`3×`). Am Punkt, der gerade angehalten hat, steht „getroffen“. Ein Klick auf das kleine **x** entfernt einen Punkt; mit der rechten Maustaste öffnest du ein Menü mit **Entfernen**, **Bearbeiten**, **Speicher ab … anzeigen**, **Zur Quellzeile** und **Adresse kopieren**.

Weitere Haltepunkte legst du im Bereich **Neu** des Panels an: Art wählen, Adresse (`$0813`) oder ein Name aus deinem Programm (`naechste_farbe`) bei „von $“ eintragen, **Anlegen** klicken.

### Bedingte Haltepunkte

Die Schleife läuft tausende Male pro Sekunde; bei jedem Durchlauf anzuhalten hilft nicht. Ein **bedingter Haltepunkt** hält nur an, wenn ein Ausdruck stimmt.

1. Öffne im Inspektor den Reiter **Haltepunkte**. Einen Haltepunkt, den du vorher in Zeile 18 gesetzt hast, entfernst du dort mit dem **x**.
2. Im Bereich **Neu**: Art **Ausführung**, bei „von $“ `naechste_farbe`, bei „Bedingung“ den Ausdruck `[$fb] == 9`.
3. Klicke **Anlegen** und dann (im Debugger angehalten) **Weiter**.

Das Programm läuft nun, bis der Zähler an Speicherstelle `$FB` den Wert 9 hat, und hält dann am Anfang von `naechste_farbe` an.

![Bedingter Haltepunkt im Panel Haltepunkte](bild:debug-bedingung "Ein bedingter Haltepunkt im Panel Haltepunkte: Art „Ausführung“, die Adresse als Name, darunter die Bedingung. In der Liste steht „wenn …“ mit dem Trefferzähler.")

In Bedingungen sind erlaubt:

| Was | Schreibweise |
|---|---|
| Register | `a`, `x`, `y`, `sp`, `pc`, `p` |
| Flags | `fn`, `fv`, `fb`, `fd`, `fi`, `fz`, `fc` |
| Speicher | `[$d020]` oder `peek($d020)`; ein Wort mit `peekw($fffe)` |
| Rasterposition | `lin` (Zeile), `cyc` (Zyklus) |
| Namen aus deinem Programm | zum Beispiel `[ZAEHLER]` (Marken und Konstanten der letzten Übersetzung) |

Operatoren gelten wie in C (`==`, `!=`, `<`, `>`, `&&`, `||`, `&`, `|`). Vergleiche binden stärker als `&` und `|`; im Zweifel setze Klammern, zum Beispiel `(x & 15) == 5 && fc`. Ein Syntaxfehler wird sofort gemeldet. Ein Fehler beim Auswerten (zum Beispiel ein unbekannter Name) hält an.

Das Feld **Ignorieren** überspringt die ersten *N* Treffer: Mit `100` hält der Punkt erst beim 101. Durchlauf an.

> **Achtung:** Die IDE wertet die Bedingung bei jedem Halt aus und setzt das Programm fort, wenn sie nicht stimmt. An sehr oft durchlaufenen Stellen kostet das spürbar Zeit. Das Programm läuft dadurch langsamer, als du es gewohnt bist.

### Datenhaltepunkte

Manchmal weißt du nicht, *wo* etwas passiert, nur *was*: „Wer verändert eigentlich `$FB`?“ Dafür gibt es **Datenhaltepunkte**. Sie halten an, wenn auf eine Speicherstelle zugegriffen wird.

1. Im Bereich **Neu** wählst du bei der Art **Schreiben**, **Lesen** oder **Zugriff** (Lesen und Schreiben).
2. Bei „von $“ trägst du die Adresse ein, zum Beispiel `00FB`. Für einen Bereich füllst du zusätzlich „bis $“ aus.
3. **Anlegen**, dann **Weiter**.

Der Emulator hält **hinter** dem Befehl an, der zugegriffen hat. Bei einem Schreibzugriff auf `$FB` landet der Programmzähler also auf dem Befehl nach `inc ZAEHLER` (`lda ZAEHLER`).

Schneller geht es aus dem Reiter **Speicher**: Klicke mit der rechten Maustaste auf eine Zeile und wähle **Haltepunkt: Schreiben auf $…** oder **Haltepunkt: Lesen von $…** (der Punkt gilt für den Zeilenanfang).

![Ein Datenhaltepunkt auf $00FB](bild:debug-datenhaltepunkt "Ein Datenhaltepunkt auf Schreibzugriffe. „getroffen“ in der Liste zeigt, dass er angehalten hat.")

> **Achtung:** Datenhaltepunkte gibt es nur mit VICE. In der Auswahl der Art sind *Lesen*, *Schreiben* und *Zugriff* bei anderen Emulatoren ausgegraut.

## Einzelschritt

Ist das Programm angehalten, steuerst du es mit diesen Knöpfen und Tasten (auch im Menü **Debug**):

| Knopf | Taste | Was passiert |
|---|---|---|
| **Weiter** | F8 | läuft bis zum nächsten Haltepunkt |
| **Schritt** | F11 | führt genau einen Befehl aus; ein `JSR` springt *hinein* in das Unterprogramm |
| **Über** | F10 | führt einen Befehl aus; ein `JSR` läuft *ganz durch* und hält dahinter an |
| **Raus** | Umschalt+F11 | läuft bis zum Rücksprung (`RTS` oder `RTI`) des aktuellen Unterprogramms |
| **Pause** | – | hält den laufenden Emulator an (bei atari800: F8 im Emulatorfenster) |
| **Stop** | – | beendet die Debug-Sitzung (Menü **Debug ▸ Debug-Sitzung beenden**) |

> **Info:** F8, F10 und F11 wirken nur, wenn das Programm angehalten ist. F6 startet den Debugger nur, wenn gerade keine Sitzung läuft.

**Probiere es aus.** Entferne vorher alle Haltepunkte und setze einen auf Zeile 13 (`jsr naechste_farbe`), wenn die Sitzung neu gestartet wird:

1. Drücke **F6**. Das Programm hält in Zeile 13 an.
2. Drücke **F11** (Schritt). Du bist *in* `naechste_farbe` gelandet, auf `inc ZAEHLER` (Zeile 18).
3. Drücke noch viermal **F11**: `inc`, `lda`, `and` und `sta` werden ausgeführt, dann steht der Programmzähler auf dem `rts`. Beobachte nach jedem Schritt die Register (siehe unten) und den Rand des Emulatorfensters.
4. Drücke **Umschalt+F11** (Raus). Das `rts` wird ausgeführt, und das Programm hält hinter dem Aufruf an, in Zeile 14 (`jmp schleife`).
5. Drücke **F10** (Über) zweimal. Jetzt hält der Debugger in Zeile 13 wieder an, dann steht er hinter dem `jsr`. **Über** hat das ganze Unterprogramm in einem Rutsch ausgeführt.

**Raus** zählt mit: Aufrufe innerhalb des Unterprogramms werden übersprungen, sonst führt die IDE Einzelschritte aus. Nach zehn Sekunden ohne Rücksprung bricht sie mit einer Meldung ab. Setze dann einen Haltepunkt direkt am Rücksprung.

> **Tipp:** Willst du nur bis zu einer bestimmten Stelle laufen, ohne einen dauerhaften Haltepunkt zu setzen: Im Disassembler gibt es im Kontextmenü (rechte Maustaste auf die Zeile) **Bis hierher ausführen**. Das ist ein einmaliger Haltepunkt, der nach dem nächsten Halt verschwindet.

Läuft der Einzelschritt aus einem Grund nicht (zum Beispiel weil der Emulator es nicht kann), sind die Schritt-Knöpfe gesperrt, und der Hinweis „Einzelschritt gesperrt: …“ nennt den Grund.

## Die Register

Der Reiter **Register** im Inspektor zeigt den Zustand der CPU:

![Der Reiter Register: PC, A, X, Y, SP und die Flag-Leiste](bild:debug-register "①: der Programmzähler PC. ②: Akkumulator A und die Indexregister X und Y. ③: der Stapelzeiger SP. ④: die Flags N V B D I Z C. ⑤: das Feld Chip-Register.")

- **①** **PC**: Adresse des nächsten Befehls (der noch *nicht* ausgeführt ist).
- **②** **A**, **X**, **Y**: die Rechenregister.
- **③** **SP**: der Stapelzeiger (der Stapel liegt auf Seite 1, also ab `$0100`).
- **④** Die **Flags** N V B D I Z C als Schalter-Leiste. Ein Flag, das gesetzt ist, ist hervorgehoben. Die Leiste ist reine Anzeige; ein Klick ändert nichts im Emulator.
- Bei VICE kommen **RL** und **CYC** (Rasterzeile und Zyklus) hinzu.

Ein Wert, der sich bei einem Schritt verändert hat, ist leicht zu erkennen, wenn du den Schritt wiederholst und die Anzeige vergleichst. Die Zahlen stehen in Hex.

**Rechtsklick auf ein Register** öffnet ein Menü: **Wert kopieren (hex)** oder **(dezimal)**, **Beobachten: a** (nimmt es in die Liste der beobachteten Ausdrücke auf) und bei PC und SP **Speicher ab $… anzeigen**.

### Chip-Register

Unter den Flags steht das Feld **Chip-Register**. Es zeigt die Register eines Chips deines Zielsystems, zum Beispiel `vic2`, `sid`, `cia1` beim C64 oder `antic` und `pokey` beim Atari. Den Chip wählst du in der Auswahlliste. Du siehst Adresse, Name und Wert in Hex; die Beschreibung des Namens erscheint, wenn du mit der Maus darauf bleibst.

Im Beispiel: Wähle `vic2` und suche `border`. Während das Programm steht, zeigt der Wert den aktuellen Rand. Schreib-Register, deren Wert sich nicht lesen lässt, zeigen „–“. Die Anzeige ist bewusst nur lesend, damit Lesezugriffe nichts verfälschen.

## Der Disassembler

Der Reiter **Disassembler** im Fußpanel zeigt den **Maschinencode ab dem Programmzähler**, so wie die CPU ihn sieht: Adresse, Bytes, Befehl und die Zahl der Zyklen.

![Der Disassembler mit Programmzähler-Zeile und Haltepunkt](bild:debug-disassembler "Der Disassembler. ①: die Zeile, an der das Programm steht. ②: ein roter Punkt für einen Haltepunkt. ③: die Spaltenköpfe Adresse, Bytes und Befehl. Marken aus deinem Quelltext stehen vor dem Befehl.")

Praktisch daran:

- **Klick auf eine Zeile** setzt dort einen Haltepunkt oder entfernt ihn.
- Stehen Marken aus deinem Quelltext an einer Adresse, siehst du sie vor dem Befehl (`naechste_farbe:`).
- Befehle, die gar nicht zum offiziellen Befehlssatz gehören, sind mit „undok.“ gekennzeichnet.
- Das Kontextmenü (rechte Maustaste) bietet: **Haltepunkt setzen/entfernen** (auch an Adressen ohne Quelltext), **Bis hierher ausführen**, **Speicher ab … anzeigen**, **Beobachten: [$…]**, **Zur Quellzeile** (springt in den Editor), **Adresse kopieren** und **Zeile kopieren**.

## Der Speicher

Im Reiter **Speicher** liest du den Arbeitsspeicher des laufenden Rechners:

![Der Reiter Speicher mit Hexdump](bild:debug-speicher "Der Reiter Speicher: oben das Adressfeld mit „Lesen“ sowie −256 und +256, darunter der Hexdump mit 16 Bytes je Zeile und rechts die Zeichen. Geänderte Bytes sind farbig hinterlegt.")

1. Gib bei **Adresse $** eine Hex-Zahl ein, zum Beispiel `00FB`, und klicke **Lesen** (oder drücke Enter).
2. Mit **−256** und **+256** blätterst du seitenweise.
3. Führe einen Schritt aus oder drücke **Weiter** bis zum nächsten Halt: Bytes, die sich *seit dem vorherigen Halt* geändert haben, sind farbig hinterlegt. So siehst du auf einen Blick, was dein Programm verändert hat.

Mit der rechten Maustaste auf eine Zeile kannst du sie **beobachten**, einen **Schreib-** oder **Lese-Haltepunkt** auf den Zeilenanfang setzen oder Adresse und Zeile kopieren.

Die Hilfe-Seite zum Debugger beschreibt außerdem das Einklappfeld **Bereich als Zeichensatz speichern**. Damit liest du einen Zeichensatz aus dem Speicher des Rechners und legst ihn als Asset im Projekt an, siehe [Zeichensatz-Editor](kapitel:zeichensatz).

Ein schneller Weg zum Speicher: Menü **Debug ▸ Speicher anzeigen**.

## Beobachten

Im Reiter **Beobachten** (Inspektor) legst du **Ausdrücke** an, die die IDE bei *jedem* Halt neu ausrechnet. So musst du nicht ständig in den Speicher schauen.

1. Tippe einen Ausdruck in das Feld, zum Beispiel `a`, `x + 1`, `[$d020]` oder `[ZAEHLER]`.
2. Drücke Enter oder klicke **Hinzufügen**.
3. Schritt für Schritt (F11) siehst du, wie sich der Wert ändert. Er steht hexadezimal und dezimal da, zum Beispiel `$5  (5)`.

![Der Reiter Beobachten mit drei Ausdrücken](bild:debug-beobachten "Der Reiter Beobachten: oben das Eingabefeld, darunter die Ausdrücke mit ihrem Wert in Hex und dezimal. Das x entfernt einen Ausdruck.")

Ausdrücke können Register, Flags, Namen und `[adresse]` enthalten, wie bei den Bedingungen der Haltepunkte. Ist ein Ausdruck ungültig, steht an seiner Stelle die Fehlermeldung in rot. Mit dem **x** neben dem Ausdruck entfernst du ihn; das Kontextmenü bietet **Wert kopieren**, **Ausdruck kopieren**, **Ausdruck entfernen** und **Alle entfernen**.

> **Tipp:** Aus dem Disassembler, dem Speicher und den Registern nimmst du Ausdrücke über **Beobachten …** im Kontextmenü auf, ohne zu tippen.

## Momentaufnahme

Du hast ein Programm, das etwas Schönes auf den Bildschirm bringt, und möchtest daraus einen Zeichensatz oder Sprites in dein Projekt übernehmen? Die **Momentaufnahme** (Reiter im Fußpanel) liest aus dem angehaltenen Rechner den aktuellen Bildschirm, den Zeichensatz und die aktiven Sprites und legt sie als Dateien im Projekt ab (unter `graphics/snapshot/`). Das geht für den C64 und den Atari 8-Bit.

1. Halte das Programm an der Stelle an, die du festhalten willst (Haltepunkt oder **Pause**; im Reiter selbst gibt es dafür **Jetzt anhalten**).
2. Öffne den Reiter **Momentaufnahme**. Die IDE erkennt aus den Hardware-Registern, wo Bildschirm und Zeichensatz liegen. Links siehst du eine Vorschau.
3. Prüfe die **Adressen**. Sie lassen sich ändern, falls dein Programm den Bildschirm woanders zusammenbaut. **Neu erkennen** liest die Register erneut (und verwirft deine Änderungen).
4. Hake an, was gespeichert werden soll: **Bildschirm**, **Zeichensatz**, **Kacheln 2×2**, **Sprites**.
5. Vergib einen **Namen** (nur Buchstaben, Ziffern, `_` und `-`) und klicke **Speichern**.

![Der Reiter Momentaufnahme mit Vorschau und Adressen](bild:debug-momentaufnahme "Der Reiter Momentaufnahme (hier mit dem Beispiel sprite_demo): links die Vorschau, daneben die erkannten Adressen, die Auswahl, der Name und „Speichern“.")

Alles, was du speicherst, öffnet sich sofort in den passenden Editoren; vorhandene Dateien mit demselben Namen werden ersetzt. Nicht möglich sind Bitmap-Modi und ANTIC-Modi außer Modus 2.

> **Achtung:** Die Standardschriften der Rechner gehören den Herstellern. Checke einen so gespeicherten Zeichensatz nicht in ein öffentliches Repository ein.

## Die Memory Map

Die **Memory Map** hat mit dem laufenden Programm nichts zu tun, sie zeigt dir den Speicher *deines gebauten Programms*. Sie liegt im Inspektor des Arbeitsplatzes *Code* (Reiter **Memory Map**). Wichtig: Sie arbeitet mit dem Ergebnis der **letzten Übersetzung**. Ohne Bau steht dort „Bauen, um die Belegung zu sehen“.

![Memory Map: 16×16 Seiten, gefärbt nach Art](bild:memory-map "Die Memory Map: die Matrix aus 256 Seiten, gefärbt nach Art des Speichers, mit Ansicht, Filter und Segment-Inspektor.")

- Jede Kachel der **Matrix** ist eine Seite von 256 Byte. Die Farben zeigen RAM, ROM, E/A und deine Segmente.
- **Ansicht**: „256 Seiten“, „4-KB-Block“ oder „Byte“. **Filter**: Alle, Programm, RAM, ROM, E/A.
- Ein Klick auf eine Seite zeigt im **Segment-Inspektor** Adresse, Art des Speichers, das belegende Segment mit Quelle und die Symbole auf der Seite.
- Hinweise: Zahl der Überlappungen, ob der Stapel frei ist, wie viel Zeropage dein Programm belegt.
- Beim C64, C128 und Atari 800 XL/XE schalten die **Bank-Presets** die Sicht um (zum Beispiel beim C64 „Standard (BASIC, E/A, KERNAL)“ oder „Alles RAM“).
- Der zweite Reiter, die **Liste**, führt die Symbole der Systembibliothek auf.

Das Nachschlagewerk beschreibt alles Weitere: [Memory Map](hilfe:memmap).

## Der Emulator-Reiter

Der Reiter **Emulator** im Fußpanel (auch über **Debug ▸ Emulator-Bild anzeigen**) zeigt das **Bild des Emulators direkt in der IDE**. Das geht für VICE-Systeme (C64, C128, Plus/4, VC-20), nicht für Atari und MEGA65.

![Der Reiter Emulator mit dem Bild des C64 und einem weißen Rand](bild:emulator-reiter "Der Reiter Emulator: links das Bild der Maschine, darüber die Knöpfe (Bild holen, Zeitlupen-Live, Als PNG speichern …), rechts Text einspeisen und das Speicherfenster.")

- Bei jedem **Halt** (Haltepunkt, Einzelschritt, Pause) holt die IDE das Bild. Du siehst also genau den Zustand, in dem die Maschine steht. **Bild holen** holt es sofort.
- **Zeitlupen-Live**: Die IDE hält die Maschine alle paar Millisekunden kurz an, holt das Bild und setzt fort. Die Maschine läuft dabei mit etwa 40 bis 80 Prozent ihres Tempos und ohne Ton. Zum Spielen ist das nicht gedacht, zum Beobachten von Änderungen reicht es.
- **Text einspeisen** schreibt Text in den Tastaturpuffer des Emulators, zum Beispiel `RUN`. Zeilenumbruch ist RETURN. Mehr als Text wird nicht weitergegeben: Tastatur und Joystick gehen nicht zum Emulator.
- Beim C128 wählst du zwischen **VIC-II** (40 Spalten) und **VDC** (80 Spalten).
- **Als PNG speichern** legt das Bild im Projekt unter `aufnahmen/` ab, **In Zwischenablage** kopiert es, **Als Projektbild setzen** nimmt es als Titelbild für die Willkommensseite.
- Rechts daneben gibt es ein **Speicherfenster**, wie im Reiter Speicher.

## Das Live-Bild (Ultimate 64)

> **Info:** In Erprobung: Das Live-Bild ist bisher nur mit aufgezeichneten Testpaketen geprüft und noch nicht am echten Gerät erprobt (Ultimate-Firmware 3.14d). Rechne mit Abweichungen.

Der Reiter **Live-Bild** zeigt den Bildschirm einer **Ultimate 64** über das Netzwerk. Voraussetzung ist, dass du in den Einstellungen unter „Maschinen“ eine Ultimate 64 eingetragen hast. Du öffnest ihn im Fußpanel des Arbeitsplatzes Debug oder über **Projekt ▸ Live-Bild öffnen … (Ultimate 64)**.

![Der Reiter Live-Bild einer Ultimate 64](bild:live-bild "Der Reiter Live-Bild ohne eingerichtete Ultimate 64: Er weist auf die Einstellung „Maschinen“ hin.")

Die Ultimate schickt das Bild als UDP-Pakete an deinen Rechner. Dafür öffnet VintageCoder einen Empfangsanschluss im lokalen Netz; das System fragt beim ersten Mal womöglich nach der Erlaubnis. Auf der Ultimate wird nichts gespeichert und keine Einstellung verändert. Beim Schließen des Reiters stoppt VintageCoder den Strom wieder.

## Häufige Stolpersteine

| Problem | Ursache und Lösung |
|---|---|
| Nach F6 passiert nichts | Der Bau ist fehlgeschlagen: Schau in den Reiter **Probleme**. Oder es läuft schon eine Debug-Sitzung (F6 startet nur ohne Sitzung): beende sie mit **Stop**. |
| Register, Disassembler und Speicher sind leer | Der Emulator läuft. Drücke **Pause** oder setze einen Haltepunkt. |
| Der Haltepunkt wird nie getroffen | Die Zeile wird nicht erreicht, oder du hast den Quelltext nach dem Bau geändert: neu bauen. Bei atari800 gilt: nur ein Haltepunkt gleichzeitig. |
| Der Datenhaltepunkt ist ausgegraut | Dein Emulator kennt keine Datenhaltepunkte (nur VICE). |
| „Einzelschritt gesperrt“ | Der Emulator unterstützt den Schritt in dieser Lage nicht; der Hinweis nennt den Grund. |
| Eine Bedingung hält immer an | Prüfe die Schreibweise. Ein Laufzeitfehler (zum Beispiel ein unbekannter Name) hält an, statt zu ignorieren. Klammere bei `&` und `|`. |
| Das Programm läuft mit Bedingung sehr langsam | Die IDE wertet die Bedingung bei jedem Halt aus. Wähle eine Stelle, die seltener durchlaufen wird, oder nimm „ignorieren N“. |
| Der Emulator-Reiter zeigt kein Bild | Dein System liefert keines (Atari, MEGA65), oder die Maschine läuft: „Bild holen“. |
| **Raus** bricht nach zehn Sekunden ab | Es gab keinen Rücksprung. Setze einen Haltepunkt auf das `RTS` des Unterprogramms. |

## Wie geht es weiter?

- Prüfe dein Programm automatisch statt von Hand: [Automatisch testen](kapitel:tests).
- Alles über die Besonderheiten deines Zielsystems: [Zielsysteme im Vergleich](kapitel:systeme).
- Alle Tasten auf einen Blick: [Tastenkürzel, FAQ, Fehlersuche](kapitel:kuerzel-faq).
- Zum Nachschlagen: [Debugger](hilfe:debugger), [Haltepunkte](hilfe:breakpoints), [Memory Map](hilfe:memmap).
