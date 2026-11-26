# SPDX-License-Identifier: GPL-3.0-or-later
# Erklärtexte zu den Diagnosecodes B150 bis B196 des BASIC-Teils: Sie erscheinen im Problem-Panel beim Überfahren und Anklicken des Codes.
# Reiner Text (kurz, ein bis zwei Sätze); Befehle und Direktiven in Rückwärtsstrichen. Die Art (Fehler, Warnung, Hinweis) und die Abschaltbarkeit hängen die Karte in Rust an.

# B150: Erklärung
basiccode-b150 = Ein Schlüsselwort steckt mitten in einem Variablennamen (zum Beispiel `TOTAL` enthält `TO`). Der Interpreter zerlegt den Namen dort. Wähle einen Namen ohne Schlüsselwort.
# B151: Erklärung
basiccode-b151 = Zu einem `FOR` fehlt das `NEXT` oder umgekehrt. Schleifen müssen paarweise stehen und dürfen sich nur ineinander schachteln, nicht überkreuzen.
# B152: Erklärung
basiccode-b152 = Es gibt ein `GOSUB`, aber im ganzen Programm kein `RETURN` (oder umgekehrt). Unterprogramme brauchen beides.
# B153: Erklärung
basiccode-b153 = Ein Sprung geht zu einer Zeilennummer, die es nicht gibt. Der Fehler tritt erst beim Ablauf auf (`UNDEF'D STATEMENT`). Besser mit Marken (`@name`) springen.
# B154: Erklärung
basiccode-b154 = Zwei Namen sind für den Interpreter dieselbe Variable, weil nur die ersten Zeichen zählen (bei Commodore zwei). Die Werte überschreiben sich gegenseitig.
# B155: Erklärung
basiccode-b155 = Eine Zeichenkette ist am Zeilenende nicht geschlossen. Das ist erlaubt und bleibt offen bis zum Zeilenende, ist aber oft ein vergessenes Anführungszeichen.
# B156: Erklärung
basiccode-b156 = Ein Schlüsselwort bekommt mehr oder weniger Argumente, als der Dialekt erlaubt. Die erwartete Zahl steht in der Meldung.
# B157: Erklärung
basiccode-b157 = Das Wort ist ein Schlüsselwort eines anderen Dialekts und gehört nicht zum gewählten. Der Interpreter würde es als Variable lesen oder mit Syntaxfehler abbrechen.
# B158: Erklärung
basiccode-b158 = Nach `THEN` steht ein Doppelpunkt. In Commodore-BASIC gehört alles hinter dem `THEN` zur Bedingung; der Doppelpunkt ist meist überflüssig.
# B159: Erklärung
basiccode-b159 = Eine Funktion braucht ihr Argument in Klammern, hier fehlen sie.
# B160: Erklärung
basiccode-b160 = Die Dateien des Programms mischen Zeilennummern und Marken. Beim Bauen ist das möglich, aber schwer lesbar; wandle mit „In Labels umwandeln“ oder „In Nummern umwandeln“ einheitlich um.
# B161: Erklärung
basiccode-b161 = Die Umwandlung oder das Neu-Nummerieren ist so nicht möglich (zum Beispiel berechnete Sprungziele oder ein Bereich, der die Reihenfolge ändern würde). Es wurde nichts geändert.
# B162: Erklärung
basiccode-b162 = Das Sprungziel ist berechnet (zum Beispiel `GOTO A*10`). Beim Umwandeln lässt sich nicht erkennen, welche Zeile gemeint ist; diese Stelle musst du selbst anpassen.
# B163: Erklärung
basiccode-b163 = Das Programm liest oder schreibt Zeilennummern zur Laufzeit (zum Beispiel `RESTORE`, `ERL`). Nach dem Umwandeln oder Neu-Nummerieren stimmen diese Nummern nicht mehr.
# B164: Erklärung
basiccode-b164 = Hinweis: Diese Zahl steht an einer Stelle, wo eine Zeilennummer stehen könnte. Beim Umwandeln wird sie als Zeilennummer behandelt.
# B165: Erklärung
basiccode-b165 = Eine Markerzeile `;@name` steht ohne folgende Anweisung. Der Marker hat dann kein Ziel.
# B166: Erklärung
basiccode-b166 = Eine eingebundene Datei hat einen anderen Modus (Nummern oder Marken) als die übrigen Dateien. Das Umwandeln kann so nicht einheitlich arbeiten.
# B167: Erklärung
basiccode-b167 = In diesem BASIC ist das Hochkomma kein Kommentarzeichen. Der Rest der Zeile wird als Programm gelesen. Kommentare schreibt man mit `REM`.
# B168: Erklärung
basiccode-b168 = Eine Markerzeile (`;@name`) ist ungültig oder doppelt vergeben. Marken müssen im Programm eindeutig sein.
# B170: Erklärung
basiccode-b170 = Der gewählte Dialekt ist für das Zielsystem des Projekts nicht vorgesehen. Der Bau läuft, das Ergebnis passt vielleicht nicht zur Maschine.
# B171: Erklärung
basiccode-b171 = Ein benötigter Programmteil des Dialekts fehlt (zum Beispiel eine Erweiterung). Der Bau kann nicht fortfahren.
# B172: Erklärung
basiccode-b172 = Der Name für `#define` ist ein Schlüsselwort. Das würde jedes Vorkommen des Worts im Programm ersetzen; wähle einen anderen Namen.
# B173: Erklärung
basiccode-b173 = Die Zeilennummern reichen nicht für die DATA-Zeilen des Maschinencodes. Verkleinere die Schrittweite oder fasse das Programm kürzer.
# B174: Erklärung
basiccode-b174 = Die Zeile mit `#define` ist falsch aufgebaut oder der Name ist schon vergeben. Erwartet wird `#define NAME wert`; der Name besteht aus Buchstaben, Ziffern und `_`.
# B175: Erklärung
basiccode-b175 = Der Aufbau von `#if`, `#else`, `#endif` stimmt nicht: Es fehlt ein Teil, er steht doppelt oder die Bedingung ist nicht lesbar.
# B176: Erklärung
basiccode-b176 = Ein `#define` verweist auf sich selbst oder in zu vielen Stufen auf andere. Das Ersetzen würde nie enden.
# B177: Erklärung
basiccode-b177 = Beim Einbinden von Maschinencode mit `#asm` ist etwas schiefgegangen (Syntax, fehlende Marke, zu groß, Adresse nicht stabil). Die Meldung nennt den Grund.
# B178: Erklärung
basiccode-b178 = Der Dialekt in `#dialect` oder in den Projekteinstellungen ist unbekannt. Die Meldung nennt die bekannten Kennungen.
# B180: Erklärung
basiccode-b180 = Eine Anweisung ist im Atari-BASIC an dieser Stelle falsch aufgebaut (zum Beispiel fehlt `THEN`, `TO` oder ein Gleichheitszeichen).
# B181: Erklärung
basiccode-b181 = Ein Ausdruck im Atari-BASIC ist unvollständig oder die Klammern passen nicht.
# B182: Erklärung
basiccode-b182 = Ein Name ist im Atari-BASIC ungültig: kein erlaubter Labelname, ein reserviertes Wort oder es fehlt ein Name.
# B183: Erklärung
basiccode-b183 = Eine Zahl ist im Atari-BASIC ungültig oder außerhalb des Bereichs (größte Zahl etwa 9,99E97).
# B184: Erklärung
basiccode-b184 = Ein Text enthält ein Zeichen, das es im ATASCII nicht gibt, oder ist länger als 255 Zeichen. Mit `{ "{" }{ "$" }xx{ "}" }` schreibst du ein Zeichen als Code.
# B185: Erklärung
basiccode-b185 = Ein Grenzwert von Atari-BASIC ist überschritten: zu viele Variablen, leere Zeile, Zeile zu lang oder Zeilennummer über 32767.
# B190: Erklärung
basiccode-b190 = Eine Zeile `#numbering` ist falsch aufgebaut. Erwartet wird `#numbering start=100 step=5`.
# B191: Erklärung
basiccode-b191 = Start oder Schrittweite der Zeilennummern sind ungültig: Die Schrittweite muss mindestens 1 sein, der Start darf den größten Wert des Systems nicht überschreiten.
# B192: Erklärung
basiccode-b192 = Bei dieser Schrittweite reichen die Zeilennummern für das Programm nicht aus. Kleinere Schrittweite oder weniger Zeilen wählen.
# B193: Erklärung
basiccode-b193 = Ein eingebundenes Medium (Sprite, Zeichensatz, Song …) konnte nicht gelesen oder umgewandelt werden. Die Meldung nennt den Grund.
# B194: Erklärung
basiccode-b194 = Der Pfad unter `build/export/` ist veraltet. Binde das Medium direkt ein, zum Beispiel `#include "graphics/hero.vcsprite"`; der Bau erzeugt die DATA-Zeilen selbst.
# B195: Erklärung
basiccode-b195 = Das Programm ist größer als der freie Speicher des Zielsystems. Kürzen, `#asm` für Maschinencode nutzen oder die Option „Kompakt bauen“ einschalten.
# B196: Erklärung
basiccode-b196 = Eingebundene Medien liegen als DATA-Zeilen vor, aber der Dialekt kennt kein `RESTORE` auf eine Zeile. `READ` liest streng der Reihe nach; die Reihenfolge der Medien ist wichtig.
# Kartenzeilen unter dem Erklärtext im Problem-Panel | $code: Kennung | Beispiel: B152
basiccode-anzeigen = Erklärung zu { $code } anzeigen
basiccode-abschaltbar = Lässt sich in den Projekteinstellungen abschalten (BASIC, „Abgeschaltete Meldungen“).
basiccode-fest = Ein Fehler lässt sich nicht abschalten.
