---
id: grafik-ueberblick
titel: Grafik und Klang im Überblick
reihenfolge: 11
stufe: Einsteiger
zeit: 15 Minuten
siehe_auch: [zeichensatz, sprites, bildschirm-kacheln, paletten, tracker]
referenz: [graphics, export]
---

# Grafik und Klang im Überblick

**Ziel:** Du weißt, welche Zeichen-Werkzeuge („Editoren“) VintageCoder hat, wie du eine Grafikdatei öffnest und anlegst und wie ihr Inhalt vom Editor in dein Programm kommt. Mit diesem Kapitel verstehst du den Weg, den alle folgenden Kapitel (12 bis 17) jeweils im Detail gehen.

## Das brauchst du

- ein geöffnetes Projekt (siehe [Dein erstes Projekt](kapitel:erstes-projekt)),
- für das letzte Beispiel den Emulator des Zielsystems.

## Was sind Assets?

Alles, was nicht Programmtext ist, nennen wir **Assets** (englisch für „Bestände“): ein Sprite, ein Zeichensatz, ein Bildschirm, eine Palette, ein Song. Jedes Asset liegt als **kleine Textdatei** in deinem Projekt, zum Beispiel `graphics/hero.vcsprite`. Der Inhalt sind Hex-Zeilen, die Pixel beschreiben. Das hat zwei Vorteile: Du kannst die Dateien mit Versionsverwaltung wie Git sinnvoll vergleichen, und der Editor und der Bau lesen dieselbe Quelle.

| Endung | Art | Editor | Kapitel |
|---|---|---|---|
| `.vcchar` | Zeichensatz | Zeichensatz-Editor | [12](kapitel:zeichensatz) |
| `.vcsprite` | Sprites (C64, MEGA65) | Sprite-Editor | [13](kapitel:sprites) |
| `.vcpmg` | Player/Missiles (Atari) | PMG-Editor | [13](kapitel:sprites) |
| `.vcscreen` | Textbildschirm oder Screen-Bank | Bildschirm-Editor | [14](kapitel:bildschirm-kacheln) |
| `.vctiles` | Kachelsatz | Kachel-Editor | [14](kapitel:bildschirm-kacheln) |
| `.vcpal` | Palette (MEGA65) | Paletten-Editor | [15](kapitel:paletten) |
| `.vcsong` | Song (SID, POKEY und andere) | Tracker (Hauptbereich **Musik**) | [16](kapitel:tracker) |
| `.vcplay` | PLAY-Song (MEGA65) | PLAY-Tracker (Hauptbereich **Musik**) | [17](kapitel:play-tracker) |

> **Info:** Jedes Asset gehört zu einem **Profil**. Das Profil legt fest, wie das Asset aufgebaut ist, etwa `c64.sprite.multicolor` (C64-Sprite, mehrfarbig, 12×21 Punkte) oder `atari.char.antic2` (Atari-Zeichensatz mit 128 Zeichen). Das Profil wählst du beim Anlegen.

Gebaut sind der Sprite-Editor (mit PMG), der Zeichensatz-Editor, der Paletten-Editor, der Kachel-Editor, der Bildschirm-Editor, der Karten-Editor und der Tracker (mit dem PLAY-Tracker).

## Schritt für Schritt: ein Asset anlegen und öffnen

1. Wähle in der Kopfzeile den Hauptbereich **Grafik**. In der zweiten Reihe stehen **Sprites**, **Zeichensatz**, **Kacheln**, **Bildschirm** und **Palette**. Songs bearbeitest du im Hauptbereich **Musik**.
2. Ist noch keine Datei offen, zeigt der Editor die passenden Dateien des Projekts als Knöpfe; ein Klick öffnet die Datei. Mit **Öffnen …** wählst du eine andere Datei aus dem Projekt. Ein **Doppelklick** auf die Datei im Projektbaum öffnet sie genauso im passenden Editor.
3. Eine neue Datei legst du im Editor selbst an: im Sprite-Editor mit **Neue Sprite-Datei …**, im Zeichensatz-Editor mit **Neuer Zeichensatz …**. Im Dialog wählst du unter **Art und System** das **Profil** (die passende Variante), gibst den **Namen** ein (Buchstaben, Ziffern, `_` und `-`), prüfst den **Ordner im Projekt** (Vorgabe `graphics`) und klickst auf **Anlegen**. Die Datei öffnet sich gleich. Der Dialog **Neue Datei** (Seitenleiste oder **Datei ▸ Neue Datei …**) legt dagegen nur Quelltexte und Notizen an. Für Kachelsätze, Bildschirme, Paletten und Songs gibt es keinen eigenen Anlegen-Dialog; du öffnest sie aus dem Projekt, sie liegen zum Beispiel schon in den Beispielprojekten.

