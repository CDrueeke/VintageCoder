# Startseite, Konfigurationsassistent und neues Projekt

## Startseite
Ist kein Projekt offen, zeigt der Hauptbereich **Code** die **Startseite**. Mit dem Befehl **Startseite** (Menü **Datei**, auch in der Befehlspalette) holst du sie jederzeit zurück; das schließt das Projekt (bei Ungespeichertem fragt VintageCoder vorher).

![Die Startseite ohne offenes Projekt: der Titel VintageCoder, die Karten Neues Projekt, Projekt öffnen und Beispiel öffnen, der Link Projektdatei (.vcprj) wählen, der Abschnitt Zuletzt geöffnete Projekte (noch leer) und die Beispielprojekte mit Filtern je System und Karten mit Öffnen und Kopieren.](bild:willkommen-seite "Die Startseite erscheint, solange kein Projekt offen ist.")

Oben stehen Logo, Name und Versionsnummer. Darunter folgen:
- die Karten **Neues Projekt**, **Projekt öffnen** (Cmd/Strg+Umschalt+O) und **Beispiel öffnen** sowie der Link **Projektdatei (.vcprj) wählen …**,
- die Liste **Zuletzt geöffnete Projekte** (angeheftete zuerst; die Startseite zeigt höchstens fünf, **Weitere …** klappt den Rest auf). Ein Klick öffnet das Projekt, der Stern heftet es oben an, das Menü **…** bietet **Öffnen**, **Kopieren …**, **Im Ordner zeigen** und **Aus der Liste entfernen**. Fehlt der Ordner, steht „Ordner fehlt“,
- die **Beispielprojekte**: eine Auswahl empfohlener Beispiele aus dem Ordner `examples/`, mit Filter je System und jeweils **Öffnen** und **Kopieren …**. **Alle Beispiele …** öffnet einen Dialog mit Suchfeld.

Unten stehen zwei Kästen. Der Kasten **Toolchain und Hardware** zeigt die **erkannten Emulatoren** („bereit“ oder „nicht gefunden“); die Schnellreferenz daneben nennt die wichtigsten Tastenkürzel. Solange du VintageCoder noch nicht eingerichtet hast oder die Einrichtung unvollständig ist, steht an der Stelle des Kastens **Toolchain und Hardware** die Karte **Willkommen bei VintageCoder** mit den Knöpfen **Einrichtung starten** und **Später** (**Später** blendet die Karte aus). Hat VintageCoder seit der letzten Einrichtung neue Emulatoren gefunden, erscheint die Karte **Neues erkannt** mit **Emulatoren einrichten**, **Als bekannt merken** und **Nicht mehr zeigen**.

## Konfigurationsassistent
Der Konfigurationsassistent richtet Zielsysteme, Emulatoren, ROMs, Hardware und Aussehen ein. Er öffnet sich beim allerersten Start von selbst (außer die Umgebungsvariable `VC_NO_WIZARD=1` ist gesetzt) und lässt sich jederzeit wiederholen: über **Einrichtung starten** auf der Startseite (solange die Karte **Willkommen bei VintageCoder** zu sehen ist), im Menü **Hilfe ▸ Konfigurationsassistent …**, in **Einstellungen ▸ Allgemein ▸ Einrichtung** oder über die Befehlspalette (Cmd/Strg+K, Wort „Konfigurationsassistent“).

![Erster Schritt „1. Willkommen“ des Einrichtungsassistenten: links die Schrittleiste, rechts die Sprachwahl „Automatisch“, „Deutsch“, „English“ und die Wahl zwischen „Schnelleinrichtung (7 Schritte)“ (gewählt) und „Alles selbst einstellen (12 Schritte)“.](bild:assistent-weg-waehlen "Der Willkommen-Schritt: Sprache wählen und zwischen Schnelleinrichtung und „Alles selbst einstellen“ entscheiden.")

Er arbeitet nach drei Regeln:
- **Er installiert nichts und lädt nichts herunter.** Er zeigt nur, was auf deinem Rechner gefunden wurde. Im Schritt **Emulatoren** zeigt er dieselbe Seite wie die Einstellungen; ein Emulator wird nur nach deinem Klick auf **Jetzt laden** portabel geladen.
- **Er arbeitet auf einem Entwurf.** Geschrieben wird erst bei **Übernehmen** in der **Zusammenfassung**, und vorher entsteht eine Sicherung der Einstellungen. Nur das **Design** (Farbschema, Zoom, Schrift) wirkt sofort; **Abbrechen** stellt das Alte wieder her.
- **Du kannst jederzeit aufhören.** **Abbrechen** beendet den Assistenten ohne Änderung. **Später fortsetzen** tut dasselbe und merkt sich den Schritt; die Startseite erinnert dann an die offene Einrichtung.