![Der Reiter Grafik mit der Seite „Sprites“ und der Datei hero.vcsprite: links die Sprite-Dateien und zwei Bilder, in der Mitte das 12x21 Pixel große Multicolor-Sprite einer Figur in Gelb, Rotbraun und Blau, rechts die Karten „Farben“, „Sprite“ (Mehrfarbig an), „Vorschau im Spiel“ und „Verschieben“. Unter der Zeichenfläche steht der Hinweis „Keine Animation“.](bild:sprite-c64 "Der Sprite-Editor mit einem Multicolor-Sprite.")

## Der gemeinsame Aufbau der Editoren

Sprite-, Zeichensatz-, Paletten-, Kachel- und Bildschirm-Editor sehen sich bewusst ähnlich. Wenn du einen kennst, findest du dich in den anderen zurecht. Sie liegen im Hauptbereich **Grafik** und zeigen immer eine Datei. Ist sie geändert, steht in der Fußzeile „● ungespeichert“.

1. **Links** die Liste: im Sprite-Editor die Sprite-Dateien und die Bilder (Frames), im Zeichensatz-Editor die Zeichenübersicht, im Paletten-Editor das Farbraster, im Kachel-Editor der Kachelsatz, im Bildschirm-Editor die Zeichentafel.
2. **In der Mitte** die Zeichenfläche mit der **Werkzeugleiste** darüber: Zeichenwerkzeuge, Rückgängig und Wiederholen, Spiegeln, **Raster**, Zoom und, im Sprite-Editor, **Onion Skin**. Rechts in der Leiste stehen die Dateiknöpfe **Exportieren …** und **Speichern** (im Sprite- und im Zeichensatz-Editor auch **PNG laden …**). Der Kachel- und der Bildschirm-Editor ordnen ihre Werkzeuge in eigener Weise an; siehe [Kapitel 14](kapitel:bildschirm-kacheln).
3. **Rechts** die Karten mit den Pixelwerten, Farben und Eigenschaften.
4. **Fußzeile:** Name, Maße, Zellposition und Meldungen.

Im Sprite- und im Zeichensatz-Editor passt die Zeichenfläche ihren Zoom automatisch an den freien Platz an. Willst du selbst bestimmen, nutze in der Werkzeugleiste **−** und **+** oder **Cmd/Strg + Mausrad**. Die Tasten **+** und **−** wirken ebenso, **0** passt die Fläche wieder ein (Knopf **Einpassen**).

Sinnvolle Grundregeln für die Editoren:

- **Cmd/Strg+S** speichert, **Cmd/Strg+Z** macht rückgängig, **Cmd/Strg+Umschalt+Z** wiederholt.
- Mit der **rechten Maustaste** radierst du auf der Zeichenfläche (Sprite-, Zeichensatz-Editor).
- Die Farben richten sich nach dem Zielsystem. Beim Atari wählst du rechts außerdem die **Ansicht** (Projekt, PAL oder NTSC).
- Die Einzeltasten der Werkzeuge (im Sprite- und Zeichensatz-Editor D, E, I, F, L, R, S, X, Y und die Ziffern für den Pixelwert; im Bildschirm-Editor D, T, F, R, I, E) gelten, wenn die Zeichenfläche den Fokus hat. Die Tabellen stehen in den Kapiteln zum [Sprite-Editor](kapitel:sprites) und zum [Zeichensatz-Editor](kapitel:zeichensatz) und in [Tastenkürzel, FAQ, Fehlersuche](kapitel:kuerzel-faq).

## Vom Editor ins Programm: der Export

Dein Programm kann die Textdatei nicht lesen, es braucht **Bytes**. Der **Export** wandelt ein Asset um, zum Beispiel in eine Assembler-Datei mit einer Marke und `.byte`-Zeilen. Zum Einbinden brauchst du den Export nicht: Der Assembler wandelt die Datei beim `.include` selbst um.

Es gibt zwei Wege:

- **Automatisch:** Du bindest die Datei des Assets im Quelltext ein, zum Beispiel `.include "graphics/hero.vcsprite"`. Beim **Bauen** (F7) wandelt VintageCoder sie selbst um; ein Zwischenspeicher (`build/.media-cache/`) hält das Ergebnis aktuell.
- **Von Hand:** Der Knopf **Exportieren …** in der Werkzeugleiste des Editors (oder **Cmd/Strg+Umschalt+E**, auch im Menü **Datei**) öffnet den Export-Dialog.

### Der Export-Dialog

Das **Format** wählst du oben:

| Format | Datei | Wofür |
|---|---|---|
| Assembler | `<name>.asm` | Daten als `.byte`-Zeilen mit Marken und Konstanten; der Normalfall |
| Konstanten | `<name>.inc` | nur die Größen (`<name>_len`) und optional die Adresse |
| Binär | `<name>.bin` | Rohdaten, zum Beispiel zum Nachladen |
| PRG | `<name>.prg` | Rohdaten mit Ladeadresse (die gibst du im Feld **Ladeadresse** an) |
| BASIC DATA | `<name>.data.bas` | `DATA`-Zeilen für BASIC-Programme, auf Wunsch mit **Einleseroutine mitliefern** |

Im Feld **Name** steht die Marke, die als Vorsilbe aller Namen im Export dient. Die **Vorschau** zeigt den aktuellen Stand, auch ungespeicherte Änderungen, ohne etwas im Projekt anzulegen (Reiter **Text** und **Hex**). Darunter steht, wie viele Dateien und Bytes entstehen, und bei **Ziel im Quelltext** wählst du den geöffneten Quelltext, in den eingefügt werden soll. Unten stehen die Aktionen:

- **Im Emulator testen:** speichert, exportiert, baut dein Projekt und startet es wie F5.
- **In Projekt speichern:** schreibt die Dateien nach `build/export/`.
- **Als Include einfügen** (nur bei Assembler und BASIC DATA): speichert, exportiert und setzt eine `.include`-Zeile in den gewählten Quelltext.
- **In Quelltext kopieren** (nur bei Assembler und BASIC DATA): schreibt den erzeugten Text selbst an die Cursorzeile des gewählten Quelltexts, ohne Include.

![Der Dialog „Exportieren“ über dem Sprite-Editor für graphics/hero.vcsprite. Oben die Formate Assembler, Konstanten, Binär, PRG und BASIC DATA (Assembler gewählt), darunter das Feld Name mit „hero“ und der erzeugte Assembler-Text. Unten stehen die bereiten Dateien, ein gelber Hinweis unter „Ziel im Quelltext“ und die Knöpfe „Abbrechen“, „Im Emulator testen“, „In Projekt speichern“, „Als Include einfügen“ und „In Quelltext kopieren“.](bild:grafik-exportieren "Der Export-Dialog mit Format, Name, erzeugtem Text und Zielwahl.")

> **Tipp:** Brauchst du nur die Daten im Programm, lass den automatischen Export arbeiten. Der Dialog lohnt sich, wenn du ein anderes Format brauchst oder die Bytes einmal ansehen möchtest.

## Beispiel zum Nachmachen: ein Sprite auf den Bildschirm

Das Projekt `examples/c64/sprite_demo` enthält ein mehrfarbiges Sprite `graphics/hero.vcsprite` und dieses Programm:

```asm
.target c64
.cpu 6510 { illegals = deny }

.org $0801
.basicstub start

start:  sei
        lda #$80                ; Sprite-Zeiger 0: Daten bei $2000 ($2000 / 64 = $80)
        sta $07f8
        lda #1
        sta vic2.spena          ; Sprite 0 an
        sta vic2.spmc           ; Multicolor
        lda #100
        sta vic2.sp0x
        sta vic2.sp0y
        lda #7
        sta vic2.sp0col
        lda #2
        sta vic2.spmc0
        lda #6
        sta vic2.spmc1
loop:   jmp loop

.org $2000
.include "graphics/hero.vcsprite"
```

1. Öffne das Projekt `examples/c64/sprite_demo` (auf der Startseite bei den Beispielen).
2. Drücke **F7** (Projekt bauen). Dabei wandelt VintageCoder `graphics/hero.vcsprite` beim `.include` automatisch um.
3. Drücke **F5** (Startprogramm starten). Im Emulator erscheint der kleine Held.

Die letzten beiden Zeilen sind der ganze „Einbau“: `.org $2000` legt die Adresse fest, `.include` holt die exportierten Bytes dorthin. Wie das Sprite entsteht, lernst du in [Kapitel 13](kapitel:sprites).

## Häufige Stolpersteine

- **„Label nicht gefunden“ beim Bauen:** Der Name nach dem Export (`hero`, `font`, …) richtet sich nach dem Namen im Asset, nicht nach dem Dateinamen. Im Export-Dialog siehst du den Namen im Feld **Name**.
- **Änderungen erscheinen nicht im Programm:** Dann wurde nach deiner Änderung nicht neu exportiert. Baue mit F7 oder nutze **Exportieren …**. Speichere vorher mit Cmd/Strg+S.
- **Ein Editor zeigt „Keine … geöffnet“:** Er braucht ein geöffnetes Projekt und eine Datei des passenden Typs. Wähle eine Datei aus der Liste oder öffne sie mit einem Doppelklick im Projektbaum.

## Wie geht es weiter?

Als Nächstes lernst du den einfachsten Editor kennen: den [Zeichensatz-Editor](kapitel:zeichensatz). Ausführliche Nachschlagetexte: [Editoren im Überblick](hilfe:graphics) und [Export](hilfe:export).