Links steht die Liste der **Schritte**, unten **Abbrechen**, **Später fortsetzen**, **Zurück** und **Weiter**. Der erste Schritt, **Willkommen**, hat die **Sprache** (*Automatisch*, *Deutsch*, *English*) und zwei Wege:
- **Schnelleinrichtung (7 Schritte):** Willkommen, Design, Zielsysteme, Toolchain, Projekte, Zusammenfassung, Fertig. Emulatoren, ROMs und Hardware werden automatisch gefunden und später in den Einstellungen verfeinert.
- **Alles selbst einstellen (12 Schritte):** zusätzlich **Emulatoren**, **ROMs**, **Echte Hardware**, **Startziel F5** und **Editor**.

Die Teil-Assistenten richten nur einen Teil ein: **Konfigurationsassistent: nur ROMs …**, **nur Emulatoren …**, **nur echte Hardware …** und **nur Design und Editor …** (Menü **Hilfe**, Befehlspalette, **Einstellungen ▸ Allgemein ▸ Einrichtung**). **Konfigurationsassistent: Nach Neuem suchen …** öffnet den ROM-Teil.

Die Schritte:
- **Design:** Farbschema (*System*, *Dunkel*, *Hell*), Zoom und Editor-Schrift.
- **Zielsysteme:** Für welche Systeme willst du programmieren? Abgewählte Systeme verschwinden aus der Oberfläche, ihre Einträge bleiben erhalten.
- **Toolchain:** nennt den eingebauten Assembler und zeigt die Seite der Emulatoren.
- **Emulatoren, Echte Hardware, Startziel F5:** dieselben Karten wie unter **Einstellungen ▸ Ziele und Hardware** (siehe [Einstellungen](help:settings)). Maschinen sind freiwillig, Verbindungen werden nur auf Klick getestet.
- **ROMs:** dieselbe Seite wie **Einstellungen ▸ ROMs**, siehe [ROM-Dateien](help:roms).
- **Editor:** Schrift, Zeilenumbruch, Zeile beim Verlassen formatieren.
- **Projekte:** der **Projektordner** und, wenn Beispiele vorhanden sind, der Schalter **Beispielprojekte der gewählten Systeme kopieren** mit der Liste der Beispiele (einzeln abwählbar). Vorhandene Dateien bleiben unberührt.
- **Zusammenfassung:** jede Änderung mit Haken, Bereich, Art (+ neu, ~ geändert, − entfernt) sowie altem und neuem Wert; ein abgewählter Haken heißt „nicht übernehmen“. **Übernehmen** schreibt. Danach zeigt der Schritt den Ort der Sicherung, die Zahl der kopierten Beispiele und den Knopf **Rückgängig**. Gibt es nichts zu ändern, steht das dort.
- **Fertig:** meldet, dass die Einrichtung fertig ist. Unter **Erstes Projekt** öffnest du mit **Neues Projekt anlegen …** gleich den Dialog **Neues Projekt erstellen** oder mit **Beispiel öffnen: …** ein kopiertes Beispiel. **Schließen** beendet den Assistenten.

Der Assistent baut und startet nichts von selbst. Zur Konfiguration gehören nicht: Tastenkürzel (sie sind fest), die Passwortablage im Schlüsselbund (fehlt noch). Das Sichern und Zurücksetzen der Einstellungen steht unter **Einstellungen ▸ Sicherung**, siehe [Konfiguration sichern](help:config).

## Neues Projekt
**Datei ▸ Neues Projekt …** (Cmd/Strg+Alt+N), der Knopf auf der Startseite oder `vcc new` legt ein Projekt an: Projektdatei `.vcprj`, Quelldateien aus einer Vorlage und den Ordner `src/<system>/`. Der Dialog **Neues Projekt erstellen** hat drei Spalten:

1. **Zielsystem:** C64, C128, C16/Plus/4, VC-20, Atari 800 XL/XE oder MEGA65, dazu das **Hardware-Profil** (CPU und Chips).
2. **Vorlage:** zuerst die Sprache, **Assembler** oder **BASIC**, dann die Vorlage. Gibt es für die Kombination keine, steht das da. BASIC wird im Dialekt des Zielsystems geschrieben, ohne Zeilennummern und mit Sprungmarken (`@name`); beim Bauen entsteht das native Programm.
3. **Projekt-Details:** **Projektname** (Buchstaben, Ziffern, `_` und `-`), **Speicherort** (mit **Wählen …**), **BASIC-Dialekt** (wo es mehrere gibt), **Videonorm** PAL oder NTSC (nicht beim MEGA65), beim VC-20 der **Speicherausbau**, der **Standard-Emulator** und das **Startziel (F5)** (vorbelegt aus den Einstellungen; eine Abweichung wird in der Projektdatei gespeichert) und die **Diskette**: keine oder ein Format, das zum System passt (C64 D64, C128 D64, D71 oder D81, Atari ATR, MEGA65 D81). Der **Zeichensatz des Projekts** ist *Automatisch* (dein ROM hat Vorrang, sonst der eigene Satz VC-Standard als Kopie), *Nur Standard (eigen)* oder *Keinen*. Dazu zwei Schalter: **README.md mit Bauhinweisen anlegen** und **Git-Repository anlegen** (mit `.gitignore` für `build/`). Unter **Weitere Zielsysteme** lassen sich zusätzliche Systeme der Vorlage anhaken; die **Dateistruktur** zeigt, was angelegt wird.

![Der Dialog „Neues Projekt erstellen“ über der Startseite. Links die Liste „ZIELSYSTEM“ mit Commodore 64 (gewählt) bis MEGA65 und dem Hardware-Profil „gültig“, in der Mitte die „VORLAGE“ mit „Assembler-Programm“ (gewählt), „Steckmodul“ und „Leeres Projekt“, rechts die „PROJEKT-DETAILS“ mit Projektname, Speicherort, Videonorm, Standard-Emulator, Startziel, Diskette und Zeichensatz. Unten die Knöpfe „Abbrechen“ und „Projekt erstellen und öffnen“.](bild:neues-projekt-dialog "Der Dialog „Neues Projekt erstellen“ mit Zielsystem, Vorlage und Projekt-Details.")

**Projekt erstellen und öffnen** legt alles an und öffnet das Projekt im Bereich **Code**. Ist der Knopf grau, nennt die Spalte den Grund (zum Beispiel ein unerlaubtes Zeichen im Namen). Siehe auch das Handbuchkapitel [Dein erstes Projekt](kapitel:erstes-projekt).

## Projektstruktur
Zusätzliche Ordner (*Einfach*, *Standard*, *Groß, modular*, *Mehrere Systeme*, *Bibliothek*, *Spiel mit Levels*) wählst du zurzeit **auf der Kommandozeile** mit `vcc new … --structure <id>` (`vcc new --list` nennt die Strukturen). Der Dialog **Neues Projekt erstellen** hat dafür noch keine Auswahl; er legt nur den Ordner `src/<system>/` an. Die Struktur legt die Ordner an und trägt im Abschnitt `layout` der Projektdatei ein, wohin abgelegte Dateien je Kategorie kommen. Bei *Mehrere Systeme* entsteht je gewähltem Zielsystem ein Ordner in `src/` und `res/`. Mit angehaktem Git-Repository bekommen neue leere Ordner eine `.gitkeep`. Die mitgelieferten Vorlagen stehen in `data/templates/structures.json`, eigene in `structures.json` im Konfigurationsordner.

## Kommandozeile und Vorlagen
- Systeme: `c64`, `c128`, `plus4`, `vic20`, `atari8`, `mega65` (auch mehrere, kommagetrennt).
- Vorlagen: `asm-hello`, `basic-hello`, `empty`. `asm-hello` gibt es zusätzlich als Variante **Steckmodul** (`--variant cartridge`; C64, Atari, VC-20): gleiche Projektdatei, der Startquelltext enthält `.cartridge`, die Adresse des Modulbereichs und die Kennung des Systems. `--template cartridge` bleibt als Kurzform erhalten.
- Ordner: relativ zum geöffneten Ordner; `.` nur, wenn dort noch kein Projekt liegt.
- Grafik und Klang: die Editoren bieten nur Profile der Zielsysteme des Projekts an; Sprites, Zeichensätze und Songs anderer Systeme lassen sich nicht öffnen oder anlegen.

        vcc new meinspiel --target c64,mega65 --template asm-hello
        vcc new meinmodul --target c64 --template asm-hello --variant cartridge
        vcc new --list

Eigene Vorlagen: Ordner und Eintrag in `data/templates/index.json` (`{system}` im Pfad, `{{name}}` im Text). Eine Vorlage kann unter `variants` Varianten mit eigenen Systemen, Dateien und Quellen haben; sie ersetzen die Dateien der Vorlage.

Konstanten für den Assembler stehen im Target der Projektdatei (wie `vcc asm -D NAME=Zahl`), zum Beispiel `"defines": { "DEBUG": 1 }` für `.if DEBUG`; `true` und `false` zählen als 1 und 0. Sie gelten beim Bauen und in den Live-Fehlern des Editors.

Mit `--structure <id>` (`vcc new --list` nennt die Strukturen) legt `vcc new` dieselben Ordner und das `layout` an, z. B. `vcc new spiel --target c64 --structure large`.
